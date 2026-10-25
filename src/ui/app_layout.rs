use gpui::*;
use crate::ui::sidebar::render_sidebar;
use crate::ui::state::AppState;
use crate::ui::views::render_bottom_tabs;

/// 包装带侧边栏和顶部文档标签栏的主工作区布局
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
                        .child(crate::ui::render_document_tab_bar(state, model))
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

/// 包装全屏单视图工作区布局 (包含底部模式切换菜单栏)
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
