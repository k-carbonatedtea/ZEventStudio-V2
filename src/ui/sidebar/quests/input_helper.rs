use gpui::*;
use crate::ui::components::render_input_text;
use crate::ui::state::{AppState, QuestCreatorFocusField};

/// 渲染遵循规范 8.3 的通用任务向导文本输入字段
pub fn render_modal_text_field(
    label: &str,
    value: &str,
    field: QuestCreatorFocusField,
    placeholder: &str,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let is_focused = state.quest_creator_modal.as_ref().is_some_and(|m| m.focused_field == field);
    let model_click = model.clone();
    let model_out = model.clone();
    let model_clear = model.clone();
    let val_clone = value.to_string();
    let val_len = val_clone.chars().count();
    let placeholder_str = placeholder.to_string();

    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_size(px(11.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(pal.text_secondary)
                .child(label.to_string()),
        )
        .child(
            div()
                .id(SharedString::from(format!("input_field_{:?}", field)))
                .cursor_text()
                .h(px(32.0))
                .px_2()
                .rounded_sm()
                .border_1()
                .border_color(if is_focused { pal.primary_accent } else { pal.panel_border })
                .bg(pal.panel_surface)
                .flex()
                .items_center()
                .justify_between()
                // 1. 点击获得焦点并初始化光标至文本末尾 (后续字符级交互由 render_input_text 接管)
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_click.update(cx, |st, cx| {
                        st.clear_active_text_inputs();
                        if let Some(ref mut m) = st.quest_creator_modal {
                            m.focused_field = field;
                        }
                        st.text_input_cursor = val_len;
                        st.text_input_select_anchor = None;
                        cx.notify();
                    });
                })
                // 2. 点击外部必须自动失焦 (规范 8.3 严禁遗漏！)
                .on_mouse_down_out(move |_, _, cx| {
                    model_out.update(cx, |st, cx| {
                        if let Some(ref mut m) = st.quest_creator_modal {
                            if m.focused_field == field {
                                m.focused_field = QuestCreatorFocusField::None;
                                cx.notify();
                            }
                        }
                    });
                })
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .overflow_hidden()
                        .child(if val_clone.is_empty() && !is_focused {
                            div()
                                .text_size(px(12.0))
                                .text_color(pal.text_secondary.opacity(0.6))
                                .child(placeholder_str)
                                .into_any_element()
                        } else {
                            render_input_text(&val_clone, is_focused, state, model).into_any_element()
                        }),
                )
                // 3. 非空时渲染一键清空按钮
                .children((!val_clone.is_empty()).then(|| {
                    div()
                        .cursor_pointer()
                        .px_1()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_clear.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.quest_creator_modal {
                                    if let Some(target) = m.get_mut_focused_field(field) {
                                        target.clear();
                                    }
                                    if field == QuestCreatorFocusField::Name {
                                        m.sync_flags_from_name();
                                    }
                                    st.text_input_cursor = 0;
                                    st.text_input_select_anchor = None;
                                    cx.notify();
                                }
                            });
                        })
                        .child("✕")
                })),
        )
}
