pub mod i18n;
mod model;
mod ui;


use gpui::*;
use gpui_component::{Theme, ThemeMode};
use ui::AppView;

/// 主应用程序入口函数
fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        // 初始化 gpui-component 基础组件系统并设置为统一暗色主题
        gpui_component::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);

        // 统一代码编辑器背景色、行号槽 (Gutter) 与激活行高亮，保证与全局深色玻璃拟态风格一致
        let theme = Theme::global_mut(cx);
        let mut style = theme.highlight_theme.style.clone();
        let editor_bg = hsla(240.0 / 360.0, 0.06, 0.07, 1.0);
        let gutter_bg = hsla(240.0 / 360.0, 0.06, 0.07, 1.0);
        let active_line = hsla(240.0 / 360.0, 0.06, 0.12, 1.0);
        let line_number = hsla(240.0 / 360.0, 0.05, 0.40, 1.0);
        let active_line_number = hsla(240.0 / 360.0, 0.05, 0.85, 1.0);

        style.editor_background = Some(editor_bg);
        style.editor_gutter_background = Some(gutter_bg);
        style.editor_active_line = Some(active_line);
        style.editor_line_number = Some(line_number);
        style.editor_active_line_number = Some(active_line_number);

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
            titlebar: Some(TitlebarOptions {
                title: Some("ZEvent Studio - GPU-Accelerated Nodeflow Editor".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        cx.open_window(window_options, |_, cx| {
            cx.new(AppView::new)
        })
        .expect("Failed to open ZEvent Studio main window");
    });
}

