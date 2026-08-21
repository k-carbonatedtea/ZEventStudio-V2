use gpui::*;
use crate::model::NodeKind;
use crate::ui::state::AppState;

/// 渲染左侧节点组件库面板
pub fn render_node_palette(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let node_items = vec![
        (
            NodeKind::Action,
            state.t("node_kind.action_title"),
            state.t("node_kind.action_desc"),
            state.t("default_node.action"),
        ),
        (
            NodeKind::Switch,
            state.t("node_kind.switch_title"),
            state.t("node_kind.switch_desc"),
            state.t("default_node.switch"),
        ),
        (
            NodeKind::Query,
            state.t("node_kind.query_title"),
            state.t("node_kind.query_desc"),
            state.t("default_node.query"),
        ),
        (
            NodeKind::SubFlow,
            state.t("node_kind.subflow_title"),
            state.t("node_kind.subflow_desc"),
            state.t("default_node.subflow"),
        ),
        (
            NodeKind::Fork,
            state.t("node_kind.fork_title"),
            state.t("node_kind.fork_desc"),
            state.t("default_node.fork"),
        ),
        (
            NodeKind::Join,
            state.t("node_kind.join_title"),
            state.t("node_kind.join_desc"),
            state.t("default_node.join"),
        ),
        (
            NodeKind::Math,
            state.t("node_kind.math_title"),
            state.t("node_kind.math_desc"),
            state.t("default_node.math"),
        ),
        (
            NodeKind::EntryPoint,
            state.t("node_kind.entry_title"),
            state.t("node_kind.entry_desc"),
            state.t("default_node.entry"),
        ),
    ];

    let query = state.search_query.to_lowercase();
    let filtered_items: Vec<_> = node_items
        .into_iter()
        .filter(|(_, title, desc, _)| {
            query.is_empty()
                || title.to_lowercase().contains(&query)
                || desc.to_lowercase().contains(&query)
        })
        .collect();

    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .children(filtered_items.into_iter().map(|(kind, title, desc, default_title)| {
            let k = kind.clone();
            let def_t = default_title.to_string();
            let model_add = model.clone();

            div()
                .p_2p5()
                .rounded_lg()
                .bg(hsla(220.0 / 360.0, 0.3, 0.14, 0.8))
                .border_1()
                .border_color(hsla(215.0 / 360.0, 0.25, 0.25, 0.8))
                .hover(|s| {
                    s.bg(hsla(215.0 / 360.0, 0.35, 0.22, 1.0))
                        .border_color(hsla(145.0 / 360.0, 0.6, 0.45, 1.0))
                })
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    let node_k = k.clone();
                    let dt = def_t.clone();
                    model_add.update(cx, |st, cx| {
                        let (cx_pos, cy_pos) = st.screen_to_canvas(500.0, 300.0);
                        st.add_node_at(node_k, dt, (cx_pos, cy_pos));
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(12.0))
                        .text_color(rgb(0xffffff))
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .mt_0p5()
                        .text_size(px(10.0))
                        .text_color(hsla(215.0 / 360.0, 0.25, 0.65, 1.0))
                        .child(desc.to_string()),
                )
        }))
}

