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
    let node_w = (node.size.0 * zoom).max(16.0);
    let node_h = (node.size.1 * zoom).max(12.0);

    let node_id = node.id.clone();
    let model_header_drag = model.clone();

    // Responsive scaling variables with zoom
    let z = zoom;
    let font_title = (12.0 * z).clamp(2.0, 24.0);
    let font_sub = (9.5 * z).clamp(2.0, 18.0);
    let font_prop = (9.0 * z).clamp(2.0, 16.0);
    let font_badge = (8.5 * z).clamp(2.0, 14.0);
    let pad_x = (8.0 * z).clamp(1.0, 16.0);
    let pad_y = (4.0 * z).clamp(1.0, 10.0);
    let icon_size = (13.0 * z).clamp(3.0, 26.0);
    let socket_size = (9.0 * z).clamp(3.0, 14.0);
    let corner_radius = (10.0 * z).clamp(2.0, 14.0);

    div()
        .id(SharedString::from(format!("node_{}", node.id)))
        .absolute()
        .left(px(screen_x))
        .top(px(screen_y))
        .w(px(node_w))
        .h(px(node_h))
        .rounded(px(corner_radius))
        .bg(hsla(220.0 / 360.0, 0.35, 0.12, 0.96))
        .border_2()
        .border_color(if is_selected {
            hsla(45.0 / 360.0, 0.95, 0.55, 1.0)
        } else {
            hsla(220.0 / 360.0, 0.25, 0.25, 0.8)
        })
        .shadow_xl()
        .cursor_default()
        // Card Main Content Area
        .child(
            div()
                .size_full()
                .overflow_hidden()
                .flex()
                .flex_col()
                // Header (Draggable)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(pad_x))
                        .py(px(pad_y))
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
                                .gap(px((4.0 * z).max(1.0)))
                                .child(
                                    div()
                                        .text_size(px(icon_size))
                                        .child(node.kind.icon()),
                                )
                                .child(
                                    div()
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(font_title))
                                        .text_color(rgb(0xffffff))
                                        .child(node.title.clone()),
                                ),
                        )
                        .children(if z >= 0.45 {
                            Some(
                                div()
                                    .text_size(px(font_badge))
                                    .px(px((4.0 * z).max(1.0)))
                                    .py(px((1.0 * z).max(0.5)))
                                    .rounded_sm()
                                    .bg(hsla(0.0, 0.0, 0.0, 0.3))
                                    .text_color(hsla(0.0, 0.0, 0.95, 0.85))
                                    .child(node.kind.display_name()),
                            )
                        } else {
                            None
                        }),
                )
                // Subtitle / Actor Scope
                .children(if z >= 0.40 {
                    node.subtitle.as_ref().map(|sub| {
                        div()
                            .px(px(pad_x))
                            .pt(px((2.0 * z).max(1.0)))
                            .text_size(px(font_sub))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(hsla(45.0 / 360.0, 0.85, 0.70, 1.0))
                            .child(sub.clone())
                    })
                } else {
                    None
                })
                // Properties list
                .children(if z >= 0.50 && !node.properties.is_empty() {
                    Some(
                        div()
                            .px(px(pad_x))
                            .py(px(pad_y))
                            .flex()
                            .flex_col()
                            .gap(px((2.0 * z).max(1.0)))
                            .children(node.properties.iter().take(3).map(|prop| {
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .text_size(px(font_prop))
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
        // Absolute Positioned Top Input Sockets (Matches get_port_offset exactly)
        .children(node.inputs.iter().map(|p| {
            let offset = node.get_port_offset(&p.id, PortDirection::Input);
            render_absolute_input_socket(p, &node.id, offset, z, socket_size, connecting_state, model)
        }))
        // Absolute Positioned Bottom Output Sockets (Matches get_port_offset exactly)
        .children(node.outputs.iter().map(|p| {
            let offset = node.get_port_offset(&p.id, PortDirection::Output);
            render_absolute_output_socket(p, node, offset, z, socket_size, model)
        }))
}

fn render_absolute_input_socket(
    port: &Port,
    node_id: &str,
    offset: (f32, f32),
    zoom: f32,
    socket_size: f32,
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

    let sx = offset.0 * zoom - socket_size / 2.0;
    let sy = -socket_size / 2.0;

    div()
        .absolute()
        .left(px(sx))
        .top(px(sy))
        .size(px(socket_size))
        .rounded_full()
        .bg(port_color)
        .border_2()
        .border_color(if is_connecting {
            hsla(45.0 / 360.0, 0.95, 0.65, 1.0)
        } else {
            hsla(220.0 / 360.0, 0.35, 0.15, 1.0)
        })
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
}

fn render_absolute_output_socket(
    port: &Port,
    node: &NodeData,
    offset: (f32, f32),
    zoom: f32,
    socket_size: f32,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let port_id = port.id.clone();
    let n_id = node.id.clone();
    let port_pos = (node.pos.0 + offset.0, node.pos.1 + offset.1);
    let model_start_wire = model.clone();

    let port_color = match port.port_type {
        PortType::Flow => hsla(145.0 / 360.0, 0.75, 0.55, 1.0),
        PortType::Condition => hsla(40.0 / 360.0, 0.85, 0.55, 1.0),
        PortType::Data => hsla(200.0 / 360.0, 0.85, 0.60, 1.0),
    };

    let sx = offset.0 * zoom - socket_size / 2.0;
    let sy = offset.1 * zoom - socket_size / 2.0;

    div()
        .absolute()
        .left(px(sx))
        .top(px(sy))
        .size(px(socket_size))
        .rounded_full()
        .bg(port_color)
        .border_2()
        .border_color(hsla(220.0 / 360.0, 0.35, 0.15, 1.0))
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
}
