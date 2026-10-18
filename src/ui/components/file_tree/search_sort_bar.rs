use gpui::*;
use crate::ui::state::{AppState, FileFilterType, FileSortMode};
pub fn render_file_search_and_sort_bar(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_sort = model.clone();

    let sort_mode = state.file_sort_mode;
    let sort_menu_open = state.file_tree_sort_menu_open;
    let active_filter = state.file_tree_filter_type;

    div()
        .id("file_search_and_sort_toolbar")
        .px_2()
        .pt_2()
        .pb_1p5()
        .flex()
        .flex_col()
        .gap_1p5()
        .border_b_1()
        .border_color(pal.panel_border)
        .bg(pal.panel_surface)
        // 2. 排序与操作栏
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_1()
                // 排序模式切换按钮
                .child(
                    div()
                        .id("file_sort_btn")
                        .px_1p5()
                        .py_0p5()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .hover(move |s| s.bg(pal.card_hover))
                        .cursor_pointer()
                        .border_1()
                        .border_color(if sort_menu_open {
                            pal.primary_accent
                        } else {
                            pal.panel_border
                        })
                        .flex()
                        .items_center()
                        .gap_1()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_sort.update(cx, |st, cx| {
                                st.file_tree_sort_menu_open = !st.file_tree_sort_menu_open;
                                st.file_search_focused = false;
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(pal.primary_accent)
                                .child("⇅"),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(pal.text_primary)
                                .child(format!("{} {}", sort_mode.icon(), state.t(sort_mode.label_key()))),
                        )
                        .child(
                            div()
                                .text_size(px(8.0))
                                .text_color(pal.text_secondary)
                                .child(if sort_menu_open { "▴" } else { "▾" }),
                        ),
                )
                // 过滤状态指示与重置快捷按钮 (当有类型过滤时显示)
                .children((active_filter != FileFilterType::All).then(|| {
                    let model_reset_all = model.clone();
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .bg(pal.card_surface)
                                .hover(move |s| s.bg(pal.card_hover))
                                .cursor_pointer()
                                .text_size(px(9.5))
                                .text_color(pal.secondary_accent)
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_reset_all.update(cx, |st, cx| {
                                        st.file_tree_filter_type = FileFilterType::All;
                                        cx.notify();
                                    });
                                })
                                .child("↺ 重置"),
                        )
                })),
        )
        // 2.1 排序菜单展开项 (展开在下方)
        .children(sort_menu_open.then(|| {
            let sort_options = [
                FileSortMode::NameAsc,
                FileSortMode::NameDesc,
                FileSortMode::FileType,
                FileSortMode::ModifiedFirst,
            ];
            div()
                .id("file_sort_menu_card")
                .p_1()
                .rounded_md()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.panel_border)
                .shadow_lg()
                .flex()
                .flex_col()
                .gap_0p5()
                .children(sort_options.into_iter().map(|mode| {
                    let is_active = mode == sort_mode;
                    let model_item = model.clone();
                    div()
                        .px_2()
                        .py_1()
                        .rounded_xs()
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_between()
                        .bg(if is_active {
                            pal.card_hover
                        } else {
                            rgba(0x00000000)
                        })
                        .hover(move |s| s.bg(pal.card_hover))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_item.update(cx, |st, cx| {
                                st.file_sort_mode = mode;
                                st.file_tree_sort_menu_open = false;
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .text_size(px(10.5))
                                .text_color(if is_active {
                                    pal.primary_accent
                                } else {
                                    pal.text_primary
                                })
                                .child(mode.icon())
                                .child(state.t(mode.label_key())),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(pal.primary_accent)
                                .child(if is_active { "✓" } else { " " }),
                        )
                }))
        }))
        // 3. 快速类型过滤标签 (Horizontal filter chips)
        .child(
            div()
                .id("file_filter_chips_row")
                .flex()
                .items_center()
                .gap_1()
                .overflow_x_hidden()
                .children({
                    const FILTERS: [FileFilterType; 6] = [
                        FileFilterType::All,
                        FileFilterType::EventFlow,
                        FileFilterType::Msbt,
                        FileFilterType::Param,
                        FileFilterType::Sarc,
                        FileFilterType::Modified,
                    ];
                    FILTERS.into_iter().map(|ft| {
                        let is_active = ft == active_filter;
                        let model_ft = model.clone();
                        div()
                            .px_1p5()
                            .py_0p5()
                            .rounded_md()
                            .cursor_pointer()
                            .text_size(px(9.5))
                            .font_weight(if is_active {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .bg(if is_active {
                                pal.primary_accent
                            } else {
                                pal.card_surface
                            })
                            .text_color(if is_active {
                                pal.panel_surface
                            } else {
                                pal.text_secondary
                            })
                            .border_1()
                            .border_color(if is_active {
                                pal.primary_accent
                            } else {
                                pal.panel_border
                            })
                            .hover(move |s| {
                                if !is_active {
                                    s.bg(pal.card_hover).text_color(pal.text_primary)
                                } else {
                                    s
                                }
                            })
                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                model_ft.update(cx, |st, cx| {
                                    st.file_tree_filter_type = ft;
                                    st.file_tree_sort_menu_open = false;
                                    cx.notify();
                                });
                            })
                            .child(state.t(ft.label_key()))
                    })
                }),
        )
}
