use gpui::*;
use crate::model::NodeKind;
use crate::ui::state::AppState;

/// 渲染流程入口列表组件
pub fn render_entry_point_list(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
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
                .text_color(pal.text_primary) // 亮白灰 Text Primary
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
                    pal.card_hover // 选中高亮底色
                } else {
                    pal.card_surface // 浅层蓝灰 Card / Hover
                })
                .border_1()
                .border_color(if is_selected {
                    pal.primary_accent // 遗迹青蓝 Primary Accent
                } else {
                    pal.card_border
                })
                .hover(move |s| {
                    if !is_selected {
                        s.bg(pal.card_hover).border_color(pal.primary_accent)
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
                                .text_color(if is_selected { pal.primary_accent } else { pal.text_primary })
                                .child(title),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .bg(pal.card_surface)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(pal.primary_accent)
                                .child(state.t("sidebar.jump_focus")),
                        ),
                )
        }))
}

