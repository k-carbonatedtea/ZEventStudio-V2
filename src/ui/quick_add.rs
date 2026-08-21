use gpui::*;
use gpui::StatefulInteractiveElement;
use crate::model::NodeKind;
use crate::ui::state::AppState;

pub fn render_quick_add_modal(
    canvas_pos: (f32, f32),
    source_connection: Option<(String, String)>,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let node_types = vec![
        (NodeKind::Action, "⚡ Action 动作节点", "执行具体指令/对话/行为"),
        (NodeKind::Switch, "🔀 Switch 分支路由", "条件路由与多分支判断"),
        (NodeKind::Query, "🔍 Query 状态查询", "评估游戏变量与状态判断"),
        (NodeKind::SubFlow, "📦 SubFlow 子流程", "调用封装的独立子节点图"),
        (NodeKind::Fork, "🔱 Fork 并行分流", "分发多路并行执行支流"),
        (NodeKind::Join, "🔗 Join 汇合同步", "等待多路并行支流全部完成"),
        (NodeKind::Math, "🧮 Math 数学逻辑", "数值加减乘除与数据处理"),
        (NodeKind::EntryPoint, "🚩 EntryPoint 入口点", "定义事件流触发源"),
    ];

    let model_close = model.clone();

    div()
        .absolute()
        .inset_0()
        .bg(hsla(0.0, 0.0, 0.0, 0.45))
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_close.update(cx, |st, cx| {
                st.quick_add_modal = None;
                cx.notify();
            });
        })
        .child(
            div()
                .w(px(340.0))
                .rounded_xl()
                .bg(hsla(220.0 / 360.0, 0.35, 0.14, 0.98))
                .border_1()
                .border_color(hsla(215.0 / 360.0, 0.4, 0.45, 1.0))
                .shadow_2xl()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .on_mouse_down(MouseButton::Left, |_, _, _| {}) // stop propagation
                // Header
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
                                .child("➕ 快速新建节点 (Quick Add)"),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(hsla(215.0 / 360.0, 0.3, 0.65, 1.0))
                                .child(if source_connection.is_some() {
                                    "🔗 自动连接到新节点"
                                } else {
                                    "点击立即生成"
                                }),
                        ),
                )
                // Node list
                .child(
                    div()
                        .id("quick_add_scroll_body")
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .max_h(px(320.0))
                        .overflow_y_scroll()
                        .children(node_types.into_iter().map(|(kind, label, desc)| {
                            let k = kind.clone();
                            let model_add = model.clone();
                            let src_conn = source_connection.clone();

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
                                    let node_k = k.clone();
                                    let src = src_conn.clone();
                                    model_add.update(cx, |st, cx| {
                                        let default_title = match &node_k {
                                            NodeKind::Action => "新动作 (Action)",
                                            NodeKind::Switch => "新分支 (Switch)",
                                            NodeKind::Query => "新查询 (Query)",
                                            NodeKind::SubFlow => "新子流程 (SubFlow)",
                                            NodeKind::Fork => "新分流 (Fork)",
                                            NodeKind::Join => "新汇合 (Join)",
                                            NodeKind::Math => "新运算 (Math)",
                                            NodeKind::EntryPoint => "新入口 (EntryPoint)",
                                            NodeKind::Comment => "便签备注 (Comment)",
                                        };

                                        let new_id = st.add_node_at(node_k, default_title, canvas_pos);

                                        // Auto-connect if created from wire drag
                                        if let Some((s_node, s_port)) = src {
                                            if let Some(target_n) = st.graph.find_node(&new_id) {
                                                if let Some(first_in) = target_n.inputs.first() {
                                                    let t_port = first_in.id.clone();
                                                    st.connect_ports(&s_node, &s_port, &new_id, &t_port);
                                                }
                                            }
                                        }

                                        st.quick_add_modal = None;
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_size(px(12.5))
                                        .text_color(rgb(0xffffff))
                                        .child(label),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .text_color(hsla(215.0 / 360.0, 0.25, 0.65, 1.0))
                                        .child(desc),
                                )
                        })),
                ),
        )
}
