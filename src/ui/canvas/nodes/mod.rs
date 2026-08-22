pub mod entry_card;
pub mod event_card;
pub mod socket;

use gpui::*;
use entry_card::render_entry_card;
use event_card::render_event_card;
use crate::model::{NodeData, NodeKind};
use crate::ui::state::{AppState, DragState};

/// 渲染画布节点图层 (带视口边界裁剪 Viewport Culling 与链路追踪全局缓存)
pub fn render_nodes_layer(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pan = state.pan;
    let zoom = state.zoom;
    let selected_id = state.selected_node_id.clone();
    let connecting_state = state.drag_state.clone();
    let expand_all = state.expand_all_params;

    // 链路追踪 (Flow Tracing) 单帧一次性计算，避免每个节点重复做 O(N) 图遍历
    let active_node = state.selected_node_id.as_deref().or(state.hovered_node_id.as_deref());
    let active_path_nodes: Option<std::collections::HashSet<String>> = active_node.map(|target| {
        let (up, down) = state.graph.get_upstream_and_downstream(target);
        let mut set = up;
        set.extend(down);
        set.insert(target.to_string());
        set
    });

    // 视口边界裁剪 (Viewport Culling): 剔除屏幕外的节点
    let view_min_x = -150.0;
    let view_min_y = -150.0;
    let view_max_x = 2600.0;
    let view_max_y = 1600.0;

    div()
        .absolute()
        .inset_0()
        .children(state.graph.nodes.iter().filter_map(|node| {
            let screen_x = pan.0 + node.pos.0 * zoom;
            let screen_y = pan.1 + node.pos.1 * zoom;
            let node_w = (node.size.0 * zoom).max(16.0);
            let node_h = (node.size.1 * zoom).max(12.0);

            if screen_x + node_w < view_min_x
                || screen_x > view_max_x
                || screen_y + node_h < view_min_y
                || screen_y > view_max_y
            {
                return None;
            }

            Some(render_single_node(
                state,
                node,
                (screen_x, screen_y),
                (node_w, node_h),
                zoom,
                &selected_id,
                &connecting_state,
                active_path_nodes.as_ref(),
                expand_all,
                model,
            ))
        }))
}

fn render_single_node(
    state: &AppState,
    node: &NodeData,
    screen_pos: (f32, f32),
    node_size: (f32, f32),
    zoom: f32,
    selected_id: &Option<String>,
    connecting_state: &DragState,
    active_path_nodes: Option<&std::collections::HashSet<String>>,
    expand_all: bool,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let is_selected = selected_id.as_deref() == Some(&node.id);

    if node.kind == NodeKind::EntryPoint {
        render_entry_card(
            state,
            node,
            screen_pos,
            node_size,
            zoom,
            is_selected,
            connecting_state,
            active_path_nodes,
            model,
        )
        .into_any_element()
    } else {
        render_event_card(
            state,
            node,
            screen_pos,
            node_size,
            zoom,
            is_selected,
            connecting_state,
            active_path_nodes,
            expand_all,
            model,
        )
        .into_any_element()
    }
}
