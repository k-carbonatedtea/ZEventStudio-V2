use gpui::*;
use crate::ui::state::AppState;

/// 渲染缩放控制器胶囊组件 (缩小、缩放比例重置、放大)
pub fn render_zoom_group(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let zoom_pct = (state.zoom * 100.0).round() as i32;

    let model_zoom_in = model.clone();
    let model_zoom_out = model.clone();
    let model_zoom_reset = model.clone();

    div()
        .flex()
        .items_center()
        .rounded_lg()
        .bg(hsla(220.0 / 360.0, 0.25, 0.15, 1.0))
        .border_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.28, 1.0))
        .p_0p5()
        // 缩小按钮
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
        // 缩放百分比数值 (点击快速重置至 100%)
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
        // 放大按钮
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
        )
}
