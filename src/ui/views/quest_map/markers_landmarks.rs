use crate::ui::views::quest_map::markers_data::{
    HyruleMarkerKind, HyruleMarkerTemplate, HyruleMarkerTemplate as HyruleMarker,
    HyruleRegionLabel,
};

/// 海拉鲁主要区域大字标识常量表
pub const HYRULE_REGIONS: &[HyruleRegionLabel] = &[
    HyruleRegionLabel { name_zh: "中央海拉鲁", name_en: "Central Hyrule", position: (-200.0, 200.0) },
    HyruleRegionLabel { name_zh: "海布拉地区", name_en: "Hebra", position: (-2800.0, -2800.0) },
    HyruleRegionLabel { name_zh: "奥尔汀地区", name_en: "Eldin", position: (2200.0, -2600.0) },
    HyruleRegionLabel { name_zh: "阿卡莱地区", name_en: "Akkala", position: (3800.0, -2600.0) },
    HyruleRegionLabel { name_zh: "拉聂尔地区", name_en: "Lanayru", position: (3200.0, -200.0) },
    HyruleRegionLabel { name_zh: "哈特尔地区", name_en: "Necluda", position: (2600.0, 1800.0) },
    HyruleRegionLabel { name_zh: "费罗尼地区", name_en: "Faron", position: (1600.0, 3200.0) },
    HyruleRegionLabel { name_zh: "格鲁德地区", name_en: "Gerudo", position: (-3200.0, 2800.0) },
    HyruleRegionLabel { name_zh: "塔邦挞地区", name_en: "Tabantha", position: (-3400.0, -800.0) },
    HyruleRegionLabel { name_zh: "初始台地", name_en: "Great Plateau", position: (-800.0, 1800.0) },
];

/// 海拉鲁非神庙常驻核心地标列表 (包含希卡塔、驿站、四神兽、城镇与设施)
pub const HYRULE_LANDMARK_MARKERS: &[HyruleMarkerTemplate] = &[
    // --- 四神兽 (Divine Beasts) ---
    HyruleMarker { id: "RemainsWind", kind: HyruleMarkerKind::DivineBeast, name_zh: "神兽瓦·梅德", name_en: "Divine Beast Vah Medoh", sub_name_zh: Some("风之神兽"), sub_name_en: Some("Vah Medoh"), position: (-3614.1, -1862.1), altitude: 656.5, icon_char: "🦅" },
    HyruleMarker { id: "RemainsElectric", kind: HyruleMarkerKind::DivineBeast, name_zh: "神兽瓦·娜波力斯", name_en: "Divine Beast Vah Naboris", sub_name_zh: Some("雷之神兽"), sub_name_en: Some("Vah Naboris"), position: (-2111.1, 2627.2), altitude: 603.4, icon_char: "🐪" },
    HyruleMarker { id: "RemainsFire", kind: HyruleMarkerKind::DivineBeast, name_zh: "神兽瓦·鲁达尼亚", name_en: "Divine Beast Vah Rudania", sub_name_zh: Some("火之神兽"), sub_name_en: Some("Vah Rudania"), position: (2453.6, -2561.4), altitude: 1057.1, icon_char: "🦎" },
    HyruleMarker { id: "RemainsWater", kind: HyruleMarkerKind::DivineBeast, name_zh: "神兽瓦·露塔", name_en: "Divine Beast Vah Ruta", sub_name_zh: Some("水之神兽"), sub_name_en: Some("Vah Ruta"), position: (2971.1, 233.4), altitude: 502.2, icon_char: "🐘" },
    // --- 希卡之塔 (Towers) ---
    HyruleMarker { id: "Tower02", kind: HyruleMarkerKind::Tower, name_zh: "塔邦挞之塔", name_en: "Tabantha Tower", sub_name_zh: None, sub_name_en: None, position: (-3613.7, -990.2), altitude: 465.9, icon_char: "🗼" },
    HyruleMarker { id: "Tower03", kind: HyruleMarkerKind::Tower, name_zh: "格鲁德之塔", name_en: "Gerudo Tower", sub_name_zh: None, sub_name_en: None, position: (-3666.0, 1828.6), altitude: 490.2, icon_char: "🗼" },
    HyruleMarker { id: "Tower01", kind: HyruleMarkerKind::Tower, name_zh: "海布拉之塔", name_en: "Hebra Tower", sub_name_zh: None, sub_name_en: None, position: (-2173.0, -2034.0), altitude: 550.0, icon_char: "🗼" },
    HyruleMarker { id: "Tower04", kind: HyruleMarkerKind::Tower, name_zh: "荒野之塔", name_en: "Wasteland Tower", sub_name_zh: None, sub_name_en: None, position: (-2306.8, 2437.3), altitude: 551.0, icon_char: "🗼" },
    HyruleMarker { id: "Tower15", kind: HyruleMarkerKind::Tower, name_zh: "丘陵之塔", name_en: "Ridgeland Tower", sub_name_zh: None, sub_name_en: None, position: (-1755.3, -774.3), altitude: 349.4, icon_char: "🗼" },
    HyruleMarker { id: "Tower06", kind: HyruleMarkerKind::Tower, name_zh: "平原之塔", name_en: "Central Tower", sub_name_zh: None, sub_name_en: None, position: (-788.6, 442.0), altitude: 218.5, icon_char: "🗼" },
    HyruleMarker { id: "Tower07", kind: HyruleMarkerKind::Tower, name_zh: "初始之塔", name_en: "Great Plateau Tower", sub_name_zh: None, sub_name_en: None, position: (-560.0, 1694.9), altitude: 267.2, icon_char: "🗼" },
    HyruleMarker { id: "Tower09", kind: HyruleMarkerKind::Tower, name_zh: "湖之塔", name_en: "Lake Tower", sub_name_zh: None, sub_name_en: None, position: (-31.8, 2961.6), altitude: 301.3, icon_char: "🗼" },
    HyruleMarker { id: "Tower05", kind: HyruleMarkerKind::Tower, name_zh: "森林之塔", name_en: "Woodland Tower", sub_name_zh: None, sub_name_en: None, position: (883.9, -1605.7), altitude: 371.0, icon_char: "🗼" },
    HyruleMarker { id: "Tower08", kind: HyruleMarkerKind::Tower, name_zh: "双子山之塔", name_en: "Dueling Peaks Tower", sub_name_zh: None, sub_name_en: None, position: (1016.8, 1714.1), altitude: 204.8, icon_char: "🗼" },
    HyruleMarker { id: "Tower14", kind: HyruleMarkerKind::Tower, name_zh: "费罗尼之塔", name_en: "Faron Tower", sub_name_zh: None, sub_name_en: None, position: (1331.2, 3273.7), altitude: 290.8, icon_char: "🗼" },
    HyruleMarker { id: "Tower10", kind: HyruleMarkerKind::Tower, name_zh: "奥尔汀之塔", name_en: "Eldin Tower", sub_name_zh: None, sub_name_en: None, position: (2174.2, -1556.8), altitude: 529.5, icon_char: "🗼" },
    HyruleMarker { id: "Tower12", kind: HyruleMarkerKind::Tower, name_zh: "拉聂尔之塔", name_en: "Lanayru Tower", sub_name_zh: None, sub_name_en: None, position: (2258.0, -109.0), altitude: 331.7, icon_char: "🗼" },
    HyruleMarker { id: "Tower13", kind: HyruleMarkerKind::Tower, name_zh: "哈特诺之塔", name_en: "Hateno Tower", sub_name_zh: None, sub_name_en: None, position: (2735.5, 2133.5), altitude: 357.0, icon_char: "🗼" },
    HyruleMarker { id: "Tower11", kind: HyruleMarkerKind::Tower, name_zh: "阿卡莱之塔", name_en: "Akkala Tower", sub_name_zh: None, sub_name_en: None, position: (3308.0, -1500.1), altitude: 614.6, icon_char: "🗼" },
    // --- 海拉鲁驿站 (Stables) ---
    HyruleMarker { id: "RitoHatago", kind: HyruleMarkerKind::Stable, name_zh: "利特驿站", name_en: "Rito Stable", sub_name_zh: None, sub_name_en: None, position: (-3255.6, -1757.6), altitude: 268.5, icon_char: "🐎" },
    HyruleMarker { id: "TabantaBridgeHatago", kind: HyruleMarkerKind::Stable, name_zh: "塔邦挞大桥驿站", name_en: "Tabantha Bridge Stable", sub_name_zh: None, sub_name_en: None, position: (-2931.6, -547.6), altitude: 319.2, icon_char: "🐎" },
    HyruleMarker { id: "GerudoHatago", kind: HyruleMarkerKind::Stable, name_zh: "格鲁德峡谷驿站", name_en: "Gerudo Canyon Stable", sub_name_zh: None, sub_name_en: None, position: (-2803.8, 2225.8), altitude: 178.6, icon_char: "🐎" },
    HyruleMarker { id: "TabantaHatago", kind: HyruleMarkerKind::Stable, name_zh: "雪原驿站", name_en: "Snowfield Stable", sub_name_zh: None, sub_name_en: None, position: (-1654.9, -2572.2), altitude: 382.0, icon_char: "🐎" },
    HyruleMarker { id: "HyruleWestHatago", kind: HyruleMarkerKind::Stable, name_zh: "玛丽塔驿站", name_en: "Serenne Stable", sub_name_zh: None, sub_name_en: None, position: (-1559.4, -1799.1), altitude: 254.1, icon_char: "🐎" },
    HyruleMarker { id: "HyruleDepthHatago", kind: HyruleMarkerKind::Stable, name_zh: "平原外围的驿站", name_en: "Outskirt Stable", sub_name_zh: None, sub_name_en: None, position: (-1449.5, 1269.0), altitude: 181.4, icon_char: "🐎" },
    HyruleMarker { id: "NorthHatelHatago", kind: HyruleMarkerKind::Stable, name_zh: "湿地驿站", name_en: "Wetland Stable", sub_name_zh: None, sub_name_en: None, position: (888.1, 173.7), altitude: 176.0, icon_char: "🐎" },
    HyruleMarker { id: "RiverSideHatago", kind: HyruleMarkerKind::Stable, name_zh: "河畔驿站", name_en: "Riverside Stable", sub_name_zh: None, sub_name_en: None, position: (339.2, 1095.3), altitude: 145.3, icon_char: "🐎" },
    HyruleMarker { id: "FaronHatago002", kind: HyruleMarkerKind::Stable, name_zh: "高原驿站", name_en: "Highland Stable", sub_name_zh: None, sub_name_en: None, position: (529.6, 3450.9), altitude: 200.6, icon_char: "🐎" },
    HyruleMarker { id: "ForestHatago", kind: HyruleMarkerKind::Stable, name_zh: "森林驿站", name_en: "Woodland Stable", sub_name_zh: None, sub_name_en: None, position: (1065.7, -1141.6), altitude: 172.0, icon_char: "🐎" },
    HyruleMarker { id: "FaronHatago000", kind: HyruleMarkerKind::Stable, name_zh: "双子驿站", name_en: "Dueling Peaks Stable", sub_name_zh: None, sub_name_en: None, position: (1761.3, 1926.2), altitude: 158.8, icon_char: "🐎" },
    HyruleMarker { id: "FaronHatago001", kind: HyruleMarkerKind::Stable, name_zh: "湖畔驿站", name_en: "Lakeside Stable", sub_name_zh: None, sub_name_en: None, position: (1552.0, 3537.8), altitude: 210.6, icon_char: "🐎" },
    HyruleMarker { id: "DeathMountainHatago", kind: HyruleMarkerKind::Stable, name_zh: "山麓驿站", name_en: "Foothill Stable", sub_name_zh: None, sub_name_en: None, position: (2613.3, -1143.5), altitude: 296.2, icon_char: "🐎" },
    HyruleMarker { id: "TamurulHatago_02", kind: HyruleMarkerKind::Stable, name_zh: "南阿卡莱驿站", name_en: "South Akkala Stable", sub_name_zh: None, sub_name_en: None, position: (3149.8, -1692.6), altitude: 350.5, icon_char: "🐎" },
    HyruleMarker { id: "TamourHatago", kind: HyruleMarkerKind::Stable, name_zh: "东阿卡莱驿站", name_en: "East Akkala Stable", sub_name_zh: None, sub_name_en: None, position: (4227.8, -2747.8), altitude: 275.3, icon_char: "🐎" },
    // --- 主要城镇、村庄与设施 (Settlements & Facilities) ---
    HyruleMarker { id: "Rito", kind: HyruleMarkerKind::Village, name_zh: "利特村", name_en: "Rito Village", sub_name_zh: None, sub_name_en: None, position: (-3618.1, -1807.6), altitude: 531.7, icon_char: "🏡" },
    HyruleMarker { id: "Gerudo", kind: HyruleMarkerKind::Village, name_zh: "格鲁德小镇", name_en: "Gerudo Town", sub_name_zh: None, sub_name_en: None, position: (-3835.0, 2915.0), altitude: 149.6, icon_char: "🏡" },
    HyruleMarker { id: "StartPoint", kind: HyruleMarkerKind::Facility, name_zh: "复苏神庙", name_en: "Shrine of Resurrection", sub_name_zh: None, sub_name_en: None, position: (-1102.2, 1880.1), altitude: 276.3, icon_char: "✨" },
    HyruleMarker { id: "HyruleCastle", kind: HyruleMarkerKind::Facility, name_zh: "海拉鲁城堡", name_en: "Hyrule Castle", sub_name_zh: None, sub_name_en: None, position: (-254.0, -611.4), altitude: 169.8, icon_char: "🏰" },
    HyruleMarker { id: "Cokiri", kind: HyruleMarkerKind::Village, name_zh: "克洛格森林", name_en: "Korok Forest", sub_name_zh: None, sub_name_en: None, position: (428.0, -2137.5), altitude: 365.9, icon_char: "🏡" },
    HyruleMarker { id: "Goron", kind: HyruleMarkerKind::Village, name_zh: "鼓隆城", name_en: "Goron City", sub_name_zh: None, sub_name_en: None, position: (1685.2, -2467.5), altitude: 621.9, icon_char: "🏡" },
    HyruleMarker { id: "Kakariko", kind: HyruleMarkerKind::Village, name_zh: "卡卡利科村", name_en: "Kakariko Village", sub_name_zh: None, sub_name_en: None, position: (1806.0, 984.8), altitude: -18.4, icon_char: "🏡" },
    HyruleMarker { id: "UMiiVillage", kind: HyruleMarkerKind::Village, name_zh: "一始村", name_en: "Tarrey Town", sub_name_zh: None, sub_name_en: None, position: (3964.9, -1615.0), altitude: 278.9, icon_char: "🏡" },
    HyruleMarker { id: "WhiteZora", kind: HyruleMarkerKind::Village, name_zh: "卓拉领地", name_en: "Zora's Domain", sub_name_zh: None, sub_name_en: None, position: (3271.9, -401.5), altitude: 425.9, icon_char: "🏡" },
    HyruleMarker { id: "Hateno", kind: HyruleMarkerKind::Village, name_zh: "哈特诺村", name_en: "Hateno Village", sub_name_zh: None, sub_name_en: None, position: (3592.7, 2121.9), altitude: 362.6, icon_char: "🏡" },
    HyruleMarker { id: "HatenoLabo", kind: HyruleMarkerKind::Facility, name_zh: "哈特诺古代研究所", name_en: "Hateno Ancient Tech Lab", sub_name_zh: None, sub_name_en: None, position: (3778.6, 2125.7), altitude: 412.5, icon_char: "🔬" },
    HyruleMarker { id: "Taura", kind: HyruleMarkerKind::Village, name_zh: "沃托里村", name_en: "Lurelin Village", sub_name_zh: None, sub_name_en: None, position: (3009.5, 3477.9), altitude: 368.1, icon_char: "🏡" },
    HyruleMarker { id: "AncientLabo", kind: HyruleMarkerKind::Facility, name_zh: "阿卡莱古代研究所", name_en: "Akkala Ancient Tech Lab", sub_name_zh: None, sub_name_en: None, position: (4515.6, -3164.5), altitude: 452.6, icon_char: "🔬" },
];
