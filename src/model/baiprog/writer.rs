use super::types::{BaiActionNode, BaiAiNode, BaiBehaviorDef, BaiDef, BaiProgram, BaiQueryDef};
use crate::model::graph::Graph;
use crate::model::node::NodeData;
use roead::aamp::{Parameter, ParameterIO, ParameterList, ParameterObject};

/// 将 Graph 流程图结构重新序列化为 AAMP 二进制或 YAML 文本字节流
pub fn save_baiprog_to_bytes(graph: &Graph, format_is_text: bool) -> Result<Vec<u8>, String> {
    let prog = graph_to_baiprog(graph);
    let pio = baiprog_to_pio(&prog);

    if format_is_text {
        Ok(pio.to_text().into_bytes())
    } else {
        Ok(pio.to_binary().to_vec())
    }
}

/// 从 Graph 节点拓扑中反向提取结构化 BaiProgram
pub fn graph_to_baiprog(graph: &Graph) -> BaiProgram {
    let mut prog = BaiProgram::new();

    // 1. 区分 AI 节点与 Action 节点，读取元数据
    let mut ai_node_data = Vec::new();
    let mut act_node_data = Vec::new();

    for node in &graph.nodes {
        if node.id == "__baiprog_meta__" {
            for p in &node.properties {
                if p.key == "BehaviorsJson" {
                    if let Ok(behaviors) = serde_json::from_str::<Vec<BaiBehaviorDef>>(&p.val) {
                        prog.behaviors = behaviors;
                    }
                } else if p.key == "QueriesJson" {
                    if let Ok(queries) = serde_json::from_str::<Vec<BaiQueryDef>>(&p.val) {
                        prog.queries = queries;
                    }
                }
            }
            continue;
        }
        if node.id.starts_with("ep_demo_") {
            continue;
        }
        if node.id.starts_with("ai_") || (!node.outputs.is_empty() && !node.id.starts_with("action_")) {
            ai_node_data.push(node);
        } else {
            act_node_data.push(node);
        }
    }

    // 按照原始索引排序
    ai_node_data.sort_by_key(|n| parse_node_index(&n.id));
    act_node_data.sort_by_key(|n| parse_node_index(&n.id));

    let num_ai = ai_node_data.len() as u32;

    // 2. 构造 AI 节点列表
    for (idx, node) in ai_node_data.iter().enumerate() {
        let ai_idx = parse_node_index(&node.id).unwrap_or(idx as u32);
        let def = extract_def_from_node(node);
        let mut children = Vec::new();

        // 收集引脚出度连线
        for port in &node.outputs {
            let branch_name = &port.name;
            if let Some(edge) = graph.edges.iter().find(|e| e.source_node_id == node.id && e.source_port_id == port.id) {
                let target_idx = if edge.target_node_id.starts_with("ai_") {
                    parse_node_index(&edge.target_node_id).unwrap_or(0)
                } else if edge.target_node_id.starts_with("action_") {
                    let local_act = parse_node_index(&edge.target_node_id).unwrap_or(0);
                    num_ai + local_act
                } else {
                    0
                };
                children.push((branch_name.clone(), target_idx));
            }
        }

        let (behaviors, params) = extract_behaviors_and_params(node);

        prog.ai_nodes.push(BaiAiNode {
            id_index: ai_idx,
            def,
            children,
            behaviors,
            params,
        });
    }

    // 3. 构造 Action 节点列表
    for (idx, node) in act_node_data.iter().enumerate() {
        let act_idx = parse_node_index(&node.id).unwrap_or(idx as u32);
        let def = extract_def_from_node(node);
        let (behaviors, params) = extract_behaviors_and_params(node);

        prog.action_nodes.push(BaiActionNode {
            id_index: act_idx,
            def,
            behaviors,
            params,
        });
    }

    // 4. 收集 Demo 入口连线
    for node in &graph.nodes {
        if let Some(demo_name) = node.id.strip_prefix("ep_demo_") {
            if let Some(edge) = graph.edges.iter().find(|e| e.source_node_id == node.id) {
                let act_local = parse_node_index(&edge.target_node_id).unwrap_or(0);
                prog.demo_actions.insert(demo_name.to_string(), num_ai + act_local);
            }
        }
    }

    prog
}

/// 将 BaiProgram 组装为 AAMP ParameterIO
pub fn baiprog_to_pio(prog: &BaiProgram) -> ParameterIO {
    let mut pio = ParameterIO::new();

    // 1. 构建 DemoAIActionIdx 对象
    if !prog.demo_actions.is_empty() {
        let mut demo_obj = ParameterObject::new();
        for (k, v) in &prog.demo_actions {
            demo_obj.insert(k.as_str(), Parameter::U32(*v));
        }
        pio.param_root.objects.insert("DemoAIActionIdx", demo_obj);
    }

    // 2. 构建 AI 列表
    let mut ai_list = ParameterList::new();
    for ai in &prog.ai_nodes {
        let mut item = ParameterList::new();
        item.objects.insert("Def", create_def_object(&ai.def));

        if !ai.children.is_empty() {
            let mut child_obj = ParameterObject::new();
            for (k, v) in &ai.children {
                child_obj.insert(k.as_str(), Parameter::U32(*v));
            }
            item.objects.insert("ChildIdx", child_obj);
        }

        if !ai.behaviors.is_empty() {
            let mut beh_obj = ParameterObject::new();
            for (k, v) in &ai.behaviors {
                beh_obj.insert(k.as_str(), Parameter::U32(*v));
            }
            item.objects.insert("BehaviorIdx", beh_obj);
        }

        if !ai.params.is_empty() {
            item.objects.insert("SInst", create_sinst_object(&ai.params));
        }

        let list_name = format!("AI_{}", ai.id_index);
        ai_list.lists.insert(list_name.as_str(), item);
    }
    pio.param_root.lists.insert("AI", ai_list);

    // 3. 构建 Action 列表
    let mut act_list = ParameterList::new();
    for act in &prog.action_nodes {
        let mut item = ParameterList::new();
        item.objects.insert("Def", create_def_object(&act.def));

        if !act.behaviors.is_empty() {
            let mut beh_obj = ParameterObject::new();
            for (k, v) in &act.behaviors {
                beh_obj.insert(k.as_str(), Parameter::U32(*v));
            }
            item.objects.insert("BehaviorIdx", beh_obj);
        }

        if !act.params.is_empty() {
            item.objects.insert("SInst", create_sinst_object(&act.params));
        }

        let list_name = format!("Action_{}", act.id_index);
        act_list.lists.insert(list_name.as_str(), item);
    }
    pio.param_root.lists.insert("Action", act_list);

    // 4. 构建 Behavior 列表
    if !prog.behaviors.is_empty() {
        let mut beh_list = ParameterList::new();
        for b in &prog.behaviors {
            let mut item = ParameterList::new();
            let mut def_obj = ParameterObject::new();
            def_obj.insert("ClassName", Parameter::String32(b.class_name.as_str().into()));
            item.objects.insert("Def", def_obj);

            if !b.params.is_empty() {
                item.objects.insert("SInst", create_sinst_object(&b.params));
            }
            let list_name = format!("Behavior_{}", b.id_index);
            beh_list.lists.insert(list_name.as_str(), item);
        }
        pio.param_root.lists.insert("Behavior", beh_list);
    }

    // 5. 构建 Query 列表
    if !prog.queries.is_empty() {
        let mut q_list = ParameterList::new();
        for q in &prog.queries {
            let mut item = ParameterList::new();
            let mut def_obj = ParameterObject::new();
            def_obj.insert("ClassName", Parameter::String32(q.class_name.as_str().into()));
            item.objects.insert("Def", def_obj);

            if !q.params.is_empty() {
                item.objects.insert("SInst", create_sinst_object(&q.params));
            }
            let list_name = format!("Query_{}", q.id_index);
            q_list.lists.insert(list_name.as_str(), item);
        }
        pio.param_root.lists.insert("Query", q_list);
    }

    pio
}

/// 辅助创建 Def 参数对象
fn create_def_object(def: &BaiDef) -> ParameterObject {
    let mut obj = ParameterObject::new();
    if !def.name.is_empty() {
        obj.insert("Name", Parameter::String32(def.name.as_str().into()));
    }
    if !def.class_name.is_empty() {
        obj.insert("ClassName", Parameter::String32(def.class_name.as_str().into()));
    }
    if !def.group_name.is_empty() {
        obj.insert("GroupName", Parameter::String32(def.group_name.as_str().into()));
    }
    obj
}

/// 辅助创建 SInst 参数对象
fn create_sinst_object(params: &[crate::model::PropertyItem]) -> ParameterObject {
    let mut obj = ParameterObject::new();
    for p in params {
        let param = match p.val_type.as_str() {
            "bool" => Parameter::Bool(p.val.parse::<bool>().unwrap_or(false)),
            "int" => Parameter::I32(p.val.parse::<i32>().unwrap_or(0)),
            "float" => Parameter::F32(p.val.parse::<f32>().unwrap_or(0.0)),
            _ => Parameter::String32(p.val.as_str().into()),
        };
        obj.insert(p.key.as_str(), param);
    }
    obj
}

/// 从节点属性中提取 Def
fn extract_def_from_node(node: &NodeData) -> BaiDef {
    let mut def = BaiDef {
        name: node.title.clone(),
        class_name: String::new(),
        group_name: String::new(),
    };

    if let Some(sub) = &node.subtitle {
        if let Some((_, class_part)) = sub.split_once(" :: ") {
            def.class_name = class_part.trim().to_string();
        }
    }

    for p in &node.properties {
        if p.key.eq_ignore_ascii_case("ClassName") {
            def.class_name = p.val.clone();
        } else if p.key.eq_ignore_ascii_case("GroupName") {
            def.group_name = p.val.clone();
        }
    }
    def
}

/// 从节点属性中提取行为与参数
fn extract_behaviors_and_params(node: &NodeData) -> (Vec<(String, u32)>, Vec<crate::model::PropertyItem>) {
    let mut behaviors = Vec::new();
    let mut params = Vec::new();

    for p in &node.properties {
        if p.key.starts_with("Behavior[") && p.key.ends_with(']') {
            let slot = p.key[9..p.key.len() - 1].to_string();
            let beh_idx = p.val.strip_prefix("Behavior_")
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            behaviors.push((slot, beh_idx));
        } else if !p.key.eq_ignore_ascii_case("ClassName") && !p.key.eq_ignore_ascii_case("GroupName") {
            params.push(p.clone());
        }
    }

    (behaviors, params)
}

/// 从节点 ID (如 "ai_5", "action_10") 中提取数值
fn parse_node_index(id: &str) -> Option<u32> {
    id.rsplit('_').next().and_then(|s| s.parse::<u32>().ok())
}
