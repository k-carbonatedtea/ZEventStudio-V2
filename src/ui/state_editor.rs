use crate::model::{Edge, NodeKind, Port, PropertyItem};
use crate::model::{compile_code_to_bytes, write_file_to_target, DocumentTabKind};
use crate::ui::canvas::node_editor::{ActiveEditField, BranchItem, NodeEditorModalState};
use crate::ui::components::Toast;
use crate::ui::state::AppState;

impl AppState {
    /// 打开指定节点的属性与分支编辑模态框
    pub fn open_node_editor(&mut self, node_id: &str) {
        if let Some(node) = self.graph.find_node(node_id) {
            let node_kind = node.kind.clone();
            let title = node.title.clone();
            let subtitle = node.subtitle.clone().unwrap_or_default();
            let properties = node.properties.clone();

            // 1. 精确提取 SubFlow 的流程图名称
            let mut res_flowchart_name = properties
                .iter()
                .find(|p| {
                    p.key.eq_ignore_ascii_case("flowchart")
                        || p.key.eq_ignore_ascii_case("resflowchartname")
                })
                .map(|p| p.val.clone())
                .unwrap_or_default();

            // 2. 精确提取 SubFlow 的入口点名称
            let mut entry_point_name = properties
                .iter()
                .find(|p| {
                    p.key.eq_ignore_ascii_case("entrypoint")
                        || p.key.eq_ignore_ascii_case("entrypointname")
                })
                .map(|p| p.val.clone())
                .unwrap_or_default();

            // 3. 若属性未显式定义，从副标题中解析
            if res_flowchart_name.is_empty() && entry_point_name.is_empty() {
                if let Some(ref sub) = node.subtitle {
                    if let Some((fc, ep)) = sub.split_once(" :: ") {
                        res_flowchart_name = fc.trim().to_string();
                        entry_point_name = ep.trim().to_string();
                    } else if !sub.is_empty() {
                        entry_point_name = sub.trim().to_string();
                    }
                }
            }

            // 4. 对 SubFlow 过滤伪属性
            let mut clean_properties = properties;
            if node_kind == NodeKind::SubFlow {
                clean_properties.retain(|p| {
                    !p.key.eq_ignore_ascii_case("flowchart")
                        && !p.key.eq_ignore_ascii_case("resflowchartname")
                        && !p.key.eq_ignore_ascii_case("entrypoint")
                        && !p.key.eq_ignore_ascii_case("entrypointname")
                });
            }

            // 5. 提取 Action / Query 的参与 Actor 与方法名
            let mut actor_name = String::new();
            let mut action_name = String::new();
            if node_kind != NodeKind::SubFlow {
                if let Some(ref sub) = node.subtitle {
                    if let Some((act, act_or_q)) = sub.split_once(" :: ") {
                        actor_name = act.trim().to_string();
                        action_name = act_or_q.trim().to_string();
                    } else if !sub.is_empty() {
                        actor_name = sub.trim().to_string();
                    }
                }
            }

            let mut branches = Vec::new();
            match node_kind {
                NodeKind::Switch => {
                    for port in &node.outputs {
                        let target_id = self
                            .graph
                            .edges
                            .iter()
                            .find(|e| e.source_node_id == node.id && e.source_port_id == port.id)
                            .map(|e| e.target_node_id.clone())
                            .unwrap_or_default();
                        branches.push(BranchItem {
                            value: port.name.replace("Case ", "").replace("case_", ""),
                            target_id,
                        });
                    }
                    if branches.is_empty() {
                        branches.push(BranchItem {
                            value: "0".to_string(),
                            target_id: String::new(),
                        });
                        branches.push(BranchItem {
                            value: "1".to_string(),
                            target_id: String::new(),
                        });
                    }
                }
                NodeKind::Fork => {
                    for port in &node.outputs {
                        let target_id = self
                            .graph
                            .edges
                            .iter()
                            .find(|e| e.source_node_id == node.id && e.source_port_id == port.id)
                            .map(|e| e.target_node_id.clone())
                            .unwrap_or_default();
                        branches.push(BranchItem {
                            value: port.name.clone(),
                            target_id,
                        });
                    }
                    if branches.is_empty() {
                        branches.push(BranchItem {
                            value: "Branch 1".to_string(),
                            target_id: String::new(),
                        });
                        branches.push(BranchItem {
                            value: "Branch 2".to_string(),
                            target_id: String::new(),
                        });
                    }
                }
                _ => {}
            }

            let join_node_id = self
                .graph
                .edges
                .iter()
                .find(|e| {
                    e.source_node_id == node.id
                        && self
                            .graph
                            .find_node(&e.target_node_id)
                            .map(|n| n.kind == NodeKind::Join)
                            .unwrap_or(false)
                })
                .map(|e| e.target_node_id.clone());

            self.node_context_menu = None;
            self.active_node_editor_modal = Some(NodeEditorModalState {
                node_id: node.id.clone(),
                node_kind,
                title,
                subtitle,
                actor_name,
                action_name,
                show_actor_dropdown: false,
                show_action_dropdown: false,
                res_flowchart_name,
                entry_point_name,
                join_node_id,
                branches,
                properties: clean_properties,
                active_field: ActiveEditField::None,
                is_adding_param: false,
                new_param_type: "string".to_string(),
                new_param_key: String::new(),
                new_param_val: String::new(),
            });
        }
    }

    /// 关闭节点编辑模态框
    pub fn close_node_editor(&mut self) {
        self.active_node_editor_modal = None;
    }

    /// 保存节点编辑模态框所做的属性/分支修改
    pub fn save_node_editor(&mut self) {
        if let Some(modal) = self.active_node_editor_modal.take() {
            let node_id = modal.node_id.clone();
            let modal_title = modal.title.clone();

            if let Some(node) = self.graph.find_node_mut(&node_id) {
                node.title = modal.title.clone();
                let mut props = modal.properties.clone();

                if modal.node_kind == NodeKind::SubFlow {
                    props.retain(|p| {
                        !p.key.eq_ignore_ascii_case("flowchart")
                            && !p.key.eq_ignore_ascii_case("resflowchartname")
                            && !p.key.eq_ignore_ascii_case("entrypoint")
                            && !p.key.eq_ignore_ascii_case("entrypointname")
                    });
                    if !modal.entry_point_name.is_empty() {
                        props.push(PropertyItem {
                            key: "EntryPoint".to_string(),
                            val: modal.entry_point_name.clone(),
                            val_type: "string".to_string(),
                        });
                    }
                    if !modal.res_flowchart_name.is_empty() {
                        props.push(PropertyItem {
                            key: "Flowchart".to_string(),
                            val: modal.res_flowchart_name.clone(),
                            val_type: "string".to_string(),
                        });
                    }
                    let sub = if modal.res_flowchart_name.is_empty() {
                        modal.entry_point_name.clone()
                    } else {
                        format!("{} :: {}", modal.res_flowchart_name, modal.entry_point_name)
                    };
                    node.subtitle = Some(sub);
                } else if modal.node_kind == NodeKind::Action || modal.node_kind == NodeKind::Query {
                    let sub = if modal.actor_name.is_empty() {
                        None
                    } else if modal.action_name.is_empty() {
                        Some(modal.actor_name.clone())
                    } else {
                        Some(format!("{} :: {}", modal.actor_name, modal.action_name))
                    };
                    node.subtitle = sub;
                } else {
                    node.subtitle = if modal.subtitle.is_empty() {
                        None
                    } else {
                        Some(modal.subtitle.clone())
                    };
                }
                node.properties = props;

                // 同步 Switch / Fork 输出引脚
                if matches!(modal.node_kind, NodeKind::Switch | NodeKind::Fork) {
                    node.outputs.clear();
                    for (idx, branch) in modal.branches.iter().enumerate() {
                        let port_id = format!("{}_out{}", modal.node_id, idx);
                        let port_name = if modal.node_kind == NodeKind::Switch {
                            format!("Case {}", branch.value)
                        } else {
                            format!("Branch {}", idx + 1)
                        };
                        node.outputs.push(Port::new_flow_out(port_id.clone(), port_name));
                    }
                }
            }

            // 同步分支的外部连线
            if matches!(modal.node_kind, NodeKind::Switch | NodeKind::Fork) {
                self.graph.edges.retain(|e| e.source_node_id != node_id);
                for (idx, branch) in modal.branches.iter().enumerate() {
                    let port_id = format!("{}_out{}", node_id, idx);
                    if !branch.target_id.is_empty()
                        && self.graph.find_node(&branch.target_id).is_some()
                    {
                        let target_port = self
                            .graph
                            .find_node(&branch.target_id)
                            .and_then(|t| t.inputs.first().map(|p| p.id.clone()))
                            .unwrap_or_else(|| format!("{}_in", branch.target_id));
                        self.graph.add_edge(Edge::new(
                            &node_id,
                            &port_id,
                            &branch.target_id,
                            &target_port,
                        ));
                    }
                }
            }

            self.mark_current_tab_modified();
            let msg = format!("{} {}", self.t("toast.node_saved"), modal_title);
            self.toast = Some(Toast::success(&msg));
        }
    }

    /// 保存指定文档标签页（无论是当前活跃或后台标签、流程图还是代码文件，写回磁盘或 SARC）
    pub fn save_document_tab(&mut self, tab_id: &str) -> bool {
        let is_active = self.active_tab_id.as_deref() == Some(tab_id);

        let tab_info = {
            let tab = match self.open_tabs.iter().find(|t| t.id == tab_id) {
                Some(t) => t,
                None => return false,
            };
            let code_info = tab.code_state.as_ref().map(|c| (c.content.clone(), c.format, c.endian));
            let graph_to_save = if is_active && tab.tab_kind == DocumentTabKind::Flowchart {
                Some(self.graph.clone())
            } else {
                tab.graph.clone()
            };
            let file_type = tab.file_type.clone();
            (tab.tab_kind, tab.file_path.clone(), file_type, code_info, graph_to_save)
        };

        let (tab_kind, file_path, file_type, code_info, graph_to_save) = tab_info;

        match tab_kind {
            DocumentTabKind::Flowchart => {
                let target_graph = match graph_to_save {
                    Some(g) => g,
                    None => {
                        self.toast = Some(Toast::error(self.t("toast.no_active_file")));
                        return false;
                    }
                };

                let is_bai = file_type == "baiprog" || crate::model::is_baiprog_extension(&file_path);
                let compile_res = if is_bai {
                    let is_text = file_path.ends_with(".txt") || file_path.ends_with(".yaml") || file_path.ends_with(".yml");
                    crate::model::baiprog::save_baiprog_to_bytes(&target_graph, is_text)
                } else {
                    crate::model::evfl::save_bfevfl_to_bytes(&target_graph)
                };

                match compile_res {
                    Ok(bytes) => {
                        let final_bytes = if crate::model::should_compress_yaz0(&file_path) {
                            roead::yaz0::compress(&bytes)
                        } else {
                            bytes
                        };
                        match write_file_to_target(&file_path, &final_bytes) {
                            Ok(_) => {
                                if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                                    tab.original_graph = Some(target_graph.clone());
                                    tab.graph = Some(target_graph.clone());
                                    tab.is_modified = false;
                                }
                                self.toast = Some(Toast::success(self.t("toast.save_success")));
                                true
                            }
                            Err(e) => {
                                self.toast = Some(Toast::error(format!("{}: {}", self.t("toast.save_failed"), e)));
                                false
                            }
                        }
                    }
                    Err(e) => {
                        self.toast = Some(Toast::error(format!("{}: {}", self.t("toast.compile_failed"), e)));
                        false
                    }
                }
            }
            DocumentTabKind::CodeEditor => {
                if let Some((content, format, endian)) = code_info {
                    match compile_code_to_bytes(&content, format, endian, &file_path) {
                        Ok(bytes) => {
                            match write_file_to_target(&file_path, &bytes) {
                                Ok(_) => {
                                    if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                                        if let Some(ref mut code) = tab.code_state {
                                            code.mark_saved();
                                        }
                                        tab.is_modified = false;
                                    }
                                    self.toast = Some(Toast::success(self.t("toast.save_success")));
                                    true
                                }
                                Err(e) => {
                                    self.toast = Some(Toast::error(format!("{}: {}", self.t("toast.save_failed"), e)));
                                    false
                                }
                            }
                        }
                        Err(e) => {
                            self.toast = Some(Toast::error(e));
                            false
                        }
                    }
                } else {
                    false
                }
            }
        }
    }

    /// 保存当前活跃文档（无论是流程图还是代码编辑器文件，自动写回磁盘或 SARC）
    pub fn save_active_document(&mut self) -> bool {
        let active_id = match self.active_tab_id.clone() {
            Some(id) => id,
            None => {
                self.toast = Some(Toast::info(self.t("toast.no_active_file")));
                return false;
            }
        };
        self.save_document_tab(&active_id)
    }

    /// 更新当前活跃代码编辑器的文本内容
    #[allow(dead_code)]
    pub fn update_active_code_text(&mut self, text: &str) {
        if let Some(active_id) = &self.active_tab_id {
            if let Some(tab) = self.open_tabs.iter_mut().find(|t| &t.id == active_id) {
                if let Some(ref mut code) = tab.code_state {
                    code.content = text.to_string();
                }
                tab.recompute_modified();
            }
        }
    }

    /// 重置当前活跃代码编辑器的文本内容为初始快照
    pub fn revert_active_code(&mut self) {
        if let Some(active_id) = &self.active_tab_id {
            if let Some(tab) = self.open_tabs.iter_mut().find(|t| &t.id == active_id) {
                if let Some(ref mut code) = tab.code_state {
                    code.revert();
                }
                tab.is_modified = false;
                self.toast = Some(Toast::info(self.t("toast.revert_success")));
            }
        }
    }

    /// 基于 AI 定义文件自动填充 Action/Query 节点的参数属性
    pub fn autofill_node_parameters(&mut self, node_id: &str) {
        let (actor_name, action_query_name, is_query) = {
            if let Some(node) = self.graph.find_node(node_id) {
                let is_q = node.kind == crate::model::NodeKind::Query;
                if node.kind != crate::model::NodeKind::Action && !is_q {
                    return;
                }

                let actor = node.properties.iter()
                    .find(|p| p.key == "ActorName")
                    .map(|p| p.val.clone())
                    .unwrap_or_default();
                let name = node.properties.iter()
                    .find(|p| p.key == "Name")
                    .map(|p| p.val.clone())
                    .unwrap_or_default();

                if actor.is_empty() || name.is_empty() {
                    self.toast = Some(Toast::error(self.t("toast.node_missing_info")));
                    return;
                }
                (actor, name, is_q)
            } else {
                return;
            }
        };

        let search_dirs = crate::model::ai_def::build_search_dirs(
            Some(&self.game_paths.switch.game_dir),
            None,
            Some(&self.game_paths.switch.dlc_dir),
            None,
            Some(&self.game_paths.custom_ai_dump_dir),
        );

        match crate::model::ai_def::get_autofill_parameters(
            &actor_name, &action_query_name, is_query, &search_dirs,
        ) {
            Ok(params) => {
                if let Some(node) = self.graph.find_node_mut(node_id) {
                    let mut added_count = 0;
                    for (key, value) in params {
                        if !node.properties.iter().any(|p| p.key == key) {
                            if let Some(obj) = value.as_object() {
                                if let Some((param_type, param_val)) = obj.iter().next() {
                                    node.properties.push(PropertyItem {
                                        val_type: param_type.clone(),
                                        key: key.clone(),
                                        val: if let Some(s) = param_val.as_str() {
                                            s.to_string()
                                        } else {
                                            param_val.to_string()
                                        },
                                    });
                                    added_count += 1;
                                }
                            }
                        }
                    }
                    if added_count > 0 {
                        self.toast = Some(Toast::success(
                            format!("{} {}", self.t("toast.params_autofilled"), added_count),
                        ));
                    } else {
                        self.toast = Some(Toast::success(self.t("toast.params_complete")));
                    }
                }
            }
            Err(e) => {
                let msg = format!("{}: {}", self.t("toast.autofill_failed"), e);
                self.toast = Some(Toast::error(&msg));
            }
        }
    }
}
