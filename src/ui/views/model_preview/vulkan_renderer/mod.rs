//! 3D Vulkan GPU 硬件加速渲染引擎 (Vulkano 0.35 原生硬件驱动)

pub mod context;
pub mod geometry;
pub mod pipeline;
pub mod shaders;
pub mod target;
pub mod textures;
pub mod types;

use context::{get_or_init_vulkan, VulkanContext};
use geometry::GpuGeometryCache;
use gpui::RenderImage;
use image::{Frame, ImageBuffer, Rgba};
use pipeline::VulkanPipeline;
use std::sync::{Arc, Mutex};
use target::RenderTargetBundle;
use textures::GpuTextureManager;
use types::{GpuUniforms, Mat4, VulkanCacheKey};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo, PrimaryCommandBufferAbstract,
    RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::sync::GpuFuture;

use crate::model::model_preview::ModelDocumentState;
use crate::ui::views::model_preview::gizmos::get_material_color;

struct VulkanEngineState {
    pipeline: VulkanPipeline,
    textures: GpuTextureManager,
    geometries: GpuGeometryCache,
    target: Option<RenderTargetBundle>,
    cached_frame: Option<(VulkanCacheKey, Arc<RenderImage>)>,
}

static ENGINE_STATE: Mutex<Option<VulkanEngineState>> = Mutex::new(None);

/// 渲染 3D 场景并输出高质量 Vulkan 硬件渲染图像
pub fn render_vulkan_scene(
    state: &ModelDocumentState,
    view_w: f32,
    view_h: f32,
) -> Option<Arc<RenderImage>> {
    let target_w = (view_w.round() as u32).clamp(16, 3840);
    let target_h = (view_h.round() as u32).clamp(16, 2160);

    let cache_key = compute_cache_key(state, target_w, target_h);

    // 1. 静态画面缓存检查 (静止时瞬时返回，零 GPU 开销)
    if let Ok(guard) = ENGINE_STATE.lock() {
        if let Some(ref eng) = *guard {
            if let Some((ref key, ref img)) = eng.cached_frame {
                if key == &cache_key {
                    return Some(img.clone());
                }
            }
        }
    }

    let ctx = get_or_init_vulkan()?;

    let mut guard = ENGINE_STATE.lock().ok()?;
    if guard.is_none() {
        let pipeline = VulkanPipeline::new(ctx.device.clone())?;
        let textures = GpuTextureManager::new(&ctx)?;
        let geometries = GpuGeometryCache::new();
        *guard = Some(VulkanEngineState {
            pipeline,
            textures,
            geometries,
            target: None,
            cached_frame: None,
        });
    }

    let engine = guard.as_mut()?;
    let render_image = render_scene_pass(&ctx, engine, state, target_w, target_h)?;

    engine.cached_frame = Some((cache_key, render_image.clone()));
    Some(render_image)
}

/// 执行单帧 Vulkan 渲染 Pass 与显存读回
fn render_scene_pass(
    ctx: &VulkanContext,
    engine: &mut VulkanEngineState,
    state: &ModelDocumentState,
    target_w: u32,
    target_h: u32,
) -> Option<Arc<RenderImage>> {
    RenderTargetBundle::ensure(
        &mut engine.target,
        ctx,
        engine.pipeline.render_pass.clone(),
        target_w,
        target_h,
    )?;
    let (framebuffer, color_image, staging_buffer) = {
        let t = engine.target.as_ref()?;
        (t.framebuffer.clone(), t.color_image.clone(), t.staging_buffer.clone())
    };

    let aspect = target_w as f32 / target_h as f32;
    let proj = Mat4::perspective_rh(state.camera.fov, aspect, 0.05, 50000.0);
    let (_right, up, _forward) = state.camera.compute_basis();
    let eye = state.camera.compute_eye();
    let view = Mat4::look_at_rh(eye, state.camera.target, up);
    let mvp = proj.mul(&view);

    let sun_norm = normalize_sun([0.35, 0.80, 0.45]);
    let projector = state.camera.projector(target_w as f32, target_h as f32);

    let mut builder = AutoCommandBufferBuilder::primary(
        ctx.command_buffer_allocator.clone(),
        ctx.queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .ok()?;

    builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    Some([0.0f32, 0.0, 0.0, 0.0].into()),
                    Some(1.0f32.into()),
                ],
                ..RenderPassBeginInfo::framebuffer(framebuffer)
            },
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            },
        )
        .ok()?;

    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [target_w as f32, target_h as f32],
        depth_range: 0.0..=1.0,
    };
    builder.set_viewport(0, [viewport].into_iter().collect()).ok()?;
    builder.bind_pipeline_graphics(engine.pipeline.pipeline.clone()).ok()?;

    draw_scene_meshes(
        ctx,
        &mut builder,
        engine,
        state,
        &projector,
        &mvp,
        sun_norm,
    )?;

    builder.end_render_pass(Default::default()).ok()?;

    builder
        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
            color_image,
            staging_buffer.clone(),
        ))
        .ok()?;

    let command_buffer = builder.build().ok()?;
    command_buffer
        .execute(ctx.queue.clone())
        .ok()?
        .then_signal_fence_and_flush()
        .ok()?
        .wait(None)
        .ok()?;

    let read_guard = staging_buffer.read().ok()?;
    let img_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
        target_w,
        target_h,
        read_guard.to_vec(),
    )?;
    Some(Arc::new(RenderImage::new(vec![Frame::new(img_buffer)])))
}

/// 遍历绘制模型中所有可见子网格
fn draw_scene_meshes(
    ctx: &VulkanContext,
    builder: &mut AutoCommandBufferBuilder<vulkano::command_buffer::PrimaryAutoCommandBuffer>,
    engine: &mut VulkanEngineState,
    state: &ModelDocumentState,
    projector: &crate::model::model_preview::FastCameraProjector,
    mvp: &Mat4,
    sun_norm: [f32; 4],
) -> Option<()> {
    for (mesh_idx, mesh) in state.meshes.iter().enumerate() {
        if state.mesh_items.get(mesh_idx).is_none_or(|item| !item.visible) {
            continue;
        }

        let b_min = [mesh.bounding_box.min().x, mesh.bounding_box.min().y, mesh.bounding_box.min().z];
        let b_max = [mesh.bounding_box.max().x, mesh.bounding_box.max().y, mesh.bounding_box.max().z];
        if projector.is_all_behind_camera(b_min, b_max) {
            continue;
        }

        let is_selected = state.selected_mesh_idx == Some(mesh_idx);
        let base_hsla = get_material_color(
            mesh.material_name.as_deref().unwrap_or(""),
            mesh_idx,
            is_selected,
        );
        let base_rgba = hsla_to_vec4(base_hsla);

        let (tex_view, has_texture) = resolve_mesh_texture(ctx, engine, state, mesh);
        let geom = engine.geometries.get_or_upload(ctx, &state.file_path, mesh_idx, mesh)?;

        let uniforms = GpuUniforms {
            mvp: mvp.to_cols_array_2d(),
            sun_dir: sun_norm,
            base_color: base_rgba,
            flags: [
                if has_texture { 1 } else { 0 },
                if mesh.is_alpha_cutout() { 1 } else { 0 },
                if is_selected { 1 } else { 0 },
                0,
            ],
        };

        let ubo = Buffer::from_data(
            ctx.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            uniforms,
        )
        .ok()?;

        let set_layout = engine.pipeline.pipeline.layout().set_layouts().first()?.clone();
        let descriptor_set = DescriptorSet::new(
            ctx.descriptor_set_allocator.clone(),
            set_layout,
            [
                WriteDescriptorSet::buffer(0, ubo),
                WriteDescriptorSet::image_view(1, tex_view),
                WriteDescriptorSet::sampler(2, engine.pipeline.sampler.clone()),
            ],
            [],
        )
        .ok()?;

        builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                engine.pipeline.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .ok()?;

        builder.bind_vertex_buffers(0, geom.vertex_buffer.clone()).ok()?;
        builder.bind_index_buffer(geom.index_buffer.clone()).ok()?;
        // SAFETY: 索引缓冲与顶点缓冲格式已预校验
        unsafe {
            builder.draw_indexed(geom.index_count, 1, 0, 0, 0).ok()?;
        }
    }

    Some(())
}

/// 解析获取网格漫反射贴图视图
fn resolve_mesh_texture(
    ctx: &VulkanContext,
    engine: &mut VulkanEngineState,
    state: &ModelDocumentState,
    mesh: &rbfres_library::ExtractedMesh,
) -> (Arc<ImageView>, bool) {
    if let Some(name) = mesh.texture_name.as_deref() {
        if let Some(tex) = state.textures.get(name) {
            if let Some(view) = engine.textures.get_or_upload(ctx, name, tex) {
                return (view, true);
            }
        }
    }
    for ref_name in &mesh.texture_refs {
        if let Some(tex) = state.textures.get(ref_name) {
            if let Some(view) = engine.textures.get_or_upload(ctx, ref_name, tex) {
                return (view, true);
            }
        }
    }
    (engine.textures.default_white.clone(), false)
}

fn compute_cache_key(state: &ModelDocumentState, target_w: u32, target_h: u32) -> VulkanCacheKey {
    let mut vis_hash = 0u64;
    for (i, item) in state.mesh_items.iter().enumerate() {
        if item.visible {
            vis_hash = vis_hash.wrapping_add(1u64 << (i % 64));
        }
    }

    VulkanCacheKey {
        model_path: state.file_path.clone(),
        yaw_bits: state.camera.yaw.to_bits(),
        pitch_bits: state.camera.pitch.to_bits(),
        distance_bits: state.camera.distance.to_bits(),
        target_bits: [
            state.camera.target[0].to_bits(),
            state.camera.target[1].to_bits(),
            state.camera.target[2].to_bits(),
        ],
        fov_bits: state.camera.fov.to_bits(),
        view_w: target_w,
        view_h: target_h,
        render_mode: state.render_mode,
        selected_mesh_idx: state.selected_mesh_idx,
        visibility_hash: vis_hash,
    }
}

fn hsla_to_vec4(hsla: gpui::Hsla) -> [f32; 4] {
    let rgba = hsla.to_rgb();
    [rgba.r, rgba.g, rgba.b, rgba.a]
}

fn normalize_sun(dir: [f32; 3]) -> [f32; 4] {
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(0.001);
    [dir[0] / len, dir[1] / len, dir[2] / len, 0.0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use rbfres_library::BfresFile;

    #[test]
    fn test_vulkan_render_dungeon085() {
        let pack_path = r"C:\Game\BOTW\01007EF00011E000\romfs\Pack\Dungeon085.pack";
        if !Path::new(pack_path).exists() {
            return;
        }
        let virtual_path = format!("SARC:{}//Model/DgnMrgPrt_Dungeon085.sbfres", pack_path);
        let Ok(bytes) = crate::model::file_tree::read_file_from_path(&virtual_path) else { return; };
        let decomp = crate::model::file_converter::decompress_if_yaz0(&bytes);
        let Ok(bfres) = BfresFile::from_bytes(&decomp) else { return; };
        let mut state = ModelDocumentState::from_bfres(&virtual_path, "DgnMrgPrt_Dungeon085.sbfres", &bfres);
        crate::model::model_preview::load_companion_textures(&virtual_path, &bfres, &mut state);

        let start = std::time::Instant::now();
        let image = render_vulkan_scene(&state, 1280.0, 720.0);
        let elapsed = start.elapsed();
        println!(">>> 200,000 Triangles 1st Frame (Cold Init + Upload) Time: {:?}", elapsed);

        let start2 = std::time::Instant::now();
        let image2 = render_vulkan_scene(&state, 1280.0, 720.0);
        let elapsed2 = start2.elapsed();
        println!(">>> 2nd Frame (GPU Frame Cached) Time: {:?}", elapsed2);

        // 测试相机微旋转 (强制 GPU 重新渲染 200,000 个面)
        let mut rotated_state = state.clone();
        rotated_state.camera.yaw += 0.05;
        let start3 = std::time::Instant::now();
        let image3 = render_vulkan_scene(&rotated_state, 1280.0, 720.0);
        let elapsed3 = start3.elapsed();
        println!(">>> 3rd Frame (Camera Rotating, Full 200,000 Triangles GPU Render) Time: {:?}", elapsed3);

        assert!(image.is_some() && image2.is_some() && image3.is_some());
    }
}
