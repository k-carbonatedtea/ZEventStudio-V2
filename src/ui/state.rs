use crate::model::{auto_layout_graph, Edge, Graph, NodeData, NodeKind};
use crate::presets::{get_startup_preset, load_preset_by_id};
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
        source_pos: (f32, f32),
        current_mouse: (f32, f32),
    },
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
    pub toast: Option<Toast>,
    pub quick_add_modal: Option<(f32, f32, Option<(String, String)>)>,
    pub inspector_open: bool,
    pub sidebar_open: bool,
    pub search_query: String,
    pub expand_all_params: bool,
    pub performance_mode: bool,
    pub frame_time_ms: f32,
    pub lang: crate::i18n::Language,
}

#[allow(dead_code)]
impl AppState {
    pub fn new() -> Self {
        // 项目打开时，加载 100enemy.bfevfl 真实 EVFL 事件流
        let mut graph = get_startup_preset();
        auto_layout_graph(&mut graph);

        let mut state = Self {
            graph,
            pan: (80.0, 60.0),
            zoom: 0.85,
            selected_node_id: None,
            hovered_node_id: None,
            hovered_port: None,
            drag_state: DragState::None,
            knife_mode: false,
            canvas_origin: (0.0, 0.0),
            fps: 60.0,
            active_preset_id: "zelda_100enemy".to_string(),
            active_sidebar_tab: SidebarTab::EntryPoints,
            toast: Some(Toast::success(crate::i18n::t("toast.preset_loaded", crate::i18n::Language::Zh))),
            quick_add_modal: None,
            inspector_open: true,
            sidebar_open: true,
            search_query: String::new(),
            expand_all_params: false,
            performance_mode: false,
            frame_time_ms: 1.0,
            lang: crate::i18n::Language::Zh,
        };
        state.fit_view();
        state
    }

    /// 获取当前语言环境下的本地化文本
    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        crate::i18n::t(key, self.lang)
    }

    pub fn load_preset(&mut self, preset_id: &str) {
        if let Some(g) = load_preset_by_id(preset_id) {
            self.graph = g;
            self.active_preset_id = preset_id.to_string();
            self.selected_node_id = None;
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
        if self.expand_all_params {
            self.toast = Some(Toast::info(self.t("toast.expand_params_on")));
        } else {
            self.toast = Some(Toast::info(self.t("toast.expand_params_off")));
        }
    }

    pub fn update_node_sizes(&mut self) {
        for node in &mut self.graph.nodes {
            if node.kind == NodeKind::EntryPoint {
                node.size = (180.0, 42.0);
                continue;
            }

            if self.expand_all_params && !node.properties.is_empty() {
                let param_count = node.properties.len();
                let height = 110.0 + 22.0 * (param_count as f32);
                let mut max_text_len = node.title.len().max(node.subtitle.as_ref().map(|s| s.len()).unwrap_or(0));
                for p in &node.properties {
                    max_text_len = max_text_len.max(p.key.len() + p.val.len() + 4);
                }
                let width = 240.0f32.max(((max_text_len as f32) * 7.2 + 40.0).min(480.0));
                node.size = (width, height);
            } else {
                let highlights_count = node.extract_key_highlights().len();
                let highlight_h = (highlights_count as f32) * 24.0;

                let (base_w, base_h) = match node.kind {
                    NodeKind::Action => (240.0, 85.0 + highlight_h),
                    NodeKind::Switch => (240.0, 85.0 + highlight_h),
                    NodeKind::Query => (230.0, 85.0 + highlight_h),
                    NodeKind::SubFlow => (220.0, 85.0 + highlight_h),
                    NodeKind::Fork => (210.0, 95.0),
                    NodeKind::Join => (210.0, 95.0),
                    NodeKind::Math => (200.0, 100.0),
                    NodeKind::Comment => (240.0, 120.0),
                    NodeKind::EntryPoint => (180.0, 42.0),
                };
                node.size = (base_w, base_h);
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
        let graph_w = (max_x - min_x).max(100.0);
        let graph_h = (max_y - min_y).max(100.0);

        // Target viewport approx 1100x650
        let vp_w: f32 = 1100.0;
        let vp_h: f32 = 650.0;

        let scale_x: f32 = (vp_w - 120.0) / graph_w;
        let scale_y: f32 = (vp_h - 120.0) / graph_h;
        let target_zoom = scale_x.min(scale_y).clamp(0.45, 1.25);

        let target_pan_x = (vp_w - graph_w * target_zoom) / 2.0 - min_x * target_zoom;
        let target_pan_y = (vp_h - graph_h * target_zoom) / 2.0 - min_y * target_zoom;

        self.zoom = target_zoom;
        self.pan = (target_pan_x, target_pan_y);
    }

    pub fn focus_node(&mut self, node_id: &str) {
        if let Some(node) = self.graph.find_node(node_id) {
            let pos = node.pos;
            let size = node.size;
            let vp_w: f32 = 1100.0;
            let vp_h: f32 = 650.0;
            self.pan = (
                vp_w / 2.0 - (pos.0 + size.0 / 2.0) * self.zoom,
                vp_h / 2.0 - (pos.1 + size.1 / 2.0) * self.zoom,
            );
            self.selected_node_id = Some(node_id.to_string());
            self.toast = Some(Toast::info(format!("{}{}", self.t("toast.focus_node"), node.title)));
        }
    }

    pub fn add_node_at(&mut self, kind: NodeKind, title: impl Into<String>, pos: (f32, f32)) -> String {
        let node_id = format!("node_{}", uuid::Uuid::new_v4().simple());
        let node = NodeData::new(node_id.clone(), title, kind, pos);
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

    pub fn screen_to_canvas(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        let local_x = screen_x - self.canvas_origin.0 - self.pan.0;
        let local_y = screen_y - self.canvas_origin.1 - self.pan.1;
        (local_x / self.zoom, local_y / self.zoom)
    }

    pub fn canvas_to_screen(&self, canvas_x: f32, canvas_y: f32) -> (f32, f32) {
        let local_x = canvas_x * self.zoom + self.pan.0 + self.canvas_origin.0;
        let local_y = canvas_y * self.zoom + self.pan.1 + self.canvas_origin.1;
        (local_x, local_y)
    }

    pub fn sever_edge(&mut self, source_node: &str, target_node: &str) {
        self.graph.sever_edge(source_node, target_node);
        self.toast = Some(Toast::info(self.t("toast.sever_edge")));
    }

    pub fn connect_ports(
        &mut self,
        source_node_id: &str,
        source_port_id: &str,
        target_node_id: &str,
        target_port_id: &str,
    ) {
        if source_node_id == target_node_id {
            return; // No self-loops
        }

        let edge = Edge::new(source_node_id, source_port_id, target_node_id, target_port_id);
        self.graph.add_edge(edge);
        self.toast = Some(Toast::success(self.t("toast.connect_edge")));
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
        assert!(!state.graph.nodes.is_empty());

        state.toggle_expand_params();
        assert!(!state.expand_all_params);
    }
}
