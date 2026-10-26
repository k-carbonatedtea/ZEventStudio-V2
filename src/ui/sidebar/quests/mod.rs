pub mod input_helper;
pub mod modal;
pub mod modal_coords;
pub mod modal_deploy;
pub mod modal_flags;
pub mod modal_general;
pub mod modal_raw_data;
pub mod modal_raw_format;
pub mod modal_state;
pub mod quest_card;

use gpui::*;
use crate::model::quest::QuestType;
use crate::ui::components::render_blinking_cursor;
use crate::ui::sidebar::quests::quest_card::render_expandable_quest_card;
use crate::ui::state::AppState;

/// 渲染侧边栏任务制作与管理面板 (采用高性能虚拟列表 uniform_list 架构)
pub fn render_quests_panel(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let model_refresh = model.clone();
    let model_new = model.clone();

    let filtered_quests = state.get_filtered_quests();

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(pal.panel_surface)
        .overflow_hidden()
        // 1. 面板顶部标题栏与快捷操作
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(pal.panel_border)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_primary)
                                .child(state.t("quests.panel_title")),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .px_1p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(pal.card_hover)
                                .text_color(pal.text_secondary)
                                .child(format!("{}", state.quests_list.len())),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        // 刷新按钮
                        .child(
                            div()
                                .id("btn_refresh_quests")
                                .size(px(24.0))
                                .rounded_sm()
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(12.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_refresh.update(cx, |st, cx| {
                                        st.refresh_quests_list();
                                        st.toast = Some(crate::ui::components::Toast::info(st.t("quests.refreshed")));
                                        cx.notify();
                                    });
                                })
                                .child("🔄"),
                        )
                        // 新建任务向导按钮
                        .child(
                            div()
                                .id("btn_open_quest_creator")
                                .h(px(24.0))
                                .px_2()
                                .rounded_sm()
                                .bg(pal.primary_accent)
                                .text_color(rgb(0x000000))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .gap_1()
                                .text_size(px(11.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .hover(|s| s.opacity(0.88))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_new.update(cx, |st, cx| {
                                        st.open_quest_creator();
                                        cx.notify();
                                    });
                                })
                                .child("➕")
                                .child(state.t("quests.btn_new_quest")),
                        ),
                ),
        )
        // 2. 搜索框 (严格遵循规范 8.3)
        .child(render_quest_search_box(state, model))
        // 3. 任务类型过滤 Chips
        .child(render_type_filter_chips(state, model))
        // 4. 任务列表内容区 (采用 gpui::list 虚拟列表渲染，仅绘制视口可见项，未滚动区域零开销，支持卡片就地折叠展开)
        .child(
            div()
                .id("quests_list_scroll_body")
                .flex_1()
                .overflow_hidden()
                .p_1()
                .children(if filtered_quests.is_empty() {
                    vec![render_empty_quests_guide(state, model).into_any_element()]
                } else {
                    let total_count = filtered_quests.len();
                    if state.quests_list_state.item_count() != total_count {
                        state.quests_list_state.reset(total_count);
                    }

                    let quests_arc = std::sync::Arc::new(
                        filtered_quests.into_iter().cloned().collect::<Vec<_>>()
                    );
                    let m_clone = model.clone();
                    let lang = state.lang;
                    let pal_theme = pal;
                    let sel_id = state.quests_selected_id.clone();

                    vec![
                        list(
                            state.quests_list_state.clone(),
                            move |index, _window, _cx| {
                                if let Some(quest) = quests_arc.get(index) {
                                    let is_sel = sel_id.as_deref() == Some(&quest.name);
                                    render_expandable_quest_card(quest, is_sel, lang, pal_theme, &m_clone)
                                        .into_any_element()
                                } else {
                                    div().into_any_element()
                                }
                            },
                        )
                        .size_full()
                        .into_any_element()
                    ]
                }),
        )
}

/// 渲染任务列表搜索输入框
fn render_quest_search_box(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let is_focused = state.quests_search_focused;
    let query = state.quests_search_query.clone();

    let model_click = model.clone();
    let model_out = model.clone();
    let model_clear = model.clone();

    div()
        .px_2()
        .pt_2()
        .pb_1()
        .child(
            div()
                .id("quests_search_container")
                .cursor_text()
                .h(px(28.0))
                .px_2()
                .rounded_md()
                .border_1()
                .border_color(if is_focused { pal.primary_accent } else { pal.panel_border })
                .bg(pal.panel_surface)
                .flex()
                .items_center()
                .justify_between()
                // 点击聚焦
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_click.update(cx, |st, cx| {
                        st.clear_active_text_inputs();
                        st.quests_search_focused = true;
                        cx.notify();
                    });
                })
                // 点击外部失焦 (必须具备！)
                .on_mouse_down_out(move |_, _, cx| {
                    model_out.update(cx, |st, cx| {
                        if st.quests_search_focused {
                            st.quests_search_focused = false;
                            cx.notify();
                        }
                    });
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .flex_1()
                        .child(div().text_size(px(11.0)).text_color(pal.text_secondary).child("🔍"))
                        .child(
                            if query.is_empty() && !is_focused {
                                div()
                                    .text_size(px(11.5))
                                    .text_color(pal.text_secondary)
                                    .child(state.t("quests.search_placeholder"))
                                    .into_any_element()
                            } else {
                                div()
                                    .flex()
                                    .items_center()
                                    .text_size(px(11.5))
                                    .text_color(pal.text_primary)
                                    .child(query.clone())
                                    .children(is_focused.then(|| {
                                        render_blinking_cursor(pal.primary_accent.into()).into_any_element()
                                    }))
                                    .into_any_element()
                            },
                        ),
                )
                // 一键清空按钮
                .children((!query.is_empty()).then(|| {
                    div()
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.text_color(pal.text_primary))
                        .child("✕")
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_clear.update(cx, |st, cx| {
                                st.quests_search_query.clear();
                                cx.notify();
                            });
                        })
                })),
        )
}

/// 渲染类型筛选 Chips
fn render_type_filter_chips(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let current_filter = state.quests_filter_type;

    let types = [
        (None, state.t("quests.filter_all")),
        (Some(QuestType::Main), state.t("quests.filter_main")),
        (Some(QuestType::Sub), state.t("quests.filter_sub")),
        (Some(QuestType::Mini), state.t("quests.filter_mini")),
    ];

    div()
        .flex()
        .items_center()
        .px_2()
        .pb_1p5()
        .gap_1()
        .children(types.into_iter().map(|(q_type, label)| {
            let is_active = current_filter == q_type;
            let model_click = model.clone();
            div()
                .id(SharedString::from(format!("chip_filter_{:?}", q_type)))
                .px_2()
                .py_0p5()
                .rounded_sm()
                .text_size(px(10.5))
                .cursor_pointer()
                .bg(if is_active { pal.primary_accent } else { pal.card_hover })
                .text_color(if is_active { rgb(0x000000) } else { pal.text_secondary })
                .hover(|s| s.opacity(0.85))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_click.update(cx, |st, cx| {
                        st.quests_filter_type = q_type;
                        cx.notify();
                    });
                })
                .child(label)
        }))
}

/// 渲染无任务时的引导提示
fn render_empty_quests_guide(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let model_new = model.clone();

    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .py_10()
        .px_4()
        .gap_2()
        .text_center()
        .child(div().text_size(px(28.0)).child("📜"))
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(pal.text_primary)
                .child(state.t("quests.empty_title")),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(pal.text_secondary)
                .child(state.t("quests.empty_desc")),
        )
        .child(
            div()
                .mt_2()
                .px_3()
                .py_1p5()
                .rounded_md()
                .bg(pal.primary_accent)
                .text_color(rgb(0x000000))
                .text_size(px(11.5))
                .font_weight(FontWeight::SEMIBOLD)
                .cursor_pointer()
                .hover(|s| s.opacity(0.88))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_new.update(cx, |st, cx| {
                        st.open_quest_creator();
                        cx.notify();
                    });
                })
                .child(state.t("quests.btn_create_first_quest")),
        )
}
