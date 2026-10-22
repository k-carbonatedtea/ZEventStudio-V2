use roead::sarc::SarcWriter;
use roead::Endian;
use std::fs;
use std::path::Path;

/// 将物理文件夹内的全部文件递归收集并打包为 SARC 二进制字节流
#[allow(dead_code)]
pub fn pack_dir_to_sarc_bytes(
    source_dir: &Path,
    endian: Endian,
    compress_yaz0: bool,
) -> Result<Vec<u8>, String> {
    if !source_dir.is_dir() {
        return Err(format!("源路径不是有效物理目录: {}", source_dir.display()));
    }
    let mut writer = SarcWriter::new(endian);
    collect_files_recursively(source_dir, source_dir, &mut writer)?;
    let packed_binary = writer.to_binary();
    if compress_yaz0 {
        Ok(roead::yaz0::compress(&packed_binary))
    } else {
        Ok(packed_binary)
    }
}

/// 递归扫描目录并将文件相对路径与二进制字节写入 SarcWriter
fn collect_files_recursively(
    root: &Path,
    current: &Path,
    writer: &mut SarcWriter,
) -> Result<(), String> {
    let entries = fs::read_dir(current)
        .map_err(|e| format!("无法读取目录 {}: {}", current.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursively(root, &path, writer)?;
        } else {
            let relative_path = path
                .strip_prefix(root)
                .map_err(|e| format!("计算相对路径失败 {}: {}", path.display(), e))?
                .to_string_lossy()
                .replace('\\', "/");
            let data = fs::read(&path)
                .map_err(|e| format!("读取文件字节失败 {}: {}", path.display(), e))?;
            writer.files.insert(relative_path, data);
        }
    }
    Ok(())
}

/// 将物理文件夹打包为指定目标 SARC 文件 (自动创建父目录并写入磁盘)
pub fn pack_dir_to_sarc_file(
    source_dir: &Path,
    target_pack_file: &Path,
    endian: Endian,
    compress_yaz0: bool,
) -> Result<usize, String> {
    let bytes = pack_dir_to_sarc_bytes(source_dir, endian, compress_yaz0)?;
    if let Some(parent) = target_pack_file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(target_pack_file, &bytes)
        .map_err(|e| format!("保存 SARC 文件失败 {}: {}", target_pack_file.display(), e))?;
    Ok(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use roead::sarc::Sarc;

    #[test]
    fn test_pack_directory_to_sarc() {
        let temp_dir = std::env::temp_dir().join(format!("zevent_test_pack_{}", uuid::Uuid::new_v4()));
        let sub_dir = temp_dir.join("Quest");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("test.txt"), b"hello world").unwrap();

        let packed_bytes = pack_dir_to_sarc_bytes(&temp_dir, Endian::Big, false).unwrap();
        assert!(!packed_bytes.is_empty());

        let sarc = Sarc::new(&packed_bytes).unwrap();
        assert_eq!(sarc.endian(), Endian::Big);
        let data = sarc.get_data("Quest/test.txt").unwrap();
        assert_eq!(data, b"hello world");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
