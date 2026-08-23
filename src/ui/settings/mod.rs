pub mod canvas_helpers;
pub mod canvas_pane;
pub mod general_helpers;
pub mod general_pane;
pub mod llm_helpers;
pub mod llm_pane;
pub mod switch_pane;
pub mod wiiu_pane;
pub mod header;
pub mod sidebar;
pub mod footer;

use gpui::*;
use canvas_pane::render_canvas_pane;
use general_pane::render_general_pane;
use llm_pane::render_llm_pane;
use switch_pane::render_switch_pane;
use wiiu_pane::render_wiiu_pane;
use header::render_settings_header;
use sidebar::render_settings_category_sidebar;
use footer::render_settings_footer;
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
        .on_mouse_down(MouseButton::Left, {
            let m_bg = model.clone();
            move |_, _, cx| {
                cx.stop_propagation();
                m_bg.update(cx, |st, cx| {
                    if st.active_settings_field != crate::ui::state::ActiveSettingsField::None {
                        st.active_settings_field = crate::ui::state::ActiveSettingsField::None;
                        cx.notify();
                    }
                });
            }
        })
        .on_mouse_down(MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_move({
            let m_card_move = model.clone();
            move |e, _window, cx| {
                let mx: f32 = e.position.x.into();
                let my: f32 = e.position.y.into();
                m_card_move.update(cx, |st, cx| {
                    if let Some(((sx, sy), (w0, h0))) = st.settings_resize_start {
                        let delta_x = (mx - sx) * 2.0;
                        let delta_y = (my - sy) * 2.0;
                        st.settings_modal_width = (w0 + delta_x).clamp(680.0, 1600.0);
                        st.settings_modal_height = (h0 + delta_y).clamp(460.0, 1000.0);
                        st.settings_modal_maximized = false;
                        cx.notify();
                    }
                });
            }
        })
        .on_mouse_up(MouseButton::Left, {
            let m_card_up = model.clone();
            move |_, _, cx| {
                cx.stop_propagation();
                m_card_up.update(cx, |st, cx| {
                    if st.settings_resize_start.is_some() || st.is_resizing_settings {
                        st.settings_resize_start = None;
                        st.is_resizing_settings = false;
                        cx.notify();
                    }
                });
            }
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
        .on_mouse_move({
            let m_overlay_move = model.clone();
            move |e, _window, cx| {
                let mx: f32 = e.position.x.into();
                let my: f32 = e.position.y.into();
                m_overlay_move.update(cx, |st, cx| {
                    if let Some(((sx, sy), (w0, h0))) = st.settings_resize_start {
                        let delta_x = (mx - sx) * 2.0;
                        let delta_y = (my - sy) * 2.0;
                        st.settings_modal_width = (w0 + delta_x).clamp(680.0, 1600.0);
                        st.settings_modal_height = (h0 + delta_y).clamp(460.0, 1000.0);
                        st.settings_modal_maximized = false;
                        cx.notify();
                    }
                });
            }
        })
        .on_mouse_up(MouseButton::Left, {
            let m_overlay_up = model.clone();
            move |_, _, cx| {
                cx.stop_propagation();
                m_overlay_up.update(cx, |st, cx| {
                    if st.settings_resize_start.is_some() || st.is_resizing_settings {
                        st.settings_resize_start = None;
                        st.is_resizing_settings = false;
                        cx.notify();
                    }
                });
            }
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
                                    SettingsTab::Llm => {
                                        render_llm_pane(state, model).into_any_element()
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
