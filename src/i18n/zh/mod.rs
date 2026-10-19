pub mod editor;
pub mod file_ops;
pub mod menu;

pub use editor::get_zh_editor;
pub use file_ops::get_zh_file_ops;
pub use menu::get_zh_menu;

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
    key
}
