use gpui::*;
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::AppState;
use crate::ui::state_types::{DeleteConfirmModalState, RenameModalState};

/// 渲染文件/封包条目重命名模态对话框
pub fn render_rename_modal(
    state: &AppState,
    modal: &RenameModalState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let is_focused = modal.focused;
    let new_name = modal.new_name.clone();
    let old_name = modal.old_name.clone();
    let is_dir = modal.is_dir;

    let model_close = model.clone();
    let model_click = model.clone();
    let model_out = model.clone();
    let model_clear = model.clone();
    let model_confirm = model.clone();
    let model_cancel = model.clone();
    let model_close_btn = model.clone();

    div()
        .id("rename_modal_overlay")
        .absolute()
        .inset_0()
        .bg(rgba(0x00000077))
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_close.update(cx, |st, cx| {
                st.rename_modal = None;
                cx.notify();
            });
        })
        .child(
            div()
                .id("rename_modal_card")
                .w(px(380.0))
                .rounded_2xl()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.panel_border)
                .shadow_2xl()
                .p_5()
                .flex()
                .flex_col()
                .gap_4()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                // 1. 标题栏
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().text_size(px(16.0)).child("✏️"))
                                .child(
                                    div()
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(pal.text_primary)
                                        .child(if is_dir {
                                            state.t("modal.rename_folder_title")
                                        } else {
                                            state.t("modal.rename_title")
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .cursor_pointer()
                                .text_color(pal.text_secondary)
                                .hover(|s| s.text_color(pal.text_primary))
                                .child("✕")
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_close_btn.update(cx, |st, cx| {
                                        st.rename_modal = None;
                                        cx.notify();
                                    });
                                }),
                        ),
                )
                // 2. 原文件名提示
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .child(format!("{}: {}", state.t("modal.current_name"), old_name)),
                )
                // 3. 规范文本输入框 (包含 focus、out失焦、光标、一键清空)
                .child(
                    div()
                        .id("rename_input_container")
                        .cursor_text()
                        .h(px(34.0))
                        .px_3()
                        .rounded_lg()
                        .bg(pal.app_bg)
                        .border_1()
                        .border_color(if is_focused { pal.primary_accent } else { pal.panel_border })
                        .flex()
                        .items_center()
                        .justify_between()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_click.update(cx, |st, cx| {
                                st.clear_active_text_inputs();
                                if let Some(ref mut m) = st.rename_modal {
                                    m.focused = true;
                                }
                                cx.notify();
                            });
                        })
                        .on_mouse_down_out(move |_, _, cx| {
                            model_out.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.rename_modal {
                                    if m.focused {
                                        m.focused = false;
                                        cx.notify();
                                    }
                                }
                            });
                        })
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .child(
                                    if new_name.is_empty() && !is_focused {
                                        div()
                                            .text_size(px(12.0))
                                            .text_color(pal.text_secondary)
                                            .child(state.t("modal.rename_placeholder"))
                                            .into_any_element()
                                    } else {
                                        div()
                                            .flex()
                                            .items_center()
                                            .text_size(px(12.0))
                                            .text_color(pal.text_primary)
                                            .child(new_name.clone())
                                            .children(is_focused.then(|| {
                                                render_blinking_cursor(pal.primary_accent.into()).into_any_element()
                                            }))
                                            .into_any_element()
                                    }
                                )
                        )
                        .children((!new_name.is_empty()).then(|| {
                            div()
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.text_color(pal.text_primary))
                                .child("✕")
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_clear.update(cx, |st, cx| {
                                        if let Some(ref mut m) = st.rename_modal {
                                            m.new_name.clear();
                                            cx.notify();
                                        }
                                    });
                                })
                        }))
                )
                // 4. 底部操作按钮
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap_2()
                        .pt_1()
                        // 取消按钮
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded_lg()
                                .bg(pal.card_surface)
                                .hover(|s| s.bg(pal.card_hover))
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .text_color(pal.text_primary)
                                .child(state.t("modal.cancel"))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_cancel.update(cx, |st, cx| {
                                        st.rename_modal = None;
                                        cx.notify();
                                    });
                                }),
                        )
                        // 确认按钮
                        .child(
                            div()
                                .px_4()
                                .py_1p5()
                                .rounded_lg()
                                .bg(pal.primary_accent)
                                .hover(|s| s.opacity(0.88))
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xffffff))
                                .child(state.t("modal.confirm"))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_confirm.update(cx, |st, cx| {
                                        st.execute_rename_item();
                                        cx.notify();
                                    });
                                }),
                        ),
                ),
        )
}

/// 渲染文件/封包条目删除确认模态对话框
pub fn render_delete_confirm_modal(
    state: &AppState,
    modal: &DeleteConfirmModalState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let name = modal.name.clone();
    let is_dir = modal.is_dir;
    let is_inside_sarc = modal.is_inside_sarc;

    let model_close = model.clone();
    let model_cancel = model.clone();
    let model_confirm = model.clone();

    div()
        .id("delete_modal_overlay")
        .absolute()
        .inset_0()
        .bg(rgba(0x00000077))
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_close.update(cx, |st, cx| {
                st.delete_confirm_modal = None;
                cx.notify();
            });
        })
        .child(
            div()
                .id("delete_modal_card")
                .w(px(380.0))
                .rounded_2xl()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.panel_border)
                .shadow_2xl()
                .p_5()
                .flex()
                .flex_col()
                .gap_4()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                // 1. 标题
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_size(px(18.0)).child("⚠️"))
                        .child(
                            div()
                                .text_size(px(14.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xEF4444))
                                .child(state.t("modal.delete_title")),
                        ),
                )
                // 2. 警告描述
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(pal.text_secondary)
                                .child(if is_inside_sarc {
                                    state.t("modal.delete_sarc_confirm_msg")
                                } else if is_dir {
                                    state.t("modal.delete_folder_confirm_msg")
                                } else {
                                    state.t("modal.delete_file_confirm_msg")
                                }),
                        )
                        .child(
                            div()
                                .p_2()
                                .rounded_lg()
                                .bg(pal.app_bg)
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xEF4444))
                                .overflow_hidden()
                                .child(name),
                        ),
                )
                // 3. 底部按钮 (取消 / 危险确认)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap_2()
                        .pt_2()
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded_lg()
                                .bg(pal.card_surface)
                                .hover(|s| s.bg(pal.card_hover))
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .text_color(pal.text_primary)
                                .child(state.t("modal.cancel"))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_cancel.update(cx, |st, cx| {
                                        st.delete_confirm_modal = None;
                                        cx.notify();
                                    });
                                }),
                        )
                        .child(
                            div()
                                .px_4()
                                .py_1p5()
                                .rounded_lg()
                                .bg(rgb(0xEF4444))
                                .hover(|s| s.opacity(0.88))
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xffffff))
                                .child(state.t("modal.delete_confirm_btn"))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_confirm.update(cx, |st, cx| {
                                        st.execute_delete_item();
                                        cx.notify();
                                    });
                                }),
                        ),
                ),
        )
}
