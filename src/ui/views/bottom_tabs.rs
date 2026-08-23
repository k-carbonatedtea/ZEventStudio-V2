use gpui::*;
use crate::ui::state::{AppState, AppWorkspaceTab};

/// 渲染底部全局工作区切换栏 (Flowchart / Code Editor / Actors / Events / JSON)
pub fn render_bottom_tabs(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let current_tab = state.current_tab;

    div()
        .w_full()
        .h(px(34.0))
        .bg(rgb(0x161922)) // 蓝灰表面 Panel Surface
        .border_t_1()
        .border_color(rgb(0x232838))
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
            state.t("tab.code_editor"),
            current_tab == AppWorkspaceTab::CodeEditor,
            AppWorkspaceTab::CodeEditor,
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
            rgb(0x1E2330) // 浅层蓝灰 Card / Hover
        } else {
            rgba(0x00000000)
        })
        .border_1()
        .border_color(if is_active {
            rgb(0x00E5BA) // 遗迹青蓝 Primary Accent
        } else {
            rgba(0x00000000)
        })
        .text_size(px(11.5))
        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if is_active {
            rgb(0x00E5BA) // 遗迹青蓝 Primary Accent
        } else {
            rgb(0x94A3B8) // 蓝灰白 Text Secondary
        })
        .cursor_pointer()
        .hover(|s| s.bg(rgb(0x1E2330)).text_color(rgb(0xF1F5F9)))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_click.update(cx, |st, cx| {
                st.current_tab = target_tab;
                cx.notify();
            });
        })
        .child(label)
}
