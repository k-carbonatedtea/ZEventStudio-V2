use gpui::*;
use crate::model::{NodeKind, PortDirection};
use crate::ui::state::{AppState, DragState};

fn eval_cubic_bezier(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    t: f32,
) -> (f32, f32) {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    let x = uuu * p0.0 + 3.0 * uu * t * p1.0 + 3.0 * u * tt * p2.0 + ttt * p3.0;
    let y = uuu * p0.1 + 3.0 * uu * t * p1.1 + 3.0 * u * tt * p2.1 + ttt * p3.1;
    (x, y)
}

/// 绘制基于图2规范的垂直平滑贝塞尔曲线（支持实线与汇合虚线 Dashed Curve）
fn draw_bezier_edge(
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

    let segments = if is_dashed { 32 } else { 24 };
    let mut prev_pt = start;

    for i in 1..=segments {
        let t = (i as f32) / (segments as f32);
        let curr_pt = eval_cubic_bezier(start, ctrl1, ctrl2, end, t);

        // 虚线模式：每 4 段跳过 2 段形成虚线效果 (对应 React Flow strokeDasharray: 5,5)
        if is_dashed && (i % 4 >= 2) {
            prev_pt = curr_pt;
            continue;
        }

        let min_x = prev_pt.0.min(curr_pt.0);
        let min_y = prev_pt.1.min(curr_pt.1);
        let max_x = prev_pt.0.max(curr_pt.0) + stroke_width;
        let max_y = prev_pt.1.max(curr_pt.1) + stroke_width;

        window.paint_quad(fill(
            Bounds {
                origin: Point {
                    x: px(min_x - stroke_width / 2.0),
                    y: px(min_y - stroke_width / 2.0),
                },
                size: size(px(max_x - min_x + stroke_width), px(max_y - min_y + stroke_width)),
            },
            color,
        ));

        prev_pt = curr_pt;
    }

    // 终点闭合三角箭头 (Closed Arrowhead)
    let arrow_w = (stroke_width * 3.5).clamp(4.0, 10.0);
    let arrow_h = (stroke_width * 3.0).clamp(3.5, 9.0);
    window.paint_quad(fill(
        Bounds {
            origin: Point {
                x: px(end.0 - arrow_w * 0.5),
                y: px(end.1 - arrow_h),
            },
            size: size(px(arrow_w), px(arrow_h)),
        },
        color,
    ));
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
            let is_join_target = t_node.kind == NodeKind::Join;

            edge_items.push((
                edge.source_node_id.clone(),
                edge.target_node_id.clone(),
                edge.label.clone(),
                is_entry_point,
                is_join_target,
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
        // 1. 静态 Canvas 绘制所有平滑贝塞尔曲线
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

                    let stroke_w = (2.0 * zoom).clamp(1.2, 3.5);

                    // 参考图2规范：
                    // - EntryPoint 起始连线: 翡翠绿 (#10b981)
                    // - 并行汇合连线 (连接到 Join 节点): 虚线 (#94a3b8)
                    // - 普通/分支事件连线: 石板灰蓝 (#94a3b8)
                    // - 切刀模式: 警示红 (#ef4444)
                    for (_, _, _, is_entry_point, is_join_target, (sx, sy), (ex, ey)) in &edge_items_clone {
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

                        draw_bezier_edge(
                            (screen_sx, screen_sy),
                            (screen_ex, screen_ey),
                            edge_color,
                            if *is_entry_point { stroke_w * 1.15 } else { stroke_w },
                            *is_join_target,
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
                        draw_bezier_edge(
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
