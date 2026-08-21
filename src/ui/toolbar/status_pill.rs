use gpui::*;
use crate::ui::state::AppState;

/// 渲染顶部工具栏右侧状态区域 (多语言切换按钮与高精度 FPS 药丸徽章)
pub fn render_status_pill(
    state: &AppState,
    fps: f32,
    frame_time: f32,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let perf_mode = state.performance_mode;
    let model_lang = model.clone();
    let model_fps_toggle = model.clone();

    // 动态 FPS 状态指示灯颜色
    let (fps_bg, fps_border, fps_dot) = if perf_mode {
        (
            hsla(15.0 / 360.0, 0.65, 0.20, 0.95),
            hsla(25.0 / 360.0, 0.95, 0.55, 1.0),
            hsla(25.0 / 360.0, 1.0, 0.60, 1.0),
        )
    } else if fps >= 50.0 {
        (
            hsla(145.0 / 360.0, 0.45, 0.16, 0.95),
            hsla(145.0 / 360.0, 0.70, 0.45, 1.0),
            hsla(145.0 / 360.0, 0.90, 0.60, 1.0),
        )
    } else if fps >= 28.0 {
        (
            hsla(45.0 / 360.0, 0.45, 0.16, 0.95),
            hsla(45.0 / 360.0, 0.70, 0.45, 1.0),
            hsla(45.0 / 360.0, 0.90, 0.60, 1.0),
        )
    } else {
        (
            hsla(0.0, 0.45, 0.16, 0.95),
            hsla(0.0, 0.70, 0.45, 1.0),
            hsla(0.0, 0.90, 0.60, 1.0),
        )
    };

    div()
        .flex()
        .items_center()
        .gap_2p5()
        // 多语言切换按钮
        .child(
            div()
                .px_2p5()
                .py_1()
                .rounded_lg()
                .bg(hsla(220.0 / 360.0, 0.25, 0.16, 1.0))
                .border_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.28, 1.0))
                .hover(|s| s.bg(hsla(220.0 / 360.0, 0.35, 0.25, 1.0)))
                .cursor_pointer()
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(hsla(215.0 / 360.0, 0.25, 0.85, 1.0))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_lang.update(cx, |st, cx| {
                        st.lang = st.lang.toggle();
                        cx.notify();
                    });
                })
                .child(state.t("toolbar.lang_btn")),
        )
        // 实时 FPS 监控胶囊 (点击直接切换极速性能模式)
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .px_2p5()
                .py_1()
                .rounded_lg()
                .bg(fps_bg)
                .border_1()
                .border_color(fps_border)
                .shadow_sm()
                .cursor_pointer()
                .hover(|s| s.opacity(0.88))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_fps_toggle.update(cx, |st, cx| {
                        st.toggle_performance_mode();
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .size(px(7.0))
                        .rounded_full()
                        .bg(fps_dot),
                )
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(11.0))
                        .text_color(rgb(0xffffff))
                        .child(if perf_mode {
                            format!("🔥 {:.0} FPS ({:.1}ms)", fps, frame_time)
                        } else {
                            format!("{:.0} FPS ({:.1}ms)", fps, frame_time)
                        }),
                ),
        )
}
