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

/// 海拉鲁地理地图标点核心数据结构
#[derive(Debug, Clone, PartialEq)]
pub struct HyruleMarker {
    /// 标点唯一标识 ID (如 Dungeon001, RemainsWind, RitoHatago)
    pub id: &'static str,
    /// 标点分类
    pub kind: HyruleMarkerKind,
    /// 官方简体中文名
    pub name_zh: &'static str,
    /// 官方美版英文名
    pub name_en: &'static str,
    /// 中文副标题/试炼名 (如 "动与静的选择", "风之神兽")
    pub sub_name_zh: Option<&'static str>,
    /// 英文副标题/试炼名
    pub sub_name_en: Option<&'static str>,
    /// 海拉鲁 2D 世界坐标 (X, Z)
    pub position: (f32, f32),
    /// Y 轴世界海拔高度
    pub altitude: f32,
    /// 徽章图标符号 (如 ⛩️, 🗼, 🐎, 🦅 等)
    pub icon_char: &'static str,
}

impl HyruleMarker {
    /// 获取当前语言下的主显示名称
    pub fn display_name(&self, is_zh: bool) -> &'static str {
        if is_zh { self.name_zh } else { self.name_en }
    }

    /// 获取当前语言下的副标题说明
    pub fn display_sub(&self, is_zh: bool) -> Option<&'static str> {
        if is_zh { self.sub_name_zh } else { self.sub_name_en }
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

/// 获取所有海拉鲁地标、神兽、驿站、城镇与神庙标点迭代器
pub fn get_all_hyrule_markers() -> impl Iterator<Item = &'static HyruleMarker> {
    HYRULE_LANDMARK_MARKERS.iter().chain(HYRULE_SHRINE_MARKERS.iter())
}

/// 获取全海拉鲁区域水印列表
pub fn get_hyrule_regions() -> &'static [HyruleRegionLabel] {
    HYRULE_REGIONS
}
