use super::config::LayoutNode;
use std::collections::HashMap;

/// Brandes-Köpf 风格的多轮对称中位数水平坐标计算与松弛优化
pub fn brandes_kopf_coordinates(
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
    for _ in 0..10 {
        // 自顶向下对齐 (Top-to-Bottom: Align with parents)
        for layer in layers.iter().take(max_rank).skip(1) {
            for id in layer {
                align_with_neighbors(id, preds, &mut coords);
            }
            enforce_bidirectional_spacing(layer, nodes, &mut coords, node_spacing_x);
        }

        // 自底向上对齐 (Bottom-to-Top: Align with children)
        for layer in layers.iter().take(max_rank.saturating_sub(1)).rev() {
            for id in layer {
                align_with_neighbors(id, succs, &mut coords);
            }
            enforce_bidirectional_spacing(layer, nodes, &mut coords, node_spacing_x);
        }
    }

    // 3. 对称居中微调：确保分叉与汇合点处于分支包围盒中心
    for layer in layers {
        for id in layer {
            let has_fork = succs.get(id).map(|s| s.len() > 1).unwrap_or(false);
            if has_fork {
                center_fork_node(id, succs, &mut coords);
            }
            let has_join = preds.get(id).map(|p| p.len() > 1).unwrap_or(false);
            if has_join {
                center_join_node(id, preds, &mut coords);
            }
        }
    }

    // 4. 入口点与顶层主干全局居中
    let mut all_min = f32::INFINITY;
    let mut all_max = f32::NEG_INFINITY;
    for (id, &x) in coords.iter() {
        let w = nodes.get(id).map(|n| n.width).unwrap_or(240.0);
        all_min = all_min.min(x - w / 2.0);
        all_max = all_max.max(x + w / 2.0);
    }
    if all_min.is_finite() && all_max.is_finite() {
        let comp_center_x = (all_min + all_max) / 2.0;

        for layer in layers {
            if layer.len() == 1 {
                let single_id = &layer[0];
                coords.insert(single_id.clone(), comp_center_x);
            } else {
                break;
            }
        }
    }

    // 5. 最终层内间距双向无偏保障
    for layer in layers {
        enforce_bidirectional_spacing(layer, nodes, &mut coords, node_spacing_x);
    }

    coords
}

/// 辅助函数：将节点对齐到其邻居节点群的平均 X 坐标
fn align_with_neighbors(
    id: &str,
    neighbors_map: &HashMap<String, Vec<String>>,
    coords: &mut HashMap<String, f32>,
) {
    if let Some(list) = neighbors_map.get(id) {
        if !list.is_empty() {
            let sum: f32 = list.iter().filter_map(|p| coords.get(p).copied()).sum();
            let target_x = sum / (list.len() as f32);
            coords.insert(id.to_string(), target_x);
        }
    }
}

/// 辅助函数：将分支节点对称居中于其子节点包围盒中心
fn center_fork_node(
    id: &str,
    succs: &HashMap<String, Vec<String>>,
    coords: &mut HashMap<String, f32>,
) {
    if let Some(children) = succs.get(id) {
        let child_xs: Vec<f32> = children
            .iter()
            .filter_map(|c| coords.get(c).copied())
            .collect();
        if !child_xs.is_empty() {
            let min_c = child_xs.iter().copied().fold(f32::INFINITY, f32::min);
            let max_c = child_xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            coords.insert(id.to_string(), (min_c + max_c) / 2.0);
        }
    }
}

/// 辅助函数：将多入度汇聚节点对称居中于其父节点包围盒中心
fn center_join_node(
    id: &str,
    preds: &HashMap<String, Vec<String>>,
    coords: &mut HashMap<String, f32>,
) {
    if let Some(parents) = preds.get(id) {
        let parent_xs: Vec<f32> = parents
            .iter()
            .filter_map(|p| coords.get(p).copied())
            .collect();
        if !parent_xs.is_empty() {
            let min_p = parent_xs.iter().copied().fold(f32::INFINITY, f32::min);
            let max_p = parent_xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            coords.insert(id.to_string(), (min_p + max_p) / 2.0);
        }
    }
}

/// 双向无偏层内间距强制保障 (Left-to-Right + Right-to-Left Averaging)
pub fn enforce_bidirectional_spacing(
    layer: &[String],
    nodes: &HashMap<String, LayoutNode>,
    coords: &mut HashMap<String, f32>,
    node_spacing_x: f32,
) {
    if layer.len() <= 1 {
        return;
    }

    // 1. 从左至右推挤
    let mut left_pass: Vec<f32> = Vec::with_capacity(layer.len());
    let mut cur_l = *coords.get(&layer[0]).unwrap_or(&0.0);
    left_pass.push(cur_l);

    for i in 1..layer.len() {
        let left_id = &layer[i - 1];
        let right_id = &layer[i];
        let prev_x = left_pass[i - 1];
        let target_x = *coords.get(right_id).unwrap_or(&0.0);
        let left_w = nodes.get(left_id).map(|n| n.width).unwrap_or(240.0);
        let right_w = nodes.get(right_id).map(|n| n.width).unwrap_or(240.0);
        let min_required = prev_x + (left_w + right_w) / 2.0 + node_spacing_x;
        cur_l = target_x.max(min_required);
        left_pass.push(cur_l);
    }

    // 2. 从右至左推挤
    let mut right_pass: Vec<f32> = vec![0.0; layer.len()];
    let last_idx = layer.len() - 1;
    let mut cur_r = *coords.get(&layer[last_idx]).unwrap_or(&0.0);
    right_pass[last_idx] = cur_r;

    for i in (0..last_idx).rev() {
        let left_id = &layer[i];
        let right_id = &layer[i + 1];
        let next_x = right_pass[i + 1];
        let target_x = *coords.get(left_id).unwrap_or(&0.0);
        let left_w = nodes.get(left_id).map(|n| n.width).unwrap_or(240.0);
        let right_w = nodes.get(right_id).map(|n| n.width).unwrap_or(240.0);
        let max_allowed = next_x - (left_w + right_w) / 2.0 - node_spacing_x;
        cur_r = target_x.min(max_allowed);
        right_pass[i] = cur_r;
    }

    // 3. 综合取中值，保证对称无偏
    for (i, id) in layer.iter().enumerate() {
        let final_x = (left_pass[i] + right_pass[i]) / 2.0;
        coords.insert(id.clone(), final_x);
    }
}
