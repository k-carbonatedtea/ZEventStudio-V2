use gpui::*;
use gpui_component::input::{Editor, EditorState};
use crate::ui::state::AppState;
use crate::ui::views::msbt_editor::types::MsbtEntry;

/// 渲染对白文本交互输入与编辑卡片 (采用与 MSYT 源码编辑器完全一致的 EditorState 核心引擎)
pub fn render_msbt_bubble_input(
    entry: Option<&MsbtEntry>,
    state: &AppState,
    model: &Entity<AppState>,
    bubble_editor: Option<&Entity<EditorState>>,
) -> impl IntoElement {
    let pal = state.theme();

    let entry_key = entry.map(|e| e.key.clone()).unwrap_or_default();

    let model_click = model.clone();
    let model_copy = model.clone();
    let model_del = model.clone();
    let entry_key_copy = entry_key.clone();
    let entry_key_del = entry_key.clone();

    let ed_focus = bubble_editor.cloned();

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
                                        let msg = st.t("msbt_editor.toast_key_copied");
                                        st.toast = Some(crate::ui::components::Toast::success(msg));
                                        cx.notify();
                                    });
                                })
                                .child(state.t("msbt_editor.copy_key_btn"))
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
                        .child(state.t("msbt_editor.delete_entry_btn"))
                })),
        )
        // 2. 交互式多行输入框 (原生 EditorState：支持全字符精细光标、鼠标拖拽选区、双击选词、原生 IME 与快捷键)
        .child(
            div()
                .id("msbt_text_input_container")
                .w_full()
                .min_h(px(84.0))
                .p_2p5()
                .rounded_lg()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.card_border)
                .cursor_text()
                .capture_any_mouse_down(move |_e, window, cx| {
                    model_click.update(cx, |st, cx| {
                        if st.has_active_text_input() {
                            st.clear_active_text_inputs();
                            cx.notify();
                        }
                    });
                    if let Some(ref ed) = ed_focus {
                        ed.update(cx, |editor, cx| {
                            editor.focus(window, cx);
                        });
                    }
                })
                .child(if entry.is_none() {
                    div()
                        .text_size(px(12.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("msbt_editor.select_entry_hint"))
                        .into_any_element()
                } else if let Some(ed) = bubble_editor {
                    Editor::new(ed)
                        .bordered(false)
                        .appearance(false)
                        .w_full()
                        .min_h(px(80.0))
                        .into_any_element()
                } else {
                    div()
                        .text_size(px(12.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("msbt_editor.input_placeholder"))
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
