pub mod bubble_input;
pub mod bubble_preview;
pub mod bubble_toolbar;
pub mod entry_sidebar;
pub mod header_switch;
pub mod new_entry_modal;
pub mod parser;
pub mod types;

pub use header_switch::render_msbt_view_mode_toggle;
pub use types::BubbleType;

use gpui::*;
use gpui_component::input::EditorState;
use crate::model::code_document::CodeDocumentState;
use crate::ui::state::AppState;
use bubble_input::render_msbt_bubble_input;
use bubble_preview::render_msbt_bubble_preview;
use bubble_toolbar::render_msbt_bubble_toolbar;
use entry_sidebar::render_msbt_entry_sidebar;
use new_entry_modal::render_new_entry_modal;
use parser::parse_msyt_to_document;

/// 渲染可视化 MSBT 气泡对话编辑器主视图 (参考 Bubble-Wrap 界面架构)
pub fn render_msbt_bubble_editor(
    code_state: &CodeDocumentState,
    _is_modified: bool,
    state: &AppState,
    model: &Entity<AppState>,
    bubble_editor: Option<&Entity<EditorState>>,
) -> impl IntoElement {
    let pal = state.theme();

    let doc_res = parse_msyt_to_document(&code_state.content);

    match doc_res {
        Ok(doc) => {
            let active_key = state
                .msbt_selected_entry_key
                .as_deref()
                .or_else(|| doc.entries.first().map(|e| e.key.as_str()));

            let active_entry = active_key.and_then(|k| doc.entries.iter().find(|e| e.key == k));

            div()
                .flex()
                .size_full()
                .overflow_hidden()
                .bg(pal.app_bg)
                // 1. 左侧：条目列表与检索侧栏
                .child(render_msbt_entry_sidebar(&doc, active_key, state, model))
                // 2. 右侧：气泡工具栏、荒野之息原生效果预览与文本输入区
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        // 顶栏：气泡形态切换、动作与音效、快捷标签注入
                        .child(render_msbt_bubble_toolbar(active_entry, state.msbt_bubble_type, state, model))
                        // 中下部：滚动展示与输入卡片
                        .child(
                            div()
                                .id("msbt_bubble_main_scroll")
                                .flex_1()
                                .overflow_y_scroll()
                                .flex()
                                .flex_col()
                                .justify_between()
                                .child(render_msbt_bubble_preview(active_entry, state.msbt_bubble_type, state))
                                .child(render_msbt_bubble_input(active_entry, state, model, bubble_editor)),
                        ),
                )
                // 3. 新建条目模态框
                .children(state.msbt_new_entry_modal_open.then(|| {
                    render_new_entry_modal(state, model).into_any_element()
                }))
                .into_any_element()
        }
        Err(err) => {
            let model_switch = model.clone();
            div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .p_8()
                .bg(pal.app_bg)
                .child(div().text_size(px(40.0)).child("⚠️"))
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(16.0))
                        .text_color(pal.text_primary)
                        .child("当前 MSYT YAML 包含非标准语法，无法以可视化气泡渲染"),
                )
                .child(
                    div()
                        .max_w(px(520.0))
                        .text_size(px(12.0))
                        .text_color(pal.text_secondary)
                        .child(err),
                )
                .child(
                    div()
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .bg(pal.primary_accent)
                        .cursor_pointer()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_switch.update(cx, |st, cx| {
                                if let Some(tab_id) = st.active_tab_id.clone() {
                                    if let Some(tab) = st.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                                        if let Some(ref mut code) = tab.code_state {
                                            code.msbt_view_mode = crate::model::code_document::MsbtViewMode::Code;
                                        }
                                    }
                                }
                                cx.notify();
                            });
                        })
                        .child("切换回 MSYT 源码模式修复"),
                )
                .into_any_element()
        }
    }
}
