pub mod activity_bar;
pub mod actors_manager;
pub mod entry_points;
pub mod events_list;
pub mod files_manager;
pub mod node_palette;
pub mod search_panel;
pub mod source_control_panel;
pub mod diff_modal;
pub mod sarc_subfile_view;
pub mod commit_context_menu;

pub use activity_bar::render_activity_bar;
pub use search_panel::render_search_panel;
pub use source_control_panel::render_source_control_panel;
pub use diff_modal::render_git_diff_modal;
pub use commit_context_menu::render_commit_context_menu;

use crate::ui::sidebar::actors_manager::render_actors_manager;
use crate::ui::sidebar::entry_points::render_entry_point_list;
use crate::ui::sidebar::events_list::render_events_list;
use crate::ui::sidebar::files_manager::render_files_manager;
use crate::ui::sidebar::node_palette::render_node_palette;
use crate::ui::state::{ActivityBarTab, AppState};
use gpui::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    Files,
    EntryPoints,
    Events,
    Actors,
    Palette,
}

/// 渲染左侧导航侧边栏 (整合垂直活动栏与功能面板)
pub fn render_sidebar(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let current_activity = state.active_activity_tab;
    let sidebar_open = state.sidebar_open;

    div()
        .flex()
        .h_full()
        // 1. 最左侧垂直活动栏 (48px)
        .child(render_activity_bar(state, model))
        // 2. 侧边栏主体面板 (仅在展开时显示)
        .children(sidebar_open.then(|| {
            div()
                .w(px(state.sidebar_width))
                .flex_shrink_0()
                .h_full()
                .flex()
                .flex_col()
                .bg(pal.panel_surface)
                .border_r_1()
                .border_color(pal.panel_border)
                .overflow_hidden()
                .child(match current_activity {
                    ActivityBarTab::Explorer => {
                        render_sidebar_explorer_content(state, model).into_any_element()
                    }
                    ActivityBarTab::Search => {
                        render_search_panel(state, model).into_any_element()
                    }
                    ActivityBarTab::SourceControl => {
                        render_source_control_panel(state, model).into_any_element()
                    }
                })
        }))
}

/// 渲染侧边栏资源与事件流程图专属浏览器内容
fn render_sidebar_explorer_content(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let active_tab = state.active_sidebar_tab;
    let search_val = state.search_query.clone();

    let model_tab_fl = model.clone();
    let model_tab_ep = model.clone();
    let model_tab_ev = model.clone();
    let model_tab_act = model.clone();
    let model_tab_pal = model.clone();

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(pal.panel_surface)
        .overflow_hidden()
        // 顶部 Tab 选项卡栏
        .child(
            div()
                .flex()
                .border_b_1()
                .border_color(pal.panel_border)
                .bg(pal.panel_surface)
                .p_1()
                .gap_1()
                // 0. 资源
                .child(render_tab_btn(
                    state.t("sidebar.tab_files"),
                    active_tab == SidebarTab::Files,
                    SidebarTab::Files,
                    state,
                    &model_tab_fl,
                ))
                // 1. 入口
                .child(render_tab_btn(
                    state.t("sidebar.tab_entry_points"),
                    active_tab == SidebarTab::EntryPoints,
                    SidebarTab::EntryPoints,
                    state,
                    &model_tab_ep,
                ))
                // 2. 事件
                .child(render_tab_btn(
                    state.t("sidebar.tab_events"),
                    active_tab == SidebarTab::Events,
                    SidebarTab::Events,
                    state,
                    &model_tab_ev,
                ))
                // 3. 角色
                .child(render_tab_btn(
                    state.t("sidebar.tab_actors"),
                    active_tab == SidebarTab::Actors,
                    SidebarTab::Actors,
                    state,
                    &model_tab_act,
                ))
                // 4. 组件
                .child(render_tab_btn(
                    state.t("sidebar.tab_palette"),
                    active_tab == SidebarTab::Palette,
                    SidebarTab::Palette,
                    state,
                    &model_tab_pal,
                )),
        )
        // 搜索过滤栏 (在事件/角色/组件 Tab 下展示)
        .children(
            if matches!(
                active_tab,
                SidebarTab::Events | SidebarTab::Actors | SidebarTab::Palette
            ) {
                let placeholder = match active_tab {
                    SidebarTab::Events => state.t("sidebar.events_search"),
                    SidebarTab::Actors => state.t("sidebar.actors_search"),
                    _ => state.t("sidebar.search_placeholder"),
                };
                Some(
                    div().px_2p5().pt_2().child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2p5()
                            .py_1()
                            .rounded_lg()
                            .bg(pal.app_bg)
                            .border_1()
                            .border_color(pal.panel_border)
                            .child(div().text_size(px(11.0)).child("🔍"))
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(px(11.0))
                                    .text_color(if search_val.is_empty() {
                                        pal.text_secondary
                                    } else {
                                        pal.text_primary
                                    })
                                    .child(if search_val.is_empty() {
                                        placeholder.to_string()
                                    } else {
                                        search_val
                                    }),
                            ),
                    ),
                )
            } else {
                None
            },
        )
        // 侧边栏主体内容渲染
        .child(
            div()
                .id("sidebar_scroll_body")
                .flex_1()
                .overflow_y_scroll()
                .child(match active_tab {
                    SidebarTab::Files => render_files_manager(state, model).into_any_element(),
                    SidebarTab::EntryPoints => {
                        render_entry_point_list(state, model).into_any_element()
                    }
                    SidebarTab::Events => render_events_list(state, model).into_any_element(),
                    SidebarTab::Actors => render_actors_manager(state, model).into_any_element(),
                    SidebarTab::Palette => render_node_palette(state, model).into_any_element(),
                }),
        )
}

fn render_tab_btn(
    title: &'static str,
    is_active: bool,
    target_tab: SidebarTab,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_click = model.clone();
    div()
        .flex_1()
        .py_1p5()
        .rounded_md()
        .bg(if is_active {
            pal.card_surface // 浅层蓝灰 Card / Hover
        } else {
            rgba(0x00000000)
        })
        .text_size(px(10.5))
        .font_weight(if is_active {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        })
        .text_color(if is_active {
            pal.primary_accent // 遗迹青蓝 Primary Accent
        } else {
            pal.text_secondary // 蓝灰白 Text Secondary
        })
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(pal.card_surface).text_color(pal.text_primary))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_click.update(cx, |st, cx| {
                st.active_sidebar_tab = target_tab;
                cx.notify();
            });
        })
        .child(title)
}

/// 渲染侧边栏右侧可拖拽调整宽度的分界条 (支持悬浮与拖拽高亮)
pub fn render_sidebar_resize_handle(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let is_resizing = state.is_resizing_sidebar;
    let model_down = model.clone();

    div()
        .id("sidebar_resize_handle")
        .w(px(4.0))
        .h_full()
        .flex_shrink_0()
        .cursor_col_resize()
        .bg(if is_resizing {
            pal.primary_accent // 遗迹青蓝 Primary Accent
        } else {
            pal.panel_border // 蓝灰边框
        })
        .hover(|s| s.bg(pal.primary_accent))
        .on_mouse_down(MouseButton::Left, move |_e, _, cx| {
            cx.stop_propagation();
            model_down.update(cx, |st, cx| {
                st.is_resizing_sidebar = true;
                cx.notify();
            });
        })
}
