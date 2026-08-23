use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::ui::state::AppState;
use crate::ui::state_settings::{EdgeCurveStyle, GridStyle};
use super::canvas_helpers::{
    render_culling_info_card, render_icon_choice_chip, render_perf_mode_card,
    render_perf_stats_card,
};

/// 渲染编辑器与画布设置面板 (Editor & Canvas Settings Pane)
pub fn render_canvas_pane(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    div()
        .id("settings_canvas_scroll")
        .flex_1()
        .flex()
        .flex_col()
        .gap_4()
        .overflow_y_scroll()
        .p_1()
        // 1. 默认展开全部节点参数开关
        .child(render_params_expand_card(state, model))
        // 2. 画布背景网格渲染样式
        .child(render_grid_style_card(state, model))
        // 3. 连线曲线拟合算法
        .child(render_edge_style_card(state, model))
        // 4. 极限性能模式
        .child(render_perf_mode_card(state, model))
        // 5. 硬件加速视口边界裁剪说明与状态
        .child(render_culling_info_card(state))
        // 6. 状态栏性能监视器常驻开关
        .child(render_perf_stats_card(state, model))
}

/// 渲染参数展开偏好设置卡片
fn render_params_expand_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let is_expanded = state.expand_all_params;
    let model_toggle = model.clone();

    div()
        .p_4()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(div().text_size(px(16.0)).child("📋"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.params_expand_title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.params_expand_desc")),
                        ),
                ),
        )
        .child(
            div()
                .w(px(40.0))
                .h(px(22.0))
                .rounded_full()
                .bg(if is_expanded { pal.primary_accent } else { pal.app_bg })
                .border_1()
                .border_color(if is_expanded { pal.primary_accent } else { pal.card_border })
                .p_0p5()
                .flex()
                .items_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_toggle.update(cx, |st, cx| {
                        st.toggle_expand_params();
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .size(px(16.0))
                        .rounded_full()
                        .bg(if is_expanded { rgb(0x0A0C10) } else { pal.text_secondary })
                        .when(is_expanded, |s| s.ml(px(18.0))),
                ),
        )
}

/// 渲染画布背景网格样式配置卡片
fn render_grid_style_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let current_grid = state.grid_style;

    div()
        .p_4()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(div().text_size(px(16.0)).child("📐"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.grid_style_title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.grid_style_desc")),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .gap_2p5()
                .child(render_icon_choice_chip(
                    state.t("settings.grid_dots"),
                    "::",
                    current_grid == GridStyle::Dots,
                    &pal,
                    {
                        let m = model.clone();
                        move |cx| {
                            m.update(cx, |st, cx| {
                                st.grid_style = GridStyle::Dots;
                                cx.notify();
                            });
                        }
                    },
                ))
                .child(render_icon_choice_chip(
                    state.t("settings.grid_lines"),
                    "#",
                    current_grid == GridStyle::Lines,
                    &pal,
                    {
                        let m = model.clone();
                        move |cx| {
                            m.update(cx, |st, cx| {
                                st.grid_style = GridStyle::Lines;
                                cx.notify();
                            });
                        }
                    },
                ))
                .child(render_icon_choice_chip(
                    state.t("settings.grid_none"),
                    "—",
                    current_grid == GridStyle::None,
                    &pal,
                    {
                        let m = model.clone();
                        move |cx| {
                            m.update(cx, |st, cx| {
                                st.grid_style = GridStyle::None;
                                cx.notify();
                            });
                        }
                    },
                )),
        )
}

/// 渲染连线拟合曲线算法配置卡片
fn render_edge_style_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let current_edge = state.edge_curve_style;

    div()
        .p_4()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(div().text_size(px(16.0)).child("〰️"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.edge_style_title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.edge_style_desc")),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .gap_2p5()
                .child(render_icon_choice_chip(
                    state.t("settings.edge_bezier"),
                    "∿",
                    current_edge == EdgeCurveStyle::Bezier,
                    &pal,
                    {
                        let m = model.clone();
                        move |cx| {
                            m.update(cx, |st, cx| {
                                st.edge_curve_style = EdgeCurveStyle::Bezier;
                                cx.notify();
                            });
                        }
                    },
                ))
                .child(render_icon_choice_chip(
                    state.t("settings.edge_smooth"),
                    "┐",
                    current_edge == EdgeCurveStyle::Smooth,
                    &pal,
                    {
                        let m = model.clone();
                        move |cx| {
                            m.update(cx, |st, cx| {
                                st.edge_curve_style = EdgeCurveStyle::Smooth;
                                cx.notify();
                            });
                        }
                    },
                )),
        )
}
