use gpui::*;
use crate::model::SearchResultItem;
use crate::ui::state::AppState;
pub fn render_search_row(
    item: &SearchResultItem,
    is_selected: bool,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let model_jump = model.clone();
    let path_to_load = item.path.clone();
    let is_sarc = item.is_sarc;
    let name = item.name.clone();
    let dir_path = item.dir_path.clone();
    let icon = item.icon;

    div()
        .h(px(36.0))
        .w_full()
        .px_2()
        .rounded_md()
        .cursor_pointer()
        .bg(if is_selected {
            pal.card_surface
        } else {
            rgba(0x00000000)
        })
        .border_l_2()
        .border_color(if is_selected {
            pal.primary_accent
        } else {
            rgba(0x00000000)
        })
        .hover(move |s| {
            if is_selected {
                s.bg(pal.card_hover)
            } else {
                s.bg(pal.card_surface)
            }
        })
        .flex()
        .items_center()
        .gap_2()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let p = path_to_load.clone();
            model_jump.update(cx, |st, cx| {
                st.clear_active_text_inputs();
                if !is_sarc {
                    st.load_file_by_path(&p);
                    // 从搜索界面打开文件：保持在搜索界面，不跳转回资源管理器界面
                    st.reveal_file_in_tree(&p, false);
                } else {
                    // 若点击纯 SARC 封包容器节点，在资源管理器中定位展开
                    st.reveal_file_in_tree(&p, true);
                }
                cx.notify();
            });
        })
        .child(
            div()
                .w(px(16.0))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(12.0))
                .child(icon),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .overflow_hidden()
                .justify_center()
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_primary)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(name),
                )
                .children((!dir_path.is_empty()).then(|| {
                    div()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(dir_path)
                })),
        )
}

/// 渲染搜索面板顶部标题栏与快捷操作按钮
pub fn render_search_header(
    state: &AppState,
    query: &str,
    show_filters: bool,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let model_refresh = model.clone();
    let model_clear = model.clone();
    let model_filter_toggle = model.clone();
    let has_query = !query.is_empty();

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
                .font_weight(FontWeight::BOLD)
                .text_size(px(11.0))
                .text_color(pal.text_primary)
                .child(state.t("search.title")),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
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
                            model_refresh.update(cx, |st, cx| {
                                st.update_global_search_results();
                                cx.notify();
                            });
                        })
                        .child("🔄"),
                )
                .children(has_query.then(|| {
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_md()
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .hover(move |s| s.bg(pal.card_hover).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_clear.update(cx, |st, cx| {
                                st.global_search_query.clear();
                                st.global_search_focused = true;
                                st.update_global_search_results();
                                cx.notify();
                            });
                        })
                        .child("✕")
                }))
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_md()
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .text_color(if show_filters { pal.primary_accent } else { pal.text_secondary })
                        .bg(if show_filters { pal.card_surface } else { rgba(0x00000000) })
                        .hover(move |s| s.bg(pal.card_hover))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_filter_toggle.update(cx, |st, cx| {
                                st.global_search_show_filters = !st.global_search_show_filters;
                                cx.notify();
                            });
                        })
                        .child("⋯"),
                ),
        )
}
