use gpui::*;
use super::socket::{render_absolute_input_socket, render_absolute_output_socket};
use crate::model::{KeyHighlight, NodeData, PortDirection, PropertyItem};
use crate::ui::state::{AppState, DragState};

/// 渲染标准 Event 节点卡片 (Actor Theme Banner + Smart Parameter Badges + Flow Tracing)
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

    let z = zoom;

    let socket_size = (9.0 * z).clamp(3.0, 12.0);
    let font_title = (11.5 * z).clamp(2.0, 20.0);
    let font_sub = (10.0 * z).clamp(2.0, 16.0);
    let font_badge = (8.5 * z).clamp(2.0, 13.0);
    let pad_x = (10.0 * z).clamp(1.0, 16.0);
    let pad_y = (6.0 * z).clamp(1.0, 12.0);
    let corner_radius = (8.0 * z).clamp(2.0, 12.0);

    let theme = node.actor_theme();
    let highlights = node.extract_key_highlights();

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

    let card_border = if is_selected {
        hsla(45.0 / 360.0, 0.98, 0.60, 1.0) // 选中金黄色发光 #fbbf24
    } else if is_on_active_path {
        hsla(45.0 / 360.0, 0.85, 0.50, 0.90) // 链路上游/下游节点
    } else {
        theme.border_color
    };

    let card_opacity = if is_unrelated { 0.45 } else { 1.0 };

    let node_id_left = node.id.clone();
    let node_id_right = node.id.clone();
    let model_drag_left = model.clone();
    let model_drag_right = model.clone();

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
        .opacity(card_opacity)
        .cursor_move()
        // 点击卡片任何位置（左键或右键长按）均可拖动节点
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            let id = node_id_left.clone();
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_drag_left.update(cx, |state, cx| {
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
            model_drag_right.update(cx, |state, cx| {
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
        });


    if z >= 0.45 && !is_unrelated {
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
                // 1. 角色标头栏 (Actor Identity Header Banner + 类型徽章)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .pb(px((3.0 * z).max(1.0)))
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 1.0, 0.08))

                        // 左侧：Actor 图标 + 动作标题
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px((5.0 * z).max(1.0)))
                                .child(
                                    div()
                                        .text_size(px(font_badge * 1.1))
                                        .child(theme.icon),
                                )
                                .child(
                                    div()
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(font_title))
                                        .text_color(rgb(0xffffff))
                                        .child(node.title.clone()),
                                ),
                        )
                        // 右侧：节点类型小徽章
                        .child(
                            div()
                                .px(px((4.5 * z).max(1.0)))
                                .py(px((1.0 * z).max(0.5)))
                                .rounded_sm()
                                .bg(theme.badge_bg)
                                .text_size(px(font_badge * 0.9))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xffffff))
                                .child(node.kind.display_name()),
                        ),
                )
                // 2. 副标题 / 所属 Actor 角色名称
                .children(node.subtitle.as_ref().map(|sub| {
                    div()
                        .pt(px((3.0 * z).max(1.0)))
                        .font_weight(FontWeight::MEDIUM)
                        .text_size(px(font_sub))
                        .text_color(hsla(215.0 / 360.0, 0.25, 0.85, 1.0))
                        .child(sub.clone())
                }))
                // 3. 核心高频业务参数胶囊 / 完整参数列表
                .children(if expand_all {
                    render_full_parameters(state, &node.properties, z)
                } else {
                    render_smart_highlights(&highlights, z)
                }),
        )
        // 顶部输入引脚 (Input Sockets)
        .children(if z >= 0.32 {
            node.inputs
                .iter()
                .map(|p| {
                    let offset = node.get_port_offset(&p.id, PortDirection::Input);
                    render_absolute_input_socket(p, node, offset, z, socket_size, connecting_state, model)
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
                    render_absolute_output_socket(p, node, offset, z, socket_size, connecting_state, model)
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        })
}


/// 渲染智能提炼的关键高频业务参数胶囊药丸 (Smart Parameter Highlights)
fn render_smart_highlights(
    highlights: &[KeyHighlight],
    z: f32,
) -> Option<AnyElement> {
    if z < 0.35 || highlights.is_empty() {
        return None;
    }

    let font_size = (9.0 * z).clamp(2.0, 14.0);

    Some(
        div()
            .pt(px((4.0 * z).max(1.0)))
            .flex()
            .flex_col()
            .gap(px((3.0 * z).max(1.0)))
            .children(highlights.iter().map(|hl| {
                div()
                    .flex()
                    .items_center()
                    .gap(px((4.0 * z).max(1.0)))
                    .px(px((5.0 * z).max(1.0)))
                    .py(px((1.5 * z).max(0.5)))
                    .rounded_sm()
                    .bg(hsla(220.0 / 360.0, 0.30, 0.16, 0.90))
                    .border_1()
                    .border_color(hsla(215.0 / 360.0, 0.25, 0.30, 0.70))
                    .text_size(px(font_size))
                    .child(div().child(hl.icon))
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(hl.color)
                            .child(if hl.value.len() > 22 {
                                format!("{}...", &hl.value[..22])
                            } else {
                                hl.value.clone()
                            }),
                    )
            }))
            .into_any_element(),
    )
}

/// 渲染展开模式下的全部参数属性小表格
fn render_full_parameters(
    state: &AppState,
    properties: &[PropertyItem],
    z: f32,
) -> Option<AnyElement> {
    if z < 0.35 || properties.is_empty() {
        return None;
    }

    let font_prop_header = (8.5 * z).clamp(2.0, 14.0);
    let font_prop = (9.0 * z).clamp(2.0, 15.0);

    let header_label = format!("{} ({}) :", state.t("node.parameters_label"), properties.len());

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
            .children(properties.iter().map(|prop| {
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
                            .child(prop.val.clone()),
                    )
            }))
            .into_any_element(),
    )
}


