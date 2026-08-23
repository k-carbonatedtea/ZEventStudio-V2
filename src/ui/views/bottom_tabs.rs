use gpui::*;
use crate::ui::state::{AppState, AppWorkspaceTab};

/// 渲染底部全局工作区切换栏 (Flowchart / Code Editor / Actors / Events / JSON)
pub fn render_bottom_tabs(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let current_tab = state.current_tab;

    div()
        .w_full()
        .h(px(34.0))
        .bg(pal.panel_surface) // 蓝灰表面 Panel Surface
        .border_t_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .px_3()
        .child(render_bottom_tab_item(
            state.t("tab.flowchart"),
            current_tab == AppWorkspaceTab::Flowchart,
            AppWorkspaceTab::Flowchart,
            state,
            model,
        ))
        .child(render_bottom_tab_item(
            state.t("tab.code_editor"),
            current_tab == AppWorkspaceTab::CodeEditor,
            AppWorkspaceTab::CodeEditor,
            state,
            model,
        ))
        .child(render_bottom_tab_item(
            state.t("tab.actors"),
            current_tab == AppWorkspaceTab::Actors,
            AppWorkspaceTab::Actors,
            state,
            model,
        ))
        .child(render_bottom_tab_item(
            state.t("tab.events"),
            current_tab == AppWorkspaceTab::Events,
            AppWorkspaceTab::Events,
            state,
            model,
        ))
        .child(render_bottom_tab_item(
            state.t("tab.json"),
            current_tab == AppWorkspaceTab::Json,
            AppWorkspaceTab::Json,
            state,
            model,
        ))
}

fn render_bottom_tab_item(
    label: &'static str,
    is_active: bool,
    target_tab: AppWorkspaceTab,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_click = model.clone();

    div()
        .px_3()
        .py_1()
        .rounded_md()
        .bg(if is_active {
            pal.card_surface // 浅层蓝灰 Card / Hover
        } else {
            rgba(0x00000000)
        })
        .border_1()
        .border_color(if is_active {
            pal.primary_accent // 遗迹青蓝 Primary Accent
        } else {
            rgba(0x00000000)
        })
        .text_size(px(11.5))
        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if is_active {
            pal.primary_accent // 遗迹青蓝 Primary Accent
        } else {
            pal.text_secondary // 蓝灰白 Text Secondary
        })
        .cursor_pointer()
        .hover(move |s| s.bg(pal.card_hover).text_color(pal.text_primary))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_click.update(cx, |st, cx| {
                st.switch_workspace_tab(target_tab);
                cx.notify();
            });
        })
        .child(label)
}
