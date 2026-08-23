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

/// 纯 Rust 实现的高性能行为树/流程图自动排版引擎 (向下优先纵向级联排版)
/// 保证父子节点垂直直线对齐、主逻辑体系（AI_2 Root -> AI_356 通常 -> AI_357 通常 -> 过场动画）自顶向下规整流动。
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

    // 2. 按核心入口点边界划分独立行为树组件 (EntryPoint-Aware Partitioning)
    let mut components = find_connected_components(graph, &node_indices, &directed_adj, &undirected_adj);

    // 3. 排序连通子图：遵循自顶向下逻辑执行流 (AI_2 Root -> AI_356 生命周期 -> AI_357 战斗状态机 -> Demo)
    components.sort_by(|a, b| {
        let a_score = get_comp_priority(a);
        let b_score = get_comp_priority(b);
        a_score.cmp(&b_score).then_with(|| b.len().cmp(&a.len()))
    });

    if components.is_empty() {
        return;
    }

    // 4. 对连通子图实施向下优先纵向级联布局 (Downward-Prioritized Vertical Cascade)
    let mut cur_y = config.start_y;
    let main_x = config.start_x;

    let mut demo_components = Vec::new();

    for comp in components {
        let is_demo = comp.iter().any(|id| id.starts_with("ep_"));
        if is_demo {
            demo_components.push(comp);
            continue;
        }

        let (_comp_w, comp_h) = layout_tree_component(
            &comp,
            graph,
            &node_indices,
            &directed_adj,
            main_x,
            cur_y,
            config,
        );

        cur_y += comp_h + config.component_spacing_y * 1.5;
    }

    // 5. 对所有过场动画 (Demo Actions) 实施下方整齐多列网格排版
    if !demo_components.is_empty() {
        let demo_cols = 4;
        let col_spacing = config.component_spacing_x * 1.2;
        let row_spacing = config.component_spacing_y;
        let mut row_max_h: f32 = 0.0;
        let mut col_idx = 0;
        let mut col_x = main_x;

        for demo_comp in demo_components {
            let (dw, dh) = layout_tree_component(
                &demo_comp,
                graph,
                &node_indices,
                &directed_adj,
                col_x,
                cur_y,
                config,
            );

            row_max_h = row_max_h.max(dh);
            col_x += dw.max(260.0) + col_spacing;
            col_idx += 1;

            if col_idx >= demo_cols {
                col_idx = 0;
                col_x = main_x;
                cur_y += row_max_h + row_spacing;
                row_max_h = 0.0;
            }
        }
    }
}

/// 获取组件的自顶向下优先次序
fn get_comp_priority(comp: &[String]) -> usize {
    if comp.iter().any(|id| id.eq_ignore_ascii_case("ai_2")) {
        0 // 顶层主分发器
    } else if comp.iter().any(|id| id.eq_ignore_ascii_case("ai_356")) {
        1 // 生命周期与底层反应机制
    } else if comp.iter().any(|id| id.eq_ignore_ascii_case("ai_357")) {
        2 // 主行为与战斗状态机
    } else if comp.iter().any(|id| id.starts_with("ep_")) {
        3 // 过场动画 Demo 入口
    } else {
        4 // 其余连通分支
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

/// 核心入口点感知分量划分 (EntryPoint-Aware Component Partitioning)
fn find_connected_components(
    graph: &Graph,
    _node_indices: &HashMap<String, usize>,
    directed_adj: &HashMap<String, Vec<String>>,
    undirected_adj: &HashMap<String, HashSet<String>>,
) -> Vec<Vec<String>> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut components: Vec<Vec<String>> = Vec::new();

    // 1. 提取并排序所有入口节点 (EntryPoint)
    let mut entry_nodes: Vec<String> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::EntryPoint || n.id.starts_with("ep_"))
        .map(|n| n.id.clone())
        .collect();

    entry_nodes.sort_by_key(|id| {
        if id.eq_ignore_ascii_case("ai_2") {
            0
        } else if id.eq_ignore_ascii_case("ai_356") {
            1
        } else if id.eq_ignore_ascii_case("ai_357") {
            2
        } else {
            3
        }
    });

    let entry_set: HashSet<String> = entry_nodes.iter().cloned().collect();

    // 2. 为每个 EntryPoint 独立提取其专属下游子树 (不越过其他 EntryPoint)
    for ep in entry_nodes {
        if visited.contains(&ep) {
            continue;
        }
        let mut comp = Vec::new();
        let mut q = VecDeque::new();
        visited.insert(ep.clone());
        q.push_back(ep);

        while let Some(curr) = q.pop_front() {
            comp.push(curr.clone());
            if let Some(targets) = directed_adj.get(&curr) {
                for tgt in targets {
                    if !entry_set.contains(tgt) && !visited.contains(tgt) {
                        visited.insert(tgt.clone());
                        q.push_back(tgt.clone());
                    }
                }
            }
        }
        if !comp.is_empty() {
            components.push(comp);
        }
    }

    // 3. 处理其余未被入口点直接覆盖的游离弱连通组件
    for node in &graph.nodes {
        if !visited.contains(&node.id) {
            let mut comp = Vec::new();
            let mut q = VecDeque::new();
            visited.insert(node.id.clone());
            q.push_back(node.id.clone());

            while let Some(curr) = q.pop_front() {
                comp.push(curr.clone());
                if let Some(neighbors) = undirected_adj.get(&curr) {
                    for nxt in neighbors {
                        if !visited.contains(nxt) {
                            visited.insert(nxt.clone());
                            q.push_back(nxt.clone());
                        }
                    }
                }
            }
            if !comp.is_empty() {
                components.push(comp);
            }
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
