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
            toast: Some(Toast::success("⚡ 已成功载入 100enemy.json 真实 EVFL 事件流")),
            quick_add_modal: None,
            inspector_open: true,
            sidebar_open: true,
            search_query: String::new(),
        };
        state.fit_view();
        state
    }

    pub fn load_preset(&mut self, preset_id: &str) {
        if let Some(g) = load_preset_by_id(preset_id) {
            self.graph = g;
            self.active_preset_id = preset_id.to_string();
            self.selected_node_id = None;
            self.fit_view();
            self.toast = Some(Toast::success(format!("已切换至案例：{}", self.graph.name)));
        }
    }

    pub fn auto_layout(&mut self) {
        auto_layout_graph(&mut self.graph);
        self.fit_view();
        self.toast = Some(Toast::success("✨ 已完成 DAG 拓扑分层自动排版"));
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
            self.toast = Some(Toast::info(format!("已跳转聚焦至: {}", node.title)));
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
            self.toast = Some(Toast::info("🗑️ 已删除选中节点"));
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
        self.toast = Some(Toast::info("✂️ 已剪断连线"));
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
        self.toast = Some(Toast::success("🔗 已建立连线"));
    }
}
