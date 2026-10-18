use crate::model::PortDirection;

#[derive(Debug, Clone, PartialEq)]
pub enum DragState {
    None,
    PanCanvas {
        start_mouse: (f32, f32),
        start_pan: (f32, f32),
    },
    DragNodes {
        start_mouse: (f32, f32),
        initial_positions: Vec<(String, (f32, f32))>,
    },
    Connecting {
        source_node_id: String,
        source_port_id: String,
        direction: PortDirection,
        source_pos: (f32, f32),
        current_mouse: (f32, f32),
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppWorkspaceTab {
    #[default]
    Welcome,
    Flowchart,
    CodeEditor,
    Actors,
    Events,
    Json,
    DiffViewer,
}


#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WelcomeTab {
    #[default]
    QuickStart,
    Recent,
}

pub use crate::model::RecentProjectItem;

/// 快速新建节点时的源连接端口信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickAddConnectionSource {
    pub node_id: String,
    pub port_id: String,
    pub direction: PortDirection,
}

/// 快速新建节点浮层的激活状态与位置
#[derive(Debug, Clone, PartialEq)]
pub struct QuickAddModalState {
    pub position: (f32, f32),
    pub source: Option<QuickAddConnectionSource>,
}

/// 未保存文件关闭确认弹窗状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsavedChangesModalState {
    pub tab_id: String,
    pub title: String,
    pub file_path: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveSettingsField {
    #[default]
    None,
    SwitchGameDir,
    SwitchDlcDir,
    SwitchExportDir,
    WiiuGameDir,
    WiiuUpdateDir,
    WiiuDlcDir,
    WiiuExportDir,
    AiDumpDir,
}

/// 文件资源管理器排序模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileSortMode {
    #[default]
    NameAsc,       // 名称升序 (A → Z)
    NameDesc,      // 名称降序 (Z → A)
    FileType,      // 按文件类型
    ModifiedFirst, // 已修改置顶
}

#[allow(dead_code)]
impl FileSortMode {
    pub fn label_key(&self) -> &'static str {
        match self {
            Self::NameAsc => "explorer.sort_name_asc",
            Self::NameDesc => "explorer.sort_name_desc",
            Self::FileType => "explorer.sort_type",
            Self::ModifiedFirst => "explorer.sort_modified",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Self::NameAsc => "A → Z",
            Self::NameDesc => "Z → A",
            Self::FileType => "类型",
            Self::ModifiedFirst => "修改优先",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::NameAsc => "🔤",
            Self::NameDesc => "🔤",
            Self::FileType => "🏷️",
            Self::ModifiedFirst => "📝",
        }
    }
}

/// 文件资源管理器类型筛选
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileFilterType {
    #[default]
    All,        // 全部文件
    EventFlow,  // 事件流 (.bfevfl, .evfl)
    Msbt,       // 文本对话 (.msbt, .smsbt)
    Param,      // 参数文件 (.byml, .bgyml, .aamp, ...)
    Sarc,       // 归档封包 (.sarc, .pack, ...)
    Modified,   // 仅已修改
}

#[allow(dead_code)]
impl FileFilterType {
    pub fn label_key(&self) -> &'static str {
        match self {
            Self::All => "explorer.filter_all",
            Self::EventFlow => "explorer.filter_event",
            Self::Msbt => "explorer.filter_msbt",
            Self::Param => "explorer.filter_param",
            Self::Sarc => "explorer.filter_sarc",
            Self::Modified => "explorer.filter_modified",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::All => "📁",
            Self::EventFlow => "⚡",
            Self::Msbt => "💬",
            Self::Param => "⚙️",
            Self::Sarc => "🗃️",
            Self::Modified => "●",
        }
    }
}

/// 活动栏功能面板枚举 (Activity Bar Tabs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivityBarTab {
    #[default]
    Explorer,      // 资源管理器
    Search,        // 全局搜索
    SourceControl, // 源文件管理
}

/// Git 提交输入框焦点
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitCommitFocus {
    #[default]
    None,
    Title,
    Description,
}
