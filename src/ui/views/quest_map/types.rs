//! 海拉鲁 2D 地图数学投影与分区网格算法工具

/// 将 BotW 世界坐标 (X, Z) 投影至当前视口屏幕像素坐标
#[inline]
pub fn world_to_screen(
    wx: f32,
    wz: f32,
    center: (f32, f32),
    pan: (f32, f32),
    zoom: f32,
) -> (f32, f32) {
    (
        center.0 + pan.0 + wx * zoom,
        center.1 + pan.1 + wz * zoom,
    )
}

/// 将视口屏幕像素坐标逆向投影至 BotW 世界坐标 (X, Z)
#[inline]
pub fn screen_to_world(
    sx: f32,
    sy: f32,
    center: (f32, f32),
    pan: (f32, f32),
    zoom: f32,
) -> (f32, f32) {
    if zoom.abs() < 0.0001 {
        return (0.0, 0.0);
    }
    (
        (sx - center.0 - pan.0) / zoom,
        (sy - center.1 - pan.1) / zoom,
    )
}

/// 根据世界坐标推断所属的标准 BotW 地图分区名称 (如 E-4, D-7, G-2)
pub fn get_section_for_coord(wx: f32, wz: f32) -> String {
    let col_idx = ((wx + 6000.0) / 1000.0).floor() as i32;
    let row_idx = ((wz + 5000.0) / 1000.0).floor() as i32;

    let col_char = match col_idx {
        0 => 'Z',
        1 => 'A',
        2 => 'B',
        3 => 'C',
        4 => 'D',
        5 => 'E',
        6 => 'F',
        7 => 'G',
        8 => 'H',
        9 => 'I',
        10 => 'J',
        11 => 'K',
        _ => return "Out of Bounds".to_string(),
    };

    if !(0..=9).contains(&row_idx) {
        return "Out of Bounds".to_string();
    }

    format!("{}-{}", col_char, row_idx)
}

/// 根据世界坐标推断归属的主要区域地名
pub fn get_region_name_for_coord(wx: f32, wz: f32, is_zh: bool) -> &'static str {
    if (-1200.0..=-400.0).contains(&wx) && (1400.0..=2200.0).contains(&wz) {
        return if is_zh { "初始台地 (Great Plateau)" } else { "Great Plateau" };
    }
    if (-800.0..=500.0).contains(&wx) && (-1200.0..=0.0).contains(&wz) {
        return if is_zh { "海拉鲁平原与城堡 (Central Hyrule)" } else { "Central Hyrule" };
    }
    if (1400.0..=3200.0).contains(&wx) && (-3200.0..=-1800.0).contains(&wz) {
        return if is_zh { "奥尔汀与死亡之山 (Death Mountain)" } else { "Death Mountain" };
    }
    if (1200.0..=2400.0).contains(&wx) && (600.0..=1600.0).contains(&wz) {
        return if is_zh { "卡卡利科村 (Kakariko)" } else { "Kakariko Village" };
    }
    if (2800.0..=4000.0).contains(&wx) && (1600.0..=2600.0).contains(&wz) {
        return if is_zh { "哈特诺村 (Hateno)" } else { "Hateno Village" };
    }
    if (3800.0..=4800.0).contains(&wx) && (-3600.0..=-2400.0).contains(&wz) {
        return if is_zh { "阿卡莱古代研究所 (Akkala)" } else { "Akkala Highlands" };
    }
    if (-4400.0..=-2800.0).contains(&wx) && (-2400.0..=-1200.0).contains(&wz) {
        return if is_zh { "利特村与塔邦挞 (Rito Village)" } else { "Rito Village / Tabantha" };
    }
    if (-4600.0..=-2800.0).contains(&wx) && (2200.0..=3800.0).contains(&wz) {
        return if is_zh { "格鲁德小镇与荒漠 (Gerudo Desert)" } else { "Gerudo Desert" };
    }
    if (-600.0..=400.0).contains(&wx) && (2200.0..=3200.0).contains(&wz) {
        return if is_zh { "海利亚湖 (Lake Hylia)" } else { "Lake Hylia" };
    }
    if (2600.0..=4000.0).contains(&wx) && (-800.0..=200.0).contains(&wz) {
        return if is_zh { "卓拉领地 (Zora's Domain)" } else { "Zora's Domain" };
    }
    if (0.0..=800.0).contains(&wx) && (-2500.0..=-1600.0).contains(&wz) {
        return if is_zh { "克洛格森林 (Korok Forest)" } else { "Korok Forest" };
    }
    if (800.0..=1800.0).contains(&wx) && (1400.0..=2400.0).contains(&wz) {
        return if is_zh { "双子山 (Dueling Peaks)" } else { "Dueling Peaks" };
    }
    if (-3400.0..=-1800.0).contains(&wx) && (-3800.0..=-2600.0).contains(&wz) {
        return if is_zh { "海布拉之巅 (Hebra Mountains)" } else { "Hebra Peak" };
    }
    if (800.0..=2400.0).contains(&wx) && (2800.0..=4000.0).contains(&wz) {
        return if is_zh { "费罗尼草原 (Faron)" } else { "Faron Grasslands" };
    }
    if is_zh { "海拉鲁荒野 (Hyrule Frontier)" } else { "Hyrule Wilderness" }
}
