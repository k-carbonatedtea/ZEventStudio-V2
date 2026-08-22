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
    let is_container = node.is_dir || node.is_sarc;
    let is_expanded = state.expanded_tree_paths.contains(&node.path) || !filter.is_empty();
    let is_selected = selected_path == node.path;
    let path_clone = node.path.clone();
    let model_click = model.clone();

    let pad_left = (level * 14 + 10) as f32;

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
                .pl(px(pad_left))
                .pr_2()
                .rounded_md()
                .cursor_pointer()
                .bg(if is_selected {
                    hsla(215.0 / 360.0, 0.45, 0.22, 1.0)
                } else {
                    hsla(0.0, 0.0, 0.0, 0.0)
                })
                .hover(|s| {
                    if is_selected {
                        s.bg(hsla(215.0 / 360.0, 0.50, 0.26, 1.0))
                    } else {
                        s.bg(hsla(220.0 / 360.0, 0.25, 0.16, 1.0))
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
                        .gap_1p5()
                        .overflow_hidden()
                        // 展开/折叠三角指示符
                        .child(
                            div()
                                .w(px(12.0))
                                .text_size(px(10.0))
                                .text_color(hsla(215.0 / 360.0, 0.20, 0.60, 1.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(if is_container {
                                    if is_expanded { "⌄" } else { "›" }
                                } else {
                                    " "
                                }),
                        )
                        // 节点类型图标
                        .child(
                            div()
                                .text_size(px(12.0))
                                .child(get_node_icon(node)),
                        )
                        // 节点名称
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_weight(if is_selected { FontWeight::BOLD } else { FontWeight::NORMAL })
                                .text_color(if is_selected {
                                    hsla(0.0, 0.0, 1.0, 1.0)
                                } else if is_container {
                                    hsla(215.0 / 360.0, 0.20, 0.90, 1.0)
                                } else {
                                    hsla(215.0 / 360.0, 0.20, 0.75, 1.0)
                                })
                                .overflow_hidden()
                                .child(node.name.clone()),
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
        if lower.ends_with(".bfevfl") {
            "📄"
        } else if lower.ends_with(".json") {
            "📜"
        } else if lower.ends_with(".msbt") {
            "💬"
        } else if lower.ends_with(".byml") || lower.ends_with(".aamp") {
            "⚙️"
        } else {
            "📎"
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
