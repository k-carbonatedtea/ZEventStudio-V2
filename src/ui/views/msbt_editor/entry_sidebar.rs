use gpui::*;
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::AppState;
use crate::ui::views::msbt_editor::types::MsbtDocument;

/// 渲染 MSBT 条目侧边栏列表 (支持条目搜索、过滤、切换与增删)
pub fn render_msbt_entry_sidebar(
    doc: &MsbtDocument,
    selected_key: Option<&str>,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let is_search_focused = state.msbt_search_focused;
    let search_query = state.msbt_search_query.clone();

    let model_search_click = model.clone();
    let model_search_out = model.clone();
    let model_search_clear = model.clone();
    let model_add = model.clone();

    // 过滤条目列表
    let query_lower = search_query.to_lowercase();
    let filtered_entries: Vec<_> = doc
        .entries
        .iter()
        .filter(|e| {
            if query_lower.is_empty() {
                true
            } else {
                e.key.to_lowercase().contains(&query_lower)
                    || e.clean_text.to_lowercase().contains(&query_lower)
            }
        })
        .collect();

    div()
        .w(px(280.0))
        .h_full()
        .bg(pal.panel_surface)
        .border_r_1()
        .border_color(pal.panel_border)
        .flex()
        .flex_col()
        .overflow_hidden()
        // 1. 顶栏：标题、条目计数与新建按钮
        .child(
            div()
                .h(px(38.0))
                .px_3()
                .border_b_1()
                .border_color(pal.panel_border)
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.0))
                                .text_color(pal.text_primary)
                                .child(state.t("msbt_editor.entries")),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.card_border)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_secondary)
                                .child(format!("{}", doc.entries.len())),
                        ),
                )
                // 新建条目按钮
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.primary_accent)
                        .hover(|s| s.bg(pal.primary_accent).text_color(rgb(0xFFFFFF)))
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.primary_accent)
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_add.update(cx, |st, cx| {
                                st.msbt_new_entry_modal_open = true;
                                st.msbt_new_entry_name.clear();
                                st.msbt_new_entry_focused = true;
                                cx.notify();
                            });
                        })
                        .child(format!("+ {}", state.t("msbt_editor.add_entry"))),
                ),
        )
        // 2. 搜索框 (严格遵循规范 8 实现 IME 与失焦)
        .child(
            div()
                .px_2p5()
                .py_2()
                .border_b_1()
                .border_color(pal.panel_border)
                .child(
                    div()
                        .id("msbt_search_input_container")
                        .h(px(28.0))
                        .px_2()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(if is_search_focused {
                            pal.primary_accent
                        } else {
                            pal.panel_border
                        })
                        .flex()
                        .items_center()
                        .justify_between()
                        .cursor_text()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_search_click.update(cx, |st, cx| {
                                st.clear_active_text_inputs();
                                st.msbt_search_focused = true;
                                cx.notify();
                            });
                        })
                        .on_mouse_down_out(move |_, _, cx| {
                            model_search_out.update(cx, |st, cx| {
                                if st.msbt_search_focused {
                                    st.msbt_search_focused = false;
                                    cx.notify();
                                }
                            });
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .overflow_hidden()
                                .child(div().text_size(px(11.0)).child("🔍"))
                                .child(if search_query.is_empty() && !is_search_focused {
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(pal.text_secondary)
                                        .child(state.t("msbt_editor.search_placeholder"))
                                        .into_any_element()
                                } else {
                                    div()
                                        .flex()
                                        .items_center()
                                        .text_size(px(11.0))
                                        .text_color(pal.text_primary)
                                        .child(search_query.clone())
                                        .children(is_search_focused.then(|| {
                                            render_blinking_cursor(pal.primary_accent.into())
                                                .into_any_element()
                                        }))
                                        .into_any_element()
                                }),
                        )
                        .children((!search_query.is_empty()).then(|| {
                            div()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.text_color(pal.text_primary))
                                .child("✕")
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_search_clear.update(cx, |st, cx| {
                                        st.msbt_search_query.clear();
                                        cx.notify();
                                    });
                                })
                        })),
                ),
        )
        // 3. 条目列表容器
        .child(
            div()
                .id("msbt_entries_scroll_list")
                .flex_1()
                .overflow_y_scroll()
                .p_1p5()
                .flex()
                .flex_col()
                .gap_1()
                .children(if filtered_entries.is_empty() {
                    vec![div()
                        .p_4()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("msbt_editor.no_entries"))
                        .into_any_element()]
                } else {
                    filtered_entries
                        .into_iter()
                        .map(|entry| {
                            let is_selected = selected_key == Some(&entry.key);
                            let entry_key = entry.key.clone();
                            let model_select = model.clone();

                            div()
                                .p_2()
                                .rounded_md()
                                .cursor_pointer()
                                .border_1()
                                .border_color(if is_selected {
                                    pal.primary_accent
                                } else {
                                    pal.card_border
                                })
                                .bg(if is_selected {
                                    pal.card_surface
                                } else {
                                    rgba(0x00000000)
                                })
                                .hover(|s| s.bg(pal.card_hover))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    let k = entry_key.clone();
                                    model_select.update(cx, |st, cx| {
                                        st.msbt_selected_entry_key = Some(k);
                                        st.sync_msbt_selected_entry_to_draft();
                                        st.msbt_text_editor_focused = true;
                                        cx.notify();
                                    });
                                })
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_weight(FontWeight::BOLD)
                                                .text_size(px(11.5))
                                                .text_color(if is_selected {
                                                    pal.primary_accent
                                                } else {
                                                    pal.text_primary
                                                })
                                                .child(entry.key.clone()),
                                        )
                                        .child(
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(pal.app_bg)
                                                .text_size(px(9.5))
                                                .text_color(pal.text_secondary)
                                                .child(format!("{}字", entry.char_count)),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(pal.text_secondary)
                                        .overflow_hidden()
                                        .child(if entry.clean_text.is_empty() {
                                            "「空文本」".to_string()
                                        } else {
                                            let preview: String = entry.clean_text.chars().take(30).collect();
                                            if entry.clean_text.chars().count() > 30 {
                                                format!("{}...", preview)
                                            } else {
                                                preview
                                            }
                                        }),
                                )
                                .into_any_element()
                        })
                        .collect()
                }),
        )
}
