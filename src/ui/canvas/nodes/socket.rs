use gpui::*;
use crate::model::{NodeData, NodeKind, Port};
use crate::ui::state::{AppState, DragState};

/// 渲染绝对定位的输入引脚 (Input Socket Handle)
pub fn render_absolute_input_socket(
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

/// 渲染绝对定位的输出引脚 (Output Socket Handle)
pub fn render_absolute_output_socket(
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
