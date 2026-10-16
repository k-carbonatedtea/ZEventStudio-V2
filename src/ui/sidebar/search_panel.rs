use gpui::*;
use crate::model::FileNode;
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::AppState;

/// 匹配搜索结果项
#[derive(Clone, Debug)]
pub struct SearchResultItem {
    pub name: String,
    pub path: String,
    pub dir_path: String,
    pub icon: &'static str,
}

/// 渲染 VS Code 风格的全局搜索侧边栏面板 (Global Search Panel)
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
    let include_filter = state.global_search_include.trim().to_string();
    let exclude_filter = state.global_search_exclude.trim().to_string();

    let model_input = model.clone();
    let model_clear = model.clone();
    let model_case = model.clone();
    let model_word = model.clone();
    let model_regex = model.clone();
    let model_filter_toggle = model.clone();
    let model_refresh = model.clone();

    // 收集匹配结果
    let results = if !query.is_empty() {
        search_project_files(
            &state.file_tree,
            &query,
            case_sensitive,
            whole_word,
            use_regex,
            &include_filter,
            &exclude_filter,
        )
    } else {
        Vec::new()
    };

    let result_count = results.len();

    div()
        .flex()
        .flex_col()
        .h_full()
        .w_full()
        .bg(pal.panel_surface)
        .overflow_hidden()

        // 1. 顶部搜索标题栏与操作按钮组
        .child(
            div()
                .h(px(36.0))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(pal.panel_border)
                .bg(pal.panel_surface)
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(11.0))
                        .text_color(pal.text_primary)
                        .child(state.t("search.title")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        // 刷新按钮
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .hover(move |s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_refresh.update(cx, |_, cx| cx.notify());
                                })
                                .child("🔄"),
                        )
                        // 清空输入按钮
                        .children((!query.is_empty()).then(|| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .hover(move |s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_clear.update(cx, |st, cx| {
                                        st.global_search_query.clear();
                                        st.global_search_focused = true;
                                        cx.notify();
                                    });
                                })
                                .child("✕")
                        }))
                        // 高级过滤器切换按钮 (⋯)
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(if show_filters {
                                    pal.primary_accent
                                } else {
                                    pal.text_secondary
                                })
                                .bg(if show_filters {
                                    pal.card_surface
                                } else {
                                    rgba(0x00000000)
                                })
                                .hover(move |s| s.bg(pal.card_hover))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_filter_toggle.update(cx, |st, cx| {
                                        st.global_search_show_filters = !st.global_search_show_filters;
                                        cx.notify();
                                    });
                                })
                                .child("⋯"),
                        ),
                ),
        )

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
                        .child(
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
                                        .text_color(if case_sensitive {
                                            pal.primary_accent
                                        } else {
                                            pal.text_secondary
                                        })
                                        .bg(if case_sensitive {
                                            pal.card_surface
                                        } else {
                                            rgba(0x00000000)
                                        })
                                        .hover(move |s| s.bg(pal.card_hover))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            model_case.update(cx, |st, cx| {
                                                st.global_search_case_sensitive = !st.global_search_case_sensitive;
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
                                        .text_color(if whole_word {
                                            pal.primary_accent
                                        } else {
                                            pal.text_secondary
                                        })
                                        .bg(if whole_word {
                                            pal.card_surface
                                        } else {
                                            rgba(0x00000000)
                                        })
                                        .hover(move |s| s.bg(pal.card_hover))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            model_word.update(cx, |st, cx| {
                                                st.global_search_whole_word = !st.global_search_whole_word;
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
                                        .text_color(if use_regex {
                                            pal.primary_accent
                                        } else {
                                            pal.text_secondary
                                        })
                                        .bg(if use_regex {
                                            pal.card_surface
                                        } else {
                                            rgba(0x00000000)
                                        })
                                        .hover(move |s| s.bg(pal.card_hover))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            model_regex.update(cx, |st, cx| {
                                                st.global_search_use_regex = !st.global_search_use_regex;
                                                cx.notify();
                                            });
                                        })
                                        .child(".*"),
                                ),
                        ),
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

        // 4. 搜索结果概览与列表渲染
        .child(
            div()
                .id("search_panel_scroll")
                .flex_1()
                .overflow_y_scroll()
                .child(if query.is_empty() {
                    // 空输入引导提示
                    div()
                        .p_4()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .text_center()
                        .child(
                            div()
                                .text_size(px(24.0))
                                .child("🔍"),
                        )
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(pal.text_secondary)
                                .child(state.t("search.empty_hint")),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("search.shortcut_hint")),
                        )
                        .into_any_element()
                } else if result_count == 0 {
                    // 无匹配结果
                    div()
                        .p_4()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(pal.text_secondary)
                                .child(format!("{}: \"{}\"", state.t("search.no_match"), query)),
                        )
                        .into_any_element()
                } else {
                    // 匹配结果列表
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .bg(pal.card_surface)
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(pal.text_secondary)
                                        .child(format!("{} ({} 项)", state.t("search.results_found"), result_count)),
                                ),
                        )
                        .child(
                            div()
                                .p_1()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .children(results.into_iter().map(|item| {
                                    let model_jump = model.clone();
                                    let path_to_load = item.path.clone();
                                    div()
                                        .px_2()
                                        .py_1p5()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(pal.card_hover))
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            let p = path_to_load.clone();
                                            model_jump.update(cx, |st, cx| {
                                                st.load_file_by_path(&p);
                                                cx.notify();
                                            });
                                        })
                                        .child(
                                            div()
                                                .text_size(px(13.0))
                                                .child(item.icon),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .overflow_hidden()
                                                .child(
                                                    div()
                                                        .text_size(px(11.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(pal.text_primary)
                                                        .overflow_hidden()
                                                        .whitespace_nowrap()
                                                        .child(item.name),
                                                )
                                                .children((!item.dir_path.is_empty()).then(|| {
                                                    div()
                                                        .text_size(px(9.5))
                                                        .text_color(pal.text_secondary)
                                                        .overflow_hidden()
                                                        .whitespace_nowrap()
                                                        .child(item.dir_path)
                                                })),
                                        )
                                })),
                        )
                        .into_any_element()
                }),
        )
}

/// 递归遍历项目文件树并提取匹配项
fn search_project_files(
    nodes: &[FileNode],
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
    include_filter: &str,
    exclude_filter: &str,
) -> Vec<SearchResultItem> {
    let mut results = Vec::new();
    collect_search_results(
        nodes,
        query,
        case_sensitive,
        whole_word,
        use_regex,
        include_filter,
        exclude_filter,
        &mut results,
    );
    results
}

fn collect_search_results(
    nodes: &[FileNode],
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
    include_filter: &str,
    exclude_filter: &str,
    out: &mut Vec<SearchResultItem>,
) {
    for node in nodes {
        if node.is_dir || node.is_sarc {
            if let Some(children) = &node.children {
                collect_search_results(
                    children,
                    query,
                    case_sensitive,
                    whole_word,
                    use_regex,
                    include_filter,
                    exclude_filter,
                    out,
                );
            }
        } else {
            // 叶子文件节点
            let name = &node.name;
            let path = &node.path;

            // 过滤器匹配
            if !include_filter.is_empty() && !name.to_lowercase().contains(&include_filter.to_lowercase()) {
                continue;
            }
            if !exclude_filter.is_empty() && name.to_lowercase().contains(&exclude_filter.to_lowercase()) {
                continue;
            }

            // 搜索关键字匹配
            let matched = if whole_word {
                is_whole_word_match(name, query, case_sensitive)
                    || is_whole_word_match(path, query, case_sensitive)
            } else if case_sensitive {
                name.contains(query) || path.contains(query)
            } else {
                name.to_lowercase().contains(&query.to_lowercase())
                    || path.to_lowercase().contains(&query.to_lowercase())
            };

            if matched {
                let dir_path = std::path::Path::new(path)
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .to_string();

                let icon = get_search_icon_by_name(name);
                out.push(SearchResultItem {
                    name: name.clone(),
                    path: path.clone(),
                    dir_path,
                    icon,
                });
            }

            if let Some(children) = &node.children {
                collect_search_results(
                    children,
                    query,
                    case_sensitive,
                    whole_word,
                    use_regex,
                    include_filter,
                    exclude_filter,
                    out,
                );
            }
        }
    }
}

/// 检查是否为全字单词匹配
fn is_whole_word_match(text: &str, pattern: &str, case_sensitive: bool) -> bool {
    let t = if case_sensitive { text.to_string() } else { text.to_lowercase() };
    let p = if case_sensitive { pattern.to_string() } else { pattern.to_lowercase() };

    let mut start = 0;
    while let Some(idx) = t[start..].find(&p) {
        let abs_idx = start + idx;
        let end_idx = abs_idx + p.len();

        let prev_ok = if abs_idx == 0 {
            true
        } else {
            !t[..abs_idx].chars().last().unwrap_or(' ').is_alphanumeric()
        };

        let next_ok = if end_idx >= t.len() {
            true
        } else {
            !t[end_idx..].chars().next().unwrap_or(' ').is_alphanumeric()
        };

        if prev_ok && next_ok {
            return true;
        }
        start = abs_idx + 1;
    }
    false
}

/// 根据文件名获取显示图标
fn get_search_icon_by_name(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.ends_with(".bfevfl") || lower.ends_with(".evfl") {
        "⚡"
    } else if lower.ends_with(".msbt") || lower.ends_with(".smsbt") {
        "💬"
    } else if lower.ends_with(".byml")
        || lower.ends_with(".bgyml")
        || lower.ends_with(".sbyml")
        || lower.ends_with(".aamp")
        || lower.ends_with(".bxml")
    {
        "⚙️"
    } else if lower.ends_with(".json") {
        "{}"
    } else if lower.ends_with(".yaml") || lower.ends_with(".yml") {
        "📜"
    } else if crate::model::file_tree::is_sarc_extension(name) {
        "📦"
    } else {
        "📄"
    }
}
