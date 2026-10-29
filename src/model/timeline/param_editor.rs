use revfl::container::{Container, ContainerData};

/// 参数条目展示与编辑模型
#[derive(Debug, Clone, PartialEq)]
pub struct ParamItemEntry {
    pub key: String,
    pub type_name: String,
    pub value_str: String,
}

/// 提取容器内的所有参数键值列表
pub fn get_container_param_list(container: &Container) -> Vec<ParamItemEntry> {
    let mut list = Vec::new();
    for (k, v) in &container.data {
        let (type_name, value_str) = match v {
            ContainerData::Int(val) => ("Int".to_string(), val.to_string()),
            ContainerData::Float(val) => ("Float".to_string(), format!("{:.3}", val)),
            ContainerData::Bool(val) => ("Bool".to_string(), val.to_string()),
            ContainerData::String(val) => ("String".to_string(), val.clone()),
            ContainerData::Argument(val) => ("Argument".to_string(), val.0.clone()),
            ContainerData::ActorIdentifier(val) => {
                ("Actor".to_string(), format!("{}:{}", val.name, val.sub_name))
            }
            ContainerData::IntArray(vals) => (
                "Int[]".to_string(),
                format!("[{}]", vals.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")),
            ),
            ContainerData::FloatArray(vals) => (
                "Float[]".to_string(),
                format!("[{}]", vals.iter().map(|n| format!("{:.2}", n)).collect::<Vec<_>>().join(", ")),
            ),
            ContainerData::BoolArray(vals) => (
                "Bool[]".to_string(),
                format!("[{}]", vals.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")),
            ),
            ContainerData::StringArray(vals) => (
                "String[]".to_string(),
                format!("[{}]", vals.join(", ")),
            ),
        };
        list.push(ParamItemEntry {
            key: k.clone(),
            type_name,
            value_str,
        });
    }
    list
}

/// 向容器中设置或添加参数项 (根据类型字符串自动解析)
#[allow(dead_code)]
pub fn set_container_param(container: &mut Container, key: &str, type_name: &str, raw_val: &str) {
    let data = match type_name {
        "Int" => ContainerData::Int(raw_val.parse::<i32>().unwrap_or(0)),
        "Float" => ContainerData::Float(raw_val.parse::<f32>().unwrap_or(0.0)),
        "Bool" => ContainerData::Bool(raw_val.eq_ignore_ascii_case("true") || raw_val == "1"),
        _ => ContainerData::String(raw_val.to_string()),
    };
    container.data.insert(key.to_string(), data);
}

/// 从容器中移除参数项
pub fn remove_container_param(container: &mut Container, key: &str) {
    container.data.shift_remove(key);
}
