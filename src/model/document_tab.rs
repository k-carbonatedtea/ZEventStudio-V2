use crate::model::Graph;

/// 已打开文档标签项 (Document Tab)
#[derive(Debug, Clone, PartialEq)]
pub struct OpenDocumentTab {
    pub id: String,
    pub title: String,
    pub file_path: String,
    pub file_type: String,
    pub graph: Option<Graph>,
    pub original_graph: Option<Graph>, // 原始加载时的快照，用于精确比对是否被真正修改
    pub is_modified: bool,
}

impl OpenDocumentTab {
    pub fn new(
        id: String,
        title: String,
        file_path: String,
        file_type: String,
        graph: Option<Graph>,
        original_graph: Option<Graph>,
    ) -> Self {
        Self {
            id,
            title,
            file_path,
            file_type,
            graph,
            original_graph,
            is_modified: false,
        }
    }

    /// 获取文件类型对应的精美图标 ({} / ≡ / ⚡ / 📦 / 📄)
    pub fn icon(&self) -> &'static str {
        match self.file_type.to_lowercase().as_str() {
            "json" => "{}",
            "bfevfl" | "evfl" => "⚡",
            "txt" | "log" => "≡",
            "sarc" | "pack" | "sbeventpack" | "sbactorpack" => "📦",
            _ => "📄",
        }
    }

    /// 重新根据当前图与初始快照比对，精确更新 is_modified 状态
    pub fn recompute_modified(&mut self, current: &Graph) {
        if let Some(orig) = &self.original_graph {
            self.is_modified = is_graph_modified(current, orig);
        } else {
            // 新建未保存的空白文件：如果包含节点则标记修改
            self.is_modified = !current.nodes.is_empty() || !current.edges.is_empty();
        }
    }
}

/// 深度精确比对当前图拓扑与原始快照是否发生实质性变动
pub fn is_graph_modified(current: &Graph, original: &Graph) -> bool {
    // 1. 节点或连线数量比对
    if current.nodes.len() != original.nodes.len() || current.edges.len() != original.edges.len() {
        return true;
    }

    // 2. 角色定义比对
    if current.actors != original.actors {
        return true;
    }

    // 3. 逐个节点属性、副标题、端口引脚与坐标比对
    for orig_node in &original.nodes {
        if let Some(curr_node) = current.find_node(&orig_node.id) {
            if curr_node.title != orig_node.title
                || curr_node.subtitle != orig_node.subtitle
                || curr_node.kind != orig_node.kind
                || curr_node.properties != orig_node.properties
                || curr_node.inputs != orig_node.inputs
                || curr_node.outputs != orig_node.outputs
                || (curr_node.pos.0 - orig_node.pos.0).abs() > 0.5
                || (curr_node.pos.1 - orig_node.pos.1).abs() > 0.5
            {
                return true;
            }
        } else {
            return true;
        }
    }

    // 4. 连线拓扑比对
    for orig_edge in &original.edges {
        let exists = current.edges.iter().any(|e| {
            e.source_node_id == orig_edge.source_node_id
                && e.source_port_id == orig_edge.source_port_id
                && e.target_node_id == orig_edge.target_node_id
                && e.target_port_id == orig_edge.target_port_id
        });
        if !exists {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NodeData, NodeKind};

    #[test]
    fn test_is_graph_modified_add_and_delete() {
        let mut original = Graph::new("TestFlow");
        original.add_node(NodeData::new("Node1", "Event1", NodeKind::Action, (100.0, 100.0)));

        let mut current = original.clone();
        assert!(!is_graph_modified(&current, &original));

        // 添加一个节点后，应判定为已修改
        current.add_node(NodeData::new("Node2", "Event2", NodeKind::Action, (200.0, 200.0)));
        assert!(is_graph_modified(&current, &original));

        // 删除该节点后，应精准恢复判定为未修改
        current.remove_node("Node2");
        assert!(!is_graph_modified(&current, &original));
    }
}

