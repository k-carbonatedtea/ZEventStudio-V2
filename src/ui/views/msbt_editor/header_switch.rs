use gpui::*;
use crate::model::code_document::{CodeDocumentState, CodeFormat, MsbtViewMode};
use crate::ui::state::AppState;

/// 渲染代码编辑器顶部状态栏中的 MSBT 模式切换控件 (源码模式 ⇋ 气泡对话编辑器)
pub fn render_msbt_view_mode_toggle(
    code_state: &CodeDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    if code_state.format != CodeFormat::Msbt {
        return div().into_any_element();
    }

    let pal = state.theme();
    let current_mode = code_state.msbt_view_mode;
    let model_code = model.clone();
    let model_bubble = model.clone();

    div()
        .flex()
        .items_center()
        .rounded_md()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .p_0p5()
        .gap_1()
        // 模式 1: MSYT 源码模式
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .text_size(px(10.5))
                .font_weight(FontWeight::BOLD)
                .cursor_pointer()
                .bg(if current_mode == MsbtViewMode::Code {
                    pal.primary_accent
                } else {
                    rgba(0x00000000)
                })
                .text_color(if current_mode == MsbtViewMode::Code {
                    if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) }
                } else {
                    pal.text_secondary
                })
                .hover(|s| s.opacity(0.85))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_code.update(cx, |st, cx| {
                        if let Some(tab_id) = st.active_tab_id.clone() {
                            if let Some(tab) = st.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                                if let Some(ref mut code) = tab.code_state {
                                    code.msbt_view_mode = MsbtViewMode::Code;
                                }
                            }
                        }
                        cx.notify();
                    });
                })
                .child("📄")
                .child(state.t("code_editor.mode_msyt")),
        )
        // 模式 2: 气泡对话编辑器 (Bubble-Wrap 可视化模式)
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .text_size(px(10.5))
                .font_weight(FontWeight::BOLD)
                .cursor_pointer()
                .bg(if current_mode == MsbtViewMode::Bubble {
                    pal.secondary_accent
                } else {
                    rgba(0x00000000)
                })
                .text_color(if current_mode == MsbtViewMode::Bubble {
                    if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) }
                } else {
                    pal.text_secondary
                })
                .hover(|s| s.opacity(0.85))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_bubble.update(cx, |st, cx| {
                        if let Some(tab_id) = st.active_tab_id.clone() {
                            if let Some(tab) = st.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                                if let Some(ref mut code) = tab.code_state {
                                    code.msbt_view_mode = MsbtViewMode::Bubble;
                                }
                            }
                        }
                        st.sync_msbt_selected_entry_to_draft();
                        cx.notify();
                    });
                })
                .child("💬")
                .child(state.t("code_editor.mode_bubble")),
        )
        .into_any_element()
}
