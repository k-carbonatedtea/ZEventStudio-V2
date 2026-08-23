use std::path::PathBuf;

/// BCML 路径导入结果
#[derive(Debug, Clone, Default)]
pub struct BcmlImportResult {
    pub success: bool,
    pub message: String,
    pub wiiu_game_dir: Option<String>,
    pub wiiu_update_dir: Option<String>,
    pub wiiu_dlc_dir: Option<String>,
    pub switch_game_dir: Option<String>,
    pub switch_dlc_dir: Option<String>,
}

/// 一键读取系统 BCML 配置文件 (%LOCALAPPDATA%/bcml/settings.json)
pub fn import_bcml_paths() -> BcmlImportResult {
    let local_appdata = match std::env::var("LOCALAPPDATA") {
        Ok(v) => v,
        Err(_) => {
            return BcmlImportResult {
                success: false,
                message: "未找到系统 LOCALAPPDATA 环境变量".to_string(),
                ..Default::default()
            };
        }
    };

    let bcml_path = PathBuf::from(local_appdata).join("bcml").join("settings.json");
    if !bcml_path.exists() {
        return BcmlImportResult {
            success: false,
            message: "未检测到系统 BCML 配置文件 (%LOCALAPPDATA%/bcml/settings.json)".to_string(),
            ..Default::default()
        };
    }

    let content = match std::fs::read_to_string(&bcml_path) {
        Ok(c) => c,
        Err(e) => {
            return BcmlImportResult {
                success: false,
                message: format!("读取 BCML 设置失败: {}", e),
                ..Default::default()
            };
        }
    };

    let val: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return BcmlImportResult {
                success: false,
                message: format!("解析 BCML JSON 失败: {}", e),
                ..Default::default()
            };
        }
    };

    let get_str = |key: &str| -> Option<String> {
        val.get(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
    };

    let wiiu_game_dir = get_str("game_dir");
    let wiiu_update_dir = get_str("update_dir");
    let wiiu_dlc_dir = get_str("dlc_dir");
    let switch_game_dir = get_str("game_dir_nx");
    let switch_dlc_dir = get_str("dlc_dir_nx");

    let has_any = wiiu_game_dir.is_some()
        || wiiu_update_dir.is_some()
        || wiiu_dlc_dir.is_some()
        || switch_game_dir.is_some()
        || switch_dlc_dir.is_some();

    BcmlImportResult {
        success: has_any,
        message: if has_any {
            "已成功从 BCML 导入游戏路径！".to_string()
        } else {
            "BCML 配置中未包含有效的游戏路径".to_string()
        },
        wiiu_game_dir,
        wiiu_update_dir,
        wiiu_dlc_dir,
        switch_game_dir,
        switch_dlc_dir,
    }
}
