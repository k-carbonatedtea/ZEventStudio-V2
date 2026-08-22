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

    // 收集所有连线坐标、源节点类型与虚线状态
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

            let is_dashed = edge.is_dashed || t_node.kind == NodeKind::Join;

            edge_items.push((
                edge.source_node_id.clone(),
                edge.target_node_id.clone(),
                edge.label.clone(),
                s_node.kind.clone(),
                is_dashed,
                (sx, sy),
                (ex, ey),
                edge.waypoints.clone(),
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
        // 1. GPU 原生硬件加速批处理 Canvas (支持航点走廊路由 Waypoint Corridor Routing)
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

                    let arrow_w = (stroke_w * 3.5).clamp(4.5, 9.0);
                    let arrow_h = (stroke_w * 3.0).clamp(4.0, 8.0);
                    let active_arrow_w = (active_stroke_w * 3.2).clamp(6.0, 11.0);
                    let active_arrow_h = (active_stroke_w * 2.8).clamp(5.0, 10.0);

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

                        if max_x < orig_x - 60.0
                            || min_x > orig_x + view_w + 60.0
                            || max_y < orig_y - 60.0
                            || min_y > orig_y + view_h + 60.0
                        {
                            continue;
                        }

                        let is_on_active_path = is_tracing && active_edges.contains(&(s_id.clone(), t_id.clone()));

                        if is_on_active_path {
                            has_active = true;
                            draw_routed_curve(&mut batch_active, &screen_pts, zoom);

                            batch_active_arrows.move_to(point(px(screen_ex), px(screen_ey)));
                            batch_active_arrows.line_to(point(px(screen_ex - active_arrow_w * 0.5), px(screen_ey - active_arrow_h)));
                            batch_active_arrows.line_to(point(px(screen_ex + active_arrow_w * 0.5), px(screen_ey - active_arrow_h)));
                            batch_active_arrows.close();
                        } else if *is_dashed {
                            has_dashed = true;
                            draw_routed_curve(&mut batch_dashed, &screen_pts, zoom);

                            batch_dashed_arrows.move_to(point(px(screen_ex), px(screen_ey)));
                            batch_dashed_arrows.line_to(point(px(screen_ex - arrow_w * 0.5), px(screen_ey - arrow_h)));
                            batch_dashed_arrows.line_to(point(px(screen_ex + arrow_w * 0.5), px(screen_ey - arrow_h)));
                            batch_dashed_arrows.close();
                        } else if let Some((_, stroke_builder, arrow_builder, has_items)) = kind_paths.iter_mut().find(|(k, _, _, _)| k == s_kind) {
                            *has_items = true;
                            draw_routed_curve(stroke_builder, &screen_pts, zoom);

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

                    // 实时拖拽连线
                    if let Some((start_pt, end_pt)) = live_connection {
                        let screen_sx = orig_x + pan.0 + start_pt.0 * zoom;
                        let screen_sy = orig_y + pan.1 + start_pt.1 * zoom;
                        let screen_ex = orig_x + pan.0 + end_pt.0 * zoom;
                        let screen_ey = orig_y + pan.1 + end_pt.1 * zoom;

                        let dy = (screen_ey - screen_sy).abs().max(36.0 * zoom);
                        let ctrl1 = (screen_sx, screen_sy + dy * 0.45);
                        let ctrl2 = (screen_ex, screen_ey - dy * 0.45);

                        let mut live_builder = PathBuilder::stroke(px(stroke_w * 1.25));
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
        .children(edge_items.into_iter().filter_map(move |(s_id, t_id, label, _, _, (sx, sy), (ex, ey), _)| {
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

/// 绘制支持多航点平滑弯曲的走廊连线 (Waypoint Routed Smooth Curve)
fn draw_routed_curve(builder: &mut PathBuilder, pts: &[(f32, f32)], zoom: f32) {
    if pts.is_empty() {
        return;
    }
    builder.move_to(point(px(pts[0].0), px(pts[0].1)));
    if pts.len() == 2 {
        let (sx, sy) = pts[0];
        let (ex, ey) = pts[1];
        let dy = (ey - sy).abs().max(36.0 * zoom);
        builder.cubic_bezier_to(
            point(px(ex), px(ey)),
            point(px(sx), px(sy + dy * 0.45)),
            point(px(ex), px(ey - dy * 0.45)),
        );
    } else {
        for i in 0..pts.len() - 1 {
            let (x1, y1) = pts[i];
            let (x2, y2) = pts[i + 1];
            let dy = (y2 - y1).abs().max(24.0 * zoom);
            builder.cubic_bezier_to(
                point(px(x2), px(y2)),
                point(px(x1), px(y1 + dy * 0.45)),
                point(px(x2), px(y2 - dy * 0.45)),
            );
        }
    }
}


