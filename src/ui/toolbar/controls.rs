use gpui::*;
use crate::ui::state::AppState;

/// 渲染顶部工具栏中央控制按钮组
pub fn render_controls(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let knife_mode = state.knife_mode;
    let expand_all = state.expand_all_params;
    let perf_mode = state.performance_mode;

    let model_layout = model.clone();
    let model_expand = model.clone();
    let model_knife = model.clone();
    let model_perf = model.clone();
    let model_fit = model.clone();

    div()
        .flex()
        .items_center()
        .gap_2()
        // 1. 自动排版按钮 (Auto Layout)
        .child(
            div()
                .px_3()
                .py_1p5()
                .rounded_lg()
                .bg(hsla(215.0 / 360.0, 0.40, 0.22, 1.0))
                .border_1()
                .border_color(hsla(215.0 / 360.0, 0.50, 0.40, 1.0))
                .hover(|s| s.bg(hsla(215.0 / 360.0, 0.50, 0.30, 1.0)))
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_1p5()
                .text_size(px(11.5))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_layout.update(cx, |st, cx| {
                        st.auto_layout();
                        cx.notify();
                    });
                })
                .child(format!("✨ {}", state.t("toolbar.auto_layout"))),
        )
        // 2. 展开 / 折叠节点参数按钮
        .child(
            div()
                .px_3()
                .py_1p5()
                .rounded_lg()
                .bg(if expand_all {
                    hsla(271.0 / 360.0, 0.50, 0.28, 1.0)
                } else {
                    hsla(220.0 / 360.0, 0.25, 0.18, 1.0)
                })
                .border_1()
                .border_color(if expand_all {
                    hsla(271.0 / 360.0, 0.75, 0.55, 1.0)
                } else {
                    hsla(220.0 / 360.0, 0.25, 0.28, 1.0)
                })
                .hover(|s| s.opacity(0.85))
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_1p5()
                .text_size(px(11.5))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_expand.update(cx, |st, cx| {
                        st.toggle_expand_params();
                        cx.notify();
                    });
                })
                .child(if expand_all {
                    format!("📋 {}", state.t("toolbar.collapse_params"))
                } else {
                    format!("📋 {}", state.t("toolbar.expand_params"))
                }),
        )
        // 3. 极速性能模式切换 (GPU Extreme Performance Toggle)
        .child(
            div()
                .px_3()
                .py_1p5()
                .rounded_lg()
                .bg(if perf_mode {
                    hsla(15.0 / 360.0, 0.75, 0.28, 1.0)
                } else {
                    hsla(220.0 / 360.0, 0.25, 0.18, 1.0)
                })
                .border_1()
                .border_color(if perf_mode {
                    hsla(25.0 / 360.0, 0.90, 0.55, 1.0)
                } else {
                    hsla(220.0 / 360.0, 0.25, 0.28, 1.0)
                })
                .hover(|s| s.opacity(0.85))
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_1p5()
                .text_size(px(11.5))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_perf.update(cx, |st, cx| {
                        st.toggle_performance_mode();
                        cx.notify();
                    });
                })
                .child(if perf_mode {
                    format!("🔥 {}", state.t("toolbar.perf_active"))
                } else {
                    format!("⚡ {}", state.t("toolbar.perf_mode"))
                }),
        )
        // 4. 切刀工具切换 (Knife Mode Toggle)
        .child(
            div()
                .px_3()
                .py_1p5()
                .rounded_lg()
                .bg(if knife_mode {
                    hsla(0.0, 0.7, 0.35, 1.0)
                } else {
                    hsla(220.0 / 360.0, 0.25, 0.18, 1.0)
                })
                .border_1()
                .border_color(if knife_mode {
                    hsla(0.0, 0.85, 0.60, 1.0)
                } else {
                    hsla(220.0 / 360.0, 0.25, 0.28, 1.0)
                })
                .hover(|s| s.opacity(0.85))
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_1p5()
                .text_size(px(11.5))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_knife.update(cx, |st, cx| {
                        st.knife_mode = !st.knife_mode;
                        cx.notify();
                    });
                })
                .child(if knife_mode {
                    format!("✂️ {}", state.t("toolbar.knife_active"))
                } else {
                    format!("✂️ {}", state.t("toolbar.knife_mode"))
                }),
        )
        // 5. 自适应视口居中 (Fit View)
        .child(
            div()
                .px_3()
                .py_1p5()
                .rounded_lg()
                .bg(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
                .border_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.28, 1.0))
                .hover(|s| s.bg(hsla(220.0 / 360.0, 0.35, 0.25, 1.0)))
                .cursor_pointer()
                .text_size(px(11.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0xffffff))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_fit.update(cx, |st, cx| {
                        st.fit_view();
                        cx.notify();
                    });
                })
                .child(format!("🔍 {}", state.t("toolbar.fit_view"))),
        )
}
