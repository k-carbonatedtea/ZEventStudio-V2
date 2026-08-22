use gpui::*;
use crate::model::{NodeKind, PropertyItem};
use crate::ui::canvas::node_editor::NodeEditorModalState;
use crate::ui::state::AppState;

/// 渲染 Actor 与 Action/Query 下拉选择条
pub fn render_actor_action_selectors(
    state: &AppState,
    modal: &NodeEditorModalState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_act_click = model.clone();
    let model_action_click = model.clone();

    let actor_display = if modal.actor_name.is_empty() {
        state.t("modal.select_actor").to_string()
    } else {
        modal.actor_name.clone()
    };
    let action_display = if modal.action_name.is_empty() {
        state.t("modal.select_action").to_string()
    } else {
        modal.action_name.clone()
    };

    let current_actor = state.graph.actors.iter().find(|a| a.name == modal.actor_name);
    let available_actions: Vec<String> = if modal.node_kind == NodeKind::Action {
        current_actor.map(|a| a.actions.clone()).unwrap_or_default()
    } else {
        current_actor.map(|a| a.queries.clone()).unwrap_or_default()
    };

    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_2()
                .rounded_lg()
                .bg(hsla(240.0 / 360.0, 0.06, 0.12, 1.0))
                .border_1()
                .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                // 角色选择下拉框 (Actor Selector)
                .child(
                    div()
                        .flex_1()
                        .h(px(32.0))
                        .px_3()
                        .rounded_md()
                        .bg(hsla(240.0 / 360.0, 0.06, 0.16, 1.0))
                        .border_1()
                        .border_color(if modal.show_actor_dropdown {
                            hsla(215.0 / 360.0, 0.90, 0.55, 1.0)
                        } else {
                            hsla(240.0 / 360.0, 0.06, 0.26, 1.0)
                        })
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_between()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_act_click.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.active_node_editor_modal {
                                    m.show_actor_dropdown = !m.show_actor_dropdown;
                                    m.show_action_dropdown = false;
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                                .child(actor_display),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(hsla(240.0 / 360.0, 0.05, 0.60, 1.0))
                                .child("▾"),
                        ),
                )
                // 分隔符
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(hsla(240.0 / 360.0, 0.05, 0.50, 1.0))
                        .child("::"),
                )
                // 动作/条件选择下拉框 (Action/Query Selector)
                .child(
                    div()
                        .flex_1()
                        .h(px(32.0))
                        .px_3()
                        .rounded_md()
                        .bg(hsla(240.0 / 360.0, 0.06, 0.16, 1.0))
                        .border_1()
                        .border_color(if modal.show_action_dropdown {
                            hsla(215.0 / 360.0, 0.90, 0.55, 1.0)
                        } else {
                            hsla(240.0 / 360.0, 0.06, 0.26, 1.0)
                        })
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_between()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_action_click.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.active_node_editor_modal {
                                    m.show_action_dropdown = !m.show_action_dropdown;
                                    m.show_actor_dropdown = false;
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                                .child(action_display),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(hsla(240.0 / 360.0, 0.05, 0.60, 1.0))
                                .child("▾"),
                        ),
                ),
        )
        // 角色下拉弹窗
        .children(modal.show_actor_dropdown.then(|| {
            let actors = &state.graph.actors;
            div()
                .id("actor_dropdown_scroll")
                .max_h(px(200.0))
                .overflow_y_scroll()
                .bg(hsla(240.0 / 360.0, 0.06, 0.08, 0.98))
                .border_1()
                .border_color(hsla(215.0 / 360.0, 0.50, 0.40, 1.0))
                .rounded_lg()
                .p_1()
                .flex()
                .flex_col()
                .gap_0p5()
                .children(actors.iter().map(|act| {
                    let model_sel = model.clone();
                    let a_name = act.name.clone();
                    let first_action = if modal.node_kind == NodeKind::Action {
                        act.actions.first().cloned().unwrap_or_else(|| "Demo_WaitFrame".to_string())
                    } else {
                        act.queries.first().cloned().unwrap_or_else(|| "GeneralQuery".to_string())
                    };

                    div()
                        .px_2p5()
                        .py_1p5()
                        .rounded_md()
                        .cursor_pointer()
                        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.35, 0.25, 1.0)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            let a_n = a_name.clone();
                            let f_a = first_action.clone();
                            model_sel.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.active_node_editor_modal {
                                    m.actor_name = a_n;
                                    m.action_name = f_a;
                                    m.show_actor_dropdown = false;
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xffffff))
                                .child(act.name.clone()),
                        )
                }))
        }))
        // 动作/条件下拉弹窗
        .children(modal.show_action_dropdown.then(|| {
            div()
                .id("action_dropdown_scroll")
                .max_h(px(200.0))
                .overflow_y_scroll()
                .bg(hsla(240.0 / 360.0, 0.06, 0.08, 0.98))
                .border_1()
                .border_color(hsla(215.0 / 360.0, 0.50, 0.40, 1.0))
                .rounded_lg()
                .p_1()
                .flex()
                .flex_col()
                .gap_0p5()
                .children(if available_actions.is_empty() {
                    vec![div()
                        .p_2()
                        .text_size(px(11.5))
                        .text_color(hsla(240.0 / 360.0, 0.05, 0.50, 1.0))
                        .child("暂无预设动作")]
                } else {
                    available_actions
                        .iter()
                        .map(|act_name| {
                            let model_sel = model.clone();
                            let a_name = act_name.clone();
                            div()
                                .px_2p5()
                                .py_1p5()
                                .rounded_md()
                                .cursor_pointer()
                                .hover(|s| s.bg(hsla(215.0 / 360.0, 0.35, 0.25, 1.0)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    let a_n = a_name.clone();
                                    model_sel.update(cx, |st, cx| {
                                        if let Some(ref mut m) = st.active_node_editor_modal {
                                            m.action_name = a_n;
                                            m.show_action_dropdown = false;
                                        }
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0xffffff))
                                        .child(act_name.clone()),
                                )
                        })
                        .collect()
                })
        }))
}

/// 依据当前动作名称自动填充 BotW 标准参数模版
pub fn autofill_modal_params(modal: &mut NodeEditorModalState) {
    let act = modal.action_name.as_str();
    if act.contains("FlagON") || act.contains("FlagOFF") || act.contains("Flag") {
        modal.properties = vec![
            PropertyItem {
                key: "FlagName".into(),
                val: "100enemy_Clear_Middle".into(),
                val_type: "string".into(),
            },
            PropertyItem {
                key: "IsWaitFinish".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
        ];
    } else if act.contains("Message") || act.contains("Talk") {
        modal.properties = vec![
            PropertyItem {
                key: "MessageId".into(),
                val: "Talk_01".into(),
                val_type: "string".into(),
            },
            PropertyItem {
                key: "IsWaitFinish".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
        ];
    } else if act.contains("Animation") || act.contains("PlayAnm") {
        modal.properties = vec![
            PropertyItem {
                key: "AnmName".into(),
                val: "Wait".into(),
                val_type: "string".into(),
            },
            PropertyItem {
                key: "IsWaitFinish".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
        ];
    } else if !modal.properties.iter().any(|p| p.key == "IsWaitFinish") {
        modal.properties.push(PropertyItem {
            key: "IsWaitFinish".into(),
            val: "true".into(),
            val_type: "bool".into(),
        });
    }
}
