use crate::model::code_document::CodeDocumentState;
use crate::model::file_converter::parse_file_to_code;
use crate::model::file_tree::{read_file_from_path, scan_project_path};
use crate::model::{DocumentTabKind, Graph, OpenDocumentTab};
use crate::ui::components::Toast;
use crate::ui::sidebar::SidebarTab;
use crate::ui::state::{AppState, AppWorkspaceTab, RecentProjectItem};

impl AppState {
    /// 打开并解析指定 Mod 根目录 (如 content 或 romfs)
    pub fn open_project_directory(&mut self, path: &str) {
        match scan_project_path(path) {
            Ok((name, nodes)) => {
                self.project_path = Some(path.to_string());
                self.project_name = Some(name.clone());
                self.file_tree = nodes;

                // 默认展开顶层前两级容器
                self.expanded_tree_paths.clear();
                for node in &self.file_tree {
                    if node.is_dir || node.is_sarc {
                        self.expanded_tree_paths.insert(node.path.clone());
                        if let Some(children) = &node.children {
                            for child in children {
                                if child.is_dir || child.is_sarc {
                                    self.expanded_tree_paths.insert(child.path.clone());
                                }
                            }
                        }
                    }
                }

                self.add_recent_project(name.clone(), path.to_string(), "mod".to_string());
                self.active_sidebar_tab = SidebarTab::Files;
                self.toast = Some(Toast::success(format!("{} {}", self.t("toast.project_loaded"), name)));
            }
            Err(e) => {
                self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), e)));
            }
        }
    }

    /// 打开并解析单个 SARC 封包文件 (如 .sbeventpack / .sbactorpack / .pack)
    pub fn open_pack_file(&mut self, path: &str) {
        match scan_project_path(path) {
            Ok((name, nodes)) => {
                self.project_path = Some(path.to_string());
                self.project_name = Some(name.clone());
                self.file_tree = nodes;

                self.expanded_tree_paths.clear();
                for node in &self.file_tree {
                    if node.is_dir || node.is_sarc {
                        self.expanded_tree_paths.insert(node.path.clone());
                    }
                }

                let ext = path.split('.').next_back().unwrap_or("pack").to_string();
                self.add_recent_project(name.clone(), path.to_string(), ext);
                self.active_sidebar_tab = SidebarTab::Files;
                self.toast = Some(Toast::success(format!("{} {}", self.t("toast.project_loaded"), name)));
            }
            Err(e) => {
                self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), e)));
            }
        }
    }

    /// 重新扫描并刷新当前项目文件树
    pub fn reload_project(&mut self) {
        if let Some(path) = self.project_path.clone() {
            if let Ok((_name, nodes)) = scan_project_path(&path) {
                self.file_tree = nodes;
                self.toast = Some(Toast::info(self.t("explorer.refresh_done")));
            }
        }
    }

    /// 切换文件树中目录或 SARC 封包节点的展开/折叠状态
    pub fn toggle_tree_path(&mut self, path: &str) {
        if self.expanded_tree_paths.contains(path) {
            self.expanded_tree_paths.remove(path);
        } else {
            self.expanded_tree_paths.insert(path.to_string());
        }
    }

    /// 全部折叠文件树
    pub fn collapse_all_tree_paths(&mut self) {
        self.expanded_tree_paths.clear();
    }

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
                self.unsaved_changes_modal = Some(crate::ui::state::UnsavedChangesModalState {
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

    /// 智能切换主工作区模式 (与已打开文档标签协同联动)
    pub fn switch_workspace_tab(&mut self, target_tab: AppWorkspaceTab) {
        match target_tab {
            AppWorkspaceTab::Flowchart => {
                let is_flow_active = self
                    .active_tab_id
                    .as_deref()
                    .and_then(|id| self.open_tabs.iter().find(|t| t.id == id))
                    .is_some_and(|t| t.tab_kind == DocumentTabKind::Flowchart);

                if is_flow_active {
                    self.current_tab = AppWorkspaceTab::Flowchart;
                } else if let Some(existing_flow) = self
                    .open_tabs
                    .iter()
                    .find(|t| t.tab_kind == DocumentTabKind::Flowchart)
                    .cloned()
                {
                    self.switch_document_tab(&existing_flow.id);
                } else {
                    // 没有打开任何流程图文件时，重置为空白图，避免残留已关闭文件的节点
                    self.graph = Graph::new("EventFlow");
                    self.current_tab = AppWorkspaceTab::Flowchart;
                }
            }
            AppWorkspaceTab::CodeEditor => {
                let is_code_active = self
                    .active_tab_id
                    .as_deref()
                    .and_then(|id| self.open_tabs.iter().find(|t| t.id == id))
                    .is_some_and(|t| t.tab_kind == DocumentTabKind::CodeEditor);

                if is_code_active {
                    self.current_tab = AppWorkspaceTab::CodeEditor;
                } else if let Some(existing_code) = self
                    .open_tabs
                    .iter()
                    .find(|t| t.tab_kind == DocumentTabKind::CodeEditor)
                    .cloned()
                {
                    self.switch_document_tab(&existing_code.id);
                } else {
                    self.current_tab = AppWorkspaceTab::CodeEditor;
                }
            }
            other => {
                self.current_tab = other;
            }
        }
    }

    /// 根据虚拟路径或物理路径加载事件流或数据文件至编辑器
    pub fn load_file_by_path(&mut self, path: &str) {
        self.selected_file_path = Some(path.to_string());

        let raw_name = path
            .rsplit("//")
            .next()
            .unwrap_or(path)
            .rsplit('/')
            .next()
            .unwrap_or(path)
            .rsplit('\\')
            .next()
            .unwrap_or(path)
            .to_string();

        let lower_name = raw_name.to_lowercase();

        // 缓存当前旧活跃标签图
        if let Some(active_id) = &self.active_tab_id {
            if let Some(tab) = self.open_tabs.iter_mut().find(|t| &t.id == active_id) {
                if tab.tab_kind == DocumentTabKind::Flowchart {
                    tab.graph = Some(self.graph.clone());
                }
            }
        }

        // 如果该文件已存在打开标签页，直接切换
        if let Some(existing) = self.open_tabs.iter().find(|t| t.file_path == path) {
            let id = existing.id.clone();
            self.switch_document_tab(&id);
            return;
        }

        let bytes = match read_file_from_path(path) {
            Ok(b) => b,
            Err(e) => {
                self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), e)));
                return;
            }
        };

        // 1. 优先将 BFEVFL 打开为流程图模式
        if lower_name.ends_with(".bfevfl") || bytes.starts_with(b"BFEVFL") {
            match crate::model::evfl::load_bfevfl_from_bytes(&bytes) {
                Ok(mut loaded_graph) => {
                    loaded_graph.name = raw_name.clone();
                    loaded_graph.ensure_actors_populated();
                    crate::model::layout::auto_layout_graph(&mut loaded_graph);
                    self.graph = loaded_graph.clone();

                    let new_id = uuid::Uuid::new_v4().to_string();
                    let new_tab = OpenDocumentTab::new(
                        new_id.clone(),
                        raw_name.clone(),
                        path.to_string(),
                        "bfevfl".to_string(),
                        Some(loaded_graph.clone()),
                        Some(loaded_graph),
                    );
                    self.open_tabs.push(new_tab);
                    self.active_tab_id = Some(new_id);

                    self.fit_view();
                    self.current_tab = AppWorkspaceTab::Flowchart;
                    self.toast = Some(Toast::success(format!("{} {}", self.t("toast.file_loaded"), raw_name)));
                    return;
                }
                Err(err) => {
                    self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), err)));
                    return;
                }
            }
        }

        // 2. 通用代码编辑器解析：支持 MSBT, BYML, AAMP, JSON, YAML, TXT 等任意可打开文件
        match parse_file_to_code(&bytes, path) {
            Ok((code_text, format, endian)) => {
                let code_state = CodeDocumentState::new(
                    code_text,
                    format,
                    endian,
                    path.to_string(),
                );
                let new_id = uuid::Uuid::new_v4().to_string();
                let file_type = format.language_id().to_string();
                let new_tab = OpenDocumentTab::new_code(
                    new_id.clone(),
                    raw_name.clone(),
                    path.to_string(),
                    file_type,
                    code_state,
                );

                self.open_tabs.push(new_tab);
                self.active_tab_id = Some(new_id);
                self.current_tab = AppWorkspaceTab::CodeEditor;
                self.toast = Some(Toast::success(format!("{} {} [{}]", self.t("toast.file_loaded"), raw_name, format.display_name())));
            }
            Err(e) => {
                self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), e)));
            }
        }
    }

    /// 辅助添加最近项目项
    fn add_recent_project(&mut self, name: String, path: String, project_type: String) {
        self.recent_projects.retain(|p| p.path != path);
        self.recent_projects.insert(
            0,
            RecentProjectItem {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                path,
                project_type,
            },
        );
    }
}
