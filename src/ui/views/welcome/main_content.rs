use gpui::*;
use crate::ui::components::Toast;
use crate::ui::state::{AppState, AppWorkspaceTab, RecentProjectItem};
use crate::ui::views::welcome::quick_tab::{pick_and_load_file, pick_and_load_mod_folder};
use crate::ui::views::welcome::walkthrough_col::render_walkthrough_section;

/// 渲染 VS Code 风格的右侧欢迎主编辑器内容页面
pub fn render_welcome_main_content(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    div()
        .flex_1()
        .h_full()
        .bg(pal.app_bg) // 动态背景色
        .flex()
        .flex_col()
        // 顶部编辑器选项卡栏 (Tab Strip)
        .child(
            div()
                .h(px(36.0))
                .bg(pal.panel_surface) // 蓝灰表面 Panel Surface
                .border_b_1()
                .border_color(pal.panel_border)
                .flex()
                .items_center()
                .px_2()
                .child(
                    div()
                        .h(px(35.0))
                        .px_3()
                        .bg(pal.card_surface) // 浅层蓝灰 Card / Hover
                        .border_t_2()
                        .border_color(pal.primary_accent) // 遗迹青蓝 Primary Accent
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_primary) // 亮白灰 Text Primary
                        .child("📜")
                        .child(state.t("tab.welcome")),
                ),
        )

        // 欢迎页内容主体滚动区
        .child(
            div()
                .id("welcome_main_scroll")
                .flex_1()
                .overflow_y_scroll()
                .p_8()
                .flex()
                .flex_col()
                .gap_6()
                // 顶部标题
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(26.0))
                                .text_color(pal.text_primary) // 亮白灰 Text Primary
                                .child(state.t("empty.title")),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(pal.text_secondary) // 蓝灰白 Text Secondary
                                .child(state.t("empty.subtitle")),
                        ),
                )
                // 双列网格 (左侧：启动 + 最近；右侧：演练与功能概览)
                .child(
                    div()
                        .flex()
                        .gap_10()
                        // 左列：启动与最近
                        .child(
                            div()
                                .w(px(380.0))
                                .flex()
                                .flex_col()
                                .gap_6()
                                .child(render_start_actions_section(state, model))
                                .child(render_recent_files_section(state, model)),
                        )
                        // 右列：演练与功能概览
                        .child(
                            div()
                                .flex_1()
                                .max_w(px(460.0))
                                .flex()
                                .flex_col()
                                .gap_4()
                                .child(render_walkthrough_section(state, model)),
                        ),
                ),
        )
}


fn render_start_actions_section(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_new = model.clone();
    let model_file = model.clone();
    let model_folder = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(14.0))
                .text_color(pal.text_primary)
                .child(state.t("welcome.start_section")),
        )
        .child(render_link_action("📄", state.t("empty.newFlow"), Some("Ctrl+N"), state, move |cx| {
            model_new.update(cx, |st, cx| {
                st.new_blank_flow();
                cx.notify();
            });
        }))
        .child(render_link_action("📂", state.t("empty.openFile"), Some("Ctrl+O"), state, move |cx| {
            pick_and_load_file(&model_file, cx);
        }))
        .child(render_link_action("🗂️", state.t("empty.openMod"), None, state, move |cx| {
            pick_and_load_mod_folder(&model_folder, cx);
        }))
}

fn render_link_action<F>(
    icon_str: &'static str,
    label: &str,
    shortcut: Option<&'static str>,
    state: &AppState,
    on_click: F,
) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    let pal = state.theme();
    let label_str = label.to_string();

    div()
        .flex()
        .items_center()
        .justify_between()
        .py_1()
        .hover(move |s| s.text_color(pal.primary_accent))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            on_click(cx);
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_size(px(13.0)).child(icon_str))
                .child(
                    div()
                        .text_size(px(12.5))
                        .text_color(pal.text_primary)
                        .child(label_str),
                ),
        )
        .children(shortcut.map(|sc| {
            div()
                .text_size(px(11.0))
                .text_color(pal.text_secondary)
                .child(sc)
        }))
}

fn render_recent_files_section(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let recent_count = state.recent_projects.len();
    let model_clear = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(14.0))
                        .text_color(pal.text_primary)
                        .child(state.t("welcome.recent_section")),
                )
                .children(if recent_count > 0 {
                    Some(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(0xEF4444))
                            .hover(|s| s.text_color(rgb(0xF87171)))
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                model_clear.update(cx, |st, cx| {
                                    st.recent_projects.clear();
                                    cx.notify();
                                });
                            })
                            .child("清空记录..."),
                    )
                } else {
                    None
                }),
        )
        .children(if state.recent_projects.is_empty() {
            Some(
                div()
                    .py_2()
                    .text_size(px(11.5))
                    .text_color(pal.text_secondary)
                    .child(state.t("empty.noRecentProjects")),
            )
        } else {
            None
        })
        .children(state.recent_projects.iter().take(5).enumerate().map(|(idx, item)| {
            render_vs_recent_row(item, idx, state, model)
        }))
}

fn render_vs_recent_row(
    item: &RecentProjectItem,
    idx: usize,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_open = model.clone();
    let model_del = model.clone();
    let item_clone = item.clone();

    div()
        .flex()
        .items_center()
        .justify_between()
        .py_1p5()
        .px_2()
        .rounded_md()
        .hover(move |s| s.bg(pal.card_hover))
        .cursor_pointer()

        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let idata = item_clone.clone();
            model_open.update(cx, |st, cx| {
                if !idata.path.is_empty() {
                    st.load_file_by_path(&idata.path);
                }
                st.fit_view();
                st.current_tab = AppWorkspaceTab::Flowchart;
                st.toast = Some(Toast::success(format!("📂 已加载: {}", idata.name)));
                cx.notify();
            });
        })

        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(div().text_size(px(12.0)).font_weight(FontWeight::BOLD).text_color(pal.primary_accent).child(item.name.clone()))
                .child(div().text_size(px(10.5)).text_color(pal.text_secondary).child(item.path.clone())),
        )
        .child(
            div()
                .w(px(18.0))
                .h(px(18.0))
                .rounded_md()
                .hover(|s| s.bg(rgb(0xEF4444)).text_color(rgb(0xFFFFFF)))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_color(pal.text_secondary)
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_del.update(cx, |st, cx| {
                        if idx < st.recent_projects.len() { st.recent_projects.remove(idx); }
                        cx.notify();
                    });
                })
                .child("✕"),
        )
}

