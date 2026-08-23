use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::ui::state::AppState;

/// 渲染极限性能模式设置卡片
pub fn render_perf_mode_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
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
pub fn render_culling_info_card(state: &AppState) -> impl IntoElement {
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
pub fn render_perf_stats_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
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

/// 渲染带图标的选项单选胶囊 (画布设置专用变体)
pub fn render_icon_choice_chip<F>(
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
