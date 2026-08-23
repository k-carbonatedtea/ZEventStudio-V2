use crate::model::GameLanguage;
use crate::ui::state::AppState;

/// 首选项设置分类选项卡 (Settings Tabs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Canvas,
    Performance,
    Game,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetEngine {
    #[default]
    BotW,
    TotK,
}

impl AppState {
    /// 打开首选项设置弹窗
    pub fn open_settings(&mut self, tab: SettingsTab) {
        self.settings_modal_open = true;
        self.settings_active_tab = tab;
    }

    /// 关闭首选项设置弹窗
    pub fn close_settings(&mut self) {
        self.settings_modal_open = false;
    }

    /// 恢复首选项为默认配置
    pub fn reset_settings_to_default(&mut self) {
        self.lang = crate::i18n::Language::Zh;
        self.game_lang = GameLanguage::CNzh;
        self.show_welcome_on_startup = true;
        self.expand_all_params = false;
        self.grid_style = GridStyle::Dots;
        self.edge_curve_style = EdgeCurveStyle::Bezier;
        self.performance_mode = false;
        self.show_perf_stats = true;
        self.target_engine = TargetEngine::BotW;
        self.auto_backup = true;
        self.toast = Some(crate::ui::components::Toast::info(self.t("toast.settings_reset")));
    }
}
