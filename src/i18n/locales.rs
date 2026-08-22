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

        ("toolbar.knife_active", Language::Zh) => "剪断模式中",
        ("toolbar.knife_active", Language::En) => "Cutting Mode",

        ("toolbar.perf_mode", Language::Zh) => "性能模式",
        ("toolbar.perf_mode", Language::En) => "Performance Mode",

        ("toolbar.perf_active", Language::Zh) => "极速性能",
        ("toolbar.perf_active", Language::En) => "Extreme Perf",

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

        ("toolbar.nodes_label", Language::Zh) => "节点",
        ("toolbar.nodes_label", Language::En) => "Nodes",

        ("toolbar.edges_label", Language::Zh) => "连线",
        ("toolbar.edges_label", Language::En) => "Edges",

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

        ("inspector.selected_node", Language::Zh) => "已选中节点",
        ("inspector.selected_node", Language::En) => "Selected Node",

        ("inspector.global_info", Language::Zh) => "全局信息",
        ("inspector.global_info", Language::En) => "Global Graph",

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

        ("inspector.ports_summary", Language::Zh) => "端口引脚",
        ("inspector.ports_summary", Language::En) => "Port Sockets",

        ("inspector.port_in", Language::Zh) => "◀ 输入",
        ("inspector.port_in", Language::En) => "◀ Input",

        ("inspector.port_out", Language::Zh) => "▶ 输出",
        ("inspector.port_out", Language::En) => "▶ Output",

        ("inspector.parameters", Language::Zh) => "EVFL 参数属性列表",
        ("inspector.parameters", Language::En) => "EVFL Parameters",

        ("inspector.no_parameters", Language::Zh) => "暂无自定义参数",
        ("inspector.no_parameters", Language::En) => "No custom parameters",

        ("inspector.delete_node", Language::Zh) => "🗑️ 删除节点",
        ("inspector.delete_node", Language::En) => "🗑️ Delete Node",

        ("inspector.topology_stats", Language::Zh) => "📊 图谱拓扑统计",
        ("inspector.topology_stats", Language::En) => "📊 Graph Topology Stats",

        ("inspector.total_nodes", Language::Zh) => "📦 总节点数",
        ("inspector.total_nodes", Language::En) => "📦 Total Nodes",

        ("inspector.total_edges", Language::Zh) => "🔗 拓扑连线数",
        ("inspector.total_edges", Language::En) => "🔗 Total Edges",

        ("inspector.stat_entry_points", Language::Zh) => "🚩 入口 (EntryPoints)",
        ("inspector.stat_entry_points", Language::En) => "🚩 Entry Points",

        ("inspector.stat_actions", Language::Zh) => "⚡ 动作 (Actions)",
        ("inspector.stat_actions", Language::En) => "⚡ Actions",

        ("inspector.stat_switches", Language::Zh) => "🔀 条件 (Switches)",
        ("inspector.stat_switches", Language::En) => "🔀 Switches",

        ("inspector.tips_title", Language::Zh) => "💡 交互操作提示",
        ("inspector.tips_title", Language::En) => "💡 Interaction Tips",

        ("inspector.tip_entry_jump", Language::Zh) => "• 点击左侧「流程入口」可一键跳转至对应子剧情流",
        ("inspector.tip_entry_jump", Language::En) => "• Click 'Entry Points' on left sidebar to jump to flow",

        ("inspector.tip_pan_zoom", Language::Zh) => "• 鼠标拖拽画布平移，滚轮缩放，按住节点拖动重排",
        ("inspector.tip_pan_zoom", Language::En) => "• Drag canvas to pan, wheel to zoom, drag nodes to move",

        ("inspector.tip_quick_add", Language::Zh) => "• 双击或右键画布空白处可快速唤出新建节点面板",
        ("inspector.tip_quick_add", Language::En) => "• Right click or drag wire on canvas to open Quick Add",

        // ---------------- 侧边栏与预设 (Sidebar & Presets) ----------------
        ("sidebar.tab_entry_points", Language::Zh) => "🏁 流程入口",
        ("sidebar.tab_entry_points", Language::En) => "🏁 Entry Points",

        ("sidebar.tab_presets", Language::Zh) => "🎮 预设案例",
        ("sidebar.tab_presets", Language::En) => "🎮 Presets",

        ("sidebar.tab_palette", Language::Zh) => "📦 节点组件",
        ("sidebar.tab_palette", Language::En) => "📦 Components",

        ("sidebar.search_placeholder", Language::Zh) => "搜索节点类型...",
        ("sidebar.search_placeholder", Language::En) => "Search node type...",

        ("sidebar.entry_points_title", Language::Zh) => "🚩 流程入口列表",
        ("sidebar.entry_points_title", Language::En) => "🚩 Entry Points",

        ("sidebar.jump_focus", Language::Zh) => "跳转聚焦",
        ("sidebar.jump_focus", Language::En) => "Focus",

        ("sidebar.active_badge", Language::Zh) => "当前激活",
        ("sidebar.active_badge", Language::En) => "Active",

        // ---------------- 节点组件库与类型名称 (Palette & Kinds) ----------------
        ("node_kind.action_title", Language::Zh) => "⚡ Action 动作",
        ("node_kind.action_title", Language::En) => "⚡ Action Node",
        ("node_kind.action_desc", Language::Zh) => "执行对话/音效/道具等具体行为",
        ("node_kind.action_desc", Language::En) => "Execute dialogue, audio, or item action",

        ("node_kind.switch_title", Language::Zh) => "🔀 Switch 分支",
        ("node_kind.switch_title", Language::En) => "🔀 Switch Node",
        ("node_kind.switch_desc", Language::Zh) => "多条件动态分支路由",
        ("node_kind.switch_desc", Language::En) => "Multi-condition dynamic branch routing",

        ("node_kind.query_title", Language::Zh) => "🔍 Query 查询",
        ("node_kind.query_title", Language::En) => "🔍 Query Node",
        ("node_kind.query_desc", Language::Zh) => "检查状态与游戏条件",
        ("node_kind.query_desc", Language::En) => "Evaluate game condition and variable state",

        ("node_kind.subflow_title", Language::Zh) => "📦 SubFlow 子流程",
        ("node_kind.subflow_title", Language::En) => "📦 SubFlow Node",
        ("node_kind.subflow_desc", Language::Zh) => "调用已封装的子节点图",
        ("node_kind.subflow_desc", Language::En) => "Invoke modular subflow graph",

        ("node_kind.fork_title", Language::Zh) => "🔱 Fork 并行",
        ("node_kind.fork_title", Language::En) => "🔱 Fork Node",
        ("node_kind.fork_desc", Language::Zh) => "分发多条并行流水线",
        ("node_kind.fork_desc", Language::En) => "Dispatch parallel asynchronous execution streams",

        ("node_kind.join_title", Language::Zh) => "🔗 Join 汇合",
        ("node_kind.join_title", Language::En) => "🔗 Join Node",
        ("node_kind.join_desc", Language::Zh) => "汇合同步多条流水线",
        ("node_kind.join_desc", Language::En) => "Wait and synchronize parallel branches",

        ("node_kind.math_title", Language::Zh) => "🧮 Math 运算",
        ("node_kind.math_title", Language::En) => "🧮 Math Node",
        ("node_kind.math_desc", Language::Zh) => "数据计算与逻辑判断",
        ("node_kind.math_desc", Language::En) => "Arithmetic calculations and logic expressions",

        ("node_kind.entry_title", Language::Zh) => "🚩 EntryPoint 入口",
        ("node_kind.entry_title", Language::En) => "🚩 EntryPoint",
        ("node_kind.entry_desc", Language::Zh) => "定义事件流触发源",
        ("node_kind.entry_desc", Language::En) => "Define event flow root entry trigger",

        // ---------------- 快捷新建节点 (Quick Add) ----------------
        ("quick_add.title", Language::Zh) => "➕ 快速新建节点 (Quick Add)",
        ("quick_add.title", Language::En) => "➕ Quick Add Node",

        ("quick_add.auto_connect", Language::Zh) => "🔗 自动连接到新节点",
        ("quick_add.auto_connect", Language::En) => "🔗 Auto-connect to new node",

        ("quick_add.auto_connect_in", Language::Zh) => "🔗 自动接入输入端",
        ("quick_add.auto_connect_in", Language::En) => "🔗 Auto-Connect Input",

        ("quick_add.auto_connect_out", Language::Zh) => "🔗 自动引出输出端",
        ("quick_add.auto_connect_out", Language::En) => "🔗 Auto-Connect Output",

        ("quick_add.click_create", Language::Zh) => "点击类型快速创建",
        ("quick_add.click_create", Language::En) => "Click to Create Node",

        ("quick_add.created_node", Language::Zh) => "✨ 已创建并自动连接新节点",
        ("quick_add.created_node", Language::En) => "✨ Created & Connected New Node",


        // 默认节点标题
        ("default_node.action", Language::Zh) => "新动作 (Action)",
        ("default_node.action", Language::En) => "New Action",

        ("default_node.switch", Language::Zh) => "新分支 (Switch)",
        ("default_node.switch", Language::En) => "New Switch",

        ("default_node.query", Language::Zh) => "新查询 (Query)",
        ("default_node.query", Language::En) => "New Query",

        ("default_node.subflow", Language::Zh) => "新子流程 (SubFlow)",
        ("default_node.subflow", Language::En) => "New SubFlow",

        ("default_node.fork", Language::Zh) => "新分流 (Fork)",
        ("default_node.fork", Language::En) => "New Fork",

        ("default_node.join", Language::Zh) => "新汇合 (Join)",
        ("default_node.join", Language::En) => "New Join",

        ("default_node.math", Language::Zh) => "新运算 (Math)",
        ("default_node.math", Language::En) => "New Math",

        ("default_node.entry", Language::Zh) => "新入口 (EntryPoint)",
        ("default_node.entry", Language::En) => "New EntryPoint",

        ("default_node.comment", Language::Zh) => "便签备注 (Comment)",
        ("default_node.comment", Language::En) => "Note (Comment)",

        // ---------------- Toast 提示信息 (Toast Notifications) ----------------
        ("toast.preset_loaded", Language::Zh) => "⚡ 已成功载入 100enemy.bfevfl 官方 EVFL 事件流",
        ("toast.preset_loaded", Language::En) => "⚡ Loaded 100enemy.bfevfl official EVFL graph",

        ("toast.switch_preset", Language::Zh) => "已切换至案例：",
        ("toast.switch_preset", Language::En) => "Switched to preset: ",

        ("toast.auto_layout_done", Language::Zh) => "✨ 已完成 DAG 拓扑分层自动排版",
        ("toast.auto_layout_done", Language::En) => "✨ DAG hierarchical layout completed",

        ("toast.perf_on", Language::Zh) => "🔥 极速性能模式已开启：GPU 全速无锁帧渲染",
        ("toast.perf_on", Language::En) => "🔥 Extreme Perf Mode Enabled (Uncapped GPU Rendering)",

        ("toast.perf_off", Language::Zh) => "🌿 标准模式已恢复：动态按需省电渲染",
        ("toast.perf_off", Language::En) => "🌿 Standard Mode Restored (Dynamic Power Saving)",

        ("toast.expand_params_on", Language::Zh) => "📋 已展开全部节点参数详情",
        ("toast.expand_params_on", Language::En) => "📋 Expanded all node parameters",

        ("toast.expand_params_off", Language::Zh) => "📋 已折叠节点参数详情",
        ("toast.expand_params_off", Language::En) => "📋 Collapsed node parameters",

        ("toast.focus_node", Language::Zh) => "已跳转聚焦至: ",
        ("toast.focus_node", Language::En) => "Focused on node: ",

        ("toast.deleted_node", Language::Zh) => "🗑️ 已删除选中节点",
        ("toast.deleted_node", Language::En) => "🗑️ Deleted selected node",

        ("toast.sever_edge", Language::Zh) => "✂️ 已剪断连线",
        ("toast.sever_edge", Language::En) => "✂️ Severed edge connection",

        ("toast.connect_edge", Language::Zh) => "🔗 已建立连线",
        ("toast.connect_edge", Language::En) => "🔗 Connected edge successfully",

        // ---------------- 角色分类与关键参数 (Actor Theme & Smart Params) ----------------
        ("actor.system", Language::Zh) => "系统逻辑",
        ("actor.system", Language::En) => "System Logic",

        ("actor.player", Language::Zh) => "玩家主角",
        ("actor.player", Language::En) => "Player / Link",

        ("actor.npc", Language::Zh) => "NPC 交互",
        ("actor.npc", Language::En) => "NPC Dialogue",

        ("actor.enemy", Language::Zh) => "敌人战斗",
        ("actor.enemy", Language::En) => "Enemy / Combat",

        ("actor.director", Language::Zh) => "演出导演",
        ("actor.director", Language::En) => "Cinematic / Audio",

        ("param.quest", Language::Zh) => "任务",
        ("param.quest", Language::En) => "Quest",

        ("param.step", Language::Zh) => "阶段",
        ("param.step", Language::En) => "Step",

        ("param.msg", Language::Zh) => "对白",
        ("param.msg", Language::En) => "Dialogue",

        ("param.anim", Language::Zh) => "动作",
        ("param.anim", Language::En) => "Animation",

        ("param.flag", Language::Zh) => "标志位",
        ("param.flag", Language::En) => "Flag",

        ("param.advance_quest", Language::Zh) => "推进任务",
        ("param.advance_quest", Language::En) => "Advance Quest",

        ("param.play_anim", Language::Zh) => "播放动作",
        ("param.play_anim", Language::En) => "Play Anim",

        ("param.show_msg", Language::Zh) => "显示对白",
        ("param.show_msg", Language::En) => "Dialogue",

        // ---------------- 语义分组容器 (Scope Group Frames) ----------------
        ("group.subgraph", Language::Zh) => "剧情流程分区",
        ("group.subgraph", Language::En) => "Event Flow Scope",

        ("group.nodes_count", Language::Zh) => "节点",
        ("group.nodes_count", Language::En) => "nodes",

        // ---------------- 因果链路追踪 (Flow Tracing) ----------------
        ("tracing.upstream", Language::Zh) => "上游前驱",
        ("tracing.upstream", Language::En) => "Upstream",

        ("tracing.downstream", Language::Zh) => "下游分支",
        ("tracing.downstream", Language::En) => "Downstream",

        ("tracing.active", Language::Zh) => "因果链路高亮追踪中",
        ("tracing.active", Language::En) => "Tracing Active Flow Path",

        // 默认回退原字符串
        _ => key,
    }
}




