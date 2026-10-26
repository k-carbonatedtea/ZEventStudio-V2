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

            let mut cur_anim = "none".to_string();
            let mut cur_sound = "none".to_string();
            let mut raw_text = String::new();

            if let Some(contents) = entry_val.get("contents").and_then(|c| c.as_sequence()) {
                for node in contents {
                    parse_single_content_node(
                        node,
                        &mut cur_anim,
                        &mut cur_sound,
                        &mut raw_text,
                    );
                }
            }

            let (spans, clean_text, line_count, char_count) = parse_raw_text_to_spans(&raw_text);

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

/// 解析 contents 数组中的单一内容节点 (支持原生 msyt 扁平格式与嵌套 control 格式)
fn parse_single_content_node(
    node: &serde_yaml::Value,
    cur_anim: &mut String,
    cur_sound: &mut String,
    raw_text: &mut String,
) {
    if let Some(text_str) = node.as_str() {
        raw_text.push_str(text_str);
        return;
    }

    if let Some(text_val) = node.get("text").and_then(|t| t.as_str()) {
        raw_text.push_str(text_val);
        return;
    }

    let ctrl = node.get("control").unwrap_or(node);
    if let Some(kind) = ctrl.get("kind").and_then(|k| k.as_str()) {
        match kind {
            "set_colour" => {
                let col = ctrl
                    .get("colour")
                    .or_else(|| ctrl.get("color"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("red");
                raw_text.push_str(&format!("<colour:{}>", col));
            }
            "reset_colour" => {
                raw_text.push_str("</colour>");
            }
            "text_size" => {
                let pct = ctrl.get("percent").and_then(|p| p.as_u64()).unwrap_or(100);
                if pct == 80 {
                    raw_text.push_str("<size:80>");
                } else if pct == 125 {
                    raw_text.push_str("<size:125>");
                } else {
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
            }
            "animation" => {
                let name = ctrl.get("name").and_then(|n| n.as_str()).unwrap_or("normal");
                *cur_anim = name.to_string();
                raw_text.push_str(&format!("<anim:{}>", name));
            }
            "sound" | "sound2" => {
                *cur_sound = "sound".to_string();
            }
            "icon" => {
                let icon_name = ctrl.get("icon").and_then(|i| i.as_str()).unwrap_or("icon");
                raw_text.push_str(&format!("<icon:{}>", icon_name));
            }
            _ => {
                raw_text.push_str(&format!("<control:{}>", kind));
            }
        }
    }
}

/// 解析文本中的颜色标签 (参考 https://zeldamods.org/wiki/Help:Text_modding)
pub fn parse_color_tag(tag: &str) -> Option<BubbleColor> {
    let clean = tag.trim().to_lowercase();
    let inner = clean
        .strip_prefix("colour:")
        .or_else(|| clean.strip_prefix("color:"))
        .unwrap_or(&clean);

    match inner {
        "red" => Some(BubbleColor::Red),
        "blue" => Some(BubbleColor::Blue),
        "grey" | "gray" | "light_grey" | "light_gray" => Some(BubbleColor::Grey),
        "orange" => Some(BubbleColor::Orange),
        "green" | "light_green" | "light_green1" | "light_green4" => Some(BubbleColor::Green),
        "yellow" => Some(BubbleColor::Yellow),
        "white" | "default" | "reset" => Some(BubbleColor::Default),
        _ => None,
    }
}

/// 将包含标签的 raw_text 解析为富文本 Spans、清洗纯文本及行数字数统计
pub fn parse_raw_text_to_spans(raw_text: &str) -> (Vec<BubbleSpan>, String, usize, usize) {
    let mut spans = Vec::new();
    let mut clean_text = String::with_capacity(raw_text.len());
    let mut cur_color = BubbleColor::Default;
    let mut cur_size = BubbleSize::Normal;

    let chars: Vec<char> = raw_text.chars().collect();
    let mut idx = 0;
    let total = chars.len();

    while idx < total {
        if chars[idx] == '<' {
            if let Some(close_rel) = chars[idx..].iter().position(|&c| c == '>') {
                let tag_str: String = chars[idx + 1..idx + close_rel].iter().collect();
                idx += close_rel + 1;

                let tag_clean = tag_str.trim().to_lowercase();
                if let Some(color) = parse_color_tag(&tag_clean) {
                    cur_color = color;
                } else if tag_clean == "/colour"
                    || tag_clean == "/color"
                    || tag_clean == "reset_colour"
                    || tag_clean == "reset_color"
                    || tag_clean == "/red"
                    || tag_clean == "/blue"
                    || tag_clean == "/grey"
                    || tag_clean == "/gray"
                    || tag_clean == "/orange"
                    || tag_clean == "/green"
                    || tag_clean == "/yellow"
                {
                    cur_color = BubbleColor::Default;
                } else if tag_clean == "size:80" || tag_clean == "size:small" {
                    cur_size = BubbleSize::Small;
                } else if tag_clean == "size:125" || tag_clean == "size:large" {
                    cur_size = BubbleSize::Large;
                } else if tag_clean == "/size" || tag_clean == "size:100" || tag_clean == "size:normal" {
                    cur_size = BubbleSize::Normal;
                } else if let Some(pause_val) = tag_clean.strip_prefix("pause:") {
                    spans.push(BubbleSpan {
                        text: String::new(),
                        color: cur_color,
                        size: cur_size,
                        is_pause: true,
                        pause_label: Some(pause_val.to_string()),
                        is_chip: false,
                        chip_label: None,
                    });
                } else if let Some(chip_val) = tag_clean.strip_prefix("anim:").or_else(|| tag_clean.strip_prefix("icon:")) {
                    spans.push(BubbleSpan {
                        text: String::new(),
                        color: cur_color,
                        size: cur_size,
                        is_pause: false,
                        pause_label: None,
                        is_chip: true,
                        chip_label: Some(chip_val.to_string()),
                    });
                }
                continue;
            }
        }

        let mut text_buf = String::new();
        while idx < total && chars[idx] != '<' {
            text_buf.push(chars[idx]);
            idx += 1;
        }

        if !text_buf.is_empty() {
            clean_text.push_str(&text_buf);
            spans.push(BubbleSpan {
                text: text_buf,
                color: cur_color,
                size: cur_size,
                is_pause: false,
                pause_label: None,
                is_chip: false,
                chip_label: None,
            });
        }
    }

    let line_count = clean_text.lines().count().max(1);
    let char_count = clean_text.chars().count();

    (spans, clean_text, line_count, char_count)
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

/// 发射 MSYT 控制标签 YAML 片段 (遵循 https://zeldamods.org/wiki/Help:Text_modding 标准)
fn emit_msyt_control_tag(tag: &str, out: &mut String) {
    let clean = tag.trim().to_lowercase();
    if let Some(color) = parse_color_tag(&clean) {
        let col_name = match color {
            BubbleColor::Red => "red",
            BubbleColor::Blue => "blue",
            BubbleColor::Grey => "grey",
            BubbleColor::Orange => "orange",
            BubbleColor::Green => "light_green1",
            BubbleColor::Yellow => "yellow",
            BubbleColor::Default => {
                out.push_str("      - control:\n          kind: reset_colour\n");
                return;
            }
        };
        out.push_str(&format!(
            "      - control:\n          kind: set_colour\n          colour: {}\n",
            col_name
        ));
    } else if clean == "/colour"
        || clean == "/color"
        || clean == "reset_colour"
        || clean == "reset_color"
        || clean == "/red"
        || clean == "/blue"
        || clean == "/grey"
        || clean == "/gray"
        || clean == "/orange"
        || clean == "/green"
        || clean == "/yellow"
    {
        out.push_str("      - control:\n          kind: reset_colour\n");
    } else if clean == "size:80" || clean == "size:small" {
        out.push_str("      - control:\n          kind: text_size\n          percent: 80\n");
    } else if clean == "size:125" || clean == "size:large" {
        out.push_str("      - control:\n          kind: text_size\n          percent: 125\n");
    } else if clean == "/size" || clean == "size:100" || clean == "size:normal" {
        out.push_str("      - control:\n          kind: text_size\n          percent: 100\n");
    } else if let Some(pause_val) = clean.strip_prefix("pause:") {
        if let Ok(frames) = pause_val.trim_end_matches('帧').parse::<u32>() {
            out.push_str(&format!("      - control:\n          kind: pause\n          frames: {}\n", frames));
        } else {
            out.push_str(&format!("      - control:\n          kind: pause\n          length: {}\n", pause_val));
        }
    } else if let Some(anim_name) = clean.strip_prefix("anim:") {
        out.push_str(&format!("      - control:\n          kind: animation\n          name: {}\n", anim_name));
    }
}
