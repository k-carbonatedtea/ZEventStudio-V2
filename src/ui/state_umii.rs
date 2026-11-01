//! UMii 形象参数文件加载、格式转换导出与 3D 合成派发

use crate::model::document_tab::{OpenDocumentTab};
use crate::model::umii::{
    collect_model_search_dirs, convert_mii_to_umii, ModelPlatform, UmiiData, UmiiDocumentState,
};
use crate::ui::components::Toast;
use crate::ui::state::{AppState, AppWorkspaceTab};

impl AppState {
    /// 加载 BotW UMii 形象文件 (.bumii / .umii.yml / .mii / .charinfo / .ffsd) 至专属编辑器
    pub fn load_umii_tab(&mut self, path: &str, raw_name: &str, decompressed: &[u8]) -> bool {
        let umii_data = match parse_umii_data_from_bytes(decompressed, raw_name) {
            Ok(data) => data,
            Err(e) => {
                self.toast = Some(Toast::error(format!("{}: {}", self.t("umii.toast_load_failed"), e)));
                return false;
            }
        };

        let initial_platform = detect_initial_platform(path);
        let mut umii_doc = UmiiDocumentState::new(path.to_string(), raw_name.to_string(), umii_data);
        umii_doc.target_platform = initial_platform;

        let search_dirs = collect_model_search_dirs(initial_platform, self.project_path.as_deref());
        let _ = umii_doc.synthesize_model(&search_dirs);

        let new_id = uuid::Uuid::new_v4().to_string();
        let new_tab = OpenDocumentTab::new_umii(
            new_id.clone(),
            raw_name.to_string(),
            path.to_string(),
            "bumii".to_string(),
            umii_doc,
        );

        self.open_tabs.push(new_tab);
        self.active_tab_id = Some(new_id);
        self.current_tab = AppWorkspaceTab::UmiiEditor;
        self.toast = Some(Toast::success(format!("{} {}", self.t("toast.file_loaded"), raw_name)));
        true
    }

    /// 重新合成当前活跃 UMii 标签页的 3D 模型
    #[allow(dead_code)]
    pub fn resynthesize_active_umii(&mut self) {
        let Some(active_id) = &self.active_tab_id else { return; };
        let proj_root = self.project_path.clone();
        if let Some(tab) = self.open_tabs.iter_mut().find(|t| &t.id == active_id) {
            if let Some(ref mut u) = tab.umii_state {
                let search_dirs = collect_model_search_dirs(u.target_platform, proj_root.as_deref());
                match u.synthesize_model(&search_dirs) {
                    Ok(_) => {
                        self.toast = Some(Toast::success(self.t("umii.resynth_success")));
                    }
                    Err(e) => {
                        self.toast = Some(Toast::error(format!("{}: {}", self.t("umii.resynth_failed"), e)));
                    }
                }
            }
        }
    }

    /// 导入外部 Mii 二进制文件并覆盖当前形象参数
    #[allow(dead_code)]
    pub fn import_external_mii_to_active(&mut self, file_bytes: &[u8], file_name: &str) -> bool {
        let Some(active_id) = self.active_tab_id.clone() else { return false; };
        let decompressed = roead::yaz0::decompress(file_bytes).unwrap_or_else(|_| file_bytes.to_vec());
        let umii_data = match parse_umii_data_from_bytes(&decompressed, file_name) {
            Ok(d) => d,
            Err(e) => {
                self.toast = Some(Toast::error(format!("{}: {}", self.t("umii.toast_import_failed"), e)));
                return false;
            }
        };

        let proj_root = self.project_path.clone();
        if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == active_id) {
            if let Some(ref mut u) = tab.umii_state {
                u.data = umii_data;
                tab.is_modified = u.is_modified();
                let search_dirs = collect_model_search_dirs(u.target_platform, proj_root.as_deref());
                let _ = u.synthesize_model(&search_dirs);
            }
        }
        self.toast = Some(Toast::success(self.t("umii.toast_import_success")));
        true
    }
}

/// 解析任意格式的 Mii / UMii 字节数据
pub fn parse_umii_data_from_bytes(bytes: &[u8], name_hint: &str) -> Result<UmiiData, String> {
    if bytes.starts_with(b"AAMP") {
        return UmiiData::from_binary(bytes);
    }

    if let Ok(text) = std::str::from_utf8(bytes) {
        if text.contains("ffsd:") || text.contains("Personal:") || text.contains("Body:") {
            return UmiiData::from_yaml_text(text);
        }
    }

    // 尝试经由 MiiConverter 解析 Wii / Wii U / 3DS / Switch 原生 Mii 格式
    convert_mii_to_umii(bytes, Some(name_hint))
}

/// 根据文件路径推测运行平台 (Switch NX 或 Wii U Cafe)
fn detect_initial_platform(path: &str) -> ModelPlatform {
    let lower = path.to_lowercase();
    if lower.contains("01007ef00011e000") || lower.contains("switch") || lower.contains("nx") {
        ModelPlatform::Switch
    } else if lower.contains("tes") || lower.contains("wiiu") || lower.contains("cafe") {
        ModelPlatform::WiiU
    } else {
        ModelPlatform::Switch
    }
}
