use gpui::SharedString;
use crate::ui::views::quest_map::markers_landmarks::{HYRULE_LANDMARK_MARKERS, HYRULE_REGIONS};
use crate::ui::views::quest_map::markers_shrines::HYRULE_SHRINE_MARKERS;

/// 海拉鲁地理标注类别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HyruleMarkerKind {
    /// 试炼神庙 (Dungeon)
    Shrine,
    /// 希卡之塔 (Tower)
    Tower,
    /// 海拉鲁驿站 (Stable / Hatago)
    Stable,
    /// 四神兽 (Divine Beast)
    DivineBeast,
    /// 城镇与村落 (Village)
    Village,
    /// 研究所、城堡等核心设施 (Facility)
    Facility,
}

impl HyruleMarkerKind {
    /// 获取该类别的唯一标识键
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Shrine => "shrine",
            Self::Tower => "tower",
            Self::Stable => "stable",
            Self::DivineBeast => "beast",
            Self::Village => "village",
            Self::Facility => "facility",
        }
    }
}

/// 静态模板定义（供内置预制表使用）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HyruleMarkerTemplate {
    pub id: &'static str,
    pub kind: HyruleMarkerKind,
    pub name_zh: &'static str,
    pub name_en: &'static str,
    pub sub_name_zh: Option<&'static str>,
    pub sub_name_en: Option<&'static str>,
    pub position: (f32, f32),
    pub altitude: f32,
    pub icon_char: &'static str,
}

/// 运行时海拉鲁地图标点核心数据结构（支持从项目与游戏路径动态提取与缓存）
#[derive(Debug, Clone, PartialEq)]
pub struct HyruleMarker {
    /// 标点唯一标识 ID (如 Dungeon001, RemainsWind, RitoHatago, 自定义神庙ID)
    pub id: SharedString,
    /// 标点分类
    pub kind: HyruleMarkerKind,
    /// 简体中文名
    pub name_zh: SharedString,
    /// 英文名
    pub name_en: SharedString,
    /// 中文副标题/试炼名 (如 "动与静的选择", "自定义神庙")
    pub sub_name_zh: Option<SharedString>,
    /// 英文副标题/试炼名
    pub sub_name_en: Option<SharedString>,
    /// 海拉鲁 2D 世界坐标 (X, Z)
    pub position: (f32, f32),
    /// Y 轴世界海拔高度
    pub altitude: f32,
    /// 徽章图标字符 (兜底用)
    pub icon_char: SharedString,
    /// 是否属于用户在当前项目中定义的自定义 Mod 标点
    pub is_custom: bool,
}

impl HyruleMarker {
    /// 获取当前语言下的主显示名称
    pub fn display_name(&self, is_zh: bool) -> &str {
        if is_zh { &self.name_zh } else { &self.name_en }
    }

    /// 获取当前语言下的副标题说明
    pub fn display_sub(&self, is_zh: bool) -> Option<&str> {
        if is_zh {
            self.sub_name_zh.as_deref()
        } else {
            self.sub_name_en.as_deref()
        }
    }
}

impl From<&HyruleMarkerTemplate> for HyruleMarker {
    fn from(t: &HyruleMarkerTemplate) -> Self {
        Self {
            id: t.id.into(),
            kind: t.kind,
            name_zh: t.name_zh.into(),
            name_en: t.name_en.into(),
            sub_name_zh: t.sub_name_zh.map(Into::into),
            sub_name_en: t.sub_name_en.map(Into::into),
            position: t.position,
            altitude: t.altitude,
            icon_char: t.icon_char.into(),
            is_custom: false,
        }
    }
}

/// 海拉鲁主要区域大字水印标识
#[derive(Debug, Clone, PartialEq)]
pub struct HyruleRegionLabel {
    pub name_zh: &'static str,
    pub name_en: &'static str,
    pub position: (f32, f32),
}

impl HyruleRegionLabel {
    pub fn display_name(&self, is_zh: bool) -> &'static str {
        if is_zh { self.name_zh } else { self.name_en }
    }
}

/// 获取全海拉鲁区域水印列表
pub fn get_hyrule_regions() -> &'static [HyruleRegionLabel] {
    HYRULE_REGIONS
}

/// 获取内置的官方地标与神庙模板列表
pub fn get_default_hyrule_markers() -> Vec<HyruleMarker> {
    HYRULE_LANDMARK_MARKERS
        .iter()
        .chain(HYRULE_SHRINE_MARKERS.iter())
        .map(HyruleMarker::from)
        .collect()
}
