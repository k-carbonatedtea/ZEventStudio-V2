use super::Language;

/// 根据键名与目标语言返回对应的本地化文本
pub fn translate<'a>(key: &'a str, lang: Language) -> &'a str {
    match (key, lang) {
        // ---------------- 顶部工具栏 (Toolbar) ----------------
        ("toolbar.auto_layout", Language::Zh) => "自动排版",
        ("toolbar.auto_layout", Language::En) => "Auto Layout",

        ("toolbar.expand_params", Language::Zh) => "展开参数",
        ("toolbar.expand_params", Language::En) => "Expand Params",

        ("toolbar.collapse_params", Language::Zh) => "折叠参数",
        ("toolbar.collapse_params", Language::En) => "Collapse Params",

        ("toolbar.knife_mode", Language::Zh) => "切刀工具",
        ("toolbar.knife_mode", Language::En) => "Knife Tool",

        ("toolbar.knife_active", Language::Zh) => "✂ 剪断模式中",
        ("toolbar.knife_active", Language::En) => "✂ Cutting Mode",

        ("toolbar.perf_mode", Language::Zh) => "性能模式",
        ("toolbar.perf_mode", Language::En) => "Performance Mode",

        ("toolbar.perf_active", Language::Zh) => "🔥 极速性能",
        ("toolbar.perf_active", Language::En) => "🔥 Extreme Perf",

        ("toolbar.fit_view", Language::Zh) => "自适应视口",
        ("toolbar.fit_view", Language::En) => "Fit View",

        ("toolbar.zoom_in", Language::Zh) => "放大",
        ("toolbar.zoom_in", Language::En) => "Zoom In",

        ("toolbar.zoom_out", Language::Zh) => "缩小",
        ("toolbar.zoom_out", Language::En) => "Zoom Out",

        ("toolbar.reset_zoom", Language::Zh) => "重置缩放",
        ("toolbar.reset_zoom", Language::En) => "Reset Zoom",

        ("toolbar.sidebar", Language::Zh) => "预设侧边栏",
        ("toolbar.sidebar", Language::En) => "Presets Panel",

        ("toolbar.inspector", Language::Zh) => "属性面板",
        ("toolbar.inspector", Language::En) => "Inspector",

        ("toolbar.lang_btn", Language::Zh) => "中 / EN",
        ("toolbar.lang_btn", Language::En) => "EN / 中",

        // ---------------- 节点卡片与图形 (Nodes & Canvas) ----------------
        ("node.parameters_label", Language::Zh) => "参数属性 (Parameters):",
        ("node.parameters_label", Language::En) => "Parameters:",

        ("node.wait_join", Language::Zh) => "等待并发分支汇合",
        ("node.wait_join", Language::En) => "Wait for parallel join",

        ("node.fork_count", Language::Zh) => "分支数",
        ("node.fork_count", Language::En) => "Forks",

        ("edge.cut", Language::Zh) => "✂ 剪断",
        ("edge.cut", Language::En) => "✂ Cut",

        // ---------------- 属性检查器 (Inspector) ----------------
        ("inspector.title", Language::Zh) => "属性检查器",
        ("inspector.title", Language::En) => "Inspector",

        ("inspector.basic_info", Language::Zh) => "基础信息",
        ("inspector.basic_info", Language::En) => "Basic Information",

        ("inspector.node_id", Language::Zh) => "节点标识 (ID)",
        ("inspector.node_id", Language::En) => "Node ID",

        ("inspector.node_title", Language::Zh) => "节点标题 (Title)",
        ("inspector.node_title", Language::En) => "Node Title",

        ("inspector.node_kind", Language::Zh) => "节点种类",
        ("inspector.node_kind", Language::En) => "Node Kind",

        ("inspector.position", Language::Zh) => "画布坐标 (Position)",
        ("inspector.position", Language::En) => "Canvas Position",

        ("inspector.dimensions", Language::Zh) => "尺寸大小 (Size)",
        ("inspector.dimensions", Language::En) => "Dimensions",

        ("inspector.parameters", Language::Zh) => "参数属性列表 (Properties)",
        ("inspector.parameters", Language::En) => "Parameters & Properties",

        ("inspector.add_param", Language::Zh) => "+ 添加参数属性",
        ("inspector.add_param", Language::En) => "+ Add Parameter",

        ("inspector.empty_hint", Language::Zh) => "请在画布中选中节点以查看和编辑属性",
        ("inspector.empty_hint", Language::En) => "Select a node on the canvas to view and edit properties",

        // ---------------- 侧边栏与预设 (Sidebar & Presets) ----------------
        ("sidebar.title", Language::Zh) => "官方 EVFL 预设流",
        ("sidebar.title", Language::En) => "Official EVFL Presets",

        ("sidebar.nodes_count", Language::Zh) => "节点总数",
        ("sidebar.nodes_count", Language::En) => "Total Nodes",

        ("sidebar.edges_count", Language::Zh) => "连线总数",
        ("sidebar.edges_count", Language::En) => "Total Edges",

        // ---------------- 快捷添加 (Quick Add) ----------------
        ("quick_add.title", Language::Zh) => "快捷新建节点",
        ("quick_add.title", Language::En) => "Quick Add Node",

        ("quick_add.placeholder", Language::Zh) => "搜索节点类型 (如 Action, Switch, Fork)...",
        ("quick_add.placeholder", Language::En) => "Search node type (e.g. Action, Switch, Fork)...",

        // 默认回退原字符串
        _ => key,
    }
}
