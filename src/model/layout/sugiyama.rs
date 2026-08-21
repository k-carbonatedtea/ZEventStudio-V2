use super::brandes_kopf::brandes_kopf_coordinates;
use super::config::{LayoutConfig, LayoutNode};
use crate::model::graph::Graph;
use crate::model::node::NodeKind;
use std::collections::{HashMap, HashSet, VecDeque};

/// 单个连通分量的 Sugiyama DAG 布局实现
pub fn layout_component(
    comp: &[String],
    graph: &mut Graph,
    node_indices: &HashMap<String, usize>,
    directed_adj: &HashMap<String, Vec<String>>,
    comp_start_x: f32,
    config: &LayoutConfig,
) -> f32 {
    let comp_set: HashSet<String> = comp.iter().cloned().collect();

    // 1. 提取分量内部有向边与节点尺寸
    let (comp_nodes, comp_edges, in_degrees) =
        extract_component_graph(comp, graph, node_indices, directed_adj, &comp_set);

    // 2. 拓扑分层 (Rank Assignment)
    let ranks = assign_ranks(comp, graph, node_indices, &comp_edges, &in_degrees);

    // 3. 插入虚拟节点 (Dummy Nodes) 消除跨层长边
    let (augmented_nodes, augmented_edges, layers, max_rank) =
        insert_dummy_nodes(&comp_nodes, &comp_edges, &ranks, node_indices);

    // 4. 交叉最小化 (Crossing Reduction via Barycenter Sweeps)
    let mut aug_succ: HashMap<String, Vec<String>> = HashMap::new();
    let mut aug_pred: HashMap<String, Vec<String>> = HashMap::new();
    for (u, v) in &augmented_edges {
        aug_succ.entry(u.clone()).or_default().push(v.clone());
        aug_pred.entry(v.clone()).or_default().push(u.clone());
    }

    let optimized_layers =
        reduce_crossings(layers, max_rank, &aug_pred, &aug_succ, node_indices);

    // 5. 坐标分配 (Brandes-Köpf 启发式水平对齐)
    let x_coords = brandes_kopf_coordinates(
        &optimized_layers,
        &augmented_nodes,
        &aug_pred,
        &aug_succ,
        config.node_spacing_x,
    );

    // 6. 计算 Y 坐标与包围盒应用
    apply_coordinates(
        graph,
        &x_coords,
        &augmented_nodes,
        &optimized_layers,
        max_rank,
        node_indices,
        comp_start_x,
        config,
    )
}

/// 提取子图节点与有向边数据
fn extract_component_graph(
    comp: &[String],
    graph: &Graph,
    node_indices: &HashMap<String, usize>,
    directed_adj: &HashMap<String, Vec<String>>,
    comp_set: &HashSet<String>,
) -> (
    HashMap<String, LayoutNode>,
    Vec<(String, String)>,
    HashMap<String, usize>,
) {
    let mut comp_nodes: HashMap<String, LayoutNode> = HashMap::new();
    let mut comp_edges: Vec<(String, String)> = Vec::new();
    let mut in_degrees: HashMap<String, usize> = HashMap::new();

    for id in comp {
        let (w, h) = if let Some(&idx) = node_indices.get(id) {
            let node = &graph.nodes[idx];
            (node.size.0, node.size.1)
        } else {
            (240.0, 120.0)
        };

        comp_nodes.insert(
            id.clone(),
            LayoutNode {
                id: id.clone(),
                is_dummy: false,
                width: w,
                height: h,
                rank: 0,
            },
        );
        in_degrees.insert(id.clone(), 0);
    }

    for id in comp {
        if let Some(targets) = directed_adj.get(id) {
            for t in targets {
                if comp_set.contains(t) {
                    comp_edges.push((id.clone(), t.clone()));
                    *in_degrees.entry(t.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    (comp_nodes, comp_edges, in_degrees)
}

/// 拓扑分层算法 (Rank Assignment)
fn assign_ranks(
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
fn insert_dummy_nodes(
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

/// 重心扫描交叉最小化 (Barycenter Crossing Reduction)
fn reduce_crossings(
    mut layers: Vec<Vec<String>>,
    max_rank: usize,
    aug_pred: &HashMap<String, Vec<String>>,
    aug_succ: &HashMap<String, Vec<String>>,
    _node_indices: &HashMap<String, usize>,
) -> Vec<Vec<String>> {
    for iter in 0..8 {
        if iter % 2 == 0 {
            // 自顶向下扫描 (Top-to-Bottom)
            for r in 1..=max_rank {
                let prev_layer = &layers[r - 1];
                let pos_map: HashMap<&str, f32> = prev_layer
                    .iter()
                    .enumerate()
                    .map(|(i, id)| (id.as_str(), i as f32))
                    .collect();

                let mut scored: Vec<(String, f32)> = layers[r]
                    .iter()
                    .enumerate()
                    .map(|(orig_idx, id)| {
                        if let Some(preds) = aug_pred.get(id) {
                            if !preds.is_empty() {
                                let sum: f32 = preds
                                    .iter()
                                    .filter_map(|p| pos_map.get(p.as_str()).copied())
                                    .sum();
                                return (id.clone(), sum / (preds.len() as f32));
                            }
                        }
                        (id.clone(), orig_idx as f32)
                    })
                    .collect();

                scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                layers[r] = scored.into_iter().map(|(id, _)| id).collect();
            }
        } else {
            // 自底向上扫描 (Bottom-to-Top)
            for r in (0..max_rank).rev() {
                let next_layer = &layers[r + 1];
                let pos_map: HashMap<&str, f32> = next_layer
                    .iter()
                    .enumerate()
                    .map(|(i, id)| (id.as_str(), i as f32))
                    .collect();

                let mut scored: Vec<(String, f32)> = layers[r]
                    .iter()
                    .enumerate()
                    .map(|(orig_idx, id)| {
                        if let Some(succs) = aug_succ.get(id) {
                            if !succs.is_empty() {
                                let sum: f32 = succs
                                    .iter()
                                    .filter_map(|s| pos_map.get(s.as_str()).copied())
                                    .sum();
                                return (id.clone(), sum / (succs.len() as f32));
                            }
                        }
                        (id.clone(), orig_idx as f32)
                    })
                    .collect();

                scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                layers[r] = scored.into_iter().map(|(id, _)| id).collect();
            }
        }
    }
    layers
}

/// 计算 Y 坐标并将最终几何位置应用到 Graph 节点中
fn apply_coordinates(
    graph: &mut Graph,
    x_coords: &HashMap<String, f32>,
    augmented_nodes: &HashMap<String, LayoutNode>,
    layers: &[Vec<String>],
    max_rank: usize,
    node_indices: &HashMap<String, usize>,
    comp_start_x: f32,
    config: &LayoutConfig,
) -> f32 {
    let mut layer_y: Vec<f32> = vec![0.0; max_rank + 1];
    let mut current_y = config.start_y;

    for r in 0..=max_rank {
        layer_y[r] = current_y;
        let max_h = layers[r]
            .iter()
            .map(|id| augmented_nodes.get(id).map(|n| n.height).unwrap_or(0.0))
            .fold(0.0f32, |a, b| a.max(b));
        current_y += max_h + config.layer_spacing_y;
    }

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;

    for (id, &center_x) in x_coords {
        if let Some(node) = augmented_nodes.get(id) {
            if !node.is_dummy {
                let left = center_x - node.width / 2.0;
                let right = center_x + node.width / 2.0;
                min_x = min_x.min(left);
                max_x = max_x.max(right);
            }
        }
    }

    if min_x == f32::INFINITY {
        min_x = 0.0;
        max_x = 240.0;
    }

    let shift_x = comp_start_x - min_x;

    for (id, &center_x) in x_coords {
        if let Some(node) = augmented_nodes.get(id) {
            if !node.is_dummy {
                if let Some(&n_idx) = node_indices.get(id) {
                    let final_x = center_x - node.width / 2.0 + shift_x;
                    let final_y = layer_y[node.rank];
                    graph.nodes[n_idx].pos = (final_x, final_y);
                }
            }
        }
    }

    (max_x - min_x).max(240.0)
}
