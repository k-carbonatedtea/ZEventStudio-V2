use super::graph::Graph;
use std::collections::{HashMap, HashSet, VecDeque};

/// 纯 Rust 垂直流拓扑分层排版算法 (Vertical Top-to-Bottom Sugiyama DAG Layout)
/// 入口点 (EntryPoint) 位于顶部，各子剧情流向下延展，多个子流程从左到右并行排列
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

    // 1. 构建有向邻接表、无向连通分量邻接表与入度
    let mut directed_adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut undirected_adj: HashMap<String, HashSet<String>> = HashMap::new();
    let mut in_degrees: HashMap<String, usize> = HashMap::new();

    for node in &graph.nodes {
        directed_adj.insert(node.id.clone(), Vec::new());
        undirected_adj.insert(node.id.clone(), HashSet::new());
        in_degrees.insert(node.id.clone(), 0);
    }

    for edge in &graph.edges {
        if graph.find_node(&edge.source_node_id).is_some()
            && graph.find_node(&edge.target_node_id).is_some()
        {
            if let Some(list) = directed_adj.get_mut(&edge.source_node_id) {
                list.push(edge.target_node_id.clone());
            }
            if let Some(deg) = in_degrees.get_mut(&edge.target_node_id) {
                *deg += 1;
            }
            undirected_adj
                .get_mut(&edge.source_node_id)
                .map(|s| s.insert(edge.target_node_id.clone()));
            undirected_adj
                .get_mut(&edge.target_node_id)
                .map(|s| s.insert(edge.source_node_id.clone()));
        }
    }

    // 2. 识别所有连通分量 (Connected Components)，每个分量对应一个独立的垂直流通道
    let mut visited_comp: HashSet<String> = HashSet::new();
    let mut components: Vec<Vec<String>> = Vec::new();

    // 优先从 EntryPoint 节点开始切分，确保各个入口排在醒目位置
    let mut ordered_node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    ordered_node_ids.sort_by_key(|id| {
        if id.starts_with("ep_") {
            0
        } else {
            1
        }
    });

    for node_id in ordered_node_ids {
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

    // 3. 垂直排版参数
    let start_x = 80.0;
    let start_y = 80.0;
    let layer_spacing_y = 220.0; // 垂直层级间距 (卡片高度 ~130px + 间隙 ~90px)
    let node_spacing_x = 48.0;   // 同层平行分支横向间距
    let component_spacing_x = 96.0; // 独立流程泳道横向间距

    let mut current_comp_x = start_x;

    for comp in &components {
        let comp_nodes_set: HashSet<&String> = comp.iter().collect();

        // 计算当前分量内部入度
        let mut local_in_degrees: HashMap<String, usize> = HashMap::new();
        for id in comp {
            local_in_degrees.insert(id.clone(), 0);
        }
        for id in comp {
            if let Some(targets) = directed_adj.get(id) {
                for t in targets {
                    if comp_nodes_set.contains(t) {
                        *local_in_degrees.entry(t.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        // 寻找分量根节点 (入度为 0 或 EntryPoint) -> 赋予 Rank 0 (顶部)
        let mut ranks: HashMap<String, usize> = HashMap::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        for id in comp {
            if *local_in_degrees.get(id).unwrap_or(&0) == 0 {
                queue.push_back(id.clone());
                ranks.insert(id.clone(), 0);
            }
        }

        if queue.is_empty() {
            if let Some(first) = comp.first() {
                queue.push_back(first.clone());
                ranks.insert(first.clone(), 0);
            }
        }

        let mut visited_rank: HashSet<String> = HashSet::new();
        while let Some(curr_id) = queue.pop_front() {
            visited_rank.insert(curr_id.clone());
            let curr_rank = *ranks.get(&curr_id).unwrap_or(&0);

            if let Some(neighbors) = directed_adj.get(&curr_id) {
                for neighbor in neighbors {
                    if comp_nodes_set.contains(neighbor) {
                        let next_rank = curr_rank + 1;
                        let existing = ranks.entry(neighbor.clone()).or_insert(0);
                        if next_rank > *existing {
                            *existing = next_rank;
                        }
                        if !visited_rank.contains(neighbor) {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        // 补充未遍历节点
        for id in comp {
            ranks.entry(id.clone()).or_insert(0);
        }

        // 按 rank 聚类
        let mut layers: HashMap<usize, Vec<String>> = HashMap::new();
        for id in comp {
            let r = *ranks.get(id).unwrap_or(&0);
            layers.entry(r).or_default().push(id.clone());
        }

        let mut layer_indices: Vec<usize> = layers.keys().copied().collect();
        layer_indices.sort_unstable();

        // 计算当前分量所需的最大总宽度 (横向宽度)
        let mut comp_max_width = 0.0f32;
        for &l_idx in &layer_indices {
            if let Some(node_ids) = layers.get(&l_idx) {
                let mut layer_w = 0.0f32;
                for (i, id) in node_ids.iter().enumerate() {
                    if let Some(&n_idx) = node_indices.get(id) {
                        layer_w += graph.nodes[n_idx].size.0;
                        if i > 0 {
                            layer_w += node_spacing_x;
                        }
                    }
                }
                comp_max_width = comp_max_width.max(layer_w);
            }
        }
        comp_max_width = comp_max_width.max(240.0);

        // 垂直自顶向下放置节点 (Top-to-Bottom)
        for &l_idx in &layer_indices {
            if let Some(node_ids) = layers.get(&l_idx) {
                let mut layer_w = 0.0f32;
                for (i, id) in node_ids.iter().enumerate() {
                    if let Some(&n_idx) = node_indices.get(id) {
                        layer_w += graph.nodes[n_idx].size.0;
                        if i > 0 {
                            layer_w += node_spacing_x;
                        }
                    }
                }

                // 本层节点在分量宽度内水平居中
                let x_offset = (comp_max_width - layer_w) / 2.0;
                let mut current_x = current_comp_x + x_offset.max(0.0);
                let layer_y = start_y + (l_idx as f32) * layer_spacing_y;

                for id in node_ids {
                    if let Some(&n_idx) = node_indices.get(id) {
                        graph.nodes[n_idx].pos = (current_x, layer_y);
                        current_x += graph.nodes[n_idx].size.0 + node_spacing_x;
                    }
                }
            }
        }

        // 下一个独立子流程横向顺延
        current_comp_x += comp_max_width + component_spacing_x;
    }
}
