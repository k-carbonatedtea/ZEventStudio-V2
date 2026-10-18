use crate::model::FileNode;

pub const MAX_SEARCH_RESULTS: usize = 500;
#[derive(Clone, Debug)]
pub struct SearchResultItem {
    pub name: String,
    pub path: String,
    pub dir_path: String,
    pub icon: &'static str,
    pub is_sarc: bool,
}

/// 递归遍历项目文件树并提取匹配项 (毫秒级极速检索与精准智能排序)
#[allow(clippy::too_many_arguments)]
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

#[allow(clippy::too_many_arguments)]
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