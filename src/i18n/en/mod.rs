pub mod editor;
pub mod menu;

pub use editor::get_en_editor;
pub use menu::get_en_menu;

/// 英文 (English) 本地化词典
pub fn get_en(key: &str) -> &str {
    if let Some(text) = get_en_menu(key) {
        return text;
    }
    if let Some(text) = get_en_editor(key) {
        return text;
    }
    key
}
