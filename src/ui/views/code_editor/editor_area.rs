use gpui::*;
use gpui_component::input::{Editor, EditorState};
use crate::model::CodeDocumentState;
use crate::ui::state::AppState;

/// 渲染高性能、可直接编辑的现代代码编辑器主体 (集成 GPU 硬件加速与虚拟化行流式布局)
pub fn render_editor_area(
    _code_state: &CodeDocumentState,
    _state: &AppState,
    _model: &Entity<AppState>,
    editor_state: Option<&Entity<EditorState>>,
) -> impl IntoElement {
    div()
        .id("code_editor_interactive_container")
        .flex_1()
        .size_full()
        .overflow_hidden()
        .bg(hsla(240.0 / 360.0, 0.06, 0.07, 1.0))
        .children(editor_state.map(|ed| {
            Editor::new(ed)
                .bordered(false)
                .appearance(false)
                .size_full()
        }))
}
