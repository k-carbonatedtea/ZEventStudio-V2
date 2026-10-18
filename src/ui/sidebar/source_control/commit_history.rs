use gpui::*;
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::AppState;
use super::commit_row::render_commit_row;
/// 渲染 Git 提交历史与分支图谱视图 (Git Graph & Commit History Section)
pub fn render_commit_history_section(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let is_collapsed = state.git_history_collapsed;
    let commits = state.git_commits.clone();
    let total_count = commits.len();
    let model_toggle = model.clone();
    let model_refresh = model.clone();
    let model_search_toggle = model.clone();
    let is_search_focused = state.git_history_search_focused;
    let search_query = state.git_history_search_query.clone();

    div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .border_t_1()
        .border_color(pal.panel_border)
        .bg(pal.panel_surface)

        // 1. 提交历史折叠分组标题栏 (Header)
        .child(
            div()
                .h(px(28.0))
                .px_2()
                .flex()
                .items_center()
                .justify_between()
                .bg(pal.card_surface)
                .border_b_1()
                .border_color(pal.panel_border)
                .flex_shrink_0()
                // 左侧折叠箭头 + 标题 + 数量徽章
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_toggle.update(cx, |st, cx| {
                                st.git_history_collapsed = !st.git_history_collapsed;
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_size(px(9.0))
                                .text_color(pal.text_secondary)
                                .child(if is_collapsed { "▶" } else { "▼" }),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_primary)
                                .child(state.t("source_control.history")),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(pal.card_hover)
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.primary_accent)
                                .child(total_count.to_string()),
                        ),
                )
                // 右侧操作按钮 (搜索切换、刷新)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        // 搜索按钮
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .cursor_pointer()
                                .text_size(px(10.0))
                                .text_color(if is_search_focused || !search_query.is_empty() {
                                    pal.primary_accent
                                } else {
                                    pal.text_secondary
                                })
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.primary_accent))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_search_toggle.update(cx, |st, cx| {
                                        st.git_history_search_focused = !st.git_history_search_focused;
                                        cx.notify();
                                    });
                                })
                                .child("🔍"),
                        )
                        // 刷新历史
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .cursor_pointer()
                                .text_size(px(10.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_refresh.update(cx, |st, cx| {
                                        st.refresh_git_history();
                                        cx.notify();
                                    });
                                })
                                .child("🔄"),
                        ),
                ),
        )

        // 2. 提交历史主体内容 (如果未折叠)
        .children((!is_collapsed).then(|| {
            let model_content = model.clone();
            let pal_c = *pal;
            let commits_list = commits.clone();
            let q = state.git_history_search_query.trim().to_lowercase();
            let filtered: Vec<_> = commits_list
                .iter()
                .filter(|c| {
                    if q.is_empty() {
                        true
                    } else {
                        c.subject.to_lowercase().contains(&q)
                            || c.author.to_lowercase().contains(&q)
                            || c.short_hash.to_lowercase().contains(&q)
                            || c.refs.iter().any(|r| r.to_lowercase().contains(&q))
                    }
                })
                .cloned()
                .collect();

            let model_search = model.clone();
            let model_search_out = model.clone();
            let model_clear_search = model.clone();
            let model_load_more = model.clone();
            let expanded_hash = state.git_expanded_commit_hash.clone();
            let expanded_files = state.git_expanded_commit_files.clone();
            let has_more = state.git_commits.len() >= state.git_history_limit;

            div()
                .flex_1()
                .flex()
                .flex_col()
                .overflow_hidden()

                // 2.1 搜索输入栏
                .child(
                    div()
                        .h(px(26.0))
                        .px_2()
                        .mx_1p5()
                        .my_1()
                        .rounded_sm()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(if is_search_focused {
                            pal.primary_accent
                        } else {
                            pal.panel_border
                        })
                        .flex()
                        .items_center()
                        .justify_between()
                        .cursor_text()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_search.update(cx, |st, cx| {
                                st.clear_active_text_inputs();
                                st.git_history_search_focused = true;
                                cx.notify();
                            });
                        })
                        .on_mouse_down_out(move |_, _, cx| {
                            model_search_out.update(cx, |st, cx| {
                                if st.git_history_search_focused {
                                    st.git_history_search_focused = false;
                                    cx.notify();
                                }
                            });
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(pal.text_secondary)
                                        .child("🔍"),
                                )
                                .child(
                                    if search_query.is_empty() && !is_search_focused {
                                        div()
                                            .text_size(px(9.5))
                                            .text_color(pal.text_secondary)
                                            .child(state.t("source_control.search_commits_placeholder"))
                                            .into_any_element()
                                    } else {
                                        div()
                                            .flex()
                                            .items_center()
                                            .text_size(px(9.5))
                                            .text_color(pal.text_primary)
                                            .child(search_query.clone())
                                            .children(is_search_focused.then(|| {
                                                render_blinking_cursor(pal_c.primary_accent.into()).into_any_element()
                                            }))
                                            .into_any_element()
                                    },
                                ),
                        )
                        .children((!search_query.is_empty()).then(|| {
                            div()
                                .px_1()
                                .cursor_pointer()
                                .text_size(px(9.5))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_clear_search.update(cx, |st, cx| {
                                        st.git_history_search_query.clear();
                                        cx.notify();
                                    });
                                })
                                .child("✕")
                        })),
                )

                // 2.2 提交列表渲染区
                .child(
                    div()
                        .id("git_history_scroll_list")
                        .flex_1()
                        .overflow_y_scroll()
                        .px_1()
                        .children(if filtered.is_empty() {
                            vec![
                                div()
                                    .p_4()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .gap_1()
                                    .text_color(pal.text_secondary)
                                    .child(div().text_size(px(16.0)).child("🌿"))
                                    .child(
                                        div()
                                            .text_size(px(10.0))
                                            .child(if !q.is_empty() {
                                                state.t("source_control.no_matching_commits")
                                            } else {
                                                state.t("source_control.no_commits")
                                            }),
                                    )
                                    .into_any_element(),
                            ]
                        } else {
                            let total_f = filtered.len();
                            let mut rows = Vec::new();

                            for (ix, item) in filtered.into_iter().enumerate() {
                                let is_exp = expanded_hash.as_deref() == Some(&item.hash);
                                rows.push(
                                    render_commit_row(
                                        item,
                                        ix,
                                        total_f,
                                        is_exp,
                                        &expanded_files,
                                        state,
                                        &model_content,
                                    )
                                    .into_any_element(),
                                );
                            }

                            // 2.3 加载更多按钮
                            if has_more {
                                rows.push(
                                    div()
                                        .w_full()
                                        .py_1p5()
                                        .my_1()
                                        .rounded_sm()
                                        .bg(pal_c.card_surface)
                                        .border_1()
                                        .border_color(pal_c.panel_border)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(pal_c.card_hover).border_color(pal_c.primary_accent))
                                        .text_align(TextAlign::Center)
                                        .text_size(px(10.0))
                                        .text_color(pal_c.primary_accent)
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            model_load_more.update(cx, |st, cx| {
                                                st.load_more_git_history();
                                                cx.notify();
                                            });
                                        })
                                        .child(state.t("source_control.load_more"))
                                        .into_any_element(),
                                );
                            }

                            rows
                        }),
                )
        }))
}
