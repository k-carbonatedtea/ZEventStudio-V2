use crate::model::plugin::{ApiCommand, PluginApiServer};
use crate::ui::components::Toast;
use crate::ui::state::AppState;
use gpui::*;
use std::sync::Mutex;

/// 全局保留的 API 服务器实例句柄 (使用 Mutex 保证线程安全且零 unsafe)
static API_SERVER_HANDLE: Mutex<Option<PluginApiServer>> = Mutex::new(None);

/// 启动全局插件本地 API 桥接服务器
pub fn start_plugin_bridge_server(model: &Entity<AppState>, cx: &mut App) {
    let (cmd_tx, cmd_rx) = smol::channel::bounded::<ApiCommand>(64);
    let port = 31415;

    match PluginApiServer::start(port, cmd_tx) {
        Ok(server) => {
            if let Ok(mut guard) = API_SERVER_HANDLE.lock() {
                if let Some(old) = guard.take() {
                    old.stop();
                }
                *guard = Some(server);
            }

            model.update(cx, |st, cx| {
                st.api_server_running = true;
                st.api_server_port = port;
                cx.notify();
            });

            // 派发主事件处理循环监听后台通道指令
            let model_loop = model.clone();
            cx.spawn(async move |cx| {
                while let Ok(cmd) = cmd_rx.recv().await {
                    let model_inner = model_loop.clone();
                    cx.update(|cx| {
                        model_inner.update(cx, |st, cx| {
                            handle_api_command(st, cmd);
                            cx.notify();
                        });
                    });
                }
            })
            .detach();
        }
        Err(e) => {
            model.update(cx, |st, cx| {
                st.api_server_running = false;
                st.toast = Some(Toast::error(format!("{}: {}", st.t("plugins.server_start_err"), e)));
                cx.notify();
            });
        }
    }
}

/// 停止全局插件本地 API 桥接服务器
pub fn stop_plugin_bridge_server(model: &Entity<AppState>, cx: &mut App) {
    if let Ok(mut guard) = API_SERVER_HANDLE.lock() {
        if let Some(server) = guard.take() {
            server.stop();
        }
    }
    model.update(cx, |st, cx| {
        st.api_server_running = false;
        st.toast = Some(Toast::info(st.t("plugins.server_stopped_toast")));
        cx.notify();
    });
}

/// 处理单条 API 命令并分发至主线程 AppState
fn handle_api_command(state: &mut AppState, cmd: ApiCommand) {
    match cmd {
        ApiCommand::GetStatus { responder } => {
            let summary = state.get_project_summary_json();
            let _ = responder.send_blocking(summary);
        }
        ApiCommand::GetProject { responder } => {
            let info = serde_json::json!({
                "project_name": state.project_name,
                "project_path": state.project_path,
                "file_count": state.file_tree.len(),
            });
            let _ = responder.send_blocking(info);
        }
        ApiCommand::GetGraph { responder } => {
            let graph_val = serde_json::to_value(&state.graph).unwrap_or(serde_json::Value::Null);
            let _ = responder.send_blocking(graph_val);
        }
        ApiCommand::ExecuteAction { payload, responder } => {
            let result = state.execute_plugin_action(payload);
            let _ = responder.send_blocking(result);
        }
    }
}
