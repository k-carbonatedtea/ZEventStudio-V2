//! 3D 模型预览 (BFRES) 模块简体中文词典

pub fn get_zh_model_preview(key: &str) -> Option<&'static str> {
    match key {
        "model_preview.mode_shaded" => Some("实体着色"),
        "model_preview.mode_wireframe" => Some("矢量线框"),
        "model_preview.mode_shaded_wire" => Some("线框叠加"),
        "model_preview.mode_points" => Some("顶点点云"),

        "model_preview.toggle_grid" => Some("空间网格"),
        "model_preview.toggle_axes" => Some("世界坐标轴"),
        "model_preview.toggle_bounds" => Some("三维包围盒"),
        "model_preview.toggle_stats" => Some("技术指标"),

        "model_preview.view_front" => Some("前视"),
        "model_preview.view_side" => Some("右视"),
        "model_preview.view_top" => Some("顶视"),
        "model_preview.view_iso" => Some("等轴"),
        "model_preview.fit_view" => Some("对焦居中"),

        "model_preview.stat_meshes" => Some("可见网格:"),
        "model_preview.stat_vertices" => Some("顶点总数:"),
        "model_preview.stat_triangles" => Some("三角面数:"),
        "model_preview.stat_dimensions" => Some("外包跨度:"),
        "model_preview.stat_distance" => Some("相机视距:"),
        "model_preview.stat_textures" => Some("纹理贴图:"),

        "model_preview.hint_orbit" => Some("• 鼠标左键拖拽: 环绕旋转观察"),
        "model_preview.hint_pan" => Some("• 鼠标右键拖拽: 平移视口中心"),
        "model_preview.hint_zoom" => Some("• 滚轮滑动: 自由推拉缩放"),

        "model_preview.meshes_title" => Some("子网格图层"),
        "model_preview.filter_placeholder" => Some("按名称或材质筛选..."),
        "model_preview.no_model_selected" => Some("未选中或未加载 3D 模型"),
        "model_preview.window_title" => Some("3D 模型预览"),
        "model_preview.toggle_sidebar" => Some("子网格目录"),
        "model_preview.select_all" => Some("全选"),
        "model_preview.deselect_all" => Some("全不选"),
        _ => None,
    }
}

