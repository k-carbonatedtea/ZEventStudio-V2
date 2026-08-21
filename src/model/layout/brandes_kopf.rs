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
    for _ in 0..12 {
        // 自顶向下对齐 (Top-to-Bottom: Align with parents)
        for r in 1..max_rank {
            let layer = &layers[r];
            for id in layer {
                align_with_neighbors(id, preds, &mut coords);
            }
            // 纠正同层间距重叠
            enforce_layer_spacing(layer, nodes, &mut coords, node_spacing_x);
        }

        // 自底向上对齐 (Bottom-to-Top: Align with children)
        for r in (0..max_rank - 1).rev() {
            let layer = &layers[r];
            for id in layer {
                align_with_neighbors(id, succs, &mut coords);
            }
            // 纠正同层间距重叠
            enforce_layer_spacing(layer, nodes, &mut coords, node_spacing_x);
        }
    }

    // 3. 对称居中微调：确保分叉与汇合点完美处于子节点群的几何中心
    for layer in layers {
        for id in layer {
            let has_fork = succs.get(id).map(|s| s.len() > 1).unwrap_or(false);
            if has_fork {
                center_fork_node(id, succs, &mut coords);
            }
        }
    }

    // 4. 最终层内间距强制保障
    for layer in layers {
        enforce_layer_spacing(layer, nodes, &mut coords, node_spacing_x);
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

/// 强制同层节点从左向右保持最小间距，并在中心发生偏移时进行对称平移
pub fn enforce_layer_spacing(
    layer: &[String],
    nodes: &HashMap<String, LayoutNode>,
    coords: &mut HashMap<String, f32>,
    node_spacing_x: f32,
) {
    if layer.len() <= 1 {
        return;
    }

    // 从左至右推挤，避免重叠
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
