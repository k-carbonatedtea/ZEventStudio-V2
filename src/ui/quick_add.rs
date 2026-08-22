use gpui::*;
use gpui::StatefulInteractiveElement;
use crate::model::{NodeKind, PortDirection};
use crate::ui::components::Toast;
use crate::ui::state::AppState;

/// 渲染双击/连线拉出时的快速新建节点浮层
pub fn render_quick_add_modal(
    state: &AppState,
    canvas_pos: (f32, f32),
    source_connection: Option<(String, String, PortDirection)>,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let node_types = vec![
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

    let model_close = model.clone();
    let status_hint = match &source_connection {
        Some((_, _, PortDirection::Output)) => state.t("quick_add.auto_connect_in"),
        Some((_, _, PortDirection::Input)) => state.t("quick_add.auto_connect_out"),
        None => state.t("quick_add.click_create"),
    };

    let (screen_mx, screen_my) = (
        state.pan.0 + canvas_pos.0 * state.zoom,
        state.pan.1 + canvas_pos.1 * state.zoom,
    );
    let modal_w = 340.0;
    let modal_h = 380.0;
    let pos_x = (screen_mx - 20.0).clamp(20.0, 2400.0 - modal_w);
    let pos_y = (screen_my - 20.0).clamp(20.0, 1400.0 - modal_h);

    div()
        .absolute()
        .inset_0()
        .bg(hsla(0.0, 0.0, 0.0, 0.35))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_close.update(cx, |st, cx| {
                st.quick_add_modal = None;
                cx.notify();
            });
        })
        .child(
            div()
                .absolute()
                .left(px(pos_x))
                .top(px(pos_y))
                .w(px(modal_w))
                .rounded_xl()
                .bg(hsla(220.0 / 360.0, 0.35, 0.14, 0.98))
                .border_1()
                .border_color(hsla(215.0 / 360.0, 0.4, 0.45, 1.0))
                .shadow_2xl()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(MouseButton::Right, |_, _, cx| {
                    cx.stop_propagation();
                })
                // 顶部标题
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .pb_2()
                        .border_b_1()
                        .border_color(hsla(220.0 / 360.0, 0.25, 0.25, 1.0))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(14.0))
                                .text_color(rgb(0xffffff))
                                .child(state.t("quick_add.title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(hsla(45.0 / 360.0, 0.95, 0.65, 1.0))
                                .child(status_hint),
                        ),
                )
                // 节点类型列表
                .child(
                    div()
                        .id("quick_add_scroll_body")
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .max_h(px(320.0))
                        .overflow_y_scroll()
                        .children(node_types.into_iter().map(|(kind, label, desc, def_title)| {
                            let k = kind.clone();
                            let model_add = model.clone();
                            let src_conn = source_connection.clone();
                            let dt = def_title.to_string();

                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .p_2()
                                .rounded_lg()
                                .bg(hsla(220.0 / 360.0, 0.25, 0.18, 0.8))
                                .hover(|s| {
                                    s.bg(hsla(215.0 / 360.0, 0.4, 0.28, 1.0))
                                        .border_color(hsla(145.0 / 360.0, 0.6, 0.5, 1.0))
                                })
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    let node_k = k.clone();
                                    let src = src_conn.clone();
                                    let node_title = dt.clone();

                                    model_add.update(cx, |st, cx| {
                                        let new_id = st.add_node_at(node_k, node_title, canvas_pos);

                                        // 若由引脚长按拽出触发，根据方向自动建立连线
                                        if let Some((s_node, s_port, dir)) = src {
                                            match dir {
                                                PortDirection::Output => {
                                                    // 输出引脚拽出 -> 连接到新节点的第一个输入引脚
                                                    if let Some(target_n) = st.graph.find_node(&new_id) {
                                                        if let Some(first_in) = target_n.inputs.first() {
                                                            let t_port = first_in.id.clone();
                                                            st.connect_ports(&s_node, &s_port, &new_id, &t_port);
                                                        }
                                                    }
                                                }
                                                PortDirection::Input => {
                                                    // 输入引脚拽出 -> 新节点的第一个输出引脚连接回原输入引脚
                                                    if let Some(source_n) = st.graph.find_node(&new_id) {
                                                        if let Some(first_out) = source_n.outputs.first() {
                                                            let out_port = first_out.id.clone();
                                                            st.connect_ports(&new_id, &out_port, &s_node, &s_port);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        st.quick_add_modal = None;
                                        st.toast = Some(Toast::success(st.t("quick_add.created_node")));
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_size(px(12.5))
                                        .text_color(rgb(0xffffff))
                                        .child(label.to_string()),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .text_color(hsla(215.0 / 360.0, 0.25, 0.65, 1.0))
                                        .child(desc.to_string()),
                                )
                        })),
                ),
        )
}


