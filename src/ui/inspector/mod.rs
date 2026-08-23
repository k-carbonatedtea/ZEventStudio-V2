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
        .bg(rgb(0x161922)) // 蓝灰表面 Panel Surface
        .border_l_1()
        .border_color(rgb(0x232838))
        // 1. 顶部 Header
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .py_3()
                .border_b_1()
                .border_color(rgb(0x232838))
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(rgb(0xF1F5F9)) // 亮白灰 Text Primary
                        .child(format!("⚙️ {}", state.t("inspector.title"))),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(rgb(0x1E2330)) // 浅层蓝灰 Card / Hover
                        .border_1()
                        .border_color(rgb(0x2D3548))
                        .text_color(rgb(0x00E5BA)) // 遗迹青蓝 Primary Accent
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
