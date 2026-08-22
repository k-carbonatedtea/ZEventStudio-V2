use gpui::*;
use crate::model::node::{NodeData, NodeKind};

/// 角色身份主题分类
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorCategory {
    System,    // EventSystemActor, System, Global 逻辑系统
    Player,    // Player, Link, GameROMPlayer 玩家与主角
    Npc,       // TwnObj_..., Npc_..., DekuTree_... NPC 对白与互动
    Enemy,     // Enemy_..., Boss_... 敌人与战斗
    Cinematic, // Camera, Sound, Effect 镜头与音画演出
    Default,   // 其它基础节点
}

/// 角色身份视觉主题配置
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ActorTheme {
    pub category: ActorCategory,
    pub icon: &'static str,
    pub badge_bg: Hsla,
    pub border_color: Hsla,
    pub banner_bg: Hsla,
}

/// 提炼的关键高频业务参数（如对白、任务阶段、动作动画）
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct KeyHighlight {

    pub label: String,
    pub value: String,
    pub icon: &'static str,
    pub color: Hsla,
}

/// 依据 Actor 名称和节点类型识别专属角色色彩主题
pub fn get_node_actor_theme(node: &NodeData) -> ActorTheme {
    if node.kind == NodeKind::EntryPoint {
        return ActorTheme {
            category: ActorCategory::Player,
            icon: "🚩",
            badge_bg: hsla(160.0 / 360.0, 0.84, 0.39, 1.0),
            border_color: hsla(160.0 / 360.0, 0.84, 0.28, 1.0),
            banner_bg: hsla(160.0 / 360.0, 0.50, 0.20, 0.95),
        };
    }

    let actor_str = node.subtitle.as_deref().unwrap_or(&node.title);
    let lower = actor_str.to_lowercase();

    if lower.contains("eventsystem") || lower.contains("system") || lower.contains("gameflow") {
        ActorTheme {
            category: ActorCategory::System,
            icon: "⚙️",
            badge_bg: hsla(38.0 / 360.0, 0.92, 0.45, 1.0), // Amber #d97706
            border_color: hsla(38.0 / 360.0, 0.80, 0.45, 0.70),
            banner_bg: hsla(38.0 / 360.0, 0.45, 0.22, 0.95),
        }
    } else if lower.contains("player") || lower.contains("link") {
        ActorTheme {
            category: ActorCategory::Player,
            icon: "⚔️",
            badge_bg: hsla(152.0 / 360.0, 0.75, 0.38, 1.0), // Emerald Green
            border_color: hsla(152.0 / 360.0, 0.75, 0.38, 0.70),
            banner_bg: hsla(152.0 / 360.0, 0.40, 0.20, 0.95),
        }
    } else if lower.contains("twnobj") || lower.contains("npc") || lower.contains("dekutree") || lower.contains("korok") {
        ActorTheme {
            category: ActorCategory::Npc,
            icon: "💬",
            badge_bg: hsla(200.0 / 360.0, 0.90, 0.42, 1.0), // Sky Blue #0284c7
            border_color: hsla(200.0 / 360.0, 0.90, 0.42, 0.70),
            banner_bg: hsla(200.0 / 360.0, 0.40, 0.20, 0.95),
        }
    } else if lower.contains("enemy") || lower.contains("boss") {
        ActorTheme {
            category: ActorCategory::Enemy,
            icon: "💀",
            badge_bg: hsla(0.0, 0.85, 0.50, 1.0), // Crimson #ef4444
            border_color: hsla(0.0, 0.85, 0.50, 0.70),
            banner_bg: hsla(0.0, 0.45, 0.22, 0.95),
        }
    } else if lower.contains("camera") || lower.contains("sound") || lower.contains("effect") {
        ActorTheme {
            category: ActorCategory::Cinematic,
            icon: "🎬",
            badge_bg: hsla(271.0 / 360.0, 0.80, 0.55, 1.0), // Purple #7c3aed
            border_color: hsla(271.0 / 360.0, 0.80, 0.55, 0.70),
            banner_bg: hsla(271.0 / 360.0, 0.40, 0.22, 0.95),
        }
    } else {
        ActorTheme {
            category: ActorCategory::Default,
            icon: node.kind.icon(),
            badge_bg: node.kind.badge_color(),
            border_color: node.kind.border_color(),
            banner_bg: hsla(220.0 / 360.0, 0.25, 0.18, 0.95),
        }
    }
}

/// 提炼核心高频业务参数，过滤底层冗余参数
pub fn extract_node_highlights(node: &NodeData) -> Vec<KeyHighlight> {
    let mut highlights = Vec::new();

    // 1. 对白消息 (MessageId)
    if let Some(msg) = node.properties.iter().find(|p| p.key == "MessageId") {
        let short_msg = msg.val.split(':').last().unwrap_or(&msg.val);
        highlights.push(KeyHighlight {
            label: "对白".to_string(),
            value: short_msg.to_string(),
            icon: "💬",
            color: hsla(200.0 / 360.0, 0.85, 0.65, 1.0),
        });
    }

    // 2. 任务推进 (QuestName + StepName)
    let quest = node.properties.iter().find(|p| p.key == "QuestName");
    let step = node.properties.iter().find(|p| p.key == "StepName");
    if let Some(q) = quest {
        let val_str = if let Some(s) = step {
            format!("{} ➔ {}", q.val, s.val)
        } else {
            q.val.clone()
        };
        highlights.push(KeyHighlight {
            label: "任务".to_string(),
            value: val_str,
            icon: "🚩",
            color: hsla(45.0 / 360.0, 0.95, 0.65, 1.0),
        });
    }

    // 3. 动画演出 (ASName)
    if let Some(anim) = node.properties.iter().find(|p| p.key == "ASName") {
        highlights.push(KeyHighlight {
            label: "动作".to_string(),
            value: anim.val.clone(),
            icon: "🎭",
            color: hsla(271.0 / 360.0, 0.85, 0.75, 1.0),
        });
    }

    // 4. 标志位控制 (FlagName / FlagIndex)
    if let Some(flag) = node.properties.iter().find(|p| p.key == "FlagName" || p.key == "FlagIndex") {
        highlights.push(KeyHighlight {
            label: "标志".to_string(),
            value: flag.val.clone(),
            icon: "⚙️",
            color: hsla(38.0 / 360.0, 0.90, 0.65, 1.0),
        });
    }

    highlights
}
