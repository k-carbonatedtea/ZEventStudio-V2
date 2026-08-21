use gpui::*;
use crate::model::PortDirection;
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

fn draw_bezier_curve(
    start: (f32, f32),
    end: (f32, f32),
    color: Hsla,
    stroke_width: f32,
    window: &mut Window,
) {
    let dx = (end.0 - start.0).abs().max(40.0);
    let ctrl1 = (start.0 + dx * 0.45, start.1);
    let ctrl2 = (end.0 - dx * 0.45, end.1);

    let segments = 24;
    let mut prev_pt = start;

    for i in 1..=segments {
        let t = (i as f32) / (segments as f32);
        let curr_pt = eval_cubic_bezier(start, ctrl1, ctrl2, end, t);

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

    // Draw end arrow head
    let arrow_size = (stroke_width * 3.0).max(6.0);
    window.paint_quad(fill(
        Bounds {
            origin: Point {
                x: px(end.0 - arrow_size * 0.7),
                y: px(end.1 - arrow_size * 0.5),
            },
            size: size(px(arrow_size), px(arrow_size)),
        },
        color,
    ));
}

fn draw_flowing_particles(
    start: (f32, f32),
    end: (f32, f32),
    particle_color: Hsla,
    particle_size: f32,
    window: &mut Window,
) {
    let dx = (end.0 - start.0).abs().max(40.0);
    let ctrl1 = (start.0 + dx * 0.45, start.1);
    let ctrl2 = (end.0 - dx * 0.45, end.1);

    // Calculate phase based on system time for smooth 60fps flowing animation
    let time_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f32;

    let base_phase = (time_ms * 0.0012) % 1.0;

    let particle_count = 3;
    for i in 0..particle_count {
        let t = (base_phase + (i as f32) / (particle_count as f32)) % 1.0;
        let pt = eval_cubic_bezier(start, ctrl1, ctrl2, end, t);

        // Core glowing dot
        window.paint_quad(fill(
            Bounds {
                origin: Point {
                    x: px(pt.0 - particle_size / 2.0),
                    y: px(pt.1 - particle_size / 2.0),
                },
                size: size(px(particle_size), px(particle_size)),
            },
            particle_color,
        ));

        // Outer aura
        let aura_size = particle_size * 2.0;
        let mut aura_color = particle_color;
        aura_color.a *= 0.35;
        window.paint_quad(fill(
            Bounds {
                origin: Point {
                    x: px(pt.0 - aura_size / 2.0),
                    y: px(pt.1 - aura_size / 2.0),
                },
                size: size(px(aura_size), px(aura_size)),
            },
            aura_color,
        ));
    }
}

pub fn render_edges_layer(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pan = state.pan;
    let zoom = state.zoom;
    let knife_mode = state.knife_mode;
    let particles_enabled = state.particles_enabled;

    // Collect all edge coordinates
    let mut edge_coords = Vec::new();
    for edge in &state.graph.edges {
        if let (Some(s_node), Some(t_node)) = (
            state.graph.find_node(&edge.source_node_id),
            state.graph.find_node(&edge.target_node_id),
        ) {
            let s_offset_y = s_node.get_port_offset_y(&edge.source_port_id, PortDirection::Output);
            let t_offset_y = t_node.get_port_offset_y(&edge.target_port_id, PortDirection::Input);

            let sx = s_node.pos.0 + s_node.size.0;
            let sy = s_node.pos.1 + s_offset_y;
            let ex = t_node.pos.0;
            let ey = t_node.pos.1 + t_offset_y;

            edge_coords.push((
                edge.source_node_id.clone(),
                edge.target_node_id.clone(),
                edge.label.clone(),
                (sx, sy),
                (ex, ey),
            ));
        }
    }

    // Live connecting wire if currently dragging connection
    let live_connection = if let DragState::Connecting { source_pos, current_mouse, .. } = &state.drag_state {
        let (canvas_mx, canvas_my) = state.screen_to_canvas(current_mouse.0, current_mouse.1);
        Some((*source_pos, (canvas_mx, canvas_my)))
    } else {
        None
    };

    let model_bounds = model.clone();
    let edge_coords_clone = edge_coords.clone();

    div()
        .absolute()
        .inset_0()
        // Draw background bezier curves using GPUI canvas
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

                    let default_edge_color = if knife_mode {
                        hsla(0.0, 0.75, 0.55, 0.9)
                    } else {
                        hsla(215.0 / 360.0, 0.35, 0.50, 0.8)
                    };

                    let stroke_w = (2.2 * zoom).clamp(1.5, 4.0);
                    let particle_size = (5.5 * zoom).clamp(3.5, 9.0);
                    let particle_color = hsla(145.0 / 360.0, 0.85, 0.65, 1.0);

                    // 1. Draw static edges
                    for (_, _, _, (sx, sy), (ex, ey)) in &edge_coords_clone {
                        let screen_sx = orig_x + pan.0 + sx * zoom;
                        let screen_sy = orig_y + pan.1 + sy * zoom;
                        let screen_ex = orig_x + pan.0 + ex * zoom;
                        let screen_ey = orig_y + pan.1 + ey * zoom;

                        draw_bezier_curve(
                            (screen_sx, screen_sy),
                            (screen_ex, screen_ey),
                            default_edge_color,
                            stroke_w,
                            window,
                        );

                        if particles_enabled && !knife_mode {
                            draw_flowing_particles(
                                (screen_sx, screen_sy),
                                (screen_ex, screen_ey),
                                particle_color,
                                particle_size,
                                window,
                            );
                        }
                    }

                    // 2. Draw live dragging wire
                    if let Some((start_pt, end_pt)) = live_connection {
                        let screen_sx = orig_x + pan.0 + start_pt.0 * zoom;
                        let screen_sy = orig_y + pan.1 + start_pt.1 * zoom;
                        let screen_ex = orig_x + pan.0 + end_pt.0 * zoom;
                        let screen_ey = orig_y + pan.1 + end_pt.1 * zoom;

                        let live_color = hsla(45.0 / 360.0, 0.95, 0.60, 0.95);
                        draw_bezier_curve(
                            (screen_sx, screen_sy),
                            (screen_ex, screen_ey),
                            live_color,
                            stroke_w * 1.3,
                            window,
                        );
                    }
                },
            )
            .size_full(),
        )
        // Edge interactive badges (Branch labels and Knife cut triggers)
        .children(edge_coords.into_iter().filter_map(move |(s_id, t_id, label, (sx, sy), (ex, ey))| {
            // Only render badge if knife mode is active or edge has a label
            if !knife_mode && label.is_none() {
                return None;
            }

            let mid_x = pan.0 + ((sx + ex) / 2.0) * zoom;
            let mid_y = pan.1 + ((sy + ey) / 2.0) * zoom;
            let model_cut = model.clone();

            Some(
                div()
                    .absolute()
                    .left(px(mid_x - 30.0))
                    .top(px(mid_y - 11.0))
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(if knife_mode {
                        hsla(0.0, 0.8, 0.35, 1.0)
                    } else {
                        hsla(220.0 / 360.0, 0.35, 0.14, 0.92)
                    })
                    .border_1()
                    .border_color(if knife_mode {
                        hsla(0.0, 0.8, 0.65, 1.0)
                    } else {
                        hsla(215.0 / 360.0, 0.35, 0.35, 1.0)
                    })
                    .shadow_sm()
                    .cursor_pointer()
                    .hover(|s| {
                        if knife_mode {
                            s.bg(hsla(0.0, 0.9, 0.45, 1.0))
                        } else {
                            s.bg(hsla(220.0 / 360.0, 0.35, 0.22, 1.0))
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
                            .text_size(px(10.0 * zoom.clamp(0.75, 1.1)))
                            .font_weight(FontWeight::BOLD)
                            .text_color(if knife_mode {
                                hsla(0.0, 0.0, 1.0, 1.0)
                            } else {
                                hsla(45.0 / 360.0, 0.9, 0.75, 1.0)
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
