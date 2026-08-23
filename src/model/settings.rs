use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::model::GameLanguage;

/// 游戏支持的平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlatformType {
    #[default]
    Switch,
    #[serde(rename = "wiiu")]
    WiiU,
}

#[allow(dead_code)]
impl PlatformType {
    pub const fn is_switch(&self) -> bool {
        matches!(self, Self::Switch)
    }

    pub const fn is_wiiu(&self) -> bool {
        matches!(self, Self::WiiU)
    }
}

/// Wii U 平台路径配置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WiiUPathSettings {
    pub game_dir: String,
    pub update_dir: String,
    pub dlc_dir: String,
    pub export_dir: String,
}

/// Switch (NX) 平台路径配置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SwitchPathSettings {
    pub game_dir: String,
    pub dlc_dir: String,
    pub export_dir: String,
}

/// 全局游戏与编辑器偏好设置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GamePathSettings {
    pub current_platform: PlatformType,
    pub language: GameLanguage,
    pub wiiu: WiiUPathSettings,
    pub switch: SwitchPathSettings,
    #[serde(default)]
    pub custom_ai_dump_dir: String,
}

impl Default for GamePathSettings {
    fn default() -> Self {
        Self {
            current_platform: PlatformType::Switch,
            language: GameLanguage::CNzh,
            wiiu: WiiUPathSettings::default(),
            switch: SwitchPathSettings::default(),
            custom_ai_dump_dir: String::new(),
        }
    }
}

impl GamePathSettings {
    /// 获取全局设置文件存储路径 (%APPDATA%/ZEventStudio/settings.json)
    pub fn get_settings_file_path() -> Option<PathBuf> {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let dir = PathBuf::from(appdata).join("ZEventStudio");
            let _ = fs::create_dir_all(&dir);
            return Some(dir.join("settings.json"));
        }
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let dir = PathBuf::from(userprofile).join(".zevent");
            let _ = fs::create_dir_all(&dir);
            return Some(dir.join("settings.json"));
        }
        None
    }

    /// 从本地磁盘加载设置
    pub fn load_from_disk() -> Self {
        if let Some(path) = Self::get_settings_file_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(settings) = serde_json::from_str::<Self>(&content) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    /// 保存设置至本地磁盘
    pub fn save_to_disk(&self) -> Result<(), String> {
        let path = Self::get_settings_file_path()
            .ok_or_else(|| "无法定位系统配置存储目录".to_string())?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化设置失败: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("写入配置文件失败: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_path_settings_serde_roundtrip() {
        let mut settings = GamePathSettings::default();
        settings.current_platform = PlatformType::WiiU;
        settings.language = GameLanguage::JPja;
        settings.wiiu.game_dir = "E:\\Botw\\content".to_string();
        settings.switch.export_dir = "E:\\Atmosphere\\contents".to_string();

        let json = serde_json::to_string(&settings).expect("serialize settings");
        let deserialized: GamePathSettings = serde_json::from_str(&json).expect("deserialize settings");

        assert_eq!(deserialized.current_platform, PlatformType::WiiU);
        assert_eq!(deserialized.language, GameLanguage::JPja);
        assert_eq!(deserialized.wiiu.game_dir, "E:\\Botw\\content");
        assert_eq!(deserialized.switch.export_dir, "E:\\Atmosphere\\contents");
    }
}
