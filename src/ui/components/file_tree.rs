use gpui::*;
use crate::model::FileNode;
use crate::ui::state::AppState;

/// 渲染文件资源管理器树状组件
pub fn render_file_tree(
    nodes: &[FileNode],
    state: &AppState,
    model: &Entity<AppState>,
    level: usize,
) -> impl IntoElement {
    let filter = state.file_search_query.trim().to_lowercase();
    let selected_path = state.selected_file_path.as_deref().unwrap_or("");

    div()
        .flex()
        .flex_col()
        .w_full()
        .children(nodes.iter().filter_map(|node| {
            if !filter.is_empty() && !node_matches_filter(node, &filter) {
                return None;
            }

            Some(render_file_tree_node(
                node,
                state,
                model,
                level,
                selected_path,
                &filter,
            ))
        }))
}

/// 渲染单个文件或目录节点
fn render_file_tree_node(
    node: &FileNode,
    state: &AppState,
    model: &Entity<AppState>,
    level: usize,
    selected_path: &str,
    filter: &str,
) -> impl IntoElement {
    let pal = state.theme();
    let is_container = node.is_dir || node.is_sarc;
    let is_expanded = state.expanded_tree_paths.contains(&node.path) || !filter.is_empty();
    let is_selected = selected_path == node.path;
    let is_modified = !is_container && state.is_file_modified(&node.path);
    let path_clone = node.path.clone();
    let model_click = model.clone();

    let guide_line_color = if pal.is_dark {
        hsla(215.0 / 360.0, 0.20, 0.40, 0.40)
    } else {
        hsla(215.0 / 360.0, 0.20, 0.60, 0.35)
    };

    div()
        .flex()
        .flex_col()
        .w_full()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .h(px(26.0))
                .pl_1()
                .pr_2()
                .rounded_md()
                .cursor_pointer()
                .bg(if is_selected {
                    pal.card_surface // 浅层蓝灰 Card / Hover
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
                    let path = path_clone.clone();
                    model_click.update(cx, |st, cx| {
                        if is_container {
                            st.toggle_tree_path(&path);
                        } else {
                            st.load_file_by_path(&path);
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
                        .children((0..level).map(|_| {
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
                                        .bg(rgba(0x00000000))
                                        .group_hover("file-tree-container", move |s| {
                                            s.bg(guide_line_color)
                                        }),
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
                                    if is_expanded { "⌄" } else { "›" }
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
                                .child(get_node_icon(node)),
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
                                        .child(node.name.clone()),
                                )
                                .children(is_modified.then(|| {
                                    div()
                                        .text_size(px(8.0))
                                        .text_color(pal.secondary_accent)
                                        .child("●")
                                })),
                        ),
                ),
        )


        // 渲染子节点列表
        .children((is_container && is_expanded).then(|| {
            node.children.as_ref().map(|children| render_file_tree(children, state, model, level + 1))
        }).flatten())
}


/// 根据文件/封包属性获取对应展示图标
fn get_node_icon(node: &FileNode) -> &'static str {
    if node.is_sarc {
        "🗃️"
    } else if node.is_dir {
        "📁"
    } else {
        let lower = node.name.to_lowercase();
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

/// 递归检查节点自身或子节点是否匹配过滤搜索
fn node_matches_filter(node: &FileNode, filter: &str) -> bool {
    if node.name.to_lowercase().contains(filter) {
        return true;
    }
    if let Some(children) = &node.children {
        children.iter().any(|child| node_matches_filter(child, filter))
    } else {
        false
    }
}
