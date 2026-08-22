pub mod i18n;
mod model;
mod ui;


use gpui::*;
use ui::AppView;

/// 主应用程序入口函数
fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        // 初始化 gpui-component 基础组件系统
        gpui_component::init(cx);

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

