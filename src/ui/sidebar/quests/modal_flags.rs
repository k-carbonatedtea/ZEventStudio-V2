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
    let model_add = model.clone();

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
        // 5. 分割线
        .child(div().h(px(1.0)).bg(pal.panel_border))
        // 6. 扩展阶段 / 状态 Flag 标题与 + 添加 Flag 按钮
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(div().text_size(px(13.0)).child("🚩"))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(pal.text_primary)
                                .child(state.t("quests.extra_flags_header")),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .px_1p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(pal.card_hover)
                                .text_color(pal.text_secondary)
                                .child(format!("{}", modal.extra_flags.len())),
                        ),
                )
                .child(
                    div()
                        .id("btn_add_extra_flag")
                        .h(px(24.0))
                        .px_2p5()
                        .rounded_sm()
                        .bg(pal.primary_accent.opacity(0.15))
                        .border_1()
                        .border_color(pal.primary_accent.opacity(0.4))
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(pal.primary_accent)
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_1()
                        .hover(|s| s.bg(pal.primary_accent.opacity(0.28)).border_color(pal.primary_accent))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_add.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.quest_creator_modal {
                                    m.add_extra_flag();
                                    cx.notify();
                                }
                            });
                        })
                        .child("➕")
                        .child(state.t("quests.btn_add_flag")),
                ),
        )
        // 7. 扩展 Flag 列表项 / 空状态展示
        .child(if modal.extra_flags.is_empty() {
            div()
                .p_3()
                .rounded_sm()
                .border_1()
                .border_color(pal.panel_border.opacity(0.6))
                .bg(pal.card_hover.opacity(0.3))
                .flex()
                .items_center()
                .justify_center()
                .text_center()
                .text_size(px(11.0))
                .text_color(pal.text_secondary.opacity(0.8))
                .child(state.t("quests.extra_flags_empty"))
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .gap_2()
                .children(modal.extra_flags.iter().enumerate().map(|(idx, extra_flag)| {
                    let model_del = model.clone();
                    let label = format!("{} #{}", state.t("quests.field_extra_flag"), idx + 1);
                    let placeholder = format!("Quest_Step{}", idx + 1);

                    div()
                        .id(SharedString::from(format!("extra_flag_row_{}", idx)))
                        .flex()
                        .items_end()
                        .gap_2()
                        .child(
                            div().flex_1().child(render_modal_text_field(
                                &label,
                                extra_flag,
                                QuestCreatorFocusField::ExtraFlag(idx),
                                &placeholder,
                                state,
                                model,
                            )),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("btn_del_extra_flag_{}", idx)))
                                .h(px(32.0))
                                .px_2p5()
                                .rounded_sm()
                                .border_1()
                                .border_color(pal.panel_border)
                                .bg(pal.panel_surface)
                                .text_size(px(12.0))
                                .text_color(pal.text_secondary)
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .hover(|s| s.bg(rgb(0x3f1515)).text_color(rgb(0xff6b6b)).border_color(rgb(0xff6b6b)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_del.update(cx, |st, cx| {
                                        if let Some(ref mut m) = st.quest_creator_modal {
                                            m.remove_extra_flag(idx);
                                            cx.notify();
                                        }
                                    });
                                })
                                .child("✕"),
                        )
                }))
                .into_any_element()
        })
}
