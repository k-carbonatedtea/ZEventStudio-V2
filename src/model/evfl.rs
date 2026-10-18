pub use super::evfl_builder::revfl_flowchart_to_graph;
use super::graph::Graph;
use revfl::evfl::EventFlow;

/// 从任天堂 BFEVFL 二进制字节流或已保存的 JSON 流程图中加载并转换为 Graph 图结构
pub fn load_bfevfl_from_bytes(bytes: &[u8]) -> Result<Graph, String> {
    if bytes.is_empty() {
        return Err("文件内容为空".to_string());
    }

    // 1. 如果是已保存的 JSON 结构 (以 '{' 开头)，优先反序列化为 Graph 图结构
    let trimmed = bytes
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .map(|i| &bytes[i..])
        .unwrap_or(bytes);

    if trimmed.starts_with(b"{") {
        if let Ok(graph) = serde_json::from_slice::<Graph>(bytes) {
            return Ok(graph);
        }
    }

    // 2. 检查 BFEVFL 官方二进制魔数 (Magic: "BFEVFL\0\0")
    if bytes.len() >= 8 && &bytes[0..6] == b"BFEVFL" {
        let read_result = std::panic::catch_unwind(|| {
            let mut evfl = EventFlow::new();
            evfl.read(bytes);
            evfl
        });

        match read_result {
            Ok(evfl) => {
                if let Some(flowchart) = &evfl.flowchart {
                    Ok(revfl_flowchart_to_graph(flowchart, &evfl.name))
                } else {
                    Err("BFEVFL 文件中未包含有效的 Flowchart 流程图数据".to_string())
                }
            }
            Err(_) => Err("解析 BFEVFL 二进制字节流时发生异常".to_string()),
        }
    } else {
        // 3. 容错降级：尝试反序列化为 JSON
        serde_json::from_slice::<Graph>(bytes).map_err(|e| {
            format!("文件既不是有效 BFEVFL 二进制格式，也无法作为 JSON 流程图解析: {}", e)
        })
    }
}

/// 将 Graph 流程图结构序列化保存为 JSON / EVFL 字节流
pub fn save_bfevfl_to_bytes(graph: &Graph) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(graph).map_err(|e| format!("序列化 BFEVFL 数据失败: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_100enemy_bfevfl() {
        let bytes = include_bytes!("../../100enemy.bfevfl");
        let graph = load_bfevfl_from_bytes(bytes).expect("Failed to parse 100enemy.bfevfl with Revfl");
        assert_eq!(graph.nodes.len(), 89);
        // 包含 Fork/Join 汇合虚线
        assert!(graph.edges.iter().any(|e| e.is_dashed));
    }

    #[test]
    fn test_bfevfl_json_roundtrip() {
        let bytes = include_bytes!("../../100enemy.bfevfl");
        let original_graph = load_bfevfl_from_bytes(bytes).unwrap();
        let saved_json_bytes = save_bfevfl_to_bytes(&original_graph).unwrap();

        let reloaded_graph = load_bfevfl_from_bytes(&saved_json_bytes).unwrap();
        assert_eq!(reloaded_graph.nodes.len(), original_graph.nodes.len());
    }

    #[test]
    fn test_load_bfevfl_invalid_bytes() {
        assert!(load_bfevfl_from_bytes(b"").is_err());
        assert!(load_bfevfl_from_bytes(b"INVALID_DATA_HEADER").is_err());
    }
}
