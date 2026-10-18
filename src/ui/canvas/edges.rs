use gpui::*;
use super::edge_geometry::{draw_orthogonal_step_line, extract_edge_items};
use crate::model::node::NodeKind;
use crate::ui::state::AppState;
use std::collections::HashSet;

/// 渲染高性能批量连线图层 (采用现代正交折线/曼哈顿走廊拓扑与 GPU 批量渲染)
pub fn render_edges_layer(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pan = state.pan;
    let zoom = state.zoom;
    let knife_mode = state.knife_mode;
    let connecting_state = match &state.drag_state {
        crate::ui::state::DragState::Connecting {
            source_pos,
            current_mouse,
            direction,
            ..
        } => Some((*source_pos, *current_mouse, *direction)),
        _ => None,
    };

    // 1. 提取所有已连接连线的几何端点与属性
    let edge_items = extract_edge_items(&state.graph);

    // 2. 识别高亮活动链路 (Active Path Tracing)
    let active_edges = if let Some(selected_id) = &state.selected_node_id {
        state.graph.get_active_path_edges(selected_id)
    } else {
        HashSet::new()
    };
    let is_tracing = !active_edges.is_empty();

    let edge_items_clone = edge_items.clone();
    let live_connection = connecting_state;

    div()
        .absolute()
        .inset_0()
        // 1. 核心连线批量路径绘制容器 (GPU Batch Path Painter)
        .child(
            canvas(
                move |bounds, _, _| bounds,
                move |bounds, _, window, _| {
                    let orig_x: f32 = bounds.origin.x.into();
                    let orig_y: f32 = bounds.origin.y.into();
                    let view_w: f32 = bounds.size.width.into();
                    let view_h: f32 = bounds.size.height.into();

                    let stroke_w = (1.8 * zoom).clamp(0.8, 3.2);
                    let active_stroke_w = (stroke_w * 1.6).clamp(1.4, 4.5);

                    let active_color = hsla(45.0 / 360.0, 0.98, 0.58, 1.0); // 高亮金黄发光

                    let mut batch_active = PathBuilder::stroke(px(active_stroke_w));
                    let mut batch_active_arrows = PathBuilder::fill();
                    let mut has_active = false;

                    let mut batch_dashed = PathBuilder::stroke(px(stroke_w)).dash_array(&[px(6.0), px(6.0)]);
                    let mut batch_dashed_arrows = PathBuilder::fill();
                    let mut has_dashed = false;

                    // 为每种 NodeKind 建立独立的批量构建器
                    let mut kind_paths = vec![
                        (NodeKind::EntryPoint, PathBuilder::stroke(px(stroke_w * 1.15)), PathBuilder::fill(), false),
                        (NodeKind::Action, PathBuilder::stroke(px(stroke_w)), PathBuilder::fill(), false),
                        (NodeKind::Switch, PathBuilder::stroke(px(stroke_w)), PathBuilder::fill(), false),
                        (NodeKind::Query, PathBuilder::stroke(px(stroke_w)), PathBuilder::fill(), false),
                        (NodeKind::SubFlow, PathBuilder::stroke(px(stroke_w)), PathBuilder::fill(), false),
                        (NodeKind::Fork, PathBuilder::stroke(px(stroke_w)), PathBuilder::fill(), false),
                        (NodeKind::Join, PathBuilder::stroke(px(stroke_w)), PathBuilder::fill(), false),
                        (NodeKind::Math, PathBuilder::stroke(px(stroke_w)), PathBuilder::fill(), false),
                        (NodeKind::Comment, PathBuilder::stroke(px(stroke_w)), PathBuilder::fill(), false),
                    ];

                    let arrow_w = (stroke_w * 3.5).clamp(2.0, 9.0);
                    let arrow_h = (stroke_w * 3.0).clamp(1.5, 8.0);
                    let active_arrow_w = (active_stroke_w * 3.2).clamp(3.0, 11.0);
                    let active_arrow_h = (active_stroke_w * 2.8).clamp(2.5, 10.0);

                    for (s_id, t_id, _, s_kind, is_dashed, (sx, sy), (ex, ey), waypoints) in &edge_items_clone {
                        let screen_sx = orig_x + pan.0 + sx * zoom;
                        let screen_sy = orig_y + pan.1 + sy * zoom;
                        let screen_ex = orig_x + pan.0 + ex * zoom;
                        let screen_ey = orig_y + pan.1 + ey * zoom;

                        // 快速边界框视口裁剪 (BBox Culling)
                        let mut min_x = screen_sx.min(screen_ex);
                        let mut max_x = screen_sx.max(screen_ex);
                        let mut min_y = screen_sy.min(screen_ey);
                        let mut max_y = screen_sy.max(screen_ey);

                        let mut screen_pts = Vec::with_capacity(waypoints.len() + 2);
                        screen_pts.push((screen_sx, screen_sy));
                        for (wx, wy) in waypoints {
                            let swx = orig_x + pan.0 + wx * zoom;
                            let swy = orig_y + pan.1 + wy * zoom;
                            min_x = min_x.min(swx);
                            max_x = max_x.max(swx);
                            min_y = min_y.min(swy);
                            max_y = max_y.max(swy);
                            screen_pts.push((swx, swy));
                        }
                        screen_pts.push((screen_ex, screen_ey));

                        if max_x < -100.0 || min_x > view_w + 100.0 || max_y < -100.0 || min_y > view_h + 100.0
                        {
                            continue;
                        }

                        let is_on_active_path = is_tracing && active_edges.contains(&(s_id.clone(), t_id.clone()));

                        if is_on_active_path {
                            has_active = true;
                            draw_orthogonal_step_line(&mut batch_active, &screen_pts, zoom);

                            batch_active_arrows.move_to(point(px(screen_ex), px(screen_ey)));
                            batch_active_arrows.line_to(point(px(screen_ex - active_arrow_w * 0.5), px(screen_ey - active_arrow_h)));
                            batch_active_arrows.line_to(point(px(screen_ex + active_arrow_w * 0.5), px(screen_ey - active_arrow_h)));
                            batch_active_arrows.close();
                        } else if *is_dashed {
                            has_dashed = true;
                            draw_orthogonal_step_line(&mut batch_dashed, &screen_pts, zoom);

                            batch_dashed_arrows.move_to(point(px(screen_ex), px(screen_ey)));
                            batch_dashed_arrows.line_to(point(px(screen_ex - arrow_w * 0.5), px(screen_ey - arrow_h)));
                            batch_dashed_arrows.line_to(point(px(screen_ex + arrow_w * 0.5), px(screen_ey - arrow_h)));
                            batch_dashed_arrows.close();
                        } else if let Some((_, stroke_builder, arrow_builder, has_items)) = kind_paths.iter_mut().find(|(k, _, _, _)| k == s_kind) {
                            *has_items = true;
                            draw_orthogonal_step_line(stroke_builder, &screen_pts, zoom);

                            arrow_builder.move_to(point(px(screen_ex), px(screen_ey)));
                            arrow_builder.line_to(point(px(screen_ex - arrow_w * 0.5), px(screen_ey - arrow_h)));
                            arrow_builder.line_to(point(px(screen_ex + arrow_w * 0.5), px(screen_ey - arrow_h)));
                            arrow_builder.close();
                        }
                    }

                    // 单次提交 GPU 批量路径渲染：按不同节点类型专属颜色渲染
                    for (kind, stroke_builder, arrow_builder, has_items) in kind_paths {
                        if !has_items {
                            continue;
                        }

                        let color = if knife_mode {
                            hsla(0.0, 0.85, 0.55, 0.95) // 切刀剪断红色
                        } else if is_tracing {
                            let mut c = kind.edge_color();
                            c.a = 0.25; // 链路追踪时弱化非活动连线
                            c
                        } else {
                            kind.edge_color()
                        };

                        if let Ok(p) = stroke_builder.build() {
                            window.paint_path(p, color);
                        }
                        if let Ok(p) = arrow_builder.build() {
                            window.paint_path(p, color);
                        }
                    }

                    // 虚线汇合连线渲染
                    if has_dashed {
                        let color = if knife_mode {
                            hsla(0.0, 0.85, 0.55, 0.95)
                        } else if is_tracing {
                            let mut c = NodeKind::Join.edge_color();
                            c.a = 0.25;
                            c
                        } else {
                            NodeKind::Join.edge_color()
                        };

                        if let Ok(p) = batch_dashed.build() {
                            window.paint_path(p, color);
                        }
                        if let Ok(p) = batch_dashed_arrows.build() {
                            window.paint_path(p, color);
                        }
                    }

                    // 高亮活动链路置顶发光渲染
                    if has_active {
                        if let Ok(p) = batch_active.build() {
                            window.paint_path(p, active_color);
                        }
                        if let Ok(p) = batch_active_arrows.build() {
                            window.paint_path(p, active_color);
                        }
                    }

                    // 实时拖拽连线 (正交阶梯折线跟随鼠标)
                    if let Some((source_pos, current_mouse, direction)) = live_connection {
                        let src_sx = orig_x + pan.0 + source_pos.0 * zoom;
                        let src_sy = orig_y + pan.1 + source_pos.1 * zoom;
                        let mouse_sx = current_mouse.0;
                        let mouse_sy = current_mouse.1;

                        let (screen_sx, screen_sy, screen_ex, screen_ey) = match direction {
                            crate::model::PortDirection::Output => (src_sx, src_sy, mouse_sx, mouse_sy),
                            crate::model::PortDirection::Input => (mouse_sx, mouse_sy, src_sx, src_sy),
                        };

                        let mut live_builder = PathBuilder::stroke(px(stroke_w * 1.35));
                        draw_orthogonal_step_line(&mut live_builder, &[(screen_sx, screen_sy), (screen_ex, screen_ey)], zoom);

                        if let Ok(p) = live_builder.build() {
                            window.paint_path(p, hsla(45.0 / 360.0, 0.98, 0.60, 0.98));
                        }

                        // 鼠标光标处高亮跟随端点小圆球
                        let (tip_x, tip_y) = if direction == crate::model::PortDirection::Output {
                            (screen_ex, screen_ey)
                        } else {
                            (screen_sx, screen_sy)
                        };
                        let r = (4.5 * zoom).clamp(3.5, 7.0);
                        let mut tip_builder = PathBuilder::fill();
                        tip_builder.move_to(point(px(tip_x - r), px(tip_y)));
                        tip_builder.line_to(point(px(tip_x), px(tip_y - r)));
                        tip_builder.line_to(point(px(tip_x + r), px(tip_y)));
                        tip_builder.line_to(point(px(tip_x), px(tip_y + r)));
                        tip_builder.close();
                        if let Ok(p) = tip_builder.build() {
                            window.paint_path(p, hsla(45.0 / 360.0, 0.98, 0.65, 1.0));
                        }
                    }
                },
            )
            .size_full(),
        )

        // 2. 连线交互徽章（切刀剪断触发器）
        .children(edge_items.into_iter().filter_map(move |(s_id, t_id, label, _, _, (sx, sy), (ex, ey), _)| {
            if !knife_mode || zoom < 0.35 {
                return None;
            }

            let mid_x = pan.0 + ((sx + ex) / 2.0) * zoom;
            let mid_y = pan.1 + ((sy + ey) / 2.0) * zoom;
            let model_cut = model.clone();

            let cut_label = label.unwrap_or_else(|| state.t("edge.cut").to_string());

            Some(
                div()
                    .absolute()
                    .left(px(mid_x - 28.0))
                    .top(px(mid_y - 10.0))
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(hsla(0.0, 0.8, 0.35, 1.0))
                    .border_1()
                    .border_color(hsla(0.0, 0.8, 0.65, 1.0))
                    .shadow_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(hsla(0.0, 0.9, 0.45, 1.0)))
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        let s = s_id.clone();
                        let t = t_id.clone();
                        model_cut.update(cx, |state, cx| {
                            state.sever_edge(&s, &t);
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .text_size(px(9.5 * zoom.clamp(0.75, 1.1)))
                            .font_weight(FontWeight::BOLD)
                            .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                            .child(cut_label),
                    ),
            )
        }))
}
