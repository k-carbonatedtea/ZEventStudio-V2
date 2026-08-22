use super::brandes_kopf::brandes_kopf_coordinates;
use super::config::{LayoutConfig, LayoutNode};
use super::crossings::{compute_port_biases, reduce_crossings};
use super::ranks::{assign_ranks, insert_dummy_nodes};
use crate::model::graph::Graph;
use std::collections::{HashMap, HashSet};

/// 单个连通分量的 Sugiyama DAG 布局实现 (集成引脚感知质心排版与 Brandes-Köpf 水平对齐)
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

    // 3. 插入虚拟节点 (Dummy Nodes) 消除跨层长边并获取航点映射
    let (augmented_nodes, augmented_edges, layers, max_rank, edge_dummy_map) =
        insert_dummy_nodes(&comp_nodes, &comp_edges, &ranks, node_indices);

    // 4. 提取各连线所对应的引脚物理偏移分量 (Port-Aware Biases)
    let (out_biases, in_biases) = compute_port_biases(graph, &augmented_edges);

    let mut aug_succ: HashMap<String, Vec<String>> = HashMap::new();
    let mut aug_pred: HashMap<String, Vec<String>> = HashMap::new();
    for (u, v) in &augmented_edges {
        aug_succ.entry(u.clone()).or_default().push(v.clone());
        aug_pred.entry(v.clone()).or_default().push(u.clone());
    }

    // 5. 交叉最小化 (Port-Aware Barycenter Sweeps 彻底消除同父兄弟节点交叉)
    let optimized_layers =
        reduce_crossings(layers, max_rank, &aug_pred, &aug_succ, &out_biases, &in_biases);

    // 6. 坐标分配 (Brandes-Köpf 启发式水平对齐与汇聚节点居中)
    let x_coords = brandes_kopf_coordinates(
        &optimized_layers,
        &augmented_nodes,
        &aug_pred,
        &aug_succ,
        config.node_spacing_x,
    );

    // 7. 计算 Y 坐标与包围盒应用，并将航点注入跨层连线 (Waypoint Corridor Routing)
    apply_coordinates(
        graph,
        &x_coords,
        &augmented_nodes,
        &optimized_layers,
        max_rank,
        node_indices,
        comp_start_x,
        config,
        &edge_dummy_map,
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
    edge_dummy_map: &HashMap<(String, String), Vec<String>>,
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

    // 将虚拟节点航点注入跨层连线 (Waypoint Corridor Routing)
    for edge in &mut graph.edges {
        let key = (edge.source_node_id.clone(), edge.target_node_id.clone());
        if let Some(dummies) = edge_dummy_map.get(&key) {
            let mut pts = Vec::new();
            for dummy_id in dummies {
                if let (Some(&cx), Some(d_node)) = (x_coords.get(dummy_id), augmented_nodes.get(dummy_id)) {
                    let wx = cx + shift_x;
                    let wy = layer_y[d_node.rank] + 40.0;
                    pts.push((wx, wy));
                }
            }
            edge.waypoints = pts;
        }
    }

    (max_x - min_x).max(240.0)
}


