use gpui::*;
use crate::model::{NodeData, NodeKind, Port, PortDirection};
use crate::ui::state::{AppState, DragState};

pub fn render_nodes_layer(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pan = state.pan;
    let zoom = state.zoom;
    let selected_id = state.selected_node_id.clone();
    let connecting_state = state.drag_state.clone();
    let expand_all = state.expand_all_params;

    div()
        .absolute()
        .inset_0()
        .children(state.graph.nodes.iter().map(|node| {
            render_single_node(node, pan, zoom, &selected_id, &connecting_state, expand_all, model)
        }))
}

fn render_single_node(
    node: &NodeData,
    pan: (f32, f32),
    zoom: f32,
    selected_id: &Option<String>,
    connecting_state: &DragState,
    expand_all: bool,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let is_selected = selected_id.as_deref() == Some(&node.id);
    let screen_x = pan.0 + node.pos.0 * zoom;
    let screen_y = pan.1 + node.pos.1 * zoom;
    let node_w = (node.size.0 * zoom).max(16.0);
    let node_h = (node.size.1 * zoom).max(12.0);

    let node_id = node.id.clone();
    let model_header_drag = model.clone();

    let z = zoom;
    let socket_size = (9.0 * z).clamp(3.0, 12.0);

    // ==========================================
    // 1. EntryPoint 节点 (图2 顶部的翡翠绿胶囊 Pill 节点)
    // ==========================================
    if node.kind == NodeKind::EntryPoint {
        let font_title = (12.0 * z).clamp(3.0, 18.0);
        let pad_x = (14.0 * z).clamp(2.0, 20.0);
        let pad_y = (6.0 * z).clamp(1.0, 12.0);

        return div()
            .id(SharedString::from(format!("node_{}", node.id)))
            .absolute()
            .left(px(screen_x))
            .top(px(screen_y))
            .w(px(node_w))
            .h(px(node_h))
            .rounded_full()
            .bg(hsla(160.0 / 360.0, 0.84, 0.39, 1.0)) // #10b981
            .border_2()
            .border_color(if is_selected {
                hsla(45.0 / 360.0, 0.95, 0.60, 1.0)
            } else {
                hsla(160.0 / 360.0, 0.84, 0.28, 1.0) // #059669
            })
            .shadow_md()
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
            // Entry Title Center
            .child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .px(px(pad_x))
                    .py(px(pad_y))
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_size(px(font_title))
                            .text_color(rgb(0xffffff))
                            .child(node.title.clone()),
                    ),
            )
            // Bottom Output Socket
            .children(node.outputs.iter().map(|p| {
                let offset = node.get_port_offset(&p.id, PortDirection::Output);
                render_absolute_output_socket(p, node, offset, z, socket_size, model)
            }));
    }

    // ==========================================
    // 2. 标准 Event 节点 (图2: Dark Card + Type Badge + EventID + Parameters)
    // ==========================================
    let font_title = (11.5 * z).clamp(2.0, 20.0);
    let font_sub = (10.0 * z).clamp(2.0, 16.0);
    let font_prop_header = (8.5 * z).clamp(2.0, 14.0);
    let font_prop = (9.0 * z).clamp(2.0, 15.0);
    let font_badge = (8.5 * z).clamp(2.0, 13.0);
    let pad_x = (10.0 * z).clamp(1.0, 16.0);
    let pad_y = (6.0 * z).clamp(1.0, 12.0);
    let corner_radius = (8.0 * z).clamp(2.0, 12.0);

    let badge_bg = node.kind.badge_color();
    let card_border = if is_selected {
        hsla(215.0 / 360.0, 0.90, 0.60, 1.0) // #3b82f6
    } else {
        node.kind.border_color()
    };

    div()
        .id(SharedString::from(format!("node_{}", node.id)))
        .absolute()
        .left(px(screen_x))
        .top(px(screen_y))
        .w(px(node_w))
        .h(px(node_h))
        .rounded(px(corner_radius))
        .bg(hsla(240.0 / 360.0, 0.06, 0.10, 0.98)) // #18181b (Dark Zinc)
        .border_1()
        .border_color(card_border)
        .shadow_md()
        .cursor_default()
        // Card Main Content Area
        .child(
            div()
                .size_full()
                .overflow_hidden()
                .flex()
                .flex_col()
                .px(px(pad_x))
                .py(px(pad_y))
                // Header (Badge + EventID) - Draggable
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px((6.0 * z).max(1.0)))
                        .pb(px((4.0 * z).max(1.0)))
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 1.0, 0.08))
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
                        // Type Badge (e.g. ACTION, SWITCH, FORK, JOIN)
                        .child(
                            div()
                                .px(px((5.0 * z).max(1.0)))
                                .py(px((1.5 * z).max(0.5)))
                                .rounded_sm()
                                .bg(badge_bg)
                                .text_size(px(font_badge))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xffffff))
                                .child(node.kind.display_name()),
                        )
                        // Event ID (e.g. Event257)
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(font_title))
                                .text_color(rgb(0xffffff))
                                .child(node.title.clone()),
                        ),
                )
                // Subtitle / Actor :: Action Line
                .children(node.subtitle.as_ref().map(|sub| {
                    div()
                        .pt(px((4.0 * z).max(1.0)))
                        .font_weight(FontWeight::MEDIUM)
                        .text_size(px(font_sub))
                        .text_color(hsla(215.0 / 360.0, 0.25, 0.90, 1.0))
                        .child(sub.clone())
                }))
                // Parameters Section (参数属性)
                .children(if z >= 0.35 && !node.properties.is_empty() {
                    let props_to_show: Vec<&crate::model::PropertyItem> = if expand_all {
                        node.properties.iter().collect()
                    } else {
                        node.properties.iter().take(2).collect()
                    };

                    Some(
                        div()
                            .pt(px((4.0 * z).max(1.0)))
                            .flex()
                            .flex_col()
                            .gap(px((2.0 * z).max(1.0)))
                            .child(
                                div()
                                    .text_size(px(font_prop_header))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(hsla(215.0 / 360.0, 0.20, 0.50, 1.0))
                                    .child(if expand_all {
                                        format!("参数属性 ({}) :", node.properties.len())
                                    } else {
                                        "参数属性 (Parameters):".to_string()
                                    }),
                            )
                            .children(props_to_show.into_iter().map(|prop| {
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap(px((4.0 * z).max(1.0)))
                                    .text_size(px(font_prop))
                                    .child(
                                        div()
                                            .text_color(hsla(215.0 / 360.0, 0.25, 0.65, 1.0))
                                            .child(prop.key.clone()),
                                    )
                                    .child(
                                        div()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(hsla(271.0 / 360.0, 0.85, 0.75, 1.0))
                                            .child(if !expand_all && prop.val.len() > 16 {
                                                format!("{}...", &prop.val[..16])
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
        // Top Input Socket (Centered Handle Ring)
        .children(node.inputs.iter().map(|p| {
            let offset = node.get_port_offset(&p.id, PortDirection::Input);
            render_absolute_input_socket(p, &node.id, offset, z, socket_size, connecting_state, model)
        }))
        // Bottom Output Socket (Centered Handle Ring)
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
    let sx = offset.0 * zoom - socket_size / 2.0;
    let sy = -socket_size / 2.0;

    div()
        .absolute()
        .left(px(sx))
        .top(px(sy))
        .size(px(socket_size))
        .rounded_full()
        .bg(hsla(240.0 / 360.0, 0.06, 0.10, 1.0))
        .border_2()
        .border_color(if is_connecting {
            hsla(45.0 / 360.0, 0.95, 0.65, 1.0)
        } else {
            hsla(200.0 / 360.0, 0.90, 0.60, 1.0) // #38bdf8
        })
        .cursor_pointer()
        .hover(|s| s.bg(hsla(200.0 / 360.0, 0.90, 0.60, 1.0)))
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

    let sx = offset.0 * zoom - socket_size / 2.0;
    let sy = offset.1 * zoom - socket_size / 2.0;

    let socket_border = if node.kind == NodeKind::EntryPoint {
        hsla(160.0 / 360.0, 0.84, 0.28, 1.0) // #059669
    } else {
        hsla(200.0 / 360.0, 0.90, 0.60, 1.0) // #38bdf8
    };

    div()
        .absolute()
        .left(px(sx))
        .top(px(sy))
        .size(px(socket_size))
        .rounded_full()
        .bg(hsla(240.0 / 360.0, 0.06, 0.10, 1.0))
        .border_2()
        .border_color(socket_border)
        .cursor_crosshair()
        .hover(|s| s.bg(socket_border))
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
