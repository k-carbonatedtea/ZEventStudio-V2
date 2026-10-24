use gpui::*;
use crate::model::quest::QuestType;
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::{AppState, QuestCreatorFocusField, QuestCreatorModalState};

/// 渲染新建任务向导模态框
pub fn render_quest_creator_modal(
    modal: &QuestCreatorModalState,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_close = model.clone();
    let model_backdrop = model.clone();
    let model_cancel = model.clone();
    let model_submit = model.clone();

    div()
        .id("quest_creator_modal_overlay")
        .absolute()
        .inset_0()
        .bg(rgba(0x00000088))
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_backdrop.update(cx, |st, cx| {
                st.close_quest_creator();
                cx.notify();
            });
        })
        // 模态弹窗主体 (防止遮罩点击冒泡)
        .child(
            div()
                .id("quest_creator_modal_window")
                .w(px(560.0))
                .max_h(px(680.0))
                .rounded_lg()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.primary_accent.opacity(0.35))
                .shadow_lg()
                .flex()
                .flex_col()
                .overflow_hidden()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                // 1. 弹窗标题栏
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_3()
                        .border_b_1()
                        .border_color(pal.panel_border)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().text_size(px(16.0)).child("📜"))
                                .child(
                                    div()
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(pal.text_primary)
                                        .child(state.t("quests.modal_title")),
                                ),
                        )
                        .child(
                            div()
                                .size(px(24.0))
                                .rounded_sm()
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(13.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_close.update(cx, |st, cx| {
                                        st.close_quest_creator();
                                        cx.notify();
                                    });
                                })
                                .child("✕"),
                        ),
                )
                // 2. 表单内容滚动区
                .child(
                    div()
                        .id("quest_creator_scroll_body")
                        .flex_1()
                        .overflow_y_scroll()
                        .p_4()
                        .flex()
                        .flex_col()
                        .gap_3()
                        // 任务类型选择 (Main / Sub / Mini)
                        .child(render_type_selector(modal, state, model))
                        // 任务标识符与界面标题
                        .child(
                            div()
                                .flex()
                                .gap_3()
                                .child(render_input_field(
                                    state.t("quests.field_id"),
                                    &modal.name,
                                    modal.focused_field == QuestCreatorFocusField::Name,
                                    QuestCreatorFocusField::Name,
                                    "Quest_MyChallenge",
                                    state,
                                    model,
                                ))
                                .child(render_input_field(
                                    state.t("quests.field_title"),
                                    &modal.display_title,
                                    modal.focused_field == QuestCreatorFocusField::DisplayTitle,
                                    QuestCreatorFocusField::DisplayTitle,
                                    "小樱桃的新挑战",
                                    state,
                                    model,
                                )),
                        )
                        // 委托人与区域
                        .child(
                            div()
                                .flex()
                                .gap_3()
                                .child(render_input_field(
                                    state.t("quests.field_orderer"),
                                    &modal.orderer,
                                    modal.focused_field == QuestCreatorFocusField::Orderer,
                                    QuestCreatorFocusField::Orderer,
                                    "Npc_AncientDoctor_Robbie",
                                    state,
                                    model,
                                ))
                                .child(render_input_field(
                                    state.t("quests.field_location"),
                                    &modal.location,
                                    modal.focused_field == QuestCreatorFocusField::Location,
                                    QuestCreatorFocusField::Location,
                                    "Akkala",
                                    state,
                                    model,
                                )),
                        )
                        // 冒险笔记：接取初始描述
                        .child(render_input_field(
                            state.t("quests.field_desc"),
                            &modal.desc_text,
                            modal.focused_field == QuestCreatorFocusField::DescText,
                            QuestCreatorFocusField::DescText,
                            "在古代研究所接受了小樱桃的委托...",
                            state,
                            model,
                        ))
                        // 冒险笔记：完成总结描述
                        .child(render_input_field(
                            state.t("quests.field_finish"),
                            &modal.finish_text,
                            modal.focused_field == QuestCreatorFocusField::FinishText,
                            QuestCreatorFocusField::FinishText,
                            "向小樱桃汇报了成果，圆满完成了挑战！",
                            state,
                            model,
                        ))
                        // 自动化 5 支柱生成勾选项
                        .child(render_automation_options(modal, state, model)),
                )
                // 3. 底部操作按钮栏
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .px_4()
                        .py_3()
                        .border_t_1()
                        .border_color(pal.panel_border)
                        .gap_2()
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded_sm()
                                .border_1()
                                .border_color(pal.panel_border)
                                .text_size(px(12.0))
                                .text_color(pal.text_secondary)
                                .cursor_pointer()
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_cancel.update(cx, |st, cx| {
                                        st.close_quest_creator();
                                        cx.notify();
                                    });
                                })
                                .child(state.t("quests.btn_cancel")),
                        )
                        .child(
                            div()
                                .id("btn_submit_quest_creation")
                                .px_4()
                                .py_1p5()
                                .rounded_sm()
                                .bg(pal.primary_accent)
                                .text_color(rgb(0x000000))
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .hover(|s| s.opacity(0.88))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_submit.update(cx, |st, cx| {
                                        if let Err(e) = st.apply_quest_creation() {
                                            st.toast = Some(crate::ui::components::Toast::error(e));
                                        }
                                        cx.notify();
                                    });
                                })
                                .child("✨")
                                .child(state.t("quests.btn_create_quest")),
                        ),
                ),
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

/// 渲染遵循 8.3 范式的文本输入项
fn render_input_field(
    label: &str,
    value: &str,
    is_focused: bool,
    field: QuestCreatorFocusField,
    placeholder: &str,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_click = model.clone();
    let model_out = model.clone();
    let val_clone = value.to_string();
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
                .h(px(30.0))
                .px_2()
                .rounded_sm()
                .border_1()
                .border_color(if is_focused { pal.primary_accent } else { pal.panel_border })
                .bg(pal.panel_surface)
                .flex()
                .items_center()
                // 1. 点击获得焦点并设置自身字段
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_click.update(cx, |st, cx| {
                        st.clear_active_text_inputs();
                        if let Some(ref mut m) = st.quest_creator_modal {
                            m.focused_field = field;
                        }
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
                    if val_clone.is_empty() && !is_focused {
                        div()
                            .text_size(px(12.0))
                            .text_color(pal.text_secondary.opacity(0.6))
                            .child(placeholder_str)
                            .into_any_element()
                    } else {
                        div()
                            .flex()
                            .items_center()
                            .text_size(px(12.0))
                            .text_color(pal.text_primary)
                            .child(val_clone)
                            .children(is_focused.then(|| {
                                render_blinking_cursor(pal.primary_accent.into()).into_any_element()
                            }))
                            .into_any_element()
                    },
                ),
        )
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
