use crate::model::{DocumentTabKind, Graph};
use crate::ui::state::{AppState, AppWorkspaceTab, UnsavedChangesModalState};

impl AppState {
    /// 切换当前活跃文档标签页
    pub fn switch_document_tab(&mut self, tab_id: &str) {
        if self.active_tab_id.as_deref() == Some(tab_id) {
            return;
        }
        // 1. 先将当前活跃图拓扑缓存至旧标签项
        if let Some(cur_id) = &self.active_tab_id {
            if let Some(tab) = self.open_tabs.iter_mut().find(|t| &t.id == cur_id) {
                if tab.tab_kind == DocumentTabKind::Flowchart {
                    tab.graph = Some(self.graph.clone());
                }
            }
        }

        // 2. 加载目标标签项
        if let Some(target_tab) = self.open_tabs.iter().find(|t| t.id == tab_id).cloned() {
            self.active_tab_id = Some(target_tab.id);
            self.selected_file_path = Some(target_tab.file_path.clone());

            match target_tab.tab_kind {
                DocumentTabKind::Flowchart => {
                    if let Some(cached_graph) = target_tab.graph {
                        self.graph = cached_graph;
                        self.fit_view();
                        self.current_tab = AppWorkspaceTab::Flowchart;
                    } else {
                        self.load_file_by_path(&target_tab.file_path);
                    }
                }
                DocumentTabKind::CodeEditor => {
                    self.current_tab = AppWorkspaceTab::CodeEditor;
                }
            }
        }
    }

    /// 请求关闭指定文档标签页（检查未保存修改，有修改则弹出保存确认提示框）
    pub fn request_close_document_tab(&mut self, tab_id: &str) {
        // 1. 若关闭的是当前活跃流程图标签，确保当前图数据已同步至标签项并精确重算 is_modified
        if self.active_tab_id.as_deref() == Some(tab_id) {
            if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                if tab.tab_kind == DocumentTabKind::Flowchart {
                    tab.graph = Some(self.graph.clone());
                }
                tab.recompute_modified();
            }
        }

        // 2. 检查该标签是否包含未保存更改
        if let Some(tab) = self.open_tabs.iter().find(|t| t.id == tab_id) {
            if tab.is_modified {
                self.unsaved_changes_modal = Some(UnsavedChangesModalState {
                    tab_id: tab.id.clone(),
                    title: tab.title.clone(),
                    file_path: tab.file_path.clone(),
                });
                return;
            }
        }

        // 3. 无未保存修改，直接关闭
        self.close_document_tab(tab_id);
    }

    /// 确认保存并关闭当前弹窗指定的标签页
    pub fn confirm_save_and_close_tab(&mut self) {
        if let Some(modal) = self.unsaved_changes_modal.take() {
            let tab_id = modal.tab_id.clone();
            if self.save_document_tab(&tab_id) {
                self.close_document_tab(&tab_id);
            }
        }
    }

    /// 确认不保存（放弃修改）并关闭当前弹窗指定的标签页
    pub fn confirm_discard_and_close_tab(&mut self) {
        if let Some(modal) = self.unsaved_changes_modal.take() {
            self.close_document_tab(&modal.tab_id);
        }
    }

    /// 取消未保存弹窗
    pub fn cancel_unsaved_changes_modal(&mut self) {
        self.unsaved_changes_modal = None;
    }

    /// 关闭指定文档标签页
    pub fn close_document_tab(&mut self, tab_id: &str) {
        let idx = self.open_tabs.iter().position(|t| t.id == tab_id);
        if let Some(i) = idx {
            self.open_tabs.remove(i);
            if self.active_tab_id.as_deref() == Some(tab_id) {
                if let Some(next_tab) = self
                    .open_tabs
                    .get(i)
                    .or_else(|| self.open_tabs.get(i.saturating_sub(1)))
                {
                    let next_id = next_tab.id.clone();
                    let next_kind = next_tab.tab_kind;
                    let next_graph = next_tab.graph.clone();
                    self.active_tab_id = Some(next_id);
                    self.selected_file_path = Some(next_tab.file_path.clone());

                    match next_kind {
                        DocumentTabKind::Flowchart => {
                            if let Some(g) = next_graph {
                                self.graph = g;
                                self.fit_view();
                            } else {
                                self.graph = Graph::new("EventFlow");
                            }
                            self.current_tab = AppWorkspaceTab::Flowchart;
                        }
                        DocumentTabKind::CodeEditor => {
                            self.graph = Graph::new("EventFlow");
                            self.current_tab = AppWorkspaceTab::CodeEditor;
                        }
                    }
                } else {
                    self.active_tab_id = None;
                    self.selected_file_path = None;
                    self.selected_node_id = None;
                    self.graph = Graph::new("EventFlow");
                    self.current_tab = AppWorkspaceTab::Welcome;
                }
            } else if !self.open_tabs.iter().any(|t| t.tab_kind == DocumentTabKind::Flowchart) {
                // 如果关闭后台标签后已无任何流程图标签，清空旧流程图拓扑
                self.graph = Graph::new("EventFlow");
            }
        }
    }

    /// 关闭所有已打开文档标签页并重置状态
    pub fn close_all_document_tabs(&mut self) {
        self.open_tabs.clear();
        self.active_tab_id = None;
        self.selected_file_path = None;
        self.selected_node_id = None;
        self.graph = Graph::new("EventFlow");
        self.current_tab = AppWorkspaceTab::Welcome;
    }

    /// 关闭除指定标签之外的所有其他标签页
    pub fn close_other_tabs(&mut self, keep_id: &str) {
        // 先切换至该保留标签
        self.switch_document_tab(keep_id);

        // 移除所有未修改的其他标签页
        self.open_tabs.retain(|t| t.id == keep_id || t.is_modified);

        // 若仍有其他带修改的标签页，弹出第一个未保存的确认框
        if let Some(dirty_other) = self.open_tabs.iter().find(|t| t.id != keep_id && t.is_modified) {
            let tid = dirty_other.id.clone();
            self.request_close_document_tab(&tid);
        }
    }

    /// 关闭指定标签右侧的所有标签页
    pub fn close_tabs_to_right(&mut self, target_id: &str) {
        if let Some(target_idx) = self.open_tabs.iter().position(|t| t.id == target_id) {
            let to_check: Vec<(String, bool)> = self.open_tabs
                .iter()
                .skip(target_idx + 1)
                .map(|t| (t.id.clone(), t.is_modified))
                .collect();

            // 移除右侧无修改的标签
            let target_id_str = target_id.to_string();
            let mut kept = Vec::new();
            let mut found_target = false;
            for t in &self.open_tabs {
                if !found_target {
                    if t.id == target_id_str {
                        found_target = true;
                    }
                    kept.push(t.clone());
                } else if t.is_modified {
                    kept.push(t.clone());
                }
            }
            self.open_tabs = kept;

            // 如果当前活跃标签被关闭了，切换回 target
            if let Some(cur_id) = &self.active_tab_id {
                if !self.open_tabs.iter().any(|t| &t.id == cur_id) {
                    self.switch_document_tab(target_id);
                }
            }

            // 若右侧有未保存标签，触发确认提示
            if let Some((dirty_id, _)) = to_check.iter().find(|(_, modified)| *modified) {
                self.request_close_document_tab(dirty_id);
            }
        }
    }

    /// 关闭所有已保存 (未修改) 的标签页
    pub fn close_saved_tabs(&mut self) {
        let is_active_saved = self.active_tab_id.as_deref()
            .and_then(|id| self.open_tabs.iter().find(|t| t.id == id))
            .is_some_and(|t| !t.is_modified);

        self.open_tabs.retain(|t| t.is_modified);

        if self.open_tabs.is_empty() {
            self.close_all_document_tabs();
        } else if is_active_saved {
            if let Some(first_tab) = self.open_tabs.first() {
                let first_id = first_tab.id.clone();
                self.switch_document_tab(&first_id);
            }
        }
    }

    /// 请求全部关闭所有标签页 (遇到修改则逐个提示)
    pub fn request_close_all_tabs(&mut self) {
        // 先移除所有未修改标签
        self.open_tabs.retain(|t| t.is_modified);

        if self.open_tabs.is_empty() {
            self.close_all_document_tabs();
        } else if let Some(first_dirty) = self.open_tabs.first() {
            let tid = first_dirty.id.clone();
            self.request_close_document_tab(&tid);
        }
    }
}
