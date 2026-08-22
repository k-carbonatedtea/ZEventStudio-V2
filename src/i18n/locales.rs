use super::Language;
use crate::i18n::en::get_en;
use crate::i18n::zh::get_zh;

/// 根据键名与目标语言返回对应的本地化文本
pub fn translate<'a>(key: &'a str, lang: Language) -> &'a str {
    match lang {
        Language::Zh => get_zh(key),
        Language::En => get_en(key),
    }
}
