//! 3D 软光栅化渲染引擎 (分块多线程并行光栅化、视锥剔除与静态帧缓存)
//!
//! 具备：
//! 1. 静态帧缓存 (Render Cache)：摄像机静止时零开销复用上帧结果 (0.01ms / 60+ FPS)
//! 2. 交互式动态降采样：旋转平移时自动按 0.5x 分辨率加速光栅化，抬起鼠标恢复全清并缓存
//! 3. 视锥与屏幕 AABB 粗剔除：在子网格级别剔除视锥外的整个部件
//! 4. 顶点投影与法线光照预计算：每个几何顶点仅投影与受光计算一次，消除三角面间冗余求值
//! 5. Rayon 水平扫描带分块并行：将屏幕深度与颜色缓冲区划分为无竞争切片多核并发写入

use super::gizmos::get_material_color;
use crate::model::model_preview::{FastCameraProjector, ModelDocumentState, RenderMode, RgbaTexture};
use gpui::*;
use image::{Frame, ImageBuffer, Rgba};
use rayon::prelude::*;
use rbfres_library::ExtractedMesh;
use std::sync::{Arc, Mutex};

const BAND_HEIGHT: u32 = 32;

/// 光栅化渲染缓存键 (用于静止画面零开销复用)
#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderCacheKey {
    model_path: String,
    yaw_bits: u32,
    pitch_bits: u32,
    distance_bits: u32,
    target_bits: [u32; 3],
    fov_bits: u32,
    view_w: u32,
    view_h: u32,
    render_mode: RenderMode,
    selected_mesh_idx: Option<usize>,
    visibility_hash: u64,
    is_interactive: bool,
}

struct CachedImage {
    key: RenderCacheKey,
    image: Arc<RenderImage>,
}

static RENDER_CACHE: Mutex<Option<CachedImage>> = Mutex::new(None);

/// 预计算的单个三角面渲染参数
#[derive(Clone, Copy)]
struct PreparedTriangle {
    s0: [f32; 2],
    s1: [f32; 2],
    s2: [f32; 2],
    inv_z0: f32,
    inv_z1: f32,
    inv_z2: f32,
    int0: f32,
    int1: f32,
    int2: f32,
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
    denom: f32,
    min_x: u32,
    max_x: u32,
    min_y: u32,
    max_y: u32,
}

/// 预计算的子网格图元渲染批次
struct PreparedMeshBatch<'a> {
    min_y: u32,
    max_y: u32,
    texture: Option<&'a RgbaTexture>,
    base_color: [u8; 4],
    is_alpha_cutout: bool,
    triangles: Vec<PreparedTriangle>,
}

/// 执行 3D 场景软光栅化渲染，输出带 Z-Buffer 的高质量 RGBA 图像
pub fn rasterize_scene(
    state: &ModelDocumentState,
    view_w: f32,
    view_h: f32,
) -> Option<Arc<RenderImage>> {
    let is_interactive = state.is_orbiting || state.is_panning;
    let scale: f32 = if is_interactive { 0.5 } else { 1.0 };

    let target_w = ((view_w * scale).round() as u32).clamp(16, 2560);
    let target_h = ((view_h * scale).round() as u32).clamp(16, 1600);

    // 1. 检查静态渲染缓存 (画面未变动直接瞬时返回)
    let cache_key = compute_cache_key(state, target_w, target_h, is_interactive);
    if let Ok(guard) = RENDER_CACHE.lock() {
        if let Some(ref cached) = *guard {
            if cached.key == cache_key {
                return Some(cached.image.clone());
            }
        }
    }

    let projector = state.camera.projector(target_w as f32, target_h as f32);
    let sun_dir = [0.35f32, 0.80f32, 0.45f32];
    let sun_len = (sun_dir[0] * sun_dir[0] + sun_dir[1] * sun_dir[1] + sun_dir[2] * sun_dir[2]).sqrt();
    let sun_norm = [sun_dir[0] / sun_len, sun_dir[1] / sun_len, sun_dir[2] / sun_len];

    // 2. 预计算所有可见子网格的顶点投影与三角面包围盒
    let batches: Vec<PreparedMeshBatch> = state
        .meshes
        .iter()
        .enumerate()
        .filter_map(|(idx, mesh)| {
            prepare_mesh_batch(mesh, idx, state, &projector, sun_norm, target_w, target_h)
        })
        .collect();

    let total_pixels = (target_w * target_h) as usize;
    let mut color_buffer = vec![0u8; total_pixels * 4];
    let mut depth_buffer = vec![f32::INFINITY; total_pixels];

    // 3. 水平扫描带分块并行光栅化 (Rayon 多核并发)
    let band_pixel_count = (BAND_HEIGHT * target_w) as usize;
    let band_color_count = (BAND_HEIGHT * target_w * 4) as usize;

    depth_buffer
        .par_chunks_mut(band_pixel_count)
        .zip(color_buffer.par_chunks_mut(band_color_count))
        .enumerate()
        .for_each(|(band_idx, (depth_band, color_band))| {
            let band_y_start = (band_idx as u32) * BAND_HEIGHT;
            let band_rows = (depth_band.len() as u32) / target_w;
            let band_y_end = band_y_start + band_rows.saturating_sub(1);

            for batch in &batches {
                // 网格粗剔除
                if batch.min_y > band_y_end || batch.max_y < band_y_start {
                    continue;
                }

                for tri in &batch.triangles {
                    // 三角形细剔除
                    if tri.min_y > band_y_end || tri.max_y < band_y_start {
                        continue;
                    }

                    rasterize_triangle_in_band(
                        tri,
                        batch.texture,
                        batch.base_color,
                        batch.is_alpha_cutout,
                        target_w,
                        band_y_start,
                        band_y_end,
                        depth_band,
                        color_band,
                    );
                }
            }
        });

    let img_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(target_w, target_h, color_buffer)?;
    let frame = Frame::new(img_buffer);
    let render_image = Arc::new(RenderImage::new(vec![frame]));

    // 4. 写入静态渲染缓存
    if let Ok(mut guard) = RENDER_CACHE.lock() {
        *guard = Some(CachedImage {
            key: cache_key,
            image: render_image.clone(),
        });
    }

    Some(render_image)
}

/// 预计算单个子网格的全部三角面渲染批次
fn prepare_mesh_batch<'a>(
    mesh: &'a ExtractedMesh,
    mesh_idx: usize,
    state: &'a ModelDocumentState,
    projector: &FastCameraProjector,
    sun_norm: [f32; 3],
    buf_w: u32,
    buf_h: u32,
) -> Option<PreparedMeshBatch<'a>> {
    if !is_mesh_visible(state, mesh_idx) {
        return None;
    }

    let b_min = [mesh.bounding_box.min().x, mesh.bounding_box.min().y, mesh.bounding_box.min().z];
    let b_max = [mesh.bounding_box.max().x, mesh.bounding_box.max().y, mesh.bounding_box.max().z];
    if projector.is_aabb_outside(b_min, b_max, buf_w as f32, buf_h as f32) {
        return None;
    }

    let is_selected = state.selected_mesh_idx == Some(mesh_idx);
    let base_hsla = get_material_color(
        mesh.material_name.as_deref().unwrap_or(""),
        mesh_idx,
        is_selected,
    );
    let base_color = hsla_to_rgba_u8(base_hsla);
    let texture = resolve_texture(state, mesh);
    let is_alpha_cutout = mesh.is_alpha_cutout();

    let v_len = mesh.positions.len();
    let norm_len = mesh.normals.len();
    let uv_len = mesh.uvs.len();

    // 唯一顶点投影预计算 (共享顶点仅投影一次)
    let mut proj_verts: Vec<Option<([f32; 2], f32)>> = Vec::with_capacity(v_len);
    for p in &mesh.positions {
        proj_verts.push(projector.project([p.x, p.y, p.z]));
    }

    // 顶点法线平滑光照预计算 (区分正反面照度)
    let mut lighting_front: Vec<f32> = Vec::with_capacity(v_len);
    let mut lighting_back: Vec<f32> = Vec::with_capacity(v_len);
    for i in 0..v_len {
        let (lf, lb) = if i < norm_len {
            let n = &mesh.normals[i];
            let dot = n.x * sun_norm[0] + n.y * sun_norm[1] + n.z * sun_norm[2];
            let front = (0.25 + 0.75 * (dot * 0.5 + 0.5)).clamp(0.18, 1.0);
            let back = (0.25 + 0.75 * (-dot * 0.5 + 0.5)).clamp(0.18, 1.0);
            (front, back)
        } else {
            (0.85, 0.85)
        };
        lighting_front.push(lf);
        lighting_back.push(lb);
    }

    let mut triangles = Vec::with_capacity(mesh.indices.len() / 3);
    let mut batch_min_y = u32::MAX;
    let mut batch_max_y = 0;

    let max_buf_x = buf_w - 1;
    let max_buf_y = buf_h - 1;

    for chunk in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (chunk[0] as usize, chunk[1] as usize, chunk[2] as usize);
        if i0 >= v_len || i1 >= v_len || i2 >= v_len {
            continue;
        }

        let (Some((s0, z0)), Some((s1, z1)), Some((s2, z2))) =
            (proj_verts[i0], proj_verts[i1], proj_verts[i2]) else { continue };

        let denom = (s1[0] - s0[0]) * (s2[1] - s0[1]) - (s1[1] - s0[1]) * (s2[0] - s0[0]);
        if denom.abs() < 0.001 {
            continue;
        }

        let is_back = denom > 0.0;
        let (int0, int1, int2) = if is_back {
            (lighting_back[i0], lighting_back[i1], lighting_back[i2])
        } else {
            (lighting_front[i0], lighting_front[i1], lighting_front[i2])
        };

        let min_x = (s0[0].min(s1[0]).min(s2[0]).floor() as i32).clamp(0, max_buf_x as i32) as u32;
        let max_x = (s0[0].max(s1[0]).max(s2[0]).ceil() as i32).clamp(0, max_buf_x as i32) as u32;
        let min_y = (s0[1].min(s1[1]).min(s2[1]).floor() as i32).clamp(0, max_buf_y as i32) as u32;
        let max_y = (s0[1].max(s1[1]).max(s2[1]).ceil() as i32).clamp(0, max_buf_y as i32) as u32;

        if min_x > max_x || min_y > max_y {
            continue;
        }

        batch_min_y = batch_min_y.min(min_y);
        batch_max_y = batch_max_y.max(max_y);

        let uv0 = if i0 < uv_len { [mesh.uvs[i0].x, mesh.uvs[i0].y] } else { [0.0, 0.0] };
        let uv1 = if i1 < uv_len { [mesh.uvs[i1].x, mesh.uvs[i1].y] } else { [0.0, 0.0] };
        let uv2 = if i2 < uv_len { [mesh.uvs[i2].x, mesh.uvs[i2].y] } else { [0.0, 0.0] };

        triangles.push(PreparedTriangle {
            s0, s1, s2,
            inv_z0: 1.0 / z0,
            inv_z1: 1.0 / z1,
            inv_z2: 1.0 / z2,
            int0, int1, int2,
            uv0, uv1, uv2,
            denom,
            min_x, max_x, min_y, max_y,
        });
    }

    if triangles.is_empty() {
        return None;
    }

    Some(PreparedMeshBatch {
        min_y: batch_min_y,
        max_y: batch_max_y,
        texture,
        base_color,
        is_alpha_cutout,
        triangles,
    })
}

/// 在指定水平扫描带内光栅化单个三角形
#[inline]
#[allow(clippy::too_many_arguments)]
fn rasterize_triangle_in_band(
    tri: &PreparedTriangle,
    texture: Option<&RgbaTexture>,
    base_color: [u8; 4],
    is_alpha_cutout: bool,
    buf_w: u32,
    band_y_start: u32,
    band_y_end: u32,
    depth_band: &mut [f32],
    color_band: &mut [u8],
) {
    let start_y = tri.min_y.max(band_y_start);
    let end_y = tri.max_y.min(band_y_end);
    if start_y > end_y {
        return;
    }

    let start_x = tri.min_x;
    let end_x = tri.max_x;
    let inv_denom = 1.0 / tri.denom;

    for y in start_y..=end_y {
        let py = y as f32 + 0.5;
        let local_y = y - band_y_start;
        let row_offset = (local_y * buf_w) as usize;

        for x in start_x..=end_x {
            let px = x as f32 + 0.5;

            let w0 = ((tri.s1[0] - px) * (tri.s2[1] - py)
                - (tri.s1[1] - py) * (tri.s2[0] - px))
                * inv_denom;
            let w1 = ((tri.s2[0] - px) * (tri.s0[1] - py)
                - (tri.s2[1] - py) * (tri.s0[0] - px))
                * inv_denom;
            let w2 = 1.0 - w0 - w1;

            if w0 < -0.001 || w1 < -0.001 || w2 < -0.001 {
                continue;
            }

            let inv_z = w0 * tri.inv_z0 + w1 * tri.inv_z1 + w2 * tri.inv_z2;
            let z = 1.0 / inv_z;

            let pixel_idx = row_offset + x as usize;
            if pixel_idx >= depth_band.len() || z >= depth_band[pixel_idx] {
                continue;
            }

            let intensity = w0 * tri.int0 + w1 * tri.int1 + w2 * tri.int2;

            let (r, g, b, a) = if let Some(tex) = texture {
                let u = (w0 * tri.uv0[0] * tri.inv_z0
                    + w1 * tri.uv1[0] * tri.inv_z1
                    + w2 * tri.uv2[0] * tri.inv_z2)
                    * z;
                let v = (w0 * tri.uv0[1] * tri.inv_z0
                    + w1 * tri.uv1[1] * tri.inv_z1
                    + w2 * tri.uv2[1] * tri.inv_z2)
                    * z;
                let c = tex.sample_rgba_raw(u, v);
                (c[0], c[1], c[2], c[3])
            } else {
                (base_color[0], base_color[1], base_color[2], base_color[3])
            };

            if is_alpha_cutout && a < 25 {
                continue;
            }

            depth_band[pixel_idx] = z;

            let out_idx = pixel_idx * 4;
            if out_idx + 3 < color_band.len() {
                color_band[out_idx] = ((b as f32 * intensity) as u32).min(255) as u8;
                color_band[out_idx + 1] = ((g as f32 * intensity) as u32).min(255) as u8;
                color_band[out_idx + 2] = ((r as f32 * intensity) as u32).min(255) as u8;
                color_band[out_idx + 3] = if is_alpha_cutout { a } else { 255 };
            }
        }
    }
}

/// 检索子网格绑定的主漫反射贴图
fn resolve_texture<'a>(
    state: &'a ModelDocumentState,
    mesh: &ExtractedMesh,
) -> Option<&'a RgbaTexture> {
    if let Some(tn) = mesh.texture_name.as_deref() {
        if let Some(tex) = state.textures.get(tn) {
            return Some(&**tex);
        }
    }
    for ref_name in &mesh.texture_refs {
        if let Some(tex) = state.textures.get(ref_name) {
            return Some(&**tex);
        }
    }
    if let Some(mn) = mesh.material_name.as_deref() {
        if let Some(tex) = state.textures.get(mn) {
            return Some(&**tex);
        }
    }
    None
}

/// 判断指定子网格是否处于可见状态
#[inline]
fn is_mesh_visible(state: &ModelDocumentState, mesh_idx: usize) -> bool {
    state.mesh_items.get(mesh_idx).map_or(true, |item| item.visible)
}

/// 将 GPUI Hsla 快速转换为标准 RGBA u8 四元组
#[inline]
fn hsla_to_rgba_u8(color: Hsla) -> [u8; 4] {
    let rgba = gpui::Rgba::from(color);
    [
        (rgba.r * 255.0).clamp(0.0, 255.0) as u8,
        (rgba.g * 255.0).clamp(0.0, 255.0) as u8,
        (rgba.b * 255.0).clamp(0.0, 255.0) as u8,
        (rgba.a * 255.0).clamp(0.0, 255.0) as u8,
    ]
}

/// 计算当前模型状态的快照哈希缓存键
fn compute_cache_key(
    state: &ModelDocumentState,
    target_w: u32,
    target_h: u32,
    is_interactive: bool,
) -> RenderCacheKey {
    let mut vis_hash = 0u64;
    for (i, item) in state.mesh_items.iter().enumerate() {
        if item.visible {
            vis_hash = vis_hash.wrapping_add(1u64 << (i % 64));
        }
    }

    RenderCacheKey {
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
        is_interactive,
    }
}
