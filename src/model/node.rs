use super::actor::{ActorTheme, KeyHighlight};
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
            NodeKind::EntryPoint => "ENTRY",
            NodeKind::Action => "ACTION",
            NodeKind::Switch => "SWITCH",
            NodeKind::Query => "QUERY",
            NodeKind::SubFlow => "SUBFLOW",
            NodeKind::Fork => "FORK",
            NodeKind::Join => "JOIN",
            NodeKind::Math => "MATH",
            NodeKind::Comment => "NOTE",
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

    pub fn badge_color(&self) -> gpui::Hsla {
        match self {
            NodeKind::EntryPoint => gpui::hsla(160.0 / 360.0, 0.84, 0.39, 1.0), // Emerald #10b981
            NodeKind::Action => gpui::hsla(200.0 / 360.0, 0.98, 0.39, 1.0),     // Sky Blue #0284c7
            NodeKind::Switch => gpui::hsla(38.0 / 360.0, 0.92, 0.45, 1.0),      // Amber #d97706
            NodeKind::Query => gpui::hsla(210.0 / 360.0, 0.80, 0.50, 1.0),
            NodeKind::SubFlow => gpui::hsla(271.0 / 360.0, 0.81, 0.56, 1.0),    // Purple #7c3aed
            NodeKind::Fork => gpui::hsla(142.0 / 360.0, 0.72, 0.36, 1.0),       // Green #16a34a
            NodeKind::Join => gpui::hsla(142.0 / 360.0, 0.72, 0.36, 1.0),       // Green #16a34a
            NodeKind::Math => gpui::hsla(230.0 / 360.0, 0.70, 0.55, 1.0),
            NodeKind::Comment => gpui::hsla(215.0 / 360.0, 0.20, 0.35, 1.0),
        }
    }

    pub fn border_color(&self) -> gpui::Hsla {
        match self {
            NodeKind::EntryPoint => gpui::hsla(160.0 / 360.0, 0.84, 0.28, 1.0),
            NodeKind::Action => gpui::hsla(200.0 / 360.0, 0.98, 0.39, 0.45),
            NodeKind::Switch => gpui::hsla(38.0 / 360.0, 0.92, 0.45, 0.45),
            NodeKind::Query => gpui::hsla(210.0 / 360.0, 0.80, 0.50, 0.45),
            NodeKind::SubFlow => gpui::hsla(271.0 / 360.0, 0.81, 0.56, 0.45),
            NodeKind::Fork => gpui::hsla(142.0 / 360.0, 0.72, 0.36, 0.45),
            NodeKind::Join => gpui::hsla(142.0 / 360.0, 0.72, 0.36, 0.45),
            NodeKind::Math => gpui::hsla(230.0 / 360.0, 0.70, 0.55, 0.45),
            NodeKind::Comment => gpui::hsla(215.0 / 360.0, 0.20, 0.35, 0.45),
        }
    }

    pub fn header_color(&self) -> (f32, f32, f32) {
        match self {
            NodeKind::EntryPoint => (160.0 / 360.0, 0.84, 0.39),
            NodeKind::Action => (200.0 / 360.0, 0.98, 0.39),
            NodeKind::Switch => (38.0 / 360.0, 0.92, 0.45),
            NodeKind::Query => (210.0 / 360.0, 0.75, 0.50),
            NodeKind::SubFlow => (271.0 / 360.0, 0.81, 0.56),
            NodeKind::Fork => (142.0 / 360.0, 0.72, 0.36),
            NodeKind::Join => (142.0 / 360.0, 0.72, 0.36),
            NodeKind::Math => (230.0 / 360.0, 0.70, 0.55),
            NodeKind::Comment => (215.0 / 360.0, 0.15, 0.35),
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

    /// 依据 Actor 名称识别专属角色色彩主题
    pub fn actor_theme(&self) -> ActorTheme {
        super::actor::get_node_actor_theme(self)
    }

    /// 提炼核心高频业务参数，过滤底层冗余参数
    pub fn extract_key_highlights(&self) -> Vec<KeyHighlight> {
        super::actor::extract_node_highlights(self)
    }
}


