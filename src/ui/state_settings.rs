use gpui::*;
use crate::model::{GameLanguage, GamePathSettings, PlatformType};
use crate::ui::components::Toast;
use crate::ui::state::AppState;

/// 首选项设置分类选项卡 (Settings Tabs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    GameAi,
    Switch,
    WiiU,
    Canvas,
}

/// 画布背景网格样式 (Grid Rendering Style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridStyle {
    #[default]
    Dots,
    Lines,
    None,
}

/// 节点连线拟合曲线算法 (Connection Line Style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgeCurveStyle {
    #[default]
    Bezier,
    Smooth,
}

/// 目标游戏引擎 (Target Game Engine)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetEngine {
    #[default]
    BotW,
    TotK,
}

impl AppState {
    /// 打开首选项设置弹窗并初始化编辑草稿
    pub fn open_settings(&mut self, tab: SettingsTab) {
        self.settings_modal_open = true;
        self.settings_active_tab = tab;
        self.settings_draft = self.game_paths.clone();
    }

    /// 关闭首选项设置弹窗（放弃未保存的草稿修改）
    pub fn close_settings(&mut self) {
        self.settings_modal_open = false;
        self.active_settings_field = crate::ui::state::ActiveSettingsField::None;
    }

    /// 保存并应用偏好设置与游戏路径
    pub fn save_and_apply_settings(&mut self) {
        self.game_paths = self.settings_draft.clone();
        self.game_lang = self.game_paths.language;
        if self.game_paths.current_platform == PlatformType::WiiU {
            self.target_engine = TargetEngine::BotW;
        }

        if let Err(e) = self.game_paths.save_to_disk() {
            let msg = format!("{}: {}", self.t("toast.settings_save_err"), e);
            self.toast = Some(Toast::error(&msg));
        } else {
            self.toast = Some(Toast::success(self.t("toast.settings_save_ok")));
        }
        self.settings_modal_open = false;
        self.active_settings_field = crate::ui::state::ActiveSettingsField::None;
    }

    /// 从 BCML 自动同步游戏、补丁与 DLC 路径
    pub fn sync_from_bcml(&mut self) {
        let result = crate::model::import_bcml_paths();
        if result.success {
            if let Some(wg) = result.wiiu_game_dir {
                self.settings_draft.wiiu.game_dir = wg;
            }
            if let Some(wu) = result.wiiu_update_dir {
                self.settings_draft.wiiu.update_dir = wu;
            }
            if let Some(wd) = result.wiiu_dlc_dir {
                self.settings_draft.wiiu.dlc_dir = wd;
            }
            if let Some(sg) = result.switch_game_dir {
                self.settings_draft.switch.game_dir = sg;
            }
            if let Some(sd) = result.switch_dlc_dir {
                self.settings_draft.switch.dlc_dir = sd;
            }
            self.toast = Some(Toast::success(result.message));
        } else {
            self.toast = Some(Toast::info(result.message));
        }
    }

    /// 恢复首选项为默认配置
    pub fn reset_settings_to_default(&mut self) {
        self.settings_draft = GamePathSettings::default();
        self.lang = crate::i18n::Language::Zh;
        self.game_lang = GameLanguage::CNzh;
        self.show_welcome_on_startup = true;
        self.expand_all_params = false;
        self.grid_style = GridStyle::Dots;
        self.edge_curve_style = EdgeCurveStyle::Bezier;
        self.show_perf_stats = true;
        self.target_engine = TargetEngine::BotW;
        self.auto_backup = true;
        self.toast = Some(Toast::info(self.t("toast.settings_reset")));
    }

    /// 清理临时自动保存快照与缓存
    pub fn clean_autosave_cache(&mut self) {
        if let Some(settings_path) = GamePathSettings::get_settings_file_path() {
            if let Some(dir) = settings_path.parent() {
                let autosave_dir = dir.join("autosave");
                if autosave_dir.exists() {
                    let _ = std::fs::remove_dir_all(&autosave_dir);
                }
            }
        }
        self.toast = Some(Toast::success(self.t("toast.cache_cleared")));
    }
}

/// 弹出原生目录选择器以配置游戏路径
pub fn browse_platform_folder(
    model: &Entity<AppState>,
    platform: crate::model::PlatformType,
    field: &'static str,
    title: &'static str,
    cx: &mut App,
) {
    let model = model.clone();
    cx.spawn(async move |cx| {
        let handle = rfd::AsyncFileDialog::new()
            .set_title(title)
            .pick_folder()
            .await;

        if let Some(folder) = handle {
            let path_str = folder.path().to_string_lossy().to_string();
            cx.update(|cx| {
                model.update(cx, |st, cx| {
                    match platform {
                        crate::model::PlatformType::Switch => {
                            match field {
                                "game_dir" => st.settings_draft.switch.game_dir = path_str,
                                "dlc_dir" => st.settings_draft.switch.dlc_dir = path_str,
                                "export_dir" => st.settings_draft.switch.export_dir = path_str,
                                _ => {}
                            }
                        }
                        crate::model::PlatformType::WiiU => {
                            match field {
                                "game_dir" => st.settings_draft.wiiu.game_dir = path_str,
                                "update_dir" => st.settings_draft.wiiu.update_dir = path_str,
                                "dlc_dir" => st.settings_draft.wiiu.dlc_dir = path_str,
                                "export_dir" => st.settings_draft.wiiu.export_dir = path_str,
                                _ => {}
                            }
                        }
                    }
                    cx.notify();
                });
            });
        }
    })
    .detach();
}
