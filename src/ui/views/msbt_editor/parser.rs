use crate::ui::views::msbt_editor::types::{
    BubbleColor, BubbleSize, BubbleSpan, MsbtDocument, MsbtEntry,
};

/// 将 MSYT YAML 文本解析为结构化的 MsbtDocument 模型
pub fn parse_msyt_to_document(yaml_str: &str) -> Result<MsbtDocument, String> {
    let root: serde_yaml::Value = serde_yaml::from_str(yaml_str)
        .map_err(|e| format!("MSYT YAML 语法解析失败: {}", e))?;

    let group_count = root
        .get("group_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    let atr1_unknown = root
        .get("atr1_unknown")
        .and_then(|v| v.as_u64())
        .map(|n| n as u32);

    let mut entries = Vec::new();

    if let Some(entries_map) = root.get("entries").and_then(|v| v.as_mapping()) {
        for (key_val, entry_val) in entries_map {
            let key = match key_val.as_str() {
                Some(k) => k.to_string(),
                None => continue,
            };

            let mut cur_color = BubbleColor::Default;
            let mut cur_size = BubbleSize::Normal;
            let mut cur_anim = "none".to_string();
            let mut cur_sound = "none".to_string();
            let mut raw_text = String::new();
            let mut clean_text = String::new();
            let mut spans = Vec::new();

            if let Some(contents) = entry_val.get("contents").and_then(|c| c.as_sequence()) {
                for node in contents {
                    parse_single_content_node(
                        node,
                        &mut cur_color,
                        &mut cur_size,
                        &mut cur_anim,
                        &mut cur_sound,
                        &mut raw_text,
                        &mut clean_text,
                        &mut spans,
                    );
                }
            }

            let line_count = clean_text.lines().count().max(1);
            let char_count = clean_text.chars().count();

            entries.push(MsbtEntry {
                key,
                raw_text,
                clean_text,
                spans,
                animation: cur_anim,
                sound: cur_sound,
                line_count,
                char_count,
            });
        }
    }

    Ok(MsbtDocument {
        group_count,
        atr1_unknown,
        entries,
    })
}

/// 解析 contents 数组中的单一内容节点
#[allow(clippy::too_many_arguments)]
fn parse_single_content_node(
    node: &serde_yaml::Value,
    cur_color: &mut BubbleColor,
    cur_size: &mut BubbleSize,
    cur_anim: &mut String,
    cur_sound: &mut String,
    raw_text: &mut String,
    clean_text: &mut String,
    spans: &mut Vec<BubbleSpan>,
) {
    if let Some(text_str) = node.as_str() {
        clean_text.push_str(text_str);
        raw_text.push_str(text_str);
        spans.push(BubbleSpan {
            text: text_str.to_string(),
            color: *cur_color,
            size: *cur_size,
            is_pause: false,
            pause_label: None,
            is_chip: false,
            chip_label: None,
        });
        return;
    }

    if let Some(text_val) = node.get("text").and_then(|t| t.as_str()) {
        clean_text.push_str(text_val);
        raw_text.push_str(text_val);
        spans.push(BubbleSpan {
            text: text_val.to_string(),
            color: *cur_color,
            size: *cur_size,
            is_pause: false,
            pause_label: None,
            is_chip: false,
            chip_label: None,
        });
        return;
    }

    if let Some(ctrl) = node.get("control") {
        let kind = ctrl.get("kind").and_then(|k| k.as_str()).unwrap_or("");
        match kind {
            "set_colour" => {
                let col = ctrl.get("colour").and_then(|c| c.as_str()).unwrap_or("");
                match col {
                    "red" => {
                        *cur_color = BubbleColor::Red;
                        raw_text.push_str("<red>");
                    }
                    "blue" => {
                        *cur_color = BubbleColor::Blue;
                        raw_text.push_str("<blue>");
                    }
                    "grey" => {
                        *cur_color = BubbleColor::Grey;
                        raw_text.push_str("<grey>");
                    }
                    _ => {}
                }
            }
            "reset_colour" => {
                *cur_color = BubbleColor::Default;
                raw_text.push_str("</color>");
            }
            "text_size" => {
                let pct = ctrl.get("percent").and_then(|p| p.as_u64()).unwrap_or(100);
                if pct == 80 {
                    *cur_size = BubbleSize::Small;
                    raw_text.push_str("<size:80>");
                } else if pct == 125 {
                    *cur_size = BubbleSize::Large;
                    raw_text.push_str("<size:125>");
                } else {
                    *cur_size = BubbleSize::Normal;
                    raw_text.push_str("</size>");
                }
            }
            "pause" => {
                let label = if let Some(len) = ctrl.get("length").and_then(|l| l.as_str()) {
                    len.to_string()
                } else if let Some(f) = ctrl.get("frames").and_then(|f| f.as_u64()) {
                    format!("{}帧", f)
                } else {
                    "short".to_string()
                };
                raw_text.push_str(&format!("<pause:{}>", label));
                spans.push(BubbleSpan {
                    text: String::new(),
                    color: *cur_color,
                    size: *cur_size,
                    is_pause: true,
                    pause_label: Some(label),
                    is_chip: false,
                    chip_label: None,
                });
            }
            "animation" => {
                let name = ctrl.get("name").and_then(|n| n.as_str()).unwrap_or("normal");
                *cur_anim = name.to_string();
                raw_text.push_str(&format!("<anim:{}>", name));
            }
            "sound" => {
                *cur_sound = "sound".to_string();
            }
            _ => {
                raw_text.push_str(&format!("<control:{}>", kind));
                spans.push(BubbleSpan {
                    text: String::new(),
                    color: *cur_color,
                    size: *cur_size,
                    is_pause: false,
                    pause_label: None,
                    is_chip: true,
                    chip_label: Some(kind.to_string()),
                });
            }
        }
    }
}

/// 将 MsbtDocument 重新序列化为合法的 MSYT YAML 文本
pub fn serialize_document_to_msyt(doc: &MsbtDocument) -> Result<String, String> {
    let mut out = String::with_capacity(4096);
    out.push_str(&format!("group_count: {}\n", doc.group_count.max(doc.entries.len() as u32)));

    if let Some(atr1) = doc.atr1_unknown {
        out.push_str(&format!("atr1_unknown: {}\n", atr1));
    }

    out.push_str("entries:\n");

    for entry in &doc.entries {
        out.push_str(&format!("  {}:\n", entry.key));
        out.push_str("    contents:\n");
        serialize_entry_raw_text(&entry.raw_text, &mut out);
    }

    Ok(out)
}

/// 解析 entry 的 raw_text 并转换为 MSYT contents 序列项
fn serialize_entry_raw_text(raw_text: &str, out: &mut String) {
    if !raw_text.contains('<') {
        if raw_text.contains('\n') {
            out.push_str("      - |-\n");
            for line in raw_text.lines() {
                out.push_str(&format!("        {}\n", line));
            }
        } else {
            let escaped = raw_text.replace('\\', "\\\\").replace('"', "\\\"");
            out.push_str(&format!("      - \"{}\"\n", escaped));
        }
        return;
    }

    let mut cursor = 0;
    let chars: Vec<char> = raw_text.chars().collect();
    let total_len = chars.len();

    while cursor < total_len {
        if chars[cursor] == '<' {
            if let Some(close_idx) = chars[cursor..].iter().position(|&c| c == '>') {
                let tag_end = cursor + close_idx;
                let tag_str: String = chars[cursor + 1..tag_end].iter().collect();
                cursor = tag_end + 1;
                emit_msyt_control_tag(&tag_str, out);
                continue;
            }
        }

        let mut text_buf = String::new();
        while cursor < total_len && chars[cursor] != '<' {
            text_buf.push(chars[cursor]);
            cursor += 1;
        }

        if !text_buf.is_empty() {
            let escaped = text_buf.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
            out.push_str(&format!("      - text: \"{}\"\n", escaped));
        }
    }
}

/// 发射 MSYT 控制标签 YAML 片段
fn emit_msyt_control_tag(tag: &str, out: &mut String) {
    if tag == "red" {
        out.push_str("      - control:\n          kind: set_colour\n          colour: red\n");
    } else if tag == "blue" {
        out.push_str("      - control:\n          kind: set_colour\n          colour: blue\n");
    } else if tag == "grey" {
        out.push_str("      - control:\n          kind: set_colour\n          colour: grey\n");
    } else if tag == "/color" || tag == "/red" || tag == "/blue" || tag == "/grey" {
        out.push_str("      - control:\n          kind: reset_colour\n");
    } else if tag == "size:80" {
        out.push_str("      - control:\n          kind: text_size\n          percent: 80\n");
    } else if tag == "size:125" {
        out.push_str("      - control:\n          kind: text_size\n          percent: 125\n");
    } else if tag == "/size" {
        out.push_str("      - control:\n          kind: text_size\n          percent: 100\n");
    } else if let Some(pause_val) = tag.strip_prefix("pause:") {
        if let Ok(frames) = pause_val.trim_end_matches('帧').parse::<u32>() {
            out.push_str(&format!("      - control:\n          kind: pause\n          frames: {}\n", frames));
        } else {
            out.push_str(&format!("      - control:\n          kind: pause\n          length: {}\n", pause_val));
        }
    } else if let Some(anim_name) = tag.strip_prefix("anim:") {
        out.push_str(&format!("      - control:\n          kind: animation\n          name: {}\n", anim_name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_msyt_basic() {
        let yaml = r#"
group_count: 2
atr1_unknown: 0
entries:
  Armor_001_Head_Name:
    contents:
      - 海利亚兜帽
  Armor_001_Head_Desc:
    contents:
      - text: "选取海拉鲁传统工艺织出的结实布料，"
      - control:
          kind: set_colour
          colour: red
      - text: "关键提示"
      - control:
          kind: reset_colour
      - control:
          kind: pause
          length: short
"#;
        let doc = parse_msyt_to_document(yaml).expect("parse failed");
        assert_eq!(doc.entries.len(), 2);
        assert_eq!(doc.entries[0].key, "Armor_001_Head_Name");
        assert_eq!(doc.entries[0].clean_text, "海利亚兜帽");

        let desc = &doc.entries[1];
        assert_eq!(desc.key, "Armor_001_Head_Desc");
        assert!(desc.raw_text.contains("<red>关键提示</color>"));
        assert!(desc.raw_text.contains("<pause:short>"));

        let serialized = serialize_document_to_msyt(&doc).expect("serialize failed");
        assert!(serialized.contains("Armor_001_Head_Name:"));
        assert!(serialized.contains("set_colour"));
        assert!(serialized.contains("reset_colour"));
    }
}
