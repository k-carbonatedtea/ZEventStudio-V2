use std::collections::HashMap;
use std::sync::OnceLock;

static JPEN_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

fn get_jpen_map() -> &'static HashMap<String, String> {
    JPEN_MAP.get_or_init(|| {
        let json_str = include_str!("dict/jpen.json");
        serde_json::from_str(json_str).unwrap_or_default()
    })
}

/// 将任天堂原生日文动作名/类名翻译为可读英文字符串
pub fn translate_japanese_name(jp_name: &str) -> Option<String> {
    if jp_name.is_empty() {
        return None;
    }
    let map = get_jpen_map();
    map.get(jp_name).cloned()
}

/// 格式化多语言可读标题 (例如: "6足攻撃 (6 pairs attack)")
pub fn format_bilingual_title(name: &str) -> String {
    if let Some(en) = translate_japanese_name(name) {
        if en.eq_ignore_ascii_case(name) {
            name.to_string()
        } else {
            format!("{} ({})", name, en)
        }
    } else {
        name.to_string()
    }
}
