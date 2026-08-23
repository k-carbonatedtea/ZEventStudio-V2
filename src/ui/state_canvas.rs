use crate::model::{auto_layout_graph, Edge, NodeData, NodeKind};
use crate::ui::components::Toast;
use crate::ui::state::AppState;

#[allow(dead_code)]
impl AppState {
    /// 动态刷新图表中所有节点的物理尺寸与边界
    pub fn update_node_sizes(&mut self) {
        for node in &mut self.graph.nodes {
            if node.kind == NodeKind::EntryPoint {
                node.size = (180.0, 42.0);
                continue;
            }
            if self.expand_all_params && !node.properties.is_empty() {
                let height = 110.0 + 22.0 * (node.properties.len() as f32);
                let mut max_len = node.title.len().max(node.subtitle.as_ref().map(|s| s.len()).unwrap_or(0));
                for p in &node.properties {
                    max_len = max_len.max(p.key.len() + p.val.len() + 4);
                }
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

    /// 自适应视口居中
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

    /// 聚焦定位至指定节点
    pub fn focus_node(&mut self, node_id: &str) {
        if let Some(node) = self.graph.find_node(node_id) {
            let (pos, size) = (node.pos, node.size);
            self.pan = (1100.0 / 2.0 - (pos.0 + size.0 / 2.0) * self.zoom, 650.0 / 2.0 - (pos.1 + size.1 / 2.0) * self.zoom);
            self.selected_node_id = Some(node_id.to_string());
            self.toast = Some(Toast::info(format!("{}{}", self.t("toast.focus_node"), node.title)));
        }
    }

    /// 自动执行分层 DAG 拓扑排版算法
    pub fn auto_layout(&mut self) {
        self.update_node_sizes();
        auto_layout_graph(&mut self.graph);
        self.fit_view();
        self.mark_current_tab_modified();
        self.toast = Some(Toast::success(self.t("toast.auto_layout_done")));
    }

    /// 在指定画布坐标添加新节点
    pub fn add_node_at(&mut self, kind: NodeKind, _fallback_title: impl Into<String>, pos: (f32, f32)) -> String {
        let (node_id, title) = self.graph.generate_unique_node_name(&kind);
        let node = NodeData::new(node_id.clone(), title, kind, pos);
        self.graph.add_node(node);
        self.selected_node_id = Some(node_id.clone());
        self.mark_current_tab_modified();
        node_id
    }

    /// 删除当前选中的节点
    pub fn delete_selected_node(&mut self) {
        if let Some(id) = self.selected_node_id.take() {
            self.graph.remove_node(&id);
            self.mark_current_tab_modified();
            self.toast = Some(Toast::info(self.t("toast.deleted_node")));
        }
    }

    /// 获取画布绘制相对视口的原点
    pub fn get_canvas_origin(&self) -> (f32, f32) {
        (if self.sidebar_open { 280.0 } else { 0.0 }, 44.0)
    }

    /// 屏幕物理像素坐标转换为画布内部逻辑坐标
    pub fn screen_to_canvas(&self, sx: f32, sy: f32) -> (f32, f32) {
        let (ox, oy) = self.get_canvas_origin();
        ((sx - ox - self.pan.0) / self.zoom, (sy - oy - self.pan.1) / self.zoom)
    }

    /// 画布内部逻辑坐标转换为屏幕物理像素坐标
    pub fn canvas_to_screen(&self, cx: f32, cy: f32) -> (f32, f32) {
        let (ox, oy) = self.get_canvas_origin();
        (cx * self.zoom + self.pan.0 + ox, cy * self.zoom + self.pan.1 + oy)
    }

    /// 剪断两个节点之间的连线
    pub fn sever_edge(&mut self, src: &str, tgt: &str) {
        self.graph.sever_edge(src, tgt);
        self.mark_current_tab_modified();
        self.toast = Some(Toast::info(self.t("toast.sever_edge")));
    }

    /// 连接两个端口
    pub fn connect_ports(&mut self, s_node: &str, s_port: &str, t_node: &str, t_port: &str) {
        if s_node != t_node {
            self.graph.add_edge(Edge::new(s_node, s_port, t_node, t_port));
            self.mark_current_tab_modified();
            self.toast = Some(Toast::success(self.t("toast.connect_edge")));
        }
    }
}
