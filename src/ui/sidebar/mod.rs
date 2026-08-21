pub mod entry_points;
pub mod node_palette;
pub mod preset_list;

use gpui::*;
use gpui::StatefulInteractiveElement;
use crate::ui::sidebar::entry_points::render_entry_point_list;
use crate::ui::sidebar::node_palette::render_node_palette;
use crate::ui::sidebar::preset_list::render_preset_list;
use crate::ui::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    EntryPoints,
    Presets,
    Palette,
}

pub fn render_sidebar(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let active_tab = state.active_sidebar_tab;
    let search_val = state.search_query.clone();
    let model_tab_ep = model.clone();
    let model_tab_palette = model.clone();
    let model_tab_presets = model.clone();

    div()
        .w(px(260.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(hsla(220.0 / 360.0, 0.35, 0.10, 0.98))
        .border_r_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.20, 1.0))
        // Tab Header
        .child(
            div()
                .flex()
                .border_b_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.20, 1.0))
                .p_1p5()
                .gap_1()
                .child({
                    let is_active = active_tab == SidebarTab::EntryPoints;
                    div()
                        .flex_1()
                        .py_1p5()
                        .rounded_md()
                        .bg(if is_active {
                            hsla(215.0 / 360.0, 0.4, 0.22, 1.0)
                        } else {
                            hsla(0.0, 0.0, 0.0, 0.0)
                        })
                        .text_size(px(11.0))
                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .text_color(if is_active {
                            hsla(0.0, 0.0, 1.0, 1.0)
                        } else {
                            hsla(215.0 / 360.0, 0.2, 0.65, 1.0)
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_tab_ep.update(cx, |st, cx| {
                                st.active_sidebar_tab = SidebarTab::EntryPoints;
                                cx.notify();
                            });
                        })
                        .child("🏁 流程入口")
                })
                .child({
                    let is_active = active_tab == SidebarTab::Presets;
                    div()
                        .flex_1()
                        .py_1p5()
                        .rounded_md()
                        .bg(if is_active {
                            hsla(215.0 / 360.0, 0.4, 0.22, 1.0)
                        } else {
                            hsla(0.0, 0.0, 0.0, 0.0)
                        })
                        .text_size(px(11.0))
                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .text_color(if is_active {
                            hsla(0.0, 0.0, 1.0, 1.0)
                        } else {
                            hsla(215.0 / 360.0, 0.2, 0.65, 1.0)
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_tab_presets.update(cx, |st, cx| {
                                st.active_sidebar_tab = SidebarTab::Presets;
                                cx.notify();
                            });
                        })
                        .child("🎮 预设案例")
                })
                .child({
                    let is_active = active_tab == SidebarTab::Palette;
                    div()
                        .flex_1()
                        .py_1p5()
                        .rounded_md()
                        .bg(if is_active {
                            hsla(215.0 / 360.0, 0.4, 0.22, 1.0)
                        } else {
                            hsla(0.0, 0.0, 0.0, 0.0)
                        })
                        .text_size(px(11.0))
                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .text_color(if is_active {
                            hsla(0.0, 0.0, 1.0, 1.0)
                        } else {
                            hsla(215.0 / 360.0, 0.2, 0.65, 1.0)
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_tab_palette.update(cx, |st, cx| {
                                st.active_sidebar_tab = SidebarTab::Palette;
                                cx.notify();
                            });
                        })
                        .child("📦 节点组件")
                }),
        )
        // Search bar (in palette mode)
        .children(if active_tab == SidebarTab::Palette {
            Some(
                div()
                    .px_3()
                    .pt_2p5()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2p5()
                            .py_1()
                            .rounded_lg()
                            .bg(hsla(220.0 / 360.0, 0.25, 0.15, 1.0))
                            .border_1()
                            .border_color(hsla(215.0 / 360.0, 0.25, 0.28, 1.0))
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .child("🔍"),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(px(11.5))
                                    .text_color(rgb(0xffffff))
                                    .child(if search_val.is_empty() {
                                        "搜索节点类型...".to_string()
                                    } else {
                                        search_val
                                    }),
                            ),
                    ),
            )
        } else {
            None
        })
        // Content Area
        .child(
            div()
                .id("sidebar_scroll_body")
                .flex_1()
                .overflow_y_scroll()
                .child(match active_tab {
                    SidebarTab::EntryPoints => render_entry_point_list(state, model).into_any_element(),
                    SidebarTab::Presets => render_preset_list(state, model).into_any_element(),
                    SidebarTab::Palette => render_node_palette(state, model).into_any_element(),
                }),
        )
}
