pub mod canvas_pane;
pub mod general_pane;
pub mod switch_pane;
pub mod wiiu_pane;

use gpui::*;
use canvas_pane::render_canvas_pane;
use general_pane::render_general_pane;
use switch_pane::render_switch_pane;
use wiiu_pane::render_wiiu_pane;
use crate::model::PlatformType;
use crate::ui::state::AppState;
use crate::ui::state_settings::SettingsTab;

/// 渲染现代化首选项与游戏路径设置模态窗口 (Preferences & Game Settings Modal)
pub fn render_settings_modal(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    if !state.settings_modal_open {
        return div().into_any_element();
    }

    let pal = state.theme();
    let current_tab = state.settings_active_tab;
    let is_maximized = state.settings_modal_maximized;
    let modal_w = state.settings_modal_width;
    let modal_h = state.settings_modal_height;

    let model_backdrop = model.clone();
    let model_header = model.clone();
    let model_reset = model.clone();
    let model_cancel = model.clone();
    let model_save = model.clone();
    let model_resize = model.clone();

    // 模态弹窗外层全屏半透明遮罩
    let mut card = div()
        .id("settings_modal_card")
        .bg(pal.panel_surface)
        .border_1()
        .border_color(pal.panel_border)
        .rounded_xl()
        .shadow_2xl()
        .flex()
        .flex_col()
        .overflow_hidden()
        .relative()
        .on_scroll_wheel(|_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_down(MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_click(|_, _, cx| {
            cx.stop_propagation();
        });

    if is_maximized {
        card = card.size_full();
    } else {
        card = card
            .w(px(modal_w))
            .h(px(modal_h))
            .min_w(px(680.0))
            .min_h(px(460.0));
    }

    div()
        .id("settings_modal_overlay")
        .absolute()
        .inset_0()
        .occlude()
        .bg(hsla(0.0, 0.0, 0.0, 0.68))
        .flex()
        .items_center()
        .justify_center()
        .p_4()
        .on_scroll_wheel(|_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            model_backdrop.update(cx, |st, cx| {
                st.close_settings();
                cx.notify();
            });
        })
        .on_mouse_down(MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_click(|_, _, cx| {
            cx.stop_propagation();
        })
        .child(
            card
                // 1. 顶部标题栏与窗口控制按钮 (Header)
                .child(render_settings_header(state, &model_header, &pal, is_maximized))
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
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .p_5()
                                .overflow_hidden()
                                .child(match current_tab {
                                    SettingsTab::General => {
                                        render_general_pane(state, model).into_any_element()
                                    }
                                    SettingsTab::Switch => {
                                        render_switch_pane(state, model).into_any_element()
                                    }
                                    SettingsTab::WiiU => {
                                        render_wiiu_pane(state, model).into_any_element()
                                    }
                                    SettingsTab::Canvas => {
                                        render_canvas_pane(state, model).into_any_element()
                                    }
                                }),
                        ),
                )
                // 3. 底部操作按钮栏与角标拖拽缩放手柄 (Footer)
                .child(render_settings_footer(
                    state,
                    &model_reset,
                    &model_cancel,
                    &model_save,
                    &model_resize,
                    &pal,
                    is_maximized,
                )),
        )
        .into_any_element()
}

/// 渲染设置窗口标题栏
fn render_settings_header(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
    is_maximized: bool,
) -> impl IntoElement {
    let m_max = model.clone();
    let m_close = model.clone();

    div()
        .h(px(52.0))
        .px_5()
        .bg(pal.app_bg)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .items_center()
                .gap_2p5()
                .child(div().flex_shrink_0().text_size(px(17.0)).child("⚙️"))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
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
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap_1p5()
                // 最大化 / 还原按钮
                .child(
                    div()
                        .size(px(26.0))
                        .rounded_md()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .text_size(px(12.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_surface).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            m_max.update(cx, |st, cx| {
                                st.settings_modal_maximized = !st.settings_modal_maximized;
                                cx.notify();
                            });
                        })
                        .child(if is_maximized { "🗗" } else { "🗖" }),
                )
                // 关闭按钮
                .child(
                    div()
                        .size(px(26.0))
                        .rounded_md()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .text_size(px(13.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_surface).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            m_close.update(cx, |st, cx| {
                                st.close_settings();
                                cx.notify();
                            });
                        })
                        .child("✕"),
                ),
        )
}

/// 渲染左侧分类选项卡列表
fn render_settings_category_sidebar(
    state: &AppState,
    model: &Entity<AppState>,
    current_tab: SettingsTab,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let cur_plat = state.settings_draft.current_platform;
    let categories = [
        (SettingsTab::General, "⚙️", state.t("settings.tab_general"), false),
        (
            SettingsTab::Switch,
            "🎮",
            state.t("settings.tab_switch"),
            cur_plat == PlatformType::Switch,
        ),
        (
            SettingsTab::WiiU,
            "🖥️",
            state.t("settings.tab_wiiu"),
            cur_plat == PlatformType::WiiU,
        ),
        (SettingsTab::Canvas, "🎨", state.t("settings.tab_canvas"), false),
    ];

    div()
        .w(px(215.0))
        .h_full()
        .bg(pal.app_bg)
        .border_r_1()
        .border_color(pal.panel_border)
        .p_3()
        .flex()
        .flex_col()
        .gap_1p5()
        .children(categories.iter().map(|&(tab, icon, label, is_active_plat)| {
            let is_selected = current_tab == tab;
            let m = model.clone();
            let bg_color = if is_selected {
                pal.primary_accent.opacity(0.18)
            } else {
                gpui::rgba(0x00000000)
            };
            let text_color = if is_selected {
                pal.primary_accent
            } else {
                pal.text_secondary
            };

            div()
                .h(px(38.0))
                .px_2p5()
                .rounded_lg()
                .bg(bg_color)
                .flex()
                .items_center()
                .justify_between()
                .gap_1p5()
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
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().flex_shrink_0().text_size(px(14.0)).child(icon))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .font_weight(if is_selected {
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
                    if is_active_plat {
                        div()
                            .flex_shrink_0()
                            .px_1p5()
                            .py_0p5()
                            .rounded_sm()
                            .bg(pal.primary_accent.opacity(0.15))
                            .text_size(px(9.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(pal.primary_accent)
                            .child("Active")
                            .into_any_element()
                    } else if is_selected {
                        div()
                            .flex_shrink_0()
                            .size(px(5.0))
                            .rounded_full()
                            .bg(pal.primary_accent)
                            .into_any_element()
                    } else {
                        div().into_any_element()
                    },
                )
        }))
}

/// 渲染底部操作栏 (恢复默认 + 取消 + 保存并应用 + 拖拽调宽手柄)
fn render_settings_footer(
    state: &AppState,
    model_reset: &Entity<AppState>,
    model_cancel: &Entity<AppState>,
    model_save: &Entity<AppState>,
    model_resize: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
    is_maximized: bool,
) -> impl IntoElement {
    let m_reset = model_reset.clone();
    let m_cancel = model_cancel.clone();
    let m_save = model_save.clone();
    let m_resize = model_resize.clone();

    div()
        .h(px(52.0))
        .px_5()
        .bg(pal.app_bg)
        .border_t_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .relative()
        // 恢复默认按钮
        .child(
            div()
                .h(px(30.0))
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
        // 右侧操作按钮组 (取消 + 保存并应用)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                // 取消按钮
                .child(
                    div()
                        .h(px(30.0))
                        .px_3p5()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.card_border)
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .text_size(px(12.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| {
                            s.bg(pal.card_hover)
                                .text_color(pal.text_primary)
                        })
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            m_cancel.update(cx, |st, cx| {
                                st.close_settings();
                                cx.notify();
                            });
                        })
                        .child(state.t("settings.cancel_btn")),
                )
                // 保存并应用按钮
                .child(
                    div()
                        .h(px(30.0))
                        .px_4()
                        .rounded_md()
                        .bg(pal.primary_accent)
                        .border_1()
                        .border_color(pal.primary_accent_hover)
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_1p5()
                        .cursor_pointer()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if pal.is_dark { rgb(0x0A0C10) } else { rgb(0xFFFFFF) })
                        .hover(|s| s.bg(pal.primary_accent_hover).shadow_md())
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            m_save.update(cx, |st, cx| {
                                st.save_and_apply_settings();
                                cx.notify();
                            });
                        })
                        .child(div().text_size(px(12.0)).child("💾"))
                        .child(state.t("settings.save_btn")),
                ),
        )
        // 右下角弹窗缩放手柄 (非最大化状态下可自由拖拽调节窗口宽高)
        .child(
            if !is_maximized {
                div()
                    .absolute()
                    .bottom_1()
                    .right_1()
                    .size(px(14.0))
                    .flex()
                    .items_end()
                    .justify_end()
                    .cursor_nwse_resize()
                    .on_mouse_down(MouseButton::Left, move |e, _window, cx| {
                        cx.stop_propagation();
                        let mx: f32 = e.position.x.into();
                        let my: f32 = e.position.y.into();
                        m_resize.update(cx, |st, cx| {
                            st.is_resizing_settings = true;
                            st.settings_resize_start = Some(((mx, my), (st.settings_modal_width, st.settings_modal_height)));
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(pal.text_secondary.opacity(0.6))
                            .child("⤡"),
                    )
                    .into_any_element()
            } else {
                div().into_any_element()
            },
        )
}
