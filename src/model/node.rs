use super::port::{Port, PortDirection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    EntryPoint,
    Action,
    Switch,
    Query,
    SubFlow,
    Fork,
    Join,
    Math,
    Comment,
}

impl NodeKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            NodeKind::EntryPoint => "Entry Point",
            NodeKind::Action => "Action",
            NodeKind::Switch => "Switch Branch",
            NodeKind::Query => "Query",
            NodeKind::SubFlow => "SubFlow",
            NodeKind::Fork => "Fork Parallel",
            NodeKind::Join => "Join Sync",
            NodeKind::Math => "Math / Logic",
            NodeKind::Comment => "Comment Note",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            NodeKind::EntryPoint => "🚩",
            NodeKind::Action => "⚡",
            NodeKind::Switch => "🔀",
            NodeKind::Query => "🔍",
            NodeKind::SubFlow => "📦",
            NodeKind::Fork => "🔱",
            NodeKind::Join => "🔗",
            NodeKind::Math => "🧮",
            NodeKind::Comment => "📝",
        }
    }

    pub fn header_color(&self) -> (f32, f32, f32) {
        // (Hue, Saturation, Lightness)
        match self {
            NodeKind::EntryPoint => (0.0, 0.75, 0.52),        // Crimson Red
            NodeKind::Action => (145.0 / 360.0, 0.65, 0.42), // Emerald Green
            NodeKind::Switch => (35.0 / 360.0, 0.85, 0.50),  // Amber Orange
            NodeKind::Query => (210.0 / 360.0, 0.75, 0.50),   // Sky Blue
            NodeKind::SubFlow => (275.0 / 360.0, 0.70, 0.55), // Purple
            NodeKind::Fork => (185.0 / 360.0, 0.70, 0.45),    // Teal
            NodeKind::Join => (170.0 / 360.0, 0.65, 0.40),    // Cyan Dark
            NodeKind::Math => (230.0 / 360.0, 0.70, 0.55),    // Indigo
            NodeKind::Comment => (215.0 / 360.0, 0.15, 0.35), // Slate Gray
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyItem {
    pub key: String,
    pub val: String,
    pub val_type: String, // "string", "number", "bool", "actor", "event"
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub kind: NodeKind,
    pub pos: (f32, f32),
    pub size: (f32, f32),
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub properties: Vec<PropertyItem>,
    pub selected: bool,
    pub collapsed: bool,
}

#[allow(dead_code)]
impl NodeData {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        kind: NodeKind,
        pos: (f32, f32),
    ) -> Self {
        let id_str = id.into();
        let (inputs, outputs, size) = match kind {
            NodeKind::EntryPoint => (
                vec![],
                vec![Port::new_flow_out(format!("{}_out", id_str), "Start")],
                (200.0, 88.0),
            ),
            NodeKind::Action => (
                vec![Port::new_flow_in(format!("{}_in", id_str), "Exec")],
                vec![Port::new_flow_out(format!("{}_out", id_str), "Out")],
                (240.0, 130.0),
            ),
            NodeKind::Switch => (
                vec![Port::new_flow_in(format!("{}_in", id_str), "Exec")],
                vec![
                    Port::new_condition_out(format!("{}_case0", id_str), "Case 0"),
                    Port::new_condition_out(format!("{}_case1", id_str), "Case 1"),
                ],
                (240.0, 150.0),
            ),
            NodeKind::Query => (
                vec![Port::new_flow_in(format!("{}_in", id_str), "Exec")],
                vec![
                    Port::new_flow_out(format!("{}_out", id_str), "Out"),
                    Port::new_data_out(format!("{}_res", id_str), "Result"),
                ],
                (230.0, 120.0),
            ),
            NodeKind::SubFlow => (
                vec![Port::new_flow_in(format!("{}_in", id_str), "In")],
                vec![Port::new_flow_out(format!("{}_out", id_str), "Out")],
                (230.0, 110.0),
            ),
            NodeKind::Fork => (
                vec![Port::new_flow_in(format!("{}_in", id_str), "In")],
                vec![
                    Port::new_flow_out(format!("{}_fork0", id_str), "Branch 1"),
                    Port::new_flow_out(format!("{}_fork1", id_str), "Branch 2"),
                ],
                (210.0, 130.0),
            ),
            NodeKind::Join => (
                vec![
                    Port::new_flow_in(format!("{}_join0", id_str), "Join 1"),
                    Port::new_flow_in(format!("{}_join1", id_str), "Join 2"),
                ],
                vec![Port::new_flow_out(format!("{}_out", id_str), "Out")],
                (210.0, 130.0),
            ),
            NodeKind::Math => (
                vec![
                    Port::new_data_in(format!("{}_a", id_str), "A"),
                    Port::new_data_in(format!("{}_b", id_str), "B"),
                ],
                vec![Port::new_data_out(format!("{}_res", id_str), "Out")],
                (200.0, 120.0),
            ),
            NodeKind::Comment => (
                vec![],
                vec![],
                (280.0, 160.0),
            ),
        };

        Self {
            id: id_str,
            title: title.into(),
            subtitle: None,
            kind,
            pos,
            size,
            inputs,
            outputs,
            properties: Vec::new(),
            selected: false,
            collapsed: false,
        }
    }

    pub fn with_subtitle(mut self, sub: impl Into<String>) -> Self {
        self.subtitle = Some(sub.into());
        self
    }

    pub fn with_property(mut self, key: impl Into<String>, val: impl Into<String>, val_type: impl Into<String>) -> Self {
        self.properties.push(PropertyItem {
            key: key.into(),
            val: val.into(),
            val_type: val_type.into(),
        });
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<Port>) -> Self {
        self.outputs = outputs;
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<Port>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn find_input_port(&self, port_id: &str) -> Option<&Port> {
        self.inputs.iter().find(|p| p.id == port_id)
    }

    pub fn find_output_port(&self, port_id: &str) -> Option<&Port> {
        self.outputs.iter().find(|p| p.id == port_id)
    }

    pub fn get_port_offset(&self, port_id: &str, dir: PortDirection) -> (f32, f32) {
        match dir {
            PortDirection::Input => {
                let idx = self.inputs.iter().position(|p| p.id == port_id).unwrap_or(0);
                let total = self.inputs.len().max(1);
                let step = self.size.0 / (total as f32 + 1.0);
                let offset_x = step * (idx as f32 + 1.0);
                (offset_x, 0.0)
            }
            PortDirection::Output => {
                let idx = self.outputs.iter().position(|p| p.id == port_id).unwrap_or(0);
                let total = self.outputs.len().max(1);
                let step = self.size.0 / (total as f32 + 1.0);
                let offset_x = step * (idx as f32 + 1.0);
                (offset_x, self.size.1)
            }
        }
    }

    pub fn get_port_offset_y(&self, port_id: &str, dir: PortDirection) -> f32 {
        self.get_port_offset(port_id, dir).1
    }
}
