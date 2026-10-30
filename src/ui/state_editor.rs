use crate::model::DocumentTabKind;
use crate::ui::components::Toast;
use crate::ui::state::AppState;

/// 文档标签页代码同步、热重载与修改状态检测扩展
impl AppState {
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
                    DocumentTabKind::ModelPreview => {
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
                    if let Ok((code_text, format, endian)) =
                        crate::model::file_converter::parse_file_to_code(&bytes, &tab.file_path)
                    {
                        if let Some(ref mut code) = tab_mut.code_state {
                            code.reload(code_text, format, endian);
                        }
                    }
                    tab_mut.is_modified = false;
                    tab_mut.last_disk_mtime = mtime;
                }

                if is_active {
                    self.graph = loaded_graph;
                    self.fit_view();
                }
            }
            DocumentTabKind::DiffViewer => {}
            DocumentTabKind::ModelPreview => {
                let bytes = crate::model::file_tree::read_file_from_path(&tab.file_path)?;
                let decompressed = crate::model::file_converter::decompress_if_yaz0(&bytes);
                let bfres = rbfres_library::BfresFile::from_bytes(&decompressed).map_err(|e| e.to_string())?;
                let mtime = crate::model::document_tab::get_file_mtime(&tab.file_path);
                if let Some(tab_mut) = self.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                    let mut new_state = crate::model::ModelDocumentState::from_bfres(&tab.file_path, &tab.title, &bfres);
                    crate::model::model_preview::load_companion_textures(&tab.file_path, &bfres, &mut new_state);
                    tab_mut.model_state = Some(new_state);
                    tab_mut.is_modified = false;
                    tab_mut.last_disk_mtime = mtime;
                }
            }
            DocumentTabKind::Timeline => {
                let bytes = crate::model::file_tree::read_file_from_path(&tab.file_path)?;
                let decompressed = crate::model::file_converter::decompress_if_yaz0(&bytes);
                let evfl = crate::model::evfl::load_eventflow_from_bytes(&decompressed)?;
                if let Some(tl) = evfl.timeline {
                    let mtime = crate::model::document_tab::get_file_mtime(&tab.file_path);
                    if let Some(tab_mut) = self.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                        if let Some(ref mut tl_state) = tab_mut.timeline_state {
                            tl_state.timeline = tl.clone();
                            tl_state.original_timeline = tl;
                        }
                        if let Ok((code_text, format, endian)) =
                            crate::model::file_converter::parse_file_to_code(&bytes, &tab.file_path)
                        {
                            if let Some(ref mut code) = tab_mut.code_state {
                                code.reload(code_text, format, endian);
                            }
                        }
                        tab_mut.is_modified = false;
                        tab_mut.last_disk_mtime = mtime;
                    }
                }
            }
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

    /// 当离开代码编辑视图或保存时，将当前代码编辑器的原生 EventFlow JSON 或 Graph 文本同步反序列化回内存图拓扑
    pub fn sync_code_editor_to_graph_if_needed(&mut self) -> bool {
        let active_id = match self.active_tab_id.as_deref() {
            Some(id) => id.to_string(),
            None => return true,
        };
        let (file_type, code_opt) = match self.open_tabs.iter().find(|t| t.id == active_id) {
            Some(t) => (t.file_type.clone(), t.code_state.clone()),
            None => return true,
        };

        if let Some(code) = code_opt {
            if code.format == crate::model::code_document::CodeFormat::Bfevfl || file_type == "bfevfl" {
                // 1. 优先尝试解析为原生 EventFlow 结构
                let parsed_graph_opt = if let Ok(evfl) = serde_json::from_str::<revfl::evfl::EventFlow>(&code.content) {
                    evfl.flowchart.as_ref().map(|fc| crate::model::evfl::revfl_flowchart_to_graph(fc, &evfl.name))
                } else {
                    None
                };

                // 2. 若原生 EventFlow 解析不适用，降级尝试解析为 Graph
                let mut parsed_graph = match parsed_graph_opt {
                    Some(g) => g,
                    None => match serde_json::from_str::<crate::model::Graph>(&code.content) {
                        Ok(g) => g,
                        Err(e) => {
                            self.toast = Some(Toast::error(format!("{} {}", self.t("toast.json_parse_error"), e)));
                            return false;
                        }
                    },
                };

                let title = self.open_tabs.iter().find(|t| t.id == active_id).map(|t| t.title.clone()).unwrap_or_default();
                if parsed_graph.name.is_empty() {
                    parsed_graph.name = title;
                }
                parsed_graph.ensure_actors_populated();

                // 继承原有图中同 ID 节点的画布坐标与尺寸 (避免反序列化默认零坐标丢失排版)
                for node in &mut parsed_graph.nodes {
                    if let Some(old_node) = self.graph.find_node(&node.id) {
                        if old_node.pos != (0.0, 0.0) {
                            node.pos = old_node.pos;
                            node.size = old_node.size;
                        }
                    }
                }
                if parsed_graph.nodes.iter().all(|n| n.pos == (0.0, 0.0)) && !parsed_graph.nodes.is_empty() {
                    crate::model::layout::auto_layout_graph(&mut parsed_graph);
                }

                self.graph = parsed_graph.clone();
                if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == active_id) {
                    tab.graph = Some(parsed_graph);
                    tab.recompute_modified();
                }
                return true;
            }
        }
        true
    }
}
