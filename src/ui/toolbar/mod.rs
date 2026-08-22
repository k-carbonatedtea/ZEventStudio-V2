pub mod branding;
pub mod controls;
pub mod status_pill;

use gpui::*;
use branding::render_branding;
use controls::render_controls;
use status_pill::render_status_pill;
use crate::ui::state::AppState;

/// 渲染顶部现代化玻璃拟态工具栏 (Logo/Branding + Central Controls + Status Pill)
pub fn render_toolbar(
    state: &AppState,
    fps: f32,
    frame_time: f32,
    model: &Entity<AppState>,
) -> impl IntoElement {
    div()
        .h(px(48.0))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .bg(hsla(220.0 / 360.0, 0.35, 0.11, 0.98))
        .border_b_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.20, 1.0))
        .shadow_md()
        // Left: Logo & Active Graph Tag
        .child(render_branding(state))
        // Center: Interactive Controls (Auto Layout, Expand Params, Knife Mode)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(render_controls(state, model)),
        )
        // Right: Language Switch & FPS / Performance Pill (点击切换性能/省电模式)
        .child(render_status_pill(state, fps, frame_time, model))
}
