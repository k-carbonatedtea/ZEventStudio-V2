//! 3D 空间辅助参考图元 (地面网格、世界坐标轴与外接包围盒) 渲染实现

use gpui::*;
use crate::model::model_preview::{ModelDocumentState, OrbitCamera};

/// 绘制底层地面三维透视网格
pub fn draw_ground_grid(
    camera: &OrbitCamera,
    ground_y: f32,
    orig_x: f32,
    orig_y: f32,
    view_w: f32,
    view_h: f32,
    window: &mut Window,
) {
    let mut grid_path = PathBuilder::stroke(px(0.8));
    let grid_size = (camera.distance * 0.8).clamp(5.0, 100.0);
    let step = (grid_size / 10.0).max(0.5);
    let half_extent = grid_size * 0.5;

    let center_x = camera.target[0];
    let center_z = camera.target[2];

    let mut count = 0;
    let mut i = -half_extent;
    while i <= half_extent + 0.001 {
        // X 方向平行线
        let p0_3d = [center_x - half_extent, ground_y, center_z + i];
        let p1_3d = [center_x + half_extent, ground_y, center_z + i];
        if let (Some((s0, _)), Some((s1, _))) = (
            camera.project_point(p0_3d, view_w, view_h),
            camera.project_point(p1_3d, view_w, view_h),
        ) {
            grid_path.move_to(point(px(orig_x + s0[0]), px(orig_y + s0[1])));
            grid_path.line_to(point(px(orig_x + s1[0]), px(orig_y + s1[1])));
            count += 1;
        }

        // Z 方向平行线
        let p2_3d = [center_x + i, ground_y, center_z - half_extent];
        let p3_3d = [center_x + i, ground_y, center_z + half_extent];
        if let (Some((s2, _)), Some((s3, _))) = (
            camera.project_point(p2_3d, view_w, view_h),
            camera.project_point(p3_3d, view_w, view_h),
        ) {
            grid_path.move_to(point(px(orig_x + s2[0]), px(orig_y + s2[1])));
            grid_path.line_to(point(px(orig_x + s3[0]), px(orig_y + s3[1])));
            count += 1;
        }

        i += step;
    }

    if count > 0 {
        if let Ok(path) = grid_path.build() {
            window.paint_path(path, rgba(0x38bdf818)); // 浅青色参考网格线
        }
    }
}

/// 绘制三维世界坐标轴指示线 (X: 红色, Y: 绿色, Z: 蓝色)
#[allow(clippy::too_many_arguments)]
pub fn draw_axes(
    camera: &OrbitCamera,
    origin: [f32; 3],
    length: f32,
    orig_x: f32,
    orig_y: f32,
    view_w: f32,
    view_h: f32,
    window: &mut Window,
) {
    let axis_configs = [
        ([origin[0] + length, origin[1], origin[2]], rgba(0xef4444cc)), // X 轴：红
        ([origin[0], origin[1] + length, origin[2]], rgba(0x22c55ecc)), // Y 轴：绿
        ([origin[0], origin[1], origin[2] + length], rgba(0x3b82f6cc)), // Z 轴：蓝
    ];

    if let Some((s_origin, _)) = camera.project_point(origin, view_w, view_h) {
        let ox = orig_x + s_origin[0];
        let oy = orig_y + s_origin[1];

        for (end_3d, color) in axis_configs {
            if let Some((s_end, _)) = camera.project_point(end_3d, view_w, view_h) {
                let mut path = PathBuilder::stroke(px(2.0));
                path.move_to(point(px(ox), px(oy)));
                path.line_to(point(px(orig_x + s_end[0]), px(orig_y + s_end[1])));
                if let Ok(p) = path.build() {
                    window.paint_path(p, color);
                }
            }
        }
    }
}

/// 绘制模型 3D 轴对齐包围盒 (12 条线框边界)
#[allow(clippy::too_many_arguments)]
pub fn draw_bounding_box(
    camera: &OrbitCamera,
    min: [f32; 3],
    max: [f32; 3],
    orig_x: f32,
    orig_y: f32,
    view_w: f32,
    view_h: f32,
    window: &mut Window,
) {
    let corners = [
        [min[0], min[1], min[2]], // 0
        [max[0], min[1], min[2]], // 1
        [max[0], max[1], min[2]], // 2
        [min[0], max[1], min[2]], // 3
        [min[0], min[1], max[2]], // 4
        [max[0], min[1], max[2]], // 5
        [max[0], max[1], max[2]], // 6
        [min[0], max[1], max[2]], // 7
    ];

    let edges = [
        (0, 1), (1, 2), (2, 3), (3, 0), // 底面
        (4, 5), (5, 6), (6, 7), (7, 4), // 顶面
        (0, 4), (1, 5), (2, 6), (3, 7), // 立柱
    ];

    let mut path = PathBuilder::stroke(px(1.2));
    let mut has_lines = false;

    for (a, b) in edges {
        if let (Some((sa, _)), Some((sb, _))) = (
            camera.project_point(corners[a], view_w, view_h),
            camera.project_point(corners[b], view_w, view_h),
        ) {
            path.move_to(point(px(orig_x + sa[0]), px(orig_y + sa[1])));
            path.line_to(point(px(orig_x + sb[0]), px(orig_y + sb[1])));
            has_lines = true;
        }
    }

    if has_lines {
        if let Ok(p) = path.build() {
            window.paint_path(p, rgba(0xf59e0baa)); // 金黄色包围盒提示线
        }
    }
}

/// 计算三点构成的面三角形单位法线向量
#[allow(dead_code)]
pub fn compute_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let v01 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let v02 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

    let normal_raw = [
        v01[1] * v02[2] - v01[2] * v02[1],
        v01[2] * v02[0] - v01[0] * v02[2],
        v01[0] * v02[1] - v01[1] * v02[0],
    ];

    let len = (normal_raw[0] * normal_raw[0]
        + normal_raw[1] * normal_raw[1]
        + normal_raw[2] * normal_raw[2])
        .sqrt();
    if len < 0.00001 {
        [0.0, 1.0, 0.0]
    } else {
        [
            normal_raw[0] / len,
            normal_raw[1] / len,
            normal_raw[2] / len,
        ]
    }
}

/// 根据材质名称与网格索引匹配优雅的塞尔达旷野风格基准色
pub fn get_material_color(material_name: &str, mesh_idx: usize, is_selected: bool) -> Hsla {
    if is_selected {
        return hsla(45.0 / 360.0, 0.98, 0.60, 1.0); // 选中态：高亮发光金黄
    }

    let lower = material_name.to_lowercase();
    if lower.contains("skin") || lower.contains("face") || lower.contains("body") {
        hsla(28.0 / 360.0, 0.50, 0.70, 1.0)
    } else if lower.contains("metal")
        || lower.contains("iron")
        || lower.contains("steel")
        || lower.contains("armor")
    {
        hsla(215.0 / 360.0, 0.22, 0.65, 1.0)
    } else if lower.contains("gold") {
        hsla(45.0 / 360.0, 0.85, 0.58, 1.0)
    } else if lower.contains("hair") {
        hsla(32.0 / 360.0, 0.65, 0.42, 1.0)
    } else if lower.contains("cloth") || lower.contains("fabric") || lower.contains("dress") {
        hsla(205.0 / 360.0, 0.60, 0.55, 1.0)
    } else if lower.contains("eye") {
        hsla(180.0 / 360.0, 0.85, 0.60, 1.0)
    } else {
        let hue = ((mesh_idx * 67 + 200) % 360) as f32;
        hsla(hue / 360.0, 0.45, 0.60, 1.0)
    }
}

/// 快速绘制顶点散点云
pub fn draw_points_fast(
    state: &ModelDocumentState,
    orig_x: f32,
    orig_y: f32,
    view_w: f32,
    view_h: f32,
    window: &mut Window,
) {
    let camera = &state.camera;
    let point_size = 2.0;
    let mut batch_points = PathBuilder::fill();
    let mut point_count = 0;

    for (mesh_idx, mesh) in state.meshes.iter().enumerate() {
        if let Some(item) = state.mesh_items.get(mesh_idx) {
            if !item.visible {
                continue;
            }
        }

        for pos in &mesh.positions {
            let p = [pos.x, pos.y, pos.z];
            if let Some((s, _)) = camera.project_point(p, view_w, view_h) {
                let sx = orig_x + s[0];
                let sy = orig_y + s[1];
                let hs = point_size * 0.5;
                batch_points.move_to(point(px(sx - hs), px(sy - hs)));
                batch_points.line_to(point(px(sx + hs), px(sy - hs)));
                batch_points.line_to(point(px(sx + hs), px(sy + hs)));
                batch_points.line_to(point(px(sx - hs), px(sy + hs)));
                batch_points.close();
                point_count += 1;
            }
        }
    }

    if point_count > 0 {
        if let Ok(p) = batch_points.build() {
            window.paint_path(p, hsla(190.0 / 360.0, 0.90, 0.65, 0.85));
        }
    }
}

