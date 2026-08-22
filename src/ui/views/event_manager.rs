use gpui::*;
use crate::ui::state::{AppState, AppWorkspaceTab};

/// 渲染全屏事件列表工作区视图 (Event Manager: 事件索引概览表 + 选中事件详情与参数属性表)
pub fn render_event_manager_view(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let events = &state.graph.nodes;
    let total_count = events.len();
    let sel_id = state.selected_node_id.clone();
    let selected_event = sel_id.as_ref().and_then(|id| state.graph.find_node(id)).cloned();

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(hsla(220.0 / 360.0, 0.35, 0.08, 1.0))
        .text_color(rgb(0xffffff))
        // 顶部工具栏
        .child(
            div()
                .h(px(40.0))
                .px_4()
                .border_b_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
                .flex()
                .items_center()
                .justify_between()
                .bg(hsla(220.0 / 360.0, 0.30, 0.10, 0.95))
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(hsla(0.0, 0.0, 1.0, 0.95))
                        .child(format!("共 {} 个事件", total_count)),
                ),
        )
        // 上半区：事件表格
        .child(
            div()
                .id("events_table_scroll")
                .flex_1()
                .max_h(px(320.0))
                .overflow_y_scroll()
                .bg(hsla(220.0 / 360.0, 0.35, 0.09, 1.0))
                .border_b_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
                .child(render_events_table(state, events, sel_id.as_deref(), model)),
        )
        // 下半区：选中事件的详情与参数
        .child(
            div()
                .id("events_detail_scroll")
                .flex_1()
                .size_full()
                .overflow_y_scroll()
                .p_4()
                .bg(hsla(220.0 / 360.0, 0.35, 0.07, 1.0))

                .children(if let Some(ev) = selected_event {
                    vec![render_selected_event_details(state, &ev, model).into_any_element()]
                } else {
                    vec![
                        div()
                            .p_8()
                            .text_size(px(12.5))
                            .text_color(hsla(215.0 / 360.0, 0.20, 0.55, 1.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(state.t("events_view.select_prompt"))
                            .into_any_element(),
                    ]
                }),
        )
}

fn render_events_table(
    state: &AppState,
    events: &[crate::model::NodeData],
    selected_id: Option<&str>,
    model: &Entity<AppState>,
) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        // 表头
        .child(
            div()
                .flex()
                .items_center()
                .px_4()
                .py_2()
                .bg(hsla(220.0 / 360.0, 0.30, 0.12, 1.0))
                .border_b_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
                .text_size(px(11.5))
                .font_weight(FontWeight::BOLD)
                .text_color(hsla(215.0 / 360.0, 0.20, 0.65, 1.0))
                .child(div().w(px(60.0)).child(state.t("events_view.th_index")))
                .child(div().w(px(240.0)).child(state.t("events_view.th_name")))
                .child(div().w(px(120.0)).child(state.t("events_view.th_type")))
                .child(div().w(px(260.0)).child(state.t("events_view.th_actor")))
                .child(div().w(px(160.0)).child(state.t("events_view.th_next")))
                .child(div().flex_1().child(state.t("events_view.th_action"))),
        )
        // 表体行
        .children(events.iter().enumerate().map(|(idx, node)| {
            let is_sel = selected_id == Some(&node.id);
            let n_id = node.id.clone();
            let n_id_focus = node.id.clone();
            let n_id_del = node.id.clone();
            let model_row = model.clone();
            let model_focus = model.clone();
            let model_del = model.clone();

            let subtitle = node.subtitle.as_deref().unwrap_or("-");

            div()
                .flex()
                .items_center()
                .px_4()
                .py_2()
                .bg(if is_sel {
                    hsla(215.0 / 360.0, 0.50, 0.22, 1.0)
                } else if idx % 2 == 0 {
                    hsla(220.0 / 360.0, 0.20, 0.09, 0.5)
                } else {
                    hsla(220.0 / 360.0, 0.20, 0.08, 0.5)
                })
                .border_b_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.14, 0.8))
                .cursor_pointer()
                .hover(|s| {
                    if !is_sel {
                        s.bg(hsla(215.0 / 360.0, 0.30, 0.16, 0.8))
                    } else {
                        s
                    }
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    let n = n_id.clone();
                    model_row.update(cx, |st, cx| {
                        st.selected_node_id = Some(n);
                        cx.notify();
                    });
                })
                .text_size(px(12.0))
                .text_color(if is_sel {
                    hsla(0.0, 0.0, 1.0, 1.0)
                } else {
                    hsla(215.0 / 360.0, 0.20, 0.80, 1.0)
                })
                .child(div().w(px(60.0)).text_color(hsla(215.0 / 360.0, 0.20, 0.55, 1.0)).child(format!("{}", idx)))
                .child(
                    div()
                        .w(px(240.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if is_sel { hsla(45.0 / 360.0, 0.98, 0.65, 1.0) } else { hsla(0.0, 0.0, 1.0, 1.0) })
                        .child(node.title.clone()),
                )
                .child(
                    div()
                        .w(px(120.0))
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .bg(node.kind.badge_color())
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xffffff))
                                .child(node.kind.display_name()),
                        ),
                )
                .child(div().w(px(260.0)).child(subtitle.to_string()))
                .child(div().w(px(160.0)).text_color(hsla(215.0 / 360.0, 0.20, 0.65, 1.0)).child("Next ➔"))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(hsla(215.0 / 360.0, 0.35, 0.22, 1.0))
                                .hover(|s| s.bg(hsla(215.0 / 360.0, 0.50, 0.32, 1.0)))
                                .cursor_pointer()
                                .text_size(px(10.5))
                                .text_color(rgb(0xffffff))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    let focus_id = n_id_focus.clone();
                                    model_focus.update(cx, |st, cx| {
                                        st.current_tab = AppWorkspaceTab::Flowchart;
                                        st.focus_node(&focus_id);
                                        cx.notify();
                                    });
                                })
                                .child(state.t("events_view.jump_to_flowchart")),
                        )
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
                                    let del_id = n_id_del.clone();
                                    model_del.update(cx, |st, cx| {
                                        st.graph.remove_node(&del_id);
                                        if st.selected_node_id.as_deref() == Some(&del_id) {
                                            st.selected_node_id = None;
                                        }
                                        cx.notify();
                                    });
                                })
                                .child("🗑️"),
                        ),
                )
        }))
}

fn render_selected_event_details(
    state: &AppState,
    node: &crate::model::NodeData,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let n_id = node.id.clone();
    let model_focus = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .pb_2()
                .border_b_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_size(px(14.0)).child(node.kind.icon()))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(14.0))
                                .text_color(rgb(0xffffff))
                                .child(format!("{} ({})", node.title, node.kind.display_name())),
                        ),
                )
                .child(
                    div()
                        .px_3()
                        .py_1()
                        .rounded_md()
                        .bg(hsla(200.0 / 360.0, 0.40, 0.25, 1.0))
                        .hover(|s| s.bg(hsla(200.0 / 360.0, 0.55, 0.35, 1.0)))
                        .cursor_pointer()
                        .text_size(px(11.5))
                        .text_color(rgb(0xffffff))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let focus_id = n_id.clone();
                            model_focus.update(cx, |st, cx| {
                                st.current_tab = AppWorkspaceTab::Flowchart;
                                st.focus_node(&focus_id);
                                cx.notify();
                            });
                        })
                        .child(state.t("events_view.jump_to_flowchart")),
                ),
        )
        // 参数属性列表表格
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(12.0))
                .text_color(hsla(215.0 / 360.0, 0.20, 0.70, 1.0))
                .child("参数属性 (Parameters):"),
        )
        .child(
            div()
                .rounded_lg()
                .border_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
                .bg(hsla(220.0 / 360.0, 0.30, 0.09, 1.0))
                .overflow_hidden()
                .children(if node.properties.is_empty() {
                    vec![
                        div()
                            .p_4()
                            .text_size(px(11.5))
                            .text_color(hsla(215.0 / 360.0, 0.20, 0.55, 1.0))
                            .child("暂无自定义参数")
                            .into_any_element(),
                    ]
                } else {
                    node.properties
                        .iter()
                        .enumerate()
                        .map(|(p_idx, prop)| {
                            div()
                                .flex()
                                .items_center()
                                .px_4()
                                .py_2()
                                .border_b_1()
                                .border_color(hsla(220.0 / 360.0, 0.25, 0.14, 0.6))
                                .bg(if p_idx % 2 == 0 {
                                    hsla(220.0 / 360.0, 0.15, 0.10, 0.5)
                                } else {
                                    hsla(220.0 / 360.0, 0.15, 0.08, 0.5)
                                })
                                .text_size(px(12.0))
                                .child(div().w(px(120.0)).text_color(hsla(200.0 / 360.0, 0.80, 0.70, 1.0)).child(prop.val_type.clone()))
                                .child(div().w(px(240.0)).font_weight(FontWeight::SEMIBOLD).child(prop.key.clone()))
                                .child(div().flex_1().text_color(hsla(45.0 / 360.0, 0.90, 0.75, 1.0)).child(prop.val.clone()))
                                .into_any_element()
                        })
                        .collect()
                }),
        )
}
