//! 配套纹理包 (.Tex.sbfres / .bntx) 自动关联与解码加载器

use super::types::RgbaTexture;
use super::ModelDocumentState;
use rbfres_library::texture::BntxFile;
use rbfres_library::BfresFile;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// 自动探测并加载模型的内嵌纹理与同目录下的配套纹理包 (.Tex.sbfres / .Tex1.sbfres)
pub fn load_companion_textures(
    file_path: &str,
    bfres: &BfresFile,
    state: &mut ModelDocumentState,
) {
    // 1. 尝试从模型自身的外部文件字典 (ExternalFiles) 提取内嵌的 BNTX
    extract_bntx_from_bfres(bfres, state);

    // 2. 检查物理磁盘同级目录下是否存在配套的 .Tex.sbfres / .Tex1.sbfres
    let path_obj = Path::new(file_path);
    if !path_obj.exists() {
        return;
    }

    let parent_dir = match path_obj.parent() {
        Some(p) => p,
        None => return,
    };

    let file_name = path_obj
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default();

    // 提取模型文件主干名 (使用 strip_suffix 避免字符集误伤)
    let stem = file_name
        .strip_suffix(".sbfres")
        .or_else(|| file_name.strip_suffix(".bfres"))
        .unwrap_or(file_name);

    // 候选的配套纹理文件名称列表 (包含专用纹理包与 Link/Armor 通用基础贴图包)
    let mut candidates = vec![
        format!("{}.Tex.sbfres", stem),
        format!("{}.Tex1.sbfres", stem),
        format!("{}.Tex2.sbfres", stem),
        format!("{}.Tex.bfres", stem),
        format!("{}.Tex1.bfres", stem),
        format!("{}.Tex2.bfres", stem),
    ];

    // 若当前为服装/盔甲模型，追加探测主角基础身体贴图包
    if stem.starts_with("Armor_") || stem.starts_with("Npc_") {
        candidates.push("Link.Tex.sbfres".to_string());
        candidates.push("Link.Tex1.sbfres".to_string());
        candidates.push("Armor_Default.Tex.sbfres".to_string());
    }

    for candidate_name in &candidates {
        let candidate_path = parent_dir.join(candidate_name);
        load_tex_file_if_exists(&candidate_path, state);
    }

    // 若当前为服装/盔甲模型且缺少通用贴图，追加探测用户配置的游戏原版 Model 目录
    if stem.starts_with("Armor_") || stem.starts_with("Npc_") {
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
}

/// 探测并加载指定路径的纹理资源文件 (支持 Yaz0 压缩、BFRES 归档与纯 BNTX)
fn load_tex_file_if_exists(file_path: &Path, state: &mut ModelDocumentState) {
    if !file_path.exists() {
        return;
    }
    let Ok(bytes) = fs::read(file_path) else {
        return;
    };
    let decomp = if bytes.len() >= 4 && &bytes[0..4] == b"Yaz0" {
        roead::yaz0::decompress(&bytes).unwrap_or(bytes)
    } else {
        bytes
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

