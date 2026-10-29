use crate::model::{compile_code_to_bytes, write_file_to_target, DocumentTabKind};
use crate::ui::components::Toast;
use crate::ui::state::AppState;

/// 文档标签页保存、持久化写盘与修改放弃扩展
impl AppState {
    /// 保存指定文档标签页（无论是当前活跃或后台标签、流程图还是代码文件，写回磁盘或 SARC）
    pub fn save_document_tab(&mut self, tab_id: &str) -> bool {
        let is_active = self.active_tab_id.as_deref() == Some(tab_id);

        if is_active
            && self.current_tab == crate::ui::state_types::AppWorkspaceTab::CodeEditor
            && !self.sync_code_editor_to_graph_if_needed()
        {
            return false;
        }

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
            let timeline_to_save = tab.timeline_state.as_ref().map(|ts| ts.timeline.clone());
            let file_type = tab.file_type.clone();
            (tab.tab_kind, tab.file_path.clone(), file_type, code_info, graph_to_save, timeline_to_save)
        };

        let (tab_kind, file_path, _file_type, code_info, graph_to_save, timeline_to_save) = tab_info;

        match tab_kind {
            DocumentTabKind::Timeline => {
                let target_timeline = match timeline_to_save {
                    Some(tl) => tl,
                    None => {
                        self.toast = Some(Toast::error(self.t("toast.no_active_file")));
                        return false;
                    }
                };

                let mut evfl = revfl::evfl::EventFlow::new();
                evfl.name = target_timeline.name.clone();
                evfl.timeline = Some(target_timeline.clone());

                let compile_res = crate::model::evfl::save_eventflow_to_bytes(&evfl);
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
                                    if let Some(ref mut ts) = tab.timeline_state {
                                        ts.original_timeline = target_timeline.clone();
                                    }
                                    if let Some(ref mut code) = tab.code_state {
                                        let json_text = serde_json::to_string_pretty(&evfl).unwrap_or_default();
                                        code.content = json_text;
                                        code.mark_saved();
                                    }
                                    tab.is_modified = false;
                                    tab.last_disk_mtime = crate::model::document_tab::get_file_mtime(&file_path);
                                }
                                if self.git_is_repo {
                                    self.refresh_git_status();
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
            DocumentTabKind::Flowchart => {
                let target_graph = match graph_to_save {
                    Some(g) => g,
                    None => {
                        self.toast = Some(Toast::error(self.t("toast.no_active_file")));
                        return false;
                    }
                };

                let compile_res = if let Some((ref content, format, endian)) = code_info {
                    compile_code_to_bytes(content, format, endian, &file_path)
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
                                    if let Some(ref mut code) = tab.code_state {
                                        code.mark_saved();
                                    }
                                    tab.is_modified = false;
                                    tab.last_disk_mtime = crate::model::document_tab::get_file_mtime(&file_path);
                                }
                                if self.git_is_repo {
                                    self.refresh_git_status();
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
                                        tab.last_disk_mtime = crate::model::document_tab::get_file_mtime(&file_path);
                                    }
                                    if self.git_is_repo {
                                        self.refresh_git_status();
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
            DocumentTabKind::DiffViewer => true,
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

    /// 获取当前所有已修改（未保存）标签页的数量
    pub fn modified_tabs_count(&self) -> usize {
        self.open_tabs.iter().filter(|t| t.is_modified).count()
    }

    /// 保存所有包含未保存修改的文档标签页，返回成功保存的文件数量
    pub fn save_all_modified_documents(&mut self) -> usize {
        self.sync_active_tab_modified();
        let dirty_ids: Vec<String> = self.open_tabs.iter().filter(|t| t.is_modified).map(|t| t.id.clone()).collect();
        let mut saved_count = 0;
        for tid in &dirty_ids {
            if self.save_document_tab(tid) {
                saved_count += 1;
            }
        }

        if saved_count > 0 {
            self.toast = Some(Toast::success(format!("{} ({} 项)", self.t("toast.save_success"), saved_count)));
        }
        saved_count
    }

    /// 放弃指定文档标签页的所有未保存修改
    pub fn discard_document_changes(&mut self, tab_id: &str) {
        let is_active = self.active_tab_id.as_deref() == Some(tab_id);
        let mut reverted_graph = None;

        if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == tab_id) {
            match tab.tab_kind {
                DocumentTabKind::Flowchart => {
                    if let Some(ref orig) = tab.original_graph {
                        tab.graph = Some(orig.clone());
                        reverted_graph = Some(orig.clone());
                    }
                    tab.is_modified = false;
                }
                DocumentTabKind::CodeEditor => {
                    if let Some(ref mut code) = tab.code_state {
                        code.revert();
                    }
                    tab.is_modified = false;
                }
                DocumentTabKind::DiffViewer => {
                    tab.is_modified = false;
                }
                DocumentTabKind::Timeline => {
                    if let Some(ref mut tl_state) = tab.timeline_state {
                        tl_state.timeline = tl_state.original_timeline.clone();
                    }
                    if let Some(ref mut code) = tab.code_state {
                        code.revert();
                    }
                    tab.is_modified = false;
                }
            }
        }

        if is_active {
            if let Some(g) = reverted_graph {
                self.graph = g;
                self.fit_view();
            }
        }

        self.toast = Some(Toast::info(self.t("toast.revert_success")));
    }

    /// 放弃所有文档标签页的未保存修改
    pub fn discard_all_modified_documents(&mut self) {
        let dirty_ids: Vec<String> = self
            .open_tabs
            .iter()
            .filter(|t| t.is_modified)
            .map(|t| t.id.clone())
            .collect();

        for tid in dirty_ids {
            self.discard_document_changes(&tid);
        }
    }
}
