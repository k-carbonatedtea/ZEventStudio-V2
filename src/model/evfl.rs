use super::edge::Edge;
use super::graph::Graph;
use super::node::{NodeData, NodeKind};
use super::port::Port;
use revfl::container::{Container, ContainerData};
use revfl::event::EventData;
use revfl::evfl::EventFlow;
use revfl::flowchart::Flowchart;

/// 从任天堂 BFEVFL 二进制字节流中加载并转换为 Graph 图结构 (完美对齐图2规范)
pub fn load_bfevfl_from_bytes(bytes: &[u8]) -> Result<Graph, String> {
    let mut evfl = EventFlow::new();
    evfl.read(bytes);

    if let Some(flowchart) = &evfl.flowchart {
        Ok(revfl_flowchart_to_graph(flowchart, &evfl.name))
    } else {
        Err("BFEVFL 文件中未包含有效的 Flowchart 流程图数据".to_string())
    }
}

/// 将 Revfl 的 Flowchart 数据结构转换为可视化 Graph
pub fn revfl_flowchart_to_graph(fc: &Flowchart, file_name: &str) -> Graph {
    let mut graph = Graph::new(format!("🗡️ {} (BFEVFL)", file_name));
    graph.description = format!(
        "任天堂官方 EVFL 事件流程图: {} (包含 {} 个事件, {} 个入口, {} 个参与 Actor)",
        fc.name,
        fc.events.len(),
        fc.entry_points.len(),
        fc.actors.len()
    );

    let events = &fc.events;
    let actors = &fc.actors;

    // 1. 生成 EntryPoints 顶部绿色胶囊节点 (对齐图2)
    for ep in &fc.entry_points {
        let ep_id = format!("ep_{}", ep.name);
        let mut ep_node = NodeData::new(
            ep_id.clone(),
            format!("Entry: {}", ep.name),
            NodeKind::EntryPoint,
            (0.0, 0.0),
        );
        ep_node.size = (180.0, 42.0);
        ep_node.subtitle = None;

        if !ep.sub_flow_event_indices.is_empty() {
            let callers: Vec<u16> = ep.sub_flow_event_indices.clone();
            ep_node = ep_node.with_property("SubFlowCallers", format!("{:?}", callers), "array");
        }

        graph.add_node(ep_node);

        // 连接 EntryPoint 到起始 Event
        let target_idx = ep.main_event.idx;
        if target_idx != 0xFFFF && (target_idx as usize) < events.len() {
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

    // 2. 生成所有 Events 节点 (对齐图2: 标题为 EventID, 副标题为 Actor :: Action)
    for ev in events {
        let ev_id = ev.name.clone();

        match &ev.data {
            EventData::Action(a) => {
                let actor_name = if a.actor.idx != 0xFFFF && (a.actor.idx as usize) < actors.len() {
                    actors[a.actor.idx as usize].identifier.name.clone()
                } else {
                    "Actor".to_string()
                };

                let action_name = if a.actor.idx != 0xFFFF && (a.actor.idx as usize) < actors.len() {
                    let act = &actors[a.actor.idx as usize];
                    if a.actor_action.idx != 0xFFFF && (a.actor_action.idx as usize) < act.actions.len() {
                        act.actions[a.actor_action.idx as usize].0.clone()
                    } else {
                        "Action".to_string()
                    }
                } else {
                    "Action".to_string()
                };

                let mut node = NodeData::new(ev_id.clone(), ev_id.clone(), NodeKind::Action, (0.0, 0.0));
                node.subtitle = Some(format!("{} :: {}", actor_name, action_name));
                node.size = (240.0, 115.0);

                if let Some(params) = &a.params {
                    add_container_properties(&mut node, params);
                }

                graph.add_node(node);

                // 连接至 NextEvent
                let next_idx = a.nxt.idx;
                if next_idx != 0xFFFF && (next_idx as usize) < events.len() {
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

            EventData::Switch(s) => {
                let actor_name = if s.actor.idx != 0xFFFF && (s.actor.idx as usize) < actors.len() {
                    actors[s.actor.idx as usize].identifier.name.clone()
                } else {
                    "Actor".to_string()
                };

                let query_name = if s.actor.idx != 0xFFFF && (s.actor.idx as usize) < actors.len() {
                    let act = &actors[s.actor.idx as usize];
                    if s.actor_query.idx != 0xFFFF && (s.actor_query.idx as usize) < act.queries.len() {
                        act.queries[s.actor_query.idx as usize].0.clone()
                    } else {
                        "Query".to_string()
                    }
                } else {
                    "Query".to_string()
                };

                let mut node = NodeData::new(ev_id.clone(), ev_id.clone(), NodeKind::Switch, (0.0, 0.0));
                node.subtitle = Some(format!("{} :: {}", actor_name, query_name));

                let mut out_ports = Vec::new();
                for (&case_val, _) in &s.cases {
                    let port_id = format!("{}_case_{}", ev_id, case_val);
                    let label = match case_val {
                        0 => "Case 0".to_string(),
                        1 => "Case 1".to_string(),
                        v => format!("Case {}", v),
                    };
                    out_ports.push(Port::new_condition_out(port_id, label));
                }
                if out_ports.is_empty() {
                    out_ports.push(Port::new_condition_out(format!("{}_case_0", ev_id), "Case 0"));
                }

                node.size = (240.0, 120.0);
                node.outputs = out_ports;

                if let Some(params) = &s.params {
                    add_container_properties(&mut node, params);
                }

                graph.add_node(node);

                for (&case_val, target_idx) in &s.cases {
                    let t_idx = target_idx.idx;
                    if t_idx != 0xFFFF && (t_idx as usize) < events.len() {
                        let target_ev = &events[t_idx as usize];
                        let source_port = format!("{}_case_{}", ev_id, case_val);
                        let edge = Edge::new(
                            &ev_id,
                            &source_port,
                            &target_ev.name,
                            &format!("{}_in", target_ev.name),
                        ).with_label(format!("Case {}", case_val));
                        graph.add_edge(edge);
                    }
                }
            }

            EventData::SubFlow(sf) => {
                let subtitle = if sf.res_flowchart_name.is_empty() {
                    sf.entry_point_name.clone()
                } else {
                    format!("{} :: {}", sf.res_flowchart_name, sf.entry_point_name)
                };

                let mut node = NodeData::new(ev_id.clone(), ev_id.clone(), NodeKind::SubFlow, (0.0, 0.0));
                node.subtitle = Some(subtitle);
                node.size = (220.0, 105.0);

                if !sf.entry_point_name.is_empty() {
                    node = node.with_property("EntryPoint", &sf.entry_point_name, "string");
                }
                if !sf.res_flowchart_name.is_empty() {
                    node = node.with_property("Flowchart", &sf.res_flowchart_name, "string");
                }
                if let Some(params) = &sf.params {
                    add_container_properties(&mut node, params);
                }

                graph.add_node(node);

                let next_idx = sf.nxt.idx;
                if next_idx != 0xFFFF && (next_idx as usize) < events.len() {
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

            EventData::Fork(f) => {
                let mut node = NodeData::new(ev_id.clone(), ev_id.clone(), NodeKind::Fork, (0.0, 0.0));
                node.subtitle = Some(format!("Forks: {}", f.forks.len()));
                node.size = (210.0, 95.0);

                let mut out_ports = Vec::new();
                for (b_idx, _) in f.forks.iter().enumerate() {
                    out_ports.push(Port::new_flow_out(
                        format!("{}_fork_{}", ev_id, b_idx),
                        format!("Fork {}", b_idx),
                    ));
                }
                if out_ports.is_empty() {
                    out_ports.push(Port::new_flow_out(format!("{}_fork_0", ev_id), "Fork 0"));
                    out_ports.push(Port::new_flow_out(format!("{}_fork_1", ev_id), "Fork 1"));
                }
                node.outputs = out_ports;

                if f.join.idx != 0xFFFF {
                    node = node.with_property("JoinTarget", f.join.idx.to_string(), "number");
                }

                graph.add_node(node);

                for (b_idx, fork_target) in f.forks.iter().enumerate() {
                    let f_idx = fork_target.idx;
                    if f_idx != 0xFFFF && (f_idx as usize) < events.len() {
                        let target_ev = &events[f_idx as usize];
                        let edge = Edge::new(
                            &ev_id,
                            &format!("{}_fork_{}", ev_id, b_idx),
                            &target_ev.name,
                            &format!("{}_in", target_ev.name),
                        ).with_label(format!("Fork {}", b_idx));
                        graph.add_edge(edge);
                    }
                }
            }

            EventData::Join(j) => {
                let mut node = NodeData::new(ev_id.clone(), ev_id.clone(), NodeKind::Join, (0.0, 0.0));
                node.subtitle = Some("Wait for parallel join".to_string());
                node.size = (210.0, 95.0);

                graph.add_node(node);

                let next_idx = j.nxt.idx;
                if next_idx != 0xFFFF && (next_idx as usize) < events.len() {
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
        }
    }

    graph
}

/// 辅助函数：将 Revfl Container 参数字典转换为节点的 Properties
fn add_container_properties(node: &mut NodeData, container: &Container) {
    for (k, v) in &container.data {
        let (val_str, val_type) = match v {
            ContainerData::String(s) => (s.clone(), "string"),
            ContainerData::Int(i) => (i.to_string(), "int"),
            ContainerData::Float(f) => (format!("{:.2}", f), "float"),
            ContainerData::Bool(b) => (b.to_string(), "bool"),
            ContainerData::Argument(a) => (a.0.clone(), "argument"),
            ContainerData::ActorIdentifier(a) => (format!("{}:{}", a.name, a.sub_name), "actor"),
            ContainerData::IntArray(arr) => (format!("{:?}", arr), "array"),
            ContainerData::BoolArray(arr) => (format!("{:?}", arr), "array"),
            ContainerData::FloatArray(arr) => (format!("{:?}", arr), "array"),
            ContainerData::StringArray(arr) => (format!("{:?}", arr), "array"),
        };
        *node = node.clone().with_property(k, val_str, val_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_100enemy_bfevfl() {
        let bytes = include_bytes!("../../100enemy.bfevfl");
        let graph = load_bfevfl_from_bytes(bytes).expect("Failed to parse 100enemy.bfevfl with Revfl");
        assert_eq!(graph.nodes.len(), 89);
        assert_eq!(graph.edges.len(), 89);
    }
}
