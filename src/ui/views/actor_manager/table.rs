use gpui::*;
use crate::ui::state::AppState;

/// 渲染上半区角色列表表格
pub fn render_actors_table(
    state: &AppState,
    actors: &[crate::model::actor::ActorData],
    selected_idx: usize,
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
                .child(div().w(px(260.0)).child(state.t("actors_view.th_name")))
                .child(div().w(px(100.0)).child(state.t("actors_view.th_sub_name")))
                .child(div().w(px(120.0)).child(state.t("actors_view.th_argument_name")))
                .child(div().w(px(140.0)).child(state.t("actors_view.th_argument_entry")))
                .child(div().w(px(80.0)).child(state.t("actors_view.th_actions_count")))
                .child(div().w(px(80.0)).child(state.t("actors_view.th_queries_count")))
                .child(div().flex_1().child(state.t("actors_view.th_operations"))),
        )
        // 表体行
        .children(actors.iter().enumerate().map(|(idx, actor)| {
            let is_sel = idx == selected_idx;
            let model_row = model.clone();
            let model_del = model.clone();

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
                    model_row.update(cx, |st, cx| {
                        st.selected_actor_idx = idx;
                        cx.notify();
                    });
                })
                .text_size(px(12.0))
                .text_color(if is_sel {
                    hsla(0.0, 0.0, 1.0, 1.0)
                } else {
                    hsla(215.0 / 360.0, 0.20, 0.80, 1.0)
                })
                .child(div().w(px(260.0)).font_weight(FontWeight::BOLD).child(actor.name.clone()))
                .child(div().w(px(100.0)).child(if actor.sub_name.is_empty() { "-".to_string() } else { actor.sub_name.clone() }))
                .child(div().w(px(120.0)).child(if actor.argument_name.is_empty() { "-".to_string() } else { actor.argument_name.clone() }))
                .child(div().w(px(140.0)).child(if actor.argument_entry_point.is_empty() { "-".to_string() } else { actor.argument_entry_point.clone() }))
                .child(div().w(px(80.0)).child(format!("{}", actor.actions.len())))
                .child(div().w(px(80.0)).child(format!("{}", actor.queries.len())))

                .child(
                    div()
                        .flex_1()
                        .child(
                            div()
                                .w(px(24.0))
                                .h(px(24.0))
                                .rounded_md()
                                .hover(|s| s.bg(hsla(0.0, 0.70, 0.35, 0.9)))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_del.update(cx, |st, cx| {
                                        if idx < st.graph.actors.len() {
                                            st.graph.actors.remove(idx);
                                            if st.selected_actor_idx >= st.graph.actors.len() && !st.graph.actors.is_empty() {
                                                st.selected_actor_idx = st.graph.actors.len() - 1;
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                                .child("🗑️"),
                        ),
                )
        }))
}
