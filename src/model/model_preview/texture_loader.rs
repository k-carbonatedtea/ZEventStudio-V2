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

    // 提取模型文件主干名 (例如 "Item_Conductor.sbfres" -> "Item_Conductor")
    let stem = file_name
        .trim_end_matches(".sbfres")
        .trim_end_matches(".bfres");

    // 候选的配套纹理文件名称列表 (与 Switch-Toolbox 规范完全对齐)
    let candidates = [
        format!("{}.Tex.sbfres", stem),
        format!("{}.Tex1.sbfres", stem),
        format!("{}.Tex2.sbfres", stem),
        format!("{}.Tex.bfres", stem),
        format!("{}.Tex1.bfres", stem),
        format!("{}.Tex2.bfres", stem),
    ];

    for candidate_name in &candidates {
        let candidate_path = parent_dir.join(candidate_name);
        if candidate_path.exists() {
            if let Ok(bytes) = fs::read(&candidate_path) {
                let decomp = if bytes.len() >= 4 && &bytes[0..4] == b"Yaz0" {
                    roead::yaz0::decompress(&bytes).unwrap_or(bytes)
                } else {
                    bytes
                };

                if let Ok(tex_bfres) = BfresFile::from_bytes(&decomp) {
                    extract_bntx_from_bfres(&tex_bfres, state);
                }
            }
        }
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
        }
    }
}

