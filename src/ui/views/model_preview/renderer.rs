//! 3D 模型视口 GPU/Canvas 路径投影渲染器

use super::gizmos::{draw_axes, draw_bounding_box, draw_ground_grid, draw_points_fast};
use super::rasterizer::rasterize_scene;
use super::vulkan_renderer::render_vulkan_scene;
use crate::model::model_preview::{ModelDocumentState, RenderMode};
use gpui::*;
use std::sync::{Arc, Mutex};

static LAST_PAINTED_FRAME: Mutex<Option<Arc<RenderImage>>> = Mutex::new(None);

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
    // 采用 Vulkano 原生 Vulkan GPU 硬件渲染管线，支持硬件纹理双线性采样与物理级深度测试
    let scale_factor = window.scale_factor();
    let phys_w = (view_w * scale_factor).round();
    let phys_h = (view_h * scale_factor).round();

    let render_image = render_vulkan_scene(state, phys_w, phys_h)
        .or_else(|| rasterize_scene(state, phys_w, phys_h));

    if let Some(new_img) = render_image {
        let img_bounds = Bounds {
            origin: point(px(orig_x), px(orig_y)),
            size: size(px(view_w), px(view_h)),
        };
        let _ = window.paint_image(
            img_bounds,
            img_bounds,
            Corners::default(),
            new_img.clone(),
            0,
            false,
        );

        // 核心显存与内存回收防泄漏机制：
        // 显式从 GPUI DirectX 11 Atlas 中剔除释放上一帧旧图像瓦片，防止连续旋转时 Atlas 堆叠累计纹理至数十 GB
        if let Ok(mut guard) = LAST_PAINTED_FRAME.lock() {
            if let Some(old_img) = guard.replace(new_img) {
                if old_img.id != guard.as_ref().map(|g| g.id).unwrap_or(old_img.id) {
                    let _ = window.drop_image(old_img);
                }
            }
        }
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
    let projector = state.camera.projector(view_w, view_h);
    let mut stroke_path = PathBuilder::stroke(px(1.0));
    let mut line_count = 0;

    for (mesh_idx, mesh) in state.meshes.iter().enumerate() {
        if let Some(item) = state.mesh_items.get(mesh_idx) {
            if !item.visible {
                continue;
            }
        }

        let b_min = [mesh.bounding_box.min().x, mesh.bounding_box.min().y, mesh.bounding_box.min().z];
        let b_max = [mesh.bounding_box.max().x, mesh.bounding_box.max().y, mesh.bounding_box.max().z];
        if projector.is_all_behind_camera(b_min, b_max) {
            continue;
        }

        let v_len = mesh.positions.len();
        let mut proj_verts: Vec<Option<([f32; 2], f32)>> = Vec::with_capacity(v_len);
        for p in &mesh.positions {
            proj_verts.push(projector.project([p.x, p.y, p.z]));
        }

        for chunk in mesh.indices.chunks_exact(3) {
            let (i0, i1, i2) = (chunk[0] as usize, chunk[1] as usize, chunk[2] as usize);
            if i0 >= v_len || i1 >= v_len || i2 >= v_len {
                continue;
            }

            if let (Some((s0, _)), Some((s1, _)), Some((s2, _))) =
                (proj_verts[i0], proj_verts[i1], proj_verts[i2])
            {
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
    let projector = state.camera.projector(view_w, view_h);
    let mut stroke_path = PathBuilder::stroke(px(0.8));
    let mut line_count = 0;

    for (mesh_idx, mesh) in state.meshes.iter().enumerate() {
        if let Some(item) = state.mesh_items.get(mesh_idx) {
            if !item.visible {
                continue;
            }
        }

        let b_min = [mesh.bounding_box.min().x, mesh.bounding_box.min().y, mesh.bounding_box.min().z];
        let b_max = [mesh.bounding_box.max().x, mesh.bounding_box.max().y, mesh.bounding_box.max().z];
        if projector.is_all_behind_camera(b_min, b_max) {
            continue;
        }

        let v_len = mesh.positions.len();
        let mut proj_verts: Vec<Option<([f32; 2], f32)>> = Vec::with_capacity(v_len);
        for p in &mesh.positions {
            proj_verts.push(projector.project([p.x, p.y, p.z]));
        }

        for chunk in mesh.indices.chunks_exact(3) {
            let (i0, i1, i2) = (chunk[0] as usize, chunk[1] as usize, chunk[2] as usize);
            if i0 >= v_len || i1 >= v_len || i2 >= v_len {
                continue;
            }

            if let (Some((s0, _)), Some((s1, _)), Some((s2, _))) =
                (proj_verts[i0], proj_verts[i1], proj_verts[i2])
            {
                let cross = (s1[0] - s0[0]) * (s2[1] - s0[1]) - (s1[1] - s0[1]) * (s2[0] - s0[0]);
                if cross.abs() > 0.001 {
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

