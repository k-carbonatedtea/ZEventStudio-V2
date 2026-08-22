use crate::model::{Edge, NodeKind, Port, PropertyItem};
use crate::ui::canvas::{BranchItem, NodeEditorModalState};
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

            let res_flowchart_name = properties
                .iter()
                .find(|p| p.key == "ResFlowchartName")
                .map(|p| p.val.clone())
                .unwrap_or_default();

            let entry_point_name = properties
                .iter()
                .find(|p| p.key == "EntryPointName")
                .map(|p| p.val.clone())
                .unwrap_or_default();

            // 提取 Switch 分支或 Fork 并行流的目标节点
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

            // 查找 Fork 关联的 Join 目标汇合节点
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

            self.active_node_editor_modal = Some(NodeEditorModalState {
                node_id: node.id.clone(),
                node_kind,
                title,
                subtitle,
                res_flowchart_name,
                entry_point_name,
                join_node_id,
                branches,
                properties,
                is_adding_param: false,
                new_param_type: "Int".to_string(),
                new_param_key: String::new(),
                new_param_val: "0".to_string(),
            });
            self.node_context_menu = None;
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
                node.subtitle = if modal.subtitle.is_empty() {
                    None
                } else {
                    Some(modal.subtitle.clone())
                };

                let mut props = modal.properties.clone();

                if modal.node_kind == NodeKind::SubFlow {
                    if !modal.res_flowchart_name.is_empty() {
                        if let Some(p) = props.iter_mut().find(|p| p.key == "ResFlowchartName") {
                            p.val = modal.res_flowchart_name.clone();
                        } else {
                            props.push(PropertyItem {
                                key: "ResFlowchartName".into(),
                                val: modal.res_flowchart_name.clone(),
                                val_type: "string".into(),
                            });
                        }
                    }
                    if !modal.entry_point_name.is_empty() {
                        if let Some(p) = props.iter_mut().find(|p| p.key == "EntryPointName") {
                            p.val = modal.entry_point_name.clone();
                        } else {
                            props.push(PropertyItem {
                                key: "EntryPointName".into(),
                                val: modal.entry_point_name.clone(),
                                val_type: "string".into(),
                            });
                        }
                    }
                }
                node.properties = props;

                // 同步 Switch / Fork 的分支输出引脚与连线
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

            self.toast = Some(Toast::success(format!("✨ 已成功保存节点 {}", modal_title)));
        }
    }
}

