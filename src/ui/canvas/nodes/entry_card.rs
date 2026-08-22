use gpui::*;
use super::socket::render_absolute_output_socket;
use crate::model::{NodeData, PortDirection};
use crate::ui::state::{AppState, DragState};

/// 渲染 EntryPoint 节点 (翡翠绿胶囊 Pill 节点，支持 Flow Tracing)
pub fn render_entry_card(
    state: &AppState,
    node: &NodeData,
    screen_pos: (f32, f32),
    node_size: (f32, f32),
    zoom: f32,
    is_selected: bool,
    connecting_state: &DragState,
    model: &Entity<AppState>,
) -> impl IntoElement {

    let (screen_x, screen_y) = screen_pos;
    let (node_w, node_h) = node_size;
    let node_id = node.id.clone();
    let model_header_drag = model.clone();

    let z = zoom;
    let socket_size = (9.0 * z).clamp(3.0, 12.0);
    let font_title = (12.0 * z).clamp(3.0, 18.0);
    let pad_x = (14.0 * z).clamp(2.0, 20.0);
    let pad_y = (6.0 * z).clamp(1.0, 12.0);

    // 链路追踪 (Flow Tracing) 高亮判定
    let active_node = state.selected_node_id.as_deref().or(state.hovered_node_id.as_deref());
    let (is_on_active_path, is_unrelated) = if let Some(target) = active_node {
        if target == node.id {
            (true, false)
        } else {
            let (up, down) = state.graph.get_upstream_and_downstream(target);
            let on_path = up.contains(&node.id) || down.contains(&node.id);
            (on_path, !on_path)
        }
    } else {
        (false, false)
    };

    let border_col = if is_selected {
        hsla(45.0 / 360.0, 0.98, 0.60, 1.0)
    } else if is_on_active_path {
        hsla(45.0 / 360.0, 0.85, 0.50, 0.90)
    } else {
        hsla(160.0 / 360.0, 0.84, 0.28, 1.0) // #059669
    };

    let card_opacity = if is_unrelated { 0.45 } else { 1.0 };

    let mut entry_card = div()
        .id(SharedString::from(format!("node_{}", node.id)))
        .absolute()
        .left(px(screen_x))
        .top(px(screen_y))
        .w(px(node_w))
        .h(px(node_h))
        .rounded_full()
        .bg(hsla(160.0 / 360.0, 0.84, 0.39, 1.0)) // #10b981
        .border_2()
        .border_color(border_col)
        .opacity(card_opacity)
        .cursor_move();


    if z >= 0.45 {
        entry_card = entry_card.shadow_md();
    }

    let node_id_right = node.id.clone();
    let model_header_drag_right = model.clone();

    entry_card
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            let id = node_id.clone();
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_header_drag.update(cx, |state, cx| {
                if matches!(state.drag_state, DragState::Connecting { .. }) {
                    return;
                }
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
        .on_mouse_down(MouseButton::Right, move |e, _, cx| {
            let id = node_id_right.clone();
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_header_drag_right.update(cx, |state, cx| {
                if matches!(state.drag_state, DragState::Connecting { .. }) {
                    return;
                }
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
        .children(if z >= 0.32 {
            node.outputs
                .iter()
                .map(|p| {
                    let offset = node.get_port_offset(&p.id, PortDirection::Output);
                    render_absolute_output_socket(p, node, offset, z, socket_size, connecting_state, model)
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        })

}
