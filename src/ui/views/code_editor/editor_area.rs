use gpui::*;
use gpui_component::input::{Editor, EditorState};
use crate::model::CodeDocumentState;
use crate::ui::state::AppState;

/// 渲染高性能、可直接编辑的现代代码编辑器主体 (集成 GPU 硬件加速与虚拟化行流式布局)
pub fn render_editor_area(
    _code_state: &CodeDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    editor_state: Option<&Entity<EditorState>>,
) -> impl IntoElement {
    let pal = state.theme();
    let ed_click = editor_state.cloned();
    let model_click = model.clone();
    div()
        .id("code_editor_interactive_container")
        .flex_1()
        .size_full()
        .overflow_hidden()
        .bg(pal.app_bg)
        .capture_any_mouse_down(move |_e, window, cx| {
            model_click.update(cx, |st, cx| {
                if st.has_active_text_input() {
                    st.clear_active_text_inputs();
                    cx.notify();
                }
            });
            if let Some(ref ed) = ed_click {
                ed.update(cx, |editor, cx| {
                    editor.focus(window, cx);
                });
            }
        })
        .children(editor_state.map(|ed| {
            Editor::new(ed)
                .bordered(false)
                .appearance(false)
                .size_full()
        }))
}
