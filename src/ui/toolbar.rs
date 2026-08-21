use gpui::*;
use crate::ui::state::AppState;

pub fn render_toolbar(
    state: &AppState,
    fps: f32,
    frame_time: f32,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let zoom_pct = (state.zoom * 100.0).round() as i32;
    let knife_mode = state.knife_mode;
    let expand_all = state.expand_all_params;
    let perf_mode = state.performance_mode;

    let model_layout = model.clone();
    let model_expand = model.clone();
    let model_knife = model.clone();
    let model_perf = model.clone();
    let model_fps_toggle = model.clone();
    let model_fit = model.clone();
    let model_zoom_in = model.clone();
    let model_zoom_out = model.clone();
    let model_zoom_reset = model.clone();
    let model_lang = model.clone();

    // Dynamic FPS Indicator Color
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
        // Left: Logo & Title
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(18.0))
                                .child("⚡"),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::EXTRA_BOLD)
                                .text_size(px(15.0))
                                .text_color(rgb(0xffffff))
                                .child("NodeFlow Studio"),
                        ),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(hsla(215.0 / 360.0, 0.35, 0.20, 1.0))
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hsla(215.0 / 360.0, 0.5, 0.85, 1.0))
                        .child("EVFL Studio"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(hsla(145.0 / 360.0, 0.35, 0.18, 0.95))
                        .border_1()
                        .border_color(hsla(145.0 / 360.0, 0.6, 0.40, 1.0))
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hsla(145.0 / 360.0, 0.8, 0.90, 1.0))
                        .child(format!("{} ({} Nodes / {} Edges)", state.graph.name, state.graph.nodes.len(), state.graph.edges.len())),
                ),
        )
        // Center: Interactive Toolbar Controls
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                // Auto Layout Button
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
                // Expand / Collapse Parameters Toggle Button
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
                // Performance Mode (GPU Extreme Performance Toggle)
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
                // Knife Mode Toggle
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
                // Fit View Button
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
                // Zoom controls group
                .child(
                    div()
                        .flex()
                        .items_center()
                        .rounded_lg()
                        .bg(hsla(220.0 / 360.0, 0.25, 0.15, 1.0))
                        .border_1()
                        .border_color(hsla(220.0 / 360.0, 0.25, 0.28, 1.0))
                        .p_0p5()
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .hover(|s| s.bg(hsla(220.0 / 360.0, 0.35, 0.25, 1.0)))
                                .cursor_pointer()
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xffffff))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_zoom_out.update(cx, |st, cx| {
                                        st.zoom = (st.zoom * 0.85).clamp(0.25, 2.5);
                                        cx.notify();
                                    });
                                    let _ = &model_zoom_out;
                                })
                                .child("－"),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hsla(45.0 / 360.0, 0.85, 0.75, 1.0))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_zoom_reset.update(cx, |st, cx| {
                                        st.zoom = 1.0;
                                        cx.notify();
                                    });
                                })
                                .child(format!("{}%", zoom_pct)),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .hover(|s| s.bg(hsla(220.0 / 360.0, 0.35, 0.25, 1.0)))
                                .cursor_pointer()
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xffffff))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_zoom_in.update(cx, |st, cx| {
                                        st.zoom = (st.zoom * 1.18).clamp(0.25, 2.5);
                                        cx.notify();
                                    });
                                })
                                .child("＋"),
                        ),
                ),
        )
        // Right: Realtime Status & FPS Indicator Badge + Language Switch
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                // Language Switch Button
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
                // Real-time FPS Capsule Badge (Clickable toggle for Performance Mode)
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
                ),
        )
}
