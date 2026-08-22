use crate::model::file_tree::{read_file_from_path, scan_project_path};
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

                // 插入最近打开记录
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

                // 展开封包内部顶层目录
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

        match read_file_from_path(path) {
            Ok(bytes) => {
                if lower_name.ends_with(".bfevfl") || bytes.starts_with(b"BFEVFL") {
                    match crate::model::evfl::load_bfevfl_from_bytes(&bytes) {
                        Ok(mut loaded_graph) => {
                            loaded_graph.name = raw_name.clone();
                            loaded_graph.ensure_actors_populated();
                            crate::model::layout::auto_layout_graph(&mut loaded_graph);
                            self.graph = loaded_graph;
                            self.fit_view();
                            self.current_tab = AppWorkspaceTab::Flowchart;
                            self.toast = Some(Toast::success(format!("{} {}", self.t("toast.file_loaded"), raw_name)));
                        }
                        Err(err) => {
                            self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), err)));
                        }
                    }
                } else if lower_name.ends_with(".json") {
                    if let Ok(json_str) = String::from_utf8(bytes) {
                        if let Ok(mut loaded_graph) = serde_json::from_str::<crate::model::Graph>(&json_str) {
                            loaded_graph.ensure_actors_populated();
                            crate::model::layout::auto_layout_graph(&mut loaded_graph);
                            self.graph = loaded_graph;
                            self.fit_view();
                            self.current_tab = AppWorkspaceTab::Flowchart;
                            self.toast = Some(Toast::success(format!("{} {}", self.t("toast.file_loaded"), raw_name)));
                        }
                    }
                } else {
                    self.toast = Some(Toast::info(format!("📂 已选中文件: {}", raw_name)));
                }
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
