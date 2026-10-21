use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    menu::{DropdownMenu, PopupMenu, PopupMenuItem},
    Sizable,
};
use crate::ui::state::{AppState, AppWorkspaceTab};

/// 渲染「查看 (View)」下拉菜单
pub fn render_view_menu(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let model_welcome = model.clone();
    let model_flowchart = model.clone();
    let model_code = model.clone();
    let model_actors = model.clone();
    let model_events = model.clone();
    let model_plugins = model.clone();
    let model_sidebar = model.clone();
    let model_inspector = model.clone();
    let model_fit = model.clone();
    let model_zoom_in = model.clone();
    let model_zoom_out = model.clone();
    let model_reset_zoom = model.clone();

    let label_welcome = state.t("menu.view_welcome").to_string();
    let label_flowchart = state.t("menu.view_flowchart").to_string();
    let label_code = state.t("menu.view_code_editor").to_string();
    let label_actors = state.t("menu.view_actors").to_string();
    let label_events = state.t("menu.view_events").to_string();
    let label_plugins = state.t("menu.view_plugins").to_string();
    let label_sidebar = state.t("menu.toggle_sidebar").to_string();
    let label_inspector = state.t("menu.toggle_inspector").to_string();
    let label_fit = state.t("menu.fit_view").to_string();
    let label_zoom_in = state.t("menu.zoom_in").to_string();
    let label_zoom_out = state.t("menu.zoom_out").to_string();
    let label_reset_zoom = state.t("menu.reset_zoom").to_string();

    let model_theme = model.clone();
    let cur_theme = state.current_theme;
    let theme_items = crate::ui::theme::AppTheme::ALL.map(|t| {
        (
            t,
            format!(
                "{} {}",
                if cur_theme == t { "✓" } else { "   " },
                state.t(t.i18n_key())
            ),
        )
    });

    Button::new("titlebar_menu_view")
        .ghost()
        .xsmall()
        .label(state.t("menu.view"))
        .dropdown_menu(move |mut menu: PopupMenu, _window, _cx| {
            let m_wel = model_welcome.clone();
            let m_fc = model_flowchart.clone();
            let m_cd = model_code.clone();
            let m_act = model_actors.clone();
            let m_ev = model_events.clone();
            let m_plg = model_plugins.clone();
            let m_sb = model_sidebar.clone();
            let m_ins = model_inspector.clone();
            let m_fit = model_fit.clone();
            let m_zi = model_zoom_in.clone();
            let m_zo = model_zoom_out.clone();
            let m_rz = model_reset_zoom.clone();
            let m_th = model_theme.clone();

            menu = menu
                .min_w(px(220.0))
                .item(
                    PopupMenuItem::new(label_welcome.clone())
                        .on_click(move |_, _, cx| {
                            m_wel.update(cx, |st, cx| {
                                st.current_tab = AppWorkspaceTab::Welcome;
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_flowchart.clone())
                        .on_click(move |_, _, cx| {
                            m_fc.update(cx, |st, cx| {
                                st.switch_workspace_tab(AppWorkspaceTab::Flowchart);
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_code.clone())
                        .on_click(move |_, _, cx| {
                            m_cd.update(cx, |st, cx| {
                                st.switch_workspace_tab(AppWorkspaceTab::CodeEditor);
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_actors.clone())
                        .on_click(move |_, _, cx| {
                            m_act.update(cx, |st, cx| {
                                st.current_tab = AppWorkspaceTab::Actors;
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_events.clone())
                        .on_click(move |_, _, cx| {
                            m_ev.update(cx, |st, cx| {
                                st.current_tab = AppWorkspaceTab::Events;
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_plugins.clone())
                        .on_click(move |_, _, cx| {
                            m_plg.update(cx, |st, cx| {
                                st.active_activity_tab = crate::ui::state::ActivityBarTab::Plugins;
                                st.sidebar_open = true;
                                cx.notify();
                            });
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new(label_sidebar.clone())
                        .on_click(move |_, _, cx| {
                            m_sb.update(cx, |st, cx| {
                                st.sidebar_open = !st.sidebar_open;
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_inspector.clone())
                        .on_click(move |_, _, cx| {
                            m_ins.update(cx, |st, cx| {
                                st.inspector_open = !st.inspector_open;
                                cx.notify();
                            });
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new(label_fit.clone())
                        .on_click(move |_, _, cx| {
                            m_fit.update(cx, |st, cx| {
                                st.fit_view();
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_zoom_in.clone())
                        .on_click(move |_, _, cx| {
                            m_zi.update(cx, |st, cx| {
                                st.zoom = (st.zoom * 1.25).min(5.0);
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_zoom_out.clone())
                        .on_click(move |_, _, cx| {
                            m_zo.update(cx, |st, cx| {
                                st.zoom = (st.zoom / 1.25).max(0.01);
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_reset_zoom.clone())
                        .on_click(move |_, _, cx| {
                            m_rz.update(cx, |st, cx| {
                                st.zoom = 1.0;
                                cx.notify();
                            });
                        }),
                )
                .separator();

            for &(target_theme, ref label) in &theme_items {
                let m = m_th.clone();
                let l = label.clone();
                menu = menu.item(
                    PopupMenuItem::new(l).on_click(move |_, _, cx| {
                        m.update(cx, |st, cx| {
                            st.set_theme(target_theme, cx);
                            cx.notify();
                        });
                    }),
                );
            }

            menu
        })
}
