use roead::byml::Byml;
use serde_json::Value;

/// 将 roead Byml 树形结构无缝转换为自然的非标签化 JSON 结构
pub fn byml_to_clean_json(byml: &Byml) -> Value {
    match byml {
        Byml::Null => Value::Null,
        Byml::Bool(b) => Value::Bool(*b),
        Byml::I32(i) => Value::Number((*i).into()),
        Byml::U32(u) => Value::Number((*u).into()),
        Byml::I64(i) => Value::Number((*i).into()),
        Byml::U64(u) => Value::Number((*u).into()),
        Byml::Float(f) => serde_json::Number::from_f64(*f as f64)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        Byml::Double(d) => serde_json::Number::from_f64(*d)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        Byml::String(s) => Value::String(s.to_string()),
        Byml::BinaryData(b) | Byml::FileData(b) => {
            Value::Array(b.iter().map(|byte| Value::Number((*byte).into())).collect())
        }
        Byml::Array(arr) => Value::Array(arr.iter().map(byml_to_clean_json).collect()),
        Byml::Map(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                obj.insert(k.to_string(), byml_to_clean_json(v));
            }
            Value::Object(obj)
        }
        Byml::HashMap(hmap) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in hmap {
                obj.insert(k.to_string(), byml_to_clean_json(v));
            }
            Value::Object(obj)
        }
        Byml::ValueHashMap(vhmap) => {
            let mut obj = serde_json::Map::new();
            for (k, (v, extra)) in vhmap {
                obj.insert(
                    k.to_string(),
                    serde_json::json!({
                        "value": byml_to_clean_json(v),
                        "extra": extra
                    }),
                );
            }
            Value::Object(obj)
        }
    }
}

/// 将标准 JSON 结构转换为 roead 原生 Byml 节点树
pub fn clean_json_to_byml(val: &Value) -> Byml {
    match val {
        Value::Null => Byml::Null,
        Value::Bool(b) => Byml::Bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                    Byml::I32(i as i32)
                } else {
                    Byml::I64(i)
                }
            } else if let Some(u) = n.as_u64() {
                if u <= u32::MAX as u64 {
                    Byml::U32(u as u32)
                } else {
                    Byml::U64(u)
                }
            } else if let Some(f) = n.as_f64() {
                Byml::Float(f as f32)
            } else {
                Byml::Null
            }
        }
        Value::String(s) => Byml::String(s.as_str().into()),
        Value::Array(arr) => Byml::Array(arr.iter().map(clean_json_to_byml).collect()),
        Value::Object(obj) => {
            let mut map = roead::byml::Map::default();
            for (k, v) in obj {
                map.insert(k.as_str().into(), clean_json_to_byml(v));
            }
            Byml::Map(map)
        }
    }
}
