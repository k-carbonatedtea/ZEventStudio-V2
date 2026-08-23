use super::types::{BaiActionNode, BaiAiNode, BaiBehaviorDef, BaiDef, BaiProgram, BaiQueryDef};
use crate::model::node::PropertyItem;
use roead::aamp::{Parameter, ParameterIO, ParameterList, ParameterObject};

/// 解析 ParameterIO 生成 BaiProgram 中间结构
pub fn parse_pio_to_baiprog(pio: &ParameterIO) -> Result<BaiProgram, String> {
    let mut prog = BaiProgram::new();

    // 1. 解析 DemoAIActionIdx 对象
    if let Some(demo_obj) = pio.param_root.objects.get("DemoAIActionIdx") {
        for (k, v) in demo_obj.iter() {
            if let Some(val) = extract_param_u32(v) {
                prog.demo_actions.insert(k.to_string(), val);
            }
        }
    }

    // 2. 解析 Behavior 定义
    if let Some(beh_list) = pio.param_root.lists.get("Behavior") {
        parse_behavior_list(beh_list, &mut prog.behaviors);
    }

    // 3. 解析 Query 定义
    if let Some(q_list) = pio.param_root.lists.get("Query") {
        parse_query_list(q_list, &mut prog.queries);
    }

    // 4. 解析 AI 节点列表
    if let Some(ai_list) = pio.param_root.lists.get("AI") {
        parse_ai_list(ai_list, &mut prog.ai_nodes);
    }

    // 5. 解析 Action 节点列表
    if let Some(act_list) = pio.param_root.lists.get("Action") {
        parse_action_list(act_list, &mut prog.action_nodes);
    }

    Ok(prog)
}

/// 解析 Behavior 列表
fn parse_behavior_list(list: &ParameterList, target: &mut Vec<BaiBehaviorDef>) {
    for (i, (name, item_list)) in list.lists.iter().enumerate() {
        let name_str = name.to_string();
        let idx = if name_str.starts_with("Behavior_") {
            extract_index_from_name(&name_str, "Behavior_")
        } else {
            i as u32
        };
        let mut class_name = String::new();
        if let Some(def_obj) = item_list.objects.get("Def") {
            if let Some(val) = def_obj.get("ClassName") {
                class_name = extract_param_string(val).unwrap_or_default();
            }
        }
        let mut params = Vec::new();
        if let Some(sinst_obj) = item_list.objects.get("SInst") {
            extract_object_params(sinst_obj, &mut params);
        }
        target.push(BaiBehaviorDef {
            id_index: idx,
            class_name,
            params,
        });
    }
    target.sort_by_key(|b| b.id_index);
}

/// 解析 Query 列表
fn parse_query_list(list: &ParameterList, target: &mut Vec<BaiQueryDef>) {
    for (i, (name, item_list)) in list.lists.iter().enumerate() {
        let name_str = name.to_string();
        let idx = if name_str.starts_with("Query_") {
            extract_index_from_name(&name_str, "Query_")
        } else {
            i as u32
        };
        let mut class_name = String::new();
        if let Some(def_obj) = item_list.objects.get("Def") {
            if let Some(val) = def_obj.get("ClassName") {
                class_name = extract_param_string(val).unwrap_or_default();
            }
        }
        let mut params = Vec::new();
        if let Some(sinst_obj) = item_list.objects.get("SInst") {
            extract_object_params(sinst_obj, &mut params);
        }
        target.push(BaiQueryDef {
            id_index: idx,
            class_name,
            params,
        });
    }
    target.sort_by_key(|q| q.id_index);
}

/// 解析 AI 节点列表
fn parse_ai_list(list: &ParameterList, target: &mut Vec<BaiAiNode>) {
    for (i, (name, item_list)) in list.lists.iter().enumerate() {
        let name_str = name.to_string();
        let idx = if name_str.starts_with("AI_") {
            extract_index_from_name(&name_str, "AI_")
        } else {
            i as u32
        };
        let mut def = BaiDef::default();
        if let Some(def_obj) = item_list.objects.get("Def") {
            def = extract_bai_def(def_obj);
        }
        let mut children = Vec::new();
        if let Some(child_obj) = item_list.objects.get("ChildIdx") {
            for (k, v) in child_obj.iter() {
                if let Some(child_idx) = extract_param_u32(v) {
                    children.push((k.to_string(), child_idx));
                }
            }
        }
        let mut behaviors = Vec::new();
        if let Some(beh_obj) = item_list.objects.get("BehaviorIdx") {
            for (k, v) in beh_obj.iter() {
                if let Some(b_idx) = extract_param_u32(v) {
                    behaviors.push((k.to_string(), b_idx));
                }
            }
        }
        let mut params = Vec::new();
        if let Some(sinst_obj) = item_list.objects.get("SInst") {
            extract_object_params(sinst_obj, &mut params);
        }
        target.push(BaiAiNode {
            id_index: idx,
            def,
            children,
            behaviors,
            params,
        });
    }
    target.sort_by_key(|n| n.id_index);
}

/// 解析 Action 动作列表
fn parse_action_list(list: &ParameterList, target: &mut Vec<BaiActionNode>) {
    for (i, (name, item_list)) in list.lists.iter().enumerate() {
        let name_str = name.to_string();
        let idx = if name_str.starts_with("Action_") {
            extract_index_from_name(&name_str, "Action_")
        } else {
            i as u32
        };
        let mut def = BaiDef::default();
        if let Some(def_obj) = item_list.objects.get("Def") {
            def = extract_bai_def(def_obj);
        }
        let mut behaviors = Vec::new();
        if let Some(beh_obj) = item_list.objects.get("BehaviorIdx") {
            for (k, v) in beh_obj.iter() {
                if let Some(b_idx) = extract_param_u32(v) {
                    behaviors.push((k.to_string(), b_idx));
                }
            }
        }
        let mut params = Vec::new();
        if let Some(sinst_obj) = item_list.objects.get("SInst") {
            extract_object_params(sinst_obj, &mut params);
        }
        target.push(BaiActionNode {
            id_index: idx,
            def,
            behaviors,
            params,
        });
    }
    target.sort_by_key(|n| n.id_index);
}

/// 提取 Def 对象
fn extract_bai_def(def_obj: &ParameterObject) -> BaiDef {
    let mut def = BaiDef::default();
    if let Some(name_param) = def_obj.get("Name") {
        def.name = extract_param_string(name_param).unwrap_or_default();
    }
    if let Some(class_param) = def_obj.get("ClassName") {
        def.class_name = extract_param_string(class_param).unwrap_or_default();
    }
    if let Some(group_param) = def_obj.get("GroupName") {
        def.group_name = extract_param_string(group_param).unwrap_or_default();
    }
    def
}

/// 提取 ParameterObject 中的动态属性
fn extract_object_params(obj: &ParameterObject, params: &mut Vec<PropertyItem>) {
    for (k, v) in obj.iter() {
        let key_str = k.to_string();
        let (val_str, val_type) = match v {
            Parameter::Bool(b) => (b.to_string(), "bool"),
            Parameter::U32(u) => (u.to_string(), "int"),
            Parameter::I32(i) => (i.to_string(), "int"),
            Parameter::F32(f) => (format!("{}", f), "float"),
            Parameter::Vec3(v) => (format!("[{:.2}, {:.2}, {:.2}]", v.x, v.y, v.z), "vec3"),
            _ => {
                if let Ok(s) = v.as_str() {
                    (s.to_string(), "string")
                } else {
                    (format!("{:?}", v), "string")
                }
            }
        };
        params.push(PropertyItem {
            key: key_str,
            val: val_str,
            val_type: val_type.to_string(),
        });
    }
}

/// 辅助解析字符串参数
fn extract_param_string(param: &Parameter) -> Option<String> {
    param.as_str().ok().map(|s| s.to_string())
}

/// 辅助解析 u32/i32 参数
fn extract_param_u32(param: &Parameter) -> Option<u32> {
    match param {
        Parameter::U32(u) => Some(*u),
        Parameter::I32(i) => Some(*i as u32),
        _ => None,
    }
}

/// 辅助从 "AI_12" 中提取数字 12
fn extract_index_from_name(name: &str, prefix: &str) -> u32 {
    name.strip_prefix(prefix)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
}
