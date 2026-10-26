use crate::model::quest::{compute_botw_hash, QuestDefinition};
use crate::ui::sidebar::quests::modal_state::{QuestRawDataModalState, RawDataFormat};

/// 根据当前选中的格式生成格式化文本
pub fn generate_raw_data_text(modal: &QuestRawDataModalState) -> String {
    let q = &modal.quest;
    match modal.active_format {
        RawDataFormat::Json => serde_json::to_string_pretty(q).unwrap_or_else(|_| "{}".to_string()),
        RawDataFormat::BymlYaml => format_byml_yaml(q),
        RawDataFormat::Msbt => format_msbt_text(q),
        RawDataFormat::FlagsHash => format_flags_hashes(q),
    }
}

/// 构造符合 ZeldaMods QuestProduct.sbquestpack 规范的 YAML 格式
pub fn format_byml_yaml(q: &QuestDefinition) -> String {
    let mut out = String::new();
    out.push_str("# Zelda BotW TitleBG.pack -> Quest/QuestProduct.sbquestpack\n");
    out.push_str(&format!("Name: \"{}\"\n", q.name));
    out.push_str(&format!("Type: \"{}\"\n", q.quest_type.as_str()));
    out.push_str(&format!("Orderer: \"{}\"\n", q.orderer));
    out.push_str(&format!("Location: \"{}\"\n", q.location));
    out.push_str(&format!("NotPostNote: {}\n", q.not_post_note));
    if let Some(ref dep) = q.dependency_flag {
        out.push_str(&format!("QuestDependencyFlag: \"{}\"\n", dep));
        out.push_str("QuestDependencyFlagType: \"BoolData\"\n");
    } else {
        out.push_str("QuestDependencyFlag: null\n");
    }
    out.push_str("StepDependencyFlags: []\n");
    out.push_str("Steps:\n");
    for (i, step) in q.steps.iter().enumerate() {
        out.push_str(&format!("  - Index: {}\n", i));
        out.push_str(&format!("    Name: \"{}\"\n", step.name));
        out.push_str(&format!("    MessageName: \"{}\"\n", step.message_name));
        out.push_str(&format!("    NextFlag: \"{}\"\n", step.next_flag));
        out.push_str(&format!("    UpdateStep: {}\n", step.update_step));
        out.push_str(&format!("    AttentionOff: {}\n", step.attention_off));
        if step.indicators.is_empty() {
            out.push_str("    IndicatorActors: []\n");
        } else {
            out.push_str("    IndicatorActors:\n");
            for ind in &step.indicators {
                out.push_str(&format!("      - Name: \"{}\"\n", ind.actor_name.as_deref().unwrap_or("")));
                out.push_str(&format!("        InstanceName: \"{}\"\n", ind.instance_name.as_deref().unwrap_or("")));
                out.push_str(&format!("        Location: [{:.1}, {:.1}, {:.1}]\n", ind.position.0, ind.position.1, ind.position.2));
                out.push_str(&format!("        OffFlag: \"{}\"\n", ind.off_flag.as_deref().unwrap_or("")));
                out.push_str(&format!("        HasFar: {}\n", ind.has_far));
            }
        }
    }
    out
}

/// 构造冒险笔记 MSBT 文本预览
pub fn format_msbt_text(q: &QuestDefinition) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Bootup_<Lang>.pack -> Message/Msg_*.ssarc -> QuestMsg/QL_{}.msbt\n\n", q.name));
    out.push_str(&format!("[QL_{}_Name]\n{}\n\n", q.name, q.display_title.as_deref().unwrap_or("（未配置标题）")));
    out.push_str(&format!("[QL_{}_Desc]\n{}\n\n", q.name, q.desc_text.as_deref().unwrap_or("（未配置初始描述）")));
    out.push_str(&format!("[QL_{}_Finish]\n{}\n\n", q.name, q.finish_text.as_deref().unwrap_or("（未配置完成描述）")));
    for (idx, step) in q.steps.iter().filter(|s| s.name != "Ready" && s.name != "Finish").enumerate() {
        out.push_str(&format!("[QL_{}_Step{}]\n阶段 {} 说明 ({})\n\n", q.name, idx + 1, idx + 1, step.name));
    }
    out
}

/// 构造标志位 CRC32 哈希与存档槽位对照表
pub fn format_flags_hashes(q: &QuestDefinition) -> String {
    let mut out = String::new();
    out.push_str("# Bootup.pack 标志位哈希表与存档槽位映射 (GameData & SaveData)\n\n");
    let ready_flag = format!("{}_Ready", q.name);
    let act_flag = q.steps.iter().find(|s| s.name == "Ready").map(|s| s.next_flag.as_str()).unwrap_or("");
    let fin_flag = q.steps.iter().find(|s| s.name == "Finish").map(|s| s.next_flag.as_str()).unwrap_or("");

    let flags = [
        ("就绪标志 (Ready Flag)", ready_flag.as_str(), 1, "saveformat_5.bgsvdata"),
        ("进行中标志 (Activated Flag)", act_flag, 0, "saveformat_3.bgsvdata"),
        ("完成标志 (Finish Flag)", fin_flag, 0, "saveformat_0.bgsvdata"),
    ];

    for (label, flag, init_val, slot) in flags {
        if !flag.is_empty() {
            let hash = compute_botw_hash(flag);
            out.push_str(&format!("[{}]\n", label));
            out.push_str(&format!("  标志名称: {}\n", flag));
            out.push_str(&format!("  CRC32 (Dec): {}\n", hash));
            out.push_str(&format!("  CRC32 (Hex): 0x{:08X}\n", hash as u32));
            out.push_str(&format!("  GameData: bool_data_1.bgdata (InitValue: {})\n", init_val));
            out.push_str(&format!("  SaveData 槽位: {}\n\n", slot));
        }
    }

    if let Some(ref dep) = q.dependency_flag {
        let hash = compute_botw_hash(dep);
        out.push_str("[前置依赖标志 (QuestDependencyFlag)]\n");
        out.push_str(&format!("  标志名称: {}\n", dep));
        out.push_str(&format!("  CRC32 (Dec): {}\n", hash));
        out.push_str(&format!("  CRC32 (Hex): 0x{:08X}\n", hash as u32));
        out.push_str("  GameData: bool_data_1.bgdata (InitValue: 0)\n");
        out.push_str("  SaveData 槽位: saveformat_1.bgsvdata\n\n");
    }

    for (idx, step) in q.steps.iter().filter(|s| s.name != "Ready" && s.name != "Finish").enumerate() {
        if !step.next_flag.is_empty() {
            let hash = compute_botw_hash(&step.next_flag);
            let sv_slot = (idx + 2) % 8;
            out.push_str(&format!("[扩展阶段标志 (StepFlag #{})]\n", idx + 1));
            out.push_str(&format!("  标志名称: {}\n", step.next_flag));
            out.push_str(&format!("  CRC32 (Dec): {}\n", hash));
            out.push_str(&format!("  CRC32 (Hex): 0x{:08X}\n", hash as u32));
            out.push_str("  GameData: bool_data_1.bgdata (InitValue: 0)\n");
            out.push_str(&format!("  SaveData 槽位: saveformat_{}.bgsvdata\n\n", sv_slot));
        }
    }
    out
}
