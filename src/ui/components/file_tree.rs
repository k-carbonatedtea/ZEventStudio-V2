use gpui::*;
use crate::model::FileNode;
use crate::ui::state::{AppState, FileFilterType, FileSortMode};
use std::sync::Arc;

/// 扁平化展示项 (用于虚拟化列表快速分片渲染)
#[derive(Clone)]
pub struct FlatFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_sarc: bool,
    pub level: usize,
    pub is_expanded: bool,
    pub is_selected: bool,
    pub is_modified: bool,
}

/// 渲染文件资源管理器顶部排序与类型过滤工具栏
pub fn render_file_search_and_sort_bar(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_sort = model.clone();

    let sort_mode = state.file_sort_mode;
    let sort_menu_open = state.file_tree_sort_menu_open;
    let active_filter = state.file_tree_filter_type;

    div()
        .id("file_search_and_sort_toolbar")
        .px_2()
        .pt_2()
        .pb_1p5()
        .flex()
        .flex_col()
        .gap_1p5()
        .border_b_1()
        .border_color(pal.panel_border)
        .bg(pal.panel_surface)
        // 2. 排序与操作栏
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_1()
                // 排序模式切换按钮
                .child(
                    div()
                        .id("file_sort_btn")
                        .px_1p5()
                        .py_0p5()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .hover(move |s| s.bg(pal.card_hover))
                        .cursor_pointer()
                        .border_1()
                        .border_color(if sort_menu_open {
                            pal.primary_accent
                        } else {
                            pal.panel_border
                        })
                        .flex()
                        .items_center()
                        .gap_1()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_sort.update(cx, |st, cx| {
                                st.file_tree_sort_menu_open = !st.file_tree_sort_menu_open;
                                st.file_search_focused = false;
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(pal.primary_accent)
                                .child("⇅"),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(pal.text_primary)
                                .child(format!("{} {}", sort_mode.icon(), state.t(sort_mode.label_key()))),
                        )
                        .child(
                            div()
                                .text_size(px(8.0))
                                .text_color(pal.text_secondary)
                                .child(if sort_menu_open { "▴" } else { "▾" }),
                        ),
                )
                // 过滤状态指示与重置快捷按钮 (当有类型过滤时显示)
                .children((active_filter != FileFilterType::All).then(|| {
                    let model_reset_all = model.clone();
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .bg(pal.card_surface)
                                .hover(move |s| s.bg(pal.card_hover))
                                .cursor_pointer()
                                .text_size(px(9.5))
                                .text_color(pal.secondary_accent)
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_reset_all.update(cx, |st, cx| {
                                        st.file_tree_filter_type = FileFilterType::All;
                                        cx.notify();
                                    });
                                })
                                .child("↺ 重置"),
                        )
                })),
        )
        // 2.1 排序菜单展开项 (展开在下方)
        .children(sort_menu_open.then(|| {
            let sort_options = [
                FileSortMode::NameAsc,
                FileSortMode::NameDesc,
                FileSortMode::FileType,
                FileSortMode::ModifiedFirst,
            ];
            div()
                .id("file_sort_menu_card")
                .p_1()
                .rounded_md()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.panel_border)
                .shadow_lg()
                .flex()
                .flex_col()
                .gap_0p5()
                .children(sort_options.into_iter().map(|mode| {
                    let is_active = mode == sort_mode;
                    let model_item = model.clone();
                    div()
                        .px_2()
                        .py_1()
                        .rounded_xs()
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_between()
                        .bg(if is_active {
                            pal.card_hover
                        } else {
                            rgba(0x00000000)
                        })
                        .hover(move |s| s.bg(pal.card_hover))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_item.update(cx, |st, cx| {
                                st.file_sort_mode = mode;
                                st.file_tree_sort_menu_open = false;
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .text_size(px(10.5))
                                .text_color(if is_active {
                                    pal.primary_accent
                                } else {
                                    pal.text_primary
                                })
                                .child(mode.icon())
                                .child(state.t(mode.label_key())),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(pal.primary_accent)
                                .child(if is_active { "✓" } else { " " }),
                        )
                }))
        }))
        // 3. 快速类型过滤标签 (Horizontal filter chips)
        .child(
            div()
                .id("file_filter_chips_row")
                .flex()
                .items_center()
                .gap_1()
                .overflow_x_hidden()
                .children({
                    const FILTERS: [FileFilterType; 6] = [
                        FileFilterType::All,
                        FileFilterType::EventFlow,
                        FileFilterType::Msbt,
                        FileFilterType::Param,
                        FileFilterType::Sarc,
                        FileFilterType::Modified,
                    ];
                    FILTERS.into_iter().map(|ft| {
                        let is_active = ft == active_filter;
                        let model_ft = model.clone();
                        div()
                            .px_1p5()
                            .py_0p5()
                            .rounded_md()
                            .cursor_pointer()
                            .text_size(px(9.5))
                            .font_weight(if is_active {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .bg(if is_active {
                                pal.primary_accent
                            } else {
                                pal.card_surface
                            })
                            .text_color(if is_active {
                                pal.panel_surface
                            } else {
                                pal.text_secondary
                            })
                            .border_1()
                            .border_color(if is_active {
                                pal.primary_accent
                            } else {
                                pal.panel_border
                            })
                            .hover(move |s| {
                                if !is_active {
                                    s.bg(pal.card_hover).text_color(pal.text_primary)
                                } else {
                                    s
                                }
                            })
                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                model_ft.update(cx, |st, cx| {
                                    st.file_tree_filter_type = ft;
                                    st.file_tree_sort_menu_open = false;
                                    cx.notify();
                                });
                            })
                            .child(state.t(ft.label_key()))
                    })
                }),
        )
}

/// 渲染文件资源管理器树状组件 (采用扁平虚拟化渲染，解决海量文件栈溢出与卡顿)
pub fn render_file_tree(
    nodes: &[FileNode],
    state: &AppState,
    model: &Entity<AppState>,
    _level: usize,
) -> impl IntoElement {
    let filter_type = state.file_tree_filter_type;
    let is_filtering = filter_type != FileFilterType::All;
    let selected_path = state.selected_file_path.as_deref().unwrap_or("");
    let pal = state.theme();

    let mut matching_paths = std::collections::HashSet::new();
    if is_filtering {
        collect_matching_paths(
            nodes,
            filter_type,
            state,
            &mut matching_paths,
        );
    }

    let mut flat_entries = Vec::new();
    flatten_visible_nodes(
        nodes,
        &state.expanded_tree_paths,
        is_filtering,
        &matching_paths,
        selected_path,
        state,
        0,
        &mut flat_entries,
    );

    let count = flat_entries.len();
    if count == 0 {
        let model_reset = model.clone();
        return div()
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
                    .child(if !is_filtering {
                        state.t("welcome.no_folder_hint")
                    } else {
                        state.t("explorer.no_match")
                    }),
            )
            .children(is_filtering.then(|| {
                div()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(pal.card_surface)
                    .hover(|s| s.bg(pal.card_hover))
                    .cursor_pointer()
                    .text_size(px(11.0))
                    .text_color(pal.primary_accent)
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        model_reset.update(cx, |st, cx| {
                            st.file_search_query.clear();
                            st.file_tree_filter_type = FileFilterType::All;
                            cx.notify();
                        });
                    })
                    .child(state.t("explorer.reset_filter"))
            }))
            .into_any_element();
    }

    let flat_entries = Arc::new(flat_entries);
    let model_clone = model.clone();

    uniform_list("file_tree_virtual_list", count, move |range, _window, _cx| {
        let entries = flat_entries.clone();
        let m = model_clone.clone();
        range
            .map(|ix| {
                if let Some(item) = entries.get(ix) {
                    render_flat_file_row(item, &m, &pal).into_any_element()
                } else {
                    div().into_any_element()
                }
            })
            .collect()
    })
    .track_scroll(&state.file_tree_scroll_handle)
    .size_full()
    .into_any_element()
}

/// 检查文件节点是否匹配指定的类型筛选器
fn node_matches_filter_type(node: &FileNode, filter_type: FileFilterType, state: &AppState) -> bool {
    match filter_type {
        FileFilterType::All => true,
        FileFilterType::EventFlow => {
            let lower = node.name.to_lowercase();
            lower.ends_with(".bfevfl") || lower.ends_with(".evfl")
        }
        FileFilterType::Msbt => {
            let lower = node.name.to_lowercase();
            lower.ends_with(".msbt") || lower.ends_with(".smsbt")
        }
        FileFilterType::Param => {
            let lower = node.name.to_lowercase();
            lower.ends_with(".byml")
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
                || lower.ends_with(".json")
                || lower.ends_with(".yaml")
                || lower.ends_with(".yml")
        }
        FileFilterType::Sarc => {
            node.is_sarc || crate::model::file_tree::is_sarc_extension(&node.name)
        }
        FileFilterType::Modified => {
            !node.is_dir && state.is_file_modified(&node.path)
        }
    }
}

/// 收集所有自身或后代匹配类型过滤器的节点路径 (单次 O(N) 遍历)
fn collect_matching_paths(
    nodes: &[FileNode],
    filter_type: FileFilterType,
    state: &AppState,
    matching_paths: &mut std::collections::HashSet<String>,
) -> bool {
    let mut any_match = false;
    for node in nodes {
        let is_container = node.is_dir || node.is_sarc;

        let type_matches = if is_container {
            true
        } else {
            node_matches_filter_type(node, filter_type, state)
        };

        let self_leaf_matches = !is_container && type_matches;

        let children_match = if let Some(children) = &node.children {
            collect_matching_paths(
                children,
                filter_type,
                state,
                matching_paths,
            )
        } else {
            false
        };

        let container_passes = is_container && children_match;

        if self_leaf_matches || container_passes {
            matching_paths.insert(node.path.clone());
            any_match = true;
        }
    }
    any_match
}

/// 自然排序比较器 (支持文件名中的数字自然排序，如 0100 vs 0101、file1 vs file2 vs file10)
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    while let (Some(&ca), Some(&cb)) = (a_chars.peek(), b_chars.peek()) {
        if ca.is_ascii_digit() && cb.is_ascii_digit() {
            let mut num_a: u64 = 0;
            while let Some(&d) = a_chars.peek() {
                if d.is_ascii_digit() {
                    num_a = num_a.saturating_mul(10).saturating_add((d as u8 - b'0') as u64);
                    a_chars.next();
                } else {
                    break;
                }
            }
            let mut num_b: u64 = 0;
            while let Some(&d) = b_chars.peek() {
                if d.is_ascii_digit() {
                    num_b = num_b.saturating_mul(10).saturating_add((d as u8 - b'0') as u64);
                    b_chars.next();
                } else {
                    break;
                }
            }
            let ord = num_a.cmp(&num_b);
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        } else {
            let ord = ca.to_lowercase().cmp(cb.to_lowercase());
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
            a_chars.next();
            b_chars.next();
        }
    }
    a.len().cmp(&b.len())
}

/// 树节点同级排序函数
fn sort_nodes(
    nodes: &mut [&FileNode],
    sort_mode: FileSortMode,
    state: &AppState,
) {
    nodes.sort_by(|a, b| {
        let a_is_container = a.is_dir || a.is_sarc;
        let b_is_container = b.is_dir || b.is_sarc;

        if a_is_container != b_is_container {
            return b_is_container.cmp(&a_is_container);
        }

        match sort_mode {
            FileSortMode::NameAsc => natural_cmp(&a.name, &b.name),
            FileSortMode::NameDesc => natural_cmp(&b.name, &a.name),
            FileSortMode::FileType => {
                if a_is_container {
                    natural_cmp(&a.name, &b.name)
                } else {
                    let ext_a = a.name.rsplit('.').next().unwrap_or("");
                    let ext_b = b.name.rsplit('.').next().unwrap_or("");
                    ext_a.to_lowercase()
                        .cmp(&ext_b.to_lowercase())
                        .then_with(|| natural_cmp(&a.name, &b.name))
                }
            }
            FileSortMode::ModifiedFirst => {
                let mod_a = state.is_file_modified(&a.path);
                let mod_b = state.is_file_modified(&b.path);
                mod_b.cmp(&mod_a).then_with(|| natural_cmp(&a.name, &b.name))
            }
        }
    });
}

/// 将树结构中当前可见的节点扁平化收集为一维数组 (只遍历展开的目录)
pub fn flatten_visible_nodes(
    nodes: &[FileNode],
    expanded_paths: &std::collections::HashSet<String>,
    is_filtering: bool,
    matching_paths: &std::collections::HashSet<String>,
    selected_path: &str,
    state: &AppState,
    level: usize,
    out: &mut Vec<FlatFileEntry>,
) {
    let mut sorted_nodes: Vec<&FileNode> = nodes.iter().collect();
    sort_nodes(&mut sorted_nodes, state.file_sort_mode, state);

    for node in sorted_nodes {
        let is_container = node.is_dir || node.is_sarc;

        if is_filtering && !matching_paths.contains(&node.path) {
            continue;
        }

        let is_expanded = expanded_paths.contains(&node.path) || is_filtering;
        let is_selected = selected_path == node.path;
        let is_modified = !is_container && state.is_file_modified(&node.path);

        out.push(FlatFileEntry {
            name: node.name.clone(),
            path: node.path.clone(),
            is_dir: node.is_dir,
            is_sarc: node.is_sarc,
            level,
            is_expanded,
            is_selected,
            is_modified,
        });

        if is_container && is_expanded {
            if let Some(children) = &node.children {
                flatten_visible_nodes(
                    children,
                    expanded_paths,
                    is_filtering,
                    matching_paths,
                    selected_path,
                    state,
                    level + 1,
                    out,
                );
            }
        }
    }
}

/// 渲染单行文件条目 (固定高度 26px，满足 uniform_list 约束)
fn render_flat_file_row(
    item: &FlatFileEntry,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let is_container = item.is_dir || item.is_sarc;
    let is_selected = item.is_selected;
    let is_modified = item.is_modified;
    let path = item.path.clone();
    let model_click = model.clone();

    let guide_line_color = if pal.is_dark {
        hsla(215.0 / 360.0, 0.20, 0.40, 0.25)
    } else {
        hsla(215.0 / 360.0, 0.20, 0.60, 0.25)
    };

    div()
        .id(ElementId::Name(item.path.clone().into()))
        .h(px(26.0))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .pl_1()
        .pr_2()
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
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let path_clone = path.clone();
            model_click.update(cx, |st, cx| {
                st.clear_active_text_inputs();
                st.file_tree_sort_menu_open = false;
                if is_container {
                    st.toggle_tree_path(&path_clone);
                } else {
                    st.load_file_by_path(&path_clone);
                }
                cx.notify();
            });
        })
        .child(
            div()
                .flex()
                .items_center()
                .flex_1()
                .overflow_hidden()
                .whitespace_nowrap()
                // 1. 层级缩进竖向引导提示线 (Indent Guides)
                .children((0..item.level).map(|_| {
                    div()
                        .w(px(14.0))
                        .h(px(26.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .child(
                            div()
                                .w(px(1.0))
                                .h(px(26.0))
                                .bg(guide_line_color),
                        )
                }))
                // 2. 展开/折叠三角指示符 (Expand / Collapse Chevron)
                .child(
                    div()
                        .w(px(14.0))
                        .h(px(26.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .text_size(px(10.0))
                        .text_color(pal.text_secondary)
                        .child(if is_container {
                            if item.is_expanded { "⌄" } else { "›" }
                        } else {
                            " "
                        }),
                )
                // 3. 节点类型图标 (Node Type Icon)
                .child(
                    div()
                        .mr_1()
                        .text_size(px(12.0))
                        .flex_shrink_0()
                        .child(get_node_icon_by_name(&item.name, item.is_dir, item.is_sarc)),
                )
                // 4. 节点名称与修改标记 (Node Name & Dirty Indicator)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_weight(if is_selected || is_modified {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_color(if is_modified {
                                    pal.secondary_accent // 琥珀金：已被修改
                                } else if is_selected {
                                    pal.primary_accent // 遗迹青蓝：当前选中
                                } else if is_container {
                                    pal.text_primary // 亮白灰：目录
                                } else {
                                    pal.text_secondary // 蓝灰白：普通文件
                                })
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(item.name.clone()),
                        )
                        .children(is_modified.then(|| {
                            div()
                                .text_size(px(8.0))
                                .text_color(pal.secondary_accent)
                                .child("●")
                        })),
                ),
        )
}

/// 根据文件/封包属性获取对应展示图标
fn get_node_icon_by_name(name: &str, is_dir: bool, is_sarc: bool) -> &'static str {
    if is_sarc {
        "🗃️"
    } else if is_dir {
        "📁"
    } else {
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
        {
            "⚙️"
        } else if lower.ends_with(".json") {
            "{}"
        } else if lower.ends_with(".yaml") || lower.ends_with(".yml") {
            "📜"
        } else if lower.ends_with(".txt") || lower.ends_with(".log") {
            "≡"
        } else {
            "📄"
        }
    }
}
