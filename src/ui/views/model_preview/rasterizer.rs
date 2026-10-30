//! 3D 软光栅化渲染引擎 (Software Rasterizer with Z-Buffer & Gouraud Shading)
//!
//! 具备：
//! 1. 像素级深度缓冲区 (Z-Buffer)：彻底消除穿透、破面与前后层叠穿模
//! 2. 顶点法线平滑插值 (Gouraud Smooth Shading)：呈现平滑有机曲面
//! 3. 仿射/透视纹理映射：支持全精度 UV 贴图与 Alpha 镂空测试
//! 4. 屏幕空间正面判定：修正 GPUI Y 轴倒置引发的内外翻转问题

use super::gizmos::get_material_color;
use crate::model::model_preview::{ModelDocumentState, RgbaTexture};
use gpui::*;
use image::{Frame, ImageBuffer, Rgba};
use rbfres_library::ExtractedMesh;
use std::sync::Arc;

/// 光栅化顶点数据结构
#[derive(Clone, Copy)]
struct RasterVertex {
    screen: [f32; 2],
    inv_z: f32,
    intensity: f32,
    uv: [f32; 2],
}

/// 执行 3D 场景软光栅化渲染，输出带 Z-Buffer 的高质量 RGBA 图像
pub fn rasterize_scene(
    state: &ModelDocumentState,
    view_w: f32,
    view_h: f32,
) -> Option<Arc<RenderImage>> {
    let w = (view_w.round() as u32).clamp(16, 2560);
    let h = (view_h.round() as u32).clamp(16, 1600);
    let total_pixels = (w * h) as usize;

    let mut color_buffer = vec![0u8; total_pixels * 4];
    let mut depth_buffer = vec![f32::INFINITY; total_pixels];

    // 主平行光源方向 (斜上方日光) 与视线反方向补光
    let sun_dir = [0.35f32, 0.80f32, 0.45f32];
    let sun_len = (sun_dir[0] * sun_dir[0] + sun_dir[1] * sun_dir[1] + sun_dir[2] * sun_dir[2]).sqrt();
    let sun_norm = [sun_dir[0] / sun_len, sun_dir[1] / sun_len, sun_dir[2] / sun_len];

    for (mesh_idx, mesh) in state.meshes.iter().enumerate() {
        if !is_mesh_visible(state, mesh_idx) {
            continue;
        }

        let is_selected = state.selected_mesh_idx == Some(mesh_idx);
        let base_hsla = get_material_color(
            mesh.material_name.as_deref().unwrap_or(""),
            mesh_idx,
            is_selected,
        );
        let base_rgba = hsla_to_rgba_u8(base_hsla);

        let bound_tex = resolve_texture(state, mesh);

        rasterize_mesh(
            mesh,
            state,
            view_w,
            view_h,
            sun_norm,
            bound_tex,
            base_rgba,
            w,
            h,
            &mut color_buffer,
            &mut depth_buffer,
        );
    }

    let img_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(w, h, color_buffer)?;
    let frame = Frame::new(img_buffer);
    Some(Arc::new(RenderImage::new(vec![frame])))
}

/// 判断指定子网格是否处于可见状态
#[inline]
fn is_mesh_visible(state: &ModelDocumentState, mesh_idx: usize) -> bool {
    if let Some(item) = state.mesh_items.get(mesh_idx) {
        item.visible
    } else {
        true
    }
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

/// 检索子网格绑定的主漫反射贴图
fn resolve_texture<'a>(
    state: &'a ModelDocumentState,
    mesh: &ExtractedMesh,
) -> Option<&'a RgbaTexture> {
    // 1. 优先尝试主漫反射贴图名称
    if let Some(tn) = mesh.texture_name.as_deref() {
        if let Some(tex) = state.textures.get(tn) {
            return Some(&**tex);
        }
    }
    // 2. 依次匹配材质全部候选贴图清单 (挑选首个已在内存中解码成功的纹理)
    for ref_name in &mesh.texture_refs {
        if let Some(tex) = state.textures.get(ref_name) {
            return Some(&**tex);
        }
    }
    // 3. 尝试材质自身标识名
    if let Some(mn) = mesh.material_name.as_deref() {
        if let Some(tex) = state.textures.get(mn) {
            return Some(&**tex);
        }
    }
    None
}

/// 对单个子网格的所有三角面进行光栅化投影
#[allow(clippy::too_many_arguments)]
fn rasterize_mesh(
    mesh: &ExtractedMesh,
    state: &ModelDocumentState,
    view_w: f32,
    view_h: f32,
    sun_norm: [f32; 3],
    texture: Option<&RgbaTexture>,
    base_color: [u8; 4],
    buf_w: u32,
    buf_h: u32,
    color_buf: &mut [u8],
    depth_buf: &mut [f32],
) {
    let camera = &state.camera;
    let v_len = mesh.positions.len();
    let norm_len = mesh.normals.len();
    let uv_len = mesh.uvs.len();
    let is_alpha_cutout = mesh.is_alpha_cutout();

    for chunk in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (chunk[0] as usize, chunk[1] as usize, chunk[2] as usize);
        if i0 >= v_len || i1 >= v_len || i2 >= v_len {
            continue;
        }

        let p0 = [mesh.positions[i0].x, mesh.positions[i0].y, mesh.positions[i0].z];
        let p1 = [mesh.positions[i1].x, mesh.positions[i1].y, mesh.positions[i1].z];
        let p2 = [mesh.positions[i2].x, mesh.positions[i2].y, mesh.positions[i2].z];

        let proj0 = camera.project_point(p0, view_w, view_h);
        let proj1 = camera.project_point(p1, view_w, view_h);
        let proj2 = camera.project_point(p2, view_w, view_h);

        let (Some((s0, z0)), Some((s1, z1)), Some((s2, z2))) = (proj0, proj1, proj2) else {
            continue;
        };

        // 面积退化三角形过滤 (消除零面积面引发的除零错误，同时支持双面材质渲染)
        let denom = (s1[0] - s0[0]) * (s2[1] - s0[1]) - (s1[1] - s0[1]) * (s2[0] - s0[0]);
        if denom.abs() < 0.001 {
            continue;
        }

        let is_back_facing = denom > 0.0;

        let int0 = calc_lighting(mesh, i0, norm_len, p0, p1, p2, sun_norm, is_back_facing);
        let int1 = calc_lighting(mesh, i1, norm_len, p0, p1, p2, sun_norm, is_back_facing);
        let int2 = calc_lighting(mesh, i2, norm_len, p0, p1, p2, sun_norm, is_back_facing);

        let uv0 = if i0 < uv_len { [mesh.uvs[i0].x, mesh.uvs[i0].y] } else { [0.0, 0.0] };
        let uv1 = if i1 < uv_len { [mesh.uvs[i1].x, mesh.uvs[i1].y] } else { [0.0, 0.0] };
        let uv2 = if i2 < uv_len { [mesh.uvs[i2].x, mesh.uvs[i2].y] } else { [0.0, 0.0] };

        let v0 = RasterVertex { screen: s0, inv_z: 1.0 / z0, intensity: int0, uv: uv0 };
        let v1 = RasterVertex { screen: s1, inv_z: 1.0 / z1, intensity: int1, uv: uv1 };
        let v2 = RasterVertex { screen: s2, inv_z: 1.0 / z2, intensity: int2, uv: uv2 };

        rasterize_triangle(
            v0, v1, v2, denom, texture, base_color, buf_w, buf_h, is_alpha_cutout, color_buf, depth_buf,
        );
    }
}

/// 计算顶点的平滑光照强度 (Half-Lambert Nintendo 光照模型，支持背面法线翻转双面照明)
#[inline]
#[allow(clippy::too_many_arguments)]
fn calc_lighting(
    mesh: &ExtractedMesh,
    idx: usize,
    norm_len: usize,
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    sun_norm: [f32; 3],
    is_back_facing: bool,
) -> f32 {
    let mut norm = if idx < norm_len {
        [mesh.normals[idx].x, mesh.normals[idx].y, mesh.normals[idx].z]
    } else {
        super::gizmos::compute_normal(p0, p1, p2)
    };

    if is_back_facing {
        norm[0] = -norm[0];
        norm[1] = -norm[1];
        norm[2] = -norm[2];
    }

    let dot = norm[0] * sun_norm[0] + norm[1] * sun_norm[1] + norm[2] * sun_norm[2];
    let half_lambert = dot * 0.5 + 0.5;
    (0.25 + 0.75 * half_lambert).clamp(0.18, 1.0)
}

/// 扫描单个三角形并执行重心坐标插值、Z-Buffer 测试与纹理着色
#[allow(clippy::too_many_arguments)]
fn rasterize_triangle(
    v0: RasterVertex,
    v1: RasterVertex,
    v2: RasterVertex,
    denom: f32,
    texture: Option<&RgbaTexture>,
    base_color: [u8; 4],
    buf_w: u32,
    buf_h: u32,
    is_alpha_cutout: bool,
    color_buf: &mut [u8],
    depth_buf: &mut [f32],
) {
    let min_x = v0.screen[0].min(v1.screen[0]).min(v2.screen[0]).floor() as i32;
    let max_x = v0.screen[0].max(v1.screen[0]).max(v2.screen[0]).ceil() as i32;
    let min_y = v0.screen[1].min(v1.screen[1]).min(v2.screen[1]).floor() as i32;
    let max_y = v0.screen[1].max(v1.screen[1]).max(v2.screen[1]).ceil() as i32;

    let start_x = min_x.clamp(0, buf_w as i32 - 1) as u32;
    let end_x = max_x.clamp(0, buf_w as i32 - 1) as u32;
    let start_y = min_y.clamp(0, buf_h as i32 - 1) as u32;
    let end_y = max_y.clamp(0, buf_h as i32 - 1) as u32;

    if start_x > end_x || start_y > end_y {
        return;
    }

    let inv_denom = 1.0 / denom;

    for y in start_y..=end_y {
        let py = y as f32 + 0.5;
        let row_offset = (y * buf_w) as usize;

        for x in start_x..=end_x {
            let px = x as f32 + 0.5;

            let w0 = ((v1.screen[0] - px) * (v2.screen[1] - py)
                - (v1.screen[1] - py) * (v2.screen[0] - px))
                * inv_denom;
            let w1 = ((v2.screen[0] - px) * (v0.screen[1] - py)
                - (v2.screen[1] - py) * (v0.screen[0] - px))
                * inv_denom;
            let w2 = 1.0 - w0 - w1;

            if w0 < -0.001 || w1 < -0.001 || w2 < -0.001 {
                continue;
            }

            let inv_z = w0 * v0.inv_z + w1 * v1.inv_z + w2 * v2.inv_z;
            let z = 1.0 / inv_z;

            let pixel_idx = row_offset + x as usize;
            if z >= depth_buf[pixel_idx] {
                continue;
            }

            let intensity = w0 * v0.intensity + w1 * v1.intensity + w2 * v2.intensity;

            let (r, g, b, a) = if let Some(tex) = texture {
                let u = (w0 * v0.uv[0] * v0.inv_z + w1 * v1.uv[0] * v1.inv_z + w2 * v2.uv[0] * v2.inv_z) * z;
                let v = (w0 * v0.uv[1] * v0.inv_z + w1 * v1.uv[1] * v1.inv_z + w2 * v2.uv[1] * v2.inv_z) * z;
                let c = tex.sample_rgba_raw(u, v);
                (c[0], c[1], c[2], c[3])
            } else {
                (base_color[0], base_color[1], base_color[2], base_color[3])
            };

            // 仅对睫毛、半透明镂空等特定子网格执行 Alpha 镂空丢弃；
            // 实体不透明材质即使在 BC1 贴图中携带 a=0 亦完整着色，与 Switch Toolbox 渲染完全对齐
            if is_alpha_cutout && a < 25 {
                continue;
            }

            depth_buf[pixel_idx] = z;

            let out_idx = pixel_idx * 4;
            // 写入 GPUI 要求的 BGRA 格式
            color_buf[out_idx] = ((b as f32 * intensity) as u32).min(255) as u8;
            color_buf[out_idx + 1] = ((g as f32 * intensity) as u32).min(255) as u8;
            color_buf[out_idx + 2] = ((r as f32 * intensity) as u32).min(255) as u8;
            color_buf[out_idx + 3] = if is_alpha_cutout { a } else { 255 };
        }
    }
}
