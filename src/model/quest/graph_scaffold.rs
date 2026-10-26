use crate::model::edge::Edge;
use crate::model::graph::Graph;
use crate::model::node::{NodeData, NodeKind, PropertyItem};
use crate::model::quest::types::QuestScaffoldConfig;

/// 向当前流程图 Graph 注入标准的接取与完成任务节点拓扑
pub fn scaffold_quest_graph_nodes(graph: &mut Graph, config: &QuestScaffoldConfig) -> Vec<String> {
    let mut created_node_ids = Vec::new();

    // 计算放置基准坐标 (在现有图右侧或下方)
    let (base_x, base_y) = find_scaffold_spawn_pos(graph);
    let orderer = if config.orderer.is_empty() {
        "EventSystemActor".to_string()
    } else {
        config.orderer.clone()
    };

    let ready_flag = if config.ready_flag.is_empty() {
        format!("{}_Ready", config.name)
    } else {
        config.ready_flag.clone()
    };
    let act_flag = if config.activated_flag.is_empty() {
        format!("{}_Activated", config.name)
    } else {
        config.activated_flag.clone()
    };
    let fin_flag = if config.finish_flag.is_empty() {
        format!("{}_Finish", config.name)
    } else {
        config.finish_flag.clone()
    };

    // 1. 接取链路 (Ready 序列)
    // 节点 1: Demo_FlagON(<Quest>_Ready)
    let node1_id = format!("Quest_{}_FlagReady", config.name);
    let node1 = create_action_node(
        &node1_id,
        "EventSystemActor",
        "Demo_FlagON",
        vec![
            PropertyItem {
                key: "FlagName".into(),
                val: ready_flag,
                val_type: "string".into(),
            },
            PropertyItem {
                key: "IsWaitFinish".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
        ],
        (base_x, base_y),
    );

    // 节点 2: Demo_FlagON(<Quest>_Activated)
    let node2_id = format!("Quest_{}_FlagAct", config.name);
    let node2 = create_action_node(
        &node2_id,
        "EventSystemActor",
        "Demo_FlagON",
        vec![
            PropertyItem {
                key: "FlagName".into(),
                val: act_flag,
                val_type: "string".into(),
            },
            PropertyItem {
                key: "IsWaitFinish".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
        ],
        (base_x + 280.0, base_y),
    );

    // 节点 3: Demo_AdvanceQuest("Ready")
    let node3_id = format!("Quest_{}_AdvReady", config.name);
    let node3 = create_action_node(
        &node3_id,
        "EventSystemActor",
        "Demo_AdvanceQuest",
        vec![
            PropertyItem {
                key: "QuestName".into(),
                val: config.name.clone(),
                val_type: "string".into(),
            },
            PropertyItem {
                key: "StepName".into(),
                val: "Ready".into(),
                val_type: "string".into(),
            },
            PropertyItem {
                key: "ForceRunTelop".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
            PropertyItem {
                key: "IsWaitFinish".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
        ],
        (base_x + 560.0, base_y),
    );

    // 节点 4: 接取对话 Demo_OpenMessageDialog
    let node4_id = format!("Quest_{}_TalkAct", config.name);
    let node4 = create_action_node(
        &node4_id,
        &orderer,
        "Demo_OpenMessageDialog",
        vec![
            PropertyItem {
                key: "MessageId".into(),
                val: format!("EventFlowMsg/{}:talk_quest_act", orderer),
                val_type: "string".into(),
            },
            PropertyItem {
                key: "IsWaitFinish".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
        ],
        (base_x + 840.0, base_y),
    );

    // 2. 完成链路 (Finish 序列)
    let fin_y = base_y + 180.0;

    // 节点 5: Demo_FlagON(<Quest>_Finish)
    let node5_id = format!("Quest_{}_FlagFin", config.name);
    let node5 = create_action_node(
        &node5_id,
        "EventSystemActor",
        "Demo_FlagON",
        vec![
            PropertyItem {
                key: "FlagName".into(),
                val: fin_flag,
                val_type: "string".into(),
            },
            PropertyItem {
                key: "IsWaitFinish".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
        ],
        (base_x, fin_y),
    );

    // 节点 6: Demo_AdvanceQuest("Finish")
    let node6_id = format!("Quest_{}_AdvFin", config.name);
    let node6 = create_action_node(
        &node6_id,
        "EventSystemActor",
        "Demo_AdvanceQuest",
        vec![
            PropertyItem {
                key: "QuestName".into(),
                val: config.name.clone(),
                val_type: "string".into(),
            },
            PropertyItem {
                key: "StepName".into(),
                val: "Finish".into(),
                val_type: "string".into(),
            },
            PropertyItem {
                key: "ForceRunTelop".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
            PropertyItem {
                key: "IsWaitFinish".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
        ],
        (base_x + 280.0, fin_y),
    );

    // 节点 7: 完成对话 Demo_OpenMessageDialog
    let node7_id = format!("Quest_{}_TalkFin", config.name);
    let node7 = create_action_node(
        &node7_id,
        &orderer,
        "Demo_OpenMessageDialog",
        vec![
            PropertyItem {
                key: "MessageId".into(),
                val: format!("EventFlowMsg/{}:talk_quest_fin", orderer),
                val_type: "string".into(),
            },
            PropertyItem {
                key: "IsWaitFinish".into(),
                val: "true".into(),
                val_type: "bool".into(),
            },
        ],
        (base_x + 560.0, fin_y),
    );

    created_node_ids.push(node1.id.clone());
    created_node_ids.push(node2.id.clone());
    created_node_ids.push(node3.id.clone());
    created_node_ids.push(node4.id.clone());
    created_node_ids.push(node5.id.clone());
    created_node_ids.push(node6.id.clone());
    created_node_ids.push(node7.id.clone());

    graph.add_node(node1);
    graph.add_node(node2);
    graph.add_node(node3);
    graph.add_node(node4);
    graph.add_node(node5);
    graph.add_node(node6);
    graph.add_node(node7);

    // 建立连线: 1 -> 2 -> 3 -> 4
    graph.edges.push(Edge::new(&node1_id, format!("{}_out", node1_id), &node2_id, format!("{}_in", node2_id)));
    graph.edges.push(Edge::new(&node2_id, format!("{}_out", node2_id), &node3_id, format!("{}_in", node3_id)));
    graph.edges.push(Edge::new(&node3_id, format!("{}_out", node3_id), &node4_id, format!("{}_in", node4_id)));

    // 建立连线: 5 -> 6 -> 7
    graph.edges.push(Edge::new(&node5_id, format!("{}_out", node5_id), &node6_id, format!("{}_in", node6_id)));
    graph.edges.push(Edge::new(&node6_id, format!("{}_out", node6_id), &node7_id, format!("{}_in", node7_id)));

    // 确保 Actor 列表同步
    graph.ensure_actors_populated();

    created_node_ids
}

/// 辅助创建 Action 节点
fn create_action_node(
    id: &str,
    actor: &str,
    action: &str,
    properties: Vec<PropertyItem>,
    pos: (f32, f32),
) -> NodeData {
    let mut node = NodeData::new(id, action, NodeKind::Action, pos);
    node.subtitle = Some(actor.to_string());
    node.properties = properties;
    node
}

/// 寻找放置脚手架节点组的空白区域
fn find_scaffold_spawn_pos(graph: &Graph) -> (f32, f32) {
    if graph.nodes.is_empty() {
        return (100.0, 100.0);
    }

    let mut max_x = 0.0f32;
    let mut avg_y = 0.0f32;

    for n in &graph.nodes {
        if n.pos.0 > max_x {
            max_x = n.pos.0;
        }
        avg_y += n.pos.1;
    }

    avg_y /= graph.nodes.len() as f32;
    (max_x + 320.0, avg_y.max(100.0))
}
