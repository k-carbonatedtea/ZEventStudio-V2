use gpui::*;
use crate::model::PropertyItem;
use crate::ui::canvas::node_editor::{ActiveEditField, NodeEditorModalState};
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::AppState;

/// 渲染单行参数编辑项 (类型切换、键编辑、值编辑与删除)
pub fn render_param_row(
    idx: usize,
    prop: &PropertyItem,
    modal: &NodeEditorModalState,
    model: &Entity<AppState>,
) -> Div {
    let model_type = model.clone();
    let model_key = model.clone();
    let model_val = model.clone();
    let model_del = model.clone();

    let p_type = prop.val_type.clone();
    let p_key = prop.key.clone();
    let p_val = prop.val.clone();
    let is_bool = p_type.eq_ignore_ascii_case("bool");

    let is_key_active = modal.active_field == ActiveEditField::ParamKey(idx);
    let is_val_active = modal.active_field == ActiveEditField::ParamVal(idx);

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
        // 1. 类型切换徽章 (点击切换类型: string -> int -> float -> bool -> string)
        .child(
            div().w(px(85.0)).pr_2().child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .cursor_pointer()
                    .bg(match p_type.to_lowercase().as_str() {
                        "bool" => hsla(215.0 / 360.0, 0.40, 0.22, 0.9),
                        "float" => hsla(185.0 / 360.0, 0.40, 0.20, 0.9),
                        "int" => hsla(160.0 / 360.0, 0.35, 0.20, 0.9),
                        _ => hsla(280.0 / 360.0, 0.35, 0.20, 0.9),
                    })
                    .border_1()
                    .border_color(match p_type.to_lowercase().as_str() {
                        "bool" => hsla(215.0 / 360.0, 0.60, 0.40, 0.8),
                        "float" => hsla(185.0 / 360.0, 0.60, 0.40, 0.8),
                        "int" => hsla(160.0 / 360.0, 0.60, 0.35, 0.8),
                        _ => hsla(280.0 / 360.0, 0.60, 0.40, 0.8),
                    })
                    .hover(|s| s.opacity(0.85))
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        cx.stop_propagation();
                        model_type.update(cx, |st, cx| {
                            if let Some(ref mut m) = st.active_node_editor_modal {
                                if idx < m.properties.len() {
                                    let cur_t = m.properties[idx].val_type.to_lowercase();
                                    let (next_t, next_v) = match cur_t.as_str() {
                                        "string" => ("int".to_string(), "0".to_string()),
                                        "int" => ("float".to_string(), "0.0".to_string()),
                                        "float" => ("bool".to_string(), "true".to_string()),
                                        "bool" => ("string".to_string(), "".to_string()),
                                        _ => ("string".to_string(), "".to_string()),
                                    };
                                    m.properties[idx].val_type = next_t;
                                    if cur_t == "bool" || m.properties[idx].val_type == "bool" {
                                        m.properties[idx].val = next_v;
                                    }
                                }
                            }
                            cx.notify();
                        });
                    })
                    .text_size(px(11.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(p_type)
                    .child(div().text_size(px(9.0)).text_color(hsla(0.0, 0.0, 1.0, 0.6)).child("▾")),
            ),
        )
        // 2. 键输入框 (Key Input)
        .child(
            div().w(px(160.0)).pr_2().child(
                div()
                    .h(px(28.0))
                    .px_2()
                    .rounded_md()
                    .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                    .border_1()
                    .border_color(if is_key_active {
                        hsla(215.0 / 360.0, 0.90, 0.55, 1.0)
                    } else {
                        hsla(240.0 / 360.0, 0.06, 0.22, 1.0)
                    })
                    .cursor_text()
                    .flex()
                    .items_center()
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        cx.stop_propagation();
                        model_key.update(cx, |st, cx| {
                            if let Some(ref mut m) = st.active_node_editor_modal {
                                m.active_field = ActiveEditField::ParamKey(idx);
                            }
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(if p_key.is_empty() && !is_key_active {
                                hsla(240.0 / 360.0, 0.05, 0.45, 1.0)
                            } else {
                                hsla(0.0, 0.0, 1.0, 1.0)
                            })
                            .child(if p_key.is_empty() && !is_key_active {
                                "Key".to_string()
                            } else {
                                p_key
                            }),
                    )
                    .children(is_key_active.then(|| {
                        render_blinking_cursor(hsla(215.0 / 360.0, 0.90, 0.55, 1.0))
                    })),
            ),
        )
        // 3. 值输入框 (Value Input / Bool Checkbox)
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
                            .text_color(hsla(0.0, 0.0, 1.0, 1.0))
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
                    .h(px(28.0))
                    .px_2()
                    .rounded_md()
                    .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                    .border_1()
                    .border_color(if is_val_active {
                        hsla(215.0 / 360.0, 0.90, 0.55, 1.0)
                    } else {
                        hsla(240.0 / 360.0, 0.06, 0.22, 1.0)
                    })
                    .cursor_text()
                    .flex()
                    .items_center()
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        cx.stop_propagation();
                        model_val.update(cx, |st, cx| {
                            if let Some(ref mut m) = st.active_node_editor_modal {
                                m.active_field = ActiveEditField::ParamVal(idx);
                            }
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(if p_val.is_empty() && !is_val_active {
                                hsla(240.0 / 360.0, 0.05, 0.45, 1.0)
                            } else {
                                hsla(0.0, 0.0, 1.0, 1.0)
                            })
                            .child(if p_val.is_empty() && !is_val_active {
                                "Value".to_string()
                            } else {
                                p_val
                            }),
                    )
                    .children(is_val_active.then(|| {
                        render_blinking_cursor(hsla(215.0 / 360.0, 0.90, 0.55, 1.0))
                    }))
            }),
        )
        // 4. 删除按钮
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
