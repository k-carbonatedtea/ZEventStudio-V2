use roead::sarc::{Sarc, SarcWriter};

/// 递归从嵌套 SARC 结构中删除指定条目或虚拟子目录并返回重新打包后的字节流
pub(crate) fn delete_nested_sarc_entry(
    current_sarc_data: &[u8],
    remaining_paths: &[&str],
    container_name: &str,
) -> Result<Vec<u8>, String> {
    let is_yaz0 = current_sarc_data.len() >= 4 && &current_sarc_data[0..4] == b"Yaz0";
    let decompressed = if is_yaz0 {
        roead::yaz0::decompress(current_sarc_data)
            .map_err(|e| format!("解压封包失败: {:?}", e))?
    } else {
        current_sarc_data.to_vec()
    };

    let sarc = Sarc::new(&decompressed)
        .map_err(|e| format!("解析 SARC 封包结构失败: {:?}", e))?;
    let mut writer = SarcWriter::from_sarc(&sarc);

    let target_name = remaining_paths[0];
    let normalized_target = target_name.replace('\\', "/");
    let target_prefix = format!("{}/", normalized_target.trim_end_matches('/'));

    if remaining_paths.len() == 1 {
        // 查找精确文件名或以该虚拟目录为前缀的所有文件
        let keys_to_remove: Vec<String> = writer
            .files
            .keys()
            .filter(|k| {
                let norm = k.replace('\\', "/");
                norm == normalized_target
                    || norm.trim_start_matches('/') == normalized_target.trim_start_matches('/')
                    || norm.starts_with(&target_prefix)
                    || norm.trim_start_matches('/').starts_with(&target_prefix)
            })
            .cloned()
            .collect();

        if keys_to_remove.is_empty() {
            return Err(format!("未在封包中找到待删除项: {}", target_name));
        }

        for k in keys_to_remove {
            writer.files.shift_remove(&k);
        }
    } else {
        // 递归进入下一层子封包
        let child_file_data = sarc
            .files()
            .find_map(|f| {
                f.name().and_then(|n| {
                    if n == target_name || n.trim_start_matches('/') == normalized_target.trim_start_matches('/') {
                        Some(f.data().to_vec())
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| format!("未在封包中找到子封包: {}", target_name))?;

        let updated_child_bytes = delete_nested_sarc_entry(
            &child_file_data,
            &remaining_paths[1..],
            target_name,
        )?;

        let actual_child_name = sarc
            .files()
            .find_map(|f| {
                f.name().and_then(|n| {
                    if n == target_name || n.trim_start_matches('/') == normalized_target.trim_start_matches('/') {
                        Some(n.to_string())
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(normalized_target);

        writer.files.insert(actual_child_name, updated_child_bytes);
    }

    let packed_binary = writer.to_binary();
    let should_compress = is_yaz0 || crate::model::file_converter::should_compress_yaz0(container_name);
    if should_compress {
        Ok(roead::yaz0::compress(&packed_binary))
    } else {
        Ok(packed_binary)
    }
}

/// 递归重命名嵌套 SARC 结构中的指定条目并返回重新打包后的字节流
pub(crate) fn rename_nested_sarc_entry(
    current_sarc_data: &[u8],
    remaining_paths: &[&str],
    new_inner_path: &str,
    container_name: &str,
) -> Result<Vec<u8>, String> {
    let is_yaz0 = current_sarc_data.len() >= 4 && &current_sarc_data[0..4] == b"Yaz0";
    let decompressed = if is_yaz0 {
        roead::yaz0::decompress(current_sarc_data)
            .map_err(|e| format!("解压封包失败: {:?}", e))?
    } else {
        current_sarc_data.to_vec()
    };

    let sarc = Sarc::new(&decompressed)
        .map_err(|e| format!("解析 SARC 封包结构失败: {:?}", e))?;
    let mut writer = SarcWriter::from_sarc(&sarc);

    let target_name = remaining_paths[0];
    let normalized_target = target_name.replace('\\', "/");

    if remaining_paths.len() == 1 {
        let normalized_new = new_inner_path.replace('\\', "/");
        let target_prefix = format!("{}/", normalized_target.trim_end_matches('/'));
        let new_prefix = format!("{}/", normalized_new.trim_end_matches('/'));

        let mut matches = Vec::new();
        for (k, v) in &writer.files {
            let norm = k.replace('\\', "/");
            if norm == normalized_target || norm.trim_start_matches('/') == normalized_target.trim_start_matches('/') {
                matches.push((k.clone(), normalized_new.clone(), v.clone()));
            } else if norm.starts_with(&target_prefix) {
                let suffix = &norm[target_prefix.len()..];
                let replaced_new = format!("{}{}", new_prefix, suffix);
                matches.push((k.clone(), replaced_new, v.clone()));
            }
        }

        if matches.is_empty() {
            return Err(format!("未在封包中找到待重命名项: {}", target_name));
        }

        for (old_k, new_k, data) in matches {
            writer.files.shift_remove(&old_k);
            writer.files.insert(new_k, data);
        }
    } else {
        let child_file_data = sarc
            .files()
            .find_map(|f| {
                f.name().and_then(|n| {
                    if n == target_name || n.trim_start_matches('/') == normalized_target.trim_start_matches('/') {
                        Some(f.data().to_vec())
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| format!("未在封包中找到子封包: {}", target_name))?;

        let updated_child_bytes = rename_nested_sarc_entry(
            &child_file_data,
            &remaining_paths[1..],
            new_inner_path,
            target_name,
        )?;

        let actual_child_name = sarc
            .files()
            .find_map(|f| {
                f.name().and_then(|n| {
                    if n == target_name || n.trim_start_matches('/') == normalized_target.trim_start_matches('/') {
                        Some(n.to_string())
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(normalized_target);

        writer.files.insert(actual_child_name, updated_child_bytes);
    }

    let packed_binary = writer.to_binary();
    let should_compress = is_yaz0 || crate::model::file_converter::should_compress_yaz0(container_name);
    if should_compress {
        Ok(roead::yaz0::compress(&packed_binary))
    } else {
        Ok(packed_binary)
    }
}
