use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::model::settings::{GamePathSettings, PlatformType};
use crate::model::GameLanguage;
use super::cache::DialogueSource;
use super::parser::{extract_msbt_from_bootup_pack, extract_msbt_from_msg_sarc, parse_msbt_entries};

/// 解析并加载目标对白文件的全量词条字典
pub fn resolve_and_load_dialogue_file(
    file_key: &str,
    language: GameLanguage,
    project_path: Option<&str>,
    game_paths: &GamePathSettings,
) -> Option<(HashMap<String, String>, DialogueSource, String)> {
    let targets = generate_target_names(file_key);
    let target_refs: Vec<&str> = targets.iter().map(|s| s.as_str()).collect();

    // 1. 最高优先级：从当前打开的项目目录中查找
    if let Some(proj_str) = project_path {
        let proj = Path::new(proj_str);
        if proj.exists() {
            if let Some((entries, path)) = search_in_directory(proj, &target_refs, language, 5) {
                return Some((entries, DialogueSource::Project, path));
            }
        }
    }

    // 2. 回退优先级：按软件设置的 Dump 路径查找
    let dump_dirs = collect_dump_search_dirs(game_paths);
    for dir in dump_dirs {
        if dir.exists() {
            if let Some((entries, path)) = search_in_directory(&dir, &target_refs, language, 4) {
                return Some((entries, DialogueSource::GameDump, path));
            }
        }
    }

    None
}

/// 根据输入的 file_key (如 "EventFlowMsg/TwnObj_AncientOven_A_01") 生成匹配候选文件名
fn generate_target_names(file_key: &str) -> Vec<String> {
    let normalized = file_key.replace('\\', "/").trim_start_matches('/').to_string();
    let base_name = normalized.split('/').next_back().unwrap_or(&normalized).to_string();

    let mut list = Vec::new();

    // 1. 完整相对路径 (如 EventFlowMsg/TwnObj_AncientOven_A_01.msbt)
    if !normalized.ends_with(".msbt") {
        list.push(format!("{}.msbt", normalized));
    }
    list.push(normalized);

    // 2. 单纯基础文件名 (如 TwnObj_AncientOven_A_01.msbt)
    if !base_name.ends_with(".msbt") {
        list.push(format!("{}.msbt", base_name));
    }
    if !list.contains(&base_name) {
        list.push(base_name);
    }

    list
}

/// 收集并排布设置中的所有游戏 Dump 根目录候选路径
fn collect_dump_search_dirs(gp: &GamePathSettings) -> Vec<PathBuf> {
    let mut primary_dirs = Vec::new();
    let mut fallback_dirs = Vec::new();

    match gp.current_platform {
        PlatformType::Switch => {
            let sw = &gp.switch;
            if !sw.game_dir.trim().is_empty() {
                primary_dirs.push(PathBuf::from(sw.game_dir.trim()));
            }
            if !sw.dlc_dir.trim().is_empty() {
                let dlc = sw.dlc_dir.trim();
                primary_dirs.push(PathBuf::from(dlc));
                if dlc.contains("01007EF00011F001") {
                    primary_dirs.push(PathBuf::from(dlc.replace("01007EF00011F001", "01007EF00011E000")));
                }
            }
            primary_dirs.push(PathBuf::from(r"C:\Game\BOTW\01007EF00011E000\romfs"));

            if !gp.wiiu.update_dir.trim().is_empty() {
                fallback_dirs.push(PathBuf::from(gp.wiiu.update_dir.trim()));
            }
            if !gp.wiiu.game_dir.trim().is_empty() {
                fallback_dirs.push(PathBuf::from(gp.wiiu.game_dir.trim()));
            }
        }
        PlatformType::WiiU => {
            if !gp.wiiu.update_dir.trim().is_empty() {
                primary_dirs.push(PathBuf::from(gp.wiiu.update_dir.trim()));
            }
            if !gp.wiiu.game_dir.trim().is_empty() {
                primary_dirs.push(PathBuf::from(gp.wiiu.game_dir.trim()));
            }
            if !gp.wiiu.dlc_dir.trim().is_empty() {
                primary_dirs.push(PathBuf::from(gp.wiiu.dlc_dir.trim()));
            }

            if !gp.switch.game_dir.trim().is_empty() {
                fallback_dirs.push(PathBuf::from(gp.switch.game_dir.trim()));
            }
            if !gp.switch.dlc_dir.trim().is_empty() {
                fallback_dirs.push(PathBuf::from(gp.switch.dlc_dir.trim()));
            }
        }
    }

    if !gp.custom_ai_dump_dir.trim().is_empty() {
        primary_dirs.push(PathBuf::from(gp.custom_ai_dump_dir.trim()));
    }

    let mut result = Vec::new();
    for d in primary_dirs.into_iter().chain(fallback_dirs) {
        if d.exists() && !result.contains(&d) {
            result.push(d);
        }
    }
    result
}

/// 在指定目录中递归查找松散 MSBT、Msg_*.ssarc 或 Bootup_<Lang>.pack
fn search_in_directory(
    root: &Path,
    targets: &[&str],
    language: GameLanguage,
    max_depth: usize,
) -> Option<(HashMap<String, String>, String)> {
    let (primary_pack, fallback_pack) = language.bootup_pack_names();
    let primary_msg = format!("Msg_{}.product.ssarc", language.code());

    let mut dirs_to_visit = vec![(root.to_path_buf(), 0)];
    let mut discovered_packs = Vec::new();
    let mut discovered_msg_sarcs = Vec::new();

    while let Some((dir, depth)) = dirs_to_visit.pop() {
        if depth > max_depth {
            continue;
        }

        let Ok(entries) = fs::read_dir(&dir) else { continue; };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue; };

            if path.is_dir() {
                if !name.starts_with('.') && name != "target" && name != "node_modules" && name != "map_tiles" {
                    dirs_to_visit.push((path, depth + 1));
                }
            } else if path.is_file() {
                // 1. 直寻松散 MSBT 文件
                for target in targets {
                    if name.eq_ignore_ascii_case(target) {
                        if let Ok(bytes) = fs::read(&path) {
                            let entries = parse_msbt_entries(&bytes);
                            if !entries.is_empty() {
                                return Some((entries, path.display().to_string()));
                            }
                        }
                    }
                }

                // 2. 发现 Msg_*.product.ssarc
                if name.eq_ignore_ascii_case(&primary_msg) || name.ends_with(".product.ssarc") {
                    discovered_msg_sarcs.push(path);
                }
                // 3. 发现 Bootup_<Lang>.pack
                else if name.eq_ignore_ascii_case(primary_pack)
                    || fallback_pack.is_some_and(|f| name.eq_ignore_ascii_case(f))
                {
                    discovered_packs.push(path);
                }
            }
        }
    }

    // 从发现的独立 Msg SARC 中提取
    for msg_path in discovered_msg_sarcs {
        if let Ok(bytes) = fs::read(&msg_path) {
            if let Some(entries) = extract_msbt_from_msg_sarc(&bytes, targets) {
                return Some((entries, msg_path.display().to_string()));
            }
        }
    }

    // 从发现的 Bootup_<Lang>.pack 中解压并提取
    for pack_path in discovered_packs {
        if let Ok(bytes) = fs::read(&pack_path) {
            if let Some(entries) = extract_msbt_from_bootup_pack(&bytes, targets) {
                return Some((entries, pack_path.display().to_string()));
            }
        }
    }

    None
}
