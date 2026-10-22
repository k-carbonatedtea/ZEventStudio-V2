use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub source_node_id: String,
    pub source_port_id: String,
    pub target_node_id: String,
    pub target_port_id: String,
    pub label: Option<String>,
    pub color_override: Option<String>,
    pub is_dashed: bool,
    #[serde(skip_serializing, default)]
    pub selected: bool,
    #[serde(skip_serializing, default)]
    pub waypoints: Vec<(f32, f32)>,
}

impl Edge {
    pub fn new(
        source_node_id: impl Into<String>,
        source_port_id: impl Into<String>,
        target_node_id: impl Into<String>,
        target_port_id: impl Into<String>,
    ) -> Self {
        let s_node = source_node_id.into();
        let s_port = source_port_id.into();
        let t_node = target_node_id.into();
        let t_port = target_port_id.into();
        let id = format!("{}:{}->{}:{}", s_node, s_port, t_node, t_port);

        Self {
            id,
            source_node_id: s_node,
            source_port_id: s_port,
            target_node_id: t_node,
            target_port_id: t_port,
            label: None,
            color_override: None,
            is_dashed: false,
            selected: false,
            waypoints: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_waypoints(mut self, waypoints: Vec<(f32, f32)>) -> Self {
        self.waypoints = waypoints;
        self
    }



    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_dashed(mut self, dashed: bool) -> Self {
        self.is_dashed = dashed;
        self
    }
}
