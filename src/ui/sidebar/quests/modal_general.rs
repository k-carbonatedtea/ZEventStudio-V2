use gpui::*;
use crate::model::quest::QuestType;
use crate::ui::sidebar::quests::input_helper::render_modal_text_field;
use crate::ui::state::{AppState, QuestCreatorFocusField, QuestCreatorModalState};

/// 渲染任务向导：常规基础信息选项卡
pub fn render_tab_general(
    modal: &QuestCreatorModalState,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        // 1. 任务类型选择 (Main, Sub, Mini)
        .child(render_type_selector(modal, state, model))
        // 2. 任务标识符与界面标题
        .child(
            div()
                .flex()
                .gap_3()
                .child(render_modal_text_field(
                    state.t("quests.field_id"),
                    &modal.name,
                    QuestCreatorFocusField::Name,
                    "Quest_CustomMiniQuest",
                    state,
                    model,
                ))
                .child(render_modal_text_field(
                    state.t("quests.field_title"),
                    &modal.display_title,
                    QuestCreatorFocusField::DisplayTitle,
                    "自定义挑战任务",
                    state,
                    model,
                )),
        )
        // 3. 委托人与归属区域
        .child(
            div()
                .flex()
                .gap_3()
                .child(render_modal_text_field(
                    state.t("quests.field_orderer"),
                    &modal.orderer,
                    QuestCreatorFocusField::Orderer,
                    "Npc_AncientDoctor_Robbie",
                    state,
                    model,
                ))
                .child(render_modal_text_field(
                    state.t("quests.field_location"),
                    &modal.location,
                    QuestCreatorFocusField::Location,
                    "Akkala",
                    state,
                    model,
                )),
        )
}

/// 渲染任务类型选择器 (Main, Sub, Mini)
fn render_type_selector(
    modal: &QuestCreatorModalState,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let current_type = modal.quest_type;

    let items = [
        (QuestType::Mini, state.t("quests.type_mini")),
        (QuestType::Sub, state.t("quests.type_sub")),
        (QuestType::Main, state.t("quests.type_main")),
    ];

    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_size(px(11.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(pal.text_secondary)
                .child(state.t("quests.field_type")),
        )
        .child(
            div()
                .flex()
                .gap_2()
                .children(items.into_iter().map(|(t, label)| {
                    let is_active = current_type == t;
                    let model_click = model.clone();
                    div()
                        .flex_1()
                        .py_1p5()
                        .rounded_sm()
                        .border_1()
                        .border_color(if is_active { pal.primary_accent } else { pal.panel_border })
                        .bg(if is_active { pal.primary_accent.opacity(0.12) } else { pal.panel_surface })
                        .text_color(if is_active { pal.primary_accent } else { pal.text_secondary })
                        .text_size(px(12.0))
                        .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_click.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.quest_creator_modal {
                                    m.quest_type = t;
                                    cx.notify();
                                }
                            });
                        })
                        .child(label)
                })),
        )
}
