mod extract;
mod ops;
mod pack;
mod update;

pub use extract::{copy_dir_all, extract_sarc_to_dir};
#[allow(unused_imports)]
pub use pack::{pack_dir_to_sarc_bytes, pack_dir_to_sarc_file};
use ops::{delete_nested_sarc_entry, rename_nested_sarc_entry};
use std::fs;
use std::path::Path;
use update::update_nested_sarc_bytes;

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

/// 删除指定物理文件/目录或 SARC 封包内部的条目/虚拟子目录
pub fn delete_file_from_target(path: &str) -> Result<(), String> {
    if path.starts_with("SARC:") {
        let clean = path.trim_start_matches("SARC:");
        let parts: Vec<&str> = clean.split("//").collect();
        if parts.is_empty() {
            return Err("无效的 SARC 文件路径".to_string());
        }

        let root_physical_path = parts[0];
        if parts.len() == 1 {
            // 删除整个 SARC 物理归档包
            fs::remove_file(root_physical_path)
                .map_err(|e| format!("无法删除封包文件: {}", e))?;
            return Ok(());
        }

        let root_bytes = fs::read(root_physical_path)
            .map_err(|e| format!("无法读取根封包文件 {}: {}", root_physical_path, e))?;

        let nested_paths = &parts[1..];
        let updated_root_bytes = delete_nested_sarc_entry(&root_bytes, nested_paths, root_physical_path)?;
        fs::write(root_physical_path, &updated_root_bytes)
            .map_err(|e| format!("保存写回封包文件失败: {}", e))?;

        Ok(())
    } else {
        let p = Path::new(path);
        if p.is_dir() {
            fs::remove_dir_all(p).map_err(|e| format!("删除目录失败: {}", e))
        } else {
            fs::remove_file(p).map_err(|e| format!("删除文件失败: {}", e))
        }
    }
}

/// 重命名物理文件/目录或 SARC 封包内部的虚拟条目，并返回新的完整路径
pub fn rename_file_in_target(old_path: &str, new_name: &str) -> Result<String, String> {
    let clean_new_name = new_name.trim();
    if clean_new_name.is_empty() {
        return Err("新文件名不能为空".to_string());
    }

    if old_path.starts_with("SARC:") {
        let clean = old_path.trim_start_matches("SARC:");
        let parts: Vec<&str> = clean.split("//").collect();
        if parts.is_empty() {
            return Err("无效的 SARC 文件路径".to_string());
        }

        let root_physical_path = parts[0];
        if parts.len() == 1 {
            // 重命名 SARC 物理封包文件自身
            let old_p = Path::new(root_physical_path);
            let parent = old_p.parent().unwrap_or_else(|| Path::new(""));
            let new_p = parent.join(clean_new_name);
            fs::rename(old_p, &new_p).map_err(|e| format!("重命名封包物理文件失败: {}", e))?;
            return Ok(format!("SARC:{}", new_p.to_string_lossy()));
        }

        let root_bytes = fs::read(root_physical_path)
            .map_err(|e| format!("无法读取根封包文件 {}: {}", root_physical_path, e))?;

        let nested_paths = &parts[1..];
        // 计算新的最后一级相对路径 (保留路径中的父文件夹)
        let last_target = nested_paths[nested_paths.len() - 1];
        let normalized_target = last_target.replace('\\', "/");
        let new_inner_path = if let Some(idx) = normalized_target.rfind('/') {
            format!("{}/{}", &normalized_target[..idx], clean_new_name)
        } else {
            clean_new_name.to_string()
        };

        let updated_root_bytes = rename_nested_sarc_entry(
            &root_bytes,
            nested_paths,
            &new_inner_path,
            root_physical_path,
        )?;
        fs::write(root_physical_path, &updated_root_bytes)
            .map_err(|e| format!("保存写回封包文件失败: {}", e))?;

        // 构造新的虚拟完整路径
        let mut new_parts: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
        let last_idx = new_parts.len() - 1;
        new_parts[last_idx] = new_inner_path;
        Ok(format!("SARC:{}", new_parts.join("//")))
    } else {
        let old_p = Path::new(old_path);
        let parent = old_p.parent().unwrap_or_else(|| Path::new(""));
        let new_p = parent.join(clean_new_name);
        fs::rename(old_p, &new_p).map_err(|e| format!("重命名文件失败: {}", e))?;
        Ok(new_p.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roead::sarc::{Sarc, SarcWriter};
    use roead::Endian;

    #[test]
    fn test_sarc_update_single_file() {
        let mut writer = SarcWriter::new(Endian::Little);
        writer.files.insert("Actor/Armor.byml".to_string(), b"original_data".to_vec());
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
    fn test_sarc_delete_and_rename() {
        let mut writer = SarcWriter::new(Endian::Little);
        writer.files.insert("Actor/Armor.byml".to_string(), b"armor_data".to_vec());
        writer.files.insert("Actor/Weapon.byml".to_string(), b"weapon_data".to_vec());
        let sarc_bin = writer.to_binary();

        // 测试重命名
        let renamed_bin = rename_nested_sarc_entry(
            &sarc_bin,
            &["Actor/Armor.byml"],
            "Actor/Armor_V2.byml",
            "test.pack",
        ).unwrap();
        let parsed_renamed = Sarc::new(&renamed_bin).unwrap();
        assert!(parsed_renamed.get_data("Actor/Armor.byml").is_none());
        assert_eq!(parsed_renamed.get_data("Actor/Armor_V2.byml").unwrap(), b"armor_data");

        // 测试删除
        let deleted_bin = delete_nested_sarc_entry(
            &renamed_bin,
            &["Actor/Weapon.byml"],
            "test.pack",
        ).unwrap();
        let parsed_deleted = Sarc::new(&deleted_bin).unwrap();
        assert!(parsed_deleted.get_data("Actor/Weapon.byml").is_none());
        assert!(parsed_deleted.get_data("Actor/Armor_V2.byml").is_some());
    }
}
