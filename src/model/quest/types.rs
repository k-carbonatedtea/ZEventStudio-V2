use serde::{Deserialize, Serialize};

/// 任务类型分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QuestType {
    #[default]
    Mini, // 迷你挑战 (Mini)
    Sub,  // 普通挑战/神庙挑战 (Sub)
    Main, // 主线剧情挑战 (Main)
}

impl QuestType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mini => "Mini",
            Self::Sub => "Sub",
            Self::Main => "Main",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Main" => Self::Main,
            "Sub" => Self::Sub,
            _ => Self::Mini,
        }
    }

    pub fn label_key(&self) -> &'static str {
        match self {
            Self::Mini => "quests.type_mini",
            Self::Sub => "quests.type_sub",
            Self::Main => "quests.type_main",
        }
    }
}

/// 地图标点指示器模型 (Map Indicator / Target Pin)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestIndicator {
    pub actor_name: Option<String>,
    pub instance_name: Option<String>,
    pub position: (f32, f32, f32), // (X, Y, Z) 海拉鲁世界坐标
    pub off_flag: Option<String>,
    pub has_far: bool,
}

/// 任务单阶段定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestStep {
    pub name: String,
    pub message_name: String,
    pub next_flag: String,
    pub update_step: bool,
    pub attention_off: bool,
    pub indicators: Vec<QuestIndicator>,
}

impl Default for QuestStep {
    fn default() -> Self {
        Self {
            name: "Ready".to_string(),
            message_name: "Desc".to_string(),
            next_flag: String::new(),
            update_step: true,
            attention_off: false,
            indicators: Vec::new(),
        }
    }
}

/// 任务完整定义模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestDefinition {
    pub name: String,
    pub quest_type: QuestType,
    pub orderer: String,
    pub location: String,
    pub not_post_note: bool,
    pub dependency_flag: Option<String>,
    pub steps: Vec<QuestStep>,
    // 关联的本地化文本预览 (来自 QL_<Quest>.msbt)
    pub display_title: Option<String>,
    pub desc_text: Option<String>,
    pub finish_text: Option<String>,
}

impl QuestDefinition {
    /// 计算该任务所有阶段中包含的地图指示标点总数
    pub fn total_indicators_count(&self) -> usize {
        self.steps.iter().map(|s| s.indicators.len()).sum()
    }
}

impl Default for QuestDefinition {
    fn default() -> Self {
        Self {
            name: String::new(),
            quest_type: QuestType::Mini,
            orderer: String::new(),
            location: "Unknown".to_string(),
            not_post_note: false,
            dependency_flag: None,
            steps: Vec::new(),
            display_title: None,
            desc_text: None,
            finish_text: None,
        }
    }
}

/// 新建任务脚手架向导配置
#[derive(Debug, Clone, PartialEq)]
pub struct QuestScaffoldConfig {
    pub name: String,
    pub quest_type: QuestType,
    pub orderer: String,
    pub location: String,
    pub display_title: String,
    pub desc_text: String,
    pub finish_text: String,
    // Flag 标志位设定
    pub ready_flag: String,
    pub activated_flag: String,
    pub finish_flag: String,
    pub dependency_flag: String,
    pub extra_flags: Vec<String>,
    // 地图指示器与坐标设定
    pub enable_indicator: bool,
    pub indicator_actor: String,
    pub indicator_pos: (f32, f32, f32),
    pub indicator_off_flag: String,
    pub indicator_has_far: bool,
    // 其他任务元数据
    pub not_post_note: bool,
    // 自动化模块勾选项
    pub inject_titlebg: bool,
    pub inject_bootup_flags: bool,
    pub inject_ql_msbt: bool,
    pub scaffold_graph_nodes: bool,
    pub inject_npc_choice: bool,
}

impl Default for QuestScaffoldConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            quest_type: QuestType::Mini,
            orderer: String::new(),
            location: "Unknown".to_string(),
            display_title: String::new(),
            desc_text: String::new(),
            finish_text: String::new(),
            ready_flag: String::new(),
            activated_flag: String::new(),
            finish_flag: String::new(),
            dependency_flag: String::new(),
            extra_flags: Vec::new(),
            enable_indicator: false,
            indicator_actor: String::new(),
            indicator_pos: (0.0, 0.0, 0.0),
            indicator_off_flag: String::new(),
            indicator_has_far: false,
            not_post_note: false,
            inject_titlebg: true,
            inject_bootup_flags: true,
            inject_ql_msbt: true,
            scaffold_graph_nodes: true,
            inject_npc_choice: true,
        }
    }
}

/// 计算 BotW 标准 32 位带符号 CRC32 哈希值
pub fn compute_botw_hash(name: &str) -> i32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in name.as_bytes() {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    (!crc) as i32
}

/// 在 SARC 封包中查找匹配指定相对路径的文件
pub fn find_sarc_file<'a>(sarc: &'a roead::sarc::Sarc, path: &str) -> Option<roead::sarc::File<'a>> {
    let clean = path.replace('\\', "/");
    let clean_trim = clean.trim_start_matches('/');
    sarc.files().find(|f| {
        if let Some(name) = f.name() {
            let n_clean = name.replace('\\', "/");
            let n_trim = n_clean.trim_start_matches('/');
            n_clean == clean || n_trim == clean_trim || n_clean.ends_with(&clean)
        } else {
            false
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_botw_crc32_hash() {
        assert_eq!(compute_botw_hash("Test_Ready"), -1378414736);
        assert_eq!(compute_botw_hash("Test_Activated"), -122606554);
        assert_eq!(compute_botw_hash("Test_Finish"), 1795383569);
    }

    #[test]
    fn test_quest_type_mapping() {
        assert_eq!(QuestType::from_str("Main"), QuestType::Main);
        assert_eq!(QuestType::from_str("Sub"), QuestType::Sub);
        assert_eq!(QuestType::from_str("Mini"), QuestType::Mini);
        assert_eq!(QuestType::Mini.as_str(), "Mini");
        assert_eq!(QuestType::Main.label_key(), "quests.type_main");
    }
}

