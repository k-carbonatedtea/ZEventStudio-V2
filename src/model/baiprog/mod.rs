pub mod loader;
pub mod parser;
pub mod types;
pub mod writer;

pub use loader::load_baiprog_from_bytes;
pub use writer::save_baiprog_to_bytes;

/// 检查给定文件名或路径是否表示 BAI Program 文件
pub fn is_baiprog_extension(path_or_name: &str) -> bool {
    let lower = path_or_name.to_lowercase();
    lower.ends_with(".baiprog")
        || lower.ends_with(".sbaiprog")
        || (lower.contains("baiprog") && (lower.ends_with(".txt") || lower.ends_with(".yaml") || lower.ends_with(".yml")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_and_save_baiprog_txt() {
        let path = r"c:\Users\Administrator\Desktop\Zevent\baiprog.txt";
        let bytes = fs::read(path).expect("Read baiprog.txt");
        let graph = load_baiprog_from_bytes(&bytes, "Lynel.baiprog").expect("Load baiprog to graph");

        // 验证节点总数: 416 AI + 335 Action + 16 Demo Actions + 1 Meta Node = 768 节点
        assert_eq!(graph.nodes.len(), 768);
        // 验证连线总数: 1123 ChildIdx 连线 + 16 Demo 连线 = 1139 条
        assert_eq!(graph.edges.len(), 1139);

        // 验证 Root 节点存在
        let root_node = graph.nodes.iter().find(|n| n.id == "ai_2").expect("Root AI_2 node exists");
        assert_eq!(root_node.title, "Root");

        // 验证保存回 AAMP YAML 文本
        let saved_text_bytes = save_baiprog_to_bytes(&graph, true).expect("Save baiprog to text");
        assert!(!saved_text_bytes.is_empty());

        // 验证保存回 AAMP 二进制
        let saved_bin_bytes = save_baiprog_to_bytes(&graph, false).expect("Save baiprog to bin");
        assert_eq!(&saved_bin_bytes[0..4], b"AAMP");
        let reloaded_bin = load_baiprog_from_bytes(&saved_bin_bytes, "Lynel.baiprog").expect("Reload saved bin baiprog");
        assert_eq!(reloaded_bin.nodes.len(), graph.nodes.len());
        assert_eq!(reloaded_bin.edges.len(), graph.edges.len());
    }
}
