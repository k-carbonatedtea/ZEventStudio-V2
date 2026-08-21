use gpui::*;
use super::socket::{render_absolute_input_socket, render_absolute_output_socket};
use crate::model::{NodeData, PortDirection, PropertyItem};
use crate::ui::state::{AppState, DragState};

/// 渲染标准 Event 节点卡片 (Dark Zinc Card + Type Badge + Title + Subtitle + Parameters)
pub fn render_event_card(
    state: &AppState,
    node: &NodeData,
    screen_pos: (f32, f32),
    node_size: (f32, f32),
    zoom: f32,
    is_selected: bool,
    connecting_state: &DragState,
    expand_all: bool,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let (screen_x, screen_y) = screen_pos;
    let (node_w, node_h) = node_size;
    let node_id = node.id.clone();
    let model_header_drag = model.clone();

    let z = zoom;
    let socket_size = (9.0 * z).clamp(3.0, 12.0);
    let font_title = (11.5 * z).clamp(2.0, 20.0);
    let font_sub = (10.0 * z).clamp(2.0, 16.0);
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

    let mut card = div()
        .id(SharedString::from(format!("node_{}", node.id)))
        .absolute()
        .left(px(screen_x))
        .top(px(screen_y))
        .w(px(node_w))
        .h(px(node_h))
        .rounded(px(corner_radius))
        .bg(hsla(240.0 / 360.0, 0.06, 0.10, 0.98)) // #18181b
        .border_1()
        .border_color(card_border)
        .cursor_default();

    if z >= 0.45 {
        card = card.shadow_md();
    }

    card
        // 主内容区域
        .child(
            div()
                .size_full()
                .overflow_hidden()
                .flex()
                .flex_col()
                .px(px(pad_x))
                .py(px(pad_y))
                // 1. 卡片顶部标题栏 (类型徽章 + 节点标题) - 可拖拽
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
                        // 类型徽章
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
                        // 节点标题
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(font_title))
                                .text_color(rgb(0xffffff))
                                .child(node.title.clone()),
                        ),
                )
                // 2. 副标题 / Actor 行为说明
                .children(node.subtitle.as_ref().map(|sub| {
                    div()
                        .pt(px((4.0 * z).max(1.0)))
                        .font_weight(FontWeight::MEDIUM)
                        .text_size(px(font_sub))
                        .text_color(hsla(215.0 / 360.0, 0.25, 0.90, 1.0))
                        .child(sub.clone())
                }))
                // 3. 参数属性列表 (Parameters)
                .children(render_card_parameters(state, &node.properties, z, expand_all)),
        )
        // 顶部输入引脚 (Input Sockets)
        .children(if z >= 0.32 {
            node.inputs
                .iter()
                .map(|p| {
                    let offset = node.get_port_offset(&p.id, PortDirection::Input);
                    render_absolute_input_socket(p, &node.id, offset, z, socket_size, connecting_state, model)
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        })
        // 底部输出引脚 (Output Sockets)
        .children(if z >= 0.32 {
            node.outputs
                .iter()
                .map(|p| {
                    let offset = node.get_port_offset(&p.id, PortDirection::Output);
                    render_absolute_output_socket(p, node, offset, z, socket_size, model)
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        })
}

/// 渲染卡片内部的参数属性小表格
fn render_card_parameters(
    state: &AppState,
    properties: &[PropertyItem],
    z: f32,
    expand_all: bool,
) -> Option<impl IntoElement> {
    if z < 0.35 || properties.is_empty() {
        return None;
    }

    let font_prop_header = (8.5 * z).clamp(2.0, 14.0);
    let font_prop = (9.0 * z).clamp(2.0, 15.0);

    let props_to_show: Vec<&PropertyItem> = if expand_all {
        properties.iter().collect()
    } else {
        properties.iter().take(2).collect()
    };

    let header_label = if expand_all {
        format!("{} ({}) :", state.t("node.parameters_label"), properties.len())
    } else {
        state.t("node.parameters_label").to_string()
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
                    .child(header_label),
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
}
