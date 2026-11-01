//! 从 Actor 配置与 bmodellist 资源中解析与定位 3D 模型的算法核心

use std::fs;
use std::path::{Path, PathBuf};
use roead::sarc::Sarc;

use crate::model::settings::GamePathSettings;

/// 从 Actor 配置或 bmodellist 解析出的目标 3D 模型元数据
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorModelTarget {
    /// 模型资源目录与主文件名 (对应 Model/{folder}.sbfres)
    pub folder: String,
    /// Actor 指定绑定的子单元名称列表 (例如 ["Armor_017_Head"])
    pub unit_names: Vec<String>,
    /// 所属 Actor 语义名称
    pub actor_name: String,
}

/// 从代码编辑器文本或当前文件路径探测并提炼 Actor 模型配置目标
pub fn detect_actor_model_target(file_path: &str, content: &str) -> Option<ActorModelTarget> {
    let lower_path = file_path.to_lowercase();

    // 1. 若当前文件本身为 bmodellist (无论是否在 SARC 内部)
    if lower_path.ends_with(".bmodellist") || lower_path.ends_with(".sbmodellist") {
        let (folder, unit_names) = extract_model_from_yaml(content)?;
        let actor_name = extract_actor_name_from_path(file_path);
        return Some(ActorModelTarget {
            folder,
            unit_names,
            actor_name,
        });
    }

    // 2. 若当前文件位于 .sbactorpack / .actorpack 封包内 (如 ActorLink.bxml 等其他子文件)
    if file_path.starts_with("SARC:") && (lower_path.contains(".sbactorpack") || lower_path.contains(".actorpack")) {
        let archive_path = file_path.trim_start_matches("SARC:").split("//").next()?;
        if let Ok(target) = extract_target_from_sbactorpack(archive_path) {
            return Some(target);
        }
    }

    // 3. 兜底扫描：若内容中同时存在 ModelData 与 Folder 字段特征
    if content.contains("ModelData") && content.contains("Folder:") {
        let (folder, unit_names) = extract_model_from_yaml(content)?;
        let actor_name = extract_actor_name_from_path(file_path);
        return Some(ActorModelTarget {
            folder,
            unit_names,
            actor_name,
        });
    }

    None
}

/// 从 YAML 或 AAMP 文本中解析 Base.Folder 与 Unit.UnitName 字段
pub fn extract_model_from_yaml(content: &str) -> Option<(String, Vec<String>)> {
    let mut folder: Option<String> = None;
    let mut unit_names: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Folder:") {
            let raw_value = trimmed.strip_prefix("Folder:").unwrap_or("");
            if let Some(val) = extract_clean_scalar(raw_value) {
                if !val.is_empty() && folder.is_none() {
                    folder = Some(val);
                }
            }
        } else if trimmed.starts_with("UnitName:") {
            let raw_value = trimmed.strip_prefix("UnitName:").unwrap_or("");
            if let Some(val) = extract_clean_scalar(raw_value) {
                if !val.is_empty() && !unit_names.contains(&val) {
                    unit_names.push(val);
                }
            }
        }
    }

    folder.map(|f| (f, unit_names))
}

/// 清洗提取 YAML 标量值 (剥离 !str64, !str32 标签及单双引号)
fn extract_clean_scalar(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let value_part = if trimmed.starts_with('!') {
        trimmed.split_whitespace().nth(1).unwrap_or("")
    } else {
        trimmed
    };
    let cleaned = value_part.trim_matches(|c| c == '\'' || c == '"' || c == ' ');
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

/// 从路径中提炼 Actor 的语义名称 (剥离后缀与目录层级)
fn extract_actor_name_from_path(file_path: &str) -> String {
    let clean_path = file_path.replace('\\', "/");
    let file_name = clean_path.rsplit('/').next().unwrap_or(file_path);
    let stem = file_name
        .strip_suffix(".bmodellist")
        .or_else(|| file_name.strip_suffix(".sbmodellist"))
        .or_else(|| file_name.strip_suffix(".sbactorpack"))
        .or_else(|| file_name.strip_suffix(".actorpack"))
        .unwrap_or(file_name);

    if !stem.is_empty() {
        return stem.to_string();
    }

    // 若当前在 SARC 内部，尝试从外层封包提取
    if let Some(archive) = clean_path.split("//").next() {
        let arc_name = archive.rsplit('/').next().unwrap_or(archive);
        let arc_stem = arc_name
            .strip_suffix(".sbactorpack")
            .or_else(|| arc_name.strip_suffix(".actorpack"))
            .unwrap_or(arc_name);
        if !arc_stem.is_empty() {
            return arc_stem.to_string();
        }
    }

    "Actor".to_string()
}

/// 打开物理 .sbactorpack 并解析内置 ModelList/*.bmodellist 提取目标
fn extract_target_from_sbactorpack(archive_path: &str) -> Result<ActorModelTarget, String> {
    let raw_bytes = fs::read(archive_path).map_err(|e| e.to_string())?;
    let decompressed = roead::yaz0::decompress(&raw_bytes).unwrap_or(raw_bytes);
    let sarc = Sarc::new(&decompressed).map_err(|e| e.to_string())?;

    for file in sarc.files() {
        if let Some(name) = file.name() {
            if name.ends_with(".bmodellist") || name.ends_with(".sbmodellist") {
                let file_data = file.data();
                let inner_uncomp = roead::yaz0::decompress(file_data).unwrap_or_else(|_| file_data.to_vec());
                let text = if inner_uncomp.starts_with(b"AAMP") {
                    let pio = roead::aamp::ParameterIO::from_binary(&inner_uncomp)
                        .map_err(|e| format!("{:?}", e))?;
                    pio.to_text()
                } else {
                    String::from_utf8_lossy(&inner_uncomp).to_string()
                };

                if let Some((folder, unit_names)) = extract_model_from_yaml(&text) {
                    let actor_name = extract_actor_name_from_path(archive_path);
                    return Ok(ActorModelTarget {
                        folder,
                        unit_names,
                        actor_name,
                    });
                }
            }
        }
    }

    Err("未在封包中找到 ModelList 配置".to_string())
}

/// 定位目标 3D 模型 (.sbfres / .bfres) 在物理磁盘上的具体路径
pub fn locate_actor_model_file(
    target: &ActorModelTarget,
    source_path: &str,
    project_root: Option<&str>,
) -> Option<PathBuf> {
    let mut candidate_filenames = Vec::new();
    candidate_filenames.push(format!("{}.sbfres", target.folder));
    candidate_filenames.push(format!("{}.bfres", target.folder));
    for unit in &target.unit_names {
        candidate_filenames.push(format!("{}.sbfres", unit));
        candidate_filenames.push(format!("{}.bfres", unit));
    }
    candidate_filenames.push(format!("{}.sbfres", target.actor_name));
    candidate_filenames.push(format!("{}.bfres", target.actor_name));

    let mut search_dirs = Vec::new();

    // 1. 项目根目录及常见资源子目录
    if let Some(root_str) = project_root {
        let root = Path::new(root_str);
        search_dirs.push(root.join("Model"));
        search_dirs.push(root.join("Romfs").join("Model"));
        search_dirs.push(root.join("romfs").join("Model"));
        search_dirs.push(root.join("content").join("Model"));
        search_dirs.push(root.join("Actor").join("Pack"));
        search_dirs.push(root.to_path_buf());
    }

    // 2. 从源文件路径逐级向上探测父级目录
    let clean_src = source_path.trim_start_matches("SARC:").split("//").next().unwrap_or(source_path);
    if let Some(mut curr) = Path::new(clean_src).parent() {
        for _ in 0..5 {
            search_dirs.push(curr.join("Model"));
            search_dirs.push(curr.join("Romfs").join("Model"));
            search_dirs.push(curr.join("romfs").join("Model"));
            search_dirs.push(curr.join("content").join("Model"));
            search_dirs.push(curr.to_path_buf());
            if let Some(p) = curr.parent() {
                curr = p;
            } else {
                break;
            }
        }
    }

    // 3. 用户全局设置的游戏 RomFS 根目录
    let settings = GamePathSettings::load_from_disk();
    if !settings.switch.game_dir.is_empty() {
        search_dirs.push(Path::new(&settings.switch.game_dir).join("Model"));
        search_dirs.push(Path::new(&settings.switch.game_dir).to_path_buf());
    }
    if !settings.switch.dlc_dir.is_empty() {
        search_dirs.push(Path::new(&settings.switch.dlc_dir).join("Model"));
    }
    if !settings.wiiu.game_dir.is_empty() {
        search_dirs.push(Path::new(&settings.wiiu.game_dir).join("Model"));
    }
    if !settings.wiiu.update_dir.is_empty() {
        search_dirs.push(Path::new(&settings.wiiu.update_dir).join("Model"));
    }
    if !settings.wiiu.dlc_dir.is_empty() {
        search_dirs.push(Path::new(&settings.wiiu.dlc_dir).join("Model"));
    }

    // 顺序检索首个存在的物理模型文件
    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        for name in &candidate_filenames {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::actor_loader::load_actor_model_document;

    #[test]
    fn test_extract_model_from_yaml() {
        let yaml = r#"
Attention: !obj
  IsEnableAttention: true
lists:
  ModelData: !list
    objects: {}
    lists:
      ModelData_0: !list
        objects:
          Base: !obj
            Folder: !str64 Armor_017
        lists:
          Unit: !list
            objects:
              Unit_0: !obj
                UnitName: !str64 Armor_017_Head
                BindBone: !str64 ''
"#;
        let res = extract_model_from_yaml(yaml);
        assert!(res.is_some());
        let (folder, units) = res.unwrap();
        assert_eq!(folder, "Armor_017");
        assert_eq!(units, vec!["Armor_017_Head"]);
    }

    #[test]
    fn test_detect_actor_model_target() {
        let file_path = "SARC:C:/Game/Armor_017_Head.sbactorpack//Actor/ModelList/Armor_017_Head.bmodellist";
        let content = "Base: !obj\n  Folder: !str64 Armor_017\nUnit_0: !obj\n  UnitName: !str64 Armor_017_Head";
        let target = detect_actor_model_target(file_path, content);
        assert!(target.is_some());
        let t = target.unwrap();
        assert_eq!(t.folder, "Armor_017");
        assert_eq!(t.unit_names, vec!["Armor_017_Head"]);
        assert_eq!(t.actor_name, "Armor_017_Head");
    }

    #[test]
    fn test_locate_and_load_armor_017() {
        let target = ActorModelTarget {
            folder: "Armor_017".to_string(),
            unit_names: vec!["Armor_017_Head".to_string()],
            actor_name: "Armor_017_Head".to_string(),
        };
        let file_path = "SARC:C:/Game/BOTW/01007EF00011E000/romfs/Actor/Pack/Armor_017_Head.sbactorpack//Actor/ModelList/Armor_017_Head.bmodellist";
        let found = locate_actor_model_file(&target, file_path, None);
        if let Some(path) = found {
            assert!(path.exists());
            let doc = load_actor_model_document(&target, &path);
            assert!(doc.is_ok());
            let state = doc.unwrap();
            assert_eq!(state.visible_meshes_count(), 4); // Only the 4 head meshes visible
            assert_eq!(state.mesh_items.len(), 12); // All 12 meshes loaded
        }
    }
}
