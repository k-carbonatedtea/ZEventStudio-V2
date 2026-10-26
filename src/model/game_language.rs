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

    /// 获取该语言对应的 Bootup_<Lang>.pack 候选文件名 (主包名, 备选回退包名)
    pub const fn bootup_pack_names(&self) -> (&'static str, Option<&'static str>) {
        match self {
            Self::CNzh => ("Bootup_CNzh.pack", Some("Bootup_TWzh.pack")),
            Self::TWzh => ("Bootup_TWzh.pack", Some("Bootup_CNzh.pack")),
            Self::USen => ("Bootup_USen.pack", Some("Bootup_EUen.pack")),
            Self::EUen => ("Bootup_EUen.pack", Some("Bootup_USen.pack")),
            Self::JPja => ("Bootup_JPja.pack", None),
            Self::KRko => ("Bootup_KRko.pack", None),
            Self::EUfr => ("Bootup_EUfr.pack", Some("Bootup_USfr.pack")),
            Self::EUde => ("Bootup_EUde.pack", None),
            Self::EUes => ("Bootup_EUes.pack", Some("Bootup_USes.pack")),
            Self::EUit => ("Bootup_EUit.pack", None),
            Self::EUnl => ("Bootup_EUnl.pack", None),
            Self::EUru => ("Bootup_EUru.pack", None),
        }
    }

    /// 是否属于英语变体
    pub const fn is_english(&self) -> bool {
        matches!(self, Self::USen | Self::EUen)
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

