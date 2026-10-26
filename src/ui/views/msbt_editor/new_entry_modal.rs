use gpui::*;
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::AppState;

/// 渲染新建 MSBT 消息条目模态弹窗
pub fn render_new_entry_modal(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let is_focused = state.msbt_new_entry_focused;
    let name_query = state.msbt_new_entry_name.clone();

    let model_close = model.clone();
    let model_cancel = model.clone();
    let model_confirm = model.clone();
    let model_input_click = model.clone();
    let model_input_out = model.clone();

    div()
        .absolute()
        .inset_0()
        .bg(rgba(0x00000088))
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_close.update(cx, |st, cx| {
                st.msbt_new_entry_modal_open = false;
                st.msbt_new_entry_focused = false;
                cx.notify();
            });
        })
        .child(
            div()
                .w(px(380.0))
                .rounded_xl()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.panel_border)
                .shadow_lg()
                .p_5()
                .flex()
                .flex_col()
                .gap_4()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                // 标题栏
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(14.0))
                                .text_color(pal.text_primary)
                                .child(state.t("msbt_editor.new_entry_title")),
                        ),
                )
                // 输入框容器
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(pal.text_secondary)
                                .child(state.t("msbt_editor.entry_key_label")),
                        )
                        .child(
                            div()
                                .id("msbt_new_entry_input_container")
                                .h(px(32.0))
                                .px_3()
                                .rounded_md()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(if is_focused {
                                    pal.primary_accent
                                } else {
                                    pal.card_border
                                })
                                .flex()
                                .items_center()
                                .cursor_text()
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_input_click.update(cx, |st, cx| {
                                        st.clear_active_text_inputs();
                                        st.msbt_new_entry_focused = true;
                                        cx.notify();
                                    });
                                })
                                .on_mouse_down_out(move |_, _, cx| {
                                    model_input_out.update(cx, |st, cx| {
                                        if st.msbt_new_entry_focused {
                                            st.msbt_new_entry_focused = false;
                                            cx.notify();
                                        }
                                    });
                                })
                                .child(if name_query.is_empty() && !is_focused {
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(pal.text_secondary)
                                        .child("如: Npc_Zora001_Talk_00")
                                        .into_any_element()
                                } else {
                                    div()
                                        .flex()
                                        .items_center()
                                        .text_size(px(12.0))
                                        .text_color(pal.text_primary)
                                        .child(name_query.clone())
                                        .children(is_focused.then(|| {
                                            render_blinking_cursor(pal.primary_accent.into())
                                                .into_any_element()
                                        }))
                                        .into_any_element()
                                }),
                        ),
                )
                // 按钮组
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap_2()
                        // 取消按钮
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded_md()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.card_border)
                                .cursor_pointer()
                                .text_size(px(11.5))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_cancel.update(cx, |st, cx| {
                                        st.msbt_new_entry_modal_open = false;
                                        st.msbt_new_entry_focused = false;
                                        cx.notify();
                                    });
                                })
                                .child(state.t("actors_view.actors_cancel")),
                        )
                        // 确认新建按钮
                        .child(
                            div()
                                .px_4()
                                .py_1p5()
                                .rounded_md()
                                .bg(pal.primary_accent)
                                .cursor_pointer()
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                                .hover(|s| s.opacity(0.85))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_confirm.update(cx, |st, cx| {
                                        st.confirm_create_msbt_entry();
                                        cx.notify();
                                    });
                                })
                                .child(state.t("actors_view.actors_confirm")),
                        ),
                ),
        )
}
