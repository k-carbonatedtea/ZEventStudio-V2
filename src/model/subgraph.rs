use std::collections::{HashMap, HashSet, VecDeque};
use super::graph::Graph;
use super::node::{NodeData, NodeKind};

/// 语义剧情分区/分组框定义
#[derive(Debug, Clone, PartialEq)]
pub struct SubgraphGroup {
    pub id: String,
    pub title: String,
    pub bounds: (f32, f32, f32, f32), // (min_x, min_y, max_x, max_y)
    pub node_count: usize,
    pub color: gpui::Hsla,
}

/// 提取图谱中的连通分区并计算每个分区的语义包围盒
pub fn compute_subgraph_groups(graph: &Graph) -> Vec<SubgraphGroup> {
    let mut directed_adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut undirected_adj: HashMap<String, HashSet<String>> = HashMap::new();
    let node_map: HashMap<String, &NodeData> = graph.nodes.iter().map(|n| (n.id.clone(), n)).collect();

    for n in &graph.nodes {
        directed_adj.insert(n.id.clone(), Vec::new());
        undirected_adj.insert(n.id.clone(), HashSet::new());
    }

    for edge in &graph.edges {
        if undirected_adj.contains_key(&edge.source_node_id) && undirected_adj.contains_key(&edge.target_node_id) {
            if let Some(list) = directed_adj.get_mut(&edge.source_node_id) {
                list.push(edge.target_node_id.clone());
            }
            if let Some(set) = undirected_adj.get_mut(&edge.source_node_id) {
                set.insert(edge.target_node_id.clone());
            }
            if let Some(set) = undirected_adj.get_mut(&edge.target_node_id) {
                set.insert(edge.source_node_id.clone());
            }
        }
    }

    let mut visited = HashSet::new();
    let mut groups = Vec::new();

    // 1. 优先按核心入口点 (EntryPoint) 提取独立分区
    let mut entry_nodes: Vec<String> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::EntryPoint || n.id.starts_with("ep_"))
        .map(|n| n.id.clone())
        .collect();

    entry_nodes.sort_by_key(|id| {
        if id.eq_ignore_ascii_case("ai_357") {
            0
        } else if id.eq_ignore_ascii_case("ai_356") {
            1
        } else if id.eq_ignore_ascii_case("ai_2") {
            2
        } else {
            3
        }
    });

    let entry_set: HashSet<String> = entry_nodes.iter().cloned().collect();

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

        if comp.is_empty() {
            continue;
        }

        // 计算该分区的包围盒
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut group_title = String::new();
        let group_color = gpui::hsla(160.0 / 360.0, 0.84, 0.39, 1.0);

        for id in &comp {
            if let Some(n) = node_map.get(id) {
                min_x = min_x.min(n.pos.0);
                min_y = min_y.min(n.pos.1);
                max_x = max_x.max(n.pos.0 + n.size.0);
                max_y = max_y.max(n.pos.1 + n.size.1);

                if (n.kind == NodeKind::EntryPoint || n.id.starts_with("ep_")) && group_title.is_empty() {
                    group_title = format!("🚩 {}", n.title);
                }
            }
        }

        if group_title.is_empty() {
            group_title = format!("📦 分支流 ({})", comp.first().unwrap_or(&"Scope".to_string()));
        }

        let pad = 24.0;
        let top_pad = 42.0;
        let bounds = (min_x - pad, min_y - top_pad, max_x + pad, max_y + pad);

        groups.push(SubgraphGroup {
            id: format!("group_{}", comp.first().unwrap_or(&"0".to_string())),
            title: group_title,
            bounds,
            node_count: comp.len(),
            color: group_color,
        });
    }

    // 2. 补齐其余游离连通子图
    for node in &graph.nodes {
        if visited.contains(&node.id) {
            continue;
        }

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

        if comp.len() < 2 {
            continue;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for id in &comp {
            if let Some(n) = node_map.get(id) {
                min_x = min_x.min(n.pos.0);
                min_y = min_y.min(n.pos.1);
                max_x = max_x.max(n.pos.0 + n.size.0);
                max_y = max_y.max(n.pos.1 + n.size.1);
            }
        }

        let pad = 24.0;
        let top_pad = 42.0;
        let bounds = (min_x - pad, min_y - top_pad, max_x + pad, max_y + pad);

        groups.push(SubgraphGroup {
            id: format!("group_{}", comp.first().unwrap_or(&"0".to_string())),
            title: format!("📦 分支流 ({})", comp.first().unwrap_or(&"Scope".to_string())),
            bounds,
            node_count: comp.len(),
            color: gpui::hsla(215.0 / 360.0, 0.40, 0.50, 1.0),
        });
    }

    groups
}
