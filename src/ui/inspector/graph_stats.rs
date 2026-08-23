use gpui::*;
use crate::model::NodeKind;
use crate::ui::state::AppState;

/// 渲染属性检查器中的全局图谱拓扑统计与交互指南卡片
pub fn render_graph_stats(state: &AppState) -> impl IntoElement {
    let total_nodes = state.graph.nodes.len();
    let total_edges = state.graph.edges.len();
    let ep_count = state.graph.nodes.iter().filter(|n| n.kind == NodeKind::EntryPoint).count();
    let action_count = state.graph.nodes.iter().filter(|n| n.kind == NodeKind::Action).count();
    let switch_count = state.graph.nodes.iter().filter(|n| n.kind == NodeKind::Switch).count();

    div()
        .flex()
        .flex_col()
        .gap_3p5()
        // 1. 图谱基本概览
        .child(
            div()
                .p_3()
                .rounded_lg()
                .bg(rgb(0x1E2330)) // 浅层蓝灰 Card / Hover
                .border_1()
                .border_color(rgb(0x2D3548))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(rgb(0xF1F5F9)) // 亮白灰 Text Primary
                        .child(state.graph.name.clone()),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(rgb(0x94A3B8)) // 蓝灰白 Text Secondary
                        .child(state.graph.description.clone()),
                ),
        )
        // 2. 拓扑统计
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xF1F5F9))
                        .child(state.t("inspector.topology_stats")),
                )
                .child(
                    div()
                        .p_2p5()
                        .rounded_lg()
                        .bg(rgb(0x1E2330)) // 浅层蓝灰 Card / Hover
                        .border_1()
                        .border_color(rgb(0x2D3548))
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .child(render_stat_row(state.t("inspector.total_nodes"), format!("{}", total_nodes)))
                        .child(render_stat_row(state.t("inspector.total_edges"), format!("{}", total_edges)))
                        .child(render_stat_row(state.t("inspector.stat_entry_points"), format!("{}", ep_count)))
                        .child(render_stat_row(state.t("inspector.stat_actions"), format!("{}", action_count)))
                        .child(render_stat_row(state.t("inspector.stat_switches"), format!("{}", switch_count))),
                ),
        )
        // 3. 交互操作提示
        .child(
            div()
                .p_3()
                .rounded_lg()
                .bg(rgba(0x00e5ba15))
                .border_1()
                .border_color(rgba(0x00e5ba40))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(11.5))
                        .text_color(rgb(0x00E5BA)) // 遗迹青蓝 Primary Accent
                        .child(state.t("inspector.tips_title")),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(rgb(0x94A3B8))
                        .child(state.t("inspector.tip_entry_jump")),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(rgb(0x94A3B8))
                        .child(state.t("inspector.tip_pan_zoom")),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(rgb(0x94A3B8))
                        .child(state.t("inspector.tip_quick_search")),
                ),
        )
}

fn render_stat_row(label: &str, val: String) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(11.0))
        .child(
            div()
                .text_color(hsla(215.0 / 360.0, 0.2, 0.65, 1.0))
                .child(label.to_string()),
        )
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xffffff))
                .child(val),
        )
}
