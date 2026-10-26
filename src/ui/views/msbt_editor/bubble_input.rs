use gpui::*;
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::AppState;
use crate::ui::views::msbt_editor::types::MsbtEntry;

/// 渲染对白文本交互输入与编辑卡片 (严格遵循文本输入规范 8 与全局 IME 桥接)
pub fn render_msbt_bubble_input(
    entry: Option<&MsbtEntry>,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let is_focused = state.msbt_text_editor_focused;

    let entry_key = entry.map(|e| e.key.clone()).unwrap_or_default();
    let raw_text = entry.map(|e| e.raw_text.clone()).unwrap_or_default();

    let is_selected_entry = state.msbt_selected_entry_key.as_deref() == Some(&entry_key);
    let current_text = if is_selected_entry && !state.msbt_text_editor_draft.is_empty() {
        state.msbt_text_editor_draft.clone()
    } else {
        raw_text.clone()
    };

    let model_click = model.clone();
    let model_out = model.clone();
    let model_copy = model.clone();
    let model_del = model.clone();
    let entry_key_copy = entry_key.clone();
    let entry_key_del = entry_key.clone();
    let entry_key_click = entry_key.clone();
    let raw_text_click = raw_text.clone();

    let has_entry = entry.is_some();

    div()
        .w_full()
        .px_4()
        .pb_4()
        .flex()
        .flex_col()
        .gap_2()
        // 1. 卡片顶栏：当前条目键名与快捷复制/删除
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
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.0))
                                .text_color(pal.text_primary)
                                .child(if entry_key.is_empty() {
                                    state.t("msbt_editor.no_active_entry").to_string()
                                } else {
                                    format!("💬 {}: {}", state.t("msbt_editor.entry_label"), entry_key)
                                }),
                        )
                        .children((!entry_key.is_empty()).then(|| {
                            div()
                                .cursor_pointer()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.card_border)
                                .text_size(px(10.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.text_color(pal.primary_accent))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    let k = entry_key_copy.clone();
                                    cx.write_to_clipboard(ClipboardItem::new_string(k));
                                    model_copy.update(cx, |st, cx| {
                                        st.toast = Some(crate::ui::components::Toast::success("已复制条目键名"));
                                        cx.notify();
                                    });
                                })
                                .child("📋 复制键名")
                        })),
                )
                // 删除当前条目
                .children((!entry_key.is_empty()).then(|| {
                    div()
                        .cursor_pointer()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(rgb(0xEF4444))
                        .text_size(px(10.5))
                        .text_color(rgb(0xEF4444))
                        .hover(|s| s.bg(rgb(0xEF4444)).text_color(rgb(0xFFFFFF)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let k = entry_key_del.clone();
                            model_del.update(cx, |st, cx| {
                                st.delete_msbt_entry(&k);
                                cx.notify();
                            });
                        })
                        .child("🗑️ 删除条目")
                })),
        )
        // 2. 交互式多行输入框 (严格按照规范 8 实现 IME 与失焦)
        .child(
            div()
                .id("msbt_text_input_container")
                .w_full()
                .min_h(px(84.0))
                .p_3()
                .rounded_lg()
                .bg(pal.card_surface)
                .border_1()
                .border_color(if is_focused {
                    pal.primary_accent
                } else {
                    pal.card_border
                })
                .cursor_text()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    if has_entry {
                        let k = entry_key_click.clone();
                        let raw = raw_text_click.clone();
                        model_click.update(cx, |st, cx| {
                            st.clear_active_text_inputs();
                            if st.msbt_selected_entry_key.as_deref() != Some(&k) || st.msbt_text_editor_draft.is_empty() {
                                st.msbt_selected_entry_key = Some(k);
                                st.msbt_text_editor_draft = raw;
                            }
                            st.msbt_text_editor_focused = true;
                            cx.notify();
                        });
                    }
                })
                .on_mouse_down_out(move |_, _, cx| {
                    model_out.update(cx, |st, cx| {
                        if st.msbt_text_editor_focused {
                            st.msbt_text_editor_focused = false;
                            cx.notify();
                        }
                    });
                })
                .child(if entry.is_none() {
                    div()
                        .text_size(px(12.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("msbt_editor.select_entry_hint"))
                        .into_any_element()
                } else if current_text.is_empty() && !is_focused {
                    div()
                        .text_size(px(12.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("msbt_editor.input_placeholder"))
                        .into_any_element()
                } else if current_text.is_empty() && is_focused {
                    div()
                        .flex()
                        .items_center()
                        .child(render_blinking_cursor(pal.primary_accent.into()))
                        .into_any_element()
                } else {
                    let lines: Vec<String> = current_text.split('\n').map(|s| s.to_string()).collect();
                    let total_lines = lines.len();
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .children(lines.into_iter().enumerate().map(|(idx, line)| {
                            let is_last = idx + 1 == total_lines;
                            div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .text_size(px(13.0))
                                .line_height(relative(1.4))
                                .text_color(pal.text_primary)
                                .child(if line.is_empty() { " ".to_string() } else { line })
                                .children((is_focused && is_last).then(|| {
                                    render_blinking_cursor(pal.primary_accent.into()).into_any_element()
                                }))
                                .into_any_element()
                        }))
                        .into_any_element()
                }),
        )
        // 3. 底部快捷提示
        .child(
            div()
                .text_size(px(11.0))
                .text_color(pal.text_secondary)
                .child(state.t("msbt_editor.input_help_tip")),
        )
}
