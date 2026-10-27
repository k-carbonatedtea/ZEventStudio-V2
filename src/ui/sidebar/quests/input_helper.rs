use gpui::*;
use crate::ui::components::TextInput;
use crate::ui::state::{AppState, QuestCreatorFocusField};

/// 渲染遵循规范 8.3 与通用组件标准的任务向导文本输入字段
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
            TextInput::new(
                SharedString::from(format!("input_field_{:?}", field)),
                value,
                is_focused,
                model.clone(),
            )
            .placeholder(placeholder.to_string())
            .on_focus(move |st| {
                if let Some(ref mut m) = st.quest_creator_modal {
                    m.focused_field = field;
                }
            })
            .on_blur(move |st| {
                if let Some(ref mut m) = st.quest_creator_modal {
                    if m.focused_field == field {
                        m.focused_field = QuestCreatorFocusField::None;
                    }
                }
            })
            .on_clear(move |st| {
                if let Some(ref mut m) = st.quest_creator_modal {
                    if let Some(target) = m.get_mut_focused_field(field) {
                        target.clear();
                    }
                    if field == QuestCreatorFocusField::Name {
                        m.sync_flags_from_name();
                    }
                }
            }),
        )
}

