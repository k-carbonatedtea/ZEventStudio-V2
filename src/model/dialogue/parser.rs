use std::collections::HashMap;

/// 从 MSBT 字节流中提取所有文本条目 (键名 -> 纯文本内容)
pub fn parse_msbt_entries(bytes: &[u8]) -> HashMap<String, String> {
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
                    let cleaned = clean_dialogue_text(&s);
                    if !cleaned.is_empty() {
                        map.insert(k.clone(), cleaned);
                    }
                }
            }
        }
    }

    // 2. 若 msyt 解析失败或未提取到条目，使用零依赖原生 MSBT 二进制安全只读回退解析
    if map.is_empty() && bytes.starts_with(b"MsgStdBn") {
        parse_msbt_binary_fallback(bytes, &mut map);
    }

    map
}

/// 清洗对白文本中的多余控制换行与前后空格
fn clean_dialogue_text(s: &str) -> String {
    let normalized = s.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // 压缩多余连续空行
    let mut result = String::with_capacity(trimmed.len());
    let mut consecutive_newlines = 0;
    for ch in trimmed.chars() {
        if ch == '\n' {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                result.push(ch);
            }
        } else {
            consecutive_newlines = 0;
            result.push(ch);
        }
    }
    result
}

/// 从 Msg_<Lang>.product.ssarc (或 .sarc) 字节流中检索并解析指定 MSBT 文件
pub fn extract_msbt_from_msg_sarc(
    sarc_bytes: &[u8],
    target_names: &[&str],
) -> Option<HashMap<String, String>> {
    let decomp = roead::yaz0::decompress(sarc_bytes).unwrap_or_else(|_| sarc_bytes.to_vec());
    let sarc = roead::sarc::Sarc::new(&decomp).ok()?;

    for file in sarc.files() {
        if let Some(name) = file.name() {
            let normalized = name.replace('\\', "/");
            for target in target_names {
                if normalized.eq_ignore_ascii_case(target)
                    || normalized.ends_with(target)
                    || normalized.ends_with(&format!("/{}", target))
                {
                    let entries = parse_msbt_entries(file.data());
                    if !entries.is_empty() {
                        return Some(entries);
                    }
                }
            }
        }
    }
    None
}

/// 从 Bootup_<Lang>.pack 字节流中解压并提取目标 MSBT 字典
pub fn extract_msbt_from_bootup_pack(
    pack_bytes: &[u8],
    target_names: &[&str],
) -> Option<HashMap<String, String>> {
    let decomp = roead::yaz0::decompress(pack_bytes).unwrap_or_else(|_| pack_bytes.to_vec());
    let sarc = roead::sarc::Sarc::new(&decomp).ok()?;

    // 1. 直寻 MSBT (针对某些解压展开或扁平包)
    for file in sarc.files() {
        if let Some(name) = file.name() {
            let normalized = name.replace('\\', "/");
            for target in target_names {
                if normalized.eq_ignore_ascii_case(target) || normalized.ends_with(target) {
                    let entries = parse_msbt_entries(file.data());
                    if !entries.is_empty() {
                        return Some(entries);
                    }
                }
            }
        }
    }

    // 2. 深入嵌套的 Message/Msg_*.ssarc 或 .sarc
    for file in sarc.files() {
        if let Some(name) = file.name() {
            let norm = name.replace('\\', "/");
            if (norm.starts_with("Message/Msg_") || norm.contains("Msg_"))
                && (norm.ends_with(".ssarc") || norm.ends_with(".sarc"))
            {
                if let Some(entries) = extract_msbt_from_msg_sarc(file.data(), target_names) {
                    return Some(entries);
                }
            }
        }
    }

    None
}

/// 原生 MSBT 二进制安全只读回退解析器 (支持 UTF-16LE / UTF-8)
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
                    let cleaned = clean_dialogue_text(&text);
                    if !cleaned.is_empty() {
                        map.insert(label, cleaned);
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
