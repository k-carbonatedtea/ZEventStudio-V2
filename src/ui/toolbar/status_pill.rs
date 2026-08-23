use gpui::*;
use crate::ui::state::AppState;

/// 渲染顶部工具栏右侧状态区域 (多语言切换按钮与高精度 FPS 药丸徽章)
pub fn render_status_pill(
    state: &AppState,
    fps: f32,
    frame_time: f32,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let perf_mode = state.performance_mode;
    let model_lang = model.clone();
    let model_fps_toggle = model.clone();

    // 动态 FPS 状态指示灯颜色
    let (fps_bg, fps_border, fps_dot) = if perf_mode {
        (
            rgba(0xf59e0b25),
            pal.secondary_accent,
            pal.secondary_accent,
        )
    } else if fps >= 50.0 {
        (
            rgba(0x00e5ba20),
            pal.primary_accent,
            pal.primary_accent,
        )
    } else if fps >= 28.0 {
        (
            rgba(0xf59e0b20),
            pal.secondary_accent,
            pal.secondary_accent,
        )
    } else {
        (
            rgba(0xef444420),
            rgb(0xEF4444),
            rgb(0xEF4444),
        )
    };

    let model_fit_view = model.clone();

    div()
        .flex()
        .items_center()
        .gap_2p5()
        // 缩放比例指示药丸 (点击一键自适应全图视口)
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .px_2p5()
                .py_1()
                .rounded_lg()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.card_border)
                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                .cursor_pointer()
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(pal.text_secondary)
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_fit_view.update(cx, |st, cx| {
                        st.fit_view();
                        cx.notify();
                    });
                })
                .child(format!("🔍 {:.0}%", (state.zoom * 100.0).round())),
        )
        // 多语言切换按钮
        .child(
            div()
                .px_2p5()
                .py_1()
                .rounded_lg()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.card_border)
                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                .cursor_pointer()
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(pal.text_secondary)
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
