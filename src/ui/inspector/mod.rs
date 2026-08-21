pub mod graph_stats;
pub mod node_detail;
pub mod prop_list;

use gpui::*;
use graph_stats::render_graph_stats;
use node_detail::render_node_detail;
use crate::ui::state::AppState;

/// 渲染属性检查器主面板
pub fn render_inspector(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let selected_node = state
        .selected_node_id
        .as_ref()
        .and_then(|id| state.graph.find_node(id));

    let header_status_text = if selected_node.is_some() {
        state.t("inspector.selected_node")
    } else {
        state.t("inspector.global_info")
    };

    div()
        .w(px(280.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(hsla(220.0 / 360.0, 0.35, 0.10, 0.98))
        .border_l_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.20, 1.0))
        // 1. 顶部 Header
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
                        .child(format!("⚙️ {}", state.t("inspector.title"))),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(hsla(215.0 / 360.0, 0.3, 0.25, 1.0))
                        .text_color(hsla(215.0 / 360.0, 0.4, 0.85, 1.0))
                        .child(header_status_text),
                ),
        )
        // 2. 面板内容区 (支持独立滚动)
        .child(
            div()
                .id("inspector_scroll_body")
                .flex_1()
                .p_4()
                .overflow_y_scroll()
                .child(if let Some(node) = selected_node {
                    render_node_detail(state, node, model).into_any_element()
                } else {
                    render_graph_stats(state).into_any_element()
                }),
        )
}
