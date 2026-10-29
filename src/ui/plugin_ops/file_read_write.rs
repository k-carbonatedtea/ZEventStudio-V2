use crate::model::code_document::{CodeEndian, CodeFormat};
use crate::model::file_converter::{compile_code_to_bytes, parse_file_to_code, should_compress_yaz0};
use crate::model::file_tree::read_file_from_path;
use crate::model::sarc_writer::write_file_to_target;
use crate::model::{DocumentTabKind, Graph};
use crate::ui::components::Toast;
use crate::ui::state::AppState;
use std::path::Path;

/// 针对文件与代码读写修改的 API 实现
impl AppState {
    /// 读取物理文件或 SARC 虚拟条目的源代码及格式化表示
    pub fn api_read_file(&self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let raw_path = payload
            .get("path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "缺少 path 参数".to_string())?;

        let resolved_path = self.resolve_target_path(raw_path);

        // 1. 优先从当前打开的标签页缓存中读取最新代码
        if let Some(tab) = self.open_tabs.iter().find(|tab_item| {
            tab_item.file_path == resolved_path || tab_item.file_path == raw_path
        }) {
            match tab.tab_kind {
                DocumentTabKind::CodeEditor => {
                    let content = tab.code_state.as_ref().map(|code| code.content.clone()).unwrap_or_default();
                    return Ok(serde_json::json!({
                        "status": "ok",
                        "path": tab.file_path,
                        "format": tab.file_type,
                        "content": content,
                        "is_open_in_tab": true,
                        "is_modified": tab.is_modified,
                    }));
                }
                DocumentTabKind::Flowchart => {
                    let graph = if self.active_tab_id.as_deref() == Some(&tab.id) {
                        &self.graph
                    } else if let Some(ref cached_graph) = tab.graph {
                        cached_graph
                    } else {
                        &self.graph
                    };
                    let json_content = serde_json::to_string_pretty(graph)
                        .map_err(|error| format!("序列化 BFEVFL 流程图失败: {}", error))?;
                    return Ok(serde_json::json!({
                        "status": "ok",
                        "path": tab.file_path,
                        "format": "bfevfl",
                        "content": json_content,
                        "is_open_in_tab": true,
                        "is_modified": tab.is_modified,
                    }));
                }
                DocumentTabKind::DiffViewer => {}
                DocumentTabKind::Timeline => {
                    let json_content = if let Some(ref tl_state) = tab.timeline_state {
                        serde_json::to_string_pretty(&tl_state.timeline).unwrap_or_default()
                    } else if let Some(ref code) = tab.code_state {
                        code.content.clone()
                    } else {
                        String::new()
                    };
                    return Ok(serde_json::json!({
                        "status": "ok",
                        "path": tab.file_path,
                        "format": "bfevtm",
                        "content": json_content,
                        "is_open_in_tab": true,
                        "is_modified": tab.is_modified,
                    }));
                }
            }
        }

        // 2. 从物理磁盘或 SARC 封包中解包读取
        let bytes = read_file_from_path(&resolved_path)?;
        let format_option = payload.get("format").and_then(|value| value.as_str());

        if format_option == Some("raw") {
            if let Ok(utf8_text) = String::from_utf8(bytes.clone()) {
                return Ok(serde_json::json!({
                    "status": "ok",
                    "path": resolved_path,
                    "format": "text",
                    "content": utf8_text,
                    "is_open_in_tab": false,
                    "is_modified": false,
                    "size_bytes": bytes.len(),
                }));
            }
            return Err("目标文件为纯二进制，无法以文本格式返回，请使用通用编辑格式".to_string());
        }

        let lower_path = resolved_path.to_lowercase();
        if lower_path.ends_with(".bfevfl") || bytes.starts_with(b"BFEVFL") {
            let loaded_graph = crate::model::evfl::load_bfevfl_from_bytes(&bytes)?;
            let json_content = serde_json::to_string_pretty(&loaded_graph)
                .map_err(|error| format!("格式化流程图 JSON 失败: {}", error))?;
            return Ok(serde_json::json!({
                "status": "ok",
                "path": resolved_path,
                "format": "bfevfl",
                "content": json_content,
                "is_open_in_tab": false,
                "is_modified": false,
                "size_bytes": bytes.len(),
            }));
        }

        let (code_text, code_format, _endian) = parse_file_to_code(&bytes, &resolved_path)?;
        Ok(serde_json::json!({
            "status": "ok",
            "path": resolved_path,
            "format": code_format.language_id(),
            "content": code_text,
            "is_open_in_tab": false,
            "is_modified": false,
            "size_bytes": bytes.len(),
        }))
    }

    /// 写入物理文件或 SARC 虚拟条目，并实时同步已打开的编辑器标签
    pub fn api_write_file(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let raw_path = payload
            .get("path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "缺少 path 参数".to_string())?;
        let content = payload
            .get("content")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "缺少 content 代码内容".to_string())?;

        let resolved_path = self.resolve_target_path(raw_path);
        let format_param = payload.get("format").and_then(|value| value.as_str());
        let endian_param = payload.get("endian").and_then(|value| value.as_str());
        let existing_bytes = read_file_from_path(&resolved_path).ok();
        let target_endian = self.resolve_file_endian(&resolved_path, existing_bytes.as_deref(), endian_param);

        let final_bytes = compile_content_to_target_bytes(&resolved_path, content, format_param, target_endian)?;
        write_file_to_target(&resolved_path, &final_bytes)?;

        let tab_synced = self.sync_memory_tab_after_write(&resolved_path, content);
        self.reload_project();
        if self.git_is_repo {
            self.refresh_git_status();
        }

        let file_name = Path::new(&resolved_path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| resolved_path.clone());
        self.toast = Some(Toast::success(format!("MCP: 已写入 「{}」", file_name)));

        Ok(serde_json::json!({
            "status": "ok",
            "path": resolved_path,
            "bytes_written": final_bytes.len(),
            "endian": match target_endian { CodeEndian::Big => "big", CodeEndian::Little => "little" },
            "synced_tab": tab_synced,
        }))
    }

    /// 写入文件后同步内存中可能已打开的标签页
    fn sync_memory_tab_after_write(&mut self, path: &str, content: &str) -> bool {
        let mut tab_found = false;
        let active_id = self.active_tab_id.clone();
        let mut active_new_graph = None;

        for tab in &mut self.open_tabs {
            if tab.file_path == path {
                tab_found = true;
                tab.is_modified = false;
                match tab.tab_kind {
                    DocumentTabKind::CodeEditor => {
                        if let Some(ref mut code) = tab.code_state {
                            code.content = content.to_string();
                            code.mark_saved();
                            code.version += 1;
                        }
                    }
                    DocumentTabKind::Flowchart => {
                        if let Ok(graph) = serde_json::from_str::<Graph>(content) {
                            tab.graph = Some(graph.clone());
                            tab.original_graph = Some(graph.clone());
                            if active_id.as_deref() == Some(&tab.id) {
                                active_new_graph = Some(graph);
                            }
                        }
                    }
                    DocumentTabKind::DiffViewer => {}
                    DocumentTabKind::Timeline => {
                        if let Ok(timeline) = serde_json::from_str::<revfl::timeline::Timeline>(content) {
                            if let Some(ref mut tl_state) = tab.timeline_state {
                                tl_state.timeline = timeline.clone();
                                tl_state.original_timeline = timeline;
                            }
                        }
                        if let Some(ref mut code) = tab.code_state {
                            code.content = content.to_string();
                            code.mark_saved();
                            code.version += 1;
                        }
                    }
                }
            }
        }

        if let Some(graph) = active_new_graph {
            self.graph = graph;
            self.fit_view();
        }

        tab_found
    }
}


/// 辅助函数：将文本内容编译回目标格式的二进制字节流 (支持 Yaz0 自动压缩与端序控制)
fn compile_content_to_target_bytes(
    path: &str,
    content: &str,
    format_override: Option<&str>,
    target_endian: CodeEndian,
) -> Result<Vec<u8>, String> {
    let lower_path = path.to_lowercase();
    let format_str = format_override.unwrap_or("");

    if format_str == "bfevfl" || lower_path.ends_with(".bfevfl") {
        let graph: Graph = serde_json::from_str(content)
            .map_err(|error| format!("解析 BFEVFL 流程图 JSON 语法错误: {}", error))?;
        let raw = crate::model::evfl::save_bfevfl_to_bytes(&graph)?;
        if should_compress_yaz0(path) {
            Ok(roead::yaz0::compress(&raw))
        } else {
            Ok(raw)
        }
    } else {
        let code_format = match format_str {
            "byml" => CodeFormat::Byml,
            "aamp" => CodeFormat::Aamp,
            "msbt" => CodeFormat::Msbt,
            "json" => CodeFormat::Json,
            "yaml" => CodeFormat::Yaml,
            _ => {
                if crate::model::is_byml_extension(&lower_path) {
                    CodeFormat::Byml
                } else if lower_path.ends_with(".aamp") || lower_path.ends_with(".baiprog") || lower_path.ends_with(".bxml") {
                    CodeFormat::Aamp
                } else if lower_path.ends_with(".msbt") || lower_path.ends_with(".smsbt") {
                    CodeFormat::Msbt
                } else if lower_path.ends_with(".json") {
                    CodeFormat::Json
                } else if lower_path.ends_with(".yml") || lower_path.ends_with(".yaml") {
                    CodeFormat::Yaml
                } else {
                    CodeFormat::Text
                }
            }
        };
        // compile_code_to_bytes 内部已经根据 should_compress_yaz0 执行过 Yaz0 压缩
        compile_code_to_bytes(content, code_format, target_endian, path)
    }
}
