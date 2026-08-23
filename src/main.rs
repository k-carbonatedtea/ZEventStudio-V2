pub mod i18n;
mod model;
mod ui;


use gpui::*;
use gpui_component::{Theme, ThemeMode, TitleBar};
use ui::AppView;

/// 主应用程序入口函数
fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);
    app.run(|cx: &mut App| {
        // 初始化 gpui-component 基础组件系统并设置为统一暗色主题
        gpui_component::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);

        // 统一注入全局设计系统调色板规范 (Design System Theme Tokens)
        let theme = Theme::global_mut(cx);
        theme.colors.background = rgb(0x0F1117).into();
        theme.colors.foreground = rgb(0xF1F5F9).into();
        theme.colors.popover = rgb(0x161922).into();
        theme.colors.popover_foreground = rgb(0xF1F5F9).into();
        theme.colors.primary = rgb(0x00E5BA).into();
        theme.colors.primary_foreground = rgb(0x0F1117).into();
        theme.colors.secondary = rgb(0x1E2330).into();
        theme.colors.secondary_foreground = rgb(0xF1F5F9).into();
        theme.colors.muted = rgb(0x1E2330).into();
        theme.colors.muted_foreground = rgb(0x94A3B8).into();
        theme.colors.accent = rgb(0x1E2330).into();
        theme.colors.accent_foreground = rgb(0xF1F5F9).into();
        theme.colors.border = rgb(0x232838).into();
        theme.colors.input = rgb(0x232838).into();
        theme.colors.title_bar = rgb(0x161922).into();
        theme.colors.title_bar_border = rgb(0x232838).into();

        // 统一代码编辑器背景色、行号槽 (Gutter) 与激活行高亮
        let mut style = theme.highlight_theme.style.clone();
        let editor_bg = rgb(0x0F1117); // 深石墨黑 App Background
        let gutter_bg = rgb(0x161922); // 蓝灰表面 Panel Surface
        let active_line = rgb(0x1E2330); // 浅层蓝灰 Card / Hover
        let line_number = rgb(0x94A3B8); // 蓝灰白 Text Secondary
        let active_line_number = rgb(0x00E5BA); // 遗迹青蓝 Primary Accent

        style.editor_background = Some(editor_bg.into());
        style.editor_gutter_background = Some(gutter_bg.into());
        style.editor_active_line = Some(active_line.into());
        style.editor_line_number = Some(line_number.into());
        style.editor_active_line_number = Some(active_line_number.into());

        theme.highlight_theme = std::sync::Arc::new(gpui_component::highlighter::HighlightTheme {
            name: theme.highlight_theme.name.clone(),
            appearance: ThemeMode::Dark,
            style,
        });

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1280.0), px(800.0)),
                cx,
            ))),
            ..TitleBar::window_options()
        };

        cx.open_window(window_options, |_, cx| {
            cx.new(AppView::new)
        })
        .expect("Failed to open ZEvent Studio main window");
    });
}

