use gpui::*;
use crate::ui::sidebar::quests::input_helper::render_modal_text_field;
use crate::ui::state::{AppState, QuestCreatorFocusField, QuestCreatorModalState};

/// 渲染任务向导：冒险笔记文案与五大支柱部署选项卡
pub fn render_tab_deploy(
    modal: &QuestCreatorModalState,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_not_post = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_3()
        // 1. 冒险笔记：接取初始描述
        .child(render_modal_text_field(
            state.t("quests.field_desc"),
            &modal.desc_text,
            QuestCreatorFocusField::DescText,
            "在古代研究所接受了小樱桃的委托...",
            state,
            model,
        ))
        // 2. 冒险笔记：完成总结描述
        .child(render_modal_text_field(
            state.t("quests.field_finish"),
            &modal.finish_text,
            QuestCreatorFocusField::FinishText,
            "向小樱桃汇报了成果，圆满完成了挑战！",
            state,
            model,
        ))
        // 3. 不在冒险笔记记录开关 (NotPostNote)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_not_post.update(cx, |st, cx| {
                        if let Some(ref mut m) = st.quest_creator_modal {
                            m.not_post_note = !m.not_post_note;
                            cx.notify();
                        }
                    });
                })
                .child(
                    div()
                        .size(px(14.0))
                        .rounded_sm()
                        .border_1()
                        .border_color(if modal.not_post_note { pal.primary_accent } else { pal.panel_border })
                        .bg(if modal.not_post_note { pal.primary_accent } else { pal.panel_surface })
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(10.0))
                        .text_color(rgb(0x000000))
                        .child(if modal.not_post_note { "✓" } else { "" }),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(if modal.not_post_note { pal.primary_accent } else { pal.text_secondary })
                        .child(state.t("quests.opt_not_post_note")),
                ),
        )
        // 4. 自动化 5 支柱生成勾选项
        .child(render_automation_options(modal, state, model))
}

/// 渲染 5 支柱自动化生成勾选项
fn render_automation_options(
    modal: &QuestCreatorModalState,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();

    let options = [
        (modal.inject_titlebg, state.t("quests.opt_titlebg"), 0),
        (modal.inject_bootup_flags, state.t("quests.opt_bootup_flags"), 1),
        (modal.inject_ql_msbt, state.t("quests.opt_ql_msbt"), 2),
        (modal.scaffold_graph_nodes, state.t("quests.opt_scaffold_graph"), 3),
    ];

    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .p_2p5()
        .rounded_sm()
        .bg(pal.card_hover.opacity(0.5))
        .border_1()
        .border_color(pal.panel_border.opacity(0.5))
        .child(
            div()
                .text_size(px(11.5))
                .font_weight(FontWeight::BOLD)
                .text_color(pal.text_primary)
                .child(state.t("quests.automation_header")),
        )
        .children(options.into_iter().map(|(checked, label, idx)| {
            let model_click = model.clone();
            div()
                .flex()
                .items_center()
                .gap_2()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_click.update(cx, |st, cx| {
                        if let Some(ref mut m) = st.quest_creator_modal {
                            match idx {
                                0 => m.inject_titlebg = !m.inject_titlebg,
                                1 => m.inject_bootup_flags = !m.inject_bootup_flags,
                                2 => m.inject_ql_msbt = !m.inject_ql_msbt,
                                3 => m.scaffold_graph_nodes = !m.scaffold_graph_nodes,
                                _ => {}
                            }
                            cx.notify();
                        }
                    });
                })
                .child(
                    div()
                        .size(px(14.0))
                        .rounded_sm()
                        .border_1()
                        .border_color(if checked { pal.primary_accent } else { pal.panel_border })
                        .bg(if checked { pal.primary_accent } else { pal.panel_surface })
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(10.0))
                        .text_color(rgb(0x000000))
                        .child(if checked { "✓" } else { "" }),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(pal.text_secondary)
                        .child(label),
                )
        }))
}
