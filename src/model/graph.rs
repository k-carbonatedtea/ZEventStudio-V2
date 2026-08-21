use super::edge::Edge;
use super::node::NodeData;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    pub name: String,
    pub description: String,
    pub nodes: Vec<NodeData>,
    pub edges: Vec<Edge>,
}

#[allow(dead_code)]
impl Graph {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: NodeData) {
        self.nodes.push(node);
    }

    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.retain(|n| n.id != node_id);
        self.edges.retain(|e| e.source_node_id != node_id && e.target_node_id != node_id);
    }

    pub fn find_node(&self, node_id: &str) -> Option<&NodeData> {
        self.nodes.iter().find(|n| n.id == node_id)
    }

    pub fn find_node_mut(&mut self, node_id: &str) -> Option<&mut NodeData> {
        self.nodes.iter_mut().find(|n| n.id == node_id)
    }

    pub fn add_edge(&mut self, edge: Edge) {
        // Prevent duplicate edges
        if !self.edges.iter().any(|e| {
            e.source_node_id == edge.source_node_id
                && e.source_port_id == edge.source_port_id
                && e.target_node_id == edge.target_node_id
                && e.target_port_id == edge.target_port_id
        }) {
            self.edges.push(edge);
        }
    }

    pub fn remove_edge(&mut self, edge_id: &str) {
        self.edges.retain(|e| e.id != edge_id);
    }

    pub fn sever_edge(&mut self, source_node: &str, target_node: &str) {
        self.edges.retain(|e| {
            !(e.source_node_id == source_node && e.target_node_id == target_node)
        });
    }

    pub fn clear_selection(&mut self) {
        for n in &mut self.nodes {
            n.selected = false;
        }
        for e in &mut self.edges {
            e.selected = false;
        }
    }

    pub fn select_node(&mut self, node_id: &str, append: bool) {
        if !append {
            self.clear_selection();
        }
        if let Some(n) = self.find_node_mut(node_id) {
            n.selected = true;
        }
    }

    pub fn toggle_select_node(&mut self, node_id: &str) {
        if let Some(n) = self.find_node_mut(node_id) {
            n.selected = !n.selected;
        }
    }

    pub fn get_bounding_box(&self) -> ((f32, f32), (f32, f32)) {
        if self.nodes.is_empty() {
            return ((0.0, 0.0), (800.0, 600.0));
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node in &self.nodes {
            min_x = min_x.min(node.pos.0);
            min_y = min_y.min(node.pos.1);
            max_x = max_x.max(node.pos.0 + node.size.0);
            max_y = max_y.max(node.pos.1 + node.size.1);
        }

        ((min_x, min_y), (max_x, max_y))
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}
