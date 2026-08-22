pub mod en;
pub mod locales;
pub mod zh;


use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Zh,
    En,
}

impl Language {
    pub fn name(&self) -> &'static str {
        match self {
            Language::Zh => "中文",
            Language::En => "English",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Language::Zh => Language::En,
            Language::En => Language::Zh,
        }
    }
}

/// 统一国际化文本获取函数
#[inline]
pub fn t<'a>(key: &'a str, lang: Language) -> &'a str {
    locales::translate(key, lang)
}
