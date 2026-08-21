use super::graph::Graph;
use std::collections::{HashMap, HashSet, VecDeque};

/// 纯 Rust DAG 拓扑分层排版算法 (Sugiyama Layered Layout)
pub fn auto_layout_graph(graph: &mut Graph) {
    if graph.nodes.is_empty() {
        return;
    }

    let node_indices: HashMap<String, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(idx, n)| (n.id.clone(), idx))
        .collect();

    // 1. 构建邻接表与入度
    let mut in_degrees: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();

    for node in &graph.nodes {
        in_degrees.insert(node.id.clone(), 0);
        adj.insert(node.id.clone(), Vec::new());
    }

    for edge in &graph.edges {
        if graph.find_node(&edge.source_node_id).is_some()
            && graph.find_node(&edge.target_node_id).is_some()
        {
            if let Some(degree) = in_degrees.get_mut(&edge.target_node_id) {
                *degree += 1;
            }
            if let Some(list) = adj.get_mut(&edge.source_node_id) {
                list.push(edge.target_node_id.clone());
            }
        }
    }

    // 2. 拓扑分层 (Rank / Layer assignment)
    let mut ranks: HashMap<String, usize> = HashMap::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    // 找到所有根节点 (入度为 0)
    for (id, &deg) in &in_degrees {
        if deg == 0 {
            queue.push_back(id.clone());
            ranks.insert(id.clone(), 0);
        }
    }

    // 如果全部节点都有环导致无入度为 0 的节点，则任选第一个
    if queue.is_empty() {
        if let Some(first_node) = graph.nodes.first() {
            queue.push_back(first_node.id.clone());
            ranks.insert(first_node.id.clone(), 0);
        }
    }

    let mut visited: HashSet<String> = HashSet::new();

    while let Some(curr_id) = queue.pop_front() {
        visited.insert(curr_id.clone());
        let curr_rank = *ranks.get(&curr_id).unwrap_or(&0);

        if let Some(neighbors) = adj.get(&curr_id) {
            for neighbor in neighbors {
                let next_rank = curr_rank + 1;
                let existing = ranks.entry(neighbor.clone()).or_insert(0);
                if next_rank > *existing {
                    *existing = next_rank;
                }
                if !visited.contains(neighbor) {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    // 处理可能孤立的未访问节点
    for node in &graph.nodes {
        ranks.entry(node.id.clone()).or_insert(0);
    }

    // 3. 将节点按层级分组
    let mut layers: HashMap<usize, Vec<String>> = HashMap::new();
    for (id, rank) in ranks {
        layers.entry(rank).or_default().push(id);
    }

    let mut layer_indices: Vec<usize> = layers.keys().copied().collect();
    layer_indices.sort_unstable();

    // 4. 计算每层坐标 (水平向右排版 X 递增，垂直居中对齐 Y)
    let start_x = 100.0;
    let start_y = 120.0;
    let layer_spacing_x = 320.0;
    let node_spacing_y = 40.0;

    // 计算各层最大高度以实现垂直居中
    let mut max_layer_height = 0.0f32;
    for &l_idx in &layer_indices {
        if let Some(node_ids) = layers.get(&l_idx) {
            let mut total_h = 0.0f32;
            for id in node_ids {
                if let Some(&n_idx) = node_indices.get(id) {
                    total_h += graph.nodes[n_idx].size.1 + node_spacing_y;
                }
            }
            max_layer_height = max_layer_height.max(total_h);
        }
    }

    for &l_idx in &layer_indices {
        if let Some(node_ids) = layers.get(&l_idx) {
            let mut current_layer_height = 0.0f32;
            for id in node_ids {
                if let Some(&n_idx) = node_indices.get(id) {
                    current_layer_height += graph.nodes[n_idx].size.1 + node_spacing_y;
                }
            }

            let y_offset = (max_layer_height - current_layer_height) / 2.0;
            let mut current_y = start_y + y_offset.max(0.0);
            let layer_x = start_x + (l_idx as f32) * layer_spacing_x;

            for id in node_ids {
                if let Some(&n_idx) = node_indices.get(id) {
                    graph.nodes[n_idx].pos = (layer_x, current_y);
                    current_y += graph.nodes[n_idx].size.1 + node_spacing_y;
                }
            }
        }
    }
}
