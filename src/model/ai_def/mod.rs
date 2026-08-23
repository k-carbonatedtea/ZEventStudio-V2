pub mod parser;
pub mod loader;

use std::path::PathBuf;
pub use parser::{AiDefs, ActorAiProg, AiClassDef, AiParamDef};
pub use loader::{AiCacheManager, load_global_ai_defs, load_actor_aiprog};

/// 生成搜寻 AI 定义文件的有效目录列表
pub fn build_search_dirs(
    game_dir: Option<&str>,
    update_dir: Option<&str>,
    dlc_dir: Option<&str>,
    mod_dir: Option<&str>,
    custom_dump_dir: Option<&str>,
) -> Vec<PathBuf> {
    loader::get_search_directories(game_dir, update_dir, dlc_dir, mod_dir, custom_dump_dir)
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
