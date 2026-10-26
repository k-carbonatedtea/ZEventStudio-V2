use std::path::{Path, PathBuf};
use roead::byml::Byml;
use roead::sarc::Sarc;
use crate::model::quest::types::{find_sarc_file, QuestDefinition, QuestIndicator, QuestStep, QuestType};

/// 在项目根目录下查找 TitleBG.pack 的可能路径
pub fn find_titlebg_path(project_path: &Path) -> Option<PathBuf> {
    let candidates = [
        project_path.join("01007EF00011E000/romfs/Pack/TitleBG.pack"),
        project_path.join("romfs/Pack/TitleBG.pack"),
        project_path.join("Pack/TitleBG.pack"),
        project_path.join("TitleBG.pack"),
    ];

    for c in &candidates {
        if c.is_file() {
            return Some(c.clone());
        }
    }

    // 递归浅层检索 (最多 4 层深度)
    find_file_recursive(project_path, "TitleBG.pack", 4)
}

/// 扫描项目中的所有任务定义
pub fn scan_project_quests(project_path: &Path) -> Vec<QuestDefinition> {
    let titlebg_path = match find_titlebg_path(project_path) {
        Some(p) => p,
        None => return Vec::new(),
    };

    let raw_bytes = match std::fs::read(&titlebg_path) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };

    let pack_data = roead::yaz0::decompress(&raw_bytes).unwrap_or(raw_bytes);
    let sarc = match Sarc::new(&pack_data) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let qp_file = match find_sarc_file(&sarc, "Quest/QuestProduct.sbquestpack") {
        Some(f) => f,
        None => return Vec::new(),
    };

    let qp_bytes = qp_file.data();
    let qp_decomp = roead::yaz0::decompress(qp_bytes).unwrap_or_else(|_| qp_bytes.to_vec());

    let byml = match Byml::from_binary(&qp_decomp) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };

    let arr = match byml {
        Byml::Array(a) => a,
        _ => return Vec::new(),
    };

    let mut quests = Vec::new();
    for item in &arr {
        if let Some(quest) = parse_quest_byml(item) {
            quests.push(quest);
        }
    }

    // 尝试从语言包中加载冒险笔记本地化标题与描述
    enrich_quests_with_msbt(project_path, &mut quests);

    quests
}

/// 解析单个 Byml 节点为 QuestDefinition
fn parse_quest_byml(byml: &Byml) -> Option<QuestDefinition> {
    let map = match byml {
        Byml::Map(m) => m,
        _ => return None,
    };

    let name = map.get("Name")?.as_string().ok()?.to_string();
    let type_str = map.get("Type").and_then(|v| v.as_string().ok()).map(|s| s.as_str()).unwrap_or("Mini");
    let quest_type = QuestType::from_str(type_str);
    let orderer = map.get("Orderer").and_then(|v| v.as_string().ok()).map(|s| s.as_str()).unwrap_or("").to_string();
    let location = map.get("Location").and_then(|v| v.as_string().ok()).map(|s| s.as_str()).unwrap_or("Unknown").to_string();
    let not_post_note = map.get("NotPostNote").and_then(|v| v.as_bool().ok()).unwrap_or(false);
    let dependency_flag = map
        .get("QuestDependencyFlag")
        .and_then(|v| v.as_string().ok())
        .map(|s| s.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let mut steps = Vec::new();
    if let Some(Byml::Array(step_arr)) = map.get("Steps") {
        for s in step_arr {
            if let Byml::Map(sm) = s {
                let s_name = sm.get("Name").and_then(|v| v.as_string().ok()).map(|s| s.as_str()).unwrap_or("").to_string();
                let msg_name = sm.get("MessageName").and_then(|v| v.as_string().ok()).map(|s| s.as_str()).unwrap_or("").to_string();
                let next_flag = sm.get("NextFlag").and_then(|v| v.as_string().ok()).map(|s| s.as_str()).unwrap_or("").to_string();
                let update_step = sm.get("UpdateStep").and_then(|v| v.as_bool().ok()).unwrap_or(true);
                let attention_off = sm.get("AttentionOff").and_then(|v| v.as_bool().ok()).unwrap_or(false);
                let indicators = parse_step_indicators(sm);

                steps.push(QuestStep {
                    name: s_name,
                    message_name: msg_name,
                    next_flag,
                    update_step,
                    attention_off,
                    indicators,
                });
            }
        }
    }

    Some(QuestDefinition {
        name,
        quest_type,
        orderer,
        location,
        not_post_note,
        dependency_flag,
        steps,
        display_title: None,
        desc_text: None,
        finish_text: None,
    })
}

/// 扫描 Bootup_<Lang>.pack 以补充 QL_<Quest> 本地化文本
fn enrich_quests_with_msbt(project_path: &Path, quests: &mut [QuestDefinition]) {
    let cnzh_candidates = [
        project_path.join("01007EF00011E000/romfs/Pack/Bootup_CNzh.pack"),
        project_path.join("romfs/Pack/Bootup_CNzh.pack"),
        project_path.join("Pack/Bootup_CNzh.pack"),
        project_path.join("01007EF00011E000/romfs/Pack/Bootup_USen.pack"),
        project_path.join("romfs/Pack/Bootup_USen.pack"),
    ];

    let pack_path = cnzh_candidates.iter().find(|p| p.is_file())
        .cloned()
        .or_else(|| find_file_recursive(project_path, "Bootup_CNzh.pack", 4));

    let pack_path = match pack_path {
        Some(p) => p,
        None => return,
    };

    let raw = match std::fs::read(&pack_path) {
        Ok(b) => b,
        Err(_) => return,
    };

    let decomp = roead::yaz0::decompress(&raw).unwrap_or(raw);
    let sarc = match Sarc::new(&decomp) {
        Ok(s) => s,
        Err(_) => return,
    };

    // 寻找 Message/Msg_*.product.ssarc
    for file in sarc.files() {
        if let Some(name) = file.name() {
            if name.starts_with("Message/Msg_") && name.ends_with(".ssarc") {
                let msg_decomp = roead::yaz0::decompress(file.data()).unwrap_or_else(|_| file.data().to_vec());
                if let Ok(msg_sarc) = Sarc::new(&msg_decomp) {
                    for q in quests.iter_mut() {
                        let ql_file_name = format!("QuestMsg/QL_{}.msbt", q.name);
                        if let Some(ql_file) = find_sarc_file(&msg_sarc, &ql_file_name) {
                            if let Ok(msyt_doc) = msyt::Msyt::from_msbt_bytes(ql_file.data()) {
                                if let Ok(json_val) = serde_json::to_value(&msyt_doc) {
                                    if let Some(entries) = json_val.get("entries").and_then(|v| v.as_object()) {
                                        let name_key = format!("QL_{}_Name", q.name);
                                        let desc_key = format!("QL_{}_Desc", q.name);
                                        let fin_key = format!("QL_{}_Finish", q.name);

                                        if let Some(e) = entries.get(&name_key) {
                                            q.display_title = Some(entry_json_to_plain_string(e));
                                        }
                                        if let Some(e) = entries.get(&desc_key) {
                                            q.desc_text = Some(entry_json_to_plain_string(e));
                                        }
                                        if let Some(e) = entries.get(&fin_key) {
                                            q.finish_text = Some(entry_json_to_plain_string(e));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
    }
}

/// 提取 MSYT Entry 的纯文本内容
fn entry_json_to_plain_string(entry: &serde_json::Value) -> String {
    let mut out = String::new();
    if let Some(contents) = entry.get("contents").and_then(|v| v.as_array()) {
        for part in contents {
            if let Some(s) = part.as_str() {
                out.push_str(s);
            }
        }
    }
    out
}

/// 简单受限深度的递归文件查找
fn find_file_recursive(dir: &Path, target_filename: &str, depth: usize) -> Option<PathBuf> {
    if depth == 0 || !dir.is_dir() {
        return None;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.eq_ignore_ascii_case(target_filename) {
                        return Some(path);
                    }
                }
            } else if path.is_dir() {
                if let Some(found) = find_file_recursive(&path, target_filename, depth - 1) {
                    return Some(found);
                }
            }
        }
    }
    None
}

/// 解析任务单阶段中的地图指示标点 (IndicatorActors 与 IndicatorLocations)
fn parse_step_indicators(sm: &roead::byml::Map) -> Vec<QuestIndicator> {
    let mut list = Vec::new();

    // 1. 解析指定 Actor 指示器 (IndicatorActors)
    if let Some(Byml::Array(arr)) = sm.get("IndicatorActors") {
        for item in arr {
            if let Byml::Map(m) = item {
                let name = m.get("Name").and_then(|v| v.as_string().ok()).map(|s| s.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());
                let instance_name = m.get("InstanceName").and_then(|v| v.as_string().ok()).map(|s| s.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());
                let pos = parse_byml_location(m.get("Location"));
                let off_flag = m.get("OffFlag").and_then(|v| v.as_string().ok()).map(|s| s.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());
                let has_far = m.get("HasFar").and_then(|v| v.as_bool().ok()).unwrap_or(false);

                list.push(QuestIndicator {
                    actor_name: name,
                    instance_name,
                    position: pos,
                    off_flag,
                    has_far,
                });
            }
        }
    }

    // 2. 解析纯世界坐标指示器 (IndicatorLocations)
    if let Some(Byml::Array(arr)) = sm.get("IndicatorLocations") {
        for item in arr {
            if let Byml::Map(m) = item {
                let pos = parse_byml_location(m.get("Location"));
                let off_flag = m.get("OffFlag").and_then(|v| v.as_string().ok()).map(|s| s.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());

                list.push(QuestIndicator {
                    actor_name: None,
                    instance_name: None,
                    position: pos,
                    off_flag,
                    has_far: false,
                });
            }
        }
    }

    list
}

/// 从 Byml Map 中提取 (X, Y, Z) 三维坐标
fn parse_byml_location(loc_val: Option<&Byml>) -> (f32, f32, f32) {
    match loc_val {
        Some(Byml::Map(m)) => {
            let x = byml_to_f32(m.get("X"));
            let y = byml_to_f32(m.get("Y"));
            let z = byml_to_f32(m.get("Z"));
            (x, y, z)
        }
        _ => (0.0, 0.0, 0.0),
    }
}

/// 辅助函数：安全将 Byml 数值转化为 f32
fn byml_to_f32(val: Option<&Byml>) -> f32 {
    match val {
        Some(Byml::Float(f)) => *f,
        Some(Byml::Double(d)) => *d as f32,
        Some(Byml::I32(i)) => *i as f32,
        Some(Byml::U32(u)) => *u as f32,
        _ => 0.0,
    }
}
