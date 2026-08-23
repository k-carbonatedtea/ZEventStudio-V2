use super::parser::parse_pio_to_baiprog;
use super::types::{BaiBehaviorDef, BaiDef, BaiProgram};
use crate::model::edge::Edge;
use crate::model::graph::Graph;
use crate::model::layout::auto_layout_graph;
use crate::model::node::{NodeData, NodeKind, PropertyItem};
use crate::model::port::Port;
use roead::aamp::ParameterIO;

/// 从 AAMP 二进制或 YAML 文本字节流中加载并转换为 Graph 可视化图结构
pub fn load_baiprog_from_bytes(bytes: &[u8], file_name: &str) -> Result<Graph, String> {
    if bytes.is_empty() {
        return Err("文件内容为空".to_string());
    }

    // 1. 如果是 JSON 格式流程图，直接反序列化
    let trimmed = bytes
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .map(|i| &bytes[i..])
        .unwrap_or(bytes);
    if trimmed.starts_with(b"{") {
        if let Ok(graph) = serde_json::from_slice::<Graph>(bytes) {
            return Ok(graph);
        }
    }

    // 2. Yaz0 解压与 AAMP 识别
    let decompressed = if bytes.len() >= 4 && &bytes[0..4] == b"Yaz0" {
        roead::yaz0::decompress(bytes).unwrap_or_else(|_| bytes.to_vec())
    } else {
        bytes.to_vec()
    };

    let pio = if decompressed.len() >= 4 && &decompressed[0..4] == b"AAMP" {
        ParameterIO::from_binary(&decompressed).map_err(|e| format!("解析 AAMP 二进制失败: {:?}", e))?
    } else {
        let text = String::from_utf8_lossy(&decompressed);
        ParameterIO::from_text(&text).map_err(|e| format!("解析 AAMP 文本失败: {:?}", e))?
    };

    // 3. 将 ParameterIO 转化为结构化 BaiProgram
    let prog = parse_pio_to_baiprog(&pio)?;

    // 4. 将 BaiProgram 转化为 NodeFlow Graph
    let mut graph = baiprog_to_graph(&prog, file_name);
    auto_layout_graph(&mut graph);

    Ok(graph)
}

/// 将 BaiProgram 转换为 GPUI 可渲染的 Graph 节点网络
pub fn baiprog_to_graph(prog: &BaiProgram, file_name: &str) -> Graph {
    let mut graph = Graph::new(format!("🧠 {} (BAIProg)", file_name));

    // 将全局 Behavior 与 Query 元数据编码存储到 graph.description 中，无需在画布上生成孤立的假节点
    let meta_json = serde_json::json!({
        "file_name": file_name,
        "behaviors": prog.behaviors,
        "queries": prog.queries,
    });
    graph.description = format!("BAI_PROGRAM_METADATA:{}", meta_json);

    let num_ai = prog.num_ai() as u32;

    // 1. 构建所有 AI 节点
    for ai in &prog.ai_nodes {
        let node_id = format!("ai_{}", ai.id_index);
        let title = if !ai.def.name.is_empty() {
            super::dict::format_bilingual_title(&ai.def.name)
        } else {
            format!("AI_{}", ai.id_index)
        };
        let subtitle = format!("AI_{} :: {}", ai.id_index, ai.def.class_name);

        let kind = determine_ai_node_kind(&ai.def.name, &ai.def.class_name, ai.id_index);
        let mut node = NodeData::new(node_id.clone(), title, kind.clone(), (0.0, 0.0));
        node.subtitle = Some(subtitle);

        // 端口设置
        if kind != NodeKind::EntryPoint {
            node.inputs = vec![Port::new_flow_in(format!("{}_in", node_id), "In")];
        }

        let mut out_ports = Vec::new();
        for (branch_key, _) in &ai.children {
            let port_id = format!("{}_{}", node_id, branch_key);
            out_ports.push(Port::new_condition_out(port_id, branch_key.clone()));
        }
        if out_ports.is_empty() && kind != NodeKind::EntryPoint {
            out_ports.push(Port::new_flow_out(format!("{}_out", node_id), "Out"));
        }
        node.outputs = out_ports;

        // 注入属性
        populate_node_properties(&mut node, &ai.def, &ai.params, &ai.behaviors, &prog.behaviors);
        graph.add_node(node);
    }

    // 2. 构建所有 Action 节点
    for act in &prog.action_nodes {
        let node_id = format!("action_{}", act.id_index);
        let title = if !act.def.name.is_empty() {
            super::dict::format_bilingual_title(&act.def.name)
        } else {
            format!("Action_{}", act.id_index)
        };
        let subtitle = format!("Action_{} :: {}", act.id_index, act.def.class_name);

        let mut node = NodeData::new(node_id.clone(), title, NodeKind::Action, (0.0, 0.0));
        node.subtitle = Some(subtitle);
        node.inputs = vec![Port::new_flow_in(format!("{}_in", node_id), "Exec")];
        node.outputs = vec![];

        populate_node_properties(&mut node, &act.def, &act.params, &act.behaviors, &prog.behaviors);
        graph.add_node(node);
    }

    // 3. 构建 ChildIdx 连线拓扑
    for ai in &prog.ai_nodes {
        let src_id = format!("ai_{}", ai.id_index);
        for (branch_name, target_idx) in &ai.children {
            let src_port = format!("{}_{}", src_id, branch_name);
            let target_node_id = prog.resolve_child_target_id(*target_idx);
            let target_port_id = format!("{}_in", target_node_id);

            graph.add_edge(
                Edge::new(&src_id, src_port, &target_node_id, target_port_id)
                    .with_label(branch_name.clone()),
            );
        }
    }

    // 4. 构建 DemoAIActionIdx 入口连线
    for (demo_name, action_global_idx) in &prog.demo_actions {
        let ep_id = format!("ep_demo_{}", demo_name);
        let mut ep_node = NodeData::new(
            ep_id.clone(),
            format!("Demo: {}", demo_name),
            NodeKind::EntryPoint,
            (0.0, 0.0),
        );
        ep_node.size = (190.0, 42.0);
        ep_node.subtitle = Some("Demo AI Action".to_string());
        ep_node.outputs = vec![Port::new_flow_out(format!("{}_out", ep_id), "Exec")];
        graph.add_node(ep_node);

        if *action_global_idx >= num_ai {
            let act_target_id = format!("action_{}", *action_global_idx - num_ai);
            graph.add_edge(
                Edge::new(
                    &ep_id,
                    format!("{}_out", ep_id),
                    &act_target_id,
                    format!("{}_in", act_target_id),
                )
                .with_label("Demo"),
            );
        }
    }

    graph
}

/// 辅助判定 AI 节点的类型
fn determine_ai_node_kind(name: &str, class_name: &str, _id_idx: u32) -> NodeKind {
    if name.eq_ignore_ascii_case("root") || class_name.contains("Root") {
        NodeKind::EntryPoint
    } else if class_name.contains("Fork") {
        NodeKind::Fork
    } else if class_name.contains("Select") || class_name.contains("Selector") {
        NodeKind::Switch
    } else if class_name.contains("Seq") {
        NodeKind::SubFlow
    } else {
        NodeKind::Action
    }
}

/// 为节点填充 Def、SInst 参数与 Behavior 行为修饰器
fn populate_node_properties(
    node: &mut NodeData,
    def: &BaiDef,
    params: &[PropertyItem],
    behaviors: &[(String, u32)],
    all_behaviors: &[BaiBehaviorDef],
) {
    if !def.class_name.is_empty() {
        node.properties.push(PropertyItem {
            key: "ClassName".to_string(),
            val: def.class_name.clone(),
            val_type: "string".to_string(),
        });
    }
    if !def.group_name.is_empty() {
        node.properties.push(PropertyItem {
            key: "GroupName".to_string(),
            val: def.group_name.clone(),
            val_type: "string".to_string(),
        });
    }
    for p in params {
        node.properties.push(p.clone());
    }
    for (slot_key, beh_idx) in behaviors {
        let beh_name = if let Some(b) = all_behaviors.iter().find(|b| b.id_index == *beh_idx) {
            format!("Behavior_{} ({})", beh_idx, b.class_name)
        } else {
            format!("Behavior_{}", beh_idx)
        };
        node.properties.push(PropertyItem {
            key: format!("Behavior[{}]", slot_key),
            val: beh_name,
            val_type: "behavior".to_string(),
        });
    }
}
