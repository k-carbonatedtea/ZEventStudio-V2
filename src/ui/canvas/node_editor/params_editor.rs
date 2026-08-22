use gpui::*;
use crate::model::{NodeKind, PropertyItem};
use crate::ui::canvas::node_editor::actor_action_selectors::{
    autofill_modal_params, render_actor_action_selectors,
};
use crate::ui::canvas::node_editor::NodeEditorModalState;
use crate::ui::components::Toast;
use crate::ui::state::AppState;

/// 渲染事件参数属性 (Parameters) 及 Actor / Action 动作选择器
pub fn render_params_table_editor(
    state: &AppState,
    modal: &NodeEditorModalState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let is_actor_event = matches!(
        modal.node_kind,
        NodeKind::Action | NodeKind::Query | NodeKind::Switch
    );
    let has_params = !modal.properties.is_empty();

    let model_add = model.clone();
    let model_copy = model.clone();
    let model_autofill = model.clone();
    let model_sort = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_3()
        // 1. 角色 (Actor) 与 动作/查询 (Action/Query) 选择器栏
        .children(is_actor_event.then(|| {
            render_actor_action_selectors(state, modal, model)
        }))
        // 2. 参数属性栏标头与快捷操作 (自动填充 / 排序 / 复制 JSON / 添加参数)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.5))
                        .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                        .child(state.t("modal.parameters")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        // 自动填充按钮
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(hsla(200.0 / 360.0, 0.40, 0.18, 0.9))
                                .border_1()
                                .border_color(hsla(200.0 / 360.0, 0.60, 0.35, 1.0))
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(hsla(200.0 / 360.0, 0.90, 0.70, 1.0))
                                .hover(|s| s.bg(hsla(200.0 / 360.0, 0.50, 0.25, 1.0)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_autofill.update(cx, |st, cx| {
                                        if let Some(ref mut m) = st.active_node_editor_modal {
                                            autofill_modal_params(m);
                                        }
                                        st.toast = Some(Toast::success(st.t("modal.autofill_success")));
                                        cx.notify();
                                    });
                                })
                                .child(format!("🪄 {}", state.t("modal.autofill"))),
                        )
                        // 排序参数按钮
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                                .border_1()
                                .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(hsla(240.0 / 360.0, 0.05, 0.85, 1.0))
                                .hover(|s| s.bg(hsla(240.0 / 360.0, 0.06, 0.18, 1.0)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_sort.update(cx, |st, cx| {
                                        if let Some(ref mut m) = st.active_node_editor_modal {
                                            m.properties.sort_by(|a, b| a.key.cmp(&b.key));
                                        }
                                        st.toast = Some(Toast::info(st.t("modal.sort_success")));
                                        cx.notify();
                                    });
                                })
                                .child(format!("↓↑ {}", state.t("modal.sort_params"))),
                        )
                        // 复制 JSON 按钮
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                                .border_1()
                                .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(hsla(240.0 / 360.0, 0.05, 0.85, 1.0))
                                .hover(|s| s.bg(hsla(240.0 / 360.0, 0.06, 0.18, 1.0)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_copy.update(cx, |st, cx| {
                                        st.toast = Some(Toast::success("📋 已复制参数 JSON 数据"));
                                        cx.notify();
                                    });
                                })
                                .child(format!("📋 {}", state.t("modal.copy_json"))),
                        )
                        // 添加参数按钮
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                                .border_1()
                                .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(hsla(240.0 / 360.0, 0.05, 0.85, 1.0))
                                .hover(|s| s.bg(hsla(240.0 / 360.0, 0.06, 0.18, 1.0)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_add.update(cx, |st, cx| {
                                        if let Some(ref mut m) = st.active_node_editor_modal {
                                            m.properties.push(PropertyItem {
                                                key: format!("Param_{}", m.properties.len() + 1),
                                                val: "0".to_string(),
                                                val_type: "Int".to_string(),
                                            });
                                        }
                                        cx.notify();
                                    });
                                })
                                .child(state.t("modal.add_param")),
                        ),
                ),
        )
        // 3. 参数表格
        .child(
            div()
                .flex()
                .flex_col()
                .border_1()
                .border_color(hsla(240.0 / 360.0, 0.06, 0.20, 1.0))
                .rounded_lg()
                .overflow_hidden()
                // 表头
                .child(
                    div()
                        .flex()
                        .h(px(32.0))
                        .bg(hsla(240.0 / 360.0, 0.06, 0.08, 1.0))
                        .border_b_1()
                        .border_color(hsla(240.0 / 360.0, 0.06, 0.20, 1.0))
                        .items_center()
                        .px_3()
                        .child(div().w(px(85.0)).text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(hsla(240.0 / 360.0, 0.05, 0.70, 1.0)).child(state.t("modal.type")))
                        .child(div().w(px(160.0)).text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(hsla(240.0 / 360.0, 0.05, 0.70, 1.0)).child(state.t("modal.key")))
                        .child(div().flex_1().text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(hsla(240.0 / 360.0, 0.05, 0.70, 1.0)).child(state.t("modal.val")))
                        .child(div().w(px(36.0)).text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(hsla(240.0 / 360.0, 0.05, 0.70, 1.0)).child(state.t("modal.action"))),
                )
                .children(if has_params {
                    modal.properties.iter().enumerate().map(|(idx, prop)| {
                        render_param_row(idx, prop, model)
                    }).collect()
                } else {
                    vec![div()
                        .p_4()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(11.5))
                        .text_color(hsla(240.0 / 360.0, 0.05, 0.55, 1.0))
                        .child(state.t("modal.no_params"))]
                }),
        )
}

/// 渲染单行参数编辑项 (含布尔 Checkbox 与删除操作)
fn render_param_row(idx: usize, prop: &PropertyItem, model: &Entity<AppState>) -> Div {

    let model_del = model.clone();
    let p_type = prop.val_type.clone();
    let p_key = prop.key.clone();
    let p_val = prop.val.clone();
    let is_bool = p_type.eq_ignore_ascii_case("bool");

    div()
        .flex()
        .h(px(38.0))
        .items_center()
        .px_3()
        .border_b_1()
        .border_color(hsla(240.0 / 360.0, 0.06, 0.16, 1.0))
        .bg(if idx.is_multiple_of(2) {
            hsla(240.0 / 360.0, 0.06, 0.11, 1.0)
        } else {
            hsla(240.0 / 360.0, 0.06, 0.09, 1.0)
        })

        .child(
            div().w(px(85.0)).pr_2().child(
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_md()
                    .bg(match p_type.to_lowercase().as_str() {
                        "bool" => hsla(215.0 / 360.0, 0.40, 0.22, 0.9),
                        "float" => hsla(200.0 / 360.0, 0.35, 0.20, 0.9),
                        "int" => hsla(160.0 / 360.0, 0.35, 0.20, 0.9),
                        _ => hsla(280.0 / 360.0, 0.35, 0.20, 0.9),
                    })
                    .text_size(px(10.5))
                    .font_weight(FontWeight::BOLD)
                    .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                    .child(p_type),
            ),
        )
        .child(
            div().w(px(160.0)).pr_2().child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .bg(hsla(240.0 / 360.0, 0.06, 0.15, 1.0))
                    .border_1()
                    .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                    .text_size(px(11.5))
                    .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                    .child(p_key),
            ),
        )
        .child(
            div().flex_1().pr_2().child(if is_bool {
                let is_true = p_val.eq_ignore_ascii_case("true") || p_val == "1";
                let model_toggle = model.clone();
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        cx.stop_propagation();
                        model_toggle.update(cx, |st, cx| {
                            if let Some(ref mut m) = st.active_node_editor_modal {
                                if idx < m.properties.len() {
                                    let cur = &m.properties[idx].val;
                                    let next = if cur.eq_ignore_ascii_case("true") || cur == "1" {
                                        "false"
                                    } else {
                                        "true"
                                    };
                                    m.properties[idx].val = next.to_string();
                                }
                            }
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .rounded_xs()
                            .bg(if is_true {
                                hsla(215.0 / 360.0, 0.90, 0.55, 1.0)
                            } else {
                                hsla(240.0 / 360.0, 0.06, 0.16, 1.0)
                            })
                            .border_1()
                            .border_color(hsla(240.0 / 360.0, 0.06, 0.30, 1.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(10.0))
                            .text_color(rgb(0xffffff))
                            .child(if is_true { "✓" } else { "" }),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(hsla(240.0 / 360.0, 0.05, 0.85, 1.0))
                            .child(if is_true { "true" } else { "false" }),
                    )
            } else {
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .bg(hsla(240.0 / 360.0, 0.06, 0.15, 1.0))
                    .border_1()
                    .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                    .text_size(px(11.5))
                    .text_color(hsla(240.0 / 360.0, 0.05, 0.85, 1.0))
                    .child(p_val)
            }),
        )
        .child(
            div()
                .w(px(36.0))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .p_1()
                        .rounded_md()
                        .cursor_pointer()
                        .hover(|s| s.bg(hsla(0.0, 0.70, 0.35, 0.9)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_del.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.active_node_editor_modal {
                                    if idx < m.properties.len() {
                                        m.properties.remove(idx);
                                    }
                                }
                                cx.notify();
                            });
                        })
                        .child("🗑️"),
                ),
        )
}
