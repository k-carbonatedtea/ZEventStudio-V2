/// 插件中心与扩展系统中文本地化字典
pub fn get_zh_plugins(key: &str) -> Option<&'static str> {
    let text = match key {
        "plugins.title" => "插件中心",
        "plugins.scan_completed" => "插件扫描完成",
        "plugins.status_enabled" => "已启用",
        "plugins.status_disabled" => "已禁用",
        "plugins.search_placeholder" => "搜索已安装插件...",
        "plugins.no_plugins" => "未发现插件",
        "plugins.empty_hint" => "可在插件目录添加带有 plugin.json 的插件文件夹",
        "plugins.api_server" => "本地 API 桥接",
        "plugins.api_server_running" => "运行中",
        "plugins.api_server_stopped" => "未启动",
        "plugins.mcp_start" => "启动服务",
        "plugins.mcp_stop" => "停止服务",
        "plugins.copy_claude" => "📋 Claude Desktop 配置",
        "plugins.copy_cursor" => "📋 Cursor / AGY 配置",
        "plugins.copied_toast" => "已将 MCP 配置复制到剪贴板",
        "plugins.commands" => "个命令",
        "plugins.tools" => "项工具",
        "plugins.server_start_err" => "启动本地 API 桥接服务失败",
        "plugins.server_stopped_toast" => "本地 API 桥接服务已停止",
        _ => return None,
    };
    Some(text)
}
