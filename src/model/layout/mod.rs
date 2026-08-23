pub mod brandes_kopf;
pub mod config;
pub mod crossings;
pub mod ranks;
pub mod sugiyama;
pub mod tree_layout;

pub use config::LayoutConfig;

use crate::model::graph::Graph;
use crate::model::node::NodeKind;
use std::collections::{HashMap, HashSet, VecDeque};
use tree_layout::layout_tree_component;

/// 纯 Rust 实现的高性能行为树/流程图自动排版引擎 (参考 Plasticity 行为树主干展开机制与 Sugiyama DAG 结合)
/// 保证父子节点垂直直线对齐、Fork-Join 分支对称居中、长连线不交叉、各独立流程自适应多列网格排版。
pub fn auto_layout_graph(graph: &mut Graph) {
    auto_layout_graph_with_config(graph, &LayoutConfig::default());
}

pub fn auto_layout_graph_with_config(graph: &mut Graph, config: &LayoutConfig) {
    if graph.nodes.is_empty() {
        return;
    }

    let node_indices: HashMap<String, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(idx, n)| (n.id.clone(), idx))
        .collect();

    // 1. 构建全图邻接表与无向连通分量邻接表
    let (directed_adj, undirected_adj) = build_adjacency_maps(graph, &node_indices);

    // 2. 划分连通子图 (Connected Components)
    let mut components = find_connected_components(graph, &node_indices, &undirected_adj);

    // 3. 排序连通子图 (主干大图排最前，独立小分支紧随其后)
    components.sort_by(|a, b| {
        let a_has_ep = a.iter().any(|id| id.eq_ignore_ascii_case("ai_2") || id.starts_with("ep_"));
        let b_has_ep = b.iter().any(|id| id.eq_ignore_ascii_case("ai_2") || id.starts_with("ep_"));
        match (a_has_ep, b_has_ep) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.len().cmp(&a.len()),
        }
    });

    if components.is_empty() {
        return;
    }

    // 4. 对连通子图实施 2D 自适应多列布局
    let (first_w, first_h) = layout_tree_component(
        &components[0],
        graph,
        &node_indices,
        &directed_adj,
        config.start_x,
        config.start_y,
        config,
    );

    if components.len() == 1 {
        return;
    }

    let max_col_height = first_h.max(1600.0);
    let mut col_x = config.start_x + first_w + config.component_spacing_x;
    let mut cur_y = config.start_y;
    let mut max_w_in_col: f32 = 0.0;

    for comp in &components[1..] {
        let (comp_w, comp_h) = layout_tree_component(
            comp,
            graph,
            &node_indices,
            &directed_adj,
            col_x,
            cur_y,
            config,
        );

        max_w_in_col = max_w_in_col.max(comp_w);
        cur_y += comp_h + config.component_spacing_y;

        if cur_y >= max_col_height {
            col_x += max_w_in_col + config.component_spacing_x;
            cur_y = config.start_y;
            max_w_in_col = 0.0;
        }
    }
}

/// 构建有向图与无向图邻接表
fn build_adjacency_maps(
    graph: &Graph,
    node_indices: &HashMap<String, usize>,
) -> (HashMap<String, Vec<String>>, HashMap<String, HashSet<String>>) {
    let mut directed_adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut undirected_adj: HashMap<String, HashSet<String>> = HashMap::new();

    for node in &graph.nodes {
        directed_adj.insert(node.id.clone(), Vec::new());
        undirected_adj.insert(node.id.clone(), HashSet::new());
    }

    for edge in &graph.edges {
        if node_indices.contains_key(&edge.source_node_id)
            && node_indices.contains_key(&edge.target_node_id)
        {
            if let Some(list) = directed_adj.get_mut(&edge.source_node_id) {
                if !list.contains(&edge.target_node_id) {
                    list.push(edge.target_node_id.clone());
                }
            }
            if let Some(s) = undirected_adj.get_mut(&edge.source_node_id) {
                s.insert(edge.target_node_id.clone());
            }
            if let Some(s) = undirected_adj.get_mut(&edge.target_node_id) {
                s.insert(edge.source_node_id.clone());
            }
        }
    }

    (directed_adj, undirected_adj)
}

/// 划分无向连通分量 (Connected Components)
fn find_connected_components(
    graph: &Graph,
    node_indices: &HashMap<String, usize>,
    undirected_adj: &HashMap<String, HashSet<String>>,
) -> Vec<Vec<String>> {
    let mut visited_comp: HashSet<String> = HashSet::new();
    let mut components: Vec<Vec<String>> = Vec::new();

    let mut ordered_start_nodes: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    ordered_start_nodes.sort_by_key(|id| {
        if let Some(&idx) = node_indices.get(id) {
            if graph.nodes[idx].kind == NodeKind::EntryPoint || id.starts_with("ep_") {
                0
            } else {
                1
            }
        } else {
            2
        }
    });

    for node_id in ordered_start_nodes {
        if !visited_comp.contains(&node_id) {
            let mut comp = Vec::new();
            let mut q = VecDeque::new();
            visited_comp.insert(node_id.clone());
            q.push_back(node_id);

            while let Some(curr) = q.pop_front() {
                comp.push(curr.clone());
                if let Some(neighbors) = undirected_adj.get(&curr) {
                    for nxt in neighbors {
                        if !visited_comp.contains(nxt) {
                            visited_comp.insert(nxt.clone());
                            q.push_back(nxt.clone());
                        }
                    }
                }
            }
            components.push(comp);
        }
    }

    components
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::evfl::load_bfevfl_from_bytes;

    #[test]
    fn test_auto_layout_100enemy() {
        let bytes = include_bytes!("../../../100enemy.bfevfl");
        let mut graph = load_bfevfl_from_bytes(bytes).expect("load 100enemy");
        auto_layout_graph(&mut graph);

        assert!(!graph.nodes.is_empty());
    }
}
