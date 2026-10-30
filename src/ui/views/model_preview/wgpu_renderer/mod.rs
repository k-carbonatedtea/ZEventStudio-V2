//! 3D WGPU 硬件加速渲染引擎 (跨平台 Vulkan / DirectX 12 / Metal 原生驱动)

pub mod context;
pub mod geometry;
pub mod pipeline;
pub mod shaders;
pub mod target;
pub mod textures;
pub mod types;

use context::{get_or_init_wgpu, WgpuContext};
use geometry::GpuGeometryCache;
use gpui::RenderImage;
use image::{Frame, ImageBuffer, Rgba};
use pipeline::WgpuPipeline;
use std::sync::{Arc, Mutex};
use target::RenderTargetBundle;
use textures::GpuTextureManager;
use types::{GpuUniforms, Mat4, WgpuCacheKey};

use crate::model::model_preview::ModelDocumentState;
use crate::ui::views::model_preview::gizmos::get_material_color;

struct WgpuEngineState {
    pipeline: WgpuPipeline,
    textures: GpuTextureManager,
    geometries: GpuGeometryCache,
    target: Option<RenderTargetBundle>,
    cached_frame: Option<(WgpuCacheKey, Arc<RenderImage>)>,
}

static ENGINE_STATE: Mutex<Option<WgpuEngineState>> = Mutex::new(None);

/// 渲染 3D 场景并输出高质量 WGPU 硬件渲染图像
pub fn render_wgpu_scene(
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

    let ctx = get_or_init_wgpu()?;

    let mut guard = ENGINE_STATE.lock().ok()?;
    if guard.is_none() {
        let pipeline = WgpuPipeline::new(&ctx.device)?;
        let textures = GpuTextureManager::new(&ctx)?;
        let geometries = GpuGeometryCache::new();
        *guard = Some(WgpuEngineState {
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

struct PreparedMeshDraw {
    bind_group: Arc<wgpu::BindGroup>,
    index_count: u32,
    geom_key: (String, usize),
}

/// 执行单帧 WGPU 渲染 Pass 与显存读回
fn render_scene_pass(
    ctx: &WgpuContext,
    engine: &mut WgpuEngineState,
    state: &ModelDocumentState,
    target_w: u32,
    target_h: u32,
) -> Option<Arc<RenderImage>> {
    RenderTargetBundle::ensure(&mut engine.target, &ctx.device, target_w, target_h)?;

    let aspect = target_w as f32 / target_h as f32;
    let proj = Mat4::perspective_rh(state.camera.fov, aspect, 0.05, 50000.0);
    let (_right, up, _forward) = state.camera.compute_basis();
    let eye = state.camera.compute_eye();
    let view = Mat4::look_at_rh(eye, state.camera.target, up);
    let mvp = proj.mul(&view);

    let sun_norm = normalize_sun([0.35, 0.80, 0.45]);
    let projector = state.camera.projector(target_w as f32, target_h as f32);

    // 1. 预处理所有待绘制网格 (准备显存缓冲、贴图与常量 UBO)
    let mut draw_calls = Vec::new();
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
        let geom = engine.geometries.get_or_upload(&ctx.device, &state.file_path, mesh_idx, mesh)?;

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

        geom.update_uniforms(&ctx.queue, &uniforms);
        let bind_group = geom.get_or_create_bind_group(&ctx.device, &engine.pipeline, &tex_view)?;

        draw_calls.push(PreparedMeshDraw {
            bind_group,
            index_count: geom.index_count,
            geom_key: (state.file_path.clone(), mesh_idx),
        });
    }

    let target = engine.target.as_ref()?;
    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Model Preview Render Encoder"),
    });

    // 2. 执行 GPU RenderPass 绘制全部可见网格
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Model Preview Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &target.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&engine.pipeline.pipeline);

        for draw in &draw_calls {
            let Some(geom) = engine.geometries.get_or_upload(&ctx.device, &draw.geom_key.0, draw.geom_key.1, &state.meshes[draw.geom_key.1]) else {
                continue;
            };

            pass.set_bind_group(0, draw.bind_group.as_ref(), &[]);
            pass.set_vertex_buffer(0, geom.vertex_buffer.slice(..));
            pass.set_index_buffer(geom.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..draw.index_count, 0, 0..1);
        }
    }

    // 3. 将离屏颜色纹理拷贝至可回读暂存缓冲 StagingBuffer
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target.color_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &target.staging_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(target.padded_bytes_per_row),
                rows_per_image: Some(target_h),
            },
        },
        wgpu::Extent3d {
            width: target_w,
            height: target_h,
            depth_or_array_layers: 1,
        },
    );

    let submission = ctx.queue.submit(std::iter::once(encoder.finish()));

    // 4. 映射暂存缓冲并读回原始 RGBA/BGRA 像素
    let (sender, receiver) = std::sync::mpsc::channel();
    target.staging_buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });

    ctx.device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission),
        timeout: Some(std::time::Duration::from_secs(10)),
    }).ok()?;

    receiver.recv_timeout(std::time::Duration::from_secs(10)).ok()?.ok()?;

    let mapped_range = target.staging_buffer.slice(..).get_mapped_range().ok()?;
    let unpadded = target.unpadded_bytes_per_row as usize;
    let padded = target.padded_bytes_per_row as usize;

    let raw_pixels = if unpadded == padded {
        mapped_range[..(unpadded * target_h as usize)].to_vec()
    } else {
        let mut pixels = Vec::with_capacity(unpadded * target_h as usize);
        for chunk in mapped_range.chunks_exact(padded) {
            pixels.extend_from_slice(&chunk[..unpadded]);
        }
        pixels
    };

    drop(mapped_range);
    target.staging_buffer.unmap();

    let img_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
        target_w,
        target_h,
        raw_pixels,
    )?;
    Some(Arc::new(RenderImage::new(vec![Frame::new(img_buffer)])))
}

/// 解析获取网格漫反射贴图视图
fn resolve_mesh_texture(
    ctx: &WgpuContext,
    engine: &mut WgpuEngineState,
    state: &ModelDocumentState,
    mesh: &rbfres_library::ExtractedMesh,
) -> (Arc<wgpu::TextureView>, bool) {
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

fn compute_cache_key(state: &ModelDocumentState, target_w: u32, target_h: u32) -> WgpuCacheKey {
    let mut vis_hash = 0u64;
    for (i, item) in state.mesh_items.iter().enumerate() {
        if item.visible {
            vis_hash = vis_hash.wrapping_add(1u64 << (i % 64));
        }
    }

    WgpuCacheKey {
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
    fn test_wgpu_render_dungeon085() {
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
        let image = render_wgpu_scene(&state, 1280.0, 720.0);
        let elapsed = start.elapsed();
        println!(">>> 200,000 Triangles 1st Frame (Cold Init + Upload) Time: {:?}", elapsed);

        let start2 = std::time::Instant::now();
        let image2 = render_wgpu_scene(&state, 1280.0, 720.0);
        let elapsed2 = start2.elapsed();
        println!(">>> 2nd Frame (GPU Frame Cached) Time: {:?}", elapsed2);

        // 测试相机微旋转 (强制 GPU 重新渲染 200,000 个面)
        let mut rotated_state = state.clone();
        rotated_state.camera.yaw += 0.05;
        let start3 = std::time::Instant::now();
        let image3 = render_wgpu_scene(&rotated_state, 1280.0, 720.0);
        let elapsed3 = start3.elapsed();
        println!(">>> 3rd Frame (Camera Rotating, Full 200,000 Triangles GPU Render) Time: {:?}", elapsed3);

        assert!(image.is_some() && image2.is_some() && image3.is_some());
    }
}
