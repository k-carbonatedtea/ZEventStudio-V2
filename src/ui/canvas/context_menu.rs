use gpui::*;
use crate::model::PortDirection;
use crate::ui::state::{AppState, QuickAddConnectionSource, QuickAddModalState};


/// 渲染节点右键上下文快捷操作菜单 (Node Context Menu: 上方插入/下方插入/删除节点)
pub fn render_node_context_menu(
    state: &AppState,
    node_id: &str,
    canvas_pos: (f32, f32),
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_close = model.clone();
    let model_edit = model.clone();
    let model_above = model.clone();
    let model_below = model.clone();
    let model_delete = model.clone();
    let model_autofill = model.clone();
    let n_id = node_id.to_string();

    let (screen_x, screen_y) = (
        state.pan.0 + canvas_pos.0 * state.zoom,
        state.pan.1 + canvas_pos.1 * state.zoom,
    );

    let menu_w = 205.0;
    let menu_h = 200.0;
    let pos_x = (screen_x + 6.0).clamp(20.0, 2400.0 - menu_w);
    let pos_y = (screen_y + 6.0).clamp(20.0, 1400.0 - menu_h);

    let node_title = state
        .graph
        .find_node(node_id)
        .map(|n| n.title.clone())
        .unwrap_or_else(|| node_id.to_string());

    div()
        .absolute()
        .inset_0()
        .bg(hsla(0.0, 0.0, 0.0, 0.20))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_close.update(cx, |st, cx| {
                st.node_context_menu = None;
                cx.notify();
            });
        })
        .on_mouse_down(MouseButton::Right, {
            let model_c2 = model.clone();
            move |_, _, cx| {
                model_c2.update(cx, |st, cx| {
                    st.node_context_menu = None;
                    cx.notify();
                });
            }
        })
        .child(
            div()
                .absolute()
                .left(px(pos_x))
                .top(px(pos_y))
                .w(px(menu_w))
                .rounded_xl()
                .bg(hsla(220.0 / 360.0, 0.35, 0.14, 0.98))
                .border_1()
                .border_color(hsla(215.0 / 360.0, 0.4, 0.45, 1.0))
                .shadow_2xl()
                .p_1p5()
                .flex()
                .flex_col()
                .gap_1()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(MouseButton::Right, |_, _, cx| {
                    cx.stop_propagation();
                })
                // 顶部标题小标
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .flex()
                        .items_center()
                        .justify_between()
                        .border_b_1()
                        .border_color(hsla(220.0 / 360.0, 0.25, 0.25, 1.0))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.5))
                                .text_color(rgb(0xffffff))
                                .child(node_title),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(hsla(215.0 / 360.0, 0.20, 0.60, 1.0))
                                .child(state.t("context_menu.node_actions")),
                        ),
                )
                // 0. 编辑节点属性与分支 (Edit Node Properties)
                .child(
                    div()
                        .px_2p5()
                        .py_1p5()
                        .rounded_lg()
                        .cursor_pointer()
                        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.45, 0.32, 1.0)))
                        .on_mouse_down(MouseButton::Left, {
                            let n_id = n_id.clone();
                            move |_, _, cx| {
                                cx.stop_propagation();
                                model_edit.update(cx, |st, cx| {
                                    st.node_context_menu = None;
                                    st.open_node_editor(&n_id);
                                    cx.notify();
                                });
                            }
                        })
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xffffff))
                                .child(state.t("context_menu.edit_node")),
                        ),
                )

                // 0.5. 自动填充参数 (Auto-fill Parameters)
                .child(
                    div()
                        .px_2p5()
                        .py_1p5()
                        .rounded_lg()
                        .cursor_pointer()
                        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.45, 0.32, 1.0)))
                        .on_mouse_down(MouseButton::Left, {
                            let n_id = n_id.clone();
                            move |_, _, cx| {
                                cx.stop_propagation();
                                model_autofill.update(cx, |st, cx| {
                                    st.node_context_menu = None;
                                    st.autofill_node_parameters(&n_id);
                                    cx.notify();
                                });
                            }
                        })
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xffffff))
                                .child("✨ 自动填充参数"),
                        ),
                )
                // 1. 在上方插入前驱节点
                .child(
                    div()
                        .px_2p5()
                        .py_1p5()
                        .rounded_lg()
                        .cursor_pointer()
                        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.35, 0.28, 1.0)))
                        .on_mouse_down(MouseButton::Left, {
                            let n_id = n_id.clone();
                            move |_, _, cx| {
                                cx.stop_propagation();
                                model_above.update(cx, |st, cx| {
                                    if let Some(target_node) = st.graph.find_node(&n_id) {
                                        let port_id = target_node
                                            .inputs
                                            .first()
                                            .map(|p| p.id.clone())
                                            .unwrap_or_else(|| "in".to_string());
                                        let insert_pos = (target_node.pos.0, target_node.pos.1 - 120.0);
                                        st.quick_add_modal = Some(QuickAddModalState {
                                            position: (insert_pos.0, insert_pos.1),
                                            source: Some(QuickAddConnectionSource {
                                                node_id: n_id.clone(),
                                                port_id,
                                                direction: PortDirection::Input,
                                            }),
                                        });
                                    }
                                    st.node_context_menu = None;
                                    cx.notify();
                                });
                            }
                        })
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xffffff))
                                .child(state.t("context_menu.add_above")),
                        ),
                )
                // 2. 在下方插入后继节点
                .child(
                    div()
                        .px_2p5()
                        .py_1p5()
                        .rounded_lg()
                        .cursor_pointer()
                        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.35, 0.28, 1.0)))
                        .on_mouse_down(MouseButton::Left, {
                            let n_id = n_id.clone();
                            move |_, _, cx| {
                                cx.stop_propagation();
                                model_below.update(cx, |st, cx| {
                                    if let Some(source_node) = st.graph.find_node(&n_id) {
                                        let port_id = source_node
                                            .outputs
                                            .first()
                                            .map(|p| p.id.clone())
                                            .unwrap_or_else(|| "out".to_string());
                                        let insert_pos = (
                                            source_node.pos.0,
                                            source_node.pos.1 + source_node.size.1 + 80.0,
                                        );
                                        st.quick_add_modal = Some(QuickAddModalState {
                                            position: (insert_pos.0, insert_pos.1),
                                            source: Some(QuickAddConnectionSource {
                                                node_id: n_id.clone(),
                                                port_id,
                                                direction: PortDirection::Output,
                                            }),
                                        });
                                    }
                                    st.node_context_menu = None;
                                    cx.notify();
                                });
                            }
                        })
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xffffff))
                                .child(state.t("context_menu.add_below")),
                        ),
                )
                // 3. 删除节点
                .child(
                    div()
                        .px_2p5()
                        .py_1p5()
                        .rounded_lg()
                        .cursor_pointer()
                        .hover(|s| s.bg(hsla(0.0, 0.70, 0.30, 0.90)))
                        .on_mouse_down(MouseButton::Left, {
                            let n_id = n_id.clone();
                            move |_, _, cx| {
                                cx.stop_propagation();
                                model_delete.update(cx, |st, cx| {
                                    st.selected_node_id = Some(n_id.clone());
                                    st.delete_selected_node();
                                    st.node_context_menu = None;
                                    cx.notify();
                                });
                            }
                        })
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(hsla(0.0, 0.90, 0.70, 1.0))
                                .child(state.t("context_menu.delete_node")),
                        ),
                ),
        )

}
