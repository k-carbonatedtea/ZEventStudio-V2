pub mod editor;
pub mod file_ops;
pub mod menu;
pub mod plugins;
pub mod quests;

pub use editor::get_en_editor;
pub use file_ops::get_en_file_ops;
pub use menu::get_en_menu;
pub use plugins::get_en_plugins;
pub use quests::get_en_quests;

/// 英文 (English) 本地化词典
pub fn get_en(key: &str) -> &str {
    if let Some(text) = get_en_file_ops(key) {
        return text;
    }
    if let Some(text) = get_en_menu(key) {
        return text;
    }
    if let Some(text) = get_en_editor(key) {
        return text;
    }
    if let Some(text) = get_en_plugins(key) {
        return text;
    }
    if let Some(text) = get_en_quests(key) {
        return text;
    }
    key
}