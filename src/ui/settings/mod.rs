pub mod canvas_tab;
pub mod game_tab;
pub mod general_tab;
pub mod performance_tab;

use gpui::*;
use canvas_tab::render_canvas_tab;
use game_tab::render_game_tab;
use general_tab::render_general_tab;
use performance_tab::render_performance_tab;
use crate::ui::state::AppState;
use crate::ui::state_settings::SettingsTab;

/// 渲染现代化首选项与设置模态窗口 (Preferences & Settings Modal)
pub fn render_settings_modal(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    if !state.settings_modal_open {
        return div().into_any_element();
    }

    let pal = state.theme();
    let current_tab = state.settings_active_tab;
    let model_backdrop = model.clone();
    let model_close = model.clone();
    let model_reset = model.clone();
    let model_done = model.clone();

    div()
        .id("settings_modal_overlay")
        .absolute()
        .inset_0()
        .bg(hsla(0.0, 0.0, 0.0, 0.65))
        .flex()
        .items_center()
        .justify_center()
        .on_scroll_wheel(|_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_backdrop.update(cx, |st, cx| {
                st.close_settings();
                cx.notify();
            });
        })
        .child(
            div()
                .id("settings_modal_card")
                .w(px(760.0))
                .h(px(520.0))
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.panel_border)
                .rounded_xl()
                .shadow_2xl()
                .flex()
                .flex_col()
                .overflow_hidden()
                .on_scroll_wheel(|_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                // 1. 顶部标题栏 (Header)
                .child(render_settings_header(state, &model_close, &pal))
                // 2. 中间主体区 (左侧分类栏 + 右侧配置面板)
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .overflow_hidden()
                        // 左侧分类导航栏
                        .child(render_settings_category_sidebar(state, model, current_tab, &pal))
                        // 右侧各 Tab 独立配置视图
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .p_5()
                                .overflow_hidden()
                                .child(match current_tab {
                                    SettingsTab::General => {
                                        render_general_tab(state, model).into_any_element()
                                    }
                                    SettingsTab::Canvas => {
                                        render_canvas_tab(state, model).into_any_element()
                                    }
                                    SettingsTab::Performance => {
                                        render_performance_tab(state, model).into_any_element()
                                    }
                                    SettingsTab::Game => {
                                        render_game_tab(state, model).into_any_element()
                                    }
                                }),
                        ),
                )
                // 3. 底部操作按钮栏 (Footer: 恢复默认 + 完成)
                .child(render_settings_footer(state, &model_reset, &model_done, &pal)),
        )
        .into_any_element()
}

/// 渲染设置窗口标题栏
fn render_settings_header(
    state: &AppState,
    model_close: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let m = model_close.clone();

    div()
        .h(px(52.0))
        .px_5()
        .bg(pal.app_bg)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(div().text_size(px(16.0)).child("⚙️"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.5))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.title")),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.subtitle")),
                        ),
                ),
        )
        .child(
            div()
                .size(px(24.0))
                .rounded_md()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_size(px(12.0))
                .text_color(pal.text_secondary)
                .hover(|s| s.bg(pal.card_surface).text_color(pal.text_primary))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    m.update(cx, |st, cx| {
                        st.close_settings();
                        cx.notify();
                    });
                })
                .child("✕"),
        )
}

/// 渲染左侧分类选项卡列表
fn render_settings_category_sidebar(
    state: &AppState,
    model: &Entity<AppState>,
    current_tab: SettingsTab,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let categories = [
        (SettingsTab::General, "🌐", state.t("settings.tab_general")),
        (SettingsTab::Canvas, "🎨", state.t("settings.tab_canvas")),
        (SettingsTab::Performance, "⚡", state.t("settings.tab_performance")),
        (SettingsTab::Game, "🎮", state.t("settings.tab_game")),
    ];

    div()
        .w(px(190.0))
        .h_full()
        .bg(pal.app_bg)
        .border_r_1()
        .border_color(pal.panel_border)
        .p_3()
        .flex()
        .flex_col()
        .gap_1p5()
        .children(categories.iter().map(|&(tab, icon, label)| {
            let is_active = current_tab == tab;
            let m = model.clone();
            let bg_color = if is_active {
                pal.primary_accent.opacity(0.18)
            } else {
                gpui::rgba(0x00000000)
            };
            let text_color = if is_active {
                pal.primary_accent
            } else {
                pal.text_secondary
            };

            div()
                .h(px(36.0))
                .px_3()
                .rounded_lg()
                .bg(bg_color)
                .flex()
                .items_center()
                .justify_between()
                .cursor_pointer()
                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    m.update(cx, |st, cx| {
                        st.settings_active_tab = tab;
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2p5()
                        .child(div().text_size(px(14.0)).child(icon))
                        .child(
                            div()
                                .font_weight(if is_active {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_size(px(12.0))
                                .text_color(text_color)
                                .child(label.to_string()),
                        ),
                )
                .child(
                    if is_active {
                        div()
                            .size(px(5.0))
                            .rounded_full()
                            .bg(pal.primary_accent)
                    } else {
                        div()
                    },
                )
        }))
}

/// 渲染底部操作栏 (恢复默认 + 完成)
fn render_settings_footer(
    state: &AppState,
    model_reset: &Entity<AppState>,
    model_done: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let m_reset = model_reset.clone();
    let m_done = model_done.clone();

    div()
        .h(px(48.0))
        .px_5()
        .bg(pal.app_bg)
        .border_t_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        // 恢复默认按钮
        .child(
            div()
                .h(px(28.0))
                .px_3()
                .rounded_md()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.card_border)
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_size(px(11.5))
                .text_color(pal.text_secondary)
                .hover(|s| {
                    s.bg(pal.card_hover)
                        .text_color(pal.text_primary)
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    m_reset.update(cx, |st, cx| {
                        st.reset_settings_to_default();
                        cx.notify();
                    });
                })
                .child(state.t("settings.reset_defaults")),
        )
        // 完成按钮
        .child(
            div()
                .h(px(28.0))
                .px_4()
                .rounded_md()
                .bg(pal.primary_accent)
                .border_1()
                .border_color(pal.primary_accent_hover)
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_size(px(12.0))
                .font_weight(FontWeight::BOLD)
                .text_color(if pal.is_dark { rgb(0x0A0C10) } else { rgb(0xFFFFFF) })
                .hover(|s| s.bg(pal.primary_accent_hover).shadow_md())
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    m_done.update(cx, |st, cx| {
                        st.close_settings();
                        cx.notify();
                    });
                })
                .child(state.t("settings.close")),
        )
}
