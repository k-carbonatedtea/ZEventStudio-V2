use gpui::*;
use crate::model::FileNode;
use crate::ui::state::AppState;
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

/// 渲染文件资源管理器树状组件 (采用扁平虚拟化渲染，解决海量文件栈溢出与卡顿)
pub fn render_file_tree(
    nodes: &[FileNode],
    state: &AppState,
    model: &Entity<AppState>,
    _level: usize,
) -> impl IntoElement {
    let filter = state.file_search_query.trim().to_lowercase();
    let selected_path = state.selected_file_path.as_deref().unwrap_or("");
    let pal = state.theme();

    let mut matching_paths = std::collections::HashSet::new();
    if !filter.is_empty() {
        collect_matching_paths(nodes, &filter, &mut matching_paths);
    }

    let mut flat_entries = Vec::new();
    flatten_visible_nodes(
        nodes,
        &state.expanded_tree_paths,
        &filter,
        &matching_paths,
        selected_path,
        state,
        0,
        &mut flat_entries,
    );

    let count = flat_entries.len();
    if count == 0 {
        return div()
            .p_4()
            .text_size(px(12.0))
            .text_color(pal.text_secondary)
            .child(if filter.is_empty() {
                state.t("welcome.no_folder_hint")
            } else {
                "未找到匹配的文件"
            })
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

/// 收集所有自身或后代匹配过滤字符的节点路径 (单次 O(N) 遍历)
fn collect_matching_paths(
    nodes: &[FileNode],
    filter: &str,
    matching_paths: &mut std::collections::HashSet<String>,
) -> bool {
    let mut any_match = false;
    for node in nodes {
        let self_matches = node.name.to_lowercase().contains(filter);
        let children_match = if let Some(children) = &node.children {
            collect_matching_paths(children, filter, matching_paths)
        } else {
            false
        };

        if self_matches || children_match {
            matching_paths.insert(node.path.clone());
            any_match = true;
        }
    }
    any_match
}

/// 将树结构中当前可见的节点扁平化收集为一维数组 (只遍历展开的目录)
fn flatten_visible_nodes(
    nodes: &[FileNode],
    expanded_paths: &std::collections::HashSet<String>,
    filter: &str,
    matching_paths: &std::collections::HashSet<String>,
    selected_path: &str,
    state: &AppState,
    level: usize,
    out: &mut Vec<FlatFileEntry>,
) {
    for node in nodes {
        let is_container = node.is_dir || node.is_sarc;

        if !filter.is_empty() && !matching_paths.contains(&node.path) {
            continue;
        }

        let is_expanded = expanded_paths.contains(&node.path) || !filter.is_empty();
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
                    filter,
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
