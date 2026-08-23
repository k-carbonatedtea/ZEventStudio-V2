use gpui::*;
use crate::ui::components::Toast;
use crate::ui::state::AppState;
use crate::ui::theme::{AppTheme, ThemePalette};

impl AppState {
    /// 获取当前激活主题调色板
    pub fn theme(&self) -> ThemePalette {
        self.current_theme.palette()
    }

    /// 动态切换主题并同步更新底层 GPUI 组件系统
    pub fn set_theme(&mut self, theme: AppTheme, cx: &mut Context<Self>) {
        self.current_theme = theme;
        let pal = theme.palette();

        // 使用统一收拢的主题同步方法，避免与 main.rs 初始化逻辑重复维护
        pal.apply_to_gpui_component(cx);

        self.toast = Some(Toast::success(format!("🎨 已切换主题: {}", self.t(theme.i18n_key()))));
    }
}

