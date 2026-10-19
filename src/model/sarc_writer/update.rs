use roead::sarc::{Sarc, SarcWriter};

/// 递归更新嵌套 SARC 结构内部的指定文件并返回重新打包后的字节流
pub(crate) fn update_nested_sarc_bytes(
    current_sarc_data: &[u8],
    remaining_paths: &[&str],
    payload: &[u8],
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
        // 最后一层：直接将修改后的目标文件写入 SarcWriter
        let actual_entry_name = sarc
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
            .unwrap_or_else(|| normalized_target.clone());

        writer.files.insert(actual_entry_name, payload.to_vec());
    } else {
        // 递归进入下一层嵌套 SARC 封包
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

        let updated_child_bytes = update_nested_sarc_bytes(
            &child_file_data,
            &remaining_paths[1..],
            payload,
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

    // 检查是否需要重新进行 Yaz0 压缩
    let should_compress = is_yaz0 || crate::model::file_converter::should_compress_yaz0(container_name);
    if should_compress {
        Ok(roead::yaz0::compress(&packed_binary))
    } else {
        Ok(packed_binary)
    }
}
