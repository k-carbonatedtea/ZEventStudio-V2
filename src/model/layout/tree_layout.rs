use super::config::LayoutConfig;
use crate::model::graph::Graph;
use crate::model::node::NodeKind;
use std::collections::{HashMap, HashSet, VecDeque};

struct TreeLayoutContext<'a> {
    tree_children: &'a HashMap<String, Vec<String>>,
    widths: &'a HashMap<String, f32>,
    node_indices: &'a HashMap<String, usize>,
    graph: &'a Graph,
    config: &'a LayoutConfig,
}

/// 行为树主干展开排版引擎 (Behavior Tree Spanning Forest Layout)
/// 参考 Plasticity 行为树递归分层机制，保证：
/// 1. 根节点（Root AI_2 / Demo 入口）居顶展示；
/// 2. 各业务分支（战斗、探索、动作）自顶向下形成独立垂直走廊；
/// 3. 父节点严格水平居中于子节点群中轴线；
/// 4. 同级分支严格按端口顺位并排排列，杜绝不同子树交叉。
pub fn layout_tree_component(
    comp: &[String],
    graph: &mut Graph,
    node_indices: &HashMap<String, usize>,
    directed_adj: &HashMap<String, Vec<String>>,
    comp_start_x: f32,
    comp_start_y: f32,
    config: &LayoutConfig,
) -> (f32, f32) {
    if comp.is_empty() {
        return (0.0, 0.0);
    }

    let comp_set: HashSet<String> = comp.iter().cloned().collect();

    // 1. 寻找主根节点 (Root / EntryPoint / 入度为 0 节点)
    let mut roots: Vec<String> = Vec::new();
    for id in comp {
        if let Some(&idx) = node_indices.get(id) {
            let n = &graph.nodes[idx];
            if n.kind == NodeKind::EntryPoint
                || id.starts_with("ep_")
                || id.eq_ignore_ascii_case("ai_2")
                || n.title.to_lowercase().starts_with("root")
            {
                roots.push(id.clone());
            }
        }
    }
    if roots.is_empty() {
        let mut in_degrees: HashMap<String, usize> = HashMap::new();
        for u in comp {
            if let Some(targets) = directed_adj.get(u) {
                for v in targets {
                    if comp_set.contains(v) {
                        *in_degrees.entry(v.clone()).or_insert(0) += 1;
                    }
                }
            }
        }
        for id in comp {
            if *in_degrees.get(id).unwrap_or(&0) == 0 {
                roots.push(id.clone());
            }
        }
    }
    if roots.is_empty() {
        roots.push(comp[0].clone());
    }

    // 2. 提取行为树主干生成树 (Dominator Spanning Tree via BFS)
    let mut visited: HashSet<String> = HashSet::new();
    let mut tree_children: HashMap<String, Vec<String>> = HashMap::new();
    let mut depths: HashMap<String, usize> = HashMap::new();
    let mut q: VecDeque<String> = VecDeque::new();

    for r in &roots {
        visited.insert(r.clone());
        depths.insert(r.clone(), 0);
        q.push_back(r.clone());
    }

    while let Some(curr) = q.pop_front() {
        let curr_depth = *depths.get(&curr).unwrap_or(&0);
        if let Some(targets) = directed_adj.get(&curr) {
            for target in targets {
                if comp_set.contains(target) && !visited.contains(target) {
                    visited.insert(target.clone());
                    depths.insert(target.clone(), curr_depth + 1);
                    tree_children.entry(curr.clone()).or_default().push(target.clone());
                    q.push_back(target.clone());
                }
            }
        }
    }

    // 补齐未被主树直接覆盖的孤立弱连通节点
    for id in comp {
        if !visited.contains(id) {
            roots.push(id.clone());
            depths.insert(id.clone(), 0);
        }
    }

    // 3. 递归计算各子树所需包围盒宽度 (Recursive Subtree Width)
    let mut subtree_widths: HashMap<String, f32> = HashMap::new();
    for r in &roots {
        calc_subtree_width(r, &tree_children, node_indices, graph, config.node_spacing_x, &mut subtree_widths);
    }

    // 4. 递归分配几何坐标 (Assign Tree Coordinates)
    let mut coords: HashMap<String, (f32, f32)> = HashMap::new();
    let mut cur_root_x = comp_start_x;

    let ctx = TreeLayoutContext {
        tree_children: &tree_children,
        widths: &subtree_widths,
        node_indices,
        graph,
        config,
    };

    for r in &roots {
        let root_w = *subtree_widths.get(r).unwrap_or(&240.0);
        assign_node_coords(r, cur_root_x, comp_start_y, &ctx, &mut coords);
        cur_root_x += root_w + config.node_spacing_x * 1.5;
    }

    // 5. 应用最终坐标至 Graph 节点
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for id in comp {
        if let Some(&(cx, cy)) = coords.get(id) {
            if let Some(&idx) = node_indices.get(id) {
                let w = graph.nodes[idx].size.0;
                let h = graph.nodes[idx].size.1;
                let top_left_x = cx - w / 2.0;
                let top_left_y = cy;
                graph.nodes[idx].pos = (top_left_x, top_left_y);

                min_x = min_x.min(top_left_x);
                max_x = max_x.max(top_left_x + w);
                max_y = max_y.max(top_left_y + h);
            }
        }
    }

    if min_x == f32::INFINITY {
        min_x = comp_start_x;
        max_x = comp_start_x + 240.0;
        max_y = comp_start_y + 120.0;
    }

    let comp_w = (max_x - min_x).max(240.0);
    let comp_h = (max_y - comp_start_y).max(120.0);
    (comp_w, comp_h)
}

/// 递归计算子树宽度
fn calc_subtree_width(
    node_id: &str,
    tree_children: &HashMap<String, Vec<String>>,
    node_indices: &HashMap<String, usize>,
    graph: &Graph,
    spacing_x: f32,
    widths: &mut HashMap<String, f32>,
) -> f32 {
    let self_w = node_indices
        .get(node_id)
        .map(|&idx| graph.nodes[idx].size.0)
        .unwrap_or(240.0);

    let children = tree_children.get(node_id);
    if children.is_none() || children.unwrap().is_empty() {
        widths.insert(node_id.to_string(), self_w);
        return self_w;
    }

    let child_list = children.unwrap();
    let mut total_child_w = 0.0;
    for (i, child) in child_list.iter().enumerate() {
        let cw = calc_subtree_width(child, tree_children, node_indices, graph, spacing_x, widths);
        total_child_w += cw;
        if i > 0 {
            total_child_w += spacing_x;
        }
    }

    let w = self_w.max(total_child_w);
    widths.insert(node_id.to_string(), w);
    w
}

/// 递归分配节点坐标：父节点严格对称居中于子节点群中轴
fn assign_node_coords(
    node_id: &str,
    left_x: f32,
    layer_y: f32,
    ctx: &TreeLayoutContext,
    coords: &mut HashMap<String, (f32, f32)>,
) {
    let self_h = ctx
        .node_indices
        .get(node_id)
        .map(|&idx| ctx.graph.nodes[idx].size.1)
        .unwrap_or(100.0);

    let total_w = *ctx.widths.get(node_id).unwrap_or(&240.0);
    let children = ctx.tree_children.get(node_id);

    if children.is_none() || children.unwrap().is_empty() {
        let center_x = left_x + total_w / 2.0;
        coords.insert(node_id.to_string(), (center_x, layer_y));
        return;
    }

    let child_list = children.unwrap();
    let mut cur_child_left = left_x;
    let mut child_centers = Vec::new();
    let next_layer_y = layer_y + self_h + ctx.config.layer_spacing_y;

    for child in child_list {
        let cw = *ctx.widths.get(child).unwrap_or(&240.0);
        assign_node_coords(child, cur_child_left, next_layer_y, ctx, coords);
        let center_c = cur_child_left + cw / 2.0;
        child_centers.push(center_c);
        cur_child_left += cw + ctx.config.node_spacing_x;
    }

    let parent_center_x = if !child_centers.is_empty() {
        (child_centers.first().unwrap() + child_centers.last().unwrap()) / 2.0
    } else {
        left_x + total_w / 2.0
    };

    coords.insert(node_id.to_string(), (parent_center_x, layer_y));
}
