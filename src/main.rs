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
        let pal = ui::theme::AppTheme::RelicCyan.palette();
        let base_hl = if pal.is_dark {
            gpui_component::highlighter::HighlightTheme::default_dark()
        } else {
            gpui_component::highlighter::HighlightTheme::default_light()
        };

        let mut style = base_hl.style.clone();
        style.editor_background = Some(pal.app_bg.into());
        style.editor_foreground = Some(pal.text_primary.into());
        style.editor_gutter_background = Some(pal.panel_surface.into());
        style.editor_active_line = Some(pal.card_surface.into());
        style.editor_line_number = Some(pal.text_secondary.into());
        style.editor_active_line_number = Some(pal.primary_accent.into());

        let theme = Theme::global_mut(cx);
        theme.mode = if pal.is_dark { ThemeMode::Dark } else { ThemeMode::Light };
        theme.colors.background = pal.app_bg.into();
        theme.colors.foreground = pal.text_primary.into();
        theme.colors.popover = pal.panel_surface.into();
        theme.colors.popover_foreground = pal.text_primary.into();
        theme.colors.primary = pal.primary_accent.into();
        theme.colors.primary_foreground = if pal.is_dark { rgb(0x0F1117).into() } else { rgb(0xFFFFFF).into() };
        theme.colors.secondary = pal.card_surface.into();
        theme.colors.secondary_foreground = pal.text_primary.into();
        theme.colors.muted = pal.card_surface.into();
        theme.colors.muted_foreground = pal.text_secondary.into();
        theme.colors.accent = pal.card_surface.into();
        theme.colors.accent_foreground = pal.text_primary.into();
        theme.colors.border = pal.panel_border.into();
        theme.colors.input = pal.panel_border.into();
        theme.colors.title_bar = pal.panel_surface.into();
        theme.colors.title_bar_border = pal.panel_border.into();
        theme.tokens = (&theme.colors).into();

        theme.highlight_theme = std::sync::Arc::new(gpui_component::highlighter::HighlightTheme {
            name: "ZEventStudio-RelicCyan".to_string(),
            appearance: if pal.is_dark { ThemeMode::Dark } else { ThemeMode::Light },
            style,
        });

        Theme::sync_base(cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1280.0), px(800.0)),
                cx,
            ))),
            ..TitleBar::window_options()
        };

        cx.open_window(window_options, |window, cx| {
            let app_view = cx.new(AppView::new);
            let model = app_view.read(cx).model.clone();

            window.on_window_should_close(cx, move |_window, cx| {
                let model = model.clone();
                let mut should_close = true;
                model.update(cx, |st, cx| {
                    should_close = st.request_exit_application();
                    cx.notify();
                });
                should_close
            });

            app_view
        })
        .expect("Failed to open ZEvent Studio main window");
    });
}

