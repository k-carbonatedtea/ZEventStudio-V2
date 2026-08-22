use gpui::*;
use crate::model::NodeKind;
use crate::ui::state::AppState;

/// 渲染角色管理侧边栏面板 (Actor Manager: 角色概览、动作/查询列表、关联事件跳转)
pub fn render_actors_manager(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let search_val = state.search_query.to_lowercase();
    let actors = state.graph.get_actors_summary();

    let filtered_actors: Vec<_> = actors
        .into_iter()
        .filter(|a| {
            if search_val.is_empty() {
                true
            } else {
                a.name.to_lowercase().contains(&search_val)
                    || a.actions.iter().any(|act| act.to_lowercase().contains(&search_val))
                    || a.queries.iter().any(|q| q.to_lowercase().contains(&search_val))
            }
        })
        .collect();

    let total_count = filtered_actors.len();

    div()
        .flex()
        .flex_col()
        .size_full()
        .p_2p5()
        .gap_2()
        // 顶部标题与新建按钮
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_1()
                .pb_1()
                .border_b_1()
                .border_color(hsla(240.0 / 360.0, 0.06, 0.16, 1.0))
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.0))
                        .text_color(rgb(0xffffff))
                        .child(state.t("sidebar.actors_title")),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_full()
                        .bg(hsla(271.0 / 360.0, 0.40, 0.25, 0.9))
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hsla(271.0 / 360.0, 0.85, 0.75, 1.0))
                        .child(format!("{} Actors", total_count)),
                ),
        )
        // 角色列表
        .child(
            div()
                .id("actors_scroll_body")
                .flex_1()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap_2()
                .children(if filtered_actors.is_empty() {
                    vec![
                        div()
                            .p_4()
                            .text_size(px(11.5))
                            .text_color(hsla(240.0 / 360.0, 0.05, 0.55, 1.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(state.t("sidebar.no_actors_found"))
                            .into_any_element(),
                    ]
                } else {
                    filtered_actors
                        .into_iter()
                        .map(|actor| {
                            let first_event_id = actor.event_ids.first().cloned();
                            let model_focus = model.clone();
                            let model_add_node = model.clone();
                            let actor_name = actor.name.clone();
                            let act_count = actor.actions.len();
                            let qry_count = actor.queries.len();
                            let ref_count = actor.event_ids.len();

                            let icon = if actor.name.contains("System") {
                                "⚙️"
                            } else if actor.name.contains("Player") {
                                "⚔️"
                            } else if actor.name.contains("Npc") {
                                "💬"
                            } else if actor.name.contains("Enemy") {
                                "💀"
                            } else {
                                "🎭"
                            };

                            div()
                                .p_2p5()
                                .rounded_lg()
                                .bg(hsla(240.0 / 360.0, 0.06, 0.11, 1.0))
                                .border_1()
                                .border_color(hsla(240.0 / 360.0, 0.06, 0.20, 1.0))
                                .hover(|s| {
                                    s.bg(hsla(240.0 / 360.0, 0.06, 0.16, 1.0))
                                        .border_color(hsla(271.0 / 360.0, 0.60, 0.55, 1.0))
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
                                                .child(div().text_size(px(12.0)).child(icon))
                                                .child(
                                                    div()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_size(px(12.5))
                                                        .text_color(rgb(0xffffff))
                                                        .child(actor.name.clone()),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(hsla(215.0 / 360.0, 0.30, 0.22, 1.0))
                                                .text_size(px(9.5))
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(hsla(45.0 / 360.0, 0.95, 0.65, 1.0))
                                                .child(format!("{} nodes", ref_count)),
                                        ),
                                )
                                // 统计徽章栏
                                .child(
                                    div()
                                        .mt_1p5()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .child(
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(hsla(200.0 / 360.0, 0.35, 0.25, 0.9))
                                                .text_size(px(9.0))
                                                .text_color(hsla(200.0 / 360.0, 0.85, 0.70, 1.0))
                                                .child(format!("{} Actions", act_count)),
                                        )
                                        .child(
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(hsla(185.0 / 360.0, 0.35, 0.25, 0.9))
                                                .text_size(px(9.0))
                                                .text_color(hsla(185.0 / 360.0, 0.85, 0.70, 1.0))
                                                .child(format!("{} Queries", qry_count)),
                                        ),
                                )
                                // 动作胶囊列表
                                .child(
                                    div()
                                        .mt_2()
                                        .flex()
                                        .flex_wrap()
                                        .gap_1()
                                        .children(actor.actions.iter().take(4).map(|act| {
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(hsla(220.0 / 360.0, 0.35, 0.20, 0.9))
                                                .text_size(px(9.0))
                                                .text_color(hsla(215.0 / 360.0, 0.20, 0.75, 1.0))
                                                .child(format!("⚡ {}", act))
                                        }))
                                        .children(actor.queries.iter().take(2).map(|q| {
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(hsla(220.0 / 360.0, 0.35, 0.20, 0.9))
                                                .text_size(px(9.0))
                                                .text_color(hsla(185.0 / 360.0, 0.70, 0.75, 1.0))
                                                .child(format!("🔍 {}", q))
                                        })),
                                )
                                // 底部快捷操作栏 (聚焦节点 / 创建此角色事件)
                                .child(
                                    div()
                                        .mt_2()
                                        .pt_1p5()
                                        .border_t_1()
                                        .border_color(hsla(220.0 / 360.0, 0.25, 0.20, 1.0))
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(hsla(215.0 / 360.0, 0.30, 0.25, 0.9))
                                                .hover(|s| s.bg(hsla(215.0 / 360.0, 0.45, 0.35, 1.0)))
                                                .cursor_pointer()
                                                .text_size(px(9.5))
                                                .text_color(rgb(0xffffff))
                                                .on_mouse_down(MouseButton::Left, {
                                                    let fe_id = first_event_id.clone();
                                                    move |_, _, cx| {
                                                        if let Some(id) = &fe_id {
                                                            let id = id.clone();
                                                            model_focus.update(cx, |st, cx| {
                                                                st.focus_node(&id);
                                                                cx.notify();
                                                            });
                                                        }
                                                    }
                                                })
                                                .child(state.t("sidebar.jump_focus")),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(hsla(200.0 / 360.0, 0.35, 0.25, 0.9))
                                                .hover(|s| s.bg(hsla(200.0 / 360.0, 0.50, 0.35, 1.0)))
                                                .cursor_pointer()
                                                .text_size(px(9.5))
                                                .text_color(hsla(200.0 / 360.0, 0.90, 0.80, 1.0))
                                                .on_mouse_down(MouseButton::Left, {
                                                    let act_name = actor_name.clone();
                                                    move |_, _, cx| {
                                                        model_add_node.update(cx, |st, cx| {
                                                            let (cx_pos, cy_pos) = st.screen_to_canvas(500.0, 300.0);
                                                            let node_id = st.add_node_at(NodeKind::Action, "", (cx_pos, cy_pos));
                                                            if let Some(n) = st.graph.find_node_mut(&node_id) {
                                                                n.subtitle = Some(format!("{} :: Demo_Action", act_name));
                                                            }
                                                            st.focus_node(&node_id);
                                                            cx.notify();
                                                        });
                                                    }
                                                })
                                                .child("+ Action"),
                                        ),
                                )
                                .into_any_element()
                        })
                        .collect()
                }),
        )
}
