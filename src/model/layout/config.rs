/// 布局配置参数 (紧凑向下流动排版规范)
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// 画布起始 X 坐标
    pub start_x: f32,
    /// 画布起始 Y 坐标
    pub start_y: f32,
    /// 同层相邻节点的水平间距 (紧凑 36.0px)
    pub node_spacing_x: f32,
    /// 相邻层级之间的垂直间距 (紧凑 48.0px)
    pub layer_spacing_y: f32,
    /// 独立连通子图之间的水平间距 (60.0px)
    pub component_spacing_x: f32,
    /// 独立连通子图之间的垂直间距 (80.0px)
    pub component_spacing_y: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            start_x: 60.0,
            start_y: 60.0,
            node_spacing_x: 36.0,
            layer_spacing_y: 48.0,
            component_spacing_x: 60.0,
            component_spacing_y: 80.0,
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
