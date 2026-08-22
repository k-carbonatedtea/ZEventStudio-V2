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
    div()
        .flex_1()
        .h_full()
        .bg(hsla(220.0 / 360.0, 0.35, 0.06, 1.0))
        .flex()
        .flex_col()
        // 顶部编辑器选项卡栏 (Tab Strip)
        .child(
            div()
                .h(px(36.0))
                .bg(hsla(220.0 / 360.0, 0.40, 0.08, 1.0))
                .border_b_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.16, 1.0))
                .flex()
                .items_center()
                .px_2()
                .child(
                    div()
                        .h(px(35.0))
                        .px_3()
                        .bg(hsla(220.0 / 360.0, 0.35, 0.06, 1.0))
                        .border_t_2()
                        .border_color(hsla(215.0 / 360.0, 0.80, 0.55, 1.0))
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hsla(0.0, 0.0, 1.0, 0.95))
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
                                .text_color(hsla(0.0, 0.0, 1.0, 0.98))
                                .child(state.t("empty.title")),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(hsla(215.0 / 360.0, 0.20, 0.60, 1.0))
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
    let model_new = model.clone();
    let model_file = model.clone();
    let model_folder = model.clone();
    let model_100 = model.clone();
    let model_quest = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(14.0))
                .text_color(hsla(0.0, 0.0, 1.0, 0.90))
                .child(state.t("welcome.start_section")),
        )
        .child(render_link_action("📄", state.t("empty.newFlow"), Some("Ctrl+N"), move |cx| {
            model_new.update(cx, |st, cx| {
                st.new_blank_flow();
                cx.notify();
            });
        }))
        .child(render_link_action("📂", state.t("empty.openFile"), Some("Ctrl+O"), move |cx| {
            pick_and_load_file(&model_file, cx);
        }))
        .child(render_link_action("🗂️", state.t("empty.openMod"), None, move |cx| {
            pick_and_load_mod_folder(&model_folder, cx);
        }))
        .child(render_link_action("🗡️", "加载 100enemy 剑之试炼预设...", None, move |cx| {
            load_preset_to_flowchart(&model_100, "zelda_100enemy", "📂 已加载 100enemy.bfevfl 官方二进制", cx);
        }))
        .child(render_link_action("🎮", "加载任务与对话事件流...", None, move |cx| {
            load_preset_to_flowchart(&model_quest, "quest_dialogue", "📂 已加载任务与对话事件流", cx);
        }))
}

fn load_preset_to_flowchart(model: &Entity<AppState>, preset_id: &str, toast_msg: &'static str, cx: &mut App) {
    let pid = preset_id.to_string();
    model.update(cx, |st, cx| {
        if let Some(mut loaded) = crate::presets::load_preset_by_id(&pid) {
            loaded.ensure_actors_populated();
            crate::model::layout::auto_layout_graph(&mut loaded);
            st.graph = loaded;
            st.active_preset_id = pid;
        }
        st.fit_view();
        st.current_tab = AppWorkspaceTab::Flowchart;
        st.toast = Some(Toast::success(toast_msg));
        cx.notify();
    });
}


fn render_link_action<F>(
    icon_str: &'static str,
    label: &str,
    shortcut: Option<&'static str>,
    on_click: F,
) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    let label_str = label.to_string();

    div()
        .flex()
        .items_center()
        .justify_between()
        .py_1()
        .hover(|s| s.text_color(hsla(215.0 / 360.0, 0.80, 0.70, 1.0)))
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
                        .text_color(hsla(215.0 / 360.0, 0.70, 0.65, 1.0))
                        .child(label_str),
                ),
        )
        .children(shortcut.map(|sc| {
            div()
                .text_size(px(11.0))
                .text_color(hsla(215.0 / 360.0, 0.20, 0.50, 1.0))
                .child(sc)
        }))
}

fn render_recent_files_section(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
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
                        .text_color(hsla(0.0, 0.0, 1.0, 0.90))
                        .child(state.t("welcome.recent_section")),
                )
                .children(if recent_count > 0 {
                    Some(
                        div()
                            .text_size(px(11.0))
                            .text_color(hsla(0.0, 0.70, 0.60, 0.9))
                            .hover(|s| s.text_color(hsla(0.0, 0.80, 0.70, 1.0)))
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
                    .text_color(hsla(215.0 / 360.0, 0.20, 0.50, 1.0))
                    .child(state.t("empty.noRecentProjects")),
            )
        } else {
            None
        })
        .children(state.recent_projects.iter().take(5).enumerate().map(|(idx, item)| {
            render_vs_recent_row(item, idx, model)
        }))
}

fn render_vs_recent_row(
    item: &RecentProjectItem,
    idx: usize,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_open = model.clone();
    let model_del = model.clone();
    let item_clone = item.clone();

    div()
        .flex()
        .items_center()
        .justify_between()
        .py_1()
        .hover(|s| s.bg(hsla(220.0 / 360.0, 0.25, 0.12, 1.0)))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let idata = item_clone.clone();
            model_open.update(cx, |st, cx| {
                if let Some(ref pid) = idata.preset_id {
                    if let Some(mut loaded) = crate::presets::load_preset_by_id(pid) {
                        loaded.ensure_actors_populated();
                        crate::model::layout::auto_layout_graph(&mut loaded);
                        st.graph = loaded;
                        st.active_preset_id = pid.clone();
                    }
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
                .child(div().text_size(px(12.0)).font_weight(FontWeight::BOLD).text_color(hsla(215.0 / 360.0, 0.70, 0.65, 1.0)).child(item.name.clone()))
                .child(div().text_size(px(10.5)).text_color(hsla(215.0 / 360.0, 0.20, 0.45, 1.0)).child(item.path.clone())),
        )
        .child(
            div()
                .w(px(18.0))
                .h(px(18.0))
                .rounded_md()
                .hover(|s| s.bg(hsla(0.0, 0.70, 0.35, 0.9)))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
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

