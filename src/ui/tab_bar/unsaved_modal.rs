use gpui::*;
use crate::ui::state::AppState;

/// 渲染未保存修改关闭确认对话框 (Unsaved Changes Confirmation Modal)
pub fn render_unsaved_changes_modal(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let modal = match &state.unsaved_changes_modal {
        Some(m) => m.clone(),
        None => return div().into_any_element(),
    };

    let pal = state.theme();
    let model_backdrop_cancel = model.clone();
    let model_close_btn = model.clone();
    let model_btn_cancel = model.clone();
    let model_save = model.clone();
    let model_discard = model.clone();

    let title_text = state.t("modal.unsaved_title");
    let message_text = state.t("modal.unsaved_message");
    let hint_text = state.t("modal.unsaved_hint");
    let save_text = state.t("modal.unsaved_save");
    let dont_save_text = state.t("modal.unsaved_dont_save");
    let cancel_text = state.t("modal.unsaved_cancel");

    div()
        .id("unsaved_changes_modal_overlay")
        .absolute()
        .inset_0()
        .bg(hsla(0.0, 0.0, 0.0, 0.68))
        .flex()
        .items_center()
        .justify_center()
        .on_scroll_wheel(|_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_backdrop_cancel.update(cx, |st, cx| {
                st.cancel_unsaved_changes_modal();
                cx.notify();
            });
        })
        .child(
            div()
                .id("unsaved_changes_modal_card")
                .w(px(460.0))
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.panel_border)
                .rounded_xl()
                .shadow_2xl()
                .flex()
                .flex_col()
                .overflow_hidden()
                .on_scroll_wheel(|_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                // 1. 顶部标题栏
                .child(
                    div()
                        .h(px(44.0))
                        .px_4()
                        .bg(pal.app_bg)
                        .border_b_1()
                        .border_color(pal.panel_border)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(15.0))
                                        .text_color(pal.secondary_accent)
                                        .child("⚠️"),
                                )
                                .child(
                                    div()
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(13.0))
                                        .text_color(pal.text_primary)
                                        .child(title_text),
                                ),
                        )
                        .child(
                            div()
                                .size(px(24.0))
                                .rounded_md()
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| {
                                    s.bg(pal.card_surface)
                                        .text_color(pal.text_primary)
                                })
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_close_btn.update(cx, |st, cx| {
                                        st.cancel_unsaved_changes_modal();
                                        cx.notify();
                                    });
                                })
                                .child("✕"),
                        ),
                )
                // 2. 中间内容区
                .child(
                    div()
                        .p_5()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(message_text),
                        )
                        // 文件卡片信息框
                        .child(
                            div()
                                .p_3()
                                .rounded_lg()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.card_border)
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_size(px(12.0))
                                                .text_color(pal.primary_accent)
                                                .child("📄"),
                                        )
                                        .child(
                                            div()
                                                .font_weight(FontWeight::BOLD)
                                                .text_size(px(12.5))
                                                .text_color(pal.text_primary)
                                                .child(modal.title.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .text_color(pal.text_secondary)
                                        .child(modal.file_path.clone()),
                                ),
                        )
                        // 警示提示
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(pal.text_secondary)
                                .child(hint_text),
                        ),
                )
                // 3. 底部操作按钮栏 (取消 / 不保存 / 保存)
                .child(
                    div()
                        .h(px(52.0))
                        .px_4()
                        .bg(pal.app_bg)
                        .border_t_1()
                        .border_color(pal.panel_border)
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap_2p5()
                        // 取消按钮 (Cancel) - 次要暗色中性边框卡片胶囊
                        .child(
                            div()
                                .id("btn_unsaved_cancel")
                                .h(px(28.0))
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
                                        .border_color(pal.text_secondary)
                                })
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_btn_cancel.update(cx, |st, cx| {
                                        st.cancel_unsaved_changes_modal();
                                        cx.notify();
                                    });
                                })
                                .child(cancel_text),
                        )
                        // 不保存按钮 (Don't Save) - 警示红暗底微透胶囊
                        .child(
                            div()
                                .id("btn_unsaved_discard")
                                .h(px(28.0))
                                .px_3p5()
                                .rounded_md()
                                .bg(hsla(0.0, 0.60, 0.22, 0.45))
                                .border_1()
                                .border_color(hsla(0.0, 0.60, 0.45, 0.55))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .text_color(hsla(0.0, 0.85, 0.75, 1.0))
                                .hover(|s| {
                                    s.bg(hsla(0.0, 0.70, 0.35, 0.85))
                                        .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                                        .border_color(hsla(0.0, 0.80, 0.60, 0.90))
                                })
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_discard.update(cx, |st, cx| {
                                        let should_quit = st.confirm_discard_and_close_tab();
                                        if should_quit {
                                            cx.quit();
                                        } else {
                                            cx.notify();
                                        }
                                    });
                                })
                                .child(dont_save_text),
                        )
                        // 保存按钮 (Save) - 主题主色高亮强调实体胶囊
                        .child(
                            div()
                                .id("btn_unsaved_save")
                                .h(px(28.0))
                                .px_4()
                                .rounded_md()
                                .bg(pal.primary_accent)
                                .border_1()
                                .border_color(pal.primary_accent_hover)
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if pal.is_dark { rgb(0x0A0C10) } else { rgb(0xFFFFFF) })
                                .hover(|s| {
                                    s.bg(pal.primary_accent_hover)
                                        .shadow_md()
                                })
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_save.update(cx, |st, cx| {
                                        let should_quit = st.confirm_save_and_close_tab();
                                        if should_quit {
                                            cx.quit();
                                        } else {
                                            cx.notify();
                                        }
                                    });
                                })
                                .child(save_text),
                        ),
                ),
        )
        .into_any_element()
}
