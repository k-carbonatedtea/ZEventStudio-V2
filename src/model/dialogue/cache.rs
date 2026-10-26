use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::model::GameLanguage;

/// 对白来源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogueSource {
    /// 来源于当前打开的项目目录或自定义 Mod 封包 (最高优先级)
    Project,
    /// 来源于首选项配置的游戏 Dump 路径 (回退优先级)
    GameDump,
}

impl DialogueSource {
    #[allow(dead_code)]
    pub const fn badge_text(&self) -> &'static str {
        match self {
            Self::Project => "项目",
            Self::GameDump => "Dump",
        }
    }
}

/// 解析获得的单条对白结果项
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogueItem {
    pub text: String,
    pub source: DialogueSource,
    pub source_path: String,
}

/// 缓存索引键 (语言 + 归一化文件键名)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogueCacheKey {
    pub language: GameLanguage,
    pub file_key: String,
}

/// 单个 MSBT 文件的解析缓存集合
#[derive(Debug, Clone)]
pub struct DialogueFileEntries {
    pub entries: HashMap<String, String>,
    pub source: DialogueSource,
    pub source_path: String,
}

/// 全局对白文本缓存管理器
#[derive(Clone, Default)]
pub struct DialogueCacheManager {
    files: Arc<RwLock<HashMap<DialogueCacheKey, DialogueFileEntries>>>,
}

impl DialogueCacheManager {
    pub fn global() -> &'static DialogueCacheManager {
        static INSTANCE: std::sync::OnceLock<DialogueCacheManager> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(DialogueCacheManager::default)
    }

    /// 查询单条对白文本
    pub fn get_entry(&self, key: &DialogueCacheKey, entry_name: &str) -> Option<DialogueItem> {
        let guard = self.files.read().ok()?;
        let file_cache = guard.get(key)?;
        let text = file_cache.entries.get(entry_name)?;
        Some(DialogueItem {
            text: text.clone(),
            source: file_cache.source,
            source_path: file_cache.source_path.clone(),
        })
    }

    /// 检查指定文件是否已解析并存入缓存
    pub fn contains_file(&self, key: &DialogueCacheKey) -> bool {
        self.files.read().map(|g| g.contains_key(key)).unwrap_or(false)
    }

    /// 插入解析到的完整 MSBT 文件字典
    pub fn insert_file_entries(
        &self,
        key: DialogueCacheKey,
        entries: HashMap<String, String>,
        source: DialogueSource,
        source_path: String,
    ) {
        if let Ok(mut guard) = self.files.write() {
            guard.insert(
                key,
                DialogueFileEntries {
                    entries,
                    source,
                    source_path,
                },
            );
        }
    }

    /// 清空所有缓存 (切换语言或重新加载项目时触发)
    pub fn clear(&self) {
        if let Ok(mut guard) = self.files.write() {
            guard.clear();
        }
    }
}
