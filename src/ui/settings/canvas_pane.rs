use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::ui::state::AppState;
use crate::ui::state_settings::{EdgeCurveStyle, GridStyle};

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
                .child(render_choice_chip(
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
                .child(render_choice_chip(
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
                .child(render_choice_chip(
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
                .child(render_choice_chip(
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
                .child(render_choice_chip(
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

/// 渲染极限性能模式设置卡片
fn render_perf_mode_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let is_perf = state.performance_mode;
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
                .child(div().text_size(px(16.0)).child("⚡"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.perf_mode_title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.perf_mode_desc")),
                        ),
                ),
        )
        .child(
            div()
                .w(px(40.0))
                .h(px(22.0))
                .rounded_full()
                .bg(if is_perf { pal.secondary_accent } else { pal.app_bg })
                .border_1()
                .border_color(if is_perf { pal.secondary_accent } else { pal.card_border })
                .p_0p5()
                .flex()
                .items_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_toggle.update(cx, |st, cx| {
                        st.toggle_performance_mode();
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .size(px(16.0))
                        .rounded_full()
                        .bg(if is_perf { rgb(0x0A0C10) } else { pal.text_secondary })
                        .when(is_perf, |s| s.ml(px(18.0))),
                ),
        )
}

/// 渲染硬件视口边界裁剪状态卡片
fn render_culling_info_card(state: &AppState) -> impl IntoElement {
    let pal = state.theme();

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
                .child(div().text_size(px(16.0)).child("🎯"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.culling_title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.culling_desc")),
                        ),
                ),
        )
        .child(
            div()
                .px_2p5()
                .py_1()
                .rounded_md()
                .bg(pal.primary_accent.opacity(0.15))
                .border_1()
                .border_color(pal.primary_accent)
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(pal.primary_accent)
                .child(state.t("settings.culling_active")),
        )
}

/// 渲染顶部工具栏性能监控开关卡片
fn render_perf_stats_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let is_stats_enabled = state.show_perf_stats;
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
                .child(div().text_size(px(16.0)).child("📊"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.status_bar_title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.status_bar_desc")),
                        ),
                ),
        )
        .child(
            div()
                .w(px(40.0))
                .h(px(22.0))
                .rounded_full()
                .bg(if is_stats_enabled { pal.primary_accent } else { pal.app_bg })
                .border_1()
                .border_color(if is_stats_enabled { pal.primary_accent } else { pal.card_border })
                .p_0p5()
                .flex()
                .items_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_toggle.update(cx, |st, cx| {
                        st.show_perf_stats = !st.show_perf_stats;
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .size(px(16.0))
                        .rounded_full()
                        .bg(if is_stats_enabled { rgb(0x0A0C10) } else { pal.text_secondary })
                        .when(is_stats_enabled, |s| s.ml(px(18.0))),
                ),
        )
}

/// 渲染选项单选胶囊
fn render_choice_chip<F>(
    label: &str,
    icon: &'static str,
    is_active: bool,
    pal: &crate::ui::theme::ThemePalette,
    on_select: F,
) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    let bg_color = if is_active {
        pal.primary_accent.opacity(0.18)
    } else {
        pal.app_bg
    };
    let border_color = if is_active {
        pal.primary_accent
    } else {
        pal.panel_border
    };
    let text_color = if is_active {
        pal.primary_accent
    } else {
        pal.text_secondary
    };

    div()
        .flex_1()
        .h(px(38.0))
        .px_3()
        .rounded_lg()
        .bg(bg_color)
        .border_1()
        .border_color(border_color)
        .flex()
        .items_center()
        .justify_between()
        .cursor_pointer()
        .hover(|s| s.border_color(pal.primary_accent))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            on_select(cx);
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().font_weight(FontWeight::BOLD).text_size(px(12.0)).child(icon))
                .child(
                    div()
                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .text_size(px(12.0))
                        .text_color(text_color)
                        .child(label.to_string()),
                ),
        )
        .child(
            if is_active {
                div()
                    .text_size(px(11.5))
                    .text_color(pal.primary_accent)
                    .child("✓")
            } else {
                div()
            },
        )
}
