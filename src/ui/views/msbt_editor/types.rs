use serde::{Deserialize, Serialize};

/// 塞尔达气泡对话框形态类型枚举 (参考 Bubble-Wrap 标准)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BubbleType {
    /// 标准 NPC 角色对白气泡 (3行, 经典圆角胶囊, 宽度 566px)
    #[default]
    Dialogue,
    /// 木牌与路标告示牌 (3行, 矩形微圆角 6px, 宽度 566px)
    Signboard,
    /// 道具与装备说明卡片 (4行, 204 字符上限限制, 宽度 508px)
    Item,
    /// 海拉鲁图鉴条目卡片 (9行, 宽度 508px)
    Compendium,
    /// 任务日志与挑战卡片 (11行, 宽度 508px)
    Quest,
    /// 屏幕加载与冒险提示 (3行, 宽矩形 634px)
    Tip,
}

impl BubbleType {
    /// 获取形态中文或展示标签
    #[allow(dead_code)]
    pub fn display_label(&self) -> &'static str {
        match self {
            BubbleType::Dialogue => "对话气泡 (Dialogue)",
            BubbleType::Signboard => "木牌告示 (Signboard)",
            BubbleType::Item => "道具说明 (Item)",
            BubbleType::Compendium => "图鉴条目 (Compendium)",
            BubbleType::Quest => "任务日志 (Quest)",
            BubbleType::Tip => "冒险提示 (Tip)",
        }
    }

    /// 获取建议行数上限
    pub fn line_limit(&self) -> usize {
        match self {
            BubbleType::Dialogue => 3,
            BubbleType::Signboard => 3,
            BubbleType::Item => 4,
            BubbleType::Compendium => 9,
            BubbleType::Quest => 11,
            BubbleType::Tip => 3,
        }
    }

    /// 获取建议字符上限 (如道具说明限制 204 字)
    pub fn char_limit(&self) -> Option<usize> {
        match self {
            BubbleType::Item => Some(204),
            _ => None,
        }
    }

    /// 获取所有支持的气泡类型列表
    pub fn all() -> &'static [BubbleType] {
        &[
            BubbleType::Dialogue,
            BubbleType::Signboard,
            BubbleType::Item,
            BubbleType::Compendium,
            BubbleType::Quest,
            BubbleType::Tip,
        ]
    }
}

/// 塞尔达对白文本富文本高亮颜色 (参考 https://zeldamods.org/wiki/Help:Text_modding)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BubbleColor {
    #[default]
    Default,
    Grey,
    Blue,
    Red,
    Orange,
    Green,
    Yellow,
}

impl BubbleColor {
    /// 获取在游戏中对应的真实 RGB 高亮颜色
    pub fn rgb_color(&self) -> gpui::Rgba {
        match self {
            BubbleColor::Default => gpui::rgb(0xF5F5EE),
            BubbleColor::Grey => gpui::rgb(0xAFAFAA),
            BubbleColor::Blue => gpui::rgb(0x38DFD8),
            BubbleColor::Red => gpui::rgb(0xF84858),
            BubbleColor::Orange => gpui::rgb(0xFB923C),
            BubbleColor::Green => gpui::rgb(0x34D399),
            BubbleColor::Yellow => gpui::rgb(0xFACC15),
        }
    }

    /// 对应的标签名
    #[allow(dead_code)]
    pub fn tag_name(&self) -> &'static str {
        match self {
            BubbleColor::Default => "default",
            BubbleColor::Grey => "grey",
            BubbleColor::Blue => "blue",
            BubbleColor::Red => "red",
            BubbleColor::Orange => "orange",
            BubbleColor::Green => "green",
            BubbleColor::Yellow => "yellow",
        }
    }
}

/// 塞尔达对白文本尺寸级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BubbleSize {
    Small,
    #[default]
    Normal,
    Large,
}

impl BubbleSize {
    /// 对应的 GPUI 渲染像素字号
    pub fn font_size_px(&self) -> f32 {
        match self {
            BubbleSize::Small => 14.5,
            BubbleSize::Normal => 18.0,
            BubbleSize::Large => 22.5,
        }
    }

    /// 对应的百分比数值 (80%, 100%, 125%)
    #[allow(dead_code)]
    pub fn percent(&self) -> u32 {
        match self {
            BubbleSize::Small => 80,
            BubbleSize::Normal => 100,
            BubbleSize::Large => 125,
        }
    }
}

/// 气泡内单一渲染片段 (富文本颜色、尺寸或控制节点)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BubbleSpan {
    pub text: String,
    pub color: BubbleColor,
    pub size: BubbleSize,
    pub is_pause: bool,
    pub pause_label: Option<String>,
    pub is_chip: bool,
    pub chip_label: Option<String>,
}

/// 单个 MSBT 消息条目数据模型
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsbtEntry {
    pub key: String,
    pub raw_text: String,
    pub clean_text: String,
    pub spans: Vec<BubbleSpan>,
    pub animation: String,
    pub sound: String,
    pub line_count: usize,
    pub char_count: usize,
}

/// 完整的 MSBT 结构化文档模型
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsbtDocument {
    pub group_count: u32,
    pub atr1_unknown: Option<u32>,
    pub entries: Vec<MsbtEntry>,
}
