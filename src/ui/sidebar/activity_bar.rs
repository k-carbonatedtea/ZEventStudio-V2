use gpui::*;
use crate::ui::state::{ActivityBarTab, AppState};

/// 渲染 VS Code 风格的垂直活动栏 (Activity Bar, 宽度 48px)
/// 包含：资源管理器、全局搜索、源文件管理 (带未保存标记徽标) 以及底部设置齿轮
pub fn render_activity_bar(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let current_tab = state.active_activity_tab;
    let sidebar_open = state.sidebar_open;
    let modified_count = state.modified_tabs_count();

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

        // 顶部核心功能图标组 (资源管理器 / 搜索 / 源文件管理)
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
                    |color, pal| render_explorer_icon(color, pal),
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
                    modified_count,
                    |color, pal| render_source_control_icon(color, pal),
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
                            st.global_search_focused = false;
                        } else {
                            st.active_activity_tab = ActivityBarTab::Search;
                            st.sidebar_open = true;
                            st.global_search_focused = true;
                            st.file_search_focused = false;
                            st.update_global_search_results();
                        }
                    }
                    ActivityBarTab::SourceControl => {
                        if st.active_activity_tab == ActivityBarTab::SourceControl && st.sidebar_open {
                            st.sidebar_open = false;
                        } else {
                            st.active_activity_tab = ActivityBarTab::SourceControl;
                            st.sidebar_open = true;
                            st.sync_active_tab_modified();
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

        // 右上角角标徽章 (如源文件管理未保存文件数量)
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

/// 渲染资源管理器矢量图标 (VS Code 风格重叠双文件页)
fn render_explorer_icon(color: Rgba, pal: &crate::ui::theme::ThemePalette) -> AnyElement {
    div()
        .size(px(22.0))
        .relative()
        // 底层文件页
        .child(
            div()
                .absolute()
                .top(px(1.5))
                .left(px(1.5))
                .w(px(13.0))
                .h(px(15.0))
                .rounded(px(1.5))
                .border_1()
                .border_color(color),
        )
        // 顶层文件页
        .child(
            div()
                .absolute()
                .top(px(5.0))
                .left(px(5.5))
                .w(px(14.0))
                .h(px(15.0))
                .bg(pal.panel_surface)
                .rounded(px(1.5))
                .border_1()
                .border_color(color),
        )
        // 顶层右上角折角切线
        .child(
            div()
                .absolute()
                .top(px(5.0))
                .right(px(2.5))
                .size(px(4.0))
                .border_b_1()
                .border_l_1()
                .border_color(color),
        )
        .into_any_element()
}

/// 渲染全局搜索矢量图标 (VS Code 风格 45度倾斜放大镜)
fn render_search_icon(color: Rgba) -> AnyElement {
    div()
        .size(px(22.0))
        .relative()
        // 放大镜圆形镜面
        .child(
            div()
                .absolute()
                .top(px(2.0))
                .left(px(2.0))
                .size(px(12.5))
                .rounded_full()
                .border_1()
                .border_color(color),
        )
        // 放大镜 45 度斜手柄 (平滑步进像素)
        .child(
            div()
                .absolute()
                .top(px(12.0))
                .left(px(12.0))
                .size(px(2.5))
                .rounded_xs()
                .bg(color),
        )
        .child(
            div()
                .absolute()
                .top(px(14.0))
                .left(px(14.0))
                .size(px(2.5))
                .rounded_xs()
                .bg(color),
        )
        .child(
            div()
                .absolute()
                .top(px(16.0))
                .left(px(16.0))
                .size(px(2.5))
                .rounded_xs()
                .bg(color),
        )
        .child(
            div()
                .absolute()
                .top(px(18.0))
                .left(px(18.0))
                .size(px(2.5))
                .rounded_xs()
                .bg(color),
        )
        .into_any_element()
}

/// 渲染源文件管理矢量图标 (VS Code 风格 Git 分支图与 3 个节点)
fn render_source_control_icon(color: Rgba, pal: &crate::ui::theme::ThemePalette) -> AnyElement {
    div()
        .size(px(22.0))
        .relative()
        // 主干坚线
        .child(
            div()
                .absolute()
                .top(px(5.5))
                .left(px(5.25))
                .w(px(1.5))
                .h(px(11.0))
                .bg(color),
        )
        // 主干顶部节点
        .child(
            div()
                .absolute()
                .top(px(2.5))
                .left(px(3.25))
                .size(px(5.5))
                .rounded_full()
                .border_1()
                .border_color(color)
                .bg(pal.panel_surface),
        )
        // 主干底部节点
        .child(
            div()
                .absolute()
                .top(px(14.5))
                .left(px(3.25))
                .size(px(5.5))
                .rounded_full()
                .border_1()
                .border_color(color)
                .bg(pal.panel_surface),
        )
        // 分支横向折线
        .child(
            div()
                .absolute()
                .top(px(10.5))
                .left(px(6.0))
                .w(px(6.5))
                .h(px(1.5))
                .bg(color),
        )
        // 分支向上竖折线
        .child(
            div()
                .absolute()
                .top(px(7.5))
                .left(px(12.5))
                .w(px(1.5))
                .h(px(4.0))
                .bg(color),
        )
        // 分支右侧节点
        .child(
            div()
                .absolute()
                .top(px(4.0))
                .left(px(10.5))
                .size(px(5.5))
                .rounded_full()
                .border_1()
                .border_color(color)
                .bg(pal.panel_surface),
        )
        .into_any_element()
}
