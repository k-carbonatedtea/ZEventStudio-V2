use std::path::{Path, PathBuf};
use roead::byml::Byml;
use roead::sarc::{Sarc, SarcWriter};
use crate::model::quest::types::{compute_botw_hash, find_sarc_file, QuestScaffoldConfig};

/// 查找 Bootup.pack 路径
pub fn find_bootup_path(project_path: &Path) -> Option<PathBuf> {
    let candidates = [
        project_path.join("01007EF00011E000/romfs/Pack/Bootup.pack"),
        project_path.join("romfs/Pack/Bootup.pack"),
        project_path.join("Pack/Bootup.pack"),
        project_path.join("Bootup.pack"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

/// 注入 GameData (bool_data_1.bgdata) 与 SaveData (savedataformat.ssarc) 标志位
pub fn inject_bootup_flags(project_path: &Path, config: &QuestScaffoldConfig) -> Result<(), String> {
    let bootup_path = find_bootup_path(project_path)
        .ok_or_else(|| "未找到 Bootup.pack 封包".to_string())?;

    let raw = std::fs::read(&bootup_path)
        .map_err(|e| format!("读取 Bootup.pack 失败: {}", e))?;

    let is_yaz0 = raw.len() >= 4 && &raw[0..4] == b"Yaz0";
    let decomp = if is_yaz0 {
        roead::yaz0::decompress(&raw).map_err(|e| format!("解压 Bootup.pack 失败: {:?}", e))?
    } else {
        raw
    };

    let sarc = Sarc::new(&decomp).map_err(|e| format!("解析 Bootup.pack SARC 失败: {:?}", e))?;
    let mut bootup_writer = SarcWriter::from_sarc(&sarc);

    // 1. 更新 gamedata.ssarc -> bool_data_1.bgdata
    update_gamedata_flags(&sarc, &mut bootup_writer, config)?;

    // 2. 更新 savedataformat.ssarc -> saveformat_*.bgsvdata
    update_savedata_flags(&sarc, &mut bootup_writer, config)?;

    // 重新写回 Bootup.pack
    let new_bootup_bytes = bootup_writer.to_binary();
    let final_bytes = if is_yaz0 {
        roead::yaz0::compress(&new_bootup_bytes)
    } else {
        new_bootup_bytes
    };

    std::fs::write(&bootup_path, &final_bytes)
        .map_err(|e| format!("保存 Bootup.pack 失败: {}", e))?;

    Ok(())
}

/// 更新 gamedata.ssarc 中的 bool_data_1.bgdata
fn update_gamedata_flags(
    bootup_sarc: &Sarc,
    bootup_writer: &mut SarcWriter,
    config: &QuestScaffoldConfig,
) -> Result<(), String> {
    let gd_file = find_sarc_file(bootup_sarc, "GameData/gamedata.ssarc")
        .ok_or_else(|| "Bootup.pack 中缺少 GameData/gamedata.ssarc".to_string())?;

    let gd_decomp = roead::yaz0::decompress(gd_file.data())
        .map_err(|e| format!("解压 gamedata.ssarc 失败: {:?}", e))?;
    let gd_sarc = Sarc::new(&gd_decomp)
        .map_err(|e| format!("解析 gamedata.ssarc 失败: {:?}", e))?;
    let mut gd_writer = SarcWriter::from_sarc(&gd_sarc);

    let b1_file = find_sarc_file(&gd_sarc, "bool_data_1.bgdata")
        .ok_or_else(|| "gamedata.ssarc 中缺少 bool_data_1.bgdata".to_string())?;

    let mut b1_byml = Byml::from_binary(b1_file.data())
        .map_err(|e| format!("解析 bool_data_1.bgdata 失败: {:?}", e))?;

    let ready_flag = format!("{}_Ready", config.name);
    let act_flag = format!("{}_Activated", config.name);
    let fin_flag = format!("{}_Finish", config.name);

    if let Byml::Map(ref mut root_map) = b1_byml {
        if let Some(Byml::Array(ref mut bool_arr)) = root_map.get_mut("bool_data") {
            insert_or_replace_bgdata_entry(bool_arr, &ready_flag, 1);
            insert_or_replace_bgdata_entry(bool_arr, &act_flag, 0);
            insert_or_replace_bgdata_entry(bool_arr, &fin_flag, 0);
        }
    }

    let b1_bin = b1_byml.to_binary(roead::Endian::Little);

    for (k, v) in gd_writer.files.iter_mut() {
        if k.ends_with("bool_data_1.bgdata") {
            *v = b1_bin.to_vec();
        }
    }

    let new_gd_bytes = gd_writer.to_binary();
    let new_gd_yaz0 = roead::yaz0::compress(&new_gd_bytes);
    bootup_writer.files.insert("GameData/gamedata.ssarc".to_string(), new_gd_yaz0);

    Ok(())
}

/// 向 bool_data_1 中插入或更新条目
fn insert_or_replace_bgdata_entry(arr: &mut Vec<Byml>, flag_name: &str, init_val: i32) {
    let hash = compute_botw_hash(flag_name);
    let existing = arr.iter_mut().find(|item| {
        if let Byml::Map(m) = item {
            m.get("DataName").and_then(|v| v.as_string().ok()).map(|s| s.as_str()) == Some(flag_name)
        } else {
            false
        }
    });

    if let Some(Byml::Map(m)) = existing {
        m.insert("InitValue".into(), Byml::I32(init_val));
        m.insert("HashValue".into(), Byml::I32(hash));
    } else {
        let mut entry = roead::byml::Map::default();
        entry.insert("DataName".into(), Byml::String(flag_name.into()));
        entry.insert("DeleteRev".into(), Byml::I32(-1));
        entry.insert("HashValue".into(), Byml::I32(hash));
        entry.insert("InitValue".into(), Byml::I32(init_val));
        entry.insert("IsEventAssociated".into(), Byml::Bool(false));
        entry.insert("IsOneTrigger".into(), Byml::Bool(true));
        entry.insert("IsProgramReadable".into(), Byml::Bool(false));
        entry.insert("IsProgramWritable".into(), Byml::Bool(false));
        entry.insert("IsSave".into(), Byml::Bool(true));
        entry.insert("MaxValue".into(), Byml::Bool(true));
        entry.insert("MinValue".into(), Byml::Bool(false));
        entry.insert("ResetType".into(), Byml::I32(0));
        arr.push(Byml::Map(entry));
    }
}

/// 更新 savedataformat.ssarc 中的 saveformat 槽位
fn update_savedata_flags(
    bootup_sarc: &Sarc,
    bootup_writer: &mut SarcWriter,
    config: &QuestScaffoldConfig,
) -> Result<(), String> {
    let sv_file = find_sarc_file(bootup_sarc, "GameData/savedataformat.ssarc")
        .ok_or_else(|| "Bootup.pack 中缺少 GameData/savedataformat.ssarc".to_string())?;

    let sv_decomp = roead::yaz0::decompress(sv_file.data())
        .map_err(|e| format!("解压 savedataformat.ssarc 失败: {:?}", e))?;
    let sv_sarc = Sarc::new(&sv_decomp)
        .map_err(|e| format!("解析 savedataformat.ssarc 失败: {:?}", e))?;
    let mut sv_writer = SarcWriter::from_sarc(&sv_sarc);

    let ready_flag = format!("{}_Ready", config.name);
    let act_flag = format!("{}_Activated", config.name);
    let fin_flag = format!("{}_Finish", config.name);

    // 分配槽位：Ready -> saveformat_5, Activated -> saveformat_3, Finish -> saveformat_0
    update_single_saveformat(&mut sv_writer, "saveformat_5.bgsvdata", &ready_flag)?;
    update_single_saveformat(&mut sv_writer, "saveformat_3.bgsvdata", &act_flag)?;
    update_single_saveformat(&mut sv_writer, "saveformat_0.bgsvdata", &fin_flag)?;

    let new_sv_bytes = sv_writer.to_binary();
    let new_sv_yaz0 = roead::yaz0::compress(&new_sv_bytes);
    bootup_writer.files.insert("GameData/savedataformat.ssarc".to_string(), new_sv_yaz0);

    Ok(())
}

/// 更新单个 saveformat 文件中的条目并保持 HashValue 排序
fn update_single_saveformat(
    sv_writer: &mut SarcWriter,
    format_name: &str,
    flag_name: &str,
) -> Result<(), String> {
    let (target_key, raw_bytes) = sv_writer.files.iter()
        .find(|(k, _)| k.ends_with(format_name))
        .map(|(k, v)| (k.clone(), v.clone()))
        .ok_or_else(|| format!("未找到 {}", format_name))?;

    let mut byml = Byml::from_binary(&raw_bytes)
        .map_err(|e| format!("解析 {} 失败: {:?}", format_name, e))?;

    let root_map = match byml {
        Byml::Map(ref mut m) => m,
        _ => return Err("saveformat 根节点非 Map".to_string()),
    };

    let file_list = match root_map.get_mut("file_list") {
        Some(Byml::Array(arr)) => arr,
        _ => return Err("saveformat 缺少 file_list 数组".to_string()),
    };

    let hash = compute_botw_hash(flag_name);
    let mut inserted = false;

    for item in file_list.iter_mut() {
        let entries = match item {
            Byml::Array(arr) => arr,
            _ => continue,
        };

        let already_exists = entries.iter().any(|e| {
            if let Byml::Map(em) = e {
                em.get("DataName")
                    .and_then(|v| v.as_string().ok())
                    .map(|s| s.as_str()) == Some(flag_name)
            } else {
                false
            }
        });

        if !already_exists {
            let mut new_e = roead::byml::Map::default();
            new_e.insert("DataName".into(), Byml::String(flag_name.into()));
            new_e.insert("HashValue".into(), Byml::I32(hash));
            entries.push(Byml::Map(new_e));

            // 必须严格按 HashValue 升序排序！
            entries.sort_by_key(|e| {
                if let Byml::Map(em) = e {
                    em.get("HashValue").and_then(|v| v.as_i32().ok()).unwrap_or(0)
                } else {
                    0
                }
            });
        }
        inserted = true;
        break;
    }

    if !inserted {
        let mut new_e = roead::byml::Map::default();
        new_e.insert("DataName".into(), Byml::String(flag_name.into()));
        new_e.insert("HashValue".into(), Byml::I32(hash));
        file_list.push(Byml::Array(vec![Byml::Map(new_e)]));
    }

    let bin = byml.to_binary(roead::Endian::Little);
    sv_writer.files.insert(target_key, bin.to_vec());
    Ok(())
}
