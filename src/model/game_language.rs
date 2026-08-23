use serde::{Deserialize, Serialize};

/// 任天堂游戏文本本地化语言 (BotW / TotK MSBT / EventFlow Language)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GameLanguage {
    #[default]
    CNzh,
    TWzh,
    USen,
    EUen,
    JPja,
    KRko,
    EUfr,
    EUde,
    EUes,
    EUit,
    EUnl,
    EUru,
}

impl GameLanguage {
    /// 获取任天堂标准语言代码 (例如 "CNzh", "USen")
    #[allow(dead_code)]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::CNzh => "CNzh",
            Self::TWzh => "TWzh",
            Self::USen => "USen",
            Self::EUen => "EUen",
            Self::JPja => "JPja",
            Self::KRko => "KRko",
            Self::EUfr => "EUfr",
            Self::EUde => "EUde",
            Self::EUes => "EUes",
            Self::EUit => "EUit",
            Self::EUnl => "EUnl",
            Self::EUru => "EUru",
        }
    }

    /// 获取语言原生显示名称
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::CNzh => "简体中文 (CNzh)",
            Self::TWzh => "繁體中文 (TWzh)",
            Self::USen => "English - US (USen)",
            Self::EUen => "English - Europe (EUen)",
            Self::JPja => "日本語 (JPja)",
            Self::KRko => "한국어 (KRko)",
            Self::EUfr => "Français (EUfr)",
            Self::EUde => "Deutsch (EUde)",
            Self::EUes => "Español (EUes)",
            Self::EUit => "Italiano (EUit)",
            Self::EUnl => "Nederlands (EUnl)",
            Self::EUru => "Русский (EUru)",
        }
    }

    /// 获取所有支持的游戏语言列表
    pub const fn all() -> &'static [GameLanguage] {
        &[
            Self::CNzh,
            Self::TWzh,
            Self::USen,
            Self::EUen,
            Self::JPja,
            Self::KRko,
            Self::EUfr,
            Self::EUde,
            Self::EUes,
            Self::EUit,
            Self::EUnl,
            Self::EUru,
        ]
    }
}
