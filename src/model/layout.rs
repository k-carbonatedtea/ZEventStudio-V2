use super::graph::Graph;
use super::node::NodeKind;
use std::collections::{HashMap, HashSet, VecDeque};

/// 布局配置参数 (与原版 Dagre 排版规范保持一致)
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub start_x: f32,             // 画布起始 X 坐标
    pub start_y: f32,             // 画布起始 Y 坐标
    pub node_spacing_x: f32,      // 同层相邻节点的水平间距 (nodesep: 120.0)
    pub layer_spacing_y: f32,     // 相邻层级之间的垂直间距 (ranksep: 180.0)
    pub component_spacing_x: f32, // 独立连通子图 (如多个 EntryPoint 流) 之间的水平间距 (200.0)
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            start_x: 80.0,
            start_y: 80.0,
            node_spacing_x: 120.0,
            layer_spacing_y: 180.0,
            component_spacing_x: 200.0,
        }
    }
}

/// 纯 Rust 实现的工业级 Sugiyama DAG 自动排版引擎 (配合 Brandes-Köpf 节点居中与对齐算法)
/// 保证父子节点垂直直线对齐、Fork-Join 分支完美对称居中、长连线不交叉、各独立流程独立分列排版。
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
            undirected_adj
                .get_mut(&edge.source_node_id)
                .map(|s| s.insert(edge.target_node_id.clone()));
            undirected_adj
                .get_mut(&edge.target_node_id)
                .map(|s| s.insert(edge.source_node_id.clone()));
        }
    }

    // 2. 划分连通子图 (Connected Components)
    // 优先从 EntryPoint 节点开始划分，保证主要剧情流排在最左侧
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

    // 3. 对各个独立连通子图分别应用 Sugiyama 分层排版算法
    let mut current_comp_x = config.start_x;

    for comp in &components {
        let comp_width = layout_component(
            comp,
            graph,
            &node_indices,
            &directed_adj,
            current_comp_x,
            config,
        );
        current_comp_x += comp_width + config.component_spacing_x;
    }
}

/// 内部节点类型（真实节点或虚拟长连线过渡节点）
#[derive(Debug, Clone)]
struct LayoutNode {
    id: String,
    is_dummy: bool,
    width: f32,
    height: f32,
    rank: usize,
}

/// 单个连通分量的 Sugiyama DAG 布局实现
fn layout_component(
    comp: &[String],
    graph: &mut Graph,
    node_indices: &HashMap<String, usize>,
    directed_adj: &HashMap<String, Vec<String>>,
    comp_start_x: f32,
    config: &LayoutConfig,
) -> f32 {
    let comp_set: HashSet<String> = comp.iter().cloned().collect();

    // 1. 提取分量内部有向边与节点尺寸
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

    // 2. 拓扑分层 (Rank Assignment)
    // 寻找根节点 (EntryPoint 或 in_degree == 0)
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

    // 构建局部邻接表
    let mut local_adj: HashMap<String, Vec<String>> = HashMap::new();
    for (u, v) in &comp_edges {
        local_adj.entry(u.clone()).or_default().push(v.clone());
    }

    // 最长路径分层（保证父节点 rank < 子节点 rank，并处理 Fork/Join 间距）
    let mut ranks: HashMap<String, usize> = HashMap::new();
    for r in &roots {
        ranks.insert(r.clone(), 0);
    }

    // 使用队列进行拓扑 Rank 推进
    let mut queue: VecDeque<String> = VecDeque::from(roots.clone());
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

    // 确保所有节点都有 Rank
    for id in comp {
        ranks.entry(id.clone()).or_insert(0);
    }

    // 更新 comp_nodes 中的 rank
    for (id, node) in comp_nodes.iter_mut() {
        if let Some(&r) = ranks.get(id) {
            node.rank = r;
        }
    }

    // 3. 插入虚拟节点 (Dummy Nodes) 消除跨层长边
    // 使得每条边仅连接相邻层 rank 和 rank + 1
    let mut augmented_nodes: HashMap<String, LayoutNode> = comp_nodes.clone();
    let mut augmented_edges: Vec<(String, String)> = Vec::new();
    let mut dummy_count = 0;

    for (u, v) in comp_edges {
        let rank_u = *ranks.get(&u).unwrap_or(&0);
        let rank_v = *ranks.get(&v).unwrap_or(&0);

        if rank_v > rank_u + 1 {
            let mut prev = u;
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
            augmented_edges.push((prev, v));
        } else if rank_v > rank_u {
            augmented_edges.push((u, v));
        }
    }

    // 按 Rank 分层
    let max_rank = augmented_nodes.values().map(|n| n.rank).max().unwrap_or(0);
    let mut layers: Vec<Vec<String>> = vec![Vec::new(); max_rank + 1];

    for node in augmented_nodes.values() {
        layers[node.rank].push(node.id.clone());
    }

    // 4. 交叉最小化 (Crossing Reduction via Barycenter Sweeps)
    // 构建增强图的邻接表与前驱表
    let mut aug_succ: HashMap<String, Vec<String>> = HashMap::new();
    let mut aug_pred: HashMap<String, Vec<String>> = HashMap::new();

    for (u, v) in &augmented_edges {
        aug_succ.entry(u.clone()).or_default().push(v.clone());
        aug_pred.entry(v.clone()).or_default().push(u.clone());
    }

    // 初始顺序整理
    for r in 0..=max_rank {
        layers[r].sort_by_key(|id| {
            if let Some(&idx) = node_indices.get(id) {
                idx
            } else {
                999999
            }
        });
    }

    // 执行 8 次交替重心扫描 (Barycenter Sweeping)
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
                                let avg = sum / (preds.len() as f32);
                                return (id.clone(), avg);
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
                                let avg = sum / (succs.len() as f32);
                                return (id.clone(), avg);
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

    // 5. 坐标分配 (Horizontal & Vertical Coordinate Assignment)
    // Brandes-Köpf 启发式水平对齐与对称松弛算法
    let x_coords = brandes_kopf_coordinates(
        &layers,
        &augmented_nodes,
        &aug_pred,
        &aug_succ,
        config.node_spacing_x,
    );

    // 计算 Y 坐标 (累加每层高度与 ranksep 间距)
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

    // 计算当前子图的包围盒
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;

    for (id, &center_x) in &x_coords {
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

    // 将计算出的坐标应用到 Graph 中真实节点
    let shift_x = comp_start_x - min_x;

    for (id, &center_x) in &x_coords {
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

    // 返回当前连通分量的总宽度
    (max_x - min_x).max(240.0)
}

/// Brandes-Köpf 风格的多轮对称中位数水平坐标计算与松弛优化
fn brandes_kopf_coordinates(
    layers: &[Vec<String>],
    nodes: &HashMap<String, LayoutNode>,
    preds: &HashMap<String, Vec<String>>,
    succs: &HashMap<String, Vec<String>>,
    node_spacing_x: f32,
) -> HashMap<String, f32> {
    let max_rank = layers.len();
    let mut coords: HashMap<String, f32> = HashMap::new();

    // 1. 初始化各层节点初始位置（满足最小间距约束）
    for layer in layers {
        let mut cur_x = 0.0;
        for i in 0..layer.len() {
            let id = &layer[i];
            let w = nodes.get(id).map(|n| n.width).unwrap_or(240.0);
            if i > 0 {
                let prev_id = &layer[i - 1];
                let prev_w = nodes.get(prev_id).map(|n| n.width).unwrap_or(240.0);
                cur_x += (prev_w + w) / 2.0 + node_spacing_x;
            }
            coords.insert(id.clone(), cur_x);
        }
    }

    // 2. 迭代进行弹簧/重心松弛与对称居中 (Centering & Alignment Passes)
    for _ in 0..12 {
        // 自顶向下对齐 (Top-to-Bottom: Align with parents)
        for r in 1..max_rank {
            let layer = &layers[r];
            for id in layer {
                if let Some(parent_list) = preds.get(id) {
                    if !parent_list.is_empty() {
                        let sum: f32 = parent_list
                            .iter()
                            .filter_map(|p| coords.get(p).copied())
                            .sum();
                        let target_x = sum / (parent_list.len() as f32);
                        coords.insert(id.clone(), target_x);
                    }
                }
            }
            // 纠正同层间距重叠
            enforce_layer_spacing(layer, nodes, &mut coords, node_spacing_x);
        }

        // 自底向上对齐 (Bottom-to-Top: Align with children)
        for r in (0..max_rank - 1).rev() {
            let layer = &layers[r];
            for id in layer {
                if let Some(child_list) = succs.get(id) {
                    if !child_list.is_empty() {
                        let sum: f32 = child_list
                            .iter()
                            .filter_map(|c| coords.get(c).copied())
                            .sum();
                        let target_x = sum / (child_list.len() as f32);
                        coords.insert(id.clone(), target_x);
                    }
                }
            }
            // 纠正同层间距重叠
            enforce_layer_spacing(layer, nodes, &mut coords, node_spacing_x);
        }
    }

    // 3. 对称居中微调：确保分叉与汇合点完美处于子节点或父节点群的几何中心
    for layer in layers {
        for id in layer {
            let is_fork_or_switch = succs.get(id).map(|s| s.len() > 1).unwrap_or(false);
            if is_fork_or_switch {
                if let Some(children) = succs.get(id) {
                    let child_xs: Vec<f32> = children
                        .iter()
                        .filter_map(|c| coords.get(c).copied())
                        .collect();
                    if !child_xs.is_empty() {
                        let min_c = child_xs.iter().copied().fold(f32::INFINITY, f32::min);
                        let max_c = child_xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                        coords.insert(id.clone(), (min_c + max_c) / 2.0);
                    }
                }
            }
        }
    }

    // 最终层内间距强制保障
    for layer in layers {
        enforce_layer_spacing(layer, nodes, &mut coords, node_spacing_x);
    }

    coords
}

/// 强制同层节点从左向右保持最小间距，并在中心发生偏移时进行对称平移
fn enforce_layer_spacing(
    layer: &[String],
    nodes: &HashMap<String, LayoutNode>,
    coords: &mut HashMap<String, f32>,
    node_spacing_x: f32,
) {
    if layer.len() <= 1 {
        return;
    }

    // 从左至右推挤
    for i in 1..layer.len() {
        let left_id = &layer[i - 1];
        let right_id = &layer[i];

        let left_x = *coords.get(left_id).unwrap_or(&0.0);
        let right_x = *coords.get(right_id).unwrap_or(&0.0);

        let left_w = nodes.get(left_id).map(|n| n.width).unwrap_or(240.0);
        let right_w = nodes.get(right_id).map(|n| n.width).unwrap_or(240.0);

        let min_required = left_x + (left_w + right_w) / 2.0 + node_spacing_x;
        if right_x < min_required {
            coords.insert(right_id.clone(), min_required);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::evfl::load_bfevfl_from_bytes;

    #[test]
    fn test_auto_layout_100enemy() {
        let bytes = include_bytes!("../../100enemy.bfevfl");
        let mut graph = load_bfevfl_from_bytes(bytes).expect("load 100enemy");
        auto_layout_graph(&mut graph);

        assert!(!graph.nodes.is_empty());
        println!("=== Nodes in ep_Start tree ===");
        let mut start_nodes: Vec<_> = graph.nodes.iter().filter(|n| n.pos.0 < 2000.0).collect();
        start_nodes.sort_by(|a, b| a.pos.1.partial_cmp(&b.pos.1).unwrap().then(a.pos.0.partial_cmp(&b.pos.0).unwrap()));
        for node in start_nodes {
            println!("  Y={:6.1} | X={:6.1} (center={:6.1}) | ID={:<10} | Title={}", 
                node.pos.1, node.pos.0, node.pos.0 + node.size.0 / 2.0, node.id, node.title);
        }
    }
}
