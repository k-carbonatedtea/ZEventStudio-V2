//! 3D 模型视口 GPU/Canvas 路径投影渲染器

use super::gizmos::{draw_axes, draw_bounding_box, draw_ground_grid, draw_points_fast};
use super::rasterizer::rasterize_scene;
use crate::model::model_preview::{ModelDocumentState, RenderMode};
use gpui::*;

/// 渲染 3D 模型完整视口画布 (包含地面参考网格、坐标轴、模型实体/线框及包围盒)
pub fn draw_model_viewport(
    state: &ModelDocumentState,
    bounds: Bounds<Pixels>,
    window: &mut Window,
) {
    let orig_x: f32 = bounds.origin.x.into();
    let orig_y: f32 = bounds.origin.y.into();
    let view_w: f32 = bounds.size.width.into();
    let view_h: f32 = bounds.size.height.into();

    if view_w <= 10.0 || view_h <= 10.0 {
        return;
    }

    let camera = &state.camera;

    // 1. 绘制底层 3D 地面空间参考网格 (Ground Grid)
    if state.show_grid {
        let ground_y = state.bounding_box_min[1];
        draw_ground_grid(camera, ground_y, orig_x, orig_y, view_w, view_h, window);
    }

    // 2. 绘制 3D 世界坐标轴指示线 (XYZ Axes)
    if state.show_axes {
        let origin = [
            state.bounding_box_min[0],
            state.bounding_box_min[1],
            state.bounding_box_min[2],
        ];
        let axis_len = (state.dimensions()[0]
            .max(state.dimensions()[1])
            .max(state.dimensions()[2])
            * 0.3)
            .max(1.0);
        draw_axes(
            camera, origin, axis_len, orig_x, orig_y, view_w, view_h, window,
        );
    }

    // 3. 绘制 3D 轴对齐外接包围盒 (Bounding Box)
    if state.show_bounds {
        draw_bounding_box(
            camera,
            state.bounding_box_min,
            state.bounding_box_max,
            orig_x,
            orig_y,
            view_w,
            view_h,
            window,
        );
    }

    // 4. 渲染模型几何网格 (依据当前渲染模式：Shaded / Wireframe / Points)
    draw_model_meshes(state, orig_x, orig_y, view_w, view_h, window);
}

/// 渲染模型几何网格图层
fn draw_model_meshes(
    state: &ModelDocumentState,
    orig_x: f32,
    orig_y: f32,
    view_w: f32,
    view_h: f32,
    window: &mut Window,
) {
    let mode = state.render_mode;

    // 1. 线框模式 / 点云模式快速批量绘制
    if mode == RenderMode::Wireframe {
        draw_wireframe_fast(state, orig_x, orig_y, view_w, view_h, window);
        return;
    }

    if mode == RenderMode::Points {
        draw_points_fast(state, orig_x, orig_y, view_w, view_h, window);
        return;
    }

    // 2. 实体着色模式 (Shaded) 或 线框叠加着色模式 (ShadedWireframe)
    // 采用高精度带 Z-Buffer 软光栅化渲染，彻底消除前后层叠穿透并实现平滑法线光照
    if let Some(render_image) = rasterize_scene(state, view_w, view_h) {
        let img_bounds = Bounds {
            origin: point(px(orig_x), px(orig_y)),
            size: size(px(view_w), px(view_h)),
        };
        let _ = window.paint_image(
            img_bounds,
            img_bounds,
            Corners::default(),
            render_image,
            0,
            false,
        );
    }

    // 若开启叠加线框模式，在实体上方追加渲染半透明矢量线框
    if mode == RenderMode::ShadedWireframe {
        draw_wireframe_overlay(state, orig_x, orig_y, view_w, view_h, window);
    }
}

/// 快速绘制纯矢量线框
fn draw_wireframe_fast(
    state: &ModelDocumentState,
    orig_x: f32,
    orig_y: f32,
    view_w: f32,
    view_h: f32,
    window: &mut Window,
) {
    let camera = &state.camera;
    let mut stroke_path = PathBuilder::stroke(px(1.0));
    let mut line_count = 0;

    for (mesh_idx, mesh) in state.meshes.iter().enumerate() {
        if let Some(item) = state.mesh_items.get(mesh_idx) {
            if !item.visible {
                continue;
            }
        }

        let v_len = mesh.positions.len();
        for chunk in mesh.indices.chunks_exact(3) {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 >= v_len || i1 >= v_len || i2 >= v_len {
                continue;
            }

            let p0 = [
                mesh.positions[i0].x,
                mesh.positions[i0].y,
                mesh.positions[i0].z,
            ];
            let p1 = [
                mesh.positions[i1].x,
                mesh.positions[i1].y,
                mesh.positions[i1].z,
            ];
            let p2 = [
                mesh.positions[i2].x,
                mesh.positions[i2].y,
                mesh.positions[i2].z,
            ];

            if let (Some((s0, _)), Some((s1, _)), Some((s2, _))) = (
                camera.project_point(p0, view_w, view_h),
                camera.project_point(p1, view_w, view_h),
                camera.project_point(p2, view_w, view_h),
            ) {
                stroke_path.move_to(point(px(orig_x + s0[0]), px(orig_y + s0[1])));
                stroke_path.line_to(point(px(orig_x + s1[0]), px(orig_y + s1[1])));
                stroke_path.line_to(point(px(orig_x + s2[0]), px(orig_y + s2[1])));
                stroke_path.close();
                line_count += 3;
            }
        }
    }

    if line_count > 0 {
        if let Ok(p) = stroke_path.build() {
            window.paint_path(p, hsla(190.0 / 360.0, 0.85, 0.65, 0.85)); // 希卡青色高亮线框
        }
    }
}

/// 绘制叠加在实体之上的精细弱线框
fn draw_wireframe_overlay(
    state: &ModelDocumentState,
    orig_x: f32,
    orig_y: f32,
    view_w: f32,
    view_h: f32,
    window: &mut Window,
) {
    let camera = &state.camera;
    let mut stroke_path = PathBuilder::stroke(px(0.8));
    let mut line_count = 0;

    for (mesh_idx, mesh) in state.meshes.iter().enumerate() {
        if let Some(item) = state.mesh_items.get(mesh_idx) {
            if !item.visible {
                continue;
            }
        }

        let v_len = mesh.positions.len();
        for chunk in mesh.indices.chunks_exact(3) {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 >= v_len || i1 >= v_len || i2 >= v_len {
                continue;
            }

            let p0 = [
                mesh.positions[i0].x,
                mesh.positions[i0].y,
                mesh.positions[i0].z,
            ];
            let p1 = [
                mesh.positions[i1].x,
                mesh.positions[i1].y,
                mesh.positions[i1].z,
            ];
            let p2 = [
                mesh.positions[i2].x,
                mesh.positions[i2].y,
                mesh.positions[i2].z,
            ];

            if let (Some((s0, _)), Some((s1, _)), Some((s2, _))) = (
                camera.project_point(p0, view_w, view_h),
                camera.project_point(p1, view_w, view_h),
                camera.project_point(p2, view_w, view_h),
            ) {
                // 仅对正面朝向的面绘制叠加线框
                let cross = (s1[0] - s0[0]) * (s2[1] - s0[1]) - (s1[1] - s0[1]) * (s2[0] - s0[0]);
                if cross > 0.0 {
                    stroke_path.move_to(point(px(orig_x + s0[0]), px(orig_y + s0[1])));
                    stroke_path.line_to(point(px(orig_x + s1[0]), px(orig_y + s1[1])));
                    stroke_path.line_to(point(px(orig_x + s2[0]), px(orig_y + s2[1])));
                    stroke_path.close();
                    line_count += 3;
                }
            }
        }
    }

    if line_count > 0 {
        if let Ok(p) = stroke_path.build() {
            window.paint_path(p, rgba(0x0f172a66)); // 深暗色精细线框
        }
    }
}

