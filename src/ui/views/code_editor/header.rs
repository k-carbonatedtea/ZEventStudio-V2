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
    let line_count = code_state.content.lines().count();
    let char_count = code_state.content.len();
    let file_path = code_state.file_path.clone();
    let format_name = code_state.format.display_name();
    let format_icon = code_state.format.icon();
    let endian_label = code_state.endian.display_label();

    let model_save = model.clone();
    let model_revert = model.clone();
    let ed_revert = editor_state.cloned();
    let original_content = code_state.original_content.clone();

    div()
        .h(px(40.0))
        .w_full()
        .px_4()
        .bg(hsla(240.0 / 360.0, 0.06, 0.10, 0.95))
        .border_b_1()
        .border_color(hsla(240.0 / 360.0, 0.06, 0.18, 1.0))
        .flex()
        .items_center()
        .justify_between()
        // 左侧：格式徽章、大小端指示与文件路径
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
                        .bg(hsla(215.0 / 360.0, 0.50, 0.25, 0.9))
                        .border_1()
                        .border_color(hsla(215.0 / 360.0, 0.70, 0.45, 1.0))
                        .child(div().text_size(px(12.0)).child(format_icon))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.0))
                                .text_color(hsla(215.0 / 360.0, 0.90, 0.85, 1.0))
                                .child(format_name),
                        ),
                )
                // 大小端徽章
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(hsla(270.0 / 360.0, 0.35, 0.20, 0.9))
                        .border_1()
                        .border_color(hsla(270.0 / 360.0, 0.55, 0.40, 1.0))
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hsla(270.0 / 360.0, 0.85, 0.80, 1.0))
                        .child(endian_label),
                )
                // 路径
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(hsla(240.0 / 360.0, 0.05, 0.70, 1.0))
                        .overflow_hidden()
                        .child(file_path),
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
                        .bg(hsla(45.0 / 360.0, 0.95, 0.25, 0.9))
                        .border_1()
                        .border_color(hsla(45.0 / 360.0, 0.95, 0.55, 1.0))
                        .child(
                            div()
                                .size(px(6.0))
                                .rounded_full()
                                .bg(hsla(45.0 / 360.0, 0.95, 0.60, 1.0)),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hsla(45.0 / 360.0, 0.95, 0.75, 1.0))
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
                        .text_color(hsla(240.0 / 360.0, 0.05, 0.50, 1.0))
                        .child(format!("{} {}, {} {}", line_count, state.t("code_editor.lines"), char_count, state.t("code_editor.chars"))),
                )
                // 重置修改按钮 (仅在已修改时高亮可用)
                .children(is_modified.then(|| {
                    div()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(hsla(0.0, 0.50, 0.20, 0.8))
                        .border_1()
                        .border_color(hsla(0.0, 0.65, 0.40, 1.0))
                        .hover(|s| s.bg(hsla(0.0, 0.65, 0.30, 1.0)))
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xffffff))
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
                            hsla(145.0 / 360.0, 0.65, 0.25, 1.0)
                        } else {
                            hsla(240.0 / 360.0, 0.06, 0.18, 0.8)
                        })
                        .border_1()
                        .border_color(if is_modified {
                            hsla(145.0 / 360.0, 0.85, 0.45, 1.0)
                        } else {
                            hsla(240.0 / 360.0, 0.06, 0.25, 1.0)
                        })
                        .hover(|s| {
                            if is_modified {
                                s.bg(hsla(145.0 / 360.0, 0.75, 0.32, 1.0))
                            } else {
                                s.bg(hsla(240.0 / 360.0, 0.06, 0.24, 1.0))
                            }
                        })
                        .cursor_pointer()
                        .text_size(px(11.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xffffff))
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
