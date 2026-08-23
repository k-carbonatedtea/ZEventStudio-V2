use gpui::*;
use serde::{Deserialize, Serialize};

/// 应用程序支持的内置主题风格枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppTheme {
    /// 遗迹青蓝 (默认深石墨黑与古老青绿)
    #[default]
    RelicCyan,
    /// 旷野黄昏 (暮光琥珀金与暖棕黑沉浸风格)
    TwilightAmber,
    /// 希卡科技 (高对比度科技黑与天蓝发光)
    SheikahSlate,
    /// 暗夜紫微 (深紫与霓虹粉紫点缀)
    MidnightPurple,
    /// 苔原深绿 (克洛格森林与清新翡翠绿)
    EmeraldForest,
    /// 极简晨曦 (明亮浅色冷灰纸感模式)
    NordicLight,
}

impl AppTheme {
    /// 返回所有可用的内置主题列表
    pub const ALL: [AppTheme; 6] = [
        AppTheme::RelicCyan,
        AppTheme::TwilightAmber,
        AppTheme::SheikahSlate,
        AppTheme::MidnightPurple,
        AppTheme::EmeraldForest,
        AppTheme::NordicLight,
    ];

    /// 获取主题对应的多语言键值
    pub fn i18n_key(&self) -> &'static str {
        match self {
            AppTheme::RelicCyan => "theme.relic_cyan",
            AppTheme::TwilightAmber => "theme.twilight_amber",
            AppTheme::SheikahSlate => "theme.sheikah_slate",
            AppTheme::MidnightPurple => "theme.midnight_purple",
            AppTheme::EmeraldForest => "theme.emerald_forest",
            AppTheme::NordicLight => "theme.nordic_light",
        }
    }

    /// 获取当前主题的完整调色板规范
    pub fn palette(&self) -> ThemePalette {
        match self {
            AppTheme::RelicCyan => ThemePalette {
                name: "Relic Cyan",
                is_dark: true,
                app_bg: rgb(0x0F1117),
                panel_surface: rgb(0x161922),
                card_surface: rgb(0x1E2330),
                card_hover: rgb(0x252C3D),
                primary_accent: rgb(0x00E5BA),
                primary_accent_hover: rgb(0x33EBCA),
                secondary_accent: rgb(0xF59E0B),
                text_primary: rgb(0xF1F5F9),
                text_secondary: rgb(0x94A3B8),
                panel_border: rgb(0x232838),
                card_border: rgb(0x2D3548),
            },
            AppTheme::TwilightAmber => ThemePalette {
                name: "Twilight Amber",
                is_dark: true,
                app_bg: rgb(0x12100E),
                panel_surface: rgb(0x1A1614),
                card_surface: rgb(0x241E1A),
                card_hover: rgb(0x302722),
                primary_accent: rgb(0xF59E0B),
                primary_accent_hover: rgb(0xFBBF24),
                secondary_accent: rgb(0xFB7185),
                text_primary: rgb(0xFAFAF9),
                text_secondary: rgb(0xA8A29E),
                panel_border: rgb(0x2E2520),
                card_border: rgb(0x3D302A),
            },
            AppTheme::SheikahSlate => ThemePalette {
                name: "Sheikah Slate",
                is_dark: true,
                app_bg: rgb(0x0B0E14),
                panel_surface: rgb(0x111622),
                card_surface: rgb(0x182030),
                card_hover: rgb(0x202B40),
                primary_accent: rgb(0x00C4FF),
                primary_accent_hover: rgb(0x38BDF8),
                secondary_accent: rgb(0x38BDF8),
                text_primary: rgb(0xF8FAFC),
                text_secondary: rgb(0x64748B),
                panel_border: rgb(0x1E293B),
                card_border: rgb(0x283548),
            },
            AppTheme::MidnightPurple => ThemePalette {
                name: "Midnight Purple",
                is_dark: true,
                app_bg: rgb(0x0F0E17),
                panel_surface: rgb(0x161424),
                card_surface: rgb(0x201D33),
                card_hover: rgb(0x2B2745),
                primary_accent: rgb(0xA855F7),
                primary_accent_hover: rgb(0xC084FC),
                secondary_accent: rgb(0xEC4899),
                text_primary: rgb(0xF3F4F6),
                text_secondary: rgb(0x9CA3AF),
                panel_border: rgb(0x2B2640),
                card_border: rgb(0x3B3456),
            },
            AppTheme::EmeraldForest => ThemePalette {
                name: "Emerald Forest",
                is_dark: true,
                app_bg: rgb(0x0B1311),
                panel_surface: rgb(0x101C19),
                card_surface: rgb(0x162824),
                card_hover: rgb(0x1E3630),
                primary_accent: rgb(0x10B981),
                primary_accent_hover: rgb(0x34D399),
                secondary_accent: rgb(0x84CC16),
                text_primary: rgb(0xF0FDF4),
                text_secondary: rgb(0x6EE7B7),
                panel_border: rgb(0x1F3832),
                card_border: rgb(0x2B4C44),
            },
            AppTheme::NordicLight => ThemePalette {
                name: "Nordic Light",
                is_dark: false,
                app_bg: rgb(0xF8FAFC),
                panel_surface: rgb(0xFFFFFF),
                card_surface: rgb(0xF1F5F9),
                card_hover: rgb(0xE2E8F0),
                primary_accent: rgb(0x0284C7),
                primary_accent_hover: rgb(0x0369A1),
                secondary_accent: rgb(0xD97706),
                text_primary: rgb(0x0F172A),
                text_secondary: rgb(0x64748B),
                panel_border: rgb(0xE2E8F0),
                card_border: rgb(0xCBD5E1),
            },
        }
    }
}

/// 完整主题调色板规范 (Theme Palette Definition)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub name: &'static str,
    pub is_dark: bool,
    pub app_bg: Rgba,
    pub panel_surface: Rgba,
    pub card_surface: Rgba,
    pub card_hover: Rgba,
    pub primary_accent: Rgba,
    pub primary_accent_hover: Rgba,
    pub secondary_accent: Rgba,
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub panel_border: Rgba,
    pub card_border: Rgba,
}

impl ThemePalette {
    /// 将当前调色板一次性同步至 gpui_component 全局主题系统
    ///
    /// 统一收拢颜色映射、代码编辑器高亮风格等全部 GPUI Theme Token 的设置逻辑，
    /// 避免在初始化入口与运行时主题切换两处重复维护相同的颜色映射代码。
    pub fn apply_to_gpui_component(&self, cx: &mut App) {
        let gpui_theme = gpui_component::Theme::global_mut(cx);
        gpui_theme.mode = if self.is_dark {
            gpui_component::ThemeMode::Dark
        } else {
            gpui_component::ThemeMode::Light
        };
        gpui_theme.colors.background = self.app_bg.into();
        gpui_theme.colors.foreground = self.text_primary.into();
        gpui_theme.colors.popover = self.panel_surface.into();
        gpui_theme.colors.popover_foreground = self.text_primary.into();
        gpui_theme.colors.primary = self.primary_accent.into();
        gpui_theme.colors.primary_foreground = if self.is_dark {
            rgb(0x0F1117).into()
        } else {
            rgb(0xFFFFFF).into()
        };
        gpui_theme.colors.secondary = self.card_surface.into();
        gpui_theme.colors.secondary_foreground = self.text_primary.into();
        gpui_theme.colors.muted = self.card_surface.into();
        gpui_theme.colors.muted_foreground = self.text_secondary.into();
        gpui_theme.colors.accent = self.card_surface.into();
        gpui_theme.colors.accent_foreground = self.text_primary.into();
        gpui_theme.colors.border = self.panel_border.into();
        gpui_theme.colors.input = self.panel_border.into();
        gpui_theme.colors.title_bar = self.panel_surface.into();
        gpui_theme.colors.title_bar_border = self.panel_border.into();
        gpui_theme.tokens = (&gpui_theme.colors).into();

        // 同步代码编辑器语法高亮风格 (从对应明暗默认主题派生)
        let base_highlight = if self.is_dark {
            gpui_component::highlighter::HighlightTheme::default_dark()
        } else {
            gpui_component::highlighter::HighlightTheme::default_light()
        };

        let mut style = base_highlight.style.clone();
        style.editor_background = Some(self.app_bg.into());
        style.editor_foreground = Some(self.text_primary.into());
        style.editor_gutter_background = Some(self.panel_surface.into());
        style.editor_active_line = Some(self.card_surface.into());
        style.editor_line_number = Some(self.text_secondary.into());
        style.editor_active_line_number = Some(self.primary_accent.into());

        gpui_theme.highlight_theme = std::sync::Arc::new(
            gpui_component::highlighter::HighlightTheme {
                name: format!("{}-Custom", self.name),
                appearance: if self.is_dark {
                    gpui_component::ThemeMode::Dark
                } else {
                    gpui_component::ThemeMode::Light
                },
                style,
            },
        );

        gpui_component::Theme::sync_base(cx);
    }
}
