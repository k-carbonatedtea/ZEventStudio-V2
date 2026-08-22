use gpui::*;
use crate::ui::state::{AppState, WelcomeTab};

pub mod quick_tab;
pub mod recent_tab;

/// 渲染全屏启动欢迎界面 (Event Studio Welcome & Launcher View)
pub fn render_welcome_view(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_tab_quick = model.clone();
    let model_tab_recent = model.clone();
    let is_quick = matches!(state.welcome_tab, WelcomeTab::QuickStart);
    let recent_count = state.recent_projects.len();

    div()
        .id("welcome_view_container")
        .w_full()
        .h_full()
        .bg(hsla(220.0 / 360.0, 0.40, 0.05, 1.0))
        .flex()
        .items_center()
        .justify_center()
        .p_6()
        // 居中毛玻璃卡片主体
        .child(
            div()
                .w(px(540.0))
                .rounded_2xl()
                .bg(hsla(220.0 / 360.0, 0.35, 0.09, 0.96))
                .border_1()
                .border_color(hsla(215.0 / 360.0, 0.30, 0.20, 1.0))
                .shadow_2xl()
                .p_6()
                .flex()
                .flex_col()
                .gap_4()
                // 顶部 Logo 徽标与标题区
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .text_center()
                        .gap_1p5()
                        // 图标徽标
                        .child(
                            div()
                                .w(px(64.0))
                                .h(px(64.0))
                                .rounded_2xl()
                                .bg(hsla(215.0 / 360.0, 0.60, 0.22, 1.0))
                                .border_1()
                                .border_color(hsla(215.0 / 360.0, 0.70, 0.45, 1.0))
                                .shadow_lg()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(30.0))
                                .child("📜"),
                        )
                        // 大标题
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(22.0))
                                .text_color(hsla(0.0, 0.0, 1.0, 0.98))
                                .child(state.t("empty.title")),
                        )
                        // 副标题描述
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(hsla(215.0 / 360.0, 0.20, 0.60, 1.0))
                                .child(state.t("empty.subtitle")),
                        ),
                )
                // 切换标签分段栏 (Segmented Tabs)
                .child(
                    div()
                        .w_full()
                        .p_1()
                        .rounded_lg()
                        .bg(hsla(220.0 / 360.0, 0.40, 0.06, 1.0))
                        .border_1()
                        .border_color(hsla(220.0 / 360.0, 0.25, 0.16, 1.0))
                        .flex()
                        .gap_1()
                        .child(
                            div()
                                .flex_1()
                                .py_1p5()
                                .rounded_md()
                                .bg(if is_quick {
                                    hsla(215.0 / 360.0, 0.60, 0.35, 1.0)
                                } else {
                                    hsla(0.0, 0.0, 0.0, 0.0)
                                })
                                .hover(|s| {
                                    if !is_quick {
                                        s.bg(hsla(220.0 / 360.0, 0.25, 0.14, 1.0))
                                    } else {
                                        s
                                    }
                                })
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_1p5()
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if is_quick {
                                    hsla(0.0, 0.0, 1.0, 1.0)
                                } else {
                                    hsla(215.0 / 360.0, 0.20, 0.65, 1.0)
                                })
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_tab_quick.update(cx, |st, cx| {
                                        st.welcome_tab = WelcomeTab::QuickStart;
                                        cx.notify();
                                    });
                                })
                                .child("⚡")
                                .child(state.t("empty.quickStartTab")),
                        )
                        .child(
                            div()
                                .flex_1()
                                .py_1p5()
                                .rounded_md()
                                .bg(if !is_quick {
                                    hsla(215.0 / 360.0, 0.60, 0.35, 1.0)
                                } else {
                                    hsla(0.0, 0.0, 0.0, 0.0)
                                })
                                .hover(|s| {
                                    if is_quick {
                                        s.bg(hsla(220.0 / 360.0, 0.25, 0.14, 1.0))
                                    } else {
                                        s
                                    }
                                })
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_1p5()
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if !is_quick {
                                    hsla(0.0, 0.0, 1.0, 1.0)
                                } else {
                                    hsla(215.0 / 360.0, 0.20, 0.65, 1.0)
                                })
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_tab_recent.update(cx, |st, cx| {
                                        st.welcome_tab = WelcomeTab::Recent;
                                        cx.notify();
                                    });
                                })
                                .child("🕒")
                                .child(state.t("empty.recentProjectsTab"))
                                .children(if recent_count > 0 {
                                    Some(
                                        div()
                                            .px_1p5()
                                            .py_0p5()
                                            .rounded_full()
                                            .bg(hsla(220.0 / 360.0, 0.40, 0.18, 1.0))
                                            .text_size(px(10.0))
                                            .child(recent_count.to_string()),
                                    )
                                } else {
                                    None
                                }),
                        ),
                )
                // 标签内容面板
                .child(
                    div().w_full().child(if is_quick {
                        quick_tab::render_quick_start_tab(state, model).into_any_element()
                    } else {
                        recent_tab::render_recent_projects_tab(state, model).into_any_element()
                    }),
                )
                // 拖拽提示条
                .child(
                    div()
                        .w_full()
                        .py_2()
                        .px_3()
                        .rounded_lg()
                        .bg(hsla(220.0 / 360.0, 0.30, 0.07, 0.8))
                        .border_1()
                        .border_color(hsla(220.0 / 360.0, 0.25, 0.16, 0.6))
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .text_size(px(11.0))
                        .text_color(hsla(215.0 / 360.0, 0.20, 0.50, 1.0))
                        .child("☁️")
                        .child(state.t("empty.dragHint")),
                )
                // 底部使用说明链接
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .hover(|s| s.text_color(hsla(215.0 / 360.0, 0.70, 0.65, 1.0)))
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(hsla(215.0 / 360.0, 0.20, 0.55, 1.0))
                                .child("📖")
                                .child(state.t("empty.viewGuide")),
                        ),
                ),
        )
}
