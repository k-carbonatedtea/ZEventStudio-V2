use crate::model::{compile_code_to_bytes, write_file_to_target, DocumentTabKind};
use crate::ui::components::Toast;
use crate::ui::state::AppState;

/// 文档标签页保存、代码热重载与修改状态同步扩展
impl AppState {
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

        let (tab_kind, file_path, _file_type, code_info, graph_to_save) = tab_info;

        match tab_kind {
            DocumentTabKind::Flowchart => {
                let target_graph = match graph_to_save {
                    Some(g) => g,
                    None => {
                        self.toast = Some(Toast::error(self.t("toast.no_active_file")));
                        return false;
                    }
                };

                let compile_res = crate::model::evfl::save_bfevfl_to_bytes(&target_graph);

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

    /// 重置当前活跃代码或流程图文档为初始快照
    pub fn revert_active_code(&mut self) {
        if let Some(active_id) = &self.active_tab_id {
            let mut reverted_graph = None;
            if let Some(tab) = self.open_tabs.iter_mut().find(|t| &t.id == active_id) {
                match tab.tab_kind {
                    DocumentTabKind::CodeEditor => {
                        if let Some(ref mut code) = tab.code_state {
                            code.revert();
                        }
                        tab.is_modified = false;
                    }
                    DocumentTabKind::Flowchart => {
                        if let Some(ref orig) = tab.original_graph {
                            tab.graph = Some(orig.clone());
                            reverted_graph = Some(orig.clone());
                        }
                        tab.is_modified = false;
                    }
                    DocumentTabKind::DiffViewer => {
                        tab.is_modified = false;
                    }
                }
                self.toast = Some(Toast::info(self.t("toast.revert_success")));
            }
            if let Some(g) = reverted_graph {
                self.graph = g;
                self.fit_view();
            }
        }
    }

    /// 同步当前活跃文档的修改状态并重算 is_modified
    pub fn sync_active_tab_modified(&mut self) {
        if let Some(cur_id) = self.active_tab_id.as_deref() {
            if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == cur_id) {
                if tab.tab_kind == DocumentTabKind::Flowchart {
                    tab.graph = Some(self.graph.clone());
                }
                tab.recompute_modified();
            }
        }
    }

    /// 获取当前所有已修改（未保存）标签页的数量
    pub fn modified_tabs_count(&self) -> usize {
        self.open_tabs.iter().filter(|t| t.is_modified).count()
    }

    /// 保存所有包含未保存修改的文档标签页，返回成功保存的文件数量
    pub fn save_all_modified_documents(&mut self) -> usize {
        self.sync_active_tab_modified();
        let dirty_ids: Vec<String> = self
            .open_tabs
            .iter()
            .filter(|t| t.is_modified)
            .map(|t| t.id.clone())
            .collect();

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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    /// 从磁盘重新载入指定文档标签页，并刷新其内容快照与界面显示
    pub fn reload_tab_from_disk(&mut self, tab_id: &str) -> Result<(), String> {
        let is_active = self.active_tab_id.as_deref() == Some(tab_id);
        let tab_opt = self.open_tabs.iter().find(|t| t.id == tab_id).cloned();
        let tab = match tab_opt {
            Some(t) => t,
            None => return Err(self.t("toast.no_active_file").to_string()),
        };

        match tab.tab_kind {
            DocumentTabKind::CodeEditor => {
                let bytes = crate::model::file_tree::read_file_from_path(&tab.file_path)?;
                let (code_text, format, endian) =
                    crate::model::file_converter::parse_file_to_code(&bytes, &tab.file_path)?;
                let mtime = crate::model::document_tab::get_file_mtime(&tab.file_path);

                if let Some(tab_mut) = self.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                    if let Some(ref mut code) = tab_mut.code_state {
                        code.reload(code_text, format, endian);
                    }
                    tab_mut.is_modified = false;
                    tab_mut.last_disk_mtime = mtime;
                }
            }
            DocumentTabKind::Flowchart => {
                let bytes = crate::model::file_tree::read_file_from_path(&tab.file_path)?;
                let mut loaded_graph = crate::model::evfl::load_bfevfl_from_bytes(&bytes)?;
                loaded_graph.name = tab.title.clone();
                loaded_graph.ensure_actors_populated();
                crate::model::layout::auto_layout_graph(&mut loaded_graph);
                let mtime = crate::model::document_tab::get_file_mtime(&tab.file_path);

                if let Some(tab_mut) = self.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                    tab_mut.graph = Some(loaded_graph.clone());
                    tab_mut.original_graph = Some(loaded_graph.clone());
                    tab_mut.is_modified = false;
                    tab_mut.last_disk_mtime = mtime;
                }

                if is_active {
                    self.graph = loaded_graph;
                    self.fit_view();
                }
            }
            DocumentTabKind::DiffViewer => {}
        }
        Ok(())
    }

    /// 当窗口激活时检测活动标签页磁盘是否有外部更新，若未在内存中修改则自动重载
    pub fn check_and_reload_active_tab_if_disk_changed(&mut self) {
        let active_id = match self.active_tab_id.clone() {
            Some(id) => id,
            None => return,
        };

        let (file_path, is_modified, last_mtime) = {
            let tab = match self.open_tabs.iter().find(|t| t.id == active_id) {
                Some(t) => t,
                None => return,
            };
            if tab.tab_kind == DocumentTabKind::DiffViewer {
                return;
            }
            (tab.file_path.clone(), tab.is_modified, tab.last_disk_mtime)
        };

        if let Some(disk_mtime) = crate::model::document_tab::get_file_mtime(&file_path) {
            let should_reload = match last_mtime {
                Some(last) => disk_mtime > last && !is_modified,
                None => false,
            };
            if should_reload {
                let _ = self.reload_tab_from_disk(&active_id);
            }
        }
    }
}
