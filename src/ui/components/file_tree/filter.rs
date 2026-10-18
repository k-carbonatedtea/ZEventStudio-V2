use crate::model::FileNode;
use crate::ui::state::{AppState, FileFilterType, FileSortMode};
use super::types::FlatFileEntry;
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
pub(crate) fn collect_matching_paths(
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
#[allow(clippy::too_many_arguments)]
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
