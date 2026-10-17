use gpui::*;
use std::sync::Arc;
use crate::model::FileNode;
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::AppState;

const MAX_SEARCH_RESULTS: usize = 500;

/// 匹配搜索结果项
#[derive(Clone, Debug)]
pub struct SearchResultItem {
    pub name: String,
    pub path: String,
    pub dir_path: String,
    pub icon: &'static str,
    pub is_sarc: bool,
}

impl AppState {
    /// 触发全局搜索结果增量计算并缓存 (毫秒级响应，避免每帧渲染反复遍历)
    pub fn update_global_search_results(&mut self) {
        let query = self.global_search_query.trim();
        if query.is_empty() {
            self.global_search_results.clear();
            return;
        }

        self.global_search_results = search_project_files(
            &self.file_tree,
            query,
            self.global_search_case_sensitive,
            self.global_search_whole_word,
            self.global_search_use_regex,
            self.global_search_include.trim(),
            self.global_search_exclude.trim(),
            self.project_path.as_deref(),
        );
    }
}

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
    let model_clear = model.clone();
    let model_case = model.clone();
    let model_word = model.clone();
    let model_regex = model.clone();
    let model_filter_toggle = model.clone();
    let model_refresh = model.clone();

    let result_count = state.global_search_results.len();

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
                                    model_refresh.update(cx, |st, cx| {
                                        st.update_global_search_results();
                                        cx.notify();
                                    });
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
                                        st.update_global_search_results();
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
                                                st.update_global_search_results();
                                                cx.notify();
                                            });
                                        })
                                        .child(".*"),
                                )
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

        // 4. 搜索结果概览与虚拟化列表渲染 (统一采用 uniform_list 实现极速丝滑渲染)
        .child(
            div()
                .id("search_panel_scroll_area")
                .flex_1()
                .overflow_hidden()
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

/// 渲染单行全局搜索结果条目 (固定高度 36px，满足 uniform_list 约束)
fn render_search_row(
    item: &SearchResultItem,
    is_selected: bool,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let model_jump = model.clone();
    let path_to_load = item.path.clone();
    let is_sarc = item.is_sarc;
    let name = item.name.clone();
    let dir_path = item.dir_path.clone();
    let icon = item.icon;

    div()
        .h(px(36.0))
        .w_full()
        .px_2()
        .rounded_md()
        .cursor_pointer()
        .bg(if is_selected {
            pal.card_surface
        } else {
            rgba(0x00000000)
        })
        .border_l_2()
        .border_color(if is_selected {
            pal.primary_accent
        } else {
            rgba(0x00000000)
        })
        .hover(move |s| {
            if is_selected {
                s.bg(pal.card_hover)
            } else {
                s.bg(pal.card_surface)
            }
        })
        .flex()
        .items_center()
        .gap_2()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let p = path_to_load.clone();
            model_jump.update(cx, |st, cx| {
                st.clear_active_text_inputs();
                if !is_sarc {
                    st.load_file_by_path(&p);
                    // 从搜索界面打开文件：保持在搜索界面，不跳转回资源管理器界面
                    st.reveal_file_in_tree(&p, false);
                } else {
                    // 若点击纯 SARC 封包容器节点，在资源管理器中定位展开
                    st.reveal_file_in_tree(&p, true);
                }
                cx.notify();
            });
        })
        .child(
            div()
                .w(px(16.0))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(12.0))
                .child(icon),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .overflow_hidden()
                .justify_center()
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_primary)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(name),
                )
                .children((!dir_path.is_empty()).then(|| {
                    div()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(dir_path)
                })),
        )
}

/// 递归遍历项目文件树并提取匹配项 (毫秒级极速检索与精准智能排序)
pub fn search_project_files(
    nodes: &[FileNode],
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
    include_filter: &str,
    exclude_filter: &str,
    project_root: Option<&str>,
) -> Vec<SearchResultItem> {
    let mut results = Vec::new();
    let query_has_separator = query.contains('/') || query.contains('\\');

    let compiled_regex = if use_regex {
        regex::RegexBuilder::new(query)
            .case_insensitive(!case_sensitive)
            .build()
            .ok()
    } else {
        None
    };

    let query_lower = if case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };

    collect_search_results_fast(
        nodes,
        query,
        &query_lower,
        query_has_separator,
        case_sensitive,
        whole_word,
        compiled_regex.as_ref(),
        include_filter,
        exclude_filter,
        project_root,
        &mut results,
    );

    // 智能相关性排序：
    // 1. 文件名前缀匹配排在最前 (例如搜 "lin"，"Link.bfevfl" 优于 "Player_Link.bfevfl")
    // 2. 文件名包含匹配次之
    // 3. 相对路径匹配排在后面
    results.sort_by(|a, b| {
        let a_starts = a.name.to_lowercase().starts_with(&query_lower);
        let b_starts = b.name.to_lowercase().starts_with(&query_lower);
        if a_starts != b_starts {
            return b_starts.cmp(&a_starts);
        }
        let a_name_has = a.name.to_lowercase().contains(&query_lower);
        let b_name_has = b.name.to_lowercase().contains(&query_lower);
        if a_name_has != b_name_has {
            return b_name_has.cmp(&a_name_has);
        }
        a.name.len().cmp(&b.name.len())
    });

    results
}

fn collect_search_results_fast(
    nodes: &[FileNode],
    query: &str,
    query_lower: &str,
    query_has_separator: bool,
    case_sensitive: bool,
    whole_word: bool,
    compiled_regex: Option<&regex::Regex>,
    include_filter: &str,
    exclude_filter: &str,
    project_root: Option<&str>,
    out: &mut Vec<SearchResultItem>,
) {
    if out.len() >= MAX_SEARCH_RESULTS {
        return;
    }

    for node in nodes {
        if out.len() >= MAX_SEARCH_RESULTS {
            return;
        }

        if node.is_dir {
            // 普通目录：直接递归子节点，无多余分配
            if let Some(children) = &node.children {
                collect_search_results_fast(
                    children,
                    query,
                    query_lower,
                    query_has_separator,
                    case_sensitive,
                    whole_word,
                    compiled_regex,
                    include_filter,
                    exclude_filter,
                    project_root,
                    out,
                );
            }
        } else if node.is_sarc {
            // SARC 封包节点：检查封包名本身是否匹配
            if matches_filters_fast(&node.name, include_filter, exclude_filter) {
                let matched = check_name_match(
                    &node.name,
                    query,
                    query_lower,
                    case_sensitive,
                    whole_word,
                    compiled_regex,
                );
                if matched {
                    let (_, display_dir) = parse_display_dir(&node.path, project_root);
                    out.push(SearchResultItem {
                        name: node.name.clone(),
                        path: node.path.clone(),
                        dir_path: display_dir,
                        icon: "📦",
                        is_sarc: true,
                    });
                }
            }

            // 递归遍历 SARC 内部文件
            if let Some(children) = &node.children {
                collect_search_results_fast(
                    children,
                    query,
                    query_lower,
                    query_has_separator,
                    case_sensitive,
                    whole_word,
                    compiled_regex,
                    include_filter,
                    exclude_filter,
                    project_root,
                    out,
                );
            }
        } else {
            // 叶子数据文件 (物理文件或 SARC 内部文件)
            if !matches_filters_fast(&node.name, include_filter, exclude_filter) {
                continue;
            }

            // 1. 优先极速检查文件名 (0 堆内存分配)
            let name_matched = check_name_match(
                &node.name,
                query,
                query_lower,
                case_sensitive,
                whole_word,
                compiled_regex,
            );

            // 2. 若文件名未命中，且用户输入了路径分隔符 '/' 或 '\\'，才检查相对路径
            let path_matched = if !name_matched && query_has_separator {
                let (rel_path, _) = parse_display_dir(&node.path, project_root);
                check_name_match(
                    &rel_path,
                    query,
                    query_lower,
                    case_sensitive,
                    whole_word,
                    compiled_regex,
                )
            } else {
                false
            };

            if name_matched || path_matched {
                let (_, display_dir) = parse_display_dir(&node.path, project_root);
                let icon = get_search_icon_by_name(&node.name);
                out.push(SearchResultItem {
                    name: node.name.clone(),
                    path: node.path.clone(),
                    dir_path: display_dir,
                    icon,
                    is_sarc: false,
                });
            }
        }
    }
}

/// 解析用于展示的相对目录路径 (仅在匹配成功时调用，绝不在遍历树时反复调用)
fn parse_display_dir(path: &str, project_root: Option<&str>) -> (String, String) {
    if path.starts_with("SARC:") {
        let clean = path.trim_start_matches("SARC:");
        if let Some((sarc_path, inner_path)) = clean.split_once("//") {
            let sarc_name = std::path::Path::new(sarc_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(sarc_path);
            let inner_parent = std::path::Path::new(inner_path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .replace('\\', "/");
            let display_dir = if inner_parent.is_empty() {
                sarc_name.to_string()
            } else {
                format!("{} › {}", sarc_name, inner_parent)
            };
            return (inner_path.replace('\\', "/"), display_dir);
        }
    }

    let norm_path = path.replace('\\', "/");
    let (rel_path, display_dir) = if let Some(root) = project_root {
        let norm_root = root.replace('\\', "/");
        if let Some(stripped) = norm_path.strip_prefix(&norm_root) {
            let clean_rel = stripped.trim_start_matches('/');
            let parent_dir = std::path::Path::new(clean_rel)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .replace('\\', "/");
            (clean_rel.to_string(), parent_dir)
        } else {
            let parent_dir = std::path::Path::new(path)
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            (norm_path.clone(), parent_dir)
        }
    } else {
        let parent_dir = std::path::Path::new(path)
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        (norm_path.clone(), parent_dir)
    };

    (rel_path, display_dir)
}

/// 检查名称是否匹配查询条件
fn check_name_match(
    name: &str,
    query: &str,
    query_lower: &str,
    case_sensitive: bool,
    whole_word: bool,
    compiled_regex: Option<&regex::Regex>,
) -> bool {
    if let Some(re) = compiled_regex {
        return re.is_match(name);
    }

    if whole_word {
        is_whole_word_match(name, query, case_sensitive)
    } else if case_sensitive {
        name.contains(query)
    } else {
        name.to_lowercase().contains(query_lower)
    }
}

/// 快速检查包含/排除过滤条件
fn matches_filters_fast(
    name: &str,
    include_filter: &str,
    exclude_filter: &str,
) -> bool {
    if !include_filter.is_empty() {
        let inc = include_filter.to_lowercase();
        let inc_clean = inc.trim_start_matches('*');
        if !name.to_lowercase().contains(inc_clean) {
            return false;
        }
    }

    if !exclude_filter.is_empty() {
        let exc = exclude_filter.to_lowercase();
        let exc_clean = exc.trim_start_matches('*');
        if name.to_lowercase().contains(exc_clean) {
            return false;
        }
    }

    true
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
            let ch = t[..abs_idx].chars().last().unwrap_or(' ');
            !ch.is_alphanumeric() && ch != '_'
        };

        let next_ok = if end_idx >= t.len() {
            true
        } else {
            let ch = t[end_idx..].chars().next().unwrap_or(' ');
            !ch.is_alphanumeric() && ch != '_'
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
        || lower.ends_with(".baev")
        || lower.ends_with(".baglbl")
        || lower.ends_with(".bglcl")
        || lower.ends_with(".bactparam")
        || lower.ends_with(".baiprog")
        || lower.ends_with(".sbaiprog")
        || lower.ends_with(".bgparamlist")
        || lower.ends_with(".brecipe")
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
