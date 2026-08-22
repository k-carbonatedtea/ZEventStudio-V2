use crate::model::Graph;

/// 已打开文档标签项 (Document Tab)
#[derive(Debug, Clone, PartialEq)]
pub struct OpenDocumentTab {
    pub id: String,
    pub title: String,
    pub file_path: String,
    pub file_type: String,
    pub graph: Option<Graph>,
    pub is_modified: bool,
}

impl OpenDocumentTab {
    pub fn new(
        id: String,
        title: String,
        file_path: String,
        file_type: String,
        graph: Option<Graph>,
    ) -> Self {
        Self {
            id,
            title,
            file_path,
            file_type,
            graph,
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
}
