use std::collections::HashMap;

/// 从 LocationMarker.msbt 字节流中提取全部地标翻译键值对 (如 "UMiiVillage" => "一始村")
pub fn parse_location_msbt(bytes: &[u8]) -> HashMap<String, String> {
    let mut map = HashMap::new();

    // 1. 优先使用标准的 msyt 库进行解析
    if let Ok(msyt_doc) = msyt::Msyt::from_msbt_bytes(bytes) {
        if let Ok(json_val) = serde_json::to_value(&msyt_doc) {
            if let Some(entries) = json_val.get("entries").and_then(|v| v.as_object()) {
                for (k, entry) in entries {
                    let mut s = String::new();
                    if let Some(contents) = entry.get("contents").and_then(|v| v.as_array()) {
                        for part in contents {
                            if let Some(txt) = part.as_str() {
                                s.push_str(txt);
                            }
                        }
                    }
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        map.insert(k.clone(), trimmed.to_string());
                    }
                }
            }
        }
    }

    // 2. 若 msyt 偶发失败，使用零依赖原生 MSBT (UTF-16LE / UTF-8) 二进制兜底快速解析
    if map.is_empty() && bytes.starts_with(b"MsgStdBn") {
        parse_msbt_binary_fallback(bytes, &mut map);
    }

    map
}

/// 从 Bootup_<Lang>.pack 中解压并提取 StaticMsg/LocationMarker.msbt 地名字典
pub fn extract_location_msbt_from_pack(pack_bytes: &[u8]) -> HashMap<String, String> {
    let decomp = roead::yaz0::decompress(pack_bytes).unwrap_or_else(|_| pack_bytes.to_vec());
    let Ok(sarc) = roead::sarc::Sarc::new(&decomp) else {
        return HashMap::new();
    };

    // 1. 直寻 LocationMarker.msbt
    for file in sarc.files() {
        if let Some(name) = file.name() {
            if name.ends_with("LocationMarker.msbt") {
                return parse_location_msbt(file.data());
            }
        }
    }

    // 2. 探查嵌套的 Msg_<Lang>.product.ssarc
    for file in sarc.files() {
        if let Some(name) = file.name() {
            if name.ends_with(".ssarc") || name.ends_with(".sarc") {
                let inner_data = roead::yaz0::decompress(file.data()).unwrap_or_else(|_| file.data().to_vec());
                if let Ok(inner_sarc) = roead::sarc::Sarc::new(&inner_data) {
                    for inner_f in inner_sarc.files() {
                        if let Some(in_name) = inner_f.name() {
                            if in_name.ends_with("LocationMarker.msbt") {
                                return parse_location_msbt(inner_f.data());
                            }
                        }
                    }
                }
            }
        }
    }

    HashMap::new()
}

/// 原生 MSBT 二进制安全只读回退解析器
fn parse_msbt_binary_fallback(bytes: &[u8], map: &mut HashMap<String, String>) {
    let lbl1_pos = find_subslice(bytes, b"LBL1");
    let txt2_pos = find_subslice(bytes, b"TXT2");

    let (Some(lbl1), Some(txt2)) = (lbl1_pos, txt2_pos) else { return };
    if lbl1 + 0x14 > bytes.len() || txt2 + 0x14 > bytes.len() { return };

    let num_groups = u32::from_le_bytes(bytes[lbl1 + 0x10..lbl1 + 0x14].try_into().unwrap_or_default()) as usize;
    let num_strings = u32::from_le_bytes(bytes[txt2 + 0x10..txt2 + 0x14].try_into().unwrap_or_default()) as usize;

    let mut string_offsets = Vec::with_capacity(num_strings);
    for i in 0..num_strings {
        let p = txt2 + 0x14 + i * 4;
        if p + 4 <= bytes.len() {
            let rel = u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap_or_default()) as usize;
            string_offsets.push(txt2 + 0x10 + rel);
        }
    }

    for i in 0..num_groups {
        let grp_pos = lbl1 + 0x14 + i * 8;
        if grp_pos + 8 > bytes.len() { break; }
        let count = u32::from_le_bytes(bytes[grp_pos..grp_pos + 4].try_into().unwrap_or_default()) as usize;
        let offset = u32::from_le_bytes(bytes[grp_pos + 4..grp_pos + 8].try_into().unwrap_or_default()) as usize;

        let mut cur = lbl1 + 0x10 + offset;
        for _ in 0..count {
            if cur >= bytes.len() { break; }
            let len = bytes[cur] as usize;
            cur += 1;
            if cur + len + 4 > bytes.len() { break; }
            let label = String::from_utf8_lossy(&bytes[cur..cur + len]).to_string();
            cur += len;
            let idx = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap_or_default()) as usize;
            cur += 4;

            if let Some(&str_pos) = string_offsets.get(idx) {
                if str_pos < bytes.len() {
                    let mut text = String::new();
                    // 检测 UTF-16LE 字符串 (双字节 0 终止)
                    let mut p = str_pos;
                    let mut u16_buf = Vec::new();
                    while p + 1 < bytes.len() {
                        let c = u16::from_le_bytes([bytes[p], bytes[p + 1]]);
                        if c == 0 { break; }
                        u16_buf.push(c);
                        p += 2;
                    }
                    if let Ok(s) = String::from_utf16(&u16_buf) {
                        text = s;
                    }
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        map.insert(label, trimmed.to_string());
                    }
                }
            }
        }
    }
}

/// 辅助查找子切片在父字节流中的偏移
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}
