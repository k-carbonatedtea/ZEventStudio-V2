use roead::sarc::{Sarc, SarcWriter};
use std::fs;
use std::path::Path;

/// 将数据字节流保存写入至指定目标路径 (支持物理文件与多层 SARC 封包内虚拟路径)
pub fn write_file_to_target(path: &str, data: &[u8]) -> Result<(), String> {
    if path.starts_with("SARC:") {
        let clean = path.trim_start_matches("SARC:");
        let parts: Vec<&str> = clean.split("//").collect();
        if parts.is_empty() {
            return Err("无效的 SARC 文件路径".to_string());
        }

        let root_physical_path = parts[0];
        let root_bytes = fs::read(root_physical_path)
            .map_err(|e| format!("无法读取根封包文件 {}: {}", root_physical_path, e))?;

        let nested_paths = &parts[1..];
        if nested_paths.is_empty() {
            return Err("未指定封包内部文件路径".to_string());
        }

        let updated_root_bytes = update_nested_sarc_bytes(&root_bytes, nested_paths, data, root_physical_path)?;
        fs::write(root_physical_path, &updated_root_bytes)
            .map_err(|e| format!("保存写回封包文件失败: {}", e))?;

        Ok(())
    } else {
        // 物理路径直接写入
        if let Some(parent) = Path::new(path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(path, data).map_err(|e| format!("写入文件失败: {}", e))?;
        Ok(())
    }
}

/// 递归更新嵌套 SARC 结构内部的指定文件并返回重新打包后的字节流
fn update_nested_sarc_bytes(
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
    let mut writer = SarcWriter::from(&sarc);

    let target_name = remaining_paths[0];
    let normalized_target = target_name.replace('\\', "/");

    if remaining_paths.len() == 1 {
        // 最后一层：直接将修改后的目标文件写入 SarcWriter
        // 尝试匹配带前导斜杠或规范路径
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

        writer.add_file(&actual_entry_name, payload.to_vec());
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

        writer.add_file(&actual_child_name, updated_child_bytes);
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

#[cfg(test)]
mod tests {
    use super::*;
    use roead::Endian;

    #[test]
    fn test_sarc_update_single_file() {
        let mut writer = SarcWriter::new(Endian::Little);
        writer.add_file("Actor/Armor.byml", b"original_data".to_vec());
        let sarc_bin = writer.to_binary();

        let updated_bin = update_nested_sarc_bytes(
            &sarc_bin,
            &["Actor/Armor.byml"],
            b"modified_data",
            "test.pack",
        ).unwrap();

        let parsed_sarc = Sarc::new(&updated_bin).unwrap();
        let file_data = parsed_sarc.get_data("Actor/Armor.byml").unwrap();
        assert_eq!(file_data, b"modified_data");
    }

    #[test]
    fn test_sarc_update_nested_sarc() {
        // 创建内层 SARC
        let mut inner_writer = SarcWriter::new(Endian::Little);
        inner_writer.add_file("Message/ArmorHead.msbt", b"inner_msbt".to_vec());
        let inner_bin = inner_writer.to_binary();

        // 创建外层 SARC 并包含内层 SARC
        let mut outer_writer = SarcWriter::new(Endian::Little);
        outer_writer.add_file("Msg.ssarc", inner_bin);
        let outer_bin = outer_writer.to_binary();

        // 递归修改外层 SARC 内部的内层 SARC 中的 MSBT
        let updated_outer = update_nested_sarc_bytes(
            &outer_bin,
            &["Msg.ssarc", "Message/ArmorHead.msbt"],
            b"new_msbt_content",
            "Bootup.pack",
        ).unwrap();

        let parsed_outer = Sarc::new(&updated_outer).unwrap();
        let inner_bytes = parsed_outer.get_data("Msg.ssarc").unwrap();
        let decompressed_inner = roead::yaz0::decompress(inner_bytes).unwrap_or_else(|_| inner_bytes.to_vec());
        let parsed_inner = Sarc::new(&decompressed_inner).unwrap();
        let target_data = parsed_inner.get_data("Message/ArmorHead.msbt").unwrap();
        assert_eq!(target_data, b"new_msbt_content");
    }
}
