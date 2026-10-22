use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use roead::sarc::Sarc;
use serde::{Deserialize, Serialize};
use super::parser::{parse_ai_defs, parse_baiprog, ActorAiProg, AiDefs};

/// AI 数据加载与缓存状态信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiDataStatus {
    pub is_loaded: bool,
    pub actions_count: usize,
    pub queries_count: usize,
    pub actors_count: usize,
    pub source_path: String,
}

/// 全局 AI 定义与程序缓存管理器
#[derive(Clone, Default)]
pub struct AiCacheManager {
    pub ai_defs: Arc<RwLock<Option<AiDefs>>>,
    pub actor_progs: Arc<RwLock<HashMap<String, ActorAiProg>>>,
    pub source_path: Arc<RwLock<String>>,
}

impl AiCacheManager {
    pub fn global() -> &'static AiCacheManager {
        static INSTANCE: std::sync::OnceLock<AiCacheManager> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(AiCacheManager::default)
    }

    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut g = self.ai_defs.write().unwrap_or_else(|p| p.into_inner());
        *g = None;
        let mut p = self.actor_progs.write().unwrap_or_else(|p| p.into_inner());
        p.clear();
        let mut s = self.source_path.write().unwrap_or_else(|p| p.into_inner());
        s.clear();
    }
}

/// 解析常用配置文件中的路径并追加至候选目录列表
fn parse_settings_paths(file_path: &Path, add_dir: &mut impl FnMut(PathBuf)) {
    if !file_path.exists() {
        return;
    }
    if let Ok(content) = std::fs::read_to_string(file_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            // 支持 snake_case 与 camelCase
            for section_key in ["wiiu", "switch"] {
                if let Some(sec) = val.get(section_key) {
                    for key in ["update_dir", "game_dir", "dlc_dir", "updateDir", "gameDir", "dlcDir"] {
                        if let Some(s) = sec.get(key).and_then(|v| v.as_str()) {
                            if !s.trim().is_empty() {
                                add_dir(PathBuf::from(s.trim()));
                            }
                        }
                    }
                }
            }
            for custom_key in ["custom_ai_dump_dir", "customAiDumpDir"] {
                if let Some(s) = val.get(custom_key).and_then(|v| v.as_str()) {
                    if !s.trim().is_empty() {
                        add_dir(PathBuf::from(s.trim()));
                    }
                }
            }
            // BCML 常用扁平字段
            for bcml_key in ["game_dir", "game_dir_nx", "update_dir", "dlc_dir", "dlc_dir_nx", "export_dir"] {
                if let Some(s) = val.get(bcml_key).and_then(|v| v.as_str()) {
                    if !s.trim().is_empty() {
                        add_dir(PathBuf::from(s.trim()));
                    }
                }
            }
        }
    }
}

/// 自动搜集游戏及环境中的有效根目录列表
pub fn get_search_directories(
    explicit_dirs: &[PathBuf],
    custom_dump_dir: Option<&str>,
) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    let mut add_dir = |p: PathBuf| {
        if p.exists() && !dirs.contains(&p) {
            dirs.push(p);
        }
    };

    // 1. 自定义解包/覆盖导出目录优先
    if let Some(d) = custom_dump_dir {
        if !d.trim().is_empty() {
            add_dir(PathBuf::from(d.trim()));
        }
    }

    // 2. 传入的显式目录列表
    for p in explicit_dirs {
        add_dir(p.clone());
    }

    // 3. 默认文档目录下的 ai_dump
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        add_dir(PathBuf::from(&user_profile).join("Documents").join("EventEditor").join("ai_dump"));
        add_dir(PathBuf::from(&user_profile).join("Documents").join("Zevent").join("ai_dump"));
    }

    // 4. 读取 Zevent 本地配置 (%LOCALAPPDATA%/Zevent/settings.json)
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let zevent_settings = PathBuf::from(&local_appdata).join("Zevent").join("settings.json");
        parse_settings_paths(&zevent_settings, &mut add_dir);
    }

    // 5. 读取 BCML 本地配置 (%LOCALAPPDATA%/bcml/settings.json)
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let bcml_settings = PathBuf::from(&local_appdata).join("bcml").join("settings.json");
        parse_settings_paths(&bcml_settings, &mut add_dir);
    }

    // 6. 读取 Event Studio / EventEditor 本地配置 (Documents/EventEditor/settings.json)
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let app_settings = PathBuf::from(user_profile).join("Documents").join("EventEditor").join("settings.json");
        parse_settings_paths(&app_settings, &mut add_dir);
    }

    dirs
}

/// 尝试从单个文件加载 AIDef 定义
fn try_load_aidef_from_file(path: &Path) -> Option<AiDefs> {
    let fname = path.file_name()?.to_string_lossy().to_lowercase();
    if fname.ends_with(".sbyml") || fname.ends_with(".byml") {
        let bytes = std::fs::read(path).ok()?;
        let defs = parse_ai_defs(&bytes).ok()?;
        if !defs.actions.is_empty() {
            return Some(defs);
        }
    } else if fname.ends_with(".pack") || fname.ends_with(".sarc") {
        let bytes = std::fs::read(path).ok()?;
        let uncompressed = roead::yaz0::decompress(&bytes).unwrap_or(bytes);
        let sarc = Sarc::new(&uncompressed).ok()?;
        if let Some(file) = sarc.files().find(|f| {
            f.name().is_some_and(|n| n.ends_with("AIDef_Game.product.sbyml") || n.ends_with("AIDef_Game.product.byml"))
        }) {
            let defs = parse_ai_defs(file.data()).ok()?;
            if !defs.actions.is_empty() {
                return Some(defs);
            }
        }
    }
    None
}

/// 尝试加载全局 AIDef_Game.product.sbyml
pub fn load_global_ai_defs(search_dirs: &[PathBuf]) -> Result<AiDefs, String> {
    let cache = AiCacheManager::global();
    {
        let read_guard = cache.ai_defs.read().unwrap_or_else(|p| p.into_inner());
        if let Some(defs) = &*read_guard {
            return Ok(defs.clone());
        }
    }

    let relative_paths = [
        "Pack/Bootup.pack",
        "content/Pack/Bootup.pack",
        "romfs/Pack/Bootup.pack",
        "Bootup.pack",
        "Actor/AIDef/AIDef_Game.product.sbyml",
        "content/Actor/AIDef/AIDef_Game.product.sbyml",
        "romfs/Actor/AIDef/AIDef_Game.product.sbyml",
        "Pack/Bootup.pack/Actor/AIDef/AIDef_Game.product.sbyml",
        "content/Pack/Bootup.pack/Actor/AIDef/AIDef_Game.product.sbyml",
        "romfs/Pack/Bootup.pack/Actor/AIDef/AIDef_Game.product.sbyml",
        "AIDef_Game.product.sbyml",
        "AIDef_Game.product.byml",
    ];

    for base in search_dirs {
        if base.is_file() {
            if let Some(defs) = try_load_aidef_from_file(base) {
                let mut write_guard = cache.ai_defs.write().unwrap_or_else(|p| p.into_inner());
                *write_guard = Some(defs.clone());
                if let Ok(mut src) = cache.source_path.write() {
                    *src = base.to_string_lossy().to_string();
                }
                return Ok(defs);
            }
        }

        for rel in &relative_paths {
            let full_path = base.join(rel);
            if full_path.is_file() {
                if let Some(defs) = try_load_aidef_from_file(&full_path) {
                    let mut write_guard = cache.ai_defs.write().unwrap_or_else(|p| p.into_inner());
                    *write_guard = Some(defs.clone());
                    if let Ok(mut src) = cache.source_path.write() {
                        *src = full_path.to_string_lossy().to_string();
                    }
                    return Ok(defs);
                }
            }
        }
    }

    Err("未在搜索路径中找到有效的 AIDef_Game.product.sbyml 定义文件".to_string())
}

/// 尝试读取指定角色的 AI 程序定义 (.baiprog)
pub fn load_actor_aiprog(actor_name: &str, search_dirs: &[PathBuf]) -> Result<ActorAiProg, String> {
    let clean_actor = actor_name.split('[').next().unwrap_or(actor_name);
    let clean_actor = clean_actor.split('(').next().unwrap_or(clean_actor).trim();
    if clean_actor.is_empty() {
        return Err("角色名称为空".to_string());
    }

    let cache = AiCacheManager::global();
    {
        let read_guard = cache.actor_progs.read().unwrap_or_else(|p| p.into_inner());
        if let Some(prog) = read_guard.get(clean_actor) {
            return Ok(prog.clone());
        }
    }

    let mut prog = ActorAiProg::default();

    // 1. 查找解包后的 loose .baiprog 文件目录
    for base in search_dirs {
        let loose_candidates = [
            base.join("Actor").join("Pack").join(format!("{}.sbactorpack", clean_actor)).join("Actor").join("AIProgram"),
            base.join("Actor").join("Pack").join(clean_actor).join("Actor").join("AIProgram"),
            base.join("Actor").join("AIProgram"),
            base.join(clean_actor).join("Actor").join("AIProgram"),
        ];
        for loose_dir in &loose_candidates {
            if loose_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(loose_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|ext| ext == "baiprog") {
                            if let Ok(bytes) = std::fs::read(&path) {
                                let _ = parse_baiprog(&bytes, &mut prog);
                            }
                        }
                    }
                }
            }
        }
        if !prog.actions.is_empty() || !prog.queries.is_empty() {
            let mut write_guard = cache.actor_progs.write().unwrap_or_else(|p| p.into_inner());
            write_guard.insert(clean_actor.to_string(), prog.clone());
            return Ok(prog);
        }
    }

    // 2. 查找独立的 .sbactorpack / .actorpack 文件 (SARC 包内提取)
    let pack_rel_paths = [
        format!("Actor/Pack/{}.sbactorpack", clean_actor),
        format!("content/Actor/Pack/{}.sbactorpack", clean_actor),
        format!("romfs/Actor/Pack/{}.sbactorpack", clean_actor),
        format!("{}.sbactorpack", clean_actor),
        format!("Actor/Pack/{}.actorpack", clean_actor),
        format!("content/Actor/Pack/{}.actorpack", clean_actor),
        format!("romfs/Actor/Pack/{}.actorpack", clean_actor),
        format!("{}.actorpack", clean_actor),
    ];

    for base in search_dirs {
        for rel in &pack_rel_paths {
            let pack_path = base.join(rel);
            if pack_path.is_file() {
                if let Ok(pack_bytes) = std::fs::read(&pack_path) {
                    let uncompressed = roead::yaz0::decompress(&pack_bytes).unwrap_or(pack_bytes);
                    if let Ok(sarc) = Sarc::new(&uncompressed) {
                        for f in sarc.files() {
                            if f.name().is_some_and(|n| n.ends_with(".baiprog")) {
                                let _ = parse_baiprog(f.data(), &mut prog);
                            }
                        }
                    }
                }
            }
            if !prog.actions.is_empty() || !prog.queries.is_empty() {
                let mut write_guard = cache.actor_progs.write().unwrap_or_else(|p| p.into_inner());
                write_guard.insert(clean_actor.to_string(), prog.clone());
                return Ok(prog);
            }
        }
    }

    // 3. 查找 Pack/*.pack 内嵌的 sbactorpack (优先 TitleBG.pack，包含 EventSystemActor 与 GameROMPlayer)
    let big_packs = ["TitleBG.pack", "Bootup.pack", "Title.pack", "RemainsWind.pack", "RemainsElectric.pack", "RemainsWater.pack", "RemainsFire.pack"];
    for base in search_dirs {
        for p_name in &big_packs {
            for sub in ["Pack", "content/Pack", "romfs/Pack", ""] {
                let p_path = if sub.is_empty() { base.join(p_name) } else { base.join(sub).join(p_name) };
                if p_path.is_file() {
                    if let Ok(p_bytes) = std::fs::read(&p_path) {
                        let uncompressed = roead::yaz0::decompress(&p_bytes).unwrap_or(p_bytes);
                        if let Ok(big_sarc) = Sarc::new(&uncompressed) {
                            let target_sbactorpack = format!("{}.sbactorpack", clean_actor);
                            let target_actorpack = format!("{}.actorpack", clean_actor);
                            if let Some(actor_file) = big_sarc.files().find(|f| {
                                f.name().is_some_and(|n| n.ends_with(&target_sbactorpack) || n.ends_with(&target_actorpack))
                            }) {
                                let actor_uncompressed = roead::yaz0::decompress(actor_file.data()).unwrap_or_else(|_| actor_file.data().to_vec());
                                if let Ok(actor_sarc) = Sarc::new(&actor_uncompressed) {
                                    for f in actor_sarc.files() {
                                        if f.name().is_some_and(|n| n.ends_with(".baiprog")) {
                                            let _ = parse_baiprog(f.data(), &mut prog);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !prog.actions.is_empty() || !prog.queries.is_empty() {
                    let mut write_guard = cache.actor_progs.write().unwrap_or_else(|p| p.into_inner());
                    write_guard.insert(clean_actor.to_string(), prog.clone());
                    return Ok(prog);
                }
            }
        }
    }

    if !prog.actions.is_empty() || !prog.queries.is_empty() {
        let mut write_guard = cache.actor_progs.write().unwrap_or_else(|p| p.into_inner());
        write_guard.insert(clean_actor.to_string(), prog.clone());
        Ok(prog)
    } else {
        Err(format!("未能加载角色 {} 的 AI 程序", clean_actor))
    }
}
