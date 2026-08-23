#![allow(dead_code)]

use super::config::LayoutNode;
use std::collections::HashMap;

/// 层次化子树中位数水平坐标对齐与对称松弛算法
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

    // 2. 迭代进行自顶向下展开与自底向上父节点对称居中
    for _ in 0..8 {
        // 自底向上 (Bottom-to-Top): 父节点居中于子节点群中点
        for layer in layers.iter().take(max_rank.saturating_sub(1)).rev() {
            for id in layer {
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
            enforce_bidirectional_spacing(layer, nodes, &mut coords, node_spacing_x);
        }

        // 自顶向下 (Top-to-Bottom): 子节点依据父节点对齐
        for layer in layers.iter().take(max_rank).skip(1) {
            for id in layer {
                if let Some(parents) = preds.get(id) {
                    let parent_xs: Vec<f32> = parents
                        .iter()
                        .filter_map(|p| coords.get(p).copied())
                        .collect();
                    if !parent_xs.is_empty() {
                        let sum: f32 = parent_xs.iter().sum();
                        coords.insert(id.to_string(), sum / (parent_xs.len() as f32));
                    }
                }
            }
            enforce_bidirectional_spacing(layer, nodes, &mut coords, node_spacing_x);
        }
    }

    // 3. 入口主干居中对齐
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

    // 4. 最终层内间距双向无偏保障
    for layer in layers {
        enforce_bidirectional_spacing(layer, nodes, &mut coords, node_spacing_x);
    }

    coords
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
