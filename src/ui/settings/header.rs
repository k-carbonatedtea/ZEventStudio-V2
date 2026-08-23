use gpui::*;
use crate::ui::state::AppState;

/// 渲染设置窗口标题栏
pub fn render_settings_header(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
    is_maximized: bool,
) -> impl IntoElement {
    let m_max = model.clone();
    let m_close = model.clone();

    div()
        .h(px(52.0))
        .px_5()
        .bg(pal.app_bg)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .items_center()
                .gap_2p5()
                .child(div().flex_shrink_0().text_size(px(17.0)).child("⚙️"))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.5))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.title")),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.subtitle")),
                        ),
                ),
        )
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap_1p5()
                // 最大化 / 还原按钮
                .child(
                    div()
                        .size(px(26.0))
                        .rounded_md()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .text_size(px(12.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_surface).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            m_max.update(cx, |st, cx| {
                                st.settings_modal_maximized = !st.settings_modal_maximized;
                                cx.notify();
                            });
                        })
                        .child(if is_maximized { "🗗" } else { "🗖" }),
                )
                // 关闭按钮
                .child(
                    div()
                        .size(px(26.0))
                        .rounded_md()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .text_size(px(13.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_surface).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            m_close.update(cx, |st, cx| {
                                st.close_settings();
                                cx.notify();
                            });
                        })
                        .child("✕"),
                ),
        )
}
