use roead::sarc::Sarc;
use std::fs;
use std::path::Path;

/// 将 SARC 封包或封包内部子目录解包导出到指定的目标物理目录中
/// 返回成功解包导出的文件数量
pub fn extract_sarc_to_dir(path: &str, dest_dir: &Path) -> Result<usize, String> {
    let (physical_path, inner_chain) = if path.starts_with("SARC:") {
        let clean = path.trim_start_matches("SARC:");
        let parts: Vec<&str> = clean.split("//").collect();
        if parts.is_empty() {
            return Err("无效的 SARC 文件路径".to_string());
        }
        (parts[0].to_string(), parts[1..].to_vec())
    } else {
        (path.to_string(), Vec::new())
    };

    let file_bytes = fs::read(&physical_path)
        .map_err(|e| format!("无法读取封包文件 {}: {}", physical_path, e))?;

    let mut current_bytes = roead::yaz0::decompress(&file_bytes).unwrap_or(file_bytes);
    let mut filter_prefix = String::new();

    for (idx, segment) in inner_chain.iter().enumerate() {
        let is_last = idx == inner_chain.len() - 1;
        // 尝试看看当前 segment 是否为一个可进一步深入的嵌套子封包
        let maybe_sub_bytes = if let Ok(sarc) = Sarc::new(&current_bytes) {
            let found = sarc.files().find(|f| {
                if let Some(name) = f.name() {
                    name == *segment || name.replace('\\', "/") == segment.replace('\\', "/")
                } else {
                    false
                }
            });
            if let Some(file) = found {
                let data = file.data();
                let decomp = roead::yaz0::decompress(data).unwrap_or_else(|_| data.to_vec());
                if Sarc::new(&decomp).is_ok() {
                    Some(decomp)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(nested_bytes) = maybe_sub_bytes {
            current_bytes = nested_bytes;
            continue;
        }

        // 否则该 segment 及其后续是虚拟目录前缀
        if is_last {
            filter_prefix = segment.replace('\\', "/");
        } else {
            filter_prefix = inner_chain[idx..].join("/").replace('\\', "/");
            break;
        }
    }

    let sarc = Sarc::new(&current_bytes)
        .map_err(|e| format!("解析目标 SARC 封包失败: {}", e))?;

    let filter_clean = filter_prefix.trim_matches('/').to_string();
    let mut exported_count = 0;

    for file in sarc.files() {
        if let Some(raw_name) = file.name() {
            let normalized_name = raw_name.replace('\\', "/");
            let relative_name = if filter_clean.is_empty() {
                normalized_name.clone()
            } else if normalized_name == filter_clean {
                // 单个文件匹配
                let file_name = Path::new(&normalized_name)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("exported_file");
                file_name.to_string()
            } else if normalized_name.starts_with(&format!("{}/", filter_clean)) {
                // 位于子目录下
                normalized_name[filter_clean.len() + 1..].to_string()
            } else {
                continue;
            };

            let out_file_path = dest_dir.join(&relative_name);
            if let Some(parent) = out_file_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败 {}: {}", parent.display(), e))?;
            }

            fs::write(&out_file_path, file.data())
                .map_err(|e| format!("写入文件失败 {}: {}", out_file_path.display(), e))?;

            exported_count += 1;
        }
    }

    Ok(exported_count)
}

/// 递归复制普通物理文件夹内容到目标目录
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<usize, String> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    }

    let mut count = 0;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        let dest_path = dst.join(entry.file_name());

        if ty.is_dir() {
            count += copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path).map_err(|e| e.to_string())?;
            count += 1;
        }
    }
    Ok(count)
}
