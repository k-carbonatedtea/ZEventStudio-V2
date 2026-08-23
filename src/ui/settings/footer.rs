use gpui::*;
use crate::ui::state::AppState;

/// 渲染底部操作栏 (恢复默认 + 取消 + 保存并应用 + 拖拽调宽手柄)
pub fn render_settings_footer(
    state: &AppState,
    model_reset: &Entity<AppState>,
    model_cancel: &Entity<AppState>,
    model_save: &Entity<AppState>,
    model_resize: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
    is_maximized: bool,
) -> impl IntoElement {
    let m_reset = model_reset.clone();
    let m_cancel = model_cancel.clone();
    let m_save = model_save.clone();
    let m_resize = model_resize.clone();

    div()
        .h(px(52.0))
        .px_5()
        .bg(pal.app_bg)
        .border_t_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .relative()
        // 恢复默认按钮
        .child(
            div()
                .h(px(30.0))
                .px_3()
                .rounded_md()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.card_border)
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_size(px(11.5))
                .text_color(pal.text_secondary)
                .hover(|s| {
                    s.bg(pal.card_hover)
                        .text_color(pal.text_primary)
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    m_reset.update(cx, |st, cx| {
                        st.reset_settings_to_default();
                        cx.notify();
                    });
                })
                .child(state.t("settings.reset_defaults")),
        )
        // 右侧操作按钮组 (取消 + 保存并应用)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                // 取消按钮
                .child(
                    div()
                        .h(px(30.0))
                        .px_3p5()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.card_border)
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .text_size(px(12.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| {
                            s.bg(pal.card_hover)
                                .text_color(pal.text_primary)
                        })
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            m_cancel.update(cx, |st, cx| {
                                st.close_settings();
                                cx.notify();
                            });
                        })
                        .child(state.t("settings.cancel_btn")),
                )
                // 保存并应用按钮
                .child(
                    div()
                        .h(px(30.0))
                        .px_4()
                        .rounded_md()
                        .bg(pal.primary_accent)
                        .border_1()
                        .border_color(pal.primary_accent_hover)
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_1p5()
                        .cursor_pointer()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if pal.is_dark { rgb(0x0A0C10) } else { rgb(0xFFFFFF) })
                        .hover(|s| s.bg(pal.primary_accent_hover).shadow_md())
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            m_save.update(cx, |st, cx| {
                                st.save_and_apply_settings();
                                cx.notify();
                            });
                        })
                        .child(div().text_size(px(12.0)).child("💾"))
                        .child(state.t("settings.save_btn")),
                ),
        )
        // 右下角弹窗缩放手柄 (非最大化状态下可自由拖拽调节窗口宽高)
        .child(
            if !is_maximized {
                div()
                    .absolute()
                    .bottom_0()
                    .right_0()
                    .size(px(20.0))
                    .flex()
                    .items_end()
                    .justify_end()
                    .p_1()
                    .cursor_nwse_resize()
                    .hover(|s| s.bg(pal.primary_accent.opacity(0.12)))
                    .rounded_br_xl()
                    .on_mouse_down(MouseButton::Left, move |e, _window, cx| {
                        cx.stop_propagation();
                        let mx: f32 = e.position.x.into();
                        let my: f32 = e.position.y.into();
                        m_resize.update(cx, |st, cx| {
                            st.is_resizing_settings = true;
                            st.settings_resize_start = Some(((mx, my), (st.settings_modal_width, st.settings_modal_height)));
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(pal.primary_accent.opacity(0.75))
                            .child("◢"),
                    )
                    .into_any_element()
            } else {
                div().into_any_element()
            },
        )
}
