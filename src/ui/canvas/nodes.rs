use gpui::*;
use crate::model::{NodeData, Port, PortDirection, PortType};
use crate::ui::state::{AppState, DragState};

pub fn render_nodes_layer(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pan = state.pan;
    let zoom = state.zoom;
    let selected_id = state.selected_node_id.clone();
    let connecting_state = state.drag_state.clone();

    div()
        .absolute()
        .inset_0()
        .children(state.graph.nodes.iter().map(|node| {
            render_single_node(node, pan, zoom, &selected_id, &connecting_state, model)
        }))
}

fn render_single_node(
    node: &NodeData,
    pan: (f32, f32),
    zoom: f32,
    selected_id: &Option<String>,
    connecting_state: &DragState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let is_selected = selected_id.as_deref() == Some(&node.id);
    let (h, s, l) = node.kind.header_color();
    let header_bg = hsla(h, s, l, 0.95);
    let header_border = hsla(h, s, (l + 0.15).min(1.0), 1.0);

    let screen_x = pan.0 + node.pos.0 * zoom;
    let screen_y = pan.1 + node.pos.1 * zoom;
    let node_w = node.size.0 * zoom;

    let node_id = node.id.clone();
    let model_header_drag = model.clone();

    div()
        .id(SharedString::from(format!("node_{}", node.id)))
        .absolute()
        .left(px(screen_x))
        .top(px(screen_y))
        .w(px(node_w))
        .rounded_xl()
        .bg(hsla(220.0 / 360.0, 0.35, 0.12, 0.96))
        .border_2()
        .border_color(if is_selected {
            hsla(45.0 / 360.0, 0.95, 0.55, 1.0)
        } else {
            hsla(220.0 / 360.0, 0.25, 0.25, 0.8)
        })
        .shadow_xl()
        .cursor_default()
        // Header
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .rounded_t_lg()
                .bg(header_bg)
                .border_b_1()
                .border_color(header_border)
                .cursor_move()
                .on_mouse_down(MouseButton::Left, move |e, _, cx| {
                    let id = node_id.clone();
                    let mx: f32 = e.position.x.into();
                    let my: f32 = e.position.y.into();
                    model_header_drag.update(cx, |state, cx| {
                        state.selected_node_id = Some(id.clone());
                        let init_positions = state
                            .graph
                            .nodes
                            .iter()
                            .map(|n| (n.id.clone(), n.pos))
                            .collect();
                        state.drag_state = DragState::DragNodes {
                            start_mouse: (mx, my),
                            initial_positions: init_positions,
                        };
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .text_size(px(14.0 * zoom.clamp(0.8, 1.2)))
                                .child(node.kind.icon()),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.0 * zoom.clamp(0.8, 1.2)))
                                .text_color(rgb(0xffffff))
                                .child(node.title.clone()),
                        ),
                )
                .child(
                    div()
                        .text_size(px(9.5 * zoom.clamp(0.8, 1.1)))
                        .px_1p5()
                        .py_0p5()
                        .rounded_sm()
                        .bg(hsla(0.0, 0.0, 0.0, 0.3))
                        .text_color(hsla(0.0, 0.0, 0.95, 0.85))
                        .child(node.kind.display_name()),
                ),
        )
        // Subtitle / info tag
        .children(node.subtitle.as_ref().map(|sub| {
            div()
                .px_3()
                .pt_1()
                .text_size(px(10.0 * zoom.clamp(0.75, 1.1)))
                .text_color(hsla(215.0 / 360.0, 0.25, 0.65, 1.0))
                .child(sub.clone())
        }))
        // Body (Ports & Parameters)
        .child(
            div()
                .p_2()
                .flex()
                .flex_col()
                .gap_2()
                // Ports area
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .gap_2()
                        // Inputs column
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1p5()
                                .items_start()
                                .children(node.inputs.iter().map(|p| {
                                    render_input_port(p, &node.id, zoom, connecting_state, model)
                                })),
                        )
                        // Outputs column
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1p5()
                                .items_end()
                                .children(node.outputs.iter().map(|p| {
                                    render_output_port(p, node, zoom, model)
                                })),
                        ),
                )
                // Properties summary preview
                .children(if !node.properties.is_empty() {
                    Some(
                        div()
                            .mt_1()
                            .pt_1p5()
                            .border_t_1()
                            .border_color(hsla(220.0 / 360.0, 0.2, 0.22, 1.0))
                            .flex()
                            .flex_col()
                            .gap_1()
                            .children(node.properties.iter().take(3).map(|prop| {
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .text_size(px(9.5 * zoom.clamp(0.75, 1.05)))
                                    .child(
                                        div()
                                            .text_color(hsla(215.0 / 360.0, 0.2, 0.55, 1.0))
                                            .child(prop.key.clone()),
                                    )
                                    .child(
                                        div()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(hsla(45.0 / 360.0, 0.8, 0.75, 1.0))
                                            .child(if prop.val.len() > 18 {
                                                format!("{}...", &prop.val[..18])
                                            } else {
                                                prop.val.clone()
                                            }),
                                    )
                            })),
                    )
                } else {
                    None
                }),
        )
}

fn render_input_port(
    port: &Port,
    node_id: &str,
    zoom: f32,
    connecting_state: &DragState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let port_id = port.id.clone();
    let n_id = node_id.to_string();
    let model_connect = model.clone();

    let is_connecting = matches!(connecting_state, DragState::Connecting { .. });
    let port_color = match port.port_type {
        PortType::Flow => hsla(145.0 / 360.0, 0.75, 0.55, 1.0),
        PortType::Condition => hsla(40.0 / 360.0, 0.85, 0.55, 1.0),
        PortType::Data => hsla(200.0 / 360.0, 0.85, 0.60, 1.0),
    };

    div()
        .flex()
        .items_center()
        .gap_1p5()
        .cursor_pointer()
        .hover(|s| s.opacity(0.85))
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            let target_n = n_id.clone();
            let target_p = port_id.clone();
            model_connect.update(cx, |state, cx| {
                if let DragState::Connecting { source_node_id, source_port_id, .. } = &state.drag_state.clone() {
                    state.connect_ports(source_node_id, source_port_id, &target_n, &target_p);
                }
                state.drag_state = DragState::None;
                cx.notify();
            });
        })
        // Port socket dot
        .child(
            div()
                .size(px(10.0 * zoom.clamp(0.8, 1.2)))
                .rounded_full()
                .bg(port_color)
                .border_2()
                .border_color(if is_connecting {
                    hsla(45.0 / 360.0, 0.95, 0.65, 1.0)
                } else {
                    hsla(220.0 / 360.0, 0.35, 0.15, 1.0)
                }),
        )
        // Port label
        .child(
            div()
                .text_size(px(10.0 * zoom.clamp(0.75, 1.1)))
                .text_color(hsla(0.0, 0.0, 0.85, 1.0))
                .child(port.name.clone()),
        )
}

fn render_output_port(
    port: &Port,
    node: &NodeData,
    zoom: f32,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let port_id = port.id.clone();
    let n_id = node.id.clone();
    let port_offset_y = node.get_port_offset_y(&port.id, PortDirection::Output);
    let port_pos = (node.pos.0 + node.size.0, node.pos.1 + port_offset_y);
    let model_start_wire = model.clone();

    let port_color = match port.port_type {
        PortType::Flow => hsla(145.0 / 360.0, 0.75, 0.55, 1.0),
        PortType::Condition => hsla(40.0 / 360.0, 0.85, 0.55, 1.0),
        PortType::Data => hsla(200.0 / 360.0, 0.85, 0.60, 1.0),
    };

    div()
        .flex()
        .items_center()
        .gap_1p5()
        .cursor_crosshair()
        .hover(|s| s.opacity(0.85))
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            let s_node = n_id.clone();
            let s_port = port_id.clone();
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_start_wire.update(cx, |state, cx| {
                state.drag_state = DragState::Connecting {
                    source_node_id: s_node,
                    source_port_id: s_port,
                    source_pos: port_pos,
                    current_mouse: (mx, my),
                };
                cx.notify();
            });
        })
        // Port label
        .child(
            div()
                .text_size(px(10.0 * zoom.clamp(0.75, 1.1)))
                .text_color(hsla(0.0, 0.0, 0.85, 1.0))
                .child(port.name.clone()),
        )
        // Port socket dot
        .child(
            div()
                .size(px(10.0 * zoom.clamp(0.8, 1.2)))
                .rounded_full()
                .bg(port_color)
                .border_2()
                .border_color(hsla(220.0 / 360.0, 0.35, 0.15, 1.0)),
        )
}
