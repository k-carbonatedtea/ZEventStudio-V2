//! 配套纹理包 (.Tex.sbfres / .bntx) 自动关联与解码加载器

use super::types::RgbaTexture;
use super::ModelDocumentState;
use rbfres_library::texture::BntxFile;
use rbfres_library::BfresFile;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// 自动探测并加载模型的内嵌纹理与配套纹理包 (.Tex.sbfres / .Tex1.sbfres / .bntx)
pub fn load_companion_textures(
    file_path: &str,
    bfres: &BfresFile,
    state: &mut ModelDocumentState,
) {
    // 1. 尝试从模型自身的外部文件字典 (ExternalFiles) 提取内嵌的 BNTX
    extract_bntx_from_bfres(bfres, state);

    // 2. 根据虚拟 SARC 路径或物理磁盘路径分发配套纹理探测
    if file_path.starts_with("SARC:") {
        load_sarc_companion_textures(file_path, state);
    } else {
        load_disk_companion_textures(file_path, state);
    }
}

/// 构建配套纹理候选文件名列表
fn build_candidate_filenames(stem: &str) -> Vec<String> {
    let mut candidates = vec![
        format!("{}.Tex.sbfres", stem),
        format!("{}.Tex1.sbfres", stem),
        format!("{}.Tex2.sbfres", stem),
        format!("{}.Tex.bfres", stem),
        format!("{}.Tex1.bfres", stem),
        format!("{}.Tex2.bfres", stem),
        format!("{}.bntx", stem),
        format!("{}.Tex.bntx", stem),
    ];

    // 若当前为服装/盔甲/NPC模型，追加探测主角基础身体贴图包
    if stem.starts_with("Armor_") || stem.starts_with("Npc_") {
        candidates.push("Link.Tex.sbfres".to_string());
        candidates.push("Link.Tex1.sbfres".to_string());
        candidates.push("Armor_Default.Tex.sbfres".to_string());
    }

    candidates
}

/// 构建 SARC 封包内部可能的纹理路径列表
fn build_sarc_candidate_paths(inner_dir: &str, candidate_names: &[String]) -> Vec<String> {
    let mut cand_paths = Vec::with_capacity(candidate_names.len() * 3);
    for cand_name in candidate_names {
        if !inner_dir.is_empty() {
            cand_paths.push(format!("{}/{}", inner_dir, cand_name));
        }
        cand_paths.push(cand_name.clone());
        if inner_dir != "Model" {
            cand_paths.push(format!("Model/{}", cand_name));
        }
    }
    cand_paths
}

/// 顺着 SARC 嵌套链展开提取目标最深层 SARC 封包的二进制数据
fn extract_innermost_sarc_bytes(archive_bytes: &[u8], intermediates: &[&str]) -> Option<Vec<u8>> {
    let mut current = archive_bytes.to_vec();
    for intermediate in intermediates {
        let decomp = if current.len() >= 4 && &current[0..4] == b"Yaz0" {
            roead::yaz0::decompress(&current).ok()?
        } else {
            current
        };
        let sarc = roead::sarc::Sarc::new(&decomp).ok()?;
        let norm = intermediate.replace('\\', "/");
        let found = sarc.files().find(|f| {
            f.name().is_some_and(|name| name == *intermediate || name.replace('\\', "/") == norm)
        })?;
        current = found.data().to_vec();
    }
    Some(current)
}

/// 从 SARC 归档封包内部加载关联的配套纹理
fn load_sarc_companion_textures(virtual_path: &str, state: &mut ModelDocumentState) {
    let parts: Vec<&str> = virtual_path.trim_start_matches("SARC:").split("//").collect();
    if parts.len() < 2 {
        return;
    }
    let physical_pack_path = parts[0];
    let inner_model_path = parts[parts.len() - 1];

    let Ok(archive_bytes) = fs::read(physical_pack_path) else {
        return;
    };

    let Some(inner_sarc_bytes) = extract_innermost_sarc_bytes(&archive_bytes, &parts[1..parts.len() - 1]) else {
        return;
    };

    let decompressed = if inner_sarc_bytes.len() >= 4 && &inner_sarc_bytes[0..4] == b"Yaz0" {
        roead::yaz0::decompress(&inner_sarc_bytes).unwrap_or(inner_sarc_bytes)
    } else {
        inner_sarc_bytes
    };

    let Ok(sarc) = roead::sarc::Sarc::new(&decompressed) else {
        return;
    };

    let inner_norm = inner_model_path.replace('\\', "/");
    let (inner_dir, file_name) = match inner_norm.rfind('/') {
        Some(idx) => (&inner_norm[..idx], &inner_norm[idx + 1..]),
        None => ("", inner_norm.as_str()),
    };

    let stem = file_name
        .strip_suffix(".sbfres")
        .or_else(|| file_name.strip_suffix(".bfres"))
        .unwrap_or(file_name);

    let candidate_names = build_candidate_filenames(stem);
    let cand_paths = build_sarc_candidate_paths(inner_dir, &candidate_names);

    let mut loaded_files = HashSet::new();

    // 1. 优先在 SARC 内部匹配候选列表中的配套纹理包
    for cand_path in &cand_paths {
        let cand_norm = cand_path.replace('\\', "/");
        for file in sarc.files() {
            let Some(name) = file.name() else { continue };
            if (name == cand_path || name.replace('\\', "/") == cand_norm)
                && loaded_files.insert(name.to_string())
            {
                load_tex_bytes(file.data(), state);
            }
        }
    }

    // 2. 匹配并加载 SARC 内部任意独立 BNTX 纹理包
    for file in sarc.files() {
        let Some(name) = file.name() else { continue };
        if name.ends_with(".bntx") && loaded_files.insert(name.to_string()) {
            load_tex_bytes(file.data(), state);
        }
    }

    // 3. 检查物理封包所在同级磁盘目录（防止用户解包散放）
    if let Some(parent_dir) = Path::new(physical_pack_path).parent() {
        for cand_name in &candidate_names {
            let candidate_path = parent_dir.join(cand_name);
            load_tex_file_if_exists(&candidate_path, state);
        }
    }

    // 4. 服装/NPC通用贴图回退至游戏 Model 目录
    if stem.starts_with("Armor_") || stem.starts_with("Npc_") {
        load_romfs_fallback_textures(state);
    }
}

/// 从物理磁盘同级目录下加载配套纹理包
fn load_disk_companion_textures(file_path: &str, state: &mut ModelDocumentState) {
    let path_obj = Path::new(file_path);
    if !path_obj.exists() {
        return;
    }

    let Some(parent_dir) = path_obj.parent() else {
        return;
    };

    let file_name = path_obj
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default();

    let stem = file_name
        .strip_suffix(".sbfres")
        .or_else(|| file_name.strip_suffix(".bfres"))
        .unwrap_or(file_name);

    let candidates = build_candidate_filenames(stem);

    for candidate_name in &candidates {
        let candidate_path = parent_dir.join(candidate_name);
        load_tex_file_if_exists(&candidate_path, state);
    }

    if stem.starts_with("Armor_") || stem.starts_with("Npc_") {
        load_romfs_fallback_textures(state);
    }
}

/// 探测用户配置的游戏原版 Model 目录中的通用贴图包
fn load_romfs_fallback_textures(state: &mut ModelDocumentState) {
    let settings = crate::model::settings::GamePathSettings::load_from_disk();
    let fallback_dirs = [
        Path::new(&settings.switch.game_dir).join("Model"),
        Path::new(&settings.wiiu.game_dir).join("Model"),
        Path::new(&settings.wiiu.update_dir).join("Model"),
    ];

    for game_model_dir in &fallback_dirs {
        if game_model_dir.exists() {
            for candidate_name in &["Link.Tex.sbfres", "Link.Tex1.sbfres", "Armor_Default.Tex.sbfres"] {
                let candidate_path = game_model_dir.join(candidate_name);
                load_tex_file_if_exists(&candidate_path, state);
            }
        }
    }
}

/// 探测并加载指定路径的物理磁盘纹理文件
fn load_tex_file_if_exists(file_path: &Path, state: &mut ModelDocumentState) {
    if !file_path.exists() {
        return;
    }
    if let Ok(bytes) = fs::read(file_path) {
        load_tex_bytes(&bytes, state);
    }
}

/// 解析任意二进制字节流中的纹理数据 (支持 Yaz0 压缩、BFRES 归档与纯 BNTX)
pub fn load_tex_bytes(bytes: &[u8], state: &mut ModelDocumentState) {
    let decomp = if bytes.len() >= 4 && &bytes[0..4] == b"Yaz0" {
        roead::yaz0::decompress(bytes).unwrap_or_else(|_| bytes.to_vec())
    } else {
        bytes.to_vec()
    };

    // 尝试直接作为 BNTX 解码
    if decomp.len() >= 4 && &decomp[0..4] == b"BNTX" {
        if let Ok(bntx) = BntxFile::from_bytes(&decomp) {
            decode_and_insert_bntx(bntx, state);
            return;
        }
    }

    // 尝试作为 BFRES 归档解包其内部外部文件
    if let Ok(tex_bfres) = BfresFile::from_bytes(&decomp) {
        extract_bntx_from_bfres(&tex_bfres, state);
    }
}

/// 从 BFRES 归档的 ExternalFiles 中提取并解码全部 BNTX 纹理
fn extract_bntx_from_bfres(bfres: &BfresFile, state: &mut ModelDocumentState) {
    for ext_file in bfres.external_files.values() {
        if ext_file.data.len() < 32 {
            continue;
        }

        // 识别魔数为 "BNTX" 的纹理封包
        if &ext_file.data[0..4] == b"BNTX" {
            if let Ok(bntx) = BntxFile::from_bytes(&ext_file.data) {
                decode_and_insert_bntx(bntx, state);
            }
        }
    }
}

/// 解码 BNTX 文件中的所有纹理切片并插入模型文档纹理池
fn decode_and_insert_bntx(bntx: BntxFile, state: &mut ModelDocumentState) {
    for tex in bntx.textures {
        if state.textures.contains_key(&tex.name) {
            continue;
        }
        let rgba = tex.decode_rgba();
        if !rgba.is_empty() {
            let rgba_tex = RgbaTexture {
                width: tex.width,
                height: tex.height,
                rgba: Arc::new(rgba),
            };
            state.textures.insert(tex.name.clone(), Arc::new(rgba_tex));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_filenames() {
        let candidates = build_candidate_filenames("DgnMrgPrt_Dungeon085");
        assert!(candidates.contains(&"DgnMrgPrt_Dungeon085.Tex.sbfres".to_string()));
        assert!(candidates.contains(&"DgnMrgPrt_Dungeon085.Tex1.sbfres".to_string()));
        assert!(candidates.contains(&"DgnMrgPrt_Dungeon085.bntx".to_string()));

        let armor_cands = build_candidate_filenames("Armor_006");
        assert!(armor_cands.contains(&"Link.Tex.sbfres".to_string()));
    }

    #[test]
    fn test_sarc_candidate_paths() {
        let names = vec!["Test.Tex.sbfres".to_string()];
        let paths = build_sarc_candidate_paths("Model", &names);
        assert!(paths.contains(&"Model/Test.Tex.sbfres".to_string()));
        assert!(paths.contains(&"Test.Tex.sbfres".to_string()));
    }

    #[test]
    fn test_load_sarc_dungeon085() {
        let pack_path = r"C:\Game\BOTW\01007EF00011E000\romfs\Pack\Dungeon085.pack";
        if !Path::new(pack_path).exists() {
            return;
        }
        let virtual_path = format!("SARC:{}//Model/DgnMrgPrt_Dungeon085.sbfres", pack_path);
        let bfres = BfresFile::default();
        let mut state = ModelDocumentState::from_bfres(&virtual_path, "DgnMrgPrt_Dungeon085.sbfres", &bfres);
        load_companion_textures(&virtual_path, &bfres, &mut state);

        assert!(!state.textures.is_empty(), "Dungeon085 companion textures should be loaded from SARC!");
        println!("Loaded textures count from Dungeon085: {}", state.textures.len());
        assert!(state.textures.contains_key("CmnTex_Cdun_MonyoLine_B_Alb"));
    }
}
