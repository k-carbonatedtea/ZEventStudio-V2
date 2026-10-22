use crate::model::{NodeData, NodeKind, PropertyItem};
use crate::ui::components::Toast;
use crate::ui::state::AppState;
use gpui::*;
use std::path::PathBuf;

impl AppState {
    /// 重新扫描插件目录并刷新已加载的插件清单
    pub fn rescan_plugins(&mut self) {
        self.plugin_manager.rescan();
        let count = self.plugin_manager.plugins.len();
        let active = self.plugin_manager.active_count();
        self.toast = Some(Toast::success(format!(
            "{}: {} ({} {})",
            self.t("plugins.scan_completed"),
            count,
            active,
            self.t("plugins.status_enabled")
        )));
    }

    /// 切换指定插件的启用状态
    pub fn toggle_plugin(&mut self, plugin_id: &str) {
        if self.plugin_manager.toggle_plugin(plugin_id) {
            let is_enabled = self
                .plugin_manager
                .find_plugin(plugin_id)
                .map(|p| p.is_enabled())
                .unwrap_or(false);
            let status_text = if is_enabled {
                self.t("plugins.status_enabled")
            } else {
                self.t("plugins.status_disabled")
            };
            self.toast = Some(Toast::info(format!("{} -> {}", plugin_id, status_text)));
        }
    }

    /// 获取处于激活状态的插件数量
    pub fn active_plugins_count(&self) -> usize {
        self.plugin_manager.active_count()
    }

    /// 在操作系统文件管理器中打开插件存放物理目录
    pub fn open_plugins_folder(&mut self) {
        let _ = self.plugin_manager.ensure_plugins_dir();
        let target_dir = &self.plugin_manager.plugins_dir;

        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(target_dir)
                .spawn();
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(target_dir)
                .spawn();
        }
    }

    /// 获取 MCP 插件绝对路径与脚本路径
    pub fn get_mcp_script_path(&self) -> Option<PathBuf> {
        let mcp_plugin = self.plugin_manager.find_plugin("zevent-mcp-bridge")?;
        let script_path = mcp_plugin.dir_path.join("mcp_server.py");
        if script_path.exists() {
            Some(script_path)
        } else {
            None
        }
    }

    /// 生成 Claude Desktop 使用的 MCP 服务器配置 JSON 文本
    pub fn generate_claude_config(&self) -> String {
        let script = self
            .get_mcp_script_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                self.plugin_manager
                    .plugins_dir
                    .join("mcp_bridge")
                    .join("mcp_server.py")
                    .to_string_lossy()
                    .to_string()
            });

        serde_json::json!({
            "mcpServers": {
                "zevent-studio": {
                    "command": "python",
                    "args": [script]
                }
            }
        })
        .to_string()
    }

    /// 生成 Cursor / Antigravity 使用的 MCP 配置 JSON 文本
    pub fn generate_cursor_config(&self) -> String {
        let script = self
            .get_mcp_script_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                self.plugin_manager
                    .plugins_dir
                    .join("mcp_bridge")
                    .join("mcp_server.py")
                    .to_string_lossy()
                    .to_string()
            });

        serde_json::json!({
            "mcpServers": {
                "zevent-studio": {
                    "command": "python",
                    "args": [script]
                }
            }
        })
        .to_string()
    }

    /// 获取当前打开的工程项目概览 JSON 结构
    pub fn get_project_summary_json(&self) -> serde_json::Value {
        let active_file = self
            .active_tab_id
            .as_ref()
            .and_then(|id| self.open_tabs.iter().find(|t| &t.id == id))
            .map(|t| t.file_path.clone());

        let tabs: Vec<serde_json::Value> = self
            .open_tabs
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "title": t.title,
                    "file_path": t.file_path,
                    "file_type": t.file_type,
                    "tab_kind": format!("{:?}", t.tab_kind),
                    "is_modified": t.is_modified,
                    "is_active": self.active_tab_id.as_deref() == Some(&t.id),
                })
            })
            .collect();

        serde_json::json!({
            "project_name": self.project_name,
            "project_path": self.project_path,
            "active_file": active_file,
            "file_count": self.file_tree.len(),
            "open_tabs": tabs,
            "open_tabs_count": self.open_tabs.len(),
            "target_engine": format!("{:?}", self.target_engine),
            "actors_count": self.graph.actors.len(),
            "nodes_count": self.graph.nodes.len(),
            "entry_points_count": self.graph.nodes.iter().filter(|n| n.kind == NodeKind::EntryPoint).count(),
        })
    }

    /// 执行插件或外部发起的图修改指令
    pub fn execute_plugin_action(
        &mut self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let action = payload
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 action 操作字段".to_string())?;

        match action {
            "add_node" => {
                let name = payload
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("NewAction")
                    .to_string();
                let kind_str = payload
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Action");
                let kind = match kind_str {
                    "Query" => NodeKind::Query,
                    "Fork" => NodeKind::Fork,
                    "Join" => NodeKind::Join,
                    "Switch" => NodeKind::Switch,
                    "Subflow" | "SubFlow" => NodeKind::SubFlow,
                    "EntryPoint" => NodeKind::EntryPoint,
                    "Comment" => NodeKind::Comment,
                    _ => NodeKind::Action,
                };
                let actor = payload
                    .get("actor")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let x = payload
                    .get("x")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(200.0);
                let y = payload
                    .get("y")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or(200.0);

                let (gen_id, gen_title) = self.graph.generate_unique_node_name(&kind);
                let node_id = payload
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(gen_id);
                let title = if name.is_empty() || name == "NewAction" {
                    gen_title
                } else {
                    name.clone()
                };

                let mut node = NodeData::new(node_id.clone(), title, kind, (x, y));
                if let Some(act) = actor {
                    node.subtitle = Some(format!("{} :: {}", act, name));
                }

                // 装载属性
                if let Some(params_obj) = payload.get("params").and_then(|v| v.as_object()) {
                    for (k, v) in params_obj {
                        let val_str = if let Some(s) = v.as_str() {
                            s.to_string()
                        } else {
                            v.to_string()
                        };
                        node.properties.push(PropertyItem {
                            val_type: "string".to_string(),
                            key: k.clone(),
                            val: val_str,
                        });
                    }
                }

                let new_id = node.id.clone();
                let should_autofill = payload.get("autofill").and_then(|v| v.as_bool()).unwrap_or(false);
                self.graph.nodes.push(node);
                if should_autofill {
                    self.autofill_node_parameters(&new_id);
                }
                self.mark_current_tab_modified();
                self.toast = Some(Toast::success(format!("MCP: 添加节点 「{}」", name)));
                Ok(serde_json::json!({ "status": "ok", "node_id": new_id }))
            }
            "connect_nodes" => {
                let from_node = payload
                    .get("from_node")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "缺少 from_node".to_string())?;
                let to_node = payload
                    .get("to_node")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "缺少 to_node".to_string())?;
                let from_port = payload
                    .get("from_port")
                    .and_then(|v| v.as_str())
                    .unwrap_or("out");
                let to_port = payload
                    .get("to_port")
                    .and_then(|v| v.as_str())
                    .unwrap_or("in");

                let edge = crate::model::edge::Edge::new(from_node, from_port, to_node, to_port);
                let edge_id = edge.id.clone();
                self.graph.add_edge(edge);
                self.mark_current_tab_modified();
                Ok(serde_json::json!({ "status": "ok", "edge_id": edge_id }))
            }
            "update_node" => {
                let node_id = payload
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "缺少 node_id".to_string())?;
                let node = self.graph.find_node_mut(node_id)
                    .ok_or_else(|| format!("未找到节点 {}", node_id))?;
                if let Some(name) = payload.get("name").and_then(|v| v.as_str()) {
                    node.title = name.to_string();
                }
                if let Some(actor) = payload.get("actor").and_then(|v| v.as_str()) {
                    node.subtitle = Some(format!("{} :: {}", actor, node.title));
                }
                if let Some(params_obj) = payload.get("params").and_then(|v| v.as_object()) {
                    for (k, v) in params_obj {
                        let val_str = if let Some(s) = v.as_str() { s.to_string() } else { v.to_string() };
                        if let Some(item) = node.properties.iter_mut().find(|p| &p.key == k) {
                            item.val = val_str;
                        } else {
                            node.properties.push(PropertyItem {
                                val_type: "string".to_string(),
                                key: k.clone(),
                                val: val_str,
                            });
                        }
                    }
                }
                let should_autofill = payload.get("autofill").and_then(|v| v.as_bool()).unwrap_or(false);
                let should_reorder = payload.get("reorder").and_then(|v| v.as_bool()).unwrap_or(false);
                if should_autofill {
                    self.autofill_node_parameters(node_id);
                } else if should_reorder {
                    self.reorder_node_parameters(node_id);
                }
                self.mark_current_tab_modified();
                Ok(serde_json::json!({ "status": "ok", "updated": node_id }))
            }
            "delete_node" => {
                let node_id = payload
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "缺少 node_id".to_string())?;
                self.graph.remove_node(node_id);
                self.mark_current_tab_modified();
                Ok(serde_json::json!({ "status": "ok", "deleted": node_id }))
            }
            "auto_layout" => {
                crate::model::layout::auto_layout_graph(&mut self.graph);
                self.fit_view();
                self.toast = Some(Toast::info(self.t("toast.auto_layout_done")));
                Ok(serde_json::json!({ "status": "ok" }))
            }
            "toast" => {
                let msg = payload
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("来自插件的通知");
                self.toast = Some(Toast::info(msg));
                Ok(serde_json::json!({ "status": "ok" }))
            }
            "autofill_parameters" => {
                let node_id = payload
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "缺少 node_id".to_string())?;
                self.autofill_node_parameters(node_id);
                let node = self.graph.find_node(node_id)
                    .ok_or_else(|| format!("未找到节点 {}", node_id))?;
                let props: Vec<_> = node.properties.iter().map(|p| {
                    serde_json::json!({
                        "key": p.key,
                        "type": p.val_type,
                        "value": p.val
                    })
                }).collect();
                Ok(serde_json::json!({
                    "status": "ok",
                    "node_id": node_id,
                    "properties": props
                }))
            }
            "reorder_parameters" => {
                let node_id = payload
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "缺少 node_id".to_string())?;
                self.reorder_node_parameters(node_id);
                let node = self.graph.find_node(node_id)
                    .ok_or_else(|| format!("未找到节点 {}", node_id))?;
                let props: Vec<_> = node.properties.iter().map(|p| {
                    serde_json::json!({
                        "key": p.key,
                        "type": p.val_type,
                        "value": p.val
                    })
                }).collect();
                Ok(serde_json::json!({
                    "status": "ok",
                    "node_id": node_id,
                    "properties": props
                }))
            }
            "get_ai_parameters" => {
                let actor = payload.get("actor").and_then(|v| v.as_str()).unwrap_or("");
                let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let is_query = payload.get("is_query").and_then(|v| v.as_bool()).unwrap_or(false);
                let search_dirs = self.build_game_ai_search_dirs();
                match crate::model::ai_def::get_autofill_parameters(actor, name, is_query, &search_dirs) {
                    Ok(params) => Ok(serde_json::json!({
                        "status": "ok",
                        "actor": actor,
                        "name": name,
                        "parameters": params
                    })),
                    Err(e) => Err(format!("获取游戏参数失败: {}", e)),
                }
            }
            _ => self.execute_plugin_file_action(action, &payload),
        }
    }
}

pub use super::state_plugin_bridge::{start_plugin_bridge_server, stop_plugin_bridge_server};
