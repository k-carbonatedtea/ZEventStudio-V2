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

        let gpui_theme = gpui_component::Theme::global_mut(cx);
        gpui_theme.mode = if pal.is_dark {
            gpui_component::ThemeMode::Dark
        } else {
            gpui_component::ThemeMode::Light
        };
        gpui_theme.colors.background = pal.app_bg.into();
        gpui_theme.colors.foreground = pal.text_primary.into();
        gpui_theme.colors.popover = pal.panel_surface.into();
        gpui_theme.colors.popover_foreground = pal.text_primary.into();
        gpui_theme.colors.primary = pal.primary_accent.into();
        gpui_theme.colors.primary_foreground = if pal.is_dark {
            rgb(0x0F1117).into()
        } else {
            rgb(0xFFFFFF).into()
        };
        gpui_theme.colors.secondary = pal.card_surface.into();
        gpui_theme.colors.secondary_foreground = pal.text_primary.into();
        gpui_theme.colors.muted = pal.card_surface.into();
        gpui_theme.colors.muted_foreground = pal.text_secondary.into();
        gpui_theme.colors.accent = pal.card_surface.into();
        gpui_theme.colors.accent_foreground = pal.text_primary.into();
        gpui_theme.colors.border = pal.panel_border.into();
        gpui_theme.colors.input = pal.panel_border.into();
        gpui_theme.colors.title_bar = pal.panel_surface.into();
        gpui_theme.colors.title_bar_border = pal.panel_border.into();

        // 统一代码编辑器高亮风格
        let mut style = gpui_theme.highlight_theme.style.clone();
        style.editor_background = Some(pal.app_bg.into());
        style.editor_gutter_background = Some(pal.panel_surface.into());
        style.editor_active_line = Some(pal.card_surface.into());
        style.editor_line_number = Some(pal.text_secondary.into());
        style.editor_active_line_number = Some(pal.primary_accent.into());

        gpui_theme.highlight_theme = std::sync::Arc::new(gpui_component::highlighter::HighlightTheme {
            name: gpui_theme.highlight_theme.name.clone(),
            appearance: if pal.is_dark {
                gpui_component::ThemeMode::Dark
            } else {
                gpui_component::ThemeMode::Light
            },
            style,
        });

        self.toast = Some(Toast::success(format!("🎨 已切换主题: {}", self.t(theme.i18n_key()))));
    }
}
