use std::path::{Path, PathBuf};
use roead::byml::Byml;
use roead::sarc::{Sarc, SarcWriter};
use crate::model::quest::flags_builder::inject_bootup_flags;
use crate::model::quest::scanner::find_titlebg_path;
use crate::model::quest::types::{find_sarc_file, QuestScaffoldConfig};

/// 执行完整任务自动化创建 (注入五大支柱中的元数据、标志与文本)
pub fn execute_full_quest_creation(
    project_path: &Path,
    config: &QuestScaffoldConfig,
) -> Result<Vec<String>, String> {
    let mut logs = Vec::new();

    // 1. 注入 TitleBG.pack (QuestProduct.sbquestpack)
    if config.inject_titlebg {
        inject_quest_product(project_path, config)?;
        logs.push(format!("已成功注入 TitleBG.pack (任务: {})", config.name));
    }

    // 2. 注入 Bootup.pack (GameData & SaveData 标志)
    if config.inject_bootup_flags {
        inject_bootup_flags(project_path, config)?;
        logs.push("已成功注册 GameData 与 SaveData 标志 (Ready/Activated/Finish)".to_string());
    }

    // 3. 注入 Bootup_<Lang>.pack (QL_<Quest>.msbt)
    if config.inject_ql_msbt {
        inject_ql_msbt(project_path, config)?;
        logs.push(format!("已成功生成冒险笔记文本 (QL_{}.msbt)", config.name));
    }

    Ok(logs)
}

/// 注入 QuestProduct.sbquestpack
fn inject_quest_product(project_path: &Path, config: &QuestScaffoldConfig) -> Result<(), String> {
    let titlebg_path = find_titlebg_path(project_path)
        .ok_or_else(|| "未找到 TitleBG.pack 封包".to_string())?;

    let raw = std::fs::read(&titlebg_path)
        .map_err(|e| format!("读取 TitleBG.pack 失败: {}", e))?;

    let is_yaz0 = raw.len() >= 4 && &raw[0..4] == b"Yaz0";
    let decomp = if is_yaz0 {
        roead::yaz0::decompress(&raw).map_err(|e| format!("解压 TitleBG.pack 失败: {:?}", e))?
    } else {
        raw
    };

    let sarc = Sarc::new(&decomp).map_err(|e| format!("解析 TitleBG.pack 失败: {:?}", e))?;
    let mut writer = SarcWriter::from_sarc(&sarc);

    let qp_file = find_sarc_file(&sarc, "Quest/QuestProduct.sbquestpack")
        .ok_or_else(|| "TitleBG.pack 中缺少 Quest/QuestProduct.sbquestpack".to_string())?;

    let qp_raw = qp_file.data();
    let is_qp_yaz0 = qp_raw.len() >= 4 && &qp_raw[0..4] == b"Yaz0";
    let qp_decomp = if is_qp_yaz0 {
        roead::yaz0::decompress(qp_raw).map_err(|e| format!("解压 QuestProduct 失败: {:?}", e))?
    } else {
        qp_raw.to_vec()
    };

    let mut byml = Byml::from_binary(&qp_decomp)
        .map_err(|e| format!("解析 QuestProduct BYML 失败: {:?}", e))?;

    let new_quest_byml = build_quest_byml_entry(config);

    if let Byml::Array(ref mut arr) = byml {
        // 如果同名任务已存在则替换，否则追加
        let existing_idx = arr.iter().position(|item| {
            if let Byml::Map(m) = item {
                m.get("Name").and_then(|v| v.as_string().ok()).map(|s| s.as_str()) == Some(config.name.as_str())
            } else {
                false
            }
        });

        if let Some(idx) = existing_idx {
            arr[idx] = new_quest_byml;
        } else {
            arr.push(new_quest_byml);
        }
    }

    let bin = byml.to_binary(roead::Endian::Little);
    let final_qp_bytes = if is_qp_yaz0 {
        roead::yaz0::compress(&bin)
    } else {
        bin.to_vec()
    };

    writer.files.insert("Quest/QuestProduct.sbquestpack".to_string(), final_qp_bytes);

    let new_pack_bytes = writer.to_binary();
    let final_pack = if is_yaz0 {
        roead::yaz0::compress(&new_pack_bytes)
    } else {
        new_pack_bytes
    };

    std::fs::write(&titlebg_path, &final_pack)
        .map_err(|e| format!("保存写回 TitleBG.pack 失败: {}", e))?;

    Ok(())
}

/// 构造单个任务的 Byml 字典
fn build_quest_byml_entry(config: &QuestScaffoldConfig) -> Byml {
    let mut map = roead::byml::Map::default();
    map.insert("Name".into(), Byml::String(config.name.clone().into()));
    map.insert("Type".into(), Byml::String(config.quest_type.as_str().into()));
    map.insert("Orderer".into(), Byml::String(config.orderer.clone().into()));
    map.insert("Location".into(), Byml::String(config.location.clone().into()));
    map.insert("NotPostNote".into(), Byml::Bool(config.not_post_note));
    map.insert("StepDependencyFlags".into(), Byml::Array(Vec::new()));

    if !config.dependency_flag.is_empty() {
        map.insert("QuestDependencyFlag".into(), Byml::String(config.dependency_flag.clone().into()));
        map.insert("QuestDependencyFlagType".into(), Byml::String("BoolData".into()));
    }

    // 默认两个阶段: Ready 与 Finish
    let mut ready_step = roead::byml::Map::default();
    ready_step.insert("Name".into(), Byml::String("Ready".into()));
    ready_step.insert("MessageName".into(), Byml::String("Desc".into()));
    let act_flag = if config.activated_flag.is_empty() {
        format!("{}_Activated", config.name)
    } else {
        config.activated_flag.clone()
    };
    ready_step.insert("NextFlag".into(), Byml::String(act_flag.into()));
    ready_step.insert("UpdateStep".into(), Byml::Bool(true));
    ready_step.insert("AttentionOff".into(), Byml::Bool(false));

    // 如果启用了地图指引标点，在 Ready 阶段生成 IndicatorActors
    if config.enable_indicator {
        let mut ind_map = roead::byml::Map::default();
        ind_map.insert("HasFar".into(), Byml::Bool(config.indicator_has_far));
        ind_map.insert("InstanceName".into(), Byml::String("".into()));
        let mut loc = roead::byml::Map::default();
        loc.insert("X".into(), Byml::Float(config.indicator_pos.0));
        loc.insert("Y".into(), Byml::Float(config.indicator_pos.1));
        loc.insert("Z".into(), Byml::Float(config.indicator_pos.2));
        ind_map.insert("Location".into(), Byml::Map(loc));
        let ind_name = if config.indicator_actor.is_empty() {
            config.orderer.clone()
        } else {
            config.indicator_actor.clone()
        };
        ind_map.insert("Name".into(), Byml::String(ind_name.into()));
        ind_map.insert("OffFlag".into(), Byml::String(config.indicator_off_flag.clone().into()));
        ready_step.insert("IndicatorActors".into(), Byml::Array(vec![Byml::Map(ind_map)]));
    }

    let mut fin_step = roead::byml::Map::default();
    fin_step.insert("Name".into(), Byml::String("Finish".into()));
    fin_step.insert("MessageName".into(), Byml::String("Finish".into()));
    let fin_flag = if config.finish_flag.is_empty() {
        format!("{}_Finish", config.name)
    } else {
        config.finish_flag.clone()
    };
    fin_step.insert("NextFlag".into(), Byml::String(fin_flag.into()));
    fin_step.insert("UpdateStep".into(), Byml::Bool(true));
    fin_step.insert("AttentionOff".into(), Byml::Bool(false));

    let mut steps = vec![Byml::Map(ready_step)];
    for (idx, flag) in config.extra_flags.iter().enumerate() {
        let clean = flag.trim();
        if !clean.is_empty() {
            let mut step_map = roead::byml::Map::default();
            step_map.insert("Name".into(), Byml::String(format!("Step{}", idx + 1).into()));
            step_map.insert("MessageName".into(), Byml::String(format!("Step{}", idx + 1).into()));
            step_map.insert("NextFlag".into(), Byml::String(clean.into()));
            step_map.insert("UpdateStep".into(), Byml::Bool(true));
            step_map.insert("AttentionOff".into(), Byml::Bool(false));
            steps.push(Byml::Map(step_map));
        }
    }
    steps.push(Byml::Map(fin_step));

    map.insert("Steps".into(), Byml::Array(steps));

    Byml::Map(map)
}

/// 注入冒险笔记文本 (QL_<Quest>.msbt)
fn inject_ql_msbt(project_path: &Path, config: &QuestScaffoldConfig) -> Result<(), String> {
    let lang_pack = find_lang_pack(project_path)
        .ok_or_else(|| "未找到语言包 (如 Bootup_CNzh.pack)".to_string())?;

    let raw = std::fs::read(&lang_pack)
        .map_err(|e| format!("读取语言包失败: {}", e))?;

    let is_yaz0 = raw.len() >= 4 && &raw[0..4] == b"Yaz0";
    let decomp = if is_yaz0 {
        roead::yaz0::decompress(&raw).map_err(|e| format!("解压语言包失败: {:?}", e))?
    } else {
        raw
    };

    let sarc = Sarc::new(&decomp).map_err(|e| format!("解析语言包 SARC 失败: {:?}", e))?;
    let mut pack_writer = SarcWriter::from_sarc(&sarc);

    // 找到内部的 Message/Msg_*.product.ssarc
    let msg_entry = sarc.files()
        .find(|f| f.name().is_some_and(|n| n.starts_with("Message/Msg_") && n.ends_with(".ssarc")))
        .ok_or_else(|| "语言包内缺少 Msg_*.product.ssarc".to_string())?;

    let msg_name = msg_entry.name().unwrap().to_string();
    let msg_decomp = roead::yaz0::decompress(msg_entry.data())
        .map_err(|e| format!("解压 Msg ssarc 失败: {:?}", e))?;
    let msg_sarc = Sarc::new(&msg_decomp)
        .map_err(|e| format!("解析 Msg ssarc 失败: {:?}", e))?;
    let mut msg_writer = SarcWriter::from_sarc(&msg_sarc);

    // 构造 QL_<Quest>.msbt 字节流
    let ql_msbt_bytes = build_ql_msbt_bytes(config)?;

    let target_ql_path = format!("QuestMsg/QL_{}.msbt", config.name);
    msg_writer.files.insert(target_ql_path, ql_msbt_bytes);

    let new_msg_bytes = msg_writer.to_binary();
    let new_msg_yaz0 = roead::yaz0::compress(&new_msg_bytes);
    pack_writer.files.insert(msg_name, new_msg_yaz0);

    let new_pack_bytes = pack_writer.to_binary();
    let final_pack = if is_yaz0 {
        roead::yaz0::compress(&new_pack_bytes)
    } else {
        new_pack_bytes
    };

    std::fs::write(&lang_pack, &final_pack)
        .map_err(|e| format!("保存写回语言包失败: {}", e))?;

    Ok(())
}

/// 构造 QL_<Quest>.msbt 二进制数据
fn build_ql_msbt_bytes(config: &QuestScaffoldConfig) -> Result<Vec<u8>, String> {
    let name_key = format!("QL_{}_Name", config.name);
    let desc_key = format!("QL_{}_Desc", config.name);
    let fin_key = format!("QL_{}_Finish", config.name);

    let mut entries = serde_json::Map::new();
    entries.insert(
        name_key,
        serde_json::json!({
            "contents": [config.display_title.clone()]
        }),
    );
    entries.insert(
        desc_key,
        serde_json::json!({
            "contents": [config.desc_text.clone()]
        }),
    );
    entries.insert(
        fin_key,
        serde_json::json!({
            "contents": [config.finish_text.clone()]
        }),
    );

    for (idx, flag) in config.extra_flags.iter().enumerate() {
        let clean = flag.trim();
        if !clean.is_empty() {
            let step_key = format!("QL_{}_Step{}", config.name, idx + 1);
            let step_desc = format!("{} (阶段 {})", config.desc_text, idx + 1);
            entries.insert(
                step_key,
                serde_json::json!({
                    "contents": [step_desc]
                }),
            );
        }
    }

    let group_count = entries.len();
    let msyt_val = serde_json::json!({
        "group_count": group_count,
        "entries": entries
    });

    let doc: msyt::Msyt = serde_json::from_value(msyt_val)
        .map_err(|e| format!("构造 MSYT 文档结构失败: {}", e))?;

    doc.into_msbt_bytes(msyt::Endianness::Little)
        .map_err(|e| format!("编译 QL MSBT 失败: {:?}", e))
}

/// 查找语言包路径
fn find_lang_pack(project_path: &Path) -> Option<PathBuf> {
    let candidates = [
        project_path.join("01007EF00011E000/romfs/Pack/Bootup_CNzh.pack"),
        project_path.join("romfs/Pack/Bootup_CNzh.pack"),
        project_path.join("Pack/Bootup_CNzh.pack"),
        project_path.join("Bootup_CNzh.pack"),
        project_path.join("01007EF00011E000/romfs/Pack/Bootup_USen.pack"),
        project_path.join("romfs/Pack/Bootup_USen.pack"),
        project_path.join("Pack/Bootup_USen.pack"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}
