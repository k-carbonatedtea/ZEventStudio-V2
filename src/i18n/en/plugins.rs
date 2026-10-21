/// Plugins Center and Extension System English dictionary
pub fn get_en_plugins(key: &str) -> Option<&'static str> {
    let text = match key {
        "plugins.title" => "Plugins Center",
        "plugins.scan_completed" => "Plugins scan completed",
        "plugins.status_enabled" => "Enabled",
        "plugins.status_disabled" => "Disabled",
        "plugins.search_placeholder" => "Search installed plugins...",
        "plugins.no_plugins" => "No Plugins Found",
        "plugins.empty_hint" => "Add plugin folders with plugin.json to plugins directory",
        "plugins.api_server" => "Local API Bridge",
        "plugins.api_server_running" => "Running",
        "plugins.api_server_stopped" => "Stopped",
        "plugins.mcp_start" => "Start Server",
        "plugins.mcp_stop" => "Stop Server",
        "plugins.copy_claude" => "📋 Claude Config",
        "plugins.copy_cursor" => "📋 Cursor / AGY Config",
        "plugins.copied_toast" => "MCP configuration copied to clipboard",
        "plugins.commands" => "Commands",
        "plugins.tools" => "Tools",
        "plugins.server_start_err" => "Failed to start local API bridge server",
        "plugins.server_stopped_toast" => "Local API bridge server stopped",
        _ => return None,
    };
    Some(text)
}
