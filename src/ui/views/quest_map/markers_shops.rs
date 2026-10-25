use gpui::SharedString;
use crate::ui::views::quest_map::markers_data::{HyruleMarker, HyruleMarkerKind};

/// 紧凑静态元组定义：(id, flag, icon_str, name_zh, name_en, sub_zh, sub_en, icon_char, (pos_x, pos_z), altitude)
type ShopTuple = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
    &'static str,
    (f32, f32),
    f32,
);

/// 全海拉鲁 37 家官方聚落商业设施基准注册表
static BOTW_SHOPS: &[ShopTuple] = &[
    // --- 一始村 (Tarrey Town) 4家店铺 ---
    ("UMiiVillageShopBougu", "Location_UMiiVillageShopBougu", "ShopBougu", "一始村·防具店", "Tarrey Town Armor Boutique", Some("佩达的防具店"), Some("Rhondson Armor"), "🛡️", (3958.9, -1636.3), 243.6),
    ("UMiiVillageShopJewel", "Location_UMiiVillageShopJewel", "ShopJewel", "一始村·宝石店", "Tarrey Town Jewelry", Some("普彭达的矿石店"), Some("Pugnant's Gems"), "💎", (3951.3, -1627.1), 235.7),
    ("UMiiVillageShopYorozu", "Location_UMiiVillageShopYorozu", "ShopYorozu", "一始村·杂货店", "Tarrey Town General Store", Some("轻飘飘一始村分店"), Some("Slippery Falcon Branch"), "🛍️", (3951.2, -1593.1), 235.4),
    ("UMiiVillageShopYadoya", "Location_UMiiVillageShopYadoya", "ShopYadoya", "一始村·旅馆", "Tarrey Town Inn", Some("卡颇达旅馆"), Some("Kapson's Inn"), "🛏️", (3999.6, -1596.0), 249.0),

    // --- 哈特诺村 (Hateno Village) 4家店铺 ---
    ("HatenoShopBougu", "Location_Hateno", "ShopBougu", "哈特诺村·防具店", "Ventest Clothing Boutique", Some("四色堂"), Some("Ventest Boutique"), "🛡️", (3357.0, 2127.9), 238.6),
    ("HatenoShopYorozu", "Location_Hateno", "ShopYorozu", "哈特诺村·杂货店", "East Wind General Store", Some("中之民杂货店"), Some("East Wind"), "🛍️", (3360.8, 2171.6), 238.6),
    ("HatenoShopColor", "Location_Hateno", "ShopColor", "哈特诺村·染坊", "Kochi Dye Shop", Some("珂奇染坊"), Some("Kochi Dye Shop"), "🎨", (3404.0, 2152.3), 238.6),
    ("HatenoShopYadoya", "Location_Hateno", "ShopYadoya", "哈特诺村·旅馆", "Great Ton Pu Inn", Some("托恩旅店"), Some("Great Ton Pu Inn"), "🛏️", (3473.2, 2111.2), 247.3),

    // --- 卡卡利科村 (Kakariko Village) 4家店铺 ---
    ("KakarikoShopBougu", "Location_Kakariko", "ShopBougu", "卡卡利科村·防具店", "Enchanted Armor Boutique", Some("传统服饰·布料工艺"), Some("Enchanted Boutique"), "🛡️", (1854.7, 981.3), 239.4),
    ("KakarikoShopYadoya", "Location_Kakariko", "ShopYadoya", "卡卡利科村·旅馆", "Shuteye Inn", Some("沉睡的竹林"), Some("Shuteye Inn"), "🛏️", (1835.6, 1021.8), 240.0),
    ("KakarikoShopYorozu_Produce", "Location_Kakariko", "ShopYorozu", "卡卡利科村·杂货店", "High Spirits Produce", Some("满腹屋"), Some("High Spirits Produce"), "🛍️", (1825.9, 1041.0), 235.7),
    ("KakarikoShopYorozu_Arrow", "Location_Kakariko", "ShopYorozu", "卡卡利科村·箭店", "The Curious Quiver", Some("飞燕箭店"), Some("The Curious Quiver"), "🛍️", (1794.5, 1029.3), 235.0),

    // --- 利特村 (Rito Village) 3家店铺 ---
    ("RitoShopBougu", "Location_Rito", "ShopBougu", "利特村·防具店", "Brazen Beak Armor Shop", Some("红孔雀"), Some("Brazen Beak"), "🛡️", (-3631.5, -1787.3), 292.2),
    ("RitoShopYorozu", "Location_Rito", "ShopYorozu", "利特村·杂货店", "The Slippery Falcon", Some("杂货店「轻飘飘」总店"), Some("The Slippery Falcon"), "🛍️", (-3644.6, -1820.5), 289.9),
    ("RitoShopYadoya", "Location_Rito", "ShopYadoya", "利特村·旅馆", "Swallow's Nest Inn", Some("燕巢旅馆"), Some("Swallow's Nest"), "🛏️", (-3612.1, -1852.5), 289.0),

    // --- 鼓隆城 (Goron City) 4家店铺 ---
    ("GoronShopBougu", "Location_Goron", "ShopBougu", "鼓隆城·防具店", "Goron Gusto Shop", Some("耐火屋"), Some("Goron Gusto Shop"), "🛡️", (1642.9, -2409.2), 490.2),
    ("GoronShopYadoya", "Location_Goron", "ShopYadoya", "鼓隆城·旅馆", "Rollin' Inn", Some("硬石旅馆"), Some("Rollin' Inn"), "🛏️", (1682.0, -2424.0), 489.2),
    ("GoronShopYorozu_Food", "Location_Goron", "ShopYorozu", "鼓隆城·食品店", "Goron General Store", Some("莫戈莫戈食品店"), Some("Goron General Store"), "🛍️", (1683.6, -2463.5), 490.8),
    ("GoronShopYorozu_BBQ", "Location_Goron", "ShopYorozu", "鼓隆城·烧烤店", "Protein Palace", Some("烧烤莫戈"), Some("Protein Palace"), "🛍️", (1667.1, -2468.1), 492.4),

    // --- 卓拉领地 (Zora's Domain) 2家店铺 ---
    ("WhiteZoraShopYadoya", "Location_WhiteZora", "ShopYadoya", "卓拉领地·旅馆", "Seabed Inn", Some("鱼居宿"), Some("Seabed Inn"), "🛏️", (3296.2, -508.5), 245.9),
    ("WhiteZoraShopYorozu", "Location_WhiteZora", "ShopYorozu", "卓拉领地·杂货店", "Coral Reef General Store", Some("珊瑚礁"), Some("Coral Reef"), "🛍️", (3322.9, -490.7), 245.5),

    // --- 沃托里村 (Lurelin Village) 2家店铺 ---
    ("TauraShopYadoya", "Location_Taura", "ShopYadoya", "沃托里村·旅馆", "Fishing Resort Inn", Some("海边之宿"), Some("Fishing Resort"), "🛏️", (2944.6, 3437.0), 134.7),
    ("TauraShopYorozu", "Location_Taura", "ShopYorozu", "沃托里村·杂货店", "Lurelin General Store", Some("渔获商会"), Some("Lurelin General Store"), "🛍️", (2959.4, 3493.9), 106.8),

    // --- 克洛格森林 (Korok Forest) 2家店铺 ---
    ("CokiriShopYadoya", "Location_Cokiri", "ShopYadoya", "克洛格森林·旅馆", "Great Deku Tree Navel Inn", Some("德库树之脐旅馆"), Some("Deku Tree Inn"), "🛏️", (406.7, -2177.9), 259.6),
    ("CokiriShopYorozu", "Location_Cokiri", "ShopYorozu", "克洛格森林·杂货店", "General Shoppe", Some("克洛格杂货店"), Some("General Shoppe"), "🛍️", (434.3, -2181.2), 258.4),

    // --- 卡拉卡拉集市 (Kara Kara Bazaar) 4家店铺 ---
    ("OasisShopYadoya", "Location_Oasis", "ShopYadoya", "卡拉卡拉集市·旅馆", "Kara Kara Bazaar Inn", Some("绿洲驿站旅店"), Some("Oasis Inn"), "🛏️", (-3267.8, 2610.1), 200.0),
    ("OasisShopYorozu_General", "Location_Oasis", "ShopYorozu", "卡拉卡拉集市·杂货摊", "Kara Kara Bazaar Stall", Some("生活杂货摊"), Some("General Stall"), "🛍️", (-3277.2, 2564.8), 200.0),
    ("OasisShopYorozu_Grocery", "Location_Oasis", "ShopYorozu", "卡拉卡拉集市·食材摊", "Kara Kara Grocery Stall", Some("热带水果与烤肉"), Some("Food Market"), "🛍️", (-3240.7, 2527.2), 200.0),
    ("OasisShopYorozu_Water", "Location_Oasis", "ShopYorozu", "卡拉卡拉集市·特产摊", "Kara Kara Bazaar Specialty", Some("集市特产摊"), Some("Specialty Stall"), "🛍️", (-3261.8, 2603.5), 200.0),

    // --- 格鲁德小镇 (Gerudo Town) 8家店铺 ---
    ("GerudoShopBougu", "Location_Gerudo", "ShopBougu", "格鲁德小镇·防具店", "Fashion Passion Boutique", Some("热沙时尚"), Some("Fashion Passion"), "🛡️", (-3806.3, 2906.1), 200.0),
    ("GerudoShopJewel", "Location_Gerudo", "ShopJewel", "格鲁德小镇·宝石店", "Starlight Memories Jewelry", Some("星光首饰店"), Some("Starlight Memories"), "💎", (-3836.8, 2877.1), 200.0),
    ("GerudoShopYadoya", "Location_Gerudo", "ShopYadoya", "格鲁德小镇·旅馆", "Hotel Oasis", Some("绿洲之宿"), Some("Hotel Oasis"), "🛏️", (-3850.3, 2890.0), 200.0),
    ("GerudoShopYorozu_Meat", "Location_Gerudo", "ShopYorozu", "格鲁德小镇·肉店", "Gerudo Meat Market", Some("鲜肉专卖店"), Some("Fresh Meat"), "🛍️", (-3857.5, 2913.2), 200.0),
    ("GerudoShopYorozu_Mushroom", "Location_Gerudo", "ShopYorozu", "格鲁德小镇·蘑菇店", "Gerudo Mushroom Stall", Some("沙漠蘑菇摊"), Some("Mushroom Stall"), "🛍️", (-3848.7, 2903.6), 200.0),
    ("GerudoShopYorozu_General", "Location_Gerudo", "ShopYorozu", "格鲁德小镇·杂货店", "Gerudo General Store", Some("全品类杂货店"), Some("General Goods"), "🛍️", (-3820.0, 2925.9), 200.0),
    ("GerudoShopYorozu_Arrow", "Location_Gerudo", "ShopYorozu", "格鲁德小镇·箭店", "Gerudo Arrow Stall", Some("格鲁德箭摊"), Some("Arrow Stall"), "🛍️", (-3821.9, 2936.3), 200.0),
    ("GerudoShopYorozu_Secret", "Location_Gerudo", "ShopYorozu", "格鲁德秘密俱乐部", "Gerudo Secret Club", Some("GSC 会员专享店"), Some("Gerudo Secret Club"), "🛡️", (-3823.0, 2878.6), 200.0),
];

/// 尝试将 Static.smubin 中的标点条目解析为专有店铺标点
pub fn resolve_shop_marker(
    flag: &str,
    icon_str: &str,
    pos: (f32, f32),
    altitude: f32,
    is_custom: bool,
) -> Option<HyruleMarker> {
    if !icon_str.starts_with("Shop") && !flag.contains("Shop") {
        return None;
    }

    // 1. 优先通过 SaveFlag 或坐标精确匹配 37 家官方店铺
    for s in BOTW_SHOPS {
        let flag_match = !flag.is_empty() && (s.1 == flag || s.0 == flag);
        let dist = ((s.8.0 - pos.0).powi(2) + (s.8.1 - pos.1).powi(2)).sqrt();
        let pos_match = dist < 25.0 && (s.2 == icon_str || icon_str.is_empty());

        if (flag_match && s.2 == icon_str) || pos_match {
            return Some(HyruleMarker {
                id: SharedString::from(s.0.to_string()),
                kind: HyruleMarkerKind::Shop,
                name_zh: SharedString::from(s.3.to_string()),
                name_en: SharedString::from(s.4.to_string()),
                sub_name_zh: s.5.map(SharedString::from),
                sub_name_en: s.6.map(SharedString::from),
                position: pos,
                altitude,
                icon_char: SharedString::from(s.7.to_string()),
                is_custom,
            });
        }
    }

    // 2. 泛型兜底推导（如自创 Mod 店铺）
    let (type_zh, type_en, icon_char) = match icon_str {
        "ShopBougu" => ("防具店", "Armor Boutique", "🛡️"),
        "ShopJewel" => ("宝石店", "Jewelry Shop", "💎"),
        "ShopYadoya" => ("旅馆", "Inn", "🛏️"),
        "ShopColor" => ("染坊", "Dye Shop", "🎨"),
        _ => ("杂货店", "General Store", "🛍️"),
    };

    let village_raw = flag.strip_prefix("Location_").unwrap_or(flag);
    let village_clean = village_raw
        .replace("ShopBougu", "")
        .replace("ShopJewel", "")
        .replace("ShopYorozu", "")
        .replace("ShopYadoya", "")
        .replace("ShopColor", "")
        .replace("Shop", "");

    let name_zh = format!("{}·{}", village_clean, type_zh);
    let name_en = format!("{} {}", village_clean, type_en);
    let unique_id = format!("{}_{}", village_raw, icon_str);

    Some(HyruleMarker {
        id: SharedString::from(unique_id),
        kind: HyruleMarkerKind::Shop,
        name_zh: SharedString::from(name_zh),
        name_en: SharedString::from(name_en),
        sub_name_zh: Some(SharedString::from(type_zh.to_string())),
        sub_name_en: Some(SharedString::from(type_en.to_string())),
        position: pos,
        altitude,
        icon_char: SharedString::from(icon_char.to_string()),
        is_custom,
    })
}
