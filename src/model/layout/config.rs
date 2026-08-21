/// 布局配置参数 (与原版 Dagre 排版规范保持一致)
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// 画布起始 X 坐标
    pub start_x: f32,
    /// 画布起始 Y 坐标
    pub start_y: f32,
    /// 同层相邻节点的水平间距 (nodesep: 120.0)
    pub node_spacing_x: f32,
    /// 相邻层级之间的垂直间距 (ranksep: 180.0)
    pub layer_spacing_y: f32,
    /// 独立连通子图 (如多个 EntryPoint 流) 之间的水平间距 (200.0)
    pub component_spacing_x: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            start_x: 80.0,
            start_y: 80.0,
            node_spacing_x: 120.0,
            layer_spacing_y: 180.0,
            component_spacing_x: 200.0,
        }
    }
}

/// 内部节点类型（真实节点或虚拟长连线过渡节点）
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: String,
    pub is_dummy: bool,
    pub width: f32,
    pub height: f32,
    pub rank: usize,
}
