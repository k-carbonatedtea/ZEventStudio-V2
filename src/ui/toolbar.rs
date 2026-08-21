use gpui::*;
use crate::ui::state::AppState;

pub fn render_toolbar(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let zoom_pct = (state.zoom * 100.0).round() as i32;
    let knife_mode = state.knife_mode;
    let particles_enabled = state.particles_enabled;
    let fps = state.fps;

    let model_layout = model.clone();
    let model_knife = model.clone();
    let model_particles = model.clone();
    let model_fit = model.clone();
    let model_zoom_in = model.clone();
    let model_zoom_out = model.clone();
    let model_zoom_reset = model.clone();

    // Dynamic FPS Indicator Color
    let (fps_bg, fps_border, fps_dot) = if fps >= 50.0 {
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
                        .child(format!("{} ({} 节点 / {} 连线)", state.graph.name, state.graph.nodes.len(), state.graph.edges.len())),
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
                        .child("✨ 自动排版"),
                )
                // Particles Toggle
                .child(
                    div()
                        .px_3()
                        .py_1p5()
                        .rounded_lg()
                        .bg(if particles_enabled {
                            hsla(145.0 / 360.0, 0.45, 0.22, 1.0)
                        } else {
                            hsla(220.0 / 360.0, 0.25, 0.18, 1.0)
                        })
                        .border_1()
                        .border_color(if particles_enabled {
                            hsla(145.0 / 360.0, 0.65, 0.45, 1.0)
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
                            model_particles.update(cx, |st, cx| {
                                st.particles_enabled = !st.particles_enabled;
                                cx.notify();
                            });
                        })
                        .child(if particles_enabled {
                            "🌊 粒子: 开启"
                        } else {
                            "🌊 粒子: 关闭"
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
                            "✂️ 切刀模式: 开启"
                        } else {
                            "✂️ 切刀剪线"
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
                        .child("🔍 居中自适应"),
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
        // Right: Realtime Status & FPS Indicator Badge
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                // Node & Edge Counts Badge
                .child(
                    div()
                        .px_2p5()
                        .py_1()
                        .rounded_lg()
                        .bg(hsla(220.0 / 360.0, 0.25, 0.16, 1.0))
                        .border_1()
                        .border_color(hsla(220.0 / 360.0, 0.25, 0.28, 1.0))
                        .text_size(px(11.0))
                        .text_color(hsla(215.0 / 360.0, 0.25, 0.75, 1.0))
                        .child(format!(
                            "{} 节点 · {} 连线",
                            state.graph.nodes.len(),
                            state.graph.edges.len()
                        )),
                )
                // Real-time FPS Capsule Badge
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
                                .child(format!("{:.0} FPS", fps)),
                        ),
                ),
        )
}
