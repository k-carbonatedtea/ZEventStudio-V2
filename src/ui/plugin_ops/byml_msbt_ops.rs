use crate::model::code_document::CodeEndian;
use crate::model::file_converter::{decompress_if_yaz0, should_compress_yaz0};
use crate::model::file_tree::read_file_from_path;
use crate::model::sarc_writer::write_file_to_target;
use crate::ui::components::Toast;
use crate::ui::state::AppState;
use roead::byml::Byml;
use roead::Endian;
use std::path::Path;

/// 针对任天堂 BYML 结构化数据与 MSBT 文本数据的细粒度操作 API 实现
impl AppState {
    /// 查询 BYML 文件内容或基于 JSON 指针检索特定子树 (避免传输庞大 YAML)
    pub fn api_query_byml(&self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let raw_path = payload
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 path 参数".to_string())?;
        let resolved_path = self.resolve_target_path(raw_path);

        let bytes = read_file_from_path(&resolved_path)?;
        let decomp = decompress_if_yaz0(&bytes);

        let byml = Byml::from_binary(&decomp)
            .map_err(|e| format!("解析 BYML 二进制失败: {:?}", e))?;
        let json_val = crate::model::byml_to_clean_json(&byml);

        let query_path = payload.get("query_path").and_then(|v| v.as_str()).unwrap_or("");
        let target_val = if query_path.is_empty() || query_path == "/" {
            &json_val
        } else {
            json_val.pointer(query_path)
                .ok_or_else(|| format!("未在 BYML 中找到指定路径: {}", query_path))?
        };

        let summary_only = payload.get("summary_only").and_then(|v| v.as_bool()).unwrap_or(false);
        let final_val = if summary_only {
            match target_val {
                serde_json::Value::Array(arr) => serde_json::json!({
                    "kind": "Array",
                    "length": arr.len(),
                }),
                serde_json::Value::Object(obj) => serde_json::json!({
                    "kind": "Map",
                    "keys": obj.keys().collect::<Vec<_>>(),
                }),
                _ => target_val.clone(),
            }
        } else {
            target_val.clone()
        };

        Ok(serde_json::json!({
            "status": "ok",
            "path": resolved_path,
            "query_path": query_path,
            "result": final_val,
        }))
    }

    /// 细粒度修改或追加 BYML 文件中的节点 (支持 set, append, delete 操作)
    pub fn api_patch_byml(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let raw_path = payload
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 path 参数".to_string())?;
        let resolved_path = self.resolve_target_path(raw_path);

        let patch_op = payload
            .get("patch_op")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 patch_op 操作类型 (set, append, delete)".to_string())?;
        let query_path = payload.get("query_path").and_then(|v| v.as_str()).unwrap_or("");

        let existing_bytes = read_file_from_path(&resolved_path).ok();
        let endian_param = payload.get("endian").and_then(|v| v.as_str());
        let target_endian = match self.resolve_file_endian(&resolved_path, existing_bytes.as_deref(), endian_param) {
            CodeEndian::Big => Endian::Big,
            CodeEndian::Little => Endian::Little,
        };

        let mut root_val = if let Some(bytes) = existing_bytes {
            let decomp = decompress_if_yaz0(&bytes);
            let byml = Byml::from_binary(&decomp)
                .map_err(|e| format!("解析目标 BYML 失败: {:?}", e))?;
            crate::model::byml_to_clean_json(&byml)
        } else {
            serde_json::json!({})
        };

        match patch_op {
            "append" => {
                let value = payload.get("value").ok_or_else(|| "append 操作缺少 value 参数".to_string())?;
                let target = if query_path.is_empty() || query_path == "/" {
                    &mut root_val
                } else {
                    root_val.pointer_mut(query_path)
                        .ok_or_else(|| format!("未找到待追加数组节点: {}", query_path))?
                };
                let arr = target.as_array_mut()
                    .ok_or_else(|| format!("目标路径 {} 不是一个有效数组", query_path))?;
                arr.push(value.clone());
            }
            "set" => {
                let value = payload.get("value").ok_or_else(|| "set 操作缺少 value 参数".to_string())?;
                if query_path.is_empty() || query_path == "/" {
                    root_val = value.clone();
                } else {
                    apply_json_pointer_set(&mut root_val, query_path, value.clone())?;
                }
            }
            "delete" => {
                apply_json_pointer_delete(&mut root_val, query_path)?;
            }
            _ => return Err(format!("不支持的 patch_op: {}", patch_op)),
        }

        let updated_byml = crate::model::clean_json_to_byml(&root_val);
        let bin = updated_byml.to_binary(target_endian);
        let final_bytes = if should_compress_yaz0(&resolved_path) {
            roead::yaz0::compress(&bin)
        } else {
            bin
        };

        write_file_to_target(&resolved_path, &final_bytes)?;
        self.reload_project();
        if self.git_is_repo {
            self.refresh_git_status();
        }

        let file_name = Path::new(&resolved_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| resolved_path.clone());
        self.toast = Some(Toast::success(format!("MCP: BYML 已更新 「{}」", file_name)));

        Ok(serde_json::json!({
            "status": "ok",
            "path": resolved_path,
            "patch_op": patch_op,
            "bytes_written": final_bytes.len(),
            "endian": if target_endian == Endian::Big { "big" } else { "little" },
        }))
    }

    /// 读取 MSBT 对话或本地化文本文件的词条映射字典
    pub fn api_read_msbt(&self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let raw_path = payload
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 path 参数".to_string())?;
        let resolved_path = self.resolve_target_path(raw_path);

        let bytes = read_file_from_path(&resolved_path)?;
        let decomp = decompress_if_yaz0(&bytes);

        let msyt_doc = msyt::Msyt::from_msbt_bytes(&decomp)
            .map_err(|e| format!("解析 MSBT 失败: {:?}", e))?;
        let msyt_json = serde_json::to_value(&msyt_doc)
            .map_err(|e| format!("序列化 MSYT 失败: {}", e))?;

        let mut entries_map = serde_json::Map::new();
        if let Some(entries_obj) = msyt_json.get("entries").and_then(|v| v.as_object()) {
            for (label, entry_val) in entries_obj {
                let mut text = String::new();
                if let Some(contents) = entry_val.get("contents").and_then(|v| v.as_array()) {
                    for item in contents {
                        if let Some(s) = item.as_str() {
                            text.push_str(s);
                        }
                    }
                }
                entries_map.insert(label.clone(), serde_json::Value::String(text));
            }
        }

        Ok(serde_json::json!({
            "status": "ok",
            "path": resolved_path,
            "total_entries": entries_map.len(),
            "entries": entries_map,
        }))
    }

    /// 写入或批量更新 MSBT 对话词条 (自动创建或追加，并写回磁盘或 SARC)
    pub fn api_write_msbt(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let raw_path = payload
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 path 参数".to_string())?;
        let resolved_path = self.resolve_target_path(raw_path);

        let new_entries = payload
            .get("entries")
            .and_then(|v| v.as_object())
            .ok_or_else(|| "缺少 entries 词条字典对象".to_string())?;

        let existing_bytes = read_file_from_path(&resolved_path).ok();

        let mut msyt_json = if let Some(ref bytes) = existing_bytes {
            let decomp = decompress_if_yaz0(bytes);
            if let Ok(doc) = msyt::Msyt::from_msbt_bytes(&decomp) {
                serde_json::to_value(&doc).unwrap_or_else(|_| create_empty_msyt_json())
            } else {
                create_empty_msyt_json()
            }
        } else {
            create_empty_msyt_json()
        };

        if !msyt_json.get("entries").map(|v| v.is_object()).unwrap_or(false) {
            msyt_json["entries"] = serde_json::json!({});
        }

        if let Some(entries_obj) = msyt_json.get_mut("entries").and_then(|v| v.as_object_mut()) {
            for (label, val) in new_entries {
                let text_val = if let Some(s) = val.as_str() {
                    s.to_string()
                } else {
                    val.to_string()
                };
                entries_obj.insert(label.clone(), serde_json::json!({
                    "contents": [text_val]
                }));
            }
        }

        let entries_count = msyt_json
            .get("entries")
            .and_then(|v| v.as_object())
            .map(|o| o.len())
            .unwrap_or(1);
        let current_gc = msyt_json.get("group_count").and_then(|v| v.as_u64()).unwrap_or(0);
        if current_gc == 0 {
            msyt_json["group_count"] = serde_json::json!((entries_count as u32).max(1));
        }

        let msyt_doc: msyt::Msyt = serde_json::from_value(msyt_json)
            .map_err(|e| format!("生成 MSYT 结构失败: {}", e))?;

        let endian_param = payload.get("endian").and_then(|v| v.as_str());
        let msyt_endian = match self.resolve_file_endian(&resolved_path, existing_bytes.as_deref(), endian_param) {
            CodeEndian::Big => msyt::Endianness::Big,
            CodeEndian::Little => msyt::Endianness::Little,
        };

        let msbt_bytes = msyt_doc
            .into_msbt_bytes(msyt_endian)
            .map_err(|e| format!("编译 MSBT 字节流失败: {:?}", e))?;

        let final_bytes = if should_compress_yaz0(&resolved_path) {
            roead::yaz0::compress(&msbt_bytes)
        } else {
            msbt_bytes
        };

        write_file_to_target(&resolved_path, &final_bytes)?;
        self.reload_project();
        if self.git_is_repo {
            self.refresh_git_status();
        }

        let file_name = Path::new(&resolved_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| resolved_path.clone());
        self.toast = Some(Toast::success(format!("MCP: 已写入 MSBT 「{}」", file_name)));

        Ok(serde_json::json!({
            "status": "ok",
            "path": resolved_path,
            "entries_updated": new_entries.len(),
            "bytes_written": final_bytes.len(),
        }))
    }
}

/// 构造初始的空 MSYT JSON 结构
fn create_empty_msyt_json() -> serde_json::Value {
    serde_json::json!({
        "group_count": 1,
        "entries": {}
    })
}

/// 辅助函数：根据 JSON 指针进行修改赋值
fn apply_json_pointer_set(
    root: &mut serde_json::Value,
    pointer: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    let clean = pointer.trim_start_matches('/');
    let mut parts: Vec<&str> = clean.split('/').collect();
    if parts.is_empty() {
        *root = value;
        return Ok(());
    }

    let last_key = parts.pop().unwrap_or("");
    let parent_pointer = if parts.is_empty() {
        "".to_string()
    } else {
        format!("/{}", parts.join("/"))
    };

    let parent = if parent_pointer.is_empty() {
        root
    } else {
        root.pointer_mut(&parent_pointer)
            .ok_or_else(|| format!("未找到父节点: {}", parent_pointer))?
    };

    if let Some(obj) = parent.as_object_mut() {
        obj.insert(last_key.to_string(), value);
        Ok(())
    } else if let Some(arr) = parent.as_array_mut() {
        let idx = last_key.parse::<usize>().map_err(|_| format!("无效的数组索引: {}", last_key))?;
        if idx < arr.len() {
            arr[idx] = value;
            Ok(())
        } else if idx == arr.len() {
            arr.push(value);
            Ok(())
        } else {
            Err(format!("数组索引越界: {} (长度: {})", idx, arr.len()))
        }
    } else {
        Err(format!("目标父节点既非对象也非数组: {}", parent_pointer))
    }
}

/// 辅助函数：根据 JSON 指针删除对应键或数组元素
fn apply_json_pointer_delete(
    root: &mut serde_json::Value,
    pointer: &str,
) -> Result<(), String> {
    let clean = pointer.trim_start_matches('/');
    let mut parts: Vec<&str> = clean.split('/').collect();
    if parts.is_empty() {
        return Err("无法删除根节点".to_string());
    }

    let last_key = parts.pop().unwrap_or("");
    let parent_pointer = if parts.is_empty() {
        "".to_string()
    } else {
        format!("/{}", parts.join("/"))
    };

    let parent = if parent_pointer.is_empty() {
        root
    } else {
        root.pointer_mut(&parent_pointer)
            .ok_or_else(|| format!("未找到父节点: {}", parent_pointer))?
    };

    if let Some(obj) = parent.as_object_mut() {
        obj.remove(last_key).ok_or_else(|| format!("未在对象中找到待删除键: {}", last_key))?;
        Ok(())
    } else if let Some(arr) = parent.as_array_mut() {
        let idx = last_key.parse::<usize>().map_err(|_| format!("无效的数组索引: {}", last_key))?;
        if idx < arr.len() {
            arr.remove(idx);
            Ok(())
        } else {
            Err(format!("数组删除索引越界: {} (长度: {})", idx, arr.len()))
        }
    } else {
        Err(format!("目标父节点既非对象也非数组: {}", parent_pointer))
    }
}
