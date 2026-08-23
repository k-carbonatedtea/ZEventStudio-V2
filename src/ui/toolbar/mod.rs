pub mod branding;
pub mod status_pill;

use gpui::*;
use branding::render_branding;
use status_pill::render_status_pill;
use crate::ui::state::AppState;

/// 渲染顶部现代化玻璃拟态工具栏 (Logo/Branding + Status Pill)
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
        .bg(rgb(0x161922))
        .border_b_1()
        .border_color(rgb(0x232838))
        .shadow_md()
        // Left: Logo & Active Graph Tag
        .child(render_branding(state))
        // Right: Language Switch & FPS / Performance Pill (点击切换性能/省电模式)
        .child(render_status_pill(state, fps, frame_time, model))
}

