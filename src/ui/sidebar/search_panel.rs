use gpui::*;
use std::sync::Arc;
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::AppState;
use super::search_row::{render_search_header, render_search_row};
pub use crate::model::{MAX_SEARCH_RESULTS, SearchResultItem};

/// 渲染 VS Code 风格的全局搜索侧边栏面板 (采用高性能虚拟列表 uniform_list 渲染)
pub fn render_search_panel(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let query = state.global_search_query.trim().to_string();
    let is_focused = state.global_search_focused;
    let case_sensitive = state.global_search_case_sensitive;
    let whole_word = state.global_search_whole_word;
    let use_regex = state.global_search_use_regex;
    let show_filters = state.global_search_show_filters;

    let model_input = model.clone();
    let model_input_out = model.clone();

    let result_count = state.global_search_results.len();

    div()
        .flex()
        .flex_col()
        .h_full()
        .w_full()
        .bg(pal.panel_surface)
        .overflow_hidden()

        // 1. 顶部搜索标题栏与操作按钮组
        .child(render_search_header(state, &query, show_filters, model, &pal))

        // 2. 搜索输入区与匹配模式开关
        .child(
            div()
                .p_2p5()
                .flex()
                .flex_col()
                .gap_2()
                .border_b_1()
                .border_color(pal.panel_border)

                // 搜索输入框
                .child(
                    div()
                        .w_full()
                        .h(px(28.0))
                        .px_2()
                        .rounded_md()
                        .bg(pal.app_bg)
                        .border_1()
                        .border_color(if is_focused {
                            pal.primary_accent
                        } else {
                            pal.panel_border
                        })
                        .flex()
                        .items_center()
                        .justify_between()
                        .cursor_text()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_input.update(cx, |st, cx| {
                                st.global_search_focused = true;
                                st.file_search_focused = false;
                                cx.notify();
                            });
                        })
                        .on_mouse_down_out(move |_, _, cx| {
                            model_input_out.update(cx, |st, cx| {
                                if st.global_search_focused {
                                    st.global_search_focused = false;
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
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(pal.text_secondary)
                                        .child("🔍"),
                                )
                                .children(if query.is_empty() && !is_focused {
                                    vec![
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(pal.text_secondary)
                                            .child(state.t("search.placeholder"))
                                            .into_any_element(),
                                    ]
                                } else {
                                    let mut elems = vec![
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(pal.text_primary)
                                            .child(state.global_search_query.clone())
                                            .into_any_element(),
                                    ];
                                    if is_focused {
                                        elems.push(render_blinking_cursor(pal.primary_accent.into()).into_any_element());
                                    }
                                    elems
                                }),
                        )
                        // 右侧内嵌模式切换按钮组
                        .child(render_search_mode_toggles(case_sensitive, whole_word, use_regex, model, &pal)),
                )

                // 3. 高级筛选过滤器面板 (展开时展示包含/排除文件模式)
                .children(show_filters.then(|| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .pt_1()
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("search.filters_hint")),
                        )
                })),
        )

        // 4. 搜索结果概览与虚拟化列表渲染 (统一采用 uniform_list 实现极速丝滑渲染)
        .child(
            div()
                .id("search_panel_scroll_area")
                .flex_1()
                .overflow_hidden()
                .child(if query.is_empty() || result_count == 0 {
                    render_search_empty_state(&query, result_count, state, &pal).into_any_element()
                } else {
                    // 匹配结果虚拟化列表
                    let flat_entries = Arc::new(state.global_search_results.clone());
                    let model_clone = model.clone();
                    let current_selected_path = state.selected_file_path.clone();
                    let count_label = if result_count >= MAX_SEARCH_RESULTS {
                        format!("{} ({} 项+)", state.t("search.results_found"), MAX_SEARCH_RESULTS)
                    } else {
                        format!("{} ({} 项)", state.t("search.results_found"), result_count)
                    };

                    div()
                        .flex()
                        .flex_col()
                        .h_full()
                        .child(
                            div()
                                .h(px(26.0))
                                .px_3()
                                .bg(pal.card_surface)
                                .flex()
                                .items_center()
                                .justify_between()
                                .border_b_1()
                                .border_color(pal.panel_border)
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(pal.text_secondary)
                                        .child(count_label),
                                 ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .overflow_hidden()
                                .p_1()
                                .child(
                                    uniform_list(
                                        "search_panel_virtual_list",
                                        result_count,
                                        move |range, _window, _cx| {
                                            let entries = flat_entries.clone();
                                            let m = model_clone.clone();
                                            let cur_selected = current_selected_path.clone();
                                            range
                                                .map(|ix| {
                                                    if let Some(item) = entries.get(ix) {
                                                        let is_sel = cur_selected.as_deref() == Some(&item.path);
                                                        render_search_row(item, is_sel, &m, &pal).into_any_element()
                                                    } else {
                                                        div().into_any_element()
                                                    }
                                                })
                                                .collect()
                                        },
                                    )
                                    .track_scroll(&state.global_search_scroll_handle)
                                    .size_full()
                                ),
                        )
                        .into_any_element()
                }),
        )
}


/// 渲染搜索模式切换按钮组 (大小写敏感、全字匹配、正则表达式)
fn render_search_mode_toggles(
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let model_case = model.clone();
    let model_word = model.clone();
    let model_regex = model.clone();

    div()
        .flex()
        .items_center()
        .gap_0p5()
        // 大小写敏感 (Aa)
        .child(
            div()
                .px_1()
                .py_0p5()
                .rounded_xs()
                .cursor_pointer()
                .text_size(px(10.0))
                .font_weight(FontWeight::BOLD)
                .text_color(if case_sensitive { pal.primary_accent } else { pal.text_secondary })
                .bg(if case_sensitive { pal.card_surface } else { rgba(0x00000000) })
                .hover(move |s| s.bg(pal.card_hover))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_case.update(cx, |st, cx| {
                        st.global_search_case_sensitive = !st.global_search_case_sensitive;
                        st.update_global_search_results();
                        cx.notify();
                    });
                })
                .child("Aa"),
        )
        // 全字匹配 (|Ab|)
        .child(
            div()
                .px_1()
                .py_0p5()
                .rounded_xs()
                .cursor_pointer()
                .text_size(px(9.5))
                .font_weight(FontWeight::BOLD)
                .text_color(if whole_word { pal.primary_accent } else { pal.text_secondary })
                .bg(if whole_word { pal.card_surface } else { rgba(0x00000000) })
                .hover(move |s| s.bg(pal.card_hover))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_word.update(cx, |st, cx| {
                        st.global_search_whole_word = !st.global_search_whole_word;
                        st.update_global_search_results();
                        cx.notify();
                    });
                })
                .child("|Ab|"),
        )
        // 正则匹配 (.*)
        .child(
            div()
                .px_1()
                .py_0p5()
                .rounded_xs()
                .cursor_pointer()
                .text_size(px(9.5))
                .font_weight(FontWeight::BOLD)
                .text_color(if use_regex { pal.primary_accent } else { pal.text_secondary })
                .bg(if use_regex { pal.card_surface } else { rgba(0x00000000) })
                .hover(move |s| s.bg(pal.card_hover))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_regex.update(cx, |st, cx| {
                        st.global_search_use_regex = !st.global_search_use_regex;
                        st.update_global_search_results();
                        cx.notify();
                    });
                })
                .child(".*"),
        )
}

/// 渲染全局搜索无输入或无匹配结果空状态引导
fn render_search_empty_state(
    query: &str,
    _result_count: usize,
    state: &AppState,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    if query.is_empty() {
        div()
            .p_4()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .text_center()
            .child(div().text_size(px(24.0)).child("🔍"))
            .child(div().text_size(px(11.5)).text_color(pal.text_secondary).child(state.t("search.empty_hint")))
            .child(div().text_size(px(10.0)).text_color(pal.text_secondary).child(state.t("search.shortcut_hint")))
    } else {
        div()
            .p_4()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .child(div().text_size(px(12.0)).text_color(pal.text_secondary).child(format!("{}: \"{}\"", state.t("search.no_match"), query)))
    }
}
