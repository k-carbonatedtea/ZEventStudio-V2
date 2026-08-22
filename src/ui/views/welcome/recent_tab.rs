use gpui::*;
use crate::ui::components::Toast;
use crate::ui::state::{AppState, AppWorkspaceTab, RecentProjectItem};

#[allow(dead_code)]
/// 渲染欢迎启动页的「最近打开的文件与模组」列表面板
pub fn render_recent_projects_tab(

    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let recent_count = state.recent_projects.len();
    let model_clear = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_2()
        .children(if state.recent_projects.is_empty() {
            Some(
                div()
                    .py_8()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .child(div().text_size(px(28.0)).text_color(hsla(0.0, 0.0, 0.5, 1.0)).child("🔍"))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(hsla(215.0 / 360.0, 0.20, 0.50, 1.0))
                            .child(state.t("empty.noRecentProjects")),
                    ),
            )
        } else {
            None
        })
        .children(if !state.recent_projects.is_empty() {
            Some(
                div()
                    .id("recent_projects_scroll")
                    .max_h(px(220.0))
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .children(state.recent_projects.iter().enumerate().map(|(idx, item)| {
                        render_recent_item_row(item, idx, model)
                    })),
            )
        } else {
            None
        })
        // 底部统计与清空按钮
        .children(if recent_count > 0 {
            Some(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .pt_1()
                    .border_t_1()
                    .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(hsla(215.0 / 360.0, 0.20, 0.55, 1.0))
                            .child(format!("共 {} 个记录", recent_count)),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .bg(hsla(0.0, 0.50, 0.25, 0.6))
                            .hover(|s| s.bg(hsla(0.0, 0.70, 0.35, 0.9)))
                            .cursor_pointer()
                            .text_size(px(10.5))
                            .text_color(hsla(0.0, 0.0, 0.90, 1.0))
                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                model_clear.update(cx, |st, cx| {
                                    st.recent_projects.clear();
                                    cx.notify();
                                });
                            })
                            .child("🗑️ 清空记录"),
                    ),
            )
        } else {
            None
        })
}

fn render_recent_item_row(
    item: &RecentProjectItem,
    idx: usize,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_open = model.clone();
    let model_del = model.clone();
    let item_clone = item.clone();

    let icon_str = match item.project_type.as_str() {
        "mod" => "🗂️",
        "bfevfl" => "⚡",
        "sbeventpack" | "pack" => "📦",
        "msbt" => "💬",
        _ => "📄",
    };

    div()
        .px_3()
        .py_2()
        .rounded_md()
        .bg(hsla(220.0 / 360.0, 0.35, 0.12, 0.9))
        .border_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
        .hover(|s| {
            s.bg(hsla(215.0 / 360.0, 0.40, 0.18, 0.98))
                .border_color(hsla(215.0 / 360.0, 0.60, 0.40, 1.0))
        })
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let item_data = item_clone.clone();
            model_open.update(cx, |st, cx| {
                if !item_data.path.is_empty() {
                    st.load_file_by_path(&item_data.path);
                }
                st.fit_view();
                st.current_tab = AppWorkspaceTab::Flowchart;
                st.toast = Some(Toast::success(format!("📂 已加载: {}", item_data.name)));
                cx.notify();
            });
        })

        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                // 类型图标
                .child(
                    div()
                        .w(px(26.0))
                        .h(px(26.0))
                        .rounded_md()
                        .bg(hsla(220.0 / 360.0, 0.35, 0.20, 1.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(14.0))
                        .child(icon_str),
                )
                // 名称与路径
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(12.0))
                                        .text_color(hsla(0.0, 0.0, 1.0, 0.95))
                                        .child(item.name.clone()),
                                )
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_md()
                                        .bg(hsla(200.0 / 360.0, 0.40, 0.25, 1.0))
                                        .text_size(px(9.5))
                                        .text_color(hsla(200.0 / 360.0, 0.80, 0.75, 1.0))
                                        .child(item.project_type.to_uppercase()),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .text_color(hsla(215.0 / 360.0, 0.20, 0.50, 1.0))
                                .child(item.path.clone()),
                        ),
                ),
        )
        // 右侧删除记录按钮
        .child(
            div()
                .w(px(22.0))
                .h(px(22.0))
                .rounded_md()
                .hover(|s| s.bg(hsla(0.0, 0.70, 0.35, 0.9)))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_del.update(cx, |st, cx| {
                        if idx < st.recent_projects.len() {
                            st.recent_projects.remove(idx);
                        }
                        cx.notify();
                    });
                })
                .child("✕"),
        )
}
