use gpui::*;
use crate::model::NodeKind;
use crate::ui::state::AppState;

pub fn render_node_palette(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let node_items = vec![
        (NodeKind::Action, "⚡ Action 动作", "执行对话/音效/道具等具体行为"),
        (NodeKind::Switch, "🔀 Switch 分支", "多条件动态分支路由"),
        (NodeKind::Query, "🔍 Query 查询", "检查状态与游戏条件"),
        (NodeKind::SubFlow, "📦 SubFlow 子流程", "调用已封装的子节点图"),
        (NodeKind::Fork, "🔱 Fork 并行", "分发多条并行流水线"),
        (NodeKind::Join, "🔗 Join 汇合", "汇合同步多条流水线"),
        (NodeKind::Math, "🧮 Math 运算", "数据计算与逻辑判断"),
        (NodeKind::EntryPoint, "🚩 EntryPoint 入口", "定义事件流触发源"),
    ];

    let query = state.search_query.to_lowercase();
    let filtered_items: Vec<_> = node_items
        .into_iter()
        .filter(|(_, title, desc)| {
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
        .children(filtered_items.into_iter().map(|(kind, title, desc)| {
            let k = kind.clone();
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
                    model_add.update(cx, |st, cx| {
                        // Place near canvas center
                        let (cx_pos, cy_pos) = st.screen_to_canvas(500.0, 300.0);
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
                        st.add_node_at(node_k, default_title, (cx_pos, cy_pos));
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(12.0))
                        .text_color(rgb(0xffffff))
                        .child(title),
                )
                .child(
                    div()
                        .mt_0p5()
                        .text_size(px(10.0))
                        .text_color(hsla(215.0 / 360.0, 0.25, 0.65, 1.0))
                        .child(desc),
                )
        }))
}
