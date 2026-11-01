use crate::model::code_document::CodeDocumentState;
use crate::model::Graph;

/// 文档标签展示形态种类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentTabKind {
    /// 交互式事件流程图
    Flowchart,
    /// 交互式过场动画时间线
    Timeline,
    /// 多格式通用代码编辑器
    CodeEditor,
    /// Git 代码双栏差异对比视图
    DiffViewer,
    /// 3D 模型与网格预览视图
    ModelPreview,
    /// Mii / UMii 形象参数可视化编辑器
    UmiiEditor,
}

/// 封包内子文件的代码差异项
#[derive(Debug, Clone, PartialEq)]
pub struct SubFileDiffItem {
    pub sub_path: String,
    pub format_name: String, // 如 "AAMP" / "BYML" / "MSBT" / "BFEVFL"
    pub diff: crate::model::git::GitFileDiff,
    pub rows: Vec<crate::model::git_diff_align::SideBySideRow>,
}

/// 对比模式标签页专属状态
#[derive(Debug, Clone, PartialEq)]
pub struct DiffViewerTabState {
    pub title: String,
    pub file_path: String,
    pub full_path: String,
    pub old_rev: String,
    pub new_rev: String,
    pub diff: crate::model::git::GitFileDiff,
    pub rows: Vec<crate::model::git_diff_align::SideBySideRow>,
    pub sub_files: Vec<SubFileDiffItem>,
    pub active_sub_file_idx: usize,
}

impl DiffViewerTabState {
    pub fn current_diff(&self) -> &crate::model::git::GitFileDiff {
        if let Some(sub) = self.sub_files.get(self.active_sub_file_idx) { &sub.diff } else { &self.diff }
    }
    pub fn current_rows(&self) -> &[crate::model::git_diff_align::SideBySideRow] {
        if let Some(sub) = self.sub_files.get(self.active_sub_file_idx) { &sub.rows } else { &self.rows }
    }
    pub fn current_format_name(&self) -> Option<&str> {
        self.sub_files.get(self.active_sub_file_idx).map(|s| s.format_name.as_str())
    }
    pub fn current_sub_path(&self) -> Option<&str> {
        self.sub_files.get(self.active_sub_file_idx).map(|s| s.sub_path.as_str())
    }
}

pub fn get_file_mtime(file_path: &str) -> Option<std::time::SystemTime> {
    if file_path.starts_with("SARC:") {
        let parts: Vec<&str> = file_path.trim_start_matches("SARC:").split("//").collect();
        parts.first().and_then(|f| std::fs::metadata(f).and_then(|m| m.modified()).ok())
    } else {
        std::fs::metadata(file_path).and_then(|m| m.modified()).ok()
    }
}

/// 已打开文档标签项 (Document Tab)
#[derive(Debug, Clone, PartialEq)]
pub struct OpenDocumentTab {
    pub id: String,
    pub title: String,
    pub file_path: String,
    pub file_type: String,
    pub tab_kind: DocumentTabKind,
    pub graph: Option<Graph>,
    pub original_graph: Option<Graph>,
    pub timeline_state: Option<crate::model::TimelineDocumentState>,
    pub code_state: Option<CodeDocumentState>,
    pub diff_state: Option<DiffViewerTabState>,
    pub model_state: Option<crate::model::ModelDocumentState>,
    pub umii_state: Option<crate::model::umii::UmiiDocumentState>,
    pub is_modified: bool,
    pub last_disk_mtime: Option<std::time::SystemTime>,
}

impl OpenDocumentTab {
    pub fn new(id: String, title: String, file_path: String, file_type: String, graph: Option<Graph>, original_graph: Option<Graph>) -> Self {
        let last_disk_mtime = get_file_mtime(&file_path);
        Self {
            id, title, file_path, file_type, tab_kind: DocumentTabKind::Flowchart,
            graph, original_graph, timeline_state: None, code_state: None, diff_state: None, model_state: None, umii_state: None,
            is_modified: false, last_disk_mtime,
        }
    }

    pub fn new_timeline(id: String, title: String, file_path: String, file_type: String, timeline: revfl::timeline::Timeline) -> Self {
        let last_disk_mtime = get_file_mtime(&file_path);
        let timeline_state = crate::model::TimelineDocumentState::new(timeline);
        Self {
            id, title, file_path, file_type, tab_kind: DocumentTabKind::Timeline,
            graph: None, original_graph: None, timeline_state: Some(timeline_state), code_state: None, diff_state: None, model_state: None, umii_state: None,
            is_modified: false, last_disk_mtime,
        }
    }

    pub fn new_code(id: String, title: String, file_path: String, file_type: String, code_state: CodeDocumentState) -> Self {
        let last_disk_mtime = get_file_mtime(&file_path);
        Self {
            id, title, file_path, file_type, tab_kind: DocumentTabKind::CodeEditor,
            graph: None, original_graph: None, timeline_state: None, code_state: Some(code_state), diff_state: None, model_state: None, umii_state: None,
            is_modified: false, last_disk_mtime,
        }
    }

    pub fn new_diff(id: String, title: String, file_path: String, diff_state: DiffViewerTabState) -> Self {
        Self {
            id, title, file_path, file_type: "diff".to_string(), tab_kind: DocumentTabKind::DiffViewer,
            graph: None, original_graph: None, timeline_state: None, code_state: None, diff_state: Some(diff_state), model_state: None, umii_state: None,
            is_modified: false, last_disk_mtime: None,
        }
    }

    pub fn new_model(id: String, title: String, file_path: String, file_type: String, model_state: crate::model::ModelDocumentState) -> Self {
        let last_disk_mtime = get_file_mtime(&file_path);
        Self {
            id, title, file_path, file_type, tab_kind: DocumentTabKind::ModelPreview,
            graph: None, original_graph: None, timeline_state: None, code_state: None, diff_state: None, model_state: Some(model_state), umii_state: None,
            is_modified: false, last_disk_mtime,
        }
    }

    pub fn new_umii(id: String, title: String, file_path: String, file_type: String, umii_state: crate::model::umii::UmiiDocumentState) -> Self {
        let last_disk_mtime = get_file_mtime(&file_path);
        Self {
            id, title, file_path, file_type, tab_kind: DocumentTabKind::UmiiEditor,
            graph: None, original_graph: None, timeline_state: None, code_state: None, diff_state: None, model_state: None, umii_state: Some(umii_state),
            is_modified: false, last_disk_mtime,
        }
    }

    pub fn icon(&self) -> &'static str {
        if self.tab_kind == DocumentTabKind::DiffViewer { return "⇄"; }
        if self.tab_kind == DocumentTabKind::Timeline { return "🎬"; }
        if self.tab_kind == DocumentTabKind::ModelPreview { return "🧊"; }
        if self.tab_kind == DocumentTabKind::UmiiEditor { return "👤"; }
        if let Some(ref code) = self.code_state { return code.format.icon(); }
        match self.file_type.to_lowercase().as_str() {
            "bfres" | "sbfres" | "fres" => "🧊",
            "bumii" | "umii" | "mii" => "👤",
            "msbt" | "smsbt" => "💬",
            "byml" | "bgyml" | "sbyml" | "aamp" | "bxml" | "baev" | "baiprog" | "sbaiprog" => "⚙️",
            "json" => "{}",
            "bfevfl" | "evfl" => "⚡",
            "bfevtm" | "evtm" => "🎬",
            "yaml" | "yml" => "📜",
            "txt" | "log" => "≡",
            "sarc" | "pack" | "sbeventpack" | "sbactorpack" | "ssarc" => "📦",
            _ => "📄",
        }
    }

    pub fn recompute_modified(&mut self) {
        if let Some(ref code) = self.code_state {
            if code.is_modified() { self.is_modified = true; return; }
        }
        match self.tab_kind {
            DocumentTabKind::Flowchart => {
                if let (Some(curr), Some(orig)) = (&self.graph, &self.original_graph) {
                    self.is_modified = is_graph_modified(curr, orig);
                } else if let Some(curr) = &self.graph {
                    self.is_modified = !curr.nodes.is_empty() || !curr.edges.is_empty();
                }
            }
            DocumentTabKind::Timeline => {
                if let Some(ref ts) = self.timeline_state { self.is_modified = ts.is_modified(); }
            }
            DocumentTabKind::CodeEditor => {
                if let Some(ref code) = self.code_state { self.is_modified = code.is_modified(); }
            }
            DocumentTabKind::UmiiEditor => {
                if let Some(ref u) = self.umii_state { self.is_modified = u.is_modified(); }
            }
            DocumentTabKind::DiffViewer | DocumentTabKind::ModelPreview => { self.is_modified = false; }
        }
    }
}

pub fn is_graph_modified(current: &Graph, original: &Graph) -> bool {
    if current.nodes.len() != original.nodes.len() || current.edges.len() != original.edges.len() || current.actors != original.actors {
        return true;
    }
    for orig_node in &original.nodes {
        if let Some(curr_node) = current.find_node(&orig_node.id) {
            if curr_node.title != orig_node.title || curr_node.subtitle != orig_node.subtitle
                || curr_node.kind != orig_node.kind || curr_node.properties != orig_node.properties
                || curr_node.inputs != orig_node.inputs || curr_node.outputs != orig_node.outputs {
                return true;
            }
        } else {
            return true;
        }
    }
    for orig_edge in &original.edges {
        let exists = current.edges.iter().any(|e| {
            e.source_node_id == orig_edge.source_node_id && e.source_port_id == orig_edge.source_port_id
                && e.target_node_id == orig_edge.target_node_id && e.target_port_id == orig_edge.target_port_id
        });
        if !exists { return true; }
    }
    false
}
