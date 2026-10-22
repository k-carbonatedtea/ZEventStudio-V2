pub mod parser;
pub mod loader;

use std::path::PathBuf;
pub use loader::{AiCacheManager, AiDataStatus};

/// 核心功能：生成搜寻 AI 定义文件的有效目录列表
pub fn build_search_dirs(
    explicit_dirs: &[PathBuf],
    custom_dump_dir: Option<&str>,
) -> Vec<PathBuf> {
    loader::get_search_directories(explicit_dirs, custom_dump_dir)
}

/// 核心功能：为指定角色的 Action 或 Query 提取参数默认值
pub fn get_autofill_parameters(
    actor_name: &str,
    action_query_name: &str,
    is_query: bool,
    search_dirs: &[PathBuf],
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    // 1. 获取角色的 AI Program
    let prog = loader::load_actor_aiprog(actor_name, search_dirs)
        .map_err(|_| format!("未能加载角色 {} 的 AI 程序", actor_name))?;

    // 2. 找到 Action/Query 对应的 ClassName
    let class_name = if is_query {
        prog.queries.get(action_query_name)
    } else {
        prog.actions.get(action_query_name)
    }.ok_or_else(|| format!("所选动作/查询 {} 未在角色 {} 的 AI 程序中注册", action_query_name, actor_name))?;

    // 3. 获取全局 AI 定义字典
    let defs = loader::load_global_ai_defs(search_dirs)
        .map_err(|_| "未能加载游戏全局 AI 定义 (AIDef_Game.product.sbyml)".to_string())?;

    // 4. 根据 ClassName 找到参数定义表
    let class_def = if is_query {
        defs.queries.get(class_name)
    } else {
        defs.actions.get(class_name)
    }.ok_or_else(|| format!("未在全局 AI 定义中找到类 {} 的定义", class_name))?;

    let mut result_params = serde_json::Map::new();
    
    // Action 默认都有 IsWaitFinish 参数
    if !is_query {
        result_params.insert("IsWaitFinish".to_string(), serde_json::json!({ "Bool": false }));
    }

    // 遍历定义，提取默认值
    for p in &class_def.params {
        let param_type = if p.param_type.is_empty() { "String" } else { &p.param_type };
        result_params.insert(p.name.clone(), serde_json::json!({ param_type: p.default_value.clone() }));
    }

    Ok(result_params)
}

/// 核心功能：按照官方 AI 规范顺序重新排布参数项 (IsWaitFinish 居首，其余按官方定义先后排序)
pub fn reorder_parameter_items(
    actor_name: &str,
    action_query_name: &str,
    is_query: bool,
    search_dirs: &[PathBuf],
    current_properties: &[crate::model::PropertyItem],
) -> Vec<crate::model::PropertyItem> {
    let mut ordered_keys: Vec<String> = Vec::new();
    if !is_query {
        ordered_keys.push("IsWaitFinish".to_string());
    }

    if let Ok(prog) = loader::load_actor_aiprog(actor_name, search_dirs) {
        let class_name = if is_query {
            prog.queries.get(action_query_name)
        } else {
            prog.actions.get(action_query_name)
        };
        if let Some(cname) = class_name {
            if let Ok(defs) = loader::load_global_ai_defs(search_dirs) {
                let class_def_opt = if is_query {
                    defs.queries.get(cname)
                } else {
                    defs.actions.get(cname)
                };
                if let Some(cdef) = class_def_opt {
                    for p in &cdef.params {
                        if !ordered_keys.contains(&p.name) {
                            ordered_keys.push(p.name.clone());
                        }
                    }
                }
            }
        }
    }

    let mut result = Vec::with_capacity(current_properties.len());
    // 1. 先按规范顺序加入已存在的参数
    for key in &ordered_keys {
        if let Some(item) = current_properties.iter().find(|p| &p.key == key) {
            result.push(item.clone());
        }
    }
    // 2. 将用户自定义的未知参数附加在尾部
    for item in current_properties {
        if !ordered_keys.contains(&item.key) && !result.iter().any(|r| r.key == item.key) {
            result.push(item.clone());
        }
    }

    result
}

/// 获取角色全部可用的 Action 列表 (来自游戏 AI 程序)
pub fn get_actor_available_actions(actor_name: &str, search_dirs: &[PathBuf]) -> Vec<String> {
    if let Ok(prog) = loader::load_actor_aiprog(actor_name, search_dirs) {
        let mut list: Vec<String> = prog.actions.into_keys().collect();
        list.sort();
        list
    } else {
        Vec::new()
    }
}

/// 获取角色全部可用的 Query 列表 (来自游戏 AI 程序)
pub fn get_actor_available_queries(actor_name: &str, search_dirs: &[PathBuf]) -> Vec<String> {
    if let Ok(prog) = loader::load_actor_aiprog(actor_name, search_dirs) {
        let mut list: Vec<String> = prog.queries.into_keys().collect();
        list.sort();
        list
    } else {
        Vec::new()
    }
}

/// 查询当前环境中的游戏 AI 加载状态
pub fn get_ai_data_status(search_dirs: &[PathBuf]) -> AiDataStatus {
    if let Ok(defs) = loader::load_global_ai_defs(search_dirs) {
        let actors_count = AiCacheManager::global().actor_progs.read().map(|g| g.len()).unwrap_or(0);
        let src = AiCacheManager::global().source_path.read().map(|s| s.clone()).unwrap_or_default();
        AiDataStatus {
            is_loaded: true,
            actions_count: defs.actions.len(),
            queries_count: defs.queries.len(),
            actors_count,
            source_path: src,
        }
    } else {
        AiDataStatus::default()
    }
}
