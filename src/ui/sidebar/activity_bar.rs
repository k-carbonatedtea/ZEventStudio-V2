use gpui::*;
use crate::ui::sidebar::activity_bar_icons::*;
use crate::ui::state::{ActivityBarTab, AppState};

/// 渲染 VS Code 风格的垂直活动栏 (Activity Bar, 宽度 48px)
/// 包含：资源管理器、全局搜索、源文件管理、插件中心 以及底部设置齿轮
pub fn render_activity_bar(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let current_tab = state.active_activity_tab;
    let sidebar_open = state.sidebar_open;
    let source_control_badge = state.total_changed_files_count();
    let plugins_badge = state.active_plugins_count();

    let model_settings = model.clone();

    div()
        .id("activity_bar")
        .w(px(48.0))
        .flex_shrink_0()
        .h_full()
        .bg(pal.panel_surface)
        .border_r_1()
        .border_color(pal.panel_border)
        .flex()
        .flex_col()
        .items_center()
        .py_1()
        .justify_between()

        // 顶部核心功能图标组 (资源管理器 / 搜索 / 源文件管理 / 插件中心)
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .w_full()
                .gap_1()

                // 1. 资源管理器 (Explorer)
                .child(render_activity_item(
                    state,
                    model,
                    ActivityBarTab::Explorer,
                    current_tab == ActivityBarTab::Explorer,
                    sidebar_open,
                    0,
                    render_explorer_icon,
                ))

                // 2. 全局搜索 (Search)
                .child(render_activity_item(
                    state,
                    model,
                    ActivityBarTab::Search,
                    current_tab == ActivityBarTab::Search,
                    sidebar_open,
                    0,
                    |color, _pal| render_search_icon(color),
                ))

                // 3. 源文件管理 (Source Control)
                .child(render_activity_item(
                    state,
                    model,
                    ActivityBarTab::SourceControl,
                    current_tab == ActivityBarTab::SourceControl,
                    sidebar_open,
                    source_control_badge,
                    render_source_control_icon,
                ))

                // 4. 插件中心 (Plugins)
                .child(render_activity_item(
                    state,
                    model,
                    ActivityBarTab::Plugins,
                    current_tab == ActivityBarTab::Plugins,
                    sidebar_open,
                    plugins_badge,
                    render_plugins_icon,
                )),
        )

        // 底部设置与首选项管理按钮
        .child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .pb_1()
                .child(
                    div()
                        .id("activity_bar_settings")
                        .size(px(40.0))
                        .rounded_md()
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_settings.update(cx, |st, cx| {
                                st.open_settings(crate::ui::state_settings::SettingsTab::General);
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_size(px(18.0))
                                .child("⚙"),
                        ),
                ),
        )
}

/// 渲染活动栏单个项目按钮
fn render_activity_item<F>(
    state: &AppState,
    model: &Entity<AppState>,
    tab: ActivityBarTab,
    is_active: bool,
    sidebar_open: bool,
    badge_count: usize,
    render_icon: F,
) -> impl IntoElement
where
    F: Fn(Rgba, &crate::ui::theme::ThemePalette) -> AnyElement,
{
    let pal = state.theme();
    let is_highlighted = is_active && sidebar_open;
    let icon_color = if is_highlighted {
        pal.text_primary
    } else {
        pal.text_secondary
    };

    let model_click = model.clone();

    div()
        .w(px(48.0))
        .h(px(44.0))
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |s| {
            if is_highlighted {
                s
            } else {
                s.bg(pal.card_hover)
            }
        })
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_click.update(cx, |st, cx| {
                st.clear_active_text_inputs();
                match tab {
                    ActivityBarTab::Explorer => {
                        if st.active_activity_tab == ActivityBarTab::Explorer && st.sidebar_open {
                            st.sidebar_open = false;
                        } else {
                            st.active_activity_tab = ActivityBarTab::Explorer;
                            st.sidebar_open = true;
                        }
                    }
                    ActivityBarTab::Search => {
                        if st.active_activity_tab == ActivityBarTab::Search && st.sidebar_open {
                            st.sidebar_open = false;
                        } else {
                            st.active_activity_tab = ActivityBarTab::Search;
                            st.sidebar_open = true;
                            st.global_search_focused = true;
                            st.update_global_search_results();
                        }
                    }
                    ActivityBarTab::SourceControl => {
                        if st.active_activity_tab == ActivityBarTab::SourceControl && st.sidebar_open {
                            st.sidebar_open = false;
                        } else {
                            st.active_activity_tab = ActivityBarTab::SourceControl;
                            st.sidebar_open = true;
                            st.refresh_git_status();
                            st.sync_active_tab_modified();
                        }
                    }
                    ActivityBarTab::Plugins => {
                        if st.active_activity_tab == ActivityBarTab::Plugins && st.sidebar_open {
                            st.sidebar_open = false;
                        } else {
                            st.active_activity_tab = ActivityBarTab::Plugins;
                            st.sidebar_open = true;
                        }
                    }
                }
                cx.notify();
            });
        })

        // 左侧激活高亮竖线 (VS Code 经典 2px 遗迹青竖条)
        .children(is_highlighted.then(|| {
            div()
                .absolute()
                .left_0()
                .top(px(8.0))
                .bottom(px(8.0))
                .w(px(2.0))
                .bg(pal.primary_accent)
        }))

        // 图标主体
        .child(render_icon(icon_color, &pal))

        // 右上角角标徽章 (如源文件管理未保存文件数量或已启用插件数量)
        .children((badge_count > 0).then(|| {
            div()
                .absolute()
                .top(px(5.0))
                .right(px(6.0))
                .min_w(px(14.0))
                .h(px(14.0))
                .px_1()
                .rounded_full()
                .bg(pal.primary_accent)
                .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                .text_size(px(8.5))
                .font_weight(FontWeight::BOLD)
                .flex()
                .items_center()
                .justify_center()
                .child(badge_count.to_string())
        }))
}
