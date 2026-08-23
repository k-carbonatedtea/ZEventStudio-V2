use crate::model::PropertyItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI 程序基础定义信息 (Def)
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BaiDef {
    pub name: String,
    pub class_name: String,
    pub group_name: String,
}

/// 单个 AI 逻辑节点数据模型 (AI_0 .. AI_N-1)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BaiAiNode {
    pub id_index: u32,
    pub def: BaiDef,
    /// 子节点引脚分支映射 (引脚名 -> 全局节点索引)
    pub children: Vec<(String, u32)>,
    /// 附加行为修饰映射 (槽位名/哈希 -> Behavior 索引)
    pub behaviors: Vec<(String, u32)>,
    /// 动态实例参数 (SInst)
    pub params: Vec<PropertyItem>,
}

/// 单个 Action 动作节点数据模型 (Action_0 .. Action_M-1)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BaiActionNode {
    pub id_index: u32,
    pub def: BaiDef,
    pub behaviors: Vec<(String, u32)>,
    pub params: Vec<PropertyItem>,
}

/// 单个 Behavior 行为修饰器定义 (Behavior_0 .. Behavior_K-1)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BaiBehaviorDef {
    pub id_index: u32,
    pub class_name: String,
    pub params: Vec<PropertyItem>,
}

/// 单个 Query 查询定义 (Query_0 ..)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BaiQueryDef {
    pub id_index: u32,
    pub class_name: String,
    pub params: Vec<PropertyItem>,
}

/// 完整 BAI 程序数据容器模型 (AAMP ParameterIO 抽象中间层)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BaiProgram {
    /// 演示动画/过场动作映射 (动作名 -> 全局 Action 节点索引)
    pub demo_actions: HashMap<String, u32>,
    /// AI 逻辑节点集合
    pub ai_nodes: Vec<BaiAiNode>,
    /// Action 动作节点集合
    pub action_nodes: Vec<BaiActionNode>,
    /// Behavior 行为修饰器集合
    pub behaviors: Vec<BaiBehaviorDef>,
    /// Query 状态查询集合
    pub queries: Vec<BaiQueryDef>,
}

impl BaiProgram {
    /// 新建空 BAI 程序
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取 AI 节点总数
    pub fn num_ai(&self) -> usize {
        self.ai_nodes.len()
    }

    /// 获取 Action 节点总数
    pub fn num_actions(&self) -> usize {
        self.action_nodes.len()
    }

    /// 根据全局索引解析目标节点名称与标识
    pub fn resolve_child_target_id(&self, child_idx: u32) -> String {
        let num_ai = self.num_ai() as u32;
        if child_idx < num_ai {
            format!("ai_{}", child_idx)
        } else {
            let act_idx = child_idx - num_ai;
            format!("action_{}", act_idx)
        }
    }
}
