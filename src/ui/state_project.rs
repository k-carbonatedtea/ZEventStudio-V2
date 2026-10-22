use crate::model::code_document::CodeDocumentState;
use crate::model::file_converter::parse_file_to_code;
use crate::model::file_tree::{read_file_from_path, scan_project_path};
use crate::model::{DocumentTabKind, Graph, OpenDocumentTab};
use crate::ui::components::Toast;
use crate::ui::sidebar::SidebarTab;
use crate::ui::state::{AppState, AppWorkspaceTab};

impl AppState {
    /// 打开并解析指定 Mod 根目录 (如 content 或 romfs)
    pub fn open_project_directory(&mut self, path: &str) {
        // 彻底重置前一个项目的 Git 状态与提交草稿，确保项目间彻底独立隔离
        self.git_is_repo = false;
        self.git_branch = None;
        self.git_staged = std::sync::Arc::new(Vec::new());
        self.git_unstaged = std::sync::Arc::new(Vec::new());
        self.git_commits = std::sync::Arc::new(Vec::new());
        self.git_commit_title.clear();
        self.git_commit_desc.clear();
        self.git_commit_focus = crate::ui::state::GitCommitFocus::None;

        match scan_project_path(path) {
            Ok((name, nodes)) => {
                self.project_path = Some(path.to_string());
                self.project_name = Some(name.clone());
                self.file_tree = nodes;
                self.update_global_search_results();

                // 默认展开顶层前几个容器，避免深层递归展开导致的卡顿
                self.expanded_tree_paths.clear();
                let mut expand_count = 0;
                for node in &self.file_tree {
                    if (node.is_dir || node.is_sarc) && expand_count < 15 {
                        self.expanded_tree_paths.insert(node.path.clone());
                        expand_count += 1;
                    }
                }

                self.add_recent_project(name.clone(), path.to_string(), "mod".to_string());
                self.active_sidebar_tab = SidebarTab::Files;
                self.refresh_git_status();
                self.refresh_git_history();
                self.toast = Some(Toast::success(format!("{} {}", self.t("toast.project_loaded"), name)));
            }
            Err(e) => {
                self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), e)));
            }
        }
    }

    /// 打开并解析单个 SARC 封包文件 (如 .sbeventpack / .sbactorpack / .pack)
    pub fn open_pack_file(&mut self, path: &str) {
        // 打开独立封包文件时，彻底重置旧项目的 Git 状态 (封包不是 Git 仓库)
        self.git_is_repo = false;
        self.git_branch = None;
        self.git_staged = std::sync::Arc::new(Vec::new());
        self.git_unstaged = std::sync::Arc::new(Vec::new());
        self.git_commits = std::sync::Arc::new(Vec::new());
        self.git_commit_title.clear();
        self.git_commit_desc.clear();
        self.git_commit_focus = crate::ui::state::GitCommitFocus::None;

        match scan_project_path(path) {
            Ok((name, nodes)) => {
                self.project_path = Some(path.to_string());
                self.project_name = Some(name.clone());
                self.file_tree = nodes;
                self.update_global_search_results();

                self.expanded_tree_paths.clear();
                for node in &self.file_tree {
                    if node.is_dir || node.is_sarc {
                        self.expanded_tree_paths.insert(node.path.clone());
                    }
                }

                let ext = path.split('.').next_back().unwrap_or("pack").to_string();
                self.add_recent_project(name.clone(), path.to_string(), ext);
                self.active_sidebar_tab = SidebarTab::Files;
                self.refresh_git_status();
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
                self.update_global_search_results();
                self.refresh_git_status();
                self.refresh_git_history();
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

    /// 智能切换主工作区模式 (与已打开文档标签协同联动)
    pub fn switch_workspace_tab(&mut self, target_tab: AppWorkspaceTab) {
        self.clear_active_text_inputs();
        if self.current_tab == target_tab {
            return;
        }

        // 1. 若当前正处于代码编辑模式且即将切换到其他视图，先尝试将代码编辑器中的 JSON 同步回图拓扑
        if self.current_tab == AppWorkspaceTab::CodeEditor {
            self.sync_code_editor_to_graph_if_needed();
        }

        match target_tab {
            AppWorkspaceTab::Flowchart => {
                let is_flow_active = self.active_tab_id.as_deref()
                    .and_then(|id| self.open_tabs.iter().find(|t| t.id == id))
                    .is_some_and(|t| t.tab_kind == DocumentTabKind::Flowchart || t.file_type == "bfevfl");

                if is_flow_active {
                    self.current_tab = AppWorkspaceTab::Flowchart;
                } else if let Some(existing_flow) = self.open_tabs.iter().find(|t| t.tab_kind == DocumentTabKind::Flowchart).cloned() {
                    self.switch_document_tab(&existing_flow.id);
                } else {
                    self.graph = Graph::new("EventFlow");
                    self.current_tab = AppWorkspaceTab::Flowchart;
                }
            }
            AppWorkspaceTab::CodeEditor => {
                // 如果当前激活的标签页是事件流（拥有图或 BFEVFL 格式），确保加载原生 EventFlow 代码状态
                if let Some(active_id) = self.active_tab_id.clone() {
                    if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == active_id) {
                        if tab.tab_kind == DocumentTabKind::Flowchart || tab.file_type == "bfevfl" || tab.graph.is_some() {
                            if tab.code_state.is_none() {
                                let decompressed_res = crate::model::file_tree::read_file_from_path(&tab.file_path)
                                    .map(|b| crate::model::file_converter::decompress_if_yaz0(&b));
                                let code_text = if let Ok(decompressed) = decompressed_res {
                                    if let Ok(evfl) = crate::model::evfl::load_eventflow_from_bytes(&decompressed) {
                                        serde_json::to_string_pretty(&evfl).unwrap_or_default()
                                    } else {
                                        serde_json::to_string_pretty(&self.graph).unwrap_or_default()
                                    }
                                } else {
                                    serde_json::to_string_pretty(&self.graph).unwrap_or_default()
                                };

                                let code_state = crate::model::code_document::CodeDocumentState::new(
                                    code_text,
                                    crate::model::code_document::CodeFormat::Bfevfl,
                                    crate::model::code_document::CodeEndian::Little,
                                    tab.file_path.clone(),
                                );
                                tab.code_state = Some(code_state);
                            }
                            self.current_tab = AppWorkspaceTab::CodeEditor;
                            return;
                        }
                    }
                }

                let is_code_active = self.active_tab_id.as_deref()
                    .and_then(|id| self.open_tabs.iter().find(|t| t.id == id))
                    .is_some_and(|t| t.tab_kind == DocumentTabKind::CodeEditor);

                if is_code_active {
                    self.current_tab = AppWorkspaceTab::CodeEditor;
                } else if let Some(existing_code) = self.open_tabs.iter().find(|t| t.tab_kind == DocumentTabKind::CodeEditor).cloned() {
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
        self.clear_active_text_inputs();
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

        // 1. 优先将 BFEVFL 打开为流程图模式，并同步生成原生 EventFlow 代码状态
        if lower_name.ends_with(".bfevfl") || bytes.starts_with(b"BFEVFL") {
            let decompressed = crate::model::file_converter::decompress_if_yaz0(&bytes);
            let evfl_res = crate::model::evfl::load_eventflow_from_bytes(&decompressed);
            let graph_res = crate::model::evfl::load_bfevfl_from_bytes(&decompressed);

            match graph_res {
                Ok(mut loaded_graph) => {
                    loaded_graph.name = raw_name.clone();
                    loaded_graph.ensure_actors_populated();
                    crate::model::layout::auto_layout_graph(&mut loaded_graph);
                    self.graph = loaded_graph.clone();

                    let code_state_opt = evfl_res.ok().map(|evfl| {
                        let json_text = serde_json::to_string_pretty(&evfl).unwrap_or_default();
                        crate::model::code_document::CodeDocumentState::new(
                            json_text,
                            crate::model::code_document::CodeFormat::Bfevfl,
                            crate::model::code_document::CodeEndian::Little,
                            path.to_string(),
                        )
                    });

                    let new_id = uuid::Uuid::new_v4().to_string();
                    let mut new_tab = OpenDocumentTab::new(
                        new_id.clone(),
                        raw_name.clone(),
                        path.to_string(),
                        "bfevfl".to_string(),
                        Some(loaded_graph.clone()),
                        Some(loaded_graph),
                    );
                    new_tab.code_state = code_state_opt;
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

        // 2. 通用代码编辑器解析：支持 MSBT, BYML, AAMP, BAIPROG, JSON, YAML, TXT 等任意可打开文件
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
                    file_type.clone(),
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
}
