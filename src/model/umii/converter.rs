#![allow(clippy::field_reassign_with_default)]
//! 外部 Mii 格式 (Wii, 3DS, Wii U, Switch, Studio) 解析与转换至 UMii

use super::data::UmiiData;

/// 肤色映射表 (Mii 原生肤色 -> BotW UMii 肤色)
pub fn map_skin_tone(face_color: u8) -> i32 {
    match face_color {
        0 => 0,  // Yellow Light
        1 => 1,  // Yellow Dark
        2 => 10, // Brown Light
        3 => 3,  // Pink Fair
        4 => 9,  // Brown Fair
        5 => 5,  // Black Dark
        6 => 6,  // Pink Light
        7 => 2,  // Yellow Fair / White Fair
        8 => 4,  // Black Fair
        _ => 0,
    }
}

/// 眼睛原生基准角度表 (EyeRotBase)
pub fn get_eye_rot_base(eye_type: i32) -> f32 {
    match eye_type {
        0x02 | 0x11 | 0x01 | 0x10 | 0x0f | 0x1b | 0x14 | 0x0b | 0x20 | 0x09 | 0x0c | 0x19
        | 0x05 | 0x29 | 0x24 | 0x25 | 0x06 | 0x1e | 0x1f | 0x1c | 0x07 | 0x2a | 0x03 | 0x16
        | 0x0a | 0x35 | 0x36 | 0x3a | 0x39 | 0x37 | 0x34 => 4.0,
        _ => 3.0,
    }
}

/// 眉毛原生基准角度表 (EyebrowRotBase)
pub fn get_eyebrow_rot_base(eyebrow_type: i32) -> f32 {
    match eyebrow_type {
        0x09 | 0x07 | 0x11 | 0x05 | 0x03 | 0x16 | 0x10 | 0x17 => 7.0,
        0x0b => 8.0,
        0x06 | 0x00 | 0x01 | 0x13 | 0x15 | 0x04 | 0x0a | 0x0e | 0x0f | 0x12 => 6.0,
        0x0c | 0x08 | 0x02 | 0x14 | 0x0d => 5.0,
        _ => 6.0,
    }
}

/// 自动探测并转换任意已知 Mii 二进制数据或十六进制文本至 UMii
pub fn convert_mii_to_umii(data: &[u8], filename_hint: Option<&str>) -> Result<UmiiData, String> {
    // 1. 若为 Studio 十六进制纯文本 (92-94 字符十六进制)
    if let Ok(text) = std::str::from_utf8(data) {
        let trimmed = text.trim();
        if trimmed.len() >= 90 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(hex_bytes) = hex_to_bytes(trimmed) {
                return parse_studio_mii(&hex_bytes);
            }
        }
    }

    // 2. 根据文件后缀或长度推断
    let ext = filename_hint.and_then(|h| h.rsplit('.').next()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "mii" | "mae" | "miigx" | "rcd" => return parse_wii_mii(data),
        "ffsd" | "cfsd" => return parse_wiiu_3ds_mii(data),
        "charinfo" => return parse_switch_charinfo(data),
        _ => {}
    }

    // 3. 按照二进制长度判定
    if data.len() == 74 {
        parse_wii_mii(data)
    } else if data.len() == 96 {
        parse_wiiu_3ds_mii(data)
    } else if data.len() >= 80 && data.len() <= 92 {
        parse_switch_charinfo(data)
    } else if data.len() == 46 || data.len() == 47 {
        parse_studio_mii(data)
    } else {
        Err(format!("无法识别的 Mii 二进制格式 (字节大小: {})", data.len()))
    }
}

/// 解析 Wii 原生 Gen 1 Mii (74 字节)
pub fn parse_wii_mii(data: &[u8]) -> Result<UmiiData, String> {
    if data.len() < 74 {
        return Err("Wii Mii 数据长度不足 74 字节".to_string());
    }

    let mut umii = UmiiData::default();
    let is_female = ((data[0] >> 6) & 1) == 1;
    umii.personal_sex_age = if is_female { 4 } else { 1 };
    umii.personal_fav_color = ((data[0] >> 1) & 0x0F) as i32;
    umii.personal_personality = if is_female { "Woman_Normal".to_string() } else { "Man_Normal".to_string() };
    umii.personal_voice_type = if is_female { "Hylia_Woman_Young_Normal04".to_string() } else { "Hylia_Man_Young_Normal03".to_string() };

    umii.name = decode_utf16_be(&data[2..22]);
    umii.body_height = (data[22] as f32 / 63.5) as i32;
    umii.body_weight = (data[23] as f32 / 42.33) as i32;

    let b32 = data[32];
    umii.shape_jaw = ((b32 >> 5) & 0x07) as i32;
    umii.shape_skin_color = map_skin_tone((b32 >> 2) & 0x07);
    umii.beard_type = (b32 & 0x03) as i32;

    let b34 = data[34];
    umii.hair_type = (b34 & 0x7F) as i32;
    let b35 = data[35];
    umii.hair_color = ((b35 >> 5) & 0x07) as i32;
    umii.hair_flip = ((b35 >> 4) & 1) == 1;

    let b36 = data[36];
    umii.eyebrow_type = ((b36 >> 3) & 0x1F) as i32;
    let b37 = data[37];
    let eb_rot = ((b37 >> 4) & 0x0F) as f32;
    umii.eyebrow_rotate = eb_rot - get_eyebrow_rot_base(umii.eyebrow_type);
    umii.eyebrow_color = ((b37 >> 1) & 0x07) as i32;

    let b38 = data[38];
    umii.eyebrow_scale = ((b38 >> 4) & 0x0F) as f32;
    umii.eyebrow_trans_v = (b38 & 0x0F) as f32 - 3.0;
    umii.eyebrow_trans_u = (data[39] & 0x0F) as f32;

    let b40 = data[40];
    umii.eye_type = (b40 & 0x3F) as i32;
    let b41 = data[41];
    let eye_rot = ((b41 >> 5) & 0x07) as f32;
    umii.eye_rotate = eye_rot - get_eye_rot_base(umii.eye_type);
    umii.eye_trans_v = (b41 & 0x1F) as f32;

    let b42 = data[42];
    umii.eye_color = ((b42 >> 5) & 0x07) as i32;
    umii.eye_scale = ((b42 >> 1) & 0x07) as f32;
    umii.eye_trans_u = (data[43] & 0x0F) as f32;

    umii.nose_type = (data[44] & 0x0F) as i32;
    umii.nose_scale = ((data[44] >> 4) & 0x0F) as f32;
    umii.nose_trans_v = (data[45] & 0x1F) as f32;

    umii.mouth_type = (data[46] & 0x1F) as i32;
    umii.mouth_color = ((data[46] >> 5) & 0x03) as i32;
    umii.mouth_scale = ((data[47] >> 4) & 0x0F) as f32;
    umii.mouth_trans_v = (data[47] & 0x1F) as f32;

    umii.glass_type = (data[48] & 0x0F) as i32;
    umii.glass_color = ((data[48] >> 4) & 0x07) as i32;

    umii.beard_mustache = (data[50] & 0x03) as i32;
    umii.beard_color = ((data[50] >> 4) & 0x07) as i32;
    umii.beard_scale = ((data[51] >> 4) & 0x0F) as f32;

    Ok(umii)
}

/// 解析 Wii U / 3DS 原生 Gen 2 FFLStoreData (96 字节)
pub fn parse_wiiu_3ds_mii(data: &[u8]) -> Result<UmiiData, String> {
    if data.len() < 96 {
        return Err("Wii U / 3DS Mii 数据长度不足 96 字节".to_string());
    }

    let mut umii = UmiiData::default();
    let data_1 = u16::from_le_bytes([data[0x14], data[0x15]]);
    umii.personal_fav_color = ((data_1 >> 10) & 0x0F) as i32;

    umii.name = decode_utf16_le(&data[0x1A..0x2E]);
    umii.body_height = (data[0x2E] as f32 / 63.5) as i32;
    umii.body_weight = (data[0x2F] as f32 / 42.33) as i32;

    let b30 = data[0x30];
    umii.shape_skin_color = map_skin_tone((b30 >> 1) & 0x07);
    umii.shape_jaw = ((b30 >> 4) & 0x0F) as i32;

    let b31 = data[0x31];
    umii.shape_make = ((b31 >> 4) & 0x0F) as i32;
    umii.shape_wrinkle = (b31 & 0x0F) as i32;

    umii.hair_type = data[0x32] as i32;
    let b33 = data[0x33];
    umii.hair_flip = (b33 & 1) == 1;
    umii.hair_color = ((b33 >> 1) & 0x07) as i32;

    let eye_word = u32::from_le_bytes([data[0x34], data[0x35], data[0x36], data[0x37]]);
    umii.eye_type = (eye_word & 0x3F) as i32;
    umii.eye_color = ((eye_word >> 6) & 0x07) as i32;
    umii.eye_scale = ((eye_word >> 9) & 0x07) as f32;
    umii.eye_aspect = ((eye_word >> 12) & 0x07) as f32;
    let eye_rot = ((eye_word >> 15) & 0x07) as f32;
    umii.eye_rotate = eye_rot - get_eye_rot_base(umii.eye_type);
    umii.eye_trans_u = ((eye_word >> 18) & 0x0F) as f32;
    umii.eye_trans_v = ((eye_word >> 22) & 0x1F) as f32;

    let eb_word = u32::from_le_bytes([data[0x38], data[0x39], data[0x3A], data[0x3B]]);
    umii.eyebrow_type = (eb_word & 0x1F) as i32;
    umii.eyebrow_color = ((eb_word >> 5) & 0x07) as i32;
    umii.eyebrow_scale = ((eb_word >> 8) & 0x0F) as f32;
    let eb_rot = ((eb_word >> 12) & 0x0F) as f32;
    umii.eyebrow_rotate = eb_rot - get_eyebrow_rot_base(umii.eyebrow_type);
    umii.eyebrow_trans_u = ((eb_word >> 16) & 0x0F) as f32;
    umii.eyebrow_trans_v = ((eb_word >> 20) & 0x1F) as f32 - 3.0;

    let nose_word = u16::from_le_bytes([data[0x3C], data[0x3D]]);
    umii.nose_type = (nose_word & 0x1F) as i32;
    umii.nose_scale = ((nose_word >> 5) & 0x0F) as f32;
    umii.nose_trans_v = ((nose_word >> 9) & 0x1F) as f32;

    let mouth_word = u32::from_le_bytes([data[0x3E], data[0x3F], data[0x40], data[0x41]]);
    umii.mouth_type = (mouth_word & 0x3F) as i32;
    umii.mouth_color = ((mouth_word >> 6) & 0x07) as i32;
    umii.mouth_scale = ((mouth_word >> 9) & 0x0F) as f32;
    umii.mouth_trans_v = ((mouth_word >> 13) & 0x1F) as f32;

    let beard_word = u16::from_le_bytes([data[0x42], data[0x43]]);
    umii.beard_mustache = (beard_word & 0x07) as i32;
    umii.beard_type = ((beard_word >> 3) & 0x07) as i32;
    umii.beard_color = ((beard_word >> 6) & 0x07) as i32;
    umii.beard_scale = ((beard_word >> 9) & 0x0F) as f32;

    let glass_word = u16::from_le_bytes([data[0x44], data[0x45]]);
    umii.glass_type = (glass_word & 0x0F) as i32;
    umii.glass_color = ((glass_word >> 4) & 0x07) as i32;

    umii.creator = decode_utf16_le(&data[0x48..0x5C]);

    Ok(umii)
}

/// 解析 Switch 原生 Gen 3 CharInfo 数据
pub fn parse_switch_charinfo(data: &[u8]) -> Result<UmiiData, String> {
    if data.len() < 24 {
        return Err("Switch CharInfo 数据长度不足".to_string());
    }

    let mut umii = UmiiData::default();
    umii.hair_type = data[0] as i32;
    umii.body_height = ((data[1] & 0x7F) as f32 / 63.5) as i32;
    umii.hair_flip = ((data[2] >> 7) & 1) == 1;
    umii.body_weight = ((data[2] & 0x7F) as f32 / 42.33) as i32;
    umii.hair_color = data[3] as i32;

    let is_female = ((data[4] >> 7) & 1) == 1;
    umii.personal_sex_age = if is_female { 4 } else { 1 };
    umii.eye_color = (data[4] & 0x7F) as i32;
    umii.eyebrow_color = data[5] as i32;
    umii.mouth_color = data[6] as i32;
    umii.beard_color = data[7] as i32;
    umii.glass_color = data[8] as i32;
    umii.eye_type = data[9] as i32;
    umii.mouth_type = data[10] as i32;

    Ok(umii)
}

/// 解析 Mii Studio 紧凑数据 (46/47 字节)
pub fn parse_studio_mii(data: &[u8]) -> Result<UmiiData, String> {
    if data.len() < 46 {
        return Err("Mii Studio 数据长度不足 46 字节".to_string());
    }

    let mut umii = UmiiData::default();
    umii.beard_color = data[0] as i32;
    umii.beard_type = data[1] as i32;
    umii.body_weight = (data[2] as f32 / 42.33) as i32;
    umii.eye_aspect = data[3] as f32;
    umii.eye_color = data[4] as i32;
    umii.eye_type = data[7] as i32;
    let eye_rot = data[5] as f32;
    umii.eye_rotate = eye_rot - get_eye_rot_base(umii.eye_type);
    umii.eye_scale = data[6] as f32;
    umii.eye_trans_u = data[8] as f32;
    umii.eye_trans_v = data[9] as f32;

    umii.eyebrow_aspect = data[10] as f32;
    umii.eyebrow_color = data[11] as i32;
    umii.eyebrow_type = data[14] as i32;
    let eb_rot = data[12] as f32;
    umii.eyebrow_rotate = eb_rot - get_eyebrow_rot_base(umii.eyebrow_type);
    umii.eyebrow_scale = data[13] as f32;
    umii.eyebrow_trans_u = data[15] as f32;
    umii.eyebrow_trans_v = data[16] as f32 - 3.0;

    umii.shape_skin_color = map_skin_tone(data[17]);
    umii.shape_make = data[18] as i32;
    umii.shape_jaw = data[19] as i32;
    umii.shape_wrinkle = data[20] as i32;
    umii.personal_fav_color = data[21] as i32;
    umii.personal_sex_age = if data[22] == 0 { 1 } else { 4 };

    umii.glass_color = data[23] as i32;
    umii.glass_type = data[25] as i32;

    umii.hair_color = data[27] as i32;
    umii.hair_flip = data[28] == 1;
    umii.hair_type = data[29] as i32;
    umii.body_height = (data[30] as f32 / 63.5) as i32;

    umii.mouth_aspect = data[33] as f32;
    umii.mouth_scale = data[34] as f32;
    umii.mouth_type = data[35] as i32;
    umii.mouth_trans_v = data[36] as f32;

    umii.beard_mustache = data[37] as i32;
    umii.nose_scale = data[40] as f32;
    umii.nose_type = data[41] as i32;
    umii.nose_trans_v = data[42] as f32;

    Ok(umii)
}

/// UTF-16 大端字节串解码为 Rust String
fn decode_utf16_be(bytes: &[u8]) -> String {
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .take_while(|&u| u != 0)
        .collect();
    String::from_utf16_lossy(&u16s)
}

/// UTF-16 小端字节串解码为 Rust String
fn decode_utf16_le(bytes: &[u8]) -> String {
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&u| u != 0)
        .collect();
    String::from_utf16_lossy(&u16s)
}

/// 十六进制字符转字节数组
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    for i in (0..hex.len()).step_by(2) {
        if i + 2 <= hex.len() {
            let b = u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("十六进制转换失败: {}", e))?;
            bytes.push(b);
        }
    }
    Ok(bytes)
}
