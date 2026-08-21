use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortType {
    Flow,
    Condition,
    Data,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Port {
    pub id: String,
    pub name: String,
    pub dir: PortDirection,
    pub port_type: PortType,
    pub color_override: Option<String>,
}

impl Port {
    pub fn new_flow_in(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            dir: PortDirection::Input,
            port_type: PortType::Flow,
            color_override: None,
        }
    }

    pub fn new_flow_out(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            dir: PortDirection::Output,
            port_type: PortType::Flow,
            color_override: None,
        }
    }

    pub fn new_condition_out(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            dir: PortDirection::Output,
            port_type: PortType::Condition,
            color_override: Some("#f59e0b".to_string()),
        }
    }

    pub fn new_data_in(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            dir: PortDirection::Input,
            port_type: PortType::Data,
            color_override: Some("#38bdf8".to_string()),
        }
    }

    pub fn new_data_out(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            dir: PortDirection::Output,
            port_type: PortType::Data,
            color_override: Some("#38bdf8".to_string()),
        }
    }
}
