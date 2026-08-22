use super::config::LayoutNode;
use crate::model::graph::Graph;
use crate::model::node::NodeKind;
use std::collections::{HashMap, VecDeque};

/// 拓扑分层算法 (Rank Assignment)
pub fn assign_ranks(
    comp: &[String],
    graph: &Graph,
    node_indices: &HashMap<String, usize>,
    comp_edges: &[(String, String)],
    in_degrees: &HashMap<String, usize>,
) -> HashMap<String, usize> {
    let mut roots: Vec<String> = Vec::new();
    for id in comp {
        if let Some(&idx) = node_indices.get(id) {
            if graph.nodes[idx].kind == NodeKind::EntryPoint || id.starts_with("ep_") {
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

    let mut local_adj: HashMap<String, Vec<String>> = HashMap::new();
    for (u, v) in comp_edges {
        local_adj.entry(u.clone()).or_default().push(v.clone());
    }

    let mut ranks: HashMap<String, usize> = HashMap::new();
    for r in &roots {
        ranks.insert(r.clone(), 0);
    }

    let mut queue: VecDeque<String> = VecDeque::from(roots);
    let mut visited_count: HashMap<String, usize> = HashMap::new();
    let max_iterations = comp.len() * 3;

    while let Some(curr) = queue.pop_front() {
        let curr_rank = *ranks.get(&curr).unwrap_or(&0);
        let count = visited_count.entry(curr.clone()).or_insert(0);
        *count += 1;
        if *count > max_iterations {
            continue; // 防止环路死循环
        }

        if let Some(targets) = local_adj.get(&curr) {
            for target in targets {
                let next_rank = curr_rank + 1;
                let is_higher = match ranks.get(target) {
                    Some(&r) => next_rank > r,
                    None => true,
                };
                if is_higher {
                    ranks.insert(target.clone(), next_rank);
                    queue.push_back(target.clone());
                }
            }
        }
    }

    for id in comp {
        ranks.entry(id.clone()).or_insert(0);
    }

    ranks
}

/// 插入虚拟过渡节点 (Dummy Nodes) 消除跨层连线
pub fn insert_dummy_nodes(
    comp_nodes: &HashMap<String, LayoutNode>,
    comp_edges: &[(String, String)],
    ranks: &HashMap<String, usize>,
    node_indices: &HashMap<String, usize>,
) -> (
    HashMap<String, LayoutNode>,
    Vec<(String, String)>,
    Vec<Vec<String>>,
    usize,
) {
    let mut augmented_nodes = comp_nodes.clone();
    let mut augmented_edges: Vec<(String, String)> = Vec::new();
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
                prev = dummy_id;
            }
            augmented_edges.push((prev, v.clone()));
        } else if rank_v > rank_u {
            augmented_edges.push((u.clone(), v.clone()));
        }
    }

    let max_rank = augmented_nodes.values().map(|n| n.rank).max().unwrap_or(0);
    let mut layers: Vec<Vec<String>> = vec![Vec::new(); max_rank + 1];

    for node in augmented_nodes.values() {
        layers[node.rank].push(node.id.clone());
    }

    for r in 0..=max_rank {
        layers[r].sort_by_key(|id| {
            if let Some(&idx) = node_indices.get(id) {
                idx
            } else {
                999999
            }
        });
    }

    (augmented_nodes, augmented_edges, layers, max_rank)
}
