use gpui::*;
use crate::model::NodeKind;
use crate::ui::state::AppState;

/// 渲染流程入口列表组件
pub fn render_entry_point_list(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let entry_points: Vec<(String, String, Option<String>)> = state
        .graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::EntryPoint)
        .map(|n| (n.id.clone(), n.title.clone(), n.subtitle.clone()))
        .collect();

    let selected_id = state.selected_node_id.clone();

    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(hsla(215.0 / 360.0, 0.3, 0.70, 1.0))
                .child(format!("{} ({})", state.t("sidebar.entry_points_title"), entry_points.len())),
        )
        .children(entry_points.into_iter().map(|(id, title, _)| {
            let is_selected = selected_id.as_deref() == Some(&id);
            let target_id = id.clone();
            let model_focus = model.clone();

            div()
                .p_2p5()
                .rounded_lg()
                .bg(if is_selected {
                    hsla(0.0, 0.65, 0.25, 0.95)
                } else {
                    hsla(240.0 / 360.0, 0.06, 0.11, 1.0)
                })
                .border_1()
                .border_color(if is_selected {
                    hsla(0.0, 0.85, 0.55, 1.0)
                } else {
                    hsla(240.0 / 360.0, 0.06, 0.20, 1.0)
                })
                .hover(|s| {
                    if !is_selected {
                        s.bg(hsla(240.0 / 360.0, 0.06, 0.16, 1.0))
                    } else {
                        s
                    }
                })
                .cursor_pointer()

                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    let tid = target_id.clone();
                    model_focus.update(cx, |st, cx| {
                        st.focus_node(&tid);
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.0))
                                .text_color(rgb(0xffffff))
                                .child(title),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .bg(hsla(0.0, 0.6, 0.40, 0.6))
                                .text_size(px(9.5))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xffffff))
                                .child(state.t("sidebar.jump_focus")),
                        ),
                )
        }))
}

