use crate::model::Graph;
use crate::ui::components::Toast;
use crate::ui::sidebar::SidebarTab;
pub use crate::ui::state_types::*;

pub struct AppState {
    pub graph: Graph,
    pub pan: (f32, f32),
    pub zoom: f32,
    pub selected_node_id: Option<String>,
    pub hovered_node_id: Option<String>,
    #[allow(dead_code)]
    pub hovered_port: Option<(String, String)>,
    pub drag_state: DragState,
    pub knife_mode: bool,
    #[allow(dead_code)]
    pub canvas_origin: (f32, f32),
    #[allow(dead_code)]
    pub fps: f32,
    pub active_sidebar_tab: SidebarTab,
    pub current_tab: AppWorkspaceTab,
    #[allow(dead_code)]
    pub welcome_tab: WelcomeTab,
    pub recent_projects: Vec<RecentProjectItem>,
    pub selected_actor_idx: usize,
    pub is_adding_action: bool,
    pub new_action_name: String,
    pub show_action_dropdown: bool,
    pub is_adding_query: bool,
    pub new_query_name: String,
    pub show_query_dropdown: bool,
    pub is_adding_actor: bool,
    pub new_actor_name: String,
    #[allow(dead_code)]
    pub new_actor_sub: String,
    pub show_actor_dropdown: bool,
    pub is_adding_param: bool,
    pub new_param_type: String,
    pub new_param_key: String,
    pub new_param_val: String,
    pub show_param_dropdown: bool,
    pub toast: Option<Toast>,

    pub quick_add_modal: Option<QuickAddModalState>,
    pub node_context_menu: Option<(String, (f32, f32))>,
    pub tab_context_menu: Option<(String, (f32, f32))>,
    pub file_tree_context_menu: Option<FileTreeContextMenu>,
    pub rename_modal: Option<RenameModalState>,
    pub delete_confirm_modal: Option<DeleteConfirmModalState>,
    pub active_node_editor_modal: Option<crate::ui::canvas::NodeEditorModalState>,
    pub unsaved_changes_modal: Option<UnsavedChangesModalState>,
    pub is_exiting_app: bool,
    pub should_quit_app: bool,

    pub inspector_open: bool,

    pub sidebar_open: bool,
    pub sidebar_width: f32,
    pub is_resizing_sidebar: bool,
    pub search_query: String,
    pub expand_all_params: bool,
    #[allow(dead_code)]
    pub frame_time_ms: f32,
    pub show_welcome_on_startup: bool,
    pub lang: crate::i18n::Language,
    pub game_lang: crate::model::GameLanguage,
    pub current_theme: crate::ui::theme::AppTheme,

    // 首选项设置模态弹窗状态与游戏路径配置
    pub game_paths: crate::model::GamePathSettings,
    pub settings_draft: crate::model::GamePathSettings,
    pub settings_modal_open: bool,
    pub settings_modal_width: f32,
    pub settings_modal_height: f32,
    pub settings_modal_maximized: bool,
    pub is_resizing_settings: bool,
    pub settings_resize_start: Option<((f32, f32), (f32, f32))>,
    pub settings_active_tab: crate::ui::state_settings::SettingsTab,
    pub active_settings_field: ActiveSettingsField,
    pub grid_style: crate::ui::state_settings::GridStyle,
    pub edge_curve_style: crate::ui::state_settings::EdgeCurveStyle,
    pub target_engine: crate::ui::state_settings::TargetEngine,
    pub auto_backup: bool,
    pub show_perf_stats: bool,

    // 资源管理器与项目目录/封包状态
    pub project_path: Option<String>,
    #[allow(dead_code)]
    pub ai_cache_manager: crate::model::ai_def::AiCacheManager,
    pub project_name: Option<String>,
    pub file_tree: Vec<crate::model::FileNode>,
    pub expanded_tree_paths: std::collections::HashSet<String>,
    pub selected_file_path: Option<String>,
    #[allow(dead_code)]
    pub file_search_query: String,
    #[allow(dead_code)]
    pub file_search_focused: bool,
    #[allow(dead_code)]
    pub file_search_case_sensitive: bool,
    pub file_tree_filter_type: FileFilterType,
    pub file_sort_mode: FileSortMode,
    pub file_tree_sort_menu_open: bool,
    pub file_tree_scroll_handle: gpui::UniformListScrollHandle,
    // 活动栏当前功能 (Activity Bar: 资源管理器, 搜索, 源文件管理)
    pub active_activity_tab: ActivityBarTab,
    pub global_search_query: String,
    pub global_search_focused: bool,
    pub global_search_case_sensitive: bool,
    pub global_search_whole_word: bool,
    pub global_search_use_regex: bool,
    pub global_search_include: String,
    pub global_search_exclude: String,
    pub global_search_show_filters: bool,
    pub global_search_results: Vec<crate::ui::sidebar::search_panel::SearchResultItem>,
    pub global_search_scroll_handle: gpui::UniformListScrollHandle,

    // 多文档标签页状态 (Document Tabs)
    pub open_tabs: Vec<crate::model::OpenDocumentTab>,
    pub active_tab_id: Option<String>,

    // Git 源代码管理状态
    pub git_is_repo: bool,
    pub git_branch: Option<String>,
    pub git_staged: std::sync::Arc<Vec<crate::model::git::GitFileItem>>,
    pub git_unstaged: std::sync::Arc<Vec<crate::model::git::GitFileItem>>,
    pub git_staged_collapsed: bool,
    pub git_unstaged_collapsed: bool,
    pub git_commit_title: String,
    pub git_commit_desc: String,
    pub git_commit_focus: GitCommitFocus,
    pub git_show_desc_field: bool,
    pub git_is_operating: bool,
    pub git_operating_files: std::collections::HashSet<String>,
    pub source_control_scroll_handle: gpui::UniformListScrollHandle,

    // Git 提交历史、分支图谱与修改差异状态
    pub git_commits: std::sync::Arc<Vec<crate::model::git::GitCommitItem>>,
    pub git_history_collapsed: bool,
    pub git_history_search_query: String,
    pub git_history_search_focused: bool,
    pub git_expanded_commit_hash: Option<String>,
    pub git_expanded_commit_files: Vec<crate::model::git::GitCommitFileChange>,
    pub git_history_limit: usize,
    pub active_diff_modal: Option<crate::ui::sidebar::diff_modal::GitDiffModalState>,

    // SARC 封包内部子文件展开与变更缓存
    pub git_expanded_sarc_keys: std::collections::HashSet<String>,
    pub git_sarc_subfiles_cache: std::collections::HashMap<String, Vec<crate::model::SarcSubFileChange>>,

    // Git 提交记录右键上下文菜单
    pub git_commit_context_menu: Option<(String, (f32, f32))>,

    // 插件系统管理状态
    pub plugin_manager: crate::model::plugin::PluginManager,
    pub plugin_search_query: String,
    pub plugin_search_focused: bool,
    #[allow(dead_code)]
    pub selected_plugin_id: Option<String>,
    pub api_server_running: bool,
    pub api_server_port: u16,

    // 任务管理器与向导状态
    pub quests_list: Vec<crate::model::quest::QuestDefinition>,
    pub quests_search_query: String,
    pub quests_search_focused: bool,
    pub quests_selected_id: Option<String>,
    pub quests_filter_type: Option<crate::model::quest::QuestType>,
    pub quest_creator_modal: Option<QuestCreatorModalState>,
    #[allow(dead_code)]
    pub quests_scroll_handle: gpui::UniformListScrollHandle,

    // 2D 任务地图交互与展示状态
    pub quest_map_pan: (f32, f32),
    pub quest_map_zoom: f32,
    pub quest_map_hovered_coord: Option<(f32, f32)>,
    pub quest_map_hovered_pin: Option<(String, usize, usize)>,
    pub quest_map_selected_pin: Option<(String, usize, usize)>,
    pub quest_map_is_dragging: bool,
    pub quest_map_drag_start: (f32, f32),
    pub quest_map_show_grid: bool,
    pub quest_map_show_all_pins: bool,
    pub quest_map_is_extracting_tiles: bool,

    // 海拉鲁游戏地标图层过滤与高亮
    pub quest_map_show_shrines: bool,
    pub quest_map_show_stables: bool,
    pub quest_map_show_beasts: bool,
    pub quest_map_show_towers: bool,
    pub quest_map_show_settlements: bool,
    pub quest_map_hovered_marker: Option<String>,
    pub quest_map_selected_marker: Option<String>,
    /// 内存中的一次性海拉鲁地标数据缓存 (从项目与游戏路径自动提取)
    pub quest_map_markers_cache: Option<Vec<crate::ui::views::quest_map::markers_data::HyruleMarker>>,
}

#[allow(dead_code)]
impl AppState {
    /// 确保海拉鲁地图标点已加载到内存缓存中（一次性提取）
    pub fn ensure_hyrule_markers_cached(&mut self) {
        if self.quest_map_markers_cache.is_none() {
            let proj_path = self.project_path.clone();
            let markers = crate::ui::views::quest_map::markers_extractor::extract_hyrule_markers(
                proj_path.as_deref(),
                Some(&self.game_paths),
            );
            self.quest_map_markers_cache = Some(markers);
        }
    }

    /// 获取内存中缓存的海拉鲁地标数据切片
    pub fn get_hyrule_markers(&self) -> &[crate::ui::views::quest_map::markers_data::HyruleMarker] {
        if let Some(cached) = &self.quest_map_markers_cache {
            cached.as_slice()
        } else {
            &[]
        }
    }

    /// 清空并失效地标缓存（当工作区切换项目或保存游戏路径时触发）
    pub fn invalidate_hyrule_markers_cache(&mut self) {
        self.quest_map_markers_cache = None;
    }

    pub fn set_sidebar_width(&mut self, width: f32) {
        self.sidebar_width = width.clamp(180.0, 750.0);
    }

    pub fn new_blank_flow(&mut self) {
        if let Some(cur_id) = &self.active_tab_id {
            if let Some(tab) = self.open_tabs.iter_mut().find(|t| &t.id == cur_id) {
                tab.graph = Some(self.graph.clone());
            }
        }

        let new_id = uuid::Uuid::new_v4().to_string();
        let tab_name = format!("Untitled_{}.bfevfl", self.open_tabs.len() + 1);
        let mut graph = Graph::new(&tab_name);
        graph.ensure_actors_populated();

        let new_tab = crate::model::OpenDocumentTab::new(
            new_id.clone(),
            tab_name.clone(),
            tab_name,
            "bfevfl".to_string(),
            Some(graph.clone()),
            Some(graph.clone()),
        );
        self.open_tabs.push(new_tab);

        self.active_tab_id = Some(new_id);
        self.graph = graph;
        self.fit_view();
        self.current_tab = AppWorkspaceTab::Flowchart;
        self.toast = Some(Toast::success(self.t("toast.new_flow_created")));
    }

    /// 获取当前语言环境下的本地化文本
    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        crate::i18n::t(key, self.lang)
    }

    pub fn toggle_expand_params(&mut self) {
        self.expand_all_params = !self.expand_all_params;
        self.update_node_sizes();
        crate::model::auto_layout_graph(&mut self.graph);
        self.fit_view();
        self.toast = Some(Toast::info(if self.expand_all_params { self.t("toast.expand_params_on") } else { self.t("toast.expand_params_off") }));
    }

    pub fn mark_current_tab_modified(&mut self) {
        if let Some(active_id) = &self.active_tab_id {
            if let Some(tab) = self.open_tabs.iter_mut().find(|t| &t.id == active_id) {
                if tab.tab_kind == crate::model::DocumentTabKind::Flowchart {
                    tab.graph = Some(self.graph.clone());
                }
                tab.recompute_modified();
            }
        }
    }

    pub fn is_file_modified(&self, path: &str) -> bool {
        self.open_tabs.iter().any(|t| t.file_path == path && t.is_modified)
    }

    /// 获取当前可用于 Git 操作的工作目录 (必须是打开的项目文件夹，封包或单文件不作为 Git 仓库)
    pub fn get_git_working_dir(&self) -> Option<String> {
        if let Some(p) = &self.project_path {
            let path = std::path::Path::new(p);
            if path.is_dir() {
                Some(p.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 关闭当前打开的 Mod 项目文件夹或封包，彻底清理项目资源与 Git 状态
    pub fn close_project(&mut self) {
        self.project_path = None;
        self.project_name = None;
        self.file_tree.clear();
        self.expanded_tree_paths.clear();
        self.selected_file_path = None;
        self.git_is_repo = false;
        self.git_branch = None;
        self.git_staged = std::sync::Arc::new(Vec::new());
        self.git_unstaged = std::sync::Arc::new(Vec::new());
        self.git_commits = std::sync::Arc::new(Vec::new());
        self.git_commit_title.clear();
        self.git_commit_desc.clear();
        self.git_commit_focus = GitCommitFocus::None;
        self.update_global_search_results();
        self.refresh_git_status();
        self.invalidate_hyrule_markers_cache();
        self.toast = Some(Toast::info(self.t("toast.project_closed")));
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_expand_params() {
        let mut state = AppState::new();
        assert!(!state.expand_all_params);
        state.toggle_expand_params();
        assert!(state.expand_all_params);
        state.toggle_expand_params();
        assert!(!state.expand_all_params);
    }
}