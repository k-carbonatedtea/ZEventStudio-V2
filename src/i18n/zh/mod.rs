pub mod editor;
pub mod file_ops;
pub mod menu;
pub mod plugins;
pub mod quests;
pub mod timeline;

pub use editor::get_zh_editor;
pub use file_ops::get_zh_file_ops;
pub use menu::get_zh_menu;
pub use plugins::get_zh_plugins;
pub use quests::get_zh_quests;
pub use timeline::get_zh_timeline;

/// 中文 (Simplified Chinese) 本地化词典
pub fn get_zh(key: &str) -> &str {
    if let Some(text) = get_zh_file_ops(key) {
        return text;
    }
    if let Some(text) = get_zh_menu(key) {
        return text;
    }
    if let Some(text) = get_zh_editor(key) {
        return text;
    }
    if let Some(text) = get_zh_timeline(key) {
        return text;
    }
    if let Some(text) = get_zh_plugins(key) {
        return text;
    }
    if let Some(text) = get_zh_quests(key) {
        return text;
    }
    key
}
