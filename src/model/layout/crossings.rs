use crate::model::graph::Graph;
use std::collections::HashMap;

/// 端口物理偏移偏置映射类型
pub type PortBiasMaps = (HashMap<(String, String), f32>, HashMap<(String, String), f32>);

/// 计算图谱连线所对应的引脚物理偏移权重 (Port-Aware Biases)
pub fn compute_port_biases(
    graph: &Graph,
    augmented_edges: &[(String, String)],
) -> PortBiasMaps {
    let mut orig_out_bias: HashMap<(String, String), f32> = HashMap::new();
    let mut orig_in_bias: HashMap<(String, String), f32> = HashMap::new();

    for edge in &graph.edges {
        if let Some(s_node) = graph.find_node(&edge.source_node_id) {
            if s_node.outputs.len() > 1 {
                if let Some(idx) = s_node.outputs.iter().position(|p| p.id == edge.source_port_id) {
                    let total = s_node.outputs.len() as f32;
                    let bias = ((idx as f32 + 1.0) / (total + 1.0)) - 0.5;
                    orig_out_bias.insert((edge.source_node_id.clone(), edge.target_node_id.clone()), bias);
                }
            }
        }
        if let Some(t_node) = graph.find_node(&edge.target_node_id) {
            if t_node.inputs.len() > 1 {
                if let Some(idx) = t_node.inputs.iter().position(|p| p.id == edge.target_port_id) {
                    let total = t_node.inputs.len() as f32;
                    let bias = ((idx as f32 + 1.0) / (total + 1.0)) - 0.5;
                    orig_in_bias.insert((edge.source_node_id.clone(), edge.target_node_id.clone()), bias);
                }
            }
        }
    }

    let mut out_biases: HashMap<(String, String), f32> = HashMap::new();
    let mut in_biases: HashMap<(String, String), f32> = HashMap::new();

    for (u, v) in augmented_edges {
        if let Some(&b) = orig_out_bias.get(&(u.clone(), v.clone())) {
            out_biases.insert((u.clone(), v.clone()), b);
        } else if v.starts_with("__dummy_") {
            let target = v.split('_').next_back().unwrap_or(v);
            if let Some(&b) = orig_out_bias.get(&(u.clone(), target.to_string())) {
                out_biases.insert((u.clone(), v.clone()), b);
            }
        }


        if let Some(&b) = orig_in_bias.get(&(u.clone(), v.clone())) {
            in_biases.insert((u.clone(), v.clone()), b);
        }
    }

    (out_biases, in_biases)
}

/// 重心扫描交叉最小化 (引脚感知质心排版算法 Port-Aware Barycenter Crossing Reduction)
pub fn reduce_crossings(
    mut layers: Vec<Vec<String>>,
    max_rank: usize,
    aug_pred: &HashMap<String, Vec<String>>,
    aug_succ: &HashMap<String, Vec<String>>,
    out_biases: &HashMap<(String, String), f32>,
    in_biases: &HashMap<(String, String), f32>,
) -> Vec<Vec<String>> {
    for iter in 0..12 {
        if iter % 2 == 0 {
            // 自顶向下扫描 (Top-to-Bottom): 子节点依据父节点坐标 + 出口引脚偏移计算质心
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
                                    .filter_map(|p| {
                                        let base_pos = pos_map.get(p.as_str()).copied()?;
                                        let bias = out_biases.get(&(p.clone(), id.clone())).copied().unwrap_or(0.0);
                                        Some(base_pos + bias * 0.8)
                                    })
                                    .sum();
                                return (id.clone(), sum / (preds.len() as f32));
                            }
                        }
                        (id.clone(), orig_idx as f32)
                    })
                    .collect();

                scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                layers[r] = scored.into_iter().map(|(id, _)| id).collect();
            }
        } else {
            // 自底向上扫描 (Bottom-to-Top): 父节点依据子节点坐标 + 入口引脚偏移计算质心
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
                                    .filter_map(|s| {
                                        let base_pos = pos_map.get(s.as_str()).copied()?;
                                        let bias = in_biases.get(&(id.clone(), s.clone())).copied().unwrap_or(0.0);
                                        Some(base_pos + bias * 0.8)
                                    })
                                    .sum();
                                return (id.clone(), sum / (succs.len() as f32));
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
    layers
}
