use gpui::*;
use super::socket::render_absolute_output_socket;
use super::NodeRenderContext;
use crate::model::{NodeData, PortDirection};
use crate::ui::state::{AppState, DragState};

/// 渲染 EntryPoint 节点 (翡翠绿胶囊 Pill 节点，支持 Flow Tracing)
pub fn render_entry_card(
    _state: &AppState,
    node: &NodeData,
    ctx: &NodeRenderContext,
) -> impl IntoElement {
    let (screen_x, screen_y) = ctx.screen_pos;
    let (node_w, node_h) = ctx.node_size;
    let node_id = node.id.clone();
    let model_header_drag = ctx.model.clone();

    let z = ctx.zoom;
    let socket_size = (9.0 * z).clamp(3.0, 12.0);
    let font_title = (12.0 * z).clamp(3.0, 18.0);
    let pad_x = (14.0 * z).clamp(2.0, 20.0);
    let pad_y = (6.0 * z).clamp(1.0, 12.0);

    // 链路追踪 (Flow Tracing) O(1) 快速判定
    let (is_on_active_path, is_unrelated) = if let Some(path_set) = ctx.active_path_nodes {
        let on_path = path_set.contains(&node.id);
        (on_path, !on_path)
    } else {
        (false, false)
    };

    let (border_col, card_bg) = if ctx.is_selected {
        (
            rgb(0x00E5BA), // 核心选中高亮：遗迹青蓝 Primary Accent
            rgb(0x00E5BA),
        )
    } else if is_on_active_path {
        (
            rgb(0xF59E0B), // 链路高亮：琥珀金 Secondary Accent
            rgb(0x10B981),
        )
    } else {
        (
            rgb(0x059669),
            rgb(0x10B981),
        )
    };

    let card_opacity = if is_unrelated { 0.38 } else { 1.0 };

    let mut entry_card = div()
        .id(SharedString::from(format!("node_{}", node.id)))
        .absolute()
        .left(px(screen_x))
        .top(px(screen_y))
        .w(px(node_w))
        .h(px(node_h))
        .rounded_full()
        .bg(card_bg)
        .border_2()
        .border_color(border_col)
        .opacity(card_opacity)
        .cursor_move();

    if ctx.is_selected {
        entry_card = entry_card.shadow_2xl();
    } else if z >= 0.45 {
        entry_card = entry_card.shadow_md();
    }


    let node_id_right = node.id.clone();
    let model_header_drag_right = ctx.model.clone();


    entry_card
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            let id = node_id.clone();
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            if e.click_count == 2 {
                model_header_drag.update(cx, |state, cx| {
                    state.open_node_editor(&id);
                    cx.notify();
                });
                return;
            }
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
        // 鼠标右键点击节点卡片任何位置，呼出操作面板 (上方添加/下方添加/删除节点)
        .on_mouse_down(MouseButton::Right, move |e, _, cx| {
            let id = node_id_right.clone();
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_header_drag_right.update(cx, |state, cx| {
                if matches!(state.drag_state, DragState::Connecting { .. }) {
                    return;
                }
                state.selected_node_id = Some(id.clone());
                let (canvas_x, canvas_y) = state.screen_to_canvas(mx, my);
                state.node_context_menu = Some((id, (canvas_x, canvas_y)));
                state.quick_add_modal = None;
                state.drag_state = DragState::None;
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
                    render_absolute_output_socket(p, node, offset, z, socket_size, ctx.connecting_state, ctx.model)
                })

                .collect::<Vec<_>>()
        } else {
            vec![]
        })

}
