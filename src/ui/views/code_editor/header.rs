use gpui::*;
use gpui_component::input::EditorState;
use crate::model::code_document::CodeDocumentState;
use crate::ui::state::AppState;

/// 渲染代码编辑器顶部状态栏 (包含格式徽章、大小端模式、路径、修改标记与快捷操作)
pub fn render_code_editor_header(
    code_state: &CodeDocumentState,
    is_modified: bool,
    state: &AppState,
    model: &Entity<AppState>,
    editor_state: Option<&Entity<EditorState>>,
) -> impl IntoElement {
    let pal = state.theme();
    let line_count = code_state.content.lines().count();
    let char_count = code_state.content.len();
    let format_name = code_state.format.display_name();
    let format_icon = code_state.format.icon();
    let endian_label = code_state.endian.display_label();

    let model_save = model.clone();
    let model_revert = model.clone();
    let ed_revert = editor_state.cloned();
    let original_content = code_state.original_content.clone();

    let save_btn_fg = if is_modified {
        if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) }
    } else {
        pal.text_primary
    };

    div()
        .h(px(40.0))
        .w_full()
        .px_4()
        .bg(pal.panel_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        // 左侧：格式徽章与大小端指示
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .overflow_hidden()
                // 格式徽章
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.primary_accent)
                        .child(div().text_size(px(12.0)).child(format_icon))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.0))
                                .text_color(pal.primary_accent)
                                .child(format_name),
                        ),
                )
                // 大小端徽章
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.secondary_accent)
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.secondary_accent)
                        .child(endian_label),
                )
                // 未保存修改指示
                .children(is_modified.then(|| {
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.secondary_accent)
                        .child(
                            div()
                                .size(px(6.0))
                                .rounded_full()
                                .bg(pal.secondary_accent),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.secondary_accent)
                                .child(state.t("code_editor.modified_label")),
                        )
                })),
        )
        // 右侧：统计与操作按钮 (保存 / 重置 / 复制)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                // 行与字符统计
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .child(format!("{} {}, {} {}", line_count, state.t("code_editor.lines"), char_count, state.t("code_editor.chars"))),
                )
                // 重置修改按钮 (仅在已修改时高亮可用)
                .children(is_modified.then(|| {
                    div()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(rgb(0xEF4444))
                        .hover(|s| s.bg(rgb(0xEF4444)).text_color(rgb(0xFFFFFF)))
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xEF4444))
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            if let Some(ref ed) = ed_revert {
                                let orig = original_content.clone();
                                ed.update(cx, |ed_state, cx| {
                                    ed_state.set_value(orig, window, cx);
                                });
                            }
                            model_revert.update(cx, |st, cx| {
                                st.revert_active_code();
                                cx.notify();
                            });
                        })
                        .child(format!("🔄 {}", state.t("code_editor.revert")))
                }))
                // 💾 保存按钮 (Ctrl+S)
                .child(
                    div()
                        .px_3()
                        .py_1()
                        .rounded_md()
                        .bg(if is_modified {
                            pal.primary_accent
                        } else {
                            pal.card_surface
                        })
                        .border_1()
                        .border_color(if is_modified {
                            pal.primary_accent
                        } else {
                            pal.card_border
                        })
                        .hover(move |s| {
                            if is_modified {
                                s.opacity(0.85)
                            } else {
                                s.bg(pal.card_hover)
                            }
                        })
                        .cursor_pointer()
                        .text_size(px(11.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(save_btn_fg)
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_save.update(cx, |st, cx| {
                                st.save_active_document();
                                cx.notify();
                            });
                        })
                        .child(format!("💾 {} (Ctrl+S)", state.t("code_editor.save"))),
                ),
        )
}
