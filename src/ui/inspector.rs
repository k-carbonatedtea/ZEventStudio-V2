use gpui::*;
use gpui::StatefulInteractiveElement;
use crate::ui::state::AppState;

pub fn render_inspector(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let selected_node = state
        .selected_node_id
        .as_ref()
        .and_then(|id| state.graph.find_node(id));

    div()
        .w(px(280.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(hsla(220.0 / 360.0, 0.35, 0.10, 0.98))
        .border_l_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.20, 1.0))
        // Inspector Header
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .py_3()
                .border_b_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.20, 1.0))
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(rgb(0xffffff))
                        .child("⚙️ 属性检查器 (Inspector)"),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(hsla(215.0 / 360.0, 0.3, 0.25, 1.0))
                        .text_color(hsla(215.0 / 360.0, 0.4, 0.85, 1.0))
                        .child(if selected_node.is_some() { "已选中节点" } else { "全局信息" }),
                ),
        )
        // Inspector Body
        .child(
            div()
                .id("inspector_scroll_body")
                .flex_1()
                .p_4()
                .overflow_y_scroll()
                .child(if let Some(node) = selected_node {
                    let model_del = model.clone();
                    let (h, s, l) = node.kind.header_color();

                    div()
                        .flex()
                        .flex_col()
                        .gap_3p5()
                        // Node Header Info
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
                                        .child(format!("ID: {}", node.id)),
                                ),
                        )
                        // Coordinates
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
                                        .child("画布坐标 (Position)"),
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
                        // Ports Summary
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
                                            "端口引脚 (输入: {} | 输出: {})",
                                            node.inputs.len(),
                                            node.outputs.len()
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
                                                .child(format!("◀ 输入: {}", p.name))
                                        }))
                                        .children(node.outputs.iter().map(|p| {
                                            div()
                                                .px_2()
                                                .py_1()
                                                .rounded_md()
                                                .bg(hsla(45.0 / 360.0, 0.25, 0.15, 0.8))
                                                .text_size(px(10.5))
                                                .text_color(hsla(45.0 / 360.0, 0.7, 0.8, 1.0))
                                                .child(format!("▶ 输出: {}", p.name))
                                        })),
                                ),
                        )
                        // Properties Table
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1p5()
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(hsla(215.0 / 360.0, 0.2, 0.7, 1.0))
                                        .child("参数键值表 (Properties)"),
                                )
                                .children(if node.properties.is_empty() {
                                    vec![div()
                                        .p_2()
                                        .rounded_md()
                                        .bg(hsla(220.0 / 360.0, 0.2, 0.15, 1.0))
                                        .text_size(px(10.5))
                                        .text_color(hsla(215.0 / 360.0, 0.2, 0.5, 1.0))
                                        .child("暂无自定义参数")]
                                } else {
                                    node.properties
                                        .iter()
                                        .map(|prop| {
                                            div()
                                                .p_2()
                                                .rounded_md()
                                                .bg(hsla(220.0 / 360.0, 0.25, 0.15, 1.0))
                                                .border_1()
                                                .border_color(hsla(215.0 / 360.0, 0.25, 0.22, 1.0))
                                                .flex()
                                                .flex_col()
                                                .gap_0p5()
                                                .child(
                                                    div()
                                                        .text_size(px(10.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(hsla(215.0 / 360.0, 0.3, 0.65, 1.0))
                                                        .child(prop.key.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(11.0))
                                                        .text_color(hsla(45.0 / 360.0, 0.8, 0.8, 1.0))
                                                        .child(prop.val.clone()),
                                                )
                                        })
                                        .collect()
                                }),
                        )
                        // Delete Node Button
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
                                .child("🗑️ 删除此节点 (Delete)"),
                        )
                        .into_any_element()
                } else {
                    // Global graph summary
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(rgb(0xffffff))
                                .child(state.graph.name.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(hsla(215.0 / 360.0, 0.25, 0.70, 1.0))
                                .child(state.graph.description.clone()),
                        )
                        .child(
                            div()
                                .p_3()
                                .rounded_lg()
                                .bg(hsla(220.0 / 360.0, 0.25, 0.15, 1.0))
                                .flex()
                                .flex_col()
                                .gap_1p5()
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(rgb(0xffffff))
                                        .child(format!("📊 节点总数：{} 个", state.graph.nodes.len())),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(rgb(0xffffff))
                                        .child(format!("🔗 连线总数：{} 条", state.graph.edges.len())),
                                ),
                        )
                        .child(
                            div()
                                .p_3()
                                .rounded_lg()
                                .bg(hsla(215.0 / 360.0, 0.35, 0.15, 0.6))
                                .text_size(px(10.5))
                                .text_color(hsla(215.0 / 360.0, 0.4, 0.8, 1.0))
                                .child("💡 提示：点击画布中的任意节点可在本面板查看与编辑详细参数；拖拽引脚可连线；右键画布可快速新建节点。"),
                        )
                        .into_any_element()
                }),
        )
}
