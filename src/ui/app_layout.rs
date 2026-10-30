use gpui::*;
use crate::ui::sidebar::render_sidebar;
use crate::ui::state::AppState;
use crate::ui::views::render_bottom_tabs;
use gpui_component::input::EditorState;
use crate::ui::canvas::render_flow_canvas;
use crate::ui::inspector::render_inspector;
use crate::ui::state::AppWorkspaceTab;
use crate::ui::views::{
    render_actor_manager_view, render_code_editor_view,
    render_diff_viewer_view, render_event_manager_view,
    render_model_preview_view, render_quest_map_view,
    render_timeline_editor_view, render_welcome_view,
};

/// 鍖呰甯︿晶杈规爮鍜岄《閮ㄦ枃妗ｆ爣绛炬爮鐨勪富宸ヤ綔鍖哄竷灞€
pub fn wrap_sidebar_workspace(
    state: &AppState,
    model: &Entity<AppState>,
    body: impl IntoElement,
    extra_right: Option<gpui::AnyElement>,
) -> impl IntoElement {
    div()
        .flex()
        .size_full()
        .overflow_hidden()
        .child(render_sidebar(state, model))
        .children(state.sidebar_open.then(|| crate::ui::sidebar::render_sidebar_resize_handle(state, model)))
        .child(
            div()
                .flex_1()
                .flex()
                .size_full()
                .overflow_hidden()
                .capture_any_mouse_down({
                    let model = model.clone();
                    move |_event, _window, cx| {
                        model.update(cx, |st, cx| {
                            if st.has_active_text_input() {
                                st.clear_active_text_inputs();
                                cx.notify();
                            }
                        });
                    }
                })
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .size_full()
                        .overflow_hidden()
                        .child(if state.current_tab == crate::ui::state::AppWorkspaceTab::Quests {
                            crate::ui::views::quest_map::render_quest_map_top_bar(state, model).into_any_element()
                        } else {
                            crate::ui::render_document_tab_bar(state, model).into_any_element()
                        })
                        .child(
                            div()
                                .flex_1()
                                .size_full()
                                .overflow_hidden()
                                .child(body),
                        )
                        .children((state.is_active_flowchart() && state.current_tab != crate::ui::state::AppWorkspaceTab::Quests).then(|| {
                            render_bottom_tabs(state, model)
                        })),
                )
                .children(extra_right),
        )
}

/// 鍖呰鍏ㄥ睆鍗曡鍥惧伐浣滃尯甯冨眬 (鍖呭惈搴曢儴妯″紡鍒囨崲鑿滃崟鏍?
pub fn wrap_fullscreen_tab(
    state: &AppState,
    model: &Entity<AppState>,
    body: impl IntoElement,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .size_full()
        .overflow_hidden()
        .child(
            div()
                .flex_1()
                .size_full()
                .overflow_hidden()
                .child(body),
        )
        .child(render_bottom_tabs(state, model))
}

/// 鏍规嵁褰撳墠閫変腑鐨勫伐浣滃尯鏍囩椤垫覆鏌撳搴旂殑鐣岄潰涓讳綋
pub fn render_workspace_tab_content(
    state: &AppState,
    model: &Entity<AppState>,
    code_editor: Option<&Entity<EditorState>>,
    msbt_bubble_editor: Option<&Entity<EditorState>>,
) -> impl IntoElement {
    match state.current_tab {
        AppWorkspaceTab::Welcome => render_welcome_view(state, model).into_any_element(),
        AppWorkspaceTab::Flowchart => {
            if state.open_tabs.is_empty() {
                render_welcome_view(state, model).into_any_element()
            } else {
                wrap_sidebar_workspace(
                    state,
                    model,
                    render_flow_canvas(state, model),
                    Some(render_inspector(state, model).into_any_element()),
                )
                .into_any_element()
            }
        }
        AppWorkspaceTab::Timeline => {
            if state.open_tabs.is_empty() {
                render_welcome_view(state, model).into_any_element()
            } else {
                wrap_sidebar_workspace(
                    state,
                    model,
                    render_timeline_editor_view(state, model),
                    None,
                )
                .into_any_element()
            }
        }
        AppWorkspaceTab::CodeEditor => {
            if state.open_tabs.is_empty() {
                render_welcome_view(state, model).into_any_element()
            } else {
                wrap_sidebar_workspace(
                    state,
                    model,
                    render_code_editor_view(
                        state,
                        model,
                        code_editor,
                        msbt_bubble_editor,
                    ),
                    None,
                )
                .into_any_element()
            }
        }
        AppWorkspaceTab::DiffViewer => wrap_sidebar_workspace(
            state,
            model,
            render_diff_viewer_view(state, model),
            None,
        )
        .into_any_element(),
        AppWorkspaceTab::Actors => {
            wrap_fullscreen_tab(state, model, render_actor_manager_view(state, model)).into_any_element()
        }
        AppWorkspaceTab::Events => {
            wrap_fullscreen_tab(state, model, render_event_manager_view(state, model)).into_any_element()
        }
        AppWorkspaceTab::Quests => wrap_sidebar_workspace(
            state,
            model,
            render_quest_map_view(state, model),
            None,
        )
        .into_any_element(),
        AppWorkspaceTab::ModelPreview => wrap_sidebar_workspace(
            state,
            model,
            render_model_preview_view(state, model),
            None,
        )
        .into_any_element(),
    }
}
