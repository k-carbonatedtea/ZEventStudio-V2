use gpui::*;
use crate::model::{NodeKind, PortDirection};
use crate::ui::state::{AppState, DragState};

/// 使用 GPUI 原生 GPU 硬件加速 Lyon 矢量路径引擎绘制平滑贝塞尔连线与原生虚线
fn draw_gpu_bezier_edge(
    start: (f32, f32),
    end: (f32, f32),
    color: Hsla,
    stroke_width: f32,
    is_dashed: bool,
    window: &mut Window,
) {
    let dy = (end.1 - start.1).abs().max(36.0);
    let ctrl1 = (start.0, start.1 + dy * 0.45);
    let ctrl2 = (end.0, end.1 - dy * 0.45);

    let mut builder = PathBuilder::stroke(px(stroke_width));
    if is_dashed {
        builder = builder.dash_array(&[px(6.0), px(6.0)]);
    }

    builder.move_to(point(px(start.0), px(start.1)));
    builder.cubic_bezier_to(
        point(px(end.0), px(end.1)),
        point(px(ctrl1.0), px(ctrl1.1)),
        point(px(ctrl2.0), px(ctrl2.1)),
    );

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }

    // 终点闭合向下三角箭头 (使用 GPU Fill 路径单次绘制)
    let arrow_w = (stroke_width * 3.5).clamp(4.5, 9.0);
    let arrow_h = (stroke_width * 3.0).clamp(4.0, 8.0);

    let mut arrow_builder = PathBuilder::fill();
    arrow_builder.move_to(point(px(end.0), px(end.1)));
    arrow_builder.line_to(point(px(end.0 - arrow_w * 0.5), px(end.1 - arrow_h)));
    arrow_builder.line_to(point(px(end.0 + arrow_w * 0.5), px(end.1 - arrow_h)));
    arrow_builder.close();

    if let Ok(arrow_path) = arrow_builder.build() {
        window.paint_path(arrow_path, color);
    }
}

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

    let live_connection = if let DragState::Connecting { source_pos, current_mouse, .. } = &state.drag_state {
        let (canvas_mx, canvas_my) = state.screen_to_canvas(current_mouse.0, current_mouse.1);
        Some((*source_pos, (canvas_mx, canvas_my)))
    } else {
        None
    };

    let model_bounds = model.clone();
    let edge_items_clone = edge_items.clone();

    div()
        .absolute()
        .inset_0()
        // 1. GPU 原生硬件加速 Canvas 绘制所有平滑贝塞尔曲线
        .child(
            canvas(
                move |bounds, _window, cx| {
                    let orig_x: f32 = bounds.origin.x.into();
                    let orig_y: f32 = bounds.origin.y.into();
                    model_bounds.update(cx, |st, _| {
                        st.canvas_origin = (orig_x, orig_y);
                    });
                },
                move |bounds, _, window, _cx| {
                    let orig_x: f32 = bounds.origin.x.into();
                    let orig_y: f32 = bounds.origin.y.into();

                    let stroke_w = (1.8 * zoom).clamp(1.2, 3.2);

                    // 参考图2规范：
                    // - EntryPoint 起始连线: 翡翠绿 (#10b981)
                    // - 并行汇合连线 (连接到 Join 节点): 原生 GPU 虚线 (#94a3b8)
                    // - 普通/分支事件连线: 石板灰蓝 (#94a3b8)
                    // - 切刀模式: 警示红 (#ef4444)
                    for (_, _, _, is_entry_point, is_dashed, (sx, sy), (ex, ey)) in &edge_items_clone {
                        let screen_sx = orig_x + pan.0 + sx * zoom;
                        let screen_sy = orig_y + pan.1 + sy * zoom;
                        let screen_ex = orig_x + pan.0 + ex * zoom;
                        let screen_ey = orig_y + pan.1 + ey * zoom;

                        let edge_color = if knife_mode {
                            hsla(0.0, 0.85, 0.55, 0.95) // #ef4444
                        } else if *is_entry_point {
                            hsla(160.0 / 360.0, 0.84, 0.39, 1.0) // #10b981
                        } else {
                            hsla(215.0 / 360.0, 0.25, 0.65, 0.90) // #94a3b8
                        };

                        draw_gpu_bezier_edge(
                            (screen_sx, screen_sy),
                            (screen_ex, screen_ey),
                            edge_color,
                            if *is_entry_point { stroke_w * 1.15 } else { stroke_w },
                            *is_dashed,
                            window,
                        );
                    }

                    // 实时拖拽连线
                    if let Some((start_pt, end_pt)) = live_connection {
                        let screen_sx = orig_x + pan.0 + start_pt.0 * zoom;
                        let screen_sy = orig_y + pan.1 + start_pt.1 * zoom;
                        let screen_ex = orig_x + pan.0 + end_pt.0 * zoom;
                        let screen_ey = orig_y + pan.1 + end_pt.1 * zoom;

                        let live_color = hsla(45.0 / 360.0, 0.95, 0.60, 0.95);
                        draw_gpu_bezier_edge(
                            (screen_sx, screen_sy),
                            (screen_ex, screen_ey),
                            live_color,
                            stroke_w * 1.2,
                            false,
                            window,
                        );
                    }
                },
            )
            .size_full(),
        )
        // 2. 连线交互徽章（Switch 条件分支标签与切刀剪断触发器）
        .children(edge_items.into_iter().filter_map(move |(s_id, t_id, label, _, _, (sx, sy), (ex, ey))| {
            if !knife_mode && label.is_none() {
                return None;
            }

            let mid_x = pan.0 + ((sx + ex) / 2.0) * zoom;
            let mid_y = pan.1 + ((sy + ey) / 2.0) * zoom;
            let model_cut = model.clone();

            Some(
                div()
                    .absolute()
                    .left(px(mid_x - 28.0))
                    .top(px(mid_y - 10.0))
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(if knife_mode {
                        hsla(0.0, 0.8, 0.35, 1.0)
                    } else {
                        hsla(240.0 / 360.0, 0.06, 0.12, 0.92)
                    })
                    .border_1()
                    .border_color(if knife_mode {
                        hsla(0.0, 0.8, 0.65, 1.0)
                    } else {
                        hsla(215.0 / 360.0, 0.25, 0.35, 0.8)
                    })
                    .shadow_sm()
                    .cursor_pointer()
                    .hover(|s| {
                        if knife_mode {
                            s.bg(hsla(0.0, 0.9, 0.45, 1.0))
                        } else {
                            s.bg(hsla(240.0 / 360.0, 0.06, 0.18, 1.0))
                        }
                    })
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        if knife_mode {
                            let s = s_id.clone();
                            let t = t_id.clone();
                            model_cut.update(cx, |state, cx| {
                                state.sever_edge(&s, &t);
                                cx.notify();
                            });
                        }
                    })
                    .child(
                        div()
                            .text_size(px(9.5 * zoom.clamp(0.75, 1.1)))
                            .font_weight(FontWeight::BOLD)
                            .text_color(if knife_mode {
                                hsla(0.0, 0.0, 1.0, 1.0)
                            } else {
                                hsla(215.0 / 360.0, 0.25, 0.80, 1.0)
                            })
                            .child(if knife_mode {
                                "✂ 剪断".to_string()
                            } else {
                                label.unwrap_or_default()
                            }),
                    ),
            )
        }))
}
