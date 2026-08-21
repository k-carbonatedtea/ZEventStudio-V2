use gpui::*;
use crate::ui::state::AppState;

/// 渲染顶部工具栏左侧 Branding 区域 (Logo、应用标题、当前图谱信息与统计徽章)
pub fn render_branding(state: &AppState) -> impl IntoElement {
    let node_count = state.graph.nodes.len();
    let edge_count = state.graph.edges.len();
    let node_label = state.t("toolbar.nodes_label");
    let edge_label = state.t("toolbar.edges_label");

    div()
        .flex()
        .items_center()
        .gap_3()
        // Logo & Title
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_size(px(18.0))
                        .child("⚡"),
                )
                .child(
                    div()
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .text_size(px(15.0))
                        .text_color(rgb(0xffffff))
                        .child("NodeFlow Studio"),
                ),
        )
        // Studio Tag
        .child(
            div()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(hsla(215.0 / 360.0, 0.35, 0.20, 1.0))
                .text_size(px(10.5))
                .font_weight(FontWeight::BOLD)
                .text_color(hsla(215.0 / 360.0, 0.5, 0.85, 1.0))
                .child("EVFL Studio"),
        )
        // Active Graph Summary Badge
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .px_2p5()
                .py_1()
                .rounded_md()
                .bg(hsla(145.0 / 360.0, 0.35, 0.18, 0.95))
                .border_1()
                .border_color(hsla(145.0 / 360.0, 0.6, 0.40, 1.0))
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(hsla(145.0 / 360.0, 0.8, 0.90, 1.0))
                .child(format!(
                    "{} ({} {} / {} {})",
                    state.graph.name, node_count, node_label, edge_count, edge_label
                )),
        )
}
