use gpui::*;
use gpui_component::input::EditorState;
use crate::ui::state::AppState;

pub mod editor_area;
pub mod header;

pub use editor_area::render_editor_area;
pub use header::render_code_editor_header;

/// 渲染通用多格式代码编辑器全屏视图 (支持 MSBT, BYML, AAMP, BFEVFL, JSON, YAML, 文本等)
pub fn render_code_editor_view(
    state: &AppState,
    model: &Entity<AppState>,
    editor_state: Option<&Entity<EditorState>>,
) -> impl IntoElement {
    let active_id = state.active_tab_id.as_deref().unwrap_or("");
    let active_tab = state.open_tabs.iter().find(|t| t.id == active_id);

    if let Some(tab) = active_tab {
        if let Some(ref code_state) = tab.code_state {
            return div()
                .flex()
                .flex_col()
                .size_full()
                .overflow_hidden()
                .bg(hsla(240.0 / 360.0, 0.06, 0.07, 1.0))
                .child(render_code_editor_header(code_state, tab.is_modified, state, model, editor_state))
                .child(render_editor_area(code_state, state, model, editor_state))
                .into_any_element();
        }
    }

    // 空白未加载提示
    div()
        .flex()
        .items_center()
        .justify_center()
        .size_full()
        .bg(hsla(240.0 / 360.0, 0.06, 0.07, 1.0))
        .text_color(hsla(240.0 / 360.0, 0.05, 0.50, 1.0))
        .child(state.t("code_editor.empty_prompt"))
        .into_any_element()
}
