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
    DiffViewer,
    Quests,
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
    Plugins,       // 插件中心
    Quests,        // 任务管理器
}

/// 任务向导分段标签页分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuestCreatorTab {
    #[default]
    General,     // 基础信息 (分类、ID、标题、委托人、区域)
    Flags,       // Flag 设定 (Ready, Activated, Finish, Dependency)
    Coordinates, // 地图坐标与标点 (启用指示器、目标 Actor、X/Y/Z、从地图选点、OffFlag)
    Deploy,      // 冒险笔记文案与自动化五大支柱部署
}

/// 任务创建向导当前激活的输入框焦点
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuestCreatorFocusField {
    #[default]
    None,
    Name,         // 任务内部标识名，如 Quest_MyMiniQuest
    DisplayTitle, // 界面显示标题，如 "我的自定义任务"
    Orderer,      // 委托人 / NPC，如 Npc_AncientDoctor_Robbie
    Location,     // 区域 / 地点，如 Akkala
    DescText,     // 冒险笔记初始描述
    FinishText,   // 冒险笔记完成描述
    // Flag 设定字段
    ReadyFlag,
    ActivatedFlag,
    FinishFlag,
    DependencyFlag,
    // 地图坐标与标点字段
    IndicatorActor,
    CoordX,
    CoordY,
    CoordZ,
    IndicatorOffFlag,
}

/// 新建任务向导模态框状态
#[derive(Debug, Clone, PartialEq)]
pub struct QuestCreatorModalState {
    pub active_tab: QuestCreatorTab,
    pub name: String,
    pub quest_type: crate::model::quest::QuestType,
    pub orderer: String,
    pub location: String,
    pub display_title: String,
    pub desc_text: String,
    pub finish_text: String,
    // Flag 设定
    pub custom_flags: bool,
    pub ready_flag: String,
    pub activated_flag: String,
    pub finish_flag: String,
    pub dependency_flag: String,
    // 地图坐标与标点
    pub enable_indicator: bool,
    pub indicator_actor: String,
    pub coord_x: String,
    pub coord_y: String,
    pub coord_z: String,
    pub indicator_off_flag: String,
    pub indicator_has_far: bool,
    // 冒险笔记选项
    pub not_post_note: bool,
    // 自动化勾选项
    pub inject_titlebg: bool,
    pub inject_bootup_flags: bool,
    pub inject_ql_msbt: bool,
    pub scaffold_graph_nodes: bool,
    // 聚焦字段
    pub focused_field: QuestCreatorFocusField,
}

impl QuestCreatorModalState {
    /// 自动根据任务 ID 同步默认生命周期 Flag 名称（在未手动自定义 Flag 时生效）
    pub fn sync_flags_from_name(&mut self) {
        if !self.custom_flags {
            let clean = self.name.trim();
            self.ready_flag = format!("{}_Ready", clean);
            self.activated_flag = format!("{}_Activated", clean);
            self.finish_flag = format!("{}_Finish", clean);
        }
    }

    /// 重置 Flag 为自动派生命名
    pub fn reset_flags_to_default(&mut self) {
        self.custom_flags = false;
        self.sync_flags_from_name();
    }

    /// 获取当前激活输入框的内容文本切片
    pub fn active_input(&self) -> Option<&str> {
        match self.focused_field {
            QuestCreatorFocusField::Name => Some(&self.name),
            QuestCreatorFocusField::DisplayTitle => Some(&self.display_title),
            QuestCreatorFocusField::Orderer => Some(&self.orderer),
            QuestCreatorFocusField::Location => Some(&self.location),
            QuestCreatorFocusField::DescText => Some(&self.desc_text),
            QuestCreatorFocusField::FinishText => Some(&self.finish_text),
            QuestCreatorFocusField::ReadyFlag => Some(&self.ready_flag),
            QuestCreatorFocusField::ActivatedFlag => Some(&self.activated_flag),
            QuestCreatorFocusField::FinishFlag => Some(&self.finish_flag),
            QuestCreatorFocusField::DependencyFlag => Some(&self.dependency_flag),
            QuestCreatorFocusField::IndicatorActor => Some(&self.indicator_actor),
            QuestCreatorFocusField::CoordX => Some(&self.coord_x),
            QuestCreatorFocusField::CoordY => Some(&self.coord_y),
            QuestCreatorFocusField::CoordZ => Some(&self.coord_z),
            QuestCreatorFocusField::IndicatorOffFlag => Some(&self.indicator_off_flag),
            QuestCreatorFocusField::None => None,
        }
    }

    /// 获取指定输入字段的可变引用
    pub fn get_mut_focused_field(&mut self, field: QuestCreatorFocusField) -> Option<&mut String> {
        match field {
            QuestCreatorFocusField::Name => Some(&mut self.name),
            QuestCreatorFocusField::DisplayTitle => Some(&mut self.display_title),
            QuestCreatorFocusField::Orderer => Some(&mut self.orderer),
            QuestCreatorFocusField::Location => Some(&mut self.location),
            QuestCreatorFocusField::DescText => Some(&mut self.desc_text),
            QuestCreatorFocusField::FinishText => Some(&mut self.finish_text),
            QuestCreatorFocusField::ReadyFlag => Some(&mut self.ready_flag),
            QuestCreatorFocusField::ActivatedFlag => Some(&mut self.activated_flag),
            QuestCreatorFocusField::FinishFlag => Some(&mut self.finish_flag),
            QuestCreatorFocusField::DependencyFlag => Some(&mut self.dependency_flag),
            QuestCreatorFocusField::IndicatorActor => Some(&mut self.indicator_actor),
            QuestCreatorFocusField::CoordX => Some(&mut self.coord_x),
            QuestCreatorFocusField::CoordY => Some(&mut self.coord_y),
            QuestCreatorFocusField::CoordZ => Some(&mut self.coord_z),
            QuestCreatorFocusField::IndicatorOffFlag => Some(&mut self.indicator_off_flag),
            QuestCreatorFocusField::None => None,
        }
    }

    /// 获取当前激活输入框的可变引用
    pub fn get_mut_focused(&mut self) -> Option<&mut String> {
        self.get_mut_focused_field(self.focused_field)
    }

    /// 向当前聚焦的输入字段写入字符并执行联动逻辑
    pub fn insert_text(&mut self, clean: &str) -> bool {
        let field = self.focused_field;
        if let Some(target) = self.get_mut_focused() {
            target.push_str(clean);
            if field == QuestCreatorFocusField::Name {
                self.sync_flags_from_name();
            } else if field == QuestCreatorFocusField::ReadyFlag
                || field == QuestCreatorFocusField::ActivatedFlag
                || field == QuestCreatorFocusField::FinishFlag
            {
                self.custom_flags = true;
            }
            true
        } else {
            false
        }
    }
}

impl Default for QuestCreatorModalState {
    fn default() -> Self {
        Self {
            active_tab: QuestCreatorTab::General,
            name: "Quest_CustomMiniQuest".into(),
            quest_type: crate::model::quest::QuestType::Mini,
            orderer: "Npc_AncientDoctor_Robbie".into(),
            location: "Akkala".into(),
            display_title: "自定义挑战任务".into(),
            desc_text: "接受了小樱桃的委托，需要前往古代研究所完成测试。".into(),
            finish_text: "成功完成了委托，向小樱桃汇报了成果！".into(),
            custom_flags: false,
            ready_flag: "Quest_CustomMiniQuest_Ready".into(),
            activated_flag: "Quest_CustomMiniQuest_Activated".into(),
            finish_flag: "Quest_CustomMiniQuest_Finish".into(),
            dependency_flag: String::new(),
            enable_indicator: true,
            indicator_actor: "Npc_AncientDoctor_Robbie".into(),
            coord_x: "4518.0".into(),
            coord_y: "360.0".into(),
            coord_z: "-3158.0".into(),
            indicator_off_flag: String::new(),
            indicator_has_far: false,
            not_post_note: false,
            inject_titlebg: true,
            inject_bootup_flags: true,
            inject_ql_msbt: true,
            scaffold_graph_nodes: true,
            focused_field: QuestCreatorFocusField::None,
        }
    }
}

/// Git 提交输入框焦点
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitCommitFocus {
    #[default]
    None,
    Title,
    Description,
}

/// 资源管理器文件树右键上下文菜单激活状态
#[derive(Debug, Clone, PartialEq)]
pub struct FileTreeContextMenu {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub is_sarc: bool,
    pub is_inside_sarc: bool,
    pub position: (f32, f32),
}

/// 文件/封包条目重命名弹窗状态
#[derive(Debug, Clone, PartialEq)]
pub struct RenameModalState {
    pub target_path: String,
    pub old_name: String,
    pub new_name: String,
    pub is_inside_sarc: bool,
    pub is_dir: bool,
    pub focused: bool,
}

/// 文件/封包条目删除确认弹窗状态
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteConfirmModalState {
    pub target_path: String,
    pub name: String,
    pub is_inside_sarc: bool,
    pub is_dir: bool,
}
