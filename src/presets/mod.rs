pub mod combat_ai;
pub mod quest_dialogue;
pub mod signal_math;
pub mod zelda_100enemy;

use crate::model::Graph;
pub use combat_ai::create_combat_ai_preset;
pub use quest_dialogue::create_quest_dialogue_preset;
pub use signal_math::create_signal_math_preset;
pub use zelda_100enemy::create_zelda_100enemy_preset;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
}

pub fn get_available_presets() -> Vec<PresetInfo> {
    vec![
        PresetInfo {
            id: "zelda_100enemy",
            name: "🗡️ 100enemy.bfevfl (Revfl 库二进制解析)",
            icon: "🗡️",
            description: "任天堂官方 BFEVFL 二进制解析：包含 10 个 EntryPoints、53 个 Action、19 个 Switch 分支、Fork/Join 与 Subflow，共 89 个节点及 89 条连线。",
        },
        PresetInfo {
            id: "quest_dialogue",
            name: "🎮 任务与对话事件流",
            icon: "🎮",
            description: "经典 RPG 任务流：入口、状态查询、3路分支、并行发放道具/音效、汇合与存档子流程。",
        },
        PresetInfo {
            id: "combat_ai",
            name: "⚔️ 战斗 AI 决策状态机",
            icon: "⚔️",
            description: "敌人行为决策：巡逻、索敌、近战斩击/远程火弓路由与战术烟雾弹撤退。",
        },
        PresetInfo {
            id: "signal_math",
            name: "🧮 信号处理与计算流",
            icon: "🧮",
            description: "数值计算数据流：多源传感器输入、加法器、增益放大、阈值路由与散热执行器。",
        },
    ]
}

pub fn get_startup_preset() -> Graph {
    create_zelda_100enemy_preset()
}

pub fn load_preset_by_id(id: &str) -> Option<Graph> {
    match id {
        "zelda_100enemy" => Some(create_zelda_100enemy_preset()),
        "quest_dialogue" => Some(create_quest_dialogue_preset()),
        "combat_ai" => Some(create_combat_ai_preset()),
        "signal_math" => Some(create_signal_math_preset()),
        _ => None,
    }
}
