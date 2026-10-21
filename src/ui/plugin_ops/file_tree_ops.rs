use crate::model::file_converter::parse_file_to_code;
use crate::model::file_tree::{is_sarc_extension, read_file_from_path, scan_project_path, FileNode};
use crate::model::sarc_writer::{delete_file_from_target, rename_file_in_target};
use crate::ui::components::Toast;
use crate::ui::state::AppState;
use std::path::Path;

/// 针对工程文件树与文件元数据的操作扩展
impl AppState {
    /// 列出工程项目中的全部或筛选文件与 SARC 虚拟节点
    pub fn api_list_files(&self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let recursive = payload.get("recursive").and_then(|value| value.as_bool()).unwrap_or(true);
        let include_sarc = payload.get("include_sarc").and_then(|value| value.as_bool()).unwrap_or(true);
        let filter_ext = payload.get("filter_ext").and_then(|value| value.as_str());
        let limit = payload.get("limit").and_then(|value| value.as_u64()).unwrap_or(2000) as usize;

        let mut output_files = Vec::new();

        if let Some(custom_path) = payload.get("path").and_then(|value| value.as_str()) {
            if let Ok((_name, nodes)) = scan_project_path(custom_path) {
                collect_nodes_flat(&nodes, recursive, include_sarc, filter_ext, limit, &mut output_files);
                return Ok(serde_json::json!({
                    "status": "ok",
                    "path": custom_path,
                    "total_count": output_files.len(),
                    "files": output_files,
                }));
            }
        }

        collect_nodes_flat(&self.file_tree, recursive, include_sarc, filter_ext, limit, &mut output_files);

        Ok(serde_json::json!({
            "status": "ok",
            "project_name": self.project_name,
            "project_path": self.project_path,
            "total_count": output_files.len(),
            "files": output_files,
        }))
    }

    /// 删除指定物理文件/目录或 SARC 封包内虚拟子文件
    pub fn api_delete_file(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let raw_path = payload
            .get("path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "缺少 path 参数".to_string())?;

        let resolved_path = self.resolve_target_path(raw_path);
        delete_file_from_target(&resolved_path)?;

        let tabs_to_close: Vec<String> = self
            .open_tabs
            .iter()
            .filter(|tab_item| tab_item.file_path == resolved_path || tab_item.file_path.starts_with(&format!("{}/", resolved_path)))
            .map(|tab_item| tab_item.id.clone())
            .collect();

        for tab_id in tabs_to_close {
            self.close_document_tab(&tab_id);
        }

        if self.selected_file_path.as_deref() == Some(&resolved_path) {
            self.selected_file_path = None;
        }

        self.reload_project();
        let file_name = Path::new(&resolved_path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| resolved_path.clone());
        self.toast = Some(Toast::success(format!("MCP: 已删除 「{}」", file_name)));

        Ok(serde_json::json!({ "status": "ok", "deleted": resolved_path }))
    }

    /// 重命名物理文件/目录或 SARC 封包内虚拟文件
    pub fn api_rename_file(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let old_raw_path = payload
            .get("old_path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "缺少 old_path 参数".to_string())?;
        let new_name = payload
            .get("new_name")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "缺少 new_name 参数".to_string())?;

        let old_resolved_path = self.resolve_target_path(old_raw_path);
        let new_resolved_path = rename_file_in_target(&old_resolved_path, new_name)?;

        for tab in &mut self.open_tabs {
            if tab.file_path == old_resolved_path {
                tab.file_path = new_resolved_path.clone();
                tab.title = new_name.to_string();
            }
        }
        if self.selected_file_path.as_deref() == Some(&old_resolved_path) {
            self.selected_file_path = Some(new_resolved_path.clone());
        }

        self.reload_project();
        self.toast = Some(Toast::success(format!("MCP: 重命名 → {}", new_name)));

        Ok(serde_json::json!({
            "status": "ok",
            "old_path": old_resolved_path,
            "new_path": new_resolved_path,
        }))
    }

    /// 创建物理子目录
    pub fn api_create_directory(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let raw_path = payload
            .get("path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "缺少 path 参数".to_string())?;

        let resolved_path = self.resolve_target_path(raw_path);
        std::fs::create_dir_all(&resolved_path)
            .map_err(|error| format!("创建目录失败: {}", error))?;

        self.reload_project();
        Ok(serde_json::json!({ "status": "ok", "path": resolved_path }))
    }

    /// 在项目文件与 SARC 封包代码中进行全文检索
    pub fn api_search_code(&self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let query = payload
            .get("query")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "缺少 query 检索关键词".to_string())?;
        let case_sensitive = payload.get("case_sensitive").and_then(|value| value.as_bool()).unwrap_or(false);
        let file_pattern = payload.get("file_pattern").and_then(|value| value.as_str());
        let max_results = payload.get("max_results").and_then(|value| value.as_u64()).unwrap_or(50) as usize;

        let mut matched_results = Vec::new();
        let mut all_files = Vec::new();
        collect_nodes_flat(&self.file_tree, true, true, None, 5000, &mut all_files);

        let query_normalized = if case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        for item in all_files {
            if matched_results.len() >= max_results {
                break;
            }
            let is_dir = item.get("is_dir").and_then(|value| value.as_bool()).unwrap_or(false);
            if is_dir {
                continue;
            }
            let path_str = match item.get("path").and_then(|value| value.as_str()) {
                Some(path) => path,
                None => continue,
            };

            if let Some(pattern) = file_pattern {
                if !path_str.to_lowercase().contains(&pattern.to_lowercase()) {
                    continue;
                }
            }

            if let Ok(bytes) = read_file_from_path(path_str) {
                if let Ok((code_text, _, _)) = parse_file_to_code(&bytes, path_str) {
                    for (line_index, line) in code_text.lines().enumerate() {
                        let line_match = if case_sensitive {
                            line.contains(&query_normalized)
                        } else {
                            line.to_lowercase().contains(&query_normalized)
                        };
                        if line_match {
                            matched_results.push(serde_json::json!({
                                "file": path_str,
                                "line": line_index + 1,
                                "snippet": line.trim(),
                            }));
                            if matched_results.len() >= max_results {
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "status": "ok",
            "query": query,
            "total_matches": matched_results.len(),
            "matches": matched_results,
        }))
    }

    /// 解析路径，支持相对于已打开工程目录的路径及文件名模糊定位
    pub fn resolve_target_path(&self, raw_path: &str) -> String {
        if raw_path.starts_with("SARC:") {
            return raw_path.to_string();
        }
        let path_obj = Path::new(raw_path);
        if path_obj.is_absolute() && path_obj.exists() {
            return raw_path.to_string();
        }

        if let Some(matched) = self.find_path_in_tree(&self.file_tree, raw_path) {
            return matched;
        }

        if let Some(root) = &self.project_path {
            let clean_sub = raw_path.trim_start_matches(['/', '\\']);
            return Path::new(root).join(clean_sub).to_string_lossy().to_string();
        }

        raw_path.to_string()
    }

    /// 在树形节点中递归匹配目标文件名或相对后缀
    fn find_path_in_tree(&self, nodes: &[FileNode], target: &str) -> Option<String> {
        let target_norm = target.replace('\\', "/").to_lowercase();
        for node in nodes {
            let node_norm = node.path.replace('\\', "/").to_lowercase();
            if node_norm == target_norm
                || node.name.eq_ignore_ascii_case(target)
                || node_norm.ends_with(&format!("/{}", target_norm))
                || node_norm.ends_with(&format!("//{}", target_norm))
            {
                return Some(node.path.clone());
            }
            if let Some(ref children) = node.children {
                if let Some(found) = self.find_path_in_tree(children, target) {
                    return Some(found);
                }
            }
        }
        None
    }
}

/// 递归扁平化遍历节点树提取文件元数据条目
pub fn collect_nodes_flat(
    nodes: &[FileNode],
    recursive: bool,
    include_sarc: bool,
    filter_ext: Option<&str>,
    limit: usize,
    out: &mut Vec<serde_json::Value>,
) {
    for node in nodes {
        if out.len() >= limit {
            return;
        }
        let is_inside_sarc = node.path.starts_with("SARC:") && node.path.contains("//");
        let format_name = if node.is_dir {
            "directory"
        } else if is_sarc_extension(&node.name) {
            "sarc"
        } else {
            infer_format_from_path(&node.path)
        };

        let matches_filter = match filter_ext {
            Some(ext) => node.name.to_lowercase().ends_with(&ext.to_lowercase()),
            None => true,
        };

        if matches_filter && (!node.is_dir || filter_ext.is_none()) {
            out.push(serde_json::json!({
                "name": node.name,
                "path": node.path,
                "is_dir": node.is_dir,
                "is_sarc": node.is_sarc,
                "is_inside_sarc": is_inside_sarc,
                "format": format_name,
            }));
        }

        if recursive {
            if let Some(ref children) = node.children {
                if !node.is_sarc || include_sarc {
                    collect_nodes_flat(children, recursive, include_sarc, filter_ext, limit, out);
                }
            }
        }
    }
}

/// 根据文件路径推断其格式名称
pub fn infer_format_from_path(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.ends_with(".bfevfl") {
        "bfevfl"
    } else if lower.ends_with(".byml") || lower.ends_with(".sbyml") || lower.ends_with(".sbgyml") {
        "byml"
    } else if lower.ends_with(".msbt") || lower.ends_with(".smsbt") {
        "msbt"
    } else if lower.ends_with(".aamp") || lower.ends_with(".baiprog") || lower.ends_with(".sbaiprog") || lower.ends_with(".bxml") || lower.ends_with(".bactparam") {
        "aamp"
    } else if lower.ends_with(".json") {
        "json"
    } else if lower.ends_with(".yml") || lower.ends_with(".yaml") {
        "yaml"
    } else if lower.ends_with(".txt") {
        "text"
    } else {
        "binary"
    }
}
