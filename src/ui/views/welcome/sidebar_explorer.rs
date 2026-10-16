use gpui::*;
use crate::ui::components::{render_file_search_and_sort_bar, render_file_tree};
use crate::ui::sidebar::{render_activity_bar, render_search_panel, render_source_control_panel};
use crate::ui::state::{ActivityBarTab, AppState};
use crate::ui::views::welcome::quick_tab::{pick_and_load_file, pick_and_load_mod_folder, pick_and_load_pack_file};

/// 渲染 VS Code 风格的左侧功能侧边栏面板 (垂直活动栏 + 资源管理器/搜索/源文件管理)
pub fn render_welcome_sidebar(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let current_activity = state.active_activity_tab;
    let sidebar_open = state.sidebar_open;

    div()
        .flex()
        .h_full()
        // 1. 最左侧垂直活动栏 (48px)
        .child(render_activity_bar(state, model))
        // 2. 侧边栏主体面板 (仅展开时显示)
        .children(sidebar_open.then(|| {
            div()
                .w(px(state.sidebar_width))
                .flex_shrink_0()
                .h_full()
                .bg(pal.panel_surface)
                .border_r_1()
                .border_color(pal.panel_border)
                .overflow_hidden()
                .child(match current_activity {
                    ActivityBarTab::Explorer => {
                        render_welcome_explorer_content(state, model).into_any_element()
                    }
                    ActivityBarTab::Search => {
                        render_search_panel(state, model).into_any_element()
                    }
                    ActivityBarTab::SourceControl => {
                        render_source_control_panel(state, model).into_any_element()
                    }
                })
        }))
}

/// 渲染启动页资源管理器面板内容
fn render_welcome_explorer_content(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_folder = model.clone();
    let model_pack = model.clone();
    let model_file = model.clone();
    let model_new = model.clone();
    let model_refresh = model.clone();
    let model_collapse = model.clone();

    let has_tree = !state.file_tree.is_empty();
    let project_name = state
        .project_name
        .clone()
        .unwrap_or_else(|| state.t("welcome.no_folder").to_string());

    div()
        .size_full()
        .overflow_hidden()
        .flex()
        .flex_col()

        // 顶部标题栏
        .child(
            div()
                .h(px(36.0))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(pal.panel_border)
                .bg(pal.panel_surface)
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_primary) // 亮白灰 Text Primary
                        .child(state.t("welcome.explorer")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        // 新建事件流按钮
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .hover(move |s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_new.update(cx, |st, cx| {
                                        st.new_blank_flow();
                                        cx.notify();
                                    });
                                })
                                .child("📄+"),
                        )
                        // 刷新按钮 (有文件树时)
                        .children(has_tree.then(|| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .hover(move |s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_refresh.update(cx, |st, cx| {
                                        st.reload_project();
                                        cx.notify();
                                    });
                                })
                                .child("🔄")
                        }))
                        // 全部折叠
                        .children(has_tree.then(|| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .hover(move |s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_collapse.update(cx, |st, cx| {
                                        st.collapse_all_tree_paths();
                                        cx.notify();
                                    });
                                })
                                .child("🗜️")
                        })),
                ),
        )
        // 核心折叠区 / 文件树区
        .child(
            div()
                .id("welcome_sidebar_scroll")
                .group("file-tree-container")
                .flex_1()
                .overflow_hidden()
                .flex()
                .flex_col()
                // 折叠头
                .child(
                    div()
                        .px_3()
                        .py_1p5()
                        .bg(pal.card_surface) // 浅层蓝灰 Card / Hover
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_primary)
                        .child("⌄")
                        .child(project_name),
                )
                .child(if has_tree {
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .flex()
                        .flex_col()
                        .child(render_file_search_and_sort_bar(state, model))
                        .child(
                            div()
                                .flex_1()
                                .overflow_hidden()
                                .p_1()
                                .child(render_file_tree(&state.file_tree, state, model, 0)),
                        )
                        .into_any_element()
                } else {
                    // 无文件时展示快捷引导按钮
                    div()
                        .p_4()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(pal.text_secondary)
                                .child(state.t("welcome.no_folder_hint")),
                        )
                        // 打开文件夹大按钮
                        .child(
                            div()
                                .w_full()
                                .py_2()
                                .px_3()
                                .rounded_md()
                                .bg(pal.primary_accent) // 遗迹青蓝 Primary Accent
                                .hover(|s| s.opacity(0.90))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    pick_and_load_mod_folder(&model_folder, cx);
                                })
                                .child("🗂️")
                                .child(state.t("welcome.open_folder_btn")),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .text_color(pal.text_secondary)
                                .child(state.t("welcome.open_folder_desc")),
                        )
                        // 打开 SARC 封包按钮
                        .child(
                            div()
                                .w_full()
                                .py_1p5()
                                .px_3()
                                .rounded_md()
                                .bg(pal.card_surface) // 浅层蓝灰 Card / Hover
                                .border_1()
                                .border_color(pal.card_border)
                                .hover(move |s| s.bg(pal.card_hover).border_color(pal.primary_accent))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .text_size(px(11.5))
                                .text_color(pal.text_primary)
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    pick_and_load_pack_file(&model_pack, cx);
                                })
                                .child("🗃️")
                                .child(state.t("explorer.open_pack")),
                        )
                        // 打开单个文件次要按钮
                        .child(
                            div()
                                .w_full()
                                .py_1p5()
                                .px_3()
                                .rounded_md()
                                .bg(pal.card_surface) // 浅层蓝灰 Card / Hover
                                .border_1()
                                .border_color(pal.card_border)
                                .hover(move |s| s.bg(pal.card_hover).border_color(pal.primary_accent))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .text_size(px(11.5))
                                .text_color(pal.text_primary)
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    pick_and_load_file(&model_file, cx);
                                })
                                .child("📁")
                                .child(state.t("welcome.open_file_btn")),
                        )
                        .into_any_element()
                }),
        )
}
