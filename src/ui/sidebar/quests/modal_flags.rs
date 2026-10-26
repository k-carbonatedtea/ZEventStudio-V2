use gpui::*;
use crate::ui::sidebar::quests::input_helper::render_modal_text_field;
use crate::ui::state::{AppState, QuestCreatorFocusField, QuestCreatorModalState};

/// 渲染任务向导：Flag 标志位设定选项卡
pub fn render_tab_flags(
    modal: &QuestCreatorModalState,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_reset = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_3()
        // 1. 顶部机制提示与快速重置按钮
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .p_2()
                .rounded_sm()
                .bg(pal.card_hover.opacity(0.5))
                .border_1()
                .border_color(pal.panel_border.opacity(0.5))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("quests.flags_desc")),
                )
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded_xs()
                        .border_1()
                        .border_color(pal.panel_border)
                        .bg(pal.panel_surface)
                        .text_size(px(11.0))
                        .text_color(pal.primary_accent)
                        .cursor_pointer()
                        .hover(|s| s.bg(pal.card_hover))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_reset.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.quest_creator_modal {
                                    m.reset_flags_to_default();
                                    cx.notify();
                                }
                            });
                        })
                        .child(state.t("quests.btn_reset_flags")),
                ),
        )
        // 2. 核心三阶段标志位：Ready 与 Activated
        .child(
            div()
                .flex()
                .gap_3()
                .child(render_modal_text_field(
                    state.t("quests.field_ready_flag"),
                    &modal.ready_flag,
                    QuestCreatorFocusField::ReadyFlag,
                    "Quest_Ready",
                    state,
                    model,
                ))
                .child(render_modal_text_field(
                    state.t("quests.field_activated_flag"),
                    &modal.activated_flag,
                    QuestCreatorFocusField::ActivatedFlag,
                    "Quest_Activated",
                    state,
                    model,
                )),
        )
        // 3. 完成标志位与前置依赖标志位
        .child(
            div()
                .flex()
                .gap_3()
                .child(render_modal_text_field(
                    state.t("quests.field_finish_flag"),
                    &modal.finish_flag,
                    QuestCreatorFocusField::FinishFlag,
                    "Quest_Finish",
                    state,
                    model,
                ))
                .child(render_modal_text_field(
                    state.t("quests.field_dependency_flag"),
                    &modal.dependency_flag,
                    QuestCreatorFocusField::DependencyFlag,
                    "IsPlayed_Demo103_0 (可选)",
                    state,
                    model,
                )),
        )
        // 4. 前置标志说明提示
        .child(
            div()
                .text_size(px(11.0))
                .text_color(pal.text_secondary.opacity(0.8))
                .child(format!("💡 {}", state.t("quests.dependency_flag_tip"))),
        )
}
