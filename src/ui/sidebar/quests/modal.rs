use gpui::*;
use crate::ui::sidebar::quests::modal_coords::render_tab_coords;
use crate::ui::sidebar::quests::modal_deploy::render_tab_deploy;
use crate::ui::sidebar::quests::modal_flags::render_tab_flags;
use crate::ui::sidebar::quests::modal_general::render_tab_general;
use crate::ui::state::{AppState, QuestCreatorModalState, QuestCreatorTab};

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
                .w(px(580.0))
                .max_h(px(700.0))
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
                                        .child(if modal.is_edit_mode {
                                            format!("{}: {}", state.t("quests.modal_edit_title"), modal.name)
                                        } else {
                                            state.t("quests.modal_title").to_string()
                                        }),
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
                // 2. 选项卡分段切换栏
                .child(render_tab_nav_bar(modal, state, model))
                // 3. 表单内容滚动区 (根据激活选项卡切换)
                .child(
                    div()
                        .id("quest_creator_scroll_body")
                        .flex_1()
                        .overflow_y_scroll()
                        .p_4()
                        .child(match modal.active_tab {
                            QuestCreatorTab::General => render_tab_general(modal, state, model).into_any_element(),
                            QuestCreatorTab::Flags => render_tab_flags(modal, state, model).into_any_element(),
                            QuestCreatorTab::Coordinates => render_tab_coords(modal, state, model).into_any_element(),
                            QuestCreatorTab::Deploy => render_tab_deploy(modal, state, model).into_any_element(),
                        }),
                )
                // 4. 底部操作按钮栏
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_3()
                        .border_t_1()
                        .border_color(pal.panel_border)
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child("按 Tab 键切换字段，Esc 取消"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
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
                                        .child(if modal.is_edit_mode { "💾 " } else { "✨ " })
                                        .child(if modal.is_edit_mode {
                                            state.t("quests.btn_save_quest")
                                        } else {
                                            state.t("quests.btn_create_quest")
                                        }),
                                ),
                        ),
                ),
        )
}

/// 渲染选项卡分段导航栏
fn render_tab_nav_bar(
    modal: &QuestCreatorModalState,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let current_tab = modal.active_tab;

    let tabs = [
        (QuestCreatorTab::General, state.t("quests.tab_general"), "🏷️"),
        (QuestCreatorTab::Flags, state.t("quests.tab_flags"), "🚩"),
        (QuestCreatorTab::Coordinates, state.t("quests.tab_coords"), "📍"),
        (QuestCreatorTab::Deploy, state.t("quests.tab_deploy"), "🚀"),
    ];

    div()
        .flex()
        .items_center()
        .px_4()
        .pt_2()
        .pb_1()
        .border_b_1()
        .border_color(pal.panel_border.opacity(0.6))
        .gap_1()
        .children(tabs.into_iter().map(|(tab, label, icon)| {
            let is_active = current_tab == tab;
            let model_click = model.clone();
            div()
                .flex_1()
                .py_1p5()
                .rounded_sm()
                .border_b_2()
                .border_color(if is_active { pal.primary_accent } else { rgba(0x00000000) })
                .bg(if is_active { pal.primary_accent.opacity(0.10) } else { pal.panel_surface })
                .text_color(if is_active { pal.primary_accent } else { pal.text_secondary })
                .text_size(px(11.5))
                .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .gap_1p5()
                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_click.update(cx, |st, cx| {
                        if let Some(ref mut m) = st.quest_creator_modal {
                            m.active_tab = tab;
                            cx.notify();
                        }
                    });
                })
                .child(icon)
                .child(label)
        }))
}
