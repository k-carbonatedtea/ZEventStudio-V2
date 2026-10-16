use gpui::*;
use crate::ui::components::render_file_tree;
use crate::ui::state::AppState;
use crate::ui::views::welcome::quick_tab::{pick_and_load_file, pick_and_load_mod_folder, pick_and_load_pack_file};

/// 渲染左侧「📁 资源管理器」侧边栏面板 (支持 Mod 目录树与 SARC 封包层级浏览)
pub fn render_files_manager(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let project_name = state
        .project_name
        .clone()
        .unwrap_or_else(|| state.t("explorer.no_project").to_string());
    let has_tree = !state.file_tree.is_empty();

    let model_refresh = model.clone();
    let model_new = model.clone();
    let model_open_mod = model.clone();
    let model_open_pack = model.clone();
    let model_open_file = model.clone();
    let model_collapse = model.clone();

    div()
        .flex()
        .flex_col()
        .h_full()
        .w_full()
        // 1. 顶部资源管理器项目标题栏与快捷操作按钮组
        .child(
            div()
                .h(px(36.0))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(pal.panel_border)
                .bg(pal.panel_surface) // 蓝灰表面 Panel Surface
                // 项目名 / 封包名
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .overflow_hidden()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.5))
                                .text_color(pal.text_primary) // 亮白灰 Text Primary
                                .overflow_hidden()
                                .child(project_name),
                        ),
                )
                // 操作按钮组
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        // 新建事件流
                        .child(render_icon_btn("📄+", state.t("explorer.new_bfevfl"), state, move |cx| {
                            model_new.update(cx, |st, cx| {
                                st.new_blank_flow();
                                cx.notify();
                            });
                        }))
                        // 全部折叠
                        .children(has_tree.then(|| {
                            render_icon_btn("🗜️", state.t("explorer.collapse_all"), state, move |cx| {
                                model_collapse.update(cx, |st, cx| {
                                    st.collapse_all_tree_paths();
                                    cx.notify();
                                });
                            })
                        }))
                        // 刷新文件树
                        .children(has_tree.then(|| {
                            render_icon_btn("🔄", state.t("explorer.refresh"), state, move |cx| {
                                model_refresh.update(cx, |st, cx| {
                                    st.reload_project();
                                    cx.notify();
                                });
                            })
                        })),
                ),
        )
        // 2. 搜索过滤栏 (仅当有文件时显示)
        .children(has_tree.then(|| {
            let search_val = state.file_search_query.clone();
            div()
                .px_2p5()
                .pt_2()
                .pb_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1()
                        .rounded_lg()
                        .bg(pal.app_bg)
                        .border_1()
                        .border_color(pal.panel_border)
                        .child(div().text_size(px(11.0)).child("🔍"))
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(11.0))
                                .text_color(if search_val.is_empty() {
                                    pal.text_secondary
                                } else {
                                    pal.text_primary
                                })
                                .child(if search_val.is_empty() {
                                    state.t("explorer.search_placeholder").to_string()
                                } else {
                                    search_val
                                }),
                        ),
                )
        }))
        // 3. 文件树主体列表 / 空白引导区
        .child(
            div()
                .id("files_explorer_scroll_area")
                .group("file-tree-container")
                .flex_1()
                .overflow_hidden()
                .p_1()
                .child(if has_tree {
                    render_file_tree(&state.file_tree, state, model, 0).into_any_element()
                } else {
                    div()
                        .p_3()
                        .flex()
                        .flex_col()
                        .gap_2p5()
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(pal.text_secondary)
                                .child(state.t("welcome.no_folder_hint")),
                        )
                        // 打开 Mod 目录按钮
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
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    pick_and_load_mod_folder(&model_open_mod, cx);
                                })
                                .child("🗂️")
                                .child(state.t("explorer.open_mod")),
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
                                    pick_and_load_pack_file(&model_open_pack, cx);
                                })
                                .child("🗃️")
                                .child(state.t("explorer.open_pack")),
                        )
                        // 打开文件按钮
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
                                    pick_and_load_file(&model_open_file, cx);
                                })
                                .child("📁")
                                .child(state.t("welcome.open_file_btn")),
                        )
                        .into_any_element()
                }),
        )
}

/// 渲染微型图标操作按钮
fn render_icon_btn<F>(icon: &'static str, tooltip: &str, state: &AppState, on_click: F) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    let _ = tooltip;
    let pal = state.theme();
    div()
        .px_1p5()
        .py_0p5()
        .rounded_md()
        .cursor_pointer()
        .text_size(px(11.0))
        .text_color(pal.text_secondary)
        .hover(move |s| {
            s.bg(pal.card_hover)
                .text_color(pal.text_primary)
        })
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            on_click(cx);
        })
        .child(icon)
}

