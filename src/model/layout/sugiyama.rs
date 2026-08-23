use super::brandes_kopf::brandes_kopf_coordinates;
use super::config::{LayoutConfig, LayoutNode};
use super::crossings::{compute_port_biases, reduce_crossings};
use super::ranks::{assign_ranks, insert_dummy_nodes};
use crate::model::graph::Graph;
use std::collections::{HashMap, HashSet};

/// 连通分量子图提取结果
pub struct ComponentGraphData {
    pub comp_nodes: HashMap<String, LayoutNode>,
    pub comp_edges: Vec<(String, String)>,
    pub in_degrees: HashMap<String, usize>,
}

/// 布局坐标应用参数上下文
pub struct CoordinateAssignmentParams<'a> {
    pub x_coords: &'a HashMap<String, f32>,
    pub augmented_nodes: &'a HashMap<String, LayoutNode>,
    pub layers: &'a [Vec<String>],
    pub max_rank: usize,
    pub node_indices: &'a HashMap<String, usize>,
    pub comp_start_x: f32,
    pub comp_start_y: f32,
    pub config: &'a LayoutConfig,
    pub edge_dummy_map: &'a HashMap<(String, String), Vec<String>>,
}

/// 单个连通分量的 Sugiyama DAG 布局实现 (集成引脚感知质心排版与 Brandes-Köpf 水平对齐)
pub fn layout_component(
    comp: &[String],
    graph: &mut Graph,
    node_indices: &HashMap<String, usize>,
    directed_adj: &HashMap<String, Vec<String>>,
    comp_start_x: f32,
    comp_start_y: f32,
    config: &LayoutConfig,
) -> (f32, f32) {
    let comp_set: HashSet<String> = comp.iter().cloned().collect();

    // 1. 提取分量内部有向边与节点尺寸
    let graph_data =
        extract_component_graph(comp, graph, node_indices, directed_adj, &comp_set);

    // 2. 拓扑分层 (Rank Assignment)
    let ranks = assign_ranks(comp, graph, node_indices, &graph_data.comp_edges, &graph_data.in_degrees);

    // 3. 插入虚拟节点 (Dummy Nodes) 消除跨层长边并获取航点映射
    let augment_res =
        insert_dummy_nodes(&graph_data.comp_nodes, &graph_data.comp_edges, &ranks, node_indices);

    // 4. 提取各连线所对应的引脚物理偏移分量 (Port-Aware Biases)
    let (out_biases, in_biases) = compute_port_biases(graph, &augment_res.augmented_edges);

    let mut aug_succ: HashMap<String, Vec<String>> = HashMap::new();
    let mut aug_pred: HashMap<String, Vec<String>> = HashMap::new();
    for (u, v) in &augment_res.augmented_edges {
        aug_succ.entry(u.clone()).or_default().push(v.clone());
        aug_pred.entry(v.clone()).or_default().push(u.clone());
    }

    // 5. 交叉最小化 (Port-Aware Barycenter Sweeps 彻底消除同父兄弟节点交叉)
    let optimized_layers =
        reduce_crossings(augment_res.layers, augment_res.max_rank, &aug_pred, &aug_succ, &out_biases, &in_biases);

    // 6. 坐标分配 (Brandes-Köpf 启发式水平对齐与汇聚节点居中)
    let x_coords = brandes_kopf_coordinates(
        &optimized_layers,
        &augment_res.augmented_nodes,
        &aug_pred,
        &aug_succ,
        config.node_spacing_x,
    );

    // 7. 计算 Y 坐标与包围盒应用，并将航点注入跨层连线 (Waypoint Corridor Routing)
    let params = CoordinateAssignmentParams {
        x_coords: &x_coords,
        augmented_nodes: &augment_res.augmented_nodes,
        layers: &optimized_layers,
        max_rank: augment_res.max_rank,
        node_indices,
        comp_start_x,
        comp_start_y,
        config,
        edge_dummy_map: &augment_res.edge_dummy_map,
    };
    apply_coordinates(graph, &params)
}

/// 提取子图节点与有向边数据
fn extract_component_graph(
    comp: &[String],
    graph: &Graph,
    node_indices: &HashMap<String, usize>,
    directed_adj: &HashMap<String, Vec<String>>,
    comp_set: &HashSet<String>,
) -> ComponentGraphData {
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

    ComponentGraphData {
        comp_nodes,
        comp_edges,
        in_degrees,
    }
}

/// 计算 Y 坐标并将最终几何位置应用到 Graph 节点中
fn apply_coordinates(
    graph: &mut Graph,
    params: &CoordinateAssignmentParams,
) -> (f32, f32) {
    let mut layer_y: Vec<f32> = vec![0.0; params.max_rank + 1];
    let mut current_y = params.comp_start_y;

    for (r, y_val) in layer_y.iter_mut().enumerate().take(params.max_rank + 1) {
        *y_val = current_y;
        let max_h = params.layers[r]
            .iter()
            .map(|id| params.augmented_nodes.get(id).map(|n| n.height).unwrap_or(0.0))
            .fold(0.0f32, |a, b| a.max(b));
        current_y += max_h + params.config.layer_spacing_y;
    }

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;

    for (id, &center_x) in params.x_coords {
        if let Some(node) = params.augmented_nodes.get(id) {
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

    let shift_x = params.comp_start_x - min_x;

    for (id, &center_x) in params.x_coords {
        if let Some(node) = params.augmented_nodes.get(id) {
            if !node.is_dummy {
                if let Some(&n_idx) = params.node_indices.get(id) {
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
        if let Some(dummies) = params.edge_dummy_map.get(&key) {
            let mut pts = Vec::new();
            for dummy_id in dummies {
                if let (Some(&cx), Some(d_node)) = (params.x_coords.get(dummy_id), params.augmented_nodes.get(dummy_id)) {
                    let wx = cx + shift_x;
                    let wy = layer_y[d_node.rank] + 40.0;
                    pts.push((wx, wy));
                }
            }
            edge.waypoints = pts;
        }
    }

    let comp_w = (max_x - min_x).max(240.0);
    let comp_h = (current_y - params.comp_start_y).max(120.0);
    (comp_w, comp_h)
}
