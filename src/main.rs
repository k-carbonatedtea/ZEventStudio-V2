#![recursion_limit = "2048"]

pub mod i18n;
mod model;
mod ui;


use gpui::*;
use gpui_component::TitleBar;
use ui::AppView;

/// 主应用程序入口函数
fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);
    app.run(|cx: &mut App| {
        // 初始化 gpui-component 基础组件系统并设置为统一暗色主题
        gpui_component::init(cx);
        gpui_component::Theme::change(gpui_component::ThemeMode::Dark, None, cx);

        // 统一注入全局设计系统调色板规范 (Design System Theme Tokens)
        ui::theme::AppTheme::default().palette().apply_to_gpui_component(cx);

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
            let focus = app_view.read(cx).focus_handle.clone();
            focus.focus(window, cx);

            // 自动启动本地 MCP 插件桥接服务 (监听 127.0.0.1:31415)
            ui::state_plugins::start_plugin_bridge_server(&model, cx);

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

