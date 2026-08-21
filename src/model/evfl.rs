use super::edge::Edge;
use super::graph::Graph;
use super::node::{NodeData, NodeKind};
use super::port::Port;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_neg_one() -> i32 {
    -1
}

fn default_one() -> i32 {
    1
}

/// EVFL 顶层事件文件定义 (例如 100enemy.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvflFile {
    #[serde(rename = "FileName")]
    pub file_name: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Flowcharts", default)]
    pub flowcharts: HashMap<String, EvflFlowchart>,
    #[serde(rename = "Timelines", default)]
    pub timelines: Option<serde_json::Value>,
}

/// 流程图 (Flowchart) 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvflFlowchart {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Actors", default)]
    pub actors: Vec<EvflActor>,
    #[serde(rename = "Events", default)]
    pub events: Vec<EvflEvent>,
    #[serde(rename = "EntryPoints", default)]
    pub entry_points: HashMap<String, EvflEntryPoint>,
}

/// 参与事件的 Actor 对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvflActor {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "SecondaryName", default)]
    pub secondary_name: String,
    #[serde(rename = "ArgumentName", default)]
    pub argument_name: String,
    #[serde(rename = "EntryPointIndex", default = "default_neg_one")]
    pub entry_point_index: i32,
    #[serde(rename = "CutNumber", default = "default_one")]
    pub cut_number: i32,
    #[serde(rename = "Actions", default)]
    pub actions: Vec<String>,
    #[serde(rename = "Queries", default)]
    pub queries: Vec<String>,
    #[serde(rename = "Parameters", default)]
    pub parameters: HashMap<String, EvflParamValue>,
}

/// EVFL 参数值枚举 (支持 typed 对象如 {"String": "val"}, {"Int": 10}, {"Bool": true} 或直接值)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EvflParamValue {
    Typed(HashMap<String, serde_json::Value>),
    Direct(serde_json::Value),
}

impl EvflParamValue {
    pub fn to_display_string(&self) -> (String, String) {
        match self {
            EvflParamValue::Typed(map) => {
                if let Some((type_name, val)) = map.iter().next() {
                    let formatted_val = match val {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        other => other.to_string(),
                    };
                    (formatted_val, type_name.to_lowercase())
                } else {
                    ("".to_string(), "unknown".to_string())
                }
            }
            EvflParamValue::Direct(v) => match v {
                serde_json::Value::String(s) => (s.clone(), "string".to_string()),
                serde_json::Value::Number(n) => (n.to_string(), "number".to_string()),
                serde_json::Value::Bool(b) => (b.to_string(), "bool".to_string()),
                other => (other.to_string(), "json".to_string()),
            },
        }
    }
}

/// Switch 条件分支定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvflSwitchCase {
    #[serde(rename = "Value")]
    pub value: i32,
    #[serde(rename = "EventIndex", default = "default_neg_one")]
    pub event_index: i32,
    #[serde(rename = "EventName", default)]
    pub event_name: Option<String>,
}

/// EVFL 事件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvflEvent {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub event_type: String, // "Action", "Switch", "Subflow", "Fork", "Join"

    // Action & Switch 相关
    #[serde(rename = "ActorIndex", default)]
    pub actor_index: Option<i32>,
    #[serde(rename = "ActorName", default)]
    pub actor_name: Option<String>,
    #[serde(rename = "ActorActionIndex", default)]
    pub actor_action_index: Option<i32>,
    #[serde(rename = "ActorAction", default)]
    pub actor_action: Option<String>,
    #[serde(rename = "ActorQueryIndex", default)]
    pub actor_query_index: Option<i32>,
    #[serde(rename = "ActorQuery", default)]
    pub actor_query: Option<String>,
    #[serde(rename = "Parameters", default)]
    pub parameters: Option<HashMap<String, EvflParamValue>>,

    // 下一跳节点 (Action / Subflow / Join)
    #[serde(rename = "NextEventIndex", default = "default_neg_one")]
    pub next_event_index: i32,
    #[serde(rename = "NextEventName", default)]
    pub next_event_name: Option<String>,

    // Switch 分支
    #[serde(rename = "SwitchCases", default)]
    pub switch_cases: Option<Vec<EvflSwitchCase>>,

    // Subflow 子流程
    #[serde(rename = "FlowchartName", default)]
    pub flowchart_name: Option<String>,
    #[serde(rename = "EntryPointName", default)]
    pub entry_point_name: Option<String>,

    // Fork & Join 并行
    #[serde(rename = "JoinEventIndex", default = "default_neg_one")]
    pub join_event_index: i32,
    #[serde(alias = "ForkEventIndices", alias = "ForkEventIndicies", default)]
    pub fork_event_indices: Option<Vec<i32>>,
}

/// 入口点 (EntryPoint) 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvflEntryPoint {
    #[serde(rename = "EventIndex", default = "default_neg_one")]
    pub event_index: i32,
    #[serde(rename = "SubFlowEventIndices", default)]
    pub subflow_event_indices: Vec<i32>,
}

/// 将 EVFL JSON 字符串转换为 Graph 图结构
pub fn load_evfl_from_str(json_str: &str) -> Result<Graph, String> {
    let evfl_file: EvflFile = serde_json::from_str(json_str)
        .map_err(|e| format!("解析 EVFL JSON 失败: {}", e))?;

    // 获取主流程图 (若存在多个则优先取与文件名同名的，或第一个)
    let flowchart = if let Some(fc) = evfl_file.flowcharts.get(&evfl_file.file_name) {
        fc
    } else if let Some((_, fc)) = evfl_file.flowcharts.iter().next() {
        fc
    } else {
        return Err("EVFL 文件中未包含任何流程图 (Flowchart)".to_string());
    };

    Ok(flowchart_to_graph(flowchart, &evfl_file.file_name, &evfl_file.version))
}

/// 将单个 Flowchart 转换为可视化节点图
pub fn flowchart_to_graph(flowchart: &EvflFlowchart, file_name: &str, version: &str) -> Graph {
    let mut graph = Graph::new(format!("{} (v{})", file_name, version));
    graph.description = format!(
        "EVFL 事件流程图: {} (包含 {} 个事件, {} 个入口, {} 个参与对象)",
        flowchart.name,
        flowchart.events.len(),
        flowchart.entry_points.len(),
        flowchart.actors.len()
    );

    let events = &flowchart.events;

    // 1. 生成 EntryPoints 节点
    for (ep_name, ep_data) in &flowchart.entry_points {
        let ep_id = format!("ep_{}", ep_name);
        let mut ep_node = NodeData::new(
            ep_id.clone(),
            format!("🏁 {}", ep_name),
            NodeKind::EntryPoint,
            (0.0, 0.0),
        );
        ep_node.subtitle = Some("Entry Point".to_string());

        if !ep_data.subflow_event_indices.is_empty() {
            ep_node = ep_node.with_property(
                "SubFlowCallers",
                format!("{:?}", ep_data.subflow_event_indices),
                "array",
            );
        }

        graph.add_node(ep_node);

        // 连接 EntryPoint 到起始 Event
        let target_idx = ep_data.event_index;
        if target_idx >= 0 && (target_idx as usize) < events.len() {
            let target_ev = &events[target_idx as usize];
            let edge = Edge::new(
                &ep_id,
                &format!("{}_out", ep_id),
                &target_ev.name,
                &format!("{}_in", target_ev.name),
            );
            graph.add_edge(edge);
        }
    }

    // 2. 生成 Events 节点
    for ev in events {
        let ev_id = ev.name.clone();
        let ev_type = ev.event_type.as_str();

        match ev_type {
            "Action" => {
                let action_title = ev
                    .actor_action
                    .clone()
                    .unwrap_or_else(|| "Action".to_string());
                let actor_subtitle = ev
                    .actor_name
                    .clone()
                    .unwrap_or_else(|| "Actor".to_string());

                let mut node = NodeData::new(ev_id.clone(), action_title, NodeKind::Action, (0.0, 0.0));
                node.subtitle = Some(actor_subtitle);

                // 添加动作参数
                if let Some(params) = &ev.parameters {
                    for (k, v) in params {
                        let (val_str, val_type) = v.to_display_string();
                        node = node.with_property(k, val_str, val_type);
                    }
                }

                graph.add_node(node);

                // 添加连线至 NextEvent
                let next_idx = ev.next_event_index;
                if next_idx >= 0 && (next_idx as usize) < events.len() {
                    let next_ev = &events[next_idx as usize];
                    let edge = Edge::new(
                        &ev_id,
                        &format!("{}_out", ev_id),
                        &next_ev.name,
                        &format!("{}_in", next_ev.name),
                    );
                    graph.add_edge(edge);
                }
            }

            "Switch" => {
                let query_title = ev
                    .actor_query
                    .clone()
                    .unwrap_or_else(|| "Switch Query".to_string());
                let actor_subtitle = ev
                    .actor_name
                    .clone()
                    .unwrap_or_else(|| "Actor".to_string());

                let mut node = NodeData::new(ev_id.clone(), query_title, NodeKind::Switch, (0.0, 0.0));
                node.subtitle = Some(actor_subtitle);

                // 配置动态 Switch 分支端口
                let cases = ev.switch_cases.clone().unwrap_or_default();
                let mut out_ports = Vec::new();
                for c in &cases {
                    let port_id = format!("{}_case_{}", ev_id, c.value);
                    let label = match c.value {
                        0 => "0 (False / Default)".to_string(),
                        1 => "1 (True / Branch)".to_string(),
                        v => format!("Case {}", v),
                    };
                    out_ports.push(Port::new_condition_out(port_id, label));
                }
                if out_ports.is_empty() {
                    out_ports.push(Port::new_condition_out(format!("{}_case_0", ev_id), "Default"));
                }

                // 调整 Switch 节点尺寸以容纳多个分支
                let calculated_height = 80.0 + (cases.len().max(2) as f32) * 26.0;
                node.size = (250.0, calculated_height.max(140.0));
                node.outputs = out_ports;

                // 添加查询参数 (如 FlagName, Operator, Value)
                if let Some(params) = &ev.parameters {
                    for (k, v) in params {
                        let (val_str, val_type) = v.to_display_string();
                        node = node.with_property(k, val_str, val_type);
                    }
                }

                graph.add_node(node);

                // 添加分支连线
                for c in &cases {
                    let case_target_idx = c.event_index;
                    if case_target_idx >= 0 && (case_target_idx as usize) < events.len() {
                        let target_ev = &events[case_target_idx as usize];
                        let source_port = format!("{}_case_{}", ev_id, c.value);
                        let edge = Edge::new(
                            &ev_id,
                            &source_port,
                            &target_ev.name,
                            &format!("{}_in", target_ev.name),
                        );
                        graph.add_edge(edge);
                    }
                }
            }

            "Subflow" => {
                let entry_title = ev
                    .entry_point_name
                    .clone()
                    .unwrap_or_else(|| "SubFlow".to_string());
                let flow_name = ev.flowchart_name.clone().unwrap_or_default();
                let subtitle = if flow_name.is_empty() {
                    "SubFlow (Local)".to_string()
                } else {
                    format!("SubFlow: {}", flow_name)
                };

                let mut node = NodeData::new(ev_id.clone(), entry_title, NodeKind::SubFlow, (0.0, 0.0));
                node.subtitle = Some(subtitle);

                if let Some(ep) = &ev.entry_point_name {
                    node = node.with_property("EntryPoint", ep, "string");
                }
                if let Some(fc) = &ev.flowchart_name {
                    if !fc.is_empty() {
                        node = node.with_property("Flowchart", fc, "string");
                    }
                }

                graph.add_node(node);

                let next_idx = ev.next_event_index;
                if next_idx >= 0 && (next_idx as usize) < events.len() {
                    let next_ev = &events[next_idx as usize];
                    let edge = Edge::new(
                        &ev_id,
                        &format!("{}_out", ev_id),
                        &next_ev.name,
                        &format!("{}_in", next_ev.name),
                    );
                    graph.add_edge(edge);
                }
            }

            "Fork" => {
                let mut node = NodeData::new(ev_id.clone(), "🔱 并行分支 (Fork)", NodeKind::Fork, (0.0, 0.0));
                node.subtitle = Some("Parallel Split".to_string());

                let fork_indices = ev.fork_event_indices.clone().unwrap_or_default();
                let mut out_ports = Vec::new();
                for (b_idx, _) in fork_indices.iter().enumerate() {
                    out_ports.push(Port::new_flow_out(
                        format!("{}_fork_{}", ev_id, b_idx),
                        format!("Branch {}", b_idx + 1),
                    ));
                }
                if out_ports.is_empty() {
                    out_ports.push(Port::new_flow_out(format!("{}_fork_0", ev_id), "Branch 1"));
                    out_ports.push(Port::new_flow_out(format!("{}_fork_1", ev_id), "Branch 2"));
                }
                let calculated_height = 80.0 + (fork_indices.len().max(2) as f32) * 26.0;
                node.size = (230.0, calculated_height.max(130.0));
                node.outputs = out_ports;

                if ev.join_event_index >= 0 {
                    node = node.with_property("JoinTargetIndex", ev.join_event_index.to_string(), "number");
                }

                graph.add_node(node);

                // 添加各并行分支连线
                for (b_idx, &f_idx) in fork_indices.iter().enumerate() {
                    if f_idx >= 0 && (f_idx as usize) < events.len() {
                        let target_ev = &events[f_idx as usize];
                        let edge = Edge::new(
                            &ev_id,
                            &format!("{}_fork_{}", ev_id, b_idx),
                            &target_ev.name,
                            &format!("{}_in", target_ev.name),
                        );
                        graph.add_edge(edge);
                    }
                }
            }

            "Join" => {
                let mut node = NodeData::new(ev_id.clone(), "🔗 并行汇合 (Join)", NodeKind::Join, (0.0, 0.0));
                node.subtitle = Some("Parallel Sync".to_string());

                graph.add_node(node);

                let next_idx = ev.next_event_index;
                if next_idx >= 0 && (next_idx as usize) < events.len() {
                    let next_ev = &events[next_idx as usize];
                    let edge = Edge::new(
                        &ev_id,
                        &format!("{}_out", ev_id),
                        &next_ev.name,
                        &format!("{}_in", next_ev.name),
                    );
                    graph.add_edge(edge);
                }
            }

            other => {
                let node = NodeData::new(
                    ev_id.clone(),
                    format!("Event: {}", other),
                    NodeKind::Action,
                    (0.0, 0.0),
                );
                graph.add_node(node);
            }
        }
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_100enemy_json() {
        let json_str = include_str!("../../100enemy.json");
        let graph = load_evfl_from_str(json_str).expect("Failed to parse 100enemy.json");
        assert_eq!(graph.nodes.len(), 89);
        assert_eq!(graph.edges.len(), 89);
    }
}

