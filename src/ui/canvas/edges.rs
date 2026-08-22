use gpui::*;
use crate::model::{NodeKind, PortDirection};
use crate::ui::state::{AppState, DragState};

pub fn render_edges_layer(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pan = state.pan;
    let zoom = state.zoom;
    let knife_mode = state.knife_mode;

    // 收集所有连线坐标与类型 (EntryPoint / Normal / Join 虚线)
    let mut edge_items = Vec::new();
    for edge in &state.graph.edges {
        if let (Some(s_node), Some(t_node)) = (
            state.graph.find_node(&edge.source_node_id),
            state.graph.find_node(&edge.target_node_id),
        ) {
            let s_offset = s_node.get_port_offset(&edge.source_port_id, PortDirection::Output);
            let t_offset = t_node.get_port_offset(&edge.target_port_id, PortDirection::Input);

            let sx = s_node.pos.0 + s_offset.0;
            let sy = s_node.pos.1 + s_offset.1;
            let ex = t_node.pos.0 + t_offset.0;
            let ey = t_node.pos.1 + t_offset.1;

            let is_entry_point = s_node.kind == NodeKind::EntryPoint;
            let is_dashed = edge.is_dashed || t_node.kind == NodeKind::Join;

            edge_items.push((
                edge.source_node_id.clone(),
                edge.target_node_id.clone(),
                edge.label.clone(),
                is_entry_point,
                is_dashed,
                (sx, sy),
                (ex, ey),
            ));
        }
    }

    let active_node_id = state.selected_node_id.as_deref().or(state.hovered_node_id.as_deref());
    let active_edges = active_node_id.map(|id| state.graph.get_active_path_edges(id)).unwrap_or_default();
    let is_tracing = active_node_id.is_some() && !active_edges.is_empty();

    let live_connection = if let DragState::Connecting { source_pos, current_mouse, .. } = &state.drag_state {
        let (canvas_mx, canvas_my) = state.screen_to_canvas(current_mouse.0, current_mouse.1);
        Some((*source_pos, (canvas_mx, canvas_my)))
    } else {
        None
    };

    let edge_items_clone = edge_items.clone();

    div()
        .absolute()
        .inset_0()
        // 1. GPU 原生硬件加速批处理 Canvas (支持全图因果链路追踪高亮 Flow Tracing)
        .child(
            canvas(
                |_bounds, _window, _cx| {},
                move |bounds, _, window, _cx| {
                    let orig_x: f32 = bounds.origin.x.into();
                    let orig_y: f32 = bounds.origin.y.into();
                    let view_w: f32 = bounds.size.width.into();
                    let view_h: f32 = bounds.size.height.into();

                    let stroke_w = (1.8 * zoom).clamp(1.2, 3.2);
                    let active_stroke_w = (stroke_w * 1.6).clamp(2.2, 4.5);

                    let entry_color = hsla(160.0 / 360.0, 0.84, 0.39, 1.0); // #10b981
                    let normal_color = if knife_mode {
                        hsla(0.0, 0.85, 0.55, 0.95) // #ef4444
                    } else if is_tracing {
                        hsla(215.0 / 360.0, 0.20, 0.35, 0.30) // 弱化非活动连线
                    } else {
                        hsla(215.0 / 360.0, 0.25, 0.65, 0.90) // #94a3b8
                    };

                    let active_color = hsla(45.0 / 360.0, 0.98, 0.58, 1.0); // 高亮金黄发光

                    let mut batch_active = PathBuilder::stroke(px(active_stroke_w));
                    let mut batch_active_arrows = PathBuilder::fill();

                    let mut batch_entry = PathBuilder::stroke(px(stroke_w * 1.15));
                    let mut batch_normal = PathBuilder::stroke(px(stroke_w));
                    let mut batch_dashed = PathBuilder::stroke(px(stroke_w)).dash_array(&[px(6.0), px(6.0)]);
                    let mut batch_arrows = PathBuilder::fill();

                    let mut has_active = false;
                    let mut has_entry = false;
                    let mut has_normal = false;
                    let mut has_dashed = false;
                    let mut has_arrows = false;

                    let arrow_w = (stroke_w * 3.5).clamp(4.5, 9.0);
                    let arrow_h = (stroke_w * 3.0).clamp(4.0, 8.0);
                    let active_arrow_w = (active_stroke_w * 3.2).clamp(6.0, 11.0);
                    let active_arrow_h = (active_stroke_w * 2.8).clamp(5.0, 10.0);

                    for (s_id, t_id, _, is_entry_point, is_dashed, (sx, sy), (ex, ey)) in &edge_items_clone {
                        let screen_sx = orig_x + pan.0 + sx * zoom;
                        let screen_sy = orig_y + pan.1 + sy * zoom;
                        let screen_ex = orig_x + pan.0 + ex * zoom;
                        let screen_ey = orig_y + pan.1 + ey * zoom;

                        // 快速边界框视口裁剪 (BBox Culling)
                        let min_x = screen_sx.min(screen_ex);
                        let max_x = screen_sx.max(screen_ex);
                        let min_y = screen_sy.min(screen_ey);
                        let max_y = screen_sy.max(screen_ey);

                        if max_x < orig_x - 60.0
                            || min_x > orig_x + view_w + 60.0
                            || max_y < orig_y - 60.0
                            || min_y > orig_y + view_h + 60.0
                        {
                            continue;
                        }

                        let dy = (screen_ey - screen_sy).abs().max(36.0 * zoom);
                        let ctrl1 = (screen_sx, screen_sy + dy * 0.45);
                        let ctrl2 = (screen_ex, screen_ey - dy * 0.45);

                        let is_on_active_path = is_tracing && active_edges.contains(&(s_id.clone(), t_id.clone()));

                        if is_on_active_path {
                            has_active = true;
                            batch_active.move_to(point(px(screen_sx), px(screen_sy)));
                            batch_active.cubic_bezier_to(
                                point(px(screen_ex), px(screen_ey)),
                                point(px(ctrl1.0), px(ctrl1.1)),
                                point(px(ctrl2.0), px(ctrl2.1)),
                            );

                            batch_active_arrows.move_to(point(px(screen_ex), px(screen_ey)));
                            batch_active_arrows.line_to(point(px(screen_ex - active_arrow_w * 0.5), px(screen_ey - active_arrow_h)));
                            batch_active_arrows.line_to(point(px(screen_ex + active_arrow_w * 0.5), px(screen_ey - active_arrow_h)));
                            batch_active_arrows.close();
                        } else if *is_entry_point {
                            has_entry = true;
                            batch_entry.move_to(point(px(screen_sx), px(screen_sy)));
                            batch_entry.cubic_bezier_to(
                                point(px(screen_ex), px(screen_ey)),
                                point(px(ctrl1.0), px(ctrl1.1)),
                                point(px(ctrl2.0), px(ctrl2.1)),
                            );

                            has_arrows = true;
                            batch_arrows.move_to(point(px(screen_ex), px(screen_ey)));
                            batch_arrows.line_to(point(px(screen_ex - arrow_w * 0.5), px(screen_ey - arrow_h)));
                            batch_arrows.line_to(point(px(screen_ex + arrow_w * 0.5), px(screen_ey - arrow_h)));
                            batch_arrows.close();
                        } else if *is_dashed {
                            has_dashed = true;
                            batch_dashed.move_to(point(px(screen_sx), px(screen_sy)));
                            batch_dashed.cubic_bezier_to(
                                point(px(screen_ex), px(screen_ey)),
                                point(px(ctrl1.0), px(ctrl1.1)),
                                point(px(ctrl2.0), px(ctrl2.1)),
                            );

                            has_arrows = true;
                            batch_arrows.move_to(point(px(screen_ex), px(screen_ey)));
                            batch_arrows.line_to(point(px(screen_ex - arrow_w * 0.5), px(screen_ey - arrow_h)));
                            batch_arrows.line_to(point(px(screen_ex + arrow_w * 0.5), px(screen_ey - arrow_h)));
                            batch_arrows.close();
                        } else {
                            has_normal = true;
                            batch_normal.move_to(point(px(screen_sx), px(screen_sy)));
                            batch_normal.cubic_bezier_to(
                                point(px(screen_ex), px(screen_ey)),
                                point(px(ctrl1.0), px(ctrl1.1)),
                                point(px(ctrl2.0), px(ctrl2.1)),
                            );

                            has_arrows = true;
                            batch_arrows.move_to(point(px(screen_ex), px(screen_ey)));
                            batch_arrows.line_to(point(px(screen_ex - arrow_w * 0.5), px(screen_ey - arrow_h)));
                            batch_arrows.line_to(point(px(screen_ex + arrow_w * 0.5), px(screen_ey - arrow_h)));
                            batch_arrows.close();
                        }
                    }

                    // 单次提交 GPU 批量路径渲染
                    if has_normal {
                        if let Ok(p) = batch_normal.build() {
                            window.paint_path(p, normal_color);
                        }
                    }
                    if has_entry {
                        if let Ok(p) = batch_entry.build() {
                            let c = if is_tracing { hsla(160.0 / 360.0, 0.84, 0.39, 0.35) } else { entry_color };
                            window.paint_path(p, c);
                        }
                    }
                    if has_dashed {
                        if let Ok(p) = batch_dashed.build() {
                            window.paint_path(p, normal_color);
                        }
                    }
                    if has_arrows {
                        if let Ok(p) = batch_arrows.build() {
                            window.paint_path(p, normal_color);
                        }
                    }
                    // 高亮活动链路置顶渲染
                    if has_active {
                        if let Ok(p) = batch_active.build() {
                            window.paint_path(p, active_color);
                        }
                        if let Ok(p) = batch_active_arrows.build() {
                            window.paint_path(p, active_color);
                        }
                    }


                    // 实时拖拽连线
                    if let Some((start_pt, end_pt)) = live_connection {
                        let screen_sx = orig_x + pan.0 + start_pt.0 * zoom;
                        let screen_sy = orig_y + pan.1 + start_pt.1 * zoom;
                        let screen_ex = orig_x + pan.0 + end_pt.0 * zoom;
                        let screen_ey = orig_y + pan.1 + end_pt.1 * zoom;

                        let dy = (screen_ey - screen_sy).abs().max(36.0 * zoom);
                        let ctrl1 = (screen_sx, screen_sy + dy * 0.45);
                        let ctrl2 = (screen_ex, screen_ey - dy * 0.45);

                        let mut live_builder = PathBuilder::stroke(px(stroke_w * 1.2));
                        live_builder.move_to(point(px(screen_sx), px(screen_sy)));
                        live_builder.cubic_bezier_to(
                            point(px(screen_ex), px(screen_ey)),
                            point(px(ctrl1.0), px(ctrl1.1)),
                            point(px(ctrl2.0), px(ctrl2.1)),
                        );

                        if let Ok(p) = live_builder.build() {
                            window.paint_path(p, hsla(45.0 / 360.0, 0.95, 0.60, 0.95));
                        }
                    }
                },
            )
            .size_full(),
        )
        // 2. 连线交互徽章（切刀剪断触发器）
        .children(edge_items.into_iter().filter_map(move |(s_id, t_id, label, _, _, (sx, sy), (ex, ey))| {
            if !knife_mode {
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

