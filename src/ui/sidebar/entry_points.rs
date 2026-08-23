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
                .text_color(rgb(0xF1F5F9)) // 亮白灰 Text Primary
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
                    rgb(0x252C3D) // 选中高亮底色
                } else {
                    rgb(0x1E2330) // 浅层蓝灰 Card / Hover
                })
                .border_1()
                .border_color(if is_selected {
                    rgb(0x00E5BA) // 遗迹青蓝 Primary Accent
                } else {
                    rgb(0x2D3548)
                })
                .hover(|s| {
                    if !is_selected {
                        s.bg(rgb(0x252C3D)).border_color(rgb(0x00E5BA))
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
                                .text_color(if is_selected { rgb(0x00E5BA) } else { rgb(0xF1F5F9) })
                                .child(title),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .bg(rgba(0x00e5ba20))
                                .text_size(px(9.5))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x00E5BA))
                                .child(state.t("sidebar.jump_focus")),
                        ),
                )
        }))
}

