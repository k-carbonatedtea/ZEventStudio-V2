#![allow(dead_code)]

use super::config::LayoutNode;
use crate::model::graph::Graph;
use crate::model::node::NodeKind;
use std::collections::{HashMap, HashSet, VecDeque};

/// 层次化树形主干分层算法 (Hierarchical Dominator Tree BFS Layering)
/// 严格依据自根向下的最短调用链深度划分层级，杜绝多入度叶子节点将整个子树拉垮变形
pub fn assign_ranks(
    comp: &[String],
    graph: &Graph,
    node_indices: &HashMap<String, usize>,
    comp_edges: &[(String, String)],
    in_degrees: &HashMap<String, usize>,
) -> HashMap<String, usize> {
    // 1. 识别核心根节点 (Root / EntryPoint > 入度为 0 节点)
    let mut roots: Vec<String> = Vec::new();
    for id in comp {
        if let Some(&idx) = node_indices.get(id) {
            let n = &graph.nodes[idx];
            if n.kind == NodeKind::EntryPoint
                || id.starts_with("ep_")
                || id.eq_ignore_ascii_case("ai_2")
                || n.title.eq_ignore_ascii_case("root")
            {
                roots.push(id.clone());
            }
        }
    }
    if roots.is_empty() {
        for id in comp {
            if *in_degrees.get(id).unwrap_or(&0) == 0 {
                roots.push(id.clone());
            }
        }
    }
    if roots.is_empty() && !comp.is_empty() {
        roots.push(comp[0].clone());
    }

    // 2. 构建邻接表
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for (u, v) in comp_edges {
        adj.entry(u.clone()).or_default().push(v.clone());
    }

    // 3. BFS 最短调用深度分层 (Shortest-Path Dominator Tree Ranking)
    let mut ranks: HashMap<String, usize> = HashMap::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();

    for r in &roots {
        ranks.insert(r.clone(), 0);
        queue.push_back((r.clone(), 0));
    }

    let mut visited: HashSet<String> = roots.iter().cloned().collect();

    while let Some((curr, curr_rank)) = queue.pop_front() {
        if let Some(targets) = adj.get(&curr) {
            for target in targets {
                if !visited.contains(target) {
                    visited.insert(target.clone());
                    let next_rank = curr_rank + 1;
                    ranks.insert(target.clone(), next_rank);
                    queue.push_back((target.clone(), next_rank));
                }
            }
        }
    }

    // 处理未被根节点直接覆盖到的孤立弱连通节点
    for id in comp {
        if !ranks.contains_key(id) {
            ranks.insert(id.clone(), 0);
        }
    }

    ranks
}

/// 虚拟节点插入与分层划分的返回结果
#[derive(Debug, Clone)]
pub struct DummyNodeAugmentResult {
    pub augmented_nodes: HashMap<String, LayoutNode>,
    pub augmented_edges: Vec<(String, String)>,
    pub layers: Vec<Vec<String>>,
    pub max_rank: usize,
    pub edge_dummy_map: HashMap<(String, String), Vec<String>>,
}

/// 插入虚拟过渡节点 (Dummy Nodes) 消除跨层连线并记录航点映射
pub fn insert_dummy_nodes(
    comp_nodes: &HashMap<String, LayoutNode>,
    comp_edges: &[(String, String)],
    ranks: &HashMap<String, usize>,
    node_indices: &HashMap<String, usize>,
) -> DummyNodeAugmentResult {
    let mut augmented_nodes = comp_nodes.clone();
    let mut augmented_edges: Vec<(String, String)> = Vec::new();
    let mut edge_dummy_map: HashMap<(String, String), Vec<String>> = HashMap::new();
    let mut dummy_count = 0;

    for (id, node) in augmented_nodes.iter_mut() {
        if let Some(&r) = ranks.get(id) {
            node.rank = r;
        }
    }

    for (u, v) in comp_edges {
        let rank_u = *ranks.get(u).unwrap_or(&0);
        let rank_v = *ranks.get(v).unwrap_or(&0);

        if rank_v > rank_u + 1 {
            let mut prev = u.clone();
            let mut dummies = Vec::new();
            for r in (rank_u + 1)..rank_v {
                dummy_count += 1;
                let dummy_id = format!("__dummy_{}_{}_{}", dummy_count, r, v);
                augmented_nodes.insert(
                    dummy_id.clone(),
                    LayoutNode {
                        id: dummy_id.clone(),
                        is_dummy: true,
                        width: 0.0,
                        height: 0.0,
                        rank: r,
                    },
                );
                augmented_edges.push((prev, dummy_id.clone()));
                dummies.push(dummy_id.clone());
                prev = dummy_id;
            }
            augmented_edges.push((prev, v.clone()));
            edge_dummy_map.insert((u.clone(), v.clone()), dummies);
        } else if rank_v > rank_u {
            augmented_edges.push((u.clone(), v.clone()));
        }
    }

    let max_rank = augmented_nodes.values().map(|n| n.rank).max().unwrap_or(0);
    let mut layers: Vec<Vec<String>> = vec![Vec::new(); max_rank + 1];

    for node in augmented_nodes.values() {
        layers[node.rank].push(node.id.clone());
    }

    for layer in layers.iter_mut().take(max_rank + 1) {
        layer.sort_by_key(|id| node_indices.get(id).copied().unwrap_or(999999));
    }

    DummyNodeAugmentResult {
        augmented_nodes,
        augmented_edges,
        layers,
        max_rank,
        edge_dummy_map,
    }
}
