use gpui::*;
use crate::ui::state::AppState;

/// 渲染查询列表列 (Query Column: 列表展示 + 下拉选择与自由输入操作条)
pub fn render_queries_column(
    state: &AppState,
    actor: &crate::model::actor::ActorData,
    actor_idx: usize,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_add = model.clone();
    let is_adding = state.is_adding_query;

    div()
        .w(px(290.0))
        .h_full()
        .border_r_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
        .flex()
        .flex_col()
        // 表头
        .child(
            div()
                .h(px(34.0))
                .px_3()
                .bg(hsla(220.0 / 360.0, 0.30, 0.12, 1.0))
                .border_b_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.0))
                        .text_color(hsla(0.0, 0.0, 1.0, 0.95))
                        .child(format!("查询 ({})", actor.queries.len())),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(if is_adding {
                            hsla(0.0, 0.0, 0.25, 1.0)
                        } else {
                            hsla(215.0 / 360.0, 0.35, 0.22, 1.0)
                        })
                        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.50, 0.32, 1.0)))
                        .cursor_pointer()
                        .text_size(px(10.5))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_add.update(cx, |st, cx| {
                                st.is_adding_query = !st.is_adding_query;
                                st.new_query_name = String::new();
                                st.show_query_dropdown = false;
                                cx.notify();
                            });
                        })
                        .child(if is_adding { "✕ 取消" } else { "+ 添加..." }),
                ),
        )
        // 内联下拉与输入条
        .children(if is_adding {
            Some(render_inline_query_bar(state, actor_idx, model))
        } else {
            None
        })
        // 列表主体
        .child(
            div()
                .id("actor_queries_scroll")
                .flex_1()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .children(actor.queries.iter().enumerate().map(|(qry_idx, qry_name)| {
                    render_query_row(qry_name, actor_idx, qry_idx, model)
                })),
        )
}

fn render_inline_query_bar(
    state: &AppState,
    actor_idx: usize,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let actor_name = state.graph.actors.get(actor_idx).map(|a| a.name.as_str()).unwrap_or("");
    let search_dirs = state.build_game_ai_search_dirs();
    let mut suggestions = crate::model::ai_def::get_actor_available_queries(actor_name, &search_dirs);
    if suggestions.is_empty() {
        suggestions = vec![
            "CheckFlag".to_string(),
            "GeneralQuery".to_string(),
            "CheckGameDataInt".to_string(),
            "CheckGameDataFloat".to_string(),
            "CheckPlayerState".to_string(),
            "CheckItemCount".to_string(),
            "CheckTargetDistance".to_string(),
            "CheckEquipWeapon".to_string(),
            "CheckCurrentTime".to_string(),
            "CheckWeather".to_string(),
        ];
    }

    let model_confirm = model.clone();
    let model_toggle_dd = model.clone();
    let model_clear = model.clone();
    let is_empty = state.new_query_name.is_empty();
    let show_dd = state.show_query_dropdown;

    div()
        .p_2()
        .bg(hsla(215.0 / 360.0, 0.35, 0.16, 0.98))
        .border_b_1()
        .border_color(hsla(215.0 / 360.0, 0.30, 0.25, 1.0))
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_1p5()
                // 输入框 + 下拉切换按钮
                .child(
                    div()
                        .flex_1()
                        .h(px(28.0))
                        .px_2()
                        .rounded_md()
                        .bg(hsla(220.0 / 360.0, 0.45, 0.08, 1.0))
                        .border_1()
                        .border_color(if show_dd {
                            hsla(185.0 / 360.0, 0.90, 0.60, 1.0)
                        } else {
                            hsla(215.0 / 360.0, 0.70, 0.45, 1.0)
                        })
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(if is_empty {
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(crate::ui::components::render_blinking_cursor(hsla(185.0 / 360.0, 0.95, 0.70, 1.0)))
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .font_weight(FontWeight::NORMAL)
                                        .text_color(hsla(215.0 / 360.0, 0.20, 0.50, 1.0))
                                        .child("输入或下拉选择查询..."),
                                )
                        } else {
                            div()
                                .flex()
                                .items_center()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(hsla(185.0 / 360.0, 0.95, 0.70, 1.0))
                                        .child(state.new_query_name.clone()),
                                )
                                .child(crate::ui::components::render_blinking_cursor(hsla(185.0 / 360.0, 0.95, 0.70, 1.0)))
                        })


                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .children(if !is_empty {
                                    Some(
                                        div()
                                            .px_1()
                                            .py_0p5()
                                            .rounded_md()
                                            .hover(|s| s.bg(hsla(0.0, 0.70, 0.35, 0.8)))
                                            .cursor_pointer()
                                            .text_size(px(9.0))
                                            .text_color(hsla(0.0, 0.0, 0.85, 1.0))
                                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                                model_clear.update(cx, |st, cx| {
                                                    st.new_query_name.clear();
                                                    cx.notify();
                                                });
                                            })
                                            .child("✕"),
                                    )
                                } else {
                                    None
                                })
                                .child(
                                    div()
                                        .px_1()
                                        .py_0p5()
                                        .rounded_md()
                                        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.35, 0.25, 0.9)))
                                        .cursor_pointer()
                                        .text_size(px(9.0))
                                        .text_color(hsla(215.0 / 360.0, 0.20, 0.85, 1.0))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            model_toggle_dd.update(cx, |st, cx| {
                                                st.show_query_dropdown = !st.show_query_dropdown;
                                                cx.notify();
                                            });
                                        })
                                        .child("▼"),
                                ),
                        ),
                )
                // 确认按钮
                .child(
                    div()
                        .px_2p5()
                        .h(px(28.0))
                        .rounded_md()
                        .bg(hsla(145.0 / 360.0, 0.65, 0.35, 1.0))
                        .hover(|s| s.bg(hsla(145.0 / 360.0, 0.75, 0.42, 1.0)))
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_confirm.update(cx, |st, cx| {
                                let qry_name = st.new_query_name.trim().to_string();
                                if !qry_name.is_empty() {
                                    if let Some(act) = st.graph.actors.get_mut(actor_idx) {
                                        if !act.queries.contains(&qry_name) {
                                            act.queries.push(qry_name);
                                        }
                                    }
                                }
                                st.is_adding_query = false;
                                st.show_query_dropdown = false;
                                cx.notify();
                            });
                        })
                        .child("✓ 确认"),
                ),
        )
        // 展开的下拉选项列表 (Dropdown List)
        .children(if show_dd {
            Some(
                div()
                    .id("query_dropdown_list")
                    .max_h(px(160.0))
                    .overflow_y_scroll()
                    .bg(hsla(220.0 / 360.0, 0.45, 0.10, 0.98))
                    .border_1()
                    .border_color(hsla(215.0 / 360.0, 0.35, 0.30, 1.0))
                    .rounded_md()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .children(suggestions.into_iter().map(|sug| {
                        let model_item = model.clone();
                        let s_str = sug.to_string();
                        div()
                            .px_2p5()
                            .py_1p5()
                            .border_b_1()
                            .border_color(hsla(220.0 / 360.0, 0.25, 0.16, 0.5))
                            .hover(|s| s.bg(hsla(185.0 / 360.0, 0.50, 0.28, 1.0)))
                            .cursor_pointer()
                            .text_size(px(11.0))
                            .text_color(hsla(215.0 / 360.0, 0.15, 0.90, 1.0))
                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                let val = s_str.clone();
                                model_item.update(cx, |st, cx| {
                                    st.new_query_name = val;
                                    st.show_query_dropdown = false;
                                    cx.notify();
                                });
                            })
                            .child(sug)
                    })),
            )
        } else {
            None
        })
}

fn render_query_row(
    name: &str,
    actor_idx: usize,
    item_idx: usize,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_del = model.clone();
    let name_str = name.to_string();

    div()
        .flex()
        .items_center()
        .justify_between()
        .px_3()
        .py_1p5()
        .border_b_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.14, 0.6))
        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.25, 0.15, 0.8)))
        .text_size(px(11.5))
        .child(name_str)
        .child(
            div()
                .w(px(20.0))
                .h(px(20.0))
                .rounded_md()
                .hover(|s| s.bg(hsla(0.0, 0.70, 0.35, 0.9)))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_del.update(cx, |st, cx| {
                        if let Some(act) = st.graph.actors.get_mut(actor_idx) {
                            if item_idx < act.queries.len() {
                                act.queries.remove(item_idx);
                            }
                        }
                        cx.notify();
                    });
                })
                .child("🗑️"),
        )
}
