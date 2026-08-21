pub mod i18n;
mod model;
mod presets;
mod ui;

use gpui::*;
use ui::AppView;

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        gpui_component::init(cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1280.0), px(800.0)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("NodeFlow Studio - GPU-Accelerated Nodeflow Editor".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        cx.open_window(window_options, |_, cx| {
            cx.new(|cx| AppView::new(cx))
        })
        .expect("Failed to open NodeFlow Studio main window");
    });
}
