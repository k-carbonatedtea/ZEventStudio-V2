use super::edge::Edge;
use super::node::{NodeData, NodeKind};
use super::subgraph::{compute_subgraph_groups, SubgraphGroup};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::model::actor::ActorData;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    pub name: String,
    pub description: String,
    pub nodes: Vec<NodeData>,
    pub edges: Vec<Edge>,
    pub actors: Vec<ActorData>,
}

#[allow(dead_code)]
impl Graph {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            actors: Vec::new(),
        }
    }


    /// 根据 EVFL 命名规范生成唯一的节点名称与 ID (如 Event0, Event1, Event59 ... 不与现有节点冲突)
    pub fn generate_unique_node_name(&self, kind: &NodeKind) -> (String, String) {
        let prefix = match kind {
            NodeKind::EntryPoint => "EntryPoint",
            _ => "Event",
        };

        let mut counter = self.nodes.len();
        let mut name = format!("{}{}", prefix, counter);
        while self.nodes.iter().any(|n| n.id == name || n.title == name) {
            counter += 1;
            name = format!("{}{}", prefix, counter);
        }

        let id = name.clone();
        let title = name;
        (id, title)
    }

    pub fn add_node(&mut self, node: NodeData) {
        self.nodes.push(node);
    }

    /// 确保所有 EntryPoint 入口节点尺寸与标题文本宽度自适应匹配
    pub fn sync_entry_point_sizes(&mut self) {
        for node in &mut self.nodes {
            if node.kind == NodeKind::EntryPoint {
                node.size = (super::node::calculate_entry_point_width(&node.title), 42.0);
            }
        }
    }

    /// 确保 actors 列表已初始化（若为空则依据 nodes 自动同步生成），并同步入口节点尺寸
    pub fn ensure_actors_populated(&mut self) {
        self.sync_entry_point_sizes();
        if self.actors.is_empty() {
            let summary = self.get_actors_summary();
            for s in summary {
                let mut a = ActorData::new(s.name);
                a.actions = s.actions;
                a.queries = s.queries;
                a.load_default_params();
                self.actors.push(a);
            }
        }
    }


    /// 提取并汇总图谱中所有引用的 Actor 角色及其动作、查询与关联节点
    pub fn get_actors_summary(&self) -> Vec<super::actor::ActorInfo> {
        type ActorSummaryEntry = (HashSet<String>, HashSet<String>, Vec<String>);
        let mut map: HashMap<String, ActorSummaryEntry> = HashMap::new();


        for node in &self.nodes {
            if let Some(sub) = &node.subtitle {
                if let Some((actor, action_or_query)) = sub.split_once(" :: ") {
                    let entry = map
                        .entry(actor.trim().to_string())
                        .or_insert_with(|| (HashSet::new(), HashSet::new(), Vec::new()));
                    entry.2.push(node.id.clone());
                    if node.kind == NodeKind::Query {
                        entry.1.insert(action_or_query.trim().to_string());
                    } else {
                        entry.0.insert(action_or_query.trim().to_string());
                    }
                }
            }
        }

        let mut list: Vec<super::actor::ActorInfo> = map
            .into_iter()
            .map(|(name, (actions, queries, event_ids))| {
                let mut act_vec: Vec<String> = actions.into_iter().collect();
                act_vec.sort();
                let mut q_vec: Vec<String> = queries.into_iter().collect();
                q_vec.sort();
                super::actor::ActorInfo {
                    name,
                    sub_name: None,
                    actions: act_vec,
                    queries: q_vec,
                    event_ids,
                }
            })
            .collect();

        list.sort_by(|a, b| b.event_ids.len().cmp(&a.event_ids.len()).then_with(|| a.name.cmp(&b.name)));
        list
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

    /// 计算指定节点的所有上游祖先与下游后继节点 (Flow Tracing)
    pub fn get_upstream_and_downstream(&self, target_id: &str) -> (HashSet<String>, HashSet<String>) {
        let mut upstream = HashSet::new();
        let mut downstream = HashSet::new();

        // 1. 向上反向追踪 (Upstream Ancestors)
        let mut queue = VecDeque::new();
        queue.push_back(target_id.to_string());
        while let Some(curr) = queue.pop_front() {
            for edge in &self.edges {
                if edge.target_node_id == curr && !upstream.contains(&edge.source_node_id) {
                    upstream.insert(edge.source_node_id.clone());
                    queue.push_back(edge.source_node_id.clone());
                }
            }
        }

        // 2. 向下正向追踪 (Downstream Descendants)
        queue.push_back(target_id.to_string());
        while let Some(curr) = queue.pop_front() {
            for edge in &self.edges {
                if edge.source_node_id == curr && !downstream.contains(&edge.target_node_id) {
                    downstream.insert(edge.target_node_id.clone());
                    queue.push_back(edge.target_node_id.clone());
                }
            }
        }

        (upstream, downstream)
    }

    /// 获取处于因果链路上的所有高亮连线集合
    pub fn get_active_path_edges(&self, target_id: &str) -> HashSet<(String, String)> {
        let (upstream, downstream) = self.get_upstream_and_downstream(target_id);
        let mut active_edges = HashSet::new();

        for edge in &self.edges {
            let s = &edge.source_node_id;
            let t = &edge.target_node_id;

            // 属于上游链路的连线
            if (upstream.contains(s) || s == target_id) && (upstream.contains(t) || t == target_id) {
                active_edges.insert((s.clone(), t.clone()));
            }
            // 属于下游链路的连线
            if (downstream.contains(s) || s == target_id) && (downstream.contains(t) || t == target_id) {
                active_edges.insert((s.clone(), t.clone()));
            }
        }

        active_edges
    }

    /// 提取图谱中的连通分区并计算每个分区的语义包围盒
    pub fn get_subgraph_groups(&self) -> Vec<SubgraphGroup> {
        compute_subgraph_groups(self)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

