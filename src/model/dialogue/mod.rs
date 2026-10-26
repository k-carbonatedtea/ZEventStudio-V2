pub mod cache;
pub mod parser;
pub mod resolver;

pub use cache::{DialogueCacheKey, DialogueCacheManager, DialogueItem, DialogueSource};
use crate::model::settings::GamePathSettings;
use crate::model::{GameLanguage, NodeData};

/// 从节点属性中提取消息文件标识与词条标签 (如 "EventFlowMsg/TwnObj_AncientOven_A_01", "talk20")
pub fn extract_node_message_target(node: &NodeData) -> Option<(String, String)> {
    let msg_prop = node.properties.iter().find(|p| {
        p.key.eq_ignore_ascii_case("MessageId")
            || p.key.eq_ignore_ascii_case("MessageID")
            || p.key.eq_ignore_ascii_case("Message")
    })?;

    let val = msg_prop.val.trim();
    if val.is_empty() {
        return None;
    }

    if let Some((file_part, label_part)) = val.split_once(':') {
        let file_key = file_part.trim().to_string();
        let label = label_part.trim().to_string();
        if !file_key.is_empty() && !label.is_empty() {
            return Some((file_key, label));
        }
    } else {
        // 无冒号情况：label 为整体，文件标识尝试从节点的所属角色名推导
        let label = val.to_string();
        let file_key = if let Some(actor) = &node.subtitle {
            format!("EventFlowMsg/{}", actor.trim())
        } else {
            "EventFlowMsg".to_string()
        };
        return Some((file_key, label));
    }

    None
}

/// 获取或解析指定节点的对白文本 (优先命中缓存，未命中则按优先级加载并缓存)
pub fn resolve_node_dialogue(
    node: &NodeData,
    language: GameLanguage,
    project_path: Option<&str>,
    game_paths: &GamePathSettings,
) -> Option<DialogueItem> {
    let (file_key, label) = extract_node_message_target(node)?;

    let cache = DialogueCacheManager::global();
    let cache_key = DialogueCacheKey {
        language,
        file_key: file_key.clone(),
    };

    // 1. 尝试从缓存中高速获取
    if let Some(item) = cache.get_entry(&cache_key, &label) {
        return Some(item);
    }

    // 2. 避免重复解析失败的文件 (若已知缓存中已有此文件但无此词条，则直接返回 None)
    if cache.contains_file(&cache_key) {
        return None;
    }

    // 3. 按规则顺序检索加载 (优先项目，回退 Dump)
    if let Some((entries, source, source_path)) =
        resolver::resolve_and_load_dialogue_file(&file_key, language, project_path, game_paths)
    {
        let target_text = entries.get(&label).cloned();
        cache.insert_file_entries(cache_key, entries, source, source_path.clone());

        return target_text.map(|text| DialogueItem {
            text,
            source,
            source_path,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PropertyItem;

    #[test]
    fn test_extract_node_message_target() {
        let mut node = NodeData::new("test", "test", crate::model::NodeKind::Action, (0.0, 0.0));
        node.properties.push(PropertyItem {
            key: "MessageId".to_string(),
            val: "EventFlowMsg/TwnObj_AncientOven_A_01:talk20".to_string(),
            val_type: "string".to_string(),
        });

        let target = extract_node_message_target(&node);
        assert_eq!(
            target,
            Some((
                "EventFlowMsg/TwnObj_AncientOven_A_01".to_string(),
                "talk20".to_string()
            ))
        );
    }

    #[test]
    fn test_real_dialogue_resolution() {
        if !std::path::Path::new(r"C:\Users\Administrator\Desktop\任务测试1").exists() {
            return;
        }
        let mut node = NodeData::new("test", "test", crate::model::NodeKind::Action, (0.0, 0.0));
        node.properties.push(PropertyItem {
            key: "MessageId".to_string(),
            val: "EventFlowMsg/TwnObj_AncientOven_A_01:talk20".to_string(),
            val_type: "string".to_string(),
        });

        let mut gp = GamePathSettings::default();
        gp.language = GameLanguage::CNzh;
        let proj = Some(r"C:\Users\Administrator\Desktop\任务测试1");

        let res = resolve_node_dialogue(&node, GameLanguage::CNzh, proj, &gp);
        if let Some(item) = res {
            assert_eq!(item.source, DialogueSource::Project);
            assert!(!item.text.is_empty());
        }
    }
}
