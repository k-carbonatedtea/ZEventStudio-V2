use super::config::LayoutConfig;
use crate::model::graph::Graph;
use crate::model::node::NodeKind;
use std::collections::{HashMap, HashSet, VecDeque};

/// 单行最大分支容纳数 (超出则自动向下折行级联排列，杜绝横向无休止扩展)
const MAX_CHILDREN_PER_ROW: usize = 2;

struct TreeLayoutContext<'a> {
    tree_children: &'a HashMap<String, Vec<String>>,
    bounds: &'a HashMap<String, (f32, f32)>,
    node_indices: &'a HashMap<String, usize>,
    graph: &'a Graph,
    config: &'a LayoutConfig,
}

/// 紧凑向下流动排版引擎 (Downward-Prioritized Compact Forest Layout)
/// 1. 单后继流程节点（Action / Seq）严格竖直向下直落对齐（X 轴一致，0 偏移）；
/// 2. 多分支决策节点（Fork / Switch / Selector）按紧凑行向下级联折行；
/// 3. 父节点严格对称居中于子节点群中轴；
/// 4. 紧凑排布（间隙收紧），长宽比均衡优化为 ~1:1 至 16:9。
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

    // 1. 寻找根节点 (Root / EntryPoint / 入度为 0 节点)
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
    let mut q: VecDeque<String> = VecDeque::new();

    for r in &roots {
        visited.insert(r.clone());
        q.push_back(r.clone());
    }

    while let Some(curr) = q.pop_front() {
        if let Some(targets) = directed_adj.get(&curr) {
            for target in targets {
                if comp_set.contains(target) && !visited.contains(target) {
                    visited.insert(target.clone());
                    tree_children.entry(curr.clone()).or_default().push(target.clone());
                    q.push_back(target.clone());
                }
            }
        }
    }

    for id in comp {
        if !visited.contains(id) {
            roots.push(id.clone());
        }
    }

    // 3. 递归计算紧凑子树包围盒 (Recursive Subtree Bounding Box)
    let mut subtree_bounds: HashMap<String, (f32, f32)> = HashMap::new();
    for r in &roots {
        calc_compact_subtree_bounds(
            r,
            &tree_children,
            node_indices,
            graph,
            config,
            &mut subtree_bounds,
        );
    }

    // 4. 递归分配几何坐标 (Assign Compact Tree Coordinates)
    let mut coords: HashMap<String, (f32, f32)> = HashMap::new();
    let mut cur_root_x = comp_start_x;

    let ctx = TreeLayoutContext {
        tree_children: &tree_children,
        bounds: &subtree_bounds,
        node_indices,
        graph,
        config,
    };

    for r in &roots {
        let (root_w, _) = *subtree_bounds.get(r).unwrap_or(&(260.0, 72.0));
        assign_compact_node_coords(r, cur_root_x, comp_start_y, &ctx, &mut coords);
        cur_root_x += root_w + config.node_spacing_x * 1.5;
    }

    // 5. 应用最终坐标至 Graph 节点
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for id in comp {
        if let Some(&(top_left_x, top_left_y)) = coords.get(id) {
            if let Some(&idx) = node_indices.get(id) {
                let w = graph.nodes[idx].size.0;
                let h = graph.nodes[idx].size.1;
                graph.nodes[idx].pos = (top_left_x, top_left_y);

                min_x = min_x.min(top_left_x);
                max_x = max_x.max(top_left_x + w);
                max_y = max_y.max(top_left_y + h);
            }
        }
    }

    if min_x == f32::INFINITY {
        min_x = comp_start_x;
        max_x = comp_start_x + 260.0;
        max_y = comp_start_y + 80.0;
    }

    let comp_w = (max_x - min_x).max(260.0);
    let comp_h = (max_y - comp_start_y).max(80.0);
    (comp_w, comp_h)
}

/// 递归计算紧凑子树所需的 (Width, Height)
fn calc_compact_subtree_bounds(
    node_id: &str,
    tree_children: &HashMap<String, Vec<String>>,
    node_indices: &HashMap<String, usize>,
    graph: &Graph,
    config: &LayoutConfig,
    bounds: &mut HashMap<String, (f32, f32)>,
) -> (f32, f32) {
    let (self_w, self_h) = node_indices
        .get(node_id)
        .map(|&idx| graph.nodes[idx].size)
        .unwrap_or((260.0, 72.0));

    let children = tree_children.get(node_id);
    if children.is_none() || children.unwrap().is_empty() {
        let b = (self_w, self_h);
        bounds.insert(node_id.to_string(), b);
        return b;
    }

    let child_list = children.unwrap();

    // 单子节点：直接垂直串联直排
    if child_list.len() == 1 {
        let (cw, ch_h) = calc_compact_subtree_bounds(
            &child_list[0],
            tree_children,
            node_indices,
            graph,
            config,
            bounds,
        );
        let w = self_w.max(cw);
        let h = self_h + config.layer_spacing_y + ch_h;
        let b = (w, h);
        bounds.insert(node_id.to_string(), b);
        return b;
    }

    // 多子节点：按 MAX_CHILDREN_PER_ROW 紧凑向下折行级联排列
    let mut child_bounds_list = Vec::with_capacity(child_list.len());
    for child in child_list {
        let cb = calc_compact_subtree_bounds(
            child,
            tree_children,
            node_indices,
            graph,
            config,
            bounds,
        );
        child_bounds_list.push(cb);
    }

    let mut total_w = self_w;
    let mut total_h = self_h + config.layer_spacing_y;

    for chunk in child_bounds_list.chunks(MAX_CHILDREN_PER_ROW) {
        let row_w: f32 = chunk.iter().map(|(w, _)| *w).sum::<f32>()
            + (chunk.len().saturating_sub(1) as f32) * config.node_spacing_x;
        let row_max_h: f32 = chunk.iter().map(|(_, h)| *h).fold(0.0, f32::max);

        total_w = total_w.max(row_w);
        total_h += row_max_h + config.layer_spacing_y;
    }

    let b = (total_w, total_h);
    bounds.insert(node_id.to_string(), b);
    b
}

/// 递归分配节点坐标：父节点与子节点流严格竖向向下对齐
fn assign_compact_node_coords(
    node_id: &str,
    left_x: f32,
    top_y: f32,
    ctx: &TreeLayoutContext,
    coords: &mut HashMap<String, (f32, f32)>,
) {
    let (self_w, self_h) = ctx
        .node_indices
        .get(node_id)
        .map(|&idx| ctx.graph.nodes[idx].size)
        .unwrap_or((260.0, 72.0));

    let (total_w, _) = *ctx.bounds.get(node_id).unwrap_or(&(self_w, self_h));
    let center_x = left_x + total_w / 2.0;
    let top_left_x = center_x - self_w / 2.0;

    coords.insert(node_id.to_string(), (top_left_x, top_y));

    let children = ctx.tree_children.get(node_id);
    if children.is_none() || children.unwrap().is_empty() {
        return;
    }

    let child_list = children.unwrap();

    // 单子节点：严格向下同轴直排
    if child_list.len() == 1 {
        let child = &child_list[0];
        let (cw, _) = *ctx.bounds.get(child).unwrap_or(&(260.0, 72.0));
        let child_left_x = center_x - cw / 2.0;
        let child_top_y = top_y + self_h + ctx.config.layer_spacing_y;
        assign_compact_node_coords(child, child_left_x, child_top_y, ctx, coords);
        return;
    }

    // 多子节点：按紧凑行向下级联排列
    let mut cur_row_y = top_y + self_h + ctx.config.layer_spacing_y;

    for chunk in child_list.chunks(MAX_CHILDREN_PER_ROW) {
        let row_w: f32 = chunk
            .iter()
            .map(|c| ctx.bounds.get(c).map(|(w, _)| *w).unwrap_or(260.0))
            .sum::<f32>()
            + (chunk.len().saturating_sub(1) as f32) * ctx.config.node_spacing_x;

        let row_max_h: f32 = chunk
            .iter()
            .map(|c| ctx.bounds.get(c).map(|(_, h)| *h).unwrap_or(72.0))
            .fold(0.0, f32::max);

        let mut row_left = center_x - row_w / 2.0;

        for child in chunk {
            let (cw, _) = *ctx.bounds.get(child).unwrap_or(&(260.0, 72.0));
            assign_compact_node_coords(child, row_left, cur_row_y, ctx, coords);
            row_left += cw + ctx.config.node_spacing_x;
        }

        cur_row_y += row_max_h + ctx.config.layer_spacing_y;
    }
}
