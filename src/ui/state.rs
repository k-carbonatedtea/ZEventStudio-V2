use crate::model::{auto_layout_graph, Edge, Graph, NodeData, NodeKind, PortDirection};
use crate::presets::load_preset_by_id;
use crate::ui::components::Toast;
use crate::ui::sidebar::SidebarTab;


#[derive(Debug, Clone, PartialEq)]
pub enum DragState {
    None,
    PanCanvas {
        start_mouse: (f32, f32),
        start_pan: (f32, f32),
    },
    DragNodes {
        start_mouse: (f32, f32),
        initial_positions: Vec<(String, (f32, f32))>,
    },
    Connecting {
        source_node_id: String,
        source_port_id: String,
        direction: PortDirection,
        source_pos: (f32, f32),
        current_mouse: (f32, f32),
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppWorkspaceTab {
    #[default]
    Welcome,
    Flowchart,
    Actors,
    Events,
    Json,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WelcomeTab {
    #[default]
    QuickStart,
    Recent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentProjectItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub project_type: String,
    pub preset_id: Option<String>,
}

#[allow(dead_code)]
pub struct AppState {
    pub graph: Graph,
    pub pan: (f32, f32),
    pub zoom: f32,
    pub selected_node_id: Option<String>,
    pub hovered_node_id: Option<String>,
    pub hovered_port: Option<(String, String)>,
    pub drag_state: DragState,
    pub knife_mode: bool,
    pub canvas_origin: (f32, f32),
    pub fps: f32,
    pub active_preset_id: String,
    pub active_sidebar_tab: SidebarTab,
    pub current_tab: AppWorkspaceTab,
    pub welcome_tab: WelcomeTab,
    pub recent_projects: Vec<RecentProjectItem>,
    pub selected_actor_idx: usize,
    pub is_adding_action: bool,
    pub new_action_name: String,
    pub show_action_dropdown: bool,
    pub is_adding_query: bool,
    pub new_query_name: String,
    pub show_query_dropdown: bool,
    pub is_adding_actor: bool,
    pub new_actor_name: String,
    pub new_actor_sub: String,
    pub show_actor_dropdown: bool,
    pub is_adding_param: bool,
    pub new_param_type: String,
    pub new_param_key: String,
    pub new_param_val: String,
    pub show_param_dropdown: bool,
    pub toast: Option<Toast>,



    pub quick_add_modal: Option<(f32, f32, Option<(String, String, PortDirection)>)>,
    pub node_context_menu: Option<(String, (f32, f32))>,

    pub inspector_open: bool,
    pub sidebar_open: bool,
    pub search_query: String,
    pub expand_all_params: bool,
    pub performance_mode: bool,
    pub frame_time_ms: f32,
    pub show_welcome_on_startup: bool,
    pub lang: crate::i18n::Language,
}


#[allow(dead_code)]
impl AppState {
    pub fn new() -> Self {
        let mut graph = Graph::new("EventFlow");
        graph.ensure_actors_populated();

        let mut state = Self {
            graph,
            pan: (100.0, 100.0),
            zoom: 1.0,
            selected_node_id: None,
            hovered_node_id: None,
            hovered_port: None,
            drag_state: DragState::None,
            knife_mode: false,
            canvas_origin: (0.0, 0.0),
            fps: 60.0,
            active_preset_id: String::new(),
            active_sidebar_tab: SidebarTab::EntryPoints,
            current_tab: AppWorkspaceTab::Welcome,
            welcome_tab: WelcomeTab::QuickStart,


            recent_projects: vec![
                RecentProjectItem {
                    id: "rec_1".to_string(),
                    name: "100enemy.bfevfl".to_string(),
                    path: "romfs/Event/EventFlow/100enemy.bfevfl".to_string(),
                    project_type: "bfevfl".to_string(),
                    preset_id: Some("zelda_100enemy".to_string()),
                },
                RecentProjectItem {
                    id: "rec_2".to_string(),
                    name: "Quest_Dialogue.bfevfl".to_string(),
                    path: "romfs/Event/EventFlow/Quest_Dialogue.bfevfl".to_string(),
                    project_type: "bfevfl".to_string(),
                    preset_id: Some("quest_dialogue".to_string()),
                },
                RecentProjectItem {
                    id: "rec_3".to_string(),
                    name: "Combat_AI.bfevfl".to_string(),
                    path: "romfs/Event/EventFlow/Combat_AI.bfevfl".to_string(),
                    project_type: "bfevfl".to_string(),
                    preset_id: Some("combat_ai".to_string()),
                },
                RecentProjectItem {
                    id: "rec_4".to_string(),
                    name: "Signal_Math.bfevfl".to_string(),
                    path: "romfs/Event/EventFlow/Signal_Math.bfevfl".to_string(),
                    project_type: "bfevfl".to_string(),
                    preset_id: Some("signal_math".to_string()),
                },
            ],
            selected_actor_idx: 0,
            is_adding_actor: false,
            new_actor_name: String::new(),
            new_actor_sub: String::new(),
            show_actor_dropdown: false,
            is_adding_action: false,
            new_action_name: String::new(),
            show_action_dropdown: false,
            is_adding_query: false,
            new_query_name: String::new(),
            show_query_dropdown: false,
            is_adding_param: false,
            new_param_type: "Int".to_string(),
            new_param_key: String::new(),
            new_param_val: "0".to_string(),
            show_param_dropdown: false,
            toast: Some(Toast::success(crate::i18n::t("toast.preset_loaded", crate::i18n::Language::Zh))),
            quick_add_modal: None,
            node_context_menu: None,
            inspector_open: true,
            sidebar_open: true,
            search_query: String::new(),
            expand_all_params: false,
            performance_mode: false,
            frame_time_ms: 1.0,
            show_welcome_on_startup: true,
            lang: crate::i18n::Language::Zh,
        };

        state.fit_view();
        state
    }

    pub fn new_blank_flow(&mut self) {
        self.graph = Graph::new("NewEventFlow");
        self.graph.ensure_actors_populated();
        self.active_preset_id = "blank".to_string();
        self.current_tab = AppWorkspaceTab::Flowchart;
        self.toast = Some(Toast::success("✨ 已创建全新空白 EventFlow 流程图"));
    }



    /// 获取当前语言环境下的本地化文本
    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        crate::i18n::t(key, self.lang)
    }

    pub fn load_preset(&mut self, preset_id: &str) {
        if let Some(mut g) = load_preset_by_id(preset_id) {
            g.ensure_actors_populated();
            self.graph = g;
            self.active_preset_id = preset_id.to_string();
            self.selected_node_id = None;
            self.selected_actor_idx = 0;
            self.update_node_sizes();
            auto_layout_graph(&mut self.graph);
            self.fit_view();
            self.toast = Some(Toast::success(format!("{}{}", self.t("toast.switch_preset"), self.graph.name)));
        }
    }


    pub fn auto_layout(&mut self) {
        self.update_node_sizes();
        auto_layout_graph(&mut self.graph);
        self.fit_view();
        self.toast = Some(Toast::success(self.t("toast.auto_layout_done")));
    }

    pub fn toggle_performance_mode(&mut self) {
        self.performance_mode = !self.performance_mode;
        if self.performance_mode {
            self.toast = Some(Toast::success(self.t("toast.perf_on")));
        } else {
            self.toast = Some(Toast::info(self.t("toast.perf_off")));
        }
    }

    pub fn toggle_expand_params(&mut self) {
        self.expand_all_params = !self.expand_all_params;
        self.update_node_sizes();
        auto_layout_graph(&mut self.graph);
        self.fit_view();
        self.toast = Some(Toast::info(if self.expand_all_params { self.t("toast.expand_params_on") } else { self.t("toast.expand_params_off") }));
    }

    pub fn update_node_sizes(&mut self) {
        for node in &mut self.graph.nodes {
            if node.kind == NodeKind::EntryPoint {
                node.size = (180.0, 42.0);
                continue;
            }
            if self.expand_all_params && !node.properties.is_empty() {
                let height = 110.0 + 22.0 * (node.properties.len() as f32);
                let mut max_len = node.title.len().max(node.subtitle.as_ref().map(|s| s.len()).unwrap_or(0));
                for p in &node.properties { max_len = max_len.max(p.key.len() + p.val.len() + 4); }
                node.size = (240.0f32.max(((max_len as f32) * 7.2 + 40.0).min(480.0)), height);
            } else {
                let hh = (node.extract_key_highlights().len() as f32) * 24.0;
                let (bw, bh) = match node.kind {
                    NodeKind::Action | NodeKind::Switch => (240.0, 85.0 + hh),
                    NodeKind::Query => (230.0, 85.0 + hh),
                    NodeKind::SubFlow => (220.0, 85.0 + hh),
                    NodeKind::Fork | NodeKind::Join => (210.0, 95.0),
                    NodeKind::Math => (200.0, 100.0),
                    NodeKind::Comment => (240.0, 120.0),
                    NodeKind::EntryPoint => (180.0, 42.0),
                };
                node.size = (bw, bh);
            }
        }
    }

    pub fn fit_view(&mut self) {
        if self.graph.nodes.is_empty() {
            self.pan = (100.0, 100.0);
            self.zoom = 1.0;
            return;
        }
        let ((min_x, min_y), (max_x, max_y)) = self.graph.get_bounding_box();
        let gw = (max_x - min_x).max(100.0);
        let gh = (max_y - min_y).max(100.0);
        let tz = ((1100.0 - 120.0) / gw).min((650.0 - 120.0) / gh).clamp(0.45, 1.25);
        self.zoom = tz;
        self.pan = ((1100.0 - gw * tz) / 2.0 - min_x * tz, (650.0 - gh * tz) / 2.0 - min_y * tz);
    }

    pub fn focus_node(&mut self, node_id: &str) {
        if let Some(node) = self.graph.find_node(node_id) {
            let (pos, size) = (node.pos, node.size);
            self.pan = (1100.0 / 2.0 - (pos.0 + size.0 / 2.0) * self.zoom, 650.0 / 2.0 - (pos.1 + size.1 / 2.0) * self.zoom);
            self.selected_node_id = Some(node_id.to_string());
            self.toast = Some(Toast::info(format!("{}{}", self.t("toast.focus_node"), node.title)));
        }
    }

    pub fn add_node_at(&mut self, kind: NodeKind, _fallback_title: impl Into<String>, pos: (f32, f32)) -> String {
        let (node_id, title) = self.graph.generate_unique_node_name(&kind);
        let mut node = NodeData::new(node_id.clone(), title, kind.clone(), pos);
        match kind {
            NodeKind::Action => {
                let def_actor = self.graph.nodes.iter().find_map(|n| n.subtitle.as_ref().and_then(|s| s.split(" :: ").next().map(|a| a.to_string()))).unwrap_or_else(|| "EventSystemActor".to_string());
                node.subtitle = Some(format!("{} :: Demo_FlagOFF", def_actor));
            }
            NodeKind::Query => {
                let def_actor = self.graph.nodes.iter().find_map(|n| n.subtitle.as_ref().and_then(|s| s.split(" :: ").next().map(|a| a.to_string()))).unwrap_or_else(|| "EventSystemActor".to_string());
                node.subtitle = Some(format!("{} :: GeneralQuery", def_actor));
            }
            NodeKind::Switch => { node.subtitle = Some("EventFlags".to_string()); }
            NodeKind::SubFlow => { node.subtitle = Some("CommonSubFlow".to_string()); }
            _ => {}
        }
        self.graph.add_node(node);
        self.selected_node_id = Some(node_id.clone());
        node_id
    }

    pub fn delete_selected_node(&mut self) {
        if let Some(id) = self.selected_node_id.take() {
            self.graph.remove_node(&id);
            self.toast = Some(Toast::info(self.t("toast.deleted_node")));
        }
    }

    pub fn get_canvas_origin(&self) -> (f32, f32) {
        (if self.sidebar_open { 280.0 } else { 0.0 }, 44.0)
    }

    pub fn screen_to_canvas(&self, sx: f32, sy: f32) -> (f32, f32) {
        let (ox, oy) = self.get_canvas_origin();
        ((sx - ox - self.pan.0) / self.zoom, (sy - oy - self.pan.1) / self.zoom)
    }

    pub fn canvas_to_screen(&self, cx: f32, cy: f32) -> (f32, f32) {
        let (ox, oy) = self.get_canvas_origin();
        (cx * self.zoom + self.pan.0 + ox, cy * self.zoom + self.pan.1 + oy)
    }

    pub fn sever_edge(&mut self, src: &str, tgt: &str) {
        self.graph.sever_edge(src, tgt);
        self.toast = Some(Toast::info(self.t("toast.sever_edge")));
    }

    pub fn connect_ports(&mut self, s_node: &str, s_port: &str, t_node: &str, t_port: &str) {
        if s_node != t_node {
            self.graph.add_edge(Edge::new(s_node, s_port, t_node, t_port));
            self.toast = Some(Toast::success(self.t("toast.connect_edge")));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_expand_params() {
        let mut state = AppState::new();
        assert!(!state.expand_all_params);
        state.toggle_expand_params();
        assert!(state.expand_all_params);
        state.toggle_expand_params();
        assert!(!state.expand_all_params);
    }
}


