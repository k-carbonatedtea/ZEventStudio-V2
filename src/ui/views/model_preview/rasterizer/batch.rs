//! 几何子网格预计算与批次装配处理

use super::types::{PreparedMeshBatch, PreparedTriangle};
use crate::model::model_preview::{FastCameraProjector, ModelDocumentState, RgbaTexture};
use crate::ui::views::model_preview::gizmos::get_material_color;
use gpui::Hsla;
use rbfres_library::ExtractedMesh;

/// 预计算单个子网格的全部三角面渲染批次
pub fn prepare_mesh_batch<'a>(
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
    if projector.is_all_behind_camera(b_min, b_max) {
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
    state.mesh_items.get(mesh_idx).is_none_or(|item| item.visible)
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
