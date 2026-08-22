use gpui::*;
use crate::ui::state::{AppState, AppWorkspaceTab};

/// 渲染底部全局工作区切换栏 (Flowchart / Actors / Events / JSON)
pub fn render_bottom_tabs(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let current_tab = state.current_tab;

    div()
        .w_full()
        .h(px(34.0))
        .bg(hsla(220.0 / 360.0, 0.40, 0.08, 0.98))
        .border_t_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
        .flex()
        .items_center()
        .px_3()
        .child(render_bottom_tab_item(
            state.t("tab.flowchart"),
            current_tab == AppWorkspaceTab::Flowchart,
            AppWorkspaceTab::Flowchart,
            model,
        ))


        .child(render_bottom_tab_item(
            state.t("tab.actors"),
            current_tab == AppWorkspaceTab::Actors,
            AppWorkspaceTab::Actors,
            model,
        ))
        .child(render_bottom_tab_item(
            state.t("tab.events"),
            current_tab == AppWorkspaceTab::Events,
            AppWorkspaceTab::Events,
            model,
        ))
        .child(render_bottom_tab_item(
            state.t("tab.json"),
            current_tab == AppWorkspaceTab::Json,
            AppWorkspaceTab::Json,
            model,
        ))
}

fn render_bottom_tab_item(
    label: &'static str,
    is_active: bool,
    target_tab: AppWorkspaceTab,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_click = model.clone();

    div()
        .px_3()
        .py_1()
        .rounded_md()
        .bg(if is_active {
            hsla(215.0 / 360.0, 0.45, 0.22, 1.0)
        } else {
            hsla(0.0, 0.0, 0.0, 0.0)
        })
        .border_1()
        .border_color(if is_active {
            hsla(215.0 / 360.0, 0.60, 0.45, 1.0)
        } else {
            hsla(0.0, 0.0, 0.0, 0.0)
        })
        .text_size(px(11.5))
        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if is_active {
            hsla(0.0, 0.0, 1.0, 1.0)
        } else {
            hsla(215.0 / 360.0, 0.20, 0.65, 1.0)
        })
        .cursor_pointer()
        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.30, 0.16, 1.0)))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_click.update(cx, |st, cx| {
                st.current_tab = target_tab;
                cx.notify();
            });
        })
        .child(label)
}
