use gpui::*;
use crate::ui::state::AppState;

/// 渲染语义剧情分区/分组容器背景框图层 (Blueprint-style Comment & Group Backdrops)
pub fn render_groups_layer(state: &AppState) -> impl IntoElement {
    let pan = state.pan;
    let zoom = state.zoom;

    if zoom < 0.22 {
        return div().into_any_element();
    }

    let groups = state.graph.get_subgraph_groups();
    let nodes_label = state.t("group.nodes_count");

    div()
        .absolute()
        .inset_0()
        .children(groups.into_iter().filter_map(|g| {
            let (min_x, min_y, max_x, max_y) = g.bounds;
            let screen_x = pan.0 + min_x * zoom;
            let screen_y = pan.1 + min_y * zoom;
            let screen_w = (max_x - min_x) * zoom;
            let screen_h = (max_y - min_y) * zoom;

            // 视口边界裁剪
            if screen_x + screen_w < -100.0
                || screen_x > 2600.0
                || screen_y + screen_h < -100.0
                || screen_y > 1600.0
            {
                return None;
            }

            let border_radius = (14.0 * zoom).clamp(4.0, 16.0);
            let font_size = (11.0 * zoom).clamp(7.0, 14.0);

            Some(
                div()
                    .absolute()
                    .left(px(screen_x))
                    .top(px(screen_y))
                    .w(px(screen_w))
                    .h(px(screen_h))
                    .rounded(px(border_radius))
                    .bg(hsla(220.0 / 360.0, 0.25, 0.08, 0.45))
                    .border_1()
                    .border_color(hsla(215.0 / 360.0, 0.25, 0.25, 0.60))
                    // 顶部流程分区徽章 (Group Header Tag)
                    .child(
                        div()
                            .absolute()
                            .left(px(14.0 * zoom))
                            .top(px(8.0 * zoom))
                            .flex()
                            .items_center()
                            .gap(px(4.0 * zoom))
                            .px(px(8.0 * zoom))
                            .py(px(2.5 * zoom))
                            .rounded_md()
                            .bg(hsla(220.0 / 360.0, 0.35, 0.16, 0.90))
                            .border_1()
                            .border_color(hsla(215.0 / 360.0, 0.35, 0.35, 0.80))
                            .shadow_sm()
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_size(px(font_size))
                                    .text_color(hsla(215.0 / 360.0, 0.15, 0.92, 1.0))
                                    .child(g.title),
                            )
                            .child(
                                div()
                                    .text_size(px(font_size * 0.9))
                                    .text_color(hsla(215.0 / 360.0, 0.25, 0.65, 1.0))
                                    .child(format!("({} {})", g.node_count, nodes_label)),
                            ),
                    ),
            )
        }))
        .into_any_element()
}
