use super::edge::Edge;
use super::graph::Graph;
use super::node::{NodeData, NodeKind};
use super::port::Port;
use revfl::container::{Container, ContainerData};
use revfl::event::EventData;
use revfl::evfl::EventFlow;
use revfl::flowchart::Flowchart;
use std::collections::HashSet;

/// 从任天堂 BFEVFL 二进制字节流中加载并转换为 Graph 图结构
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

    // 0. 载入所有参与 Actor 角色完整定义
    for a in actors {
        let mut actor_data = crate::model::actor::ActorData::new(a.identifier.name.clone());
        actor_data.sub_name = a.identifier.sub_name.clone();
        actor_data.argument_name = a.argument_name.clone();
        actor_data.actions = a.actions.iter().map(|act| act.0.clone()).collect();
        actor_data.queries = a.queries.iter().map(|q| q.0.clone()).collect();
        actor_data.load_default_params();
        graph.actors.push(actor_data);
    }

    // 1. 创建所有 EntryPoint 节点 (绿色胶囊)

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
    }

    // 2. 创建所有 Event 节点
    for ev in events {
        let ev_id = ev.name.clone();
        match &ev.data {
            EventData::Action(a) => {
                let (actor_name, action_name) = if a.actor.idx != 0xFFFF && (a.actor.idx as usize) < actors.len() {
                    let act = &actors[a.actor.idx as usize];
                    let act_name = if a.actor_action.idx != 0xFFFF && (a.actor_action.idx as usize) < act.actions.len() {
                        act.actions[a.actor_action.idx as usize].0.clone()
                    } else { "Action".to_string() };
                    (act.identifier.name.clone(), act_name)
                } else { ("Actor".to_string(), "Action".to_string()) };

                let mut node = NodeData::new(ev_id.clone(), ev_id.clone(), NodeKind::Action, (0.0, 0.0));
                node.subtitle = Some(format!("{} :: {}", actor_name, action_name));
                node.size = (240.0, 115.0);
                if let Some(params) = &a.params {
                    add_container_properties(&mut node, params);
                }
                graph.add_node(node);
            }

            EventData::Switch(s) => {
                let (actor_name, query_name) = if s.actor.idx != 0xFFFF && (s.actor.idx as usize) < actors.len() {
                    let act = &actors[s.actor.idx as usize];
                    let q_name = if s.actor_query.idx != 0xFFFF && (s.actor_query.idx as usize) < act.queries.len() {
                        act.queries[s.actor_query.idx as usize].0.clone()
                    } else { "Query".to_string() };
                    (act.identifier.name.clone(), q_name)
                } else { ("Actor".to_string(), "Query".to_string()) };

                let mut node = NodeData::new(ev_id.clone(), ev_id.clone(), NodeKind::Switch, (0.0, 0.0));
                node.subtitle = Some(format!("{} :: {}", actor_name, query_name));


                let mut out_ports = Vec::new();
                for (&case_val, _) in &s.cases {
                    let port_id = format!("{}_case_{}", ev_id, case_val);
                    let label = format!("Case {}", case_val);
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
            }

            EventData::Join(_) => {
                let mut node = NodeData::new(ev_id.clone(), ev_id.clone(), NodeKind::Join, (0.0, 0.0));
                node.subtitle = Some("Wait for parallel join".to_string());
                node.size = (210.0, 95.0);
                graph.add_node(node);
            }
        }
    }

    // 3. 构建完整的流程连线 (支持 Fork/Join 栈回溯与虚线连接)
    let mut visited: HashSet<usize> = HashSet::new();

    fn traverse_flow(
        curr_idx: usize,
        join_stack: &mut Vec<usize>,
        events: &[revfl::event::Event],
        graph: &mut Graph,
        visited: &mut HashSet<usize>,
    ) {
        if visited.contains(&curr_idx) || curr_idx >= events.len() {
            return;
        }
        visited.insert(curr_idx);

        let ev = &events[curr_idx];
        let ev_id = &ev.name;

        match &ev.data {
            EventData::Action(a) => {
                let next_idx = a.nxt.idx;
                if next_idx != 0xFFFF && (next_idx as usize) < events.len() {
                    let next_ev = &events[next_idx as usize];
                    graph.add_edge(Edge::new(
                        ev_id,
                        format!("{}_out", ev_id),
                        &next_ev.name,
                        format!("{}_in", next_ev.name),
                    ));
                    traverse_flow(next_idx as usize, join_stack, events, graph, visited);
                } else if let Some(&join_target_idx) = join_stack.last() {
                    // 分支结束，添加汇合虚线至 Join 节点
                    if join_target_idx < events.len() {
                        let join_ev = &events[join_target_idx];
                        graph.add_edge(
                            Edge::new(
                                ev_id,
                                format!("{}_out", ev_id),
                                &join_ev.name,
                                format!("{}_in", join_ev.name),
                            )
                            .with_dashed(true),
                        );
                    }
                }
            }

            EventData::SubFlow(sf) => {
                let next_idx = sf.nxt.idx;
                if next_idx != 0xFFFF && (next_idx as usize) < events.len() {
                    let next_ev = &events[next_idx as usize];
                    graph.add_edge(Edge::new(
                        ev_id,
                        format!("{}_out", ev_id),
                        &next_ev.name,
                        format!("{}_in", next_ev.name),
                    ));
                    traverse_flow(next_idx as usize, join_stack, events, graph, visited);
                } else if let Some(&join_target_idx) = join_stack.last() {
                    if join_target_idx < events.len() {
                        let join_ev = &events[join_target_idx];
                        graph.add_edge(
                            Edge::new(
                                ev_id,
                                format!("{}_out", ev_id),
                                &join_ev.name,
                                format!("{}_in", join_ev.name),
                            )
                            .with_dashed(true),
                        );
                    }
                }
            }

            EventData::Switch(s) => {
                for (&case_val, target_idx) in &s.cases {
                    let t_idx = target_idx.idx;
                    if t_idx != 0xFFFF && (t_idx as usize) < events.len() {
                        let target_ev = &events[t_idx as usize];
                        let source_port = format!("{}_case_{}", ev_id, case_val);
                        graph.add_edge(
                            Edge::new(
                                ev_id,
                                source_port,
                                &target_ev.name,
                                format!("{}_in", target_ev.name),
                            )
                            .with_label(format!("Case {}", case_val)),
                        );
                        traverse_flow(t_idx as usize, join_stack, events, graph, visited);
                    }
                }
            }

            EventData::Fork(f) => {
                let join_idx = f.join.idx;
                let has_valid_join = join_idx != 0xFFFF && (join_idx as usize) < events.len();

                if has_valid_join {
                    join_stack.push(join_idx as usize);
                }

                // 遍历各个并发分支
                for (b_idx, fork_target) in f.forks.iter().enumerate() {
                    let f_idx = fork_target.idx;
                    if f_idx != 0xFFFF && (f_idx as usize) < events.len() {
                        let target_ev = &events[f_idx as usize];
                        graph.add_edge(
                            Edge::new(
                                ev_id,
                                format!("{}_fork_{}", ev_id, b_idx),
                                &target_ev.name,
                                format!("{}_in", target_ev.name),
                            )
                            .with_label(format!("Fork {}", b_idx)),
                        );
                        traverse_flow(f_idx as usize, join_stack, events, graph, visited);
                    }
                }

                if has_valid_join {
                    join_stack.pop();
                    // 继续从 Join 节点向下遍历
                    traverse_flow(join_idx as usize, join_stack, events, graph, visited);
                }
            }

            EventData::Join(j) => {
                let next_idx = j.nxt.idx;
                if next_idx != 0xFFFF && (next_idx as usize) < events.len() {
                    let next_ev = &events[next_idx as usize];
                    graph.add_edge(Edge::new(
                        ev_id,
                        format!("{}_out", ev_id),
                        &next_ev.name,
                        format!("{}_in", next_ev.name),
                    ));
                    traverse_flow(next_idx as usize, join_stack, events, graph, visited);
                }
            }
        }
    }

    // 从所有 EntryPoint 开始执行遍历
    for ep in &fc.entry_points {
        let ep_id = format!("ep_{}", ep.name);
        let target_idx = ep.main_event.idx;
        if target_idx != 0xFFFF && (target_idx as usize) < events.len() {
            let target_ev = &events[target_idx as usize];
            graph.add_edge(Edge::new(
                &ep_id,
                format!("{}_out", ep_id),
                &target_ev.name,
                format!("{}_in", target_ev.name),
            ));
            let mut j_stack = Vec::new();
            traverse_flow(target_idx as usize, &mut j_stack, events, &mut graph, &mut visited);
        }
    }

    // 补充遍历剩余未被直接连接的游离节点
    for i in 0..events.len() {
        if !visited.contains(&i) {
            let mut j_stack = Vec::new();
            traverse_flow(i, &mut j_stack, events, &mut graph, &mut visited);
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
        // 包含 Fork/Join 汇合虚线
        assert!(graph.edges.iter().any(|e| e.is_dashed));
    }
}
