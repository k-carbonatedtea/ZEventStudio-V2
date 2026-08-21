use gpui::*;
use super::prop_list::render_prop_list;
use crate::model::NodeData;
use crate::ui::state::AppState;

/// 渲染属性检查器中的选中节点详细信息面板
pub fn render_node_detail(
    state: &AppState,
    node: &NodeData,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_del = model.clone();
    let (h, s, l) = node.kind.header_color();

    div()
        .flex()
        .flex_col()
        .gap_3p5()
        // 1. 节点基本卡片 (图标、标题、ID、Actor/Scope)
        .child(
            div()
                .p_3()
                .rounded_lg()
                .bg(hsla(h, s, l, 0.15))
                .border_1()
                .border_color(hsla(h, s, l, 0.45))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(div().text_size(px(14.0)).child(node.kind.icon()))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(rgb(0xffffff))
                                .child(node.title.clone()),
                        ),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(hsla(215.0 / 360.0, 0.25, 0.65, 1.0))
                        .child(format!("{}: {}", state.t("inspector.node_id"), node.id)),
                )
                .children(node.subtitle.as_ref().map(|sub| {
                    div()
                        .text_size(px(10.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(hsla(45.0 / 360.0, 0.85, 0.65, 1.0))
                        .child(format!("Actor / Scope: {}", sub))
                })),
        )
        // 2. 画布坐标 (Canvas Position)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hsla(215.0 / 360.0, 0.2, 0.7, 1.0))
                        .child(state.t("inspector.position")),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .px_2p5()
                                .py_1()
                                .rounded_md()
                                .bg(hsla(220.0 / 360.0, 0.25, 0.15, 1.0))
                                .text_size(px(11.0))
                                .text_color(rgb(0xffffff))
                                .child(format!("X: {:.1}", node.pos.0)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .px_2p5()
                                .py_1()
                                .rounded_md()
                                .bg(hsla(220.0 / 360.0, 0.25, 0.15, 1.0))
                                .text_size(px(11.0))
                                .text_color(rgb(0xffffff))
                                .child(format!("Y: {:.1}", node.pos.1)),
                        ),
                ),
        )
        // 3. 端口引脚摘要 (Port Sockets Summary)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hsla(215.0 / 360.0, 0.2, 0.7, 1.0))
                        .child(format!(
                            "{} ({} / {})",
                            state.t("inspector.ports_summary"),
                            format!("{}: {}", state.t("inspector.port_in"), node.inputs.len()),
                            format!("{}: {}", state.t("inspector.port_out"), node.outputs.len())
                        )),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(node.inputs.iter().map(|p| {
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(hsla(145.0 / 360.0, 0.25, 0.15, 0.8))
                                .text_size(px(10.5))
                                .text_color(hsla(145.0 / 360.0, 0.7, 0.8, 1.0))
                                .child(format!("{}: {}", state.t("inspector.port_in"), p.name))
                        }))
                        .children(node.outputs.iter().map(|p| {
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(hsla(45.0 / 360.0, 0.25, 0.15, 0.8))
                                .text_size(px(10.5))
                                .text_color(hsla(45.0 / 360.0, 0.7, 0.8, 1.0))
                                .child(format!("{}: {}", state.t("inspector.port_out"), p.name))
                        })),
                ),
        )
        // 4. EVFL 参数属性列表
        .child(render_prop_list(state, &node.properties))
        // 5. 删除节点按钮
        .child(
            div()
                .mt_2()
                .py_2()
                .rounded_lg()
                .bg(hsla(0.0, 0.6, 0.35, 0.9))
                .hover(|s| s.bg(hsla(0.0, 0.75, 0.45, 1.0)))
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(12.0))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_del.update(cx, |st, cx| {
                        st.delete_selected_node();
                        cx.notify();
                    });
                })
                .child(state.t("inspector.delete_node")),
        )
}
