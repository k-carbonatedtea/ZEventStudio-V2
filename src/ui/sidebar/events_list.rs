use gpui::*;
use crate::ui::state::AppState;


/// 渲染左侧事件列表 (Events Manager: 搜索、类型过滤、快速定位聚焦、删除)
pub fn render_events_list(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let search_val = state.search_query.to_lowercase();
    let selected_id = state.selected_node_id.clone();

    // 过滤出所有事件节点（排除 EntryPoint 或包含所有）
    let events: Vec<_> = state
        .graph
        .nodes
        .iter()
        .filter(|n| {
            if search_val.is_empty() {
                true
            } else {
                n.id.to_lowercase().contains(&search_val)
                    || n.title.to_lowercase().contains(&search_val)
                    || n.subtitle.as_deref().unwrap_or("").to_lowercase().contains(&search_val)
            }
        })
        .collect();

    let total_count = events.len();

    div()
        .flex()
        .flex_col()
        .size_full()
        .p_2p5()
        .gap_2()
        // 顶部搜索统计栏
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_1()
                .pb_1()
                .border_b_1()
                .border_color(pal.panel_border)
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.0))
                        .text_color(pal.text_primary)
                        .child(state.t("sidebar.events_title")),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_full()
                        .bg(pal.card_surface)
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.primary_accent)
                        .child(format!("{} items", total_count)),
                ),
        )
        // 事件卡片列表
        .child(
            div()
                .id("events_scroll_body")
                .flex_1()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap_1p5()
                .children(if events.is_empty() {
                    vec![
                        div()
                            .p_4()
                            .text_size(px(11.5))
                            .text_color(pal.text_secondary)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(state.t("sidebar.no_events_found"))
                            .into_any_element(),
                    ]
                } else {
                    events
                        .into_iter()
                        .map(|node| {
                            let is_active = selected_id.as_deref() == Some(&node.id);
                            let node_id = node.id.clone();
                            let node_id_del = node.id.clone();
                            let model_focus = model.clone();
                            let model_del = model.clone();

                            let icon = node.kind.icon();
                            let subtitle = node.subtitle.clone();

                            div()
                                .p_2()
                                .rounded_lg()
                                .bg(if is_active {
                                    pal.card_hover
                                } else {
                                    pal.card_surface
                                })
                                .border_1()
                                .border_color(if is_active {
                                    pal.primary_accent
                                } else {
                                    pal.card_border
                                })
                                .hover(move |s| {
                                    s.bg(pal.card_hover)
                                        .border_color(pal.primary_accent)
                                })
                                .cursor_pointer()

                                .on_mouse_down(MouseButton::Left, {
                                    let n_id = node_id.clone();
                                    move |_, _, cx| {
                                        model_focus.update(cx, |st, cx| {
                                            st.focus_node(&n_id);
                                            cx.notify();
                                        });
                                    }
                                })
                                .child(
                                    div()
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
                                                        .text_size(px(11.0))
                                                        .child(icon),
                                                )
                                                .child(
                                                    div()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_size(px(12.0))
                                                        .text_color(if is_active {
                                                            pal.primary_accent
                                                        } else {
                                                            pal.text_primary
                                                        })
                                                        .child(node.title.clone()),
                                                ),
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
                                                        .bg(node.kind.badge_color())
                                                        .text_size(px(9.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(0xffffff))
                                                        .child(node.kind.display_name()),
                                                )
                                                .child(
                                                    div()
                                                        .px_1()
                                                        .py_0p5()
                                                        .rounded_md()
                                                        .hover(|s| s.bg(hsla(0.0, 0.75, 0.35, 0.9)))
                                                        .cursor_pointer()
                                                        .on_mouse_down(MouseButton::Left, {
                                                            let n_id_del = node_id_del.clone();
                                                            move |_, _, cx| {
                                                                cx.stop_propagation();
                                                                model_del.update(cx, |st, cx| {
                                                                    st.graph.remove_node(&n_id_del);
                                                                    if st.selected_node_id.as_deref() == Some(&n_id_del) {
                                                                        st.selected_node_id = None;
                                                                    }
                                                                    st.toast = Some(crate::ui::components::toast::Toast::info(
                                                                        st.t("toast.deleted_node"),
                                                                    ));
                                                                    cx.notify();
                                                                });
                                                            }
                                                        })
                                                        .child(
                                                            div()
                                                                .text_size(px(10.0))
                                                                .text_color(hsla(0.0, 0.90, 0.70, 1.0))
                                                                .child("🗑️"),
                                                        ),
                                                ),
                                        ),
                                )
                                .children(subtitle.map(|sub| {
                                    div()
                                        .pt_1()
                                        .text_size(px(10.5))
                                        .text_color(pal.text_secondary)
                                        .child(sub)
                                }))
                                .into_any_element()
                        })
                        .collect::<Vec<_>>()
                }),
        )
}
