use crate::model::quest::QuestType;

/// 任务向导分段标签页分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuestCreatorTab {
    #[default]
    General,     // 基础信息 (分类、ID、标题、委托人、区域)
    Flags,       // Flag 设定 (Ready, Activated, Finish, Dependency, ExtraFlags)
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
    ExtraFlag(usize), // 自定义扩展 Flag (按索引对焦)
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
    pub quest_type: QuestType,
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
    pub extra_flags: Vec<String>, // 自定义扩展阶段 / 状态 Flag 列表
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
    pub is_edit_mode: bool, // 是否为编辑现有任务模式
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
        self.extra_flags.clear();
    }

    /// 从现有任务定义生成向导状态（用于编辑现有任务）
    pub fn from_quest(quest: &crate::model::quest::QuestDefinition) -> Self {
        let act_flag = quest.steps.iter().find(|s| s.name == "Ready")
            .map(|s| s.next_flag.clone())
            .unwrap_or_else(|| format!("{}_Activated", quest.name));
        let fin_flag = quest.steps.iter().find(|s| s.name == "Finish")
            .map(|s| s.next_flag.clone())
            .unwrap_or_else(|| format!("{}_Finish", quest.name));
        let extra_flags: Vec<String> = quest.steps.iter()
            .filter(|s| s.name != "Ready" && s.name != "Finish")
            .map(|s| s.next_flag.clone())
            .collect();

        let first_ind = quest.steps.iter().flat_map(|s| &s.indicators).next();
        let (enable_ind, ind_actor, cx, cy, cz, off_flag, has_far) = if let Some(ind) = first_ind {
            (
                true,
                ind.actor_name.clone().unwrap_or_default(),
                format!("{:.1}", ind.position.0),
                format!("{:.1}", ind.position.1),
                format!("{:.1}", ind.position.2),
                ind.off_flag.clone().unwrap_or_default(),
                ind.has_far,
            )
        } else {
            (false, String::new(), "0.0".into(), "0.0".into(), "0.0".into(), String::new(), false)
        };

        Self {
            active_tab: QuestCreatorTab::General,
            name: quest.name.clone(),
            quest_type: quest.quest_type,
            orderer: quest.orderer.clone(),
            location: quest.location.clone(),
            display_title: quest.display_title.clone().unwrap_or_default(),
            desc_text: quest.desc_text.clone().unwrap_or_default(),
            finish_text: quest.finish_text.clone().unwrap_or_default(),
            custom_flags: true,
            ready_flag: format!("{}_Ready", quest.name),
            activated_flag: act_flag,
            finish_flag: fin_flag,
            dependency_flag: quest.dependency_flag.clone().unwrap_or_default(),
            extra_flags,
            enable_indicator: enable_ind,
            indicator_actor: ind_actor,
            coord_x: cx,
            coord_y: cy,
            coord_z: cz,
            indicator_off_flag: off_flag,
            indicator_has_far: has_far,
            not_post_note: quest.not_post_note,
            inject_titlebg: true,
            inject_bootup_flags: true,
            inject_ql_msbt: true,
            scaffold_graph_nodes: false,
            focused_field: QuestCreatorFocusField::None,
            is_edit_mode: true,
        }
    }

    /// 添加一个新的自定义扩展阶段 Flag
    pub fn add_extra_flag(&mut self) {
        let clean = self.name.trim();
        let next_idx = self.extra_flags.len() + 1;
        let new_flag = format!("{}_Step{}", clean, next_idx);
        let idx = self.extra_flags.len();
        self.extra_flags.push(new_flag);
        self.focused_field = QuestCreatorFocusField::ExtraFlag(idx);
    }

    /// 移除指定索引处的扩展 Flag
    pub fn remove_extra_flag(&mut self, idx: usize) {
        if idx < self.extra_flags.len() {
            self.extra_flags.remove(idx);
            if self.focused_field == QuestCreatorFocusField::ExtraFlag(idx) {
                self.focused_field = QuestCreatorFocusField::None;
            }
        }
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
            QuestCreatorFocusField::ExtraFlag(idx) => self.extra_flags.get(idx).map(|s| s.as_str()),
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
            QuestCreatorFocusField::ExtraFlag(idx) => self.extra_flags.get_mut(idx),
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
}

impl Default for QuestCreatorModalState {
    fn default() -> Self {
        Self {
            active_tab: QuestCreatorTab::General,
            name: "Quest_CustomMiniQuest".into(),
            quest_type: QuestType::Mini,
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
            extra_flags: Vec::new(),
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
            is_edit_mode: false,
        }
    }
}

/// 任务原始数据查看格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RawDataFormat {
    #[default]
    Json,      // 结构化 JSON
    BymlYaml,  // BYML / YAML 拓扑规范
    Msbt,      // 冒险笔记 MSBT 文本词条
    FlagsHash, // 标志位与 CRC32 哈希 / 存档槽位
}

/// 任务原始数据查看模态框状态
#[derive(Debug, Clone, PartialEq)]
pub struct QuestRawDataModalState {
    pub quest: crate::model::quest::QuestDefinition,
    pub active_format: RawDataFormat,
}

impl QuestRawDataModalState {
    pub fn new(quest: crate::model::quest::QuestDefinition) -> Self {
        Self {
            quest,
            active_format: RawDataFormat::Json,
        }
    }
}
