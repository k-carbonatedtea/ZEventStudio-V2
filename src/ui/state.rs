use crate::model::{Graph, PortDirection};
use crate::ui::components::Toast;
use crate::ui::sidebar::SidebarTab;


#[derive(Debug, Clone, PartialEq)]
pub enum DragState {
    None,
    PanCanvas {
        start_mouse: (f32, f32),
        start_pan: (f32, f32),
    },
    DragNodes {
        start_mouse: (f32, f32),
        initial_positions: Vec<(String, (f32, f32))>,
    },
    Connecting {
        source_node_id: String,
        source_port_id: String,
        direction: PortDirection,
        source_pos: (f32, f32),
        current_mouse: (f32, f32),
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppWorkspaceTab {
    #[default]
    Welcome,
    Flowchart,
    CodeEditor,
    Actors,
    Events,
    Json,
}


#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WelcomeTab {
    #[default]
    QuickStart,
    Recent,
}

pub use crate::model::RecentProjectItem;

/// 快速新建节点时的源连接端口信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickAddConnectionSource {
    pub node_id: String,
    pub port_id: String,
    pub direction: PortDirection,
}

/// 快速新建节点浮层的激活状态与位置
#[derive(Debug, Clone, PartialEq)]
pub struct QuickAddModalState {
    pub position: (f32, f32),
    pub source: Option<QuickAddConnectionSource>,
}

/// 未保存文件关闭确认弹窗状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsavedChangesModalState {
    pub tab_id: String,
    pub title: String,
    pub file_path: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveSettingsField {
    #[default]
    None,
    SwitchGameDir,
    SwitchDlcDir,
    SwitchExportDir,
    WiiuGameDir,
    WiiuUpdateDir,
    WiiuDlcDir,
    WiiuExportDir,
    AiDumpDir,
}

/// 文件资源管理器排序模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileSortMode {
    #[default]
    NameAsc,       // 名称升序 (A → Z)
    NameDesc,      // 名称降序 (Z → A)
    FileType,      // 按文件类型
    ModifiedFirst, // 已修改置顶
}

#[allow(dead_code)]
impl FileSortMode {
    pub fn label_key(&self) -> &'static str {
        match self {
            Self::NameAsc => "explorer.sort_name_asc",
            Self::NameDesc => "explorer.sort_name_desc",
            Self::FileType => "explorer.sort_type",
            Self::ModifiedFirst => "explorer.sort_modified",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Self::NameAsc => "A → Z",
            Self::NameDesc => "Z → A",
            Self::FileType => "类型",
            Self::ModifiedFirst => "修改优先",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::NameAsc => "🔤",
            Self::NameDesc => "🔤",
            Self::FileType => "🏷️",
            Self::ModifiedFirst => "📝",
        }
    }
}

/// 文件资源管理器类型筛选
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileFilterType {
    #[default]
    All,        // 全部文件
    EventFlow,  // 事件流 (.bfevfl, .evfl)
    Msbt,       // 文本对话 (.msbt, .smsbt)
    Param,      // 参数文件 (.byml, .bgyml, .aamp, ...)
    Sarc,       // 归档封包 (.sarc, .pack, ...)
    Modified,   // 仅已修改
}

#[allow(dead_code)]
impl FileFilterType {
    pub fn label_key(&self) -> &'static str {
        match self {
            Self::All => "explorer.filter_all",
            Self::EventFlow => "explorer.filter_event",
            Self::Msbt => "explorer.filter_msbt",
            Self::Param => "explorer.filter_param",
            Self::Sarc => "explorer.filter_sarc",
            Self::Modified => "explorer.filter_modified",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::All => "📁",
            Self::EventFlow => "⚡",
            Self::Msbt => "💬",
            Self::Param => "⚙️",
            Self::Sarc => "🗃️",
            Self::Modified => "●",
        }
    }
}

/// 活动栏功能面板枚举 (Activity Bar Tabs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivityBarTab {
    #[default]
    Explorer,      // 资源管理器
    Search,        // 全局搜索
    SourceControl, // 源文件管理
}

/// Git 提交输入框焦点
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitCommitFocus {
    #[default]
    None,
    Title,
    Description,
}

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
    // 游戏 AI 定义缓存 (用于自动填充事件参数)
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
    pub source_control_scroll_handle: gpui::UniformListScrollHandle,
}

#[allow(dead_code)]
impl AppState {
    pub fn new() -> Self {
        let mut graph = Graph::new("EventFlow");
        graph.ensure_actors_populated();
        let game_paths = crate::model::GamePathSettings::load_from_disk();
        let game_lang = game_paths.language;

        let mut state = Self {
            graph,
            pan: (100.0, 100.0),
            zoom: 1.0,
            selected_node_id: None,
            hovered_node_id: None,
            hovered_port: None,
            drag_state: DragState::None,
            knife_mode: false,
            canvas_origin: (0.0, 0.0),
            fps: 60.0,
            active_sidebar_tab: SidebarTab::EntryPoints,
            current_tab: AppWorkspaceTab::Welcome,
            welcome_tab: WelcomeTab::QuickStart,
            open_tabs: Vec::new(),
            active_tab_id: None,
            recent_projects: game_paths.recent_projects.clone(),
            selected_actor_idx: 0,
            is_adding_actor: false,
            new_actor_name: String::new(),
            new_actor_sub: String::new(),
            show_actor_dropdown: false,
            is_adding_action: false,
            new_action_name: String::new(),
            show_action_dropdown: false,
            is_adding_query: false,
            new_query_name: String::new(),
            show_query_dropdown: false,
            is_adding_param: false,
            new_param_type: "Int".to_string(),
            new_param_key: String::new(),
            new_param_val: "0".to_string(),
            show_param_dropdown: false,
            toast: None,
            quick_add_modal: None,
            node_context_menu: None,
            tab_context_menu: None,
            active_node_editor_modal: None,
            unsaved_changes_modal: None,
            is_exiting_app: false,
            should_quit_app: false,
            inspector_open: true,

            sidebar_open: true,
            sidebar_width: 290.0,
            is_resizing_sidebar: false,
            search_query: String::new(),
            expand_all_params: false,
            frame_time_ms: 1.0,
            show_welcome_on_startup: true,
            lang: crate::i18n::Language::Zh,
            game_lang,
            current_theme: crate::ui::theme::AppTheme::default(),
            game_paths: game_paths.clone(),
            settings_draft: game_paths,
            settings_modal_open: false,
            settings_modal_width: 860.0,
            settings_modal_height: 580.0,
            settings_modal_maximized: false,
            is_resizing_settings: false,
            settings_resize_start: None,
            settings_active_tab: crate::ui::state_settings::SettingsTab::General,
            active_settings_field: ActiveSettingsField::None,
            grid_style: crate::ui::state_settings::GridStyle::Dots,
            edge_curve_style: crate::ui::state_settings::EdgeCurveStyle::Bezier,
            target_engine: crate::ui::state_settings::TargetEngine::BotW,
            auto_backup: true,
            show_perf_stats: true,
            project_path: None,
            ai_cache_manager: Default::default(),
            project_name: None,
            file_tree: Vec::new(),
            expanded_tree_paths: std::collections::HashSet::new(),
            selected_file_path: None,
            file_search_query: String::new(),
            file_search_focused: false,
            file_search_case_sensitive: false,
            file_tree_filter_type: FileFilterType::All,
            file_sort_mode: FileSortMode::NameAsc,
            file_tree_sort_menu_open: false,
            file_tree_scroll_handle: gpui::UniformListScrollHandle::new(),
            active_activity_tab: ActivityBarTab::Explorer,
            global_search_query: String::new(),
            global_search_focused: false,
            global_search_case_sensitive: false,
            global_search_whole_word: false,
            global_search_use_regex: false,
            global_search_include: String::new(),
            global_search_exclude: String::new(),
            global_search_show_filters: false,
            global_search_results: Vec::new(),
            global_search_scroll_handle: gpui::UniformListScrollHandle::new(),
            git_is_repo: false,
            git_branch: None,
            git_staged: std::sync::Arc::new(Vec::new()),
            git_unstaged: std::sync::Arc::new(Vec::new()),
            git_staged_collapsed: false,
            git_unstaged_collapsed: false,
            git_commit_title: String::new(),
            git_commit_desc: String::new(),
            git_commit_focus: GitCommitFocus::None,
            git_show_desc_field: false,
            source_control_scroll_handle: gpui::UniformListScrollHandle::new(),
        };

        state.fit_view();
        state
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
        self.git_commit_title.clear();
        self.git_commit_desc.clear();
        self.git_commit_focus = GitCommitFocus::None;
        self.update_global_search_results();
        self.refresh_git_status();
        self.toast = Some(Toast::info(self.t("toast.project_closed")));
    }

    /// 刷新 Git 仓库状态
    pub fn refresh_git_status(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::get_repo_status(&dir) {
                Ok(status) => {
                    self.git_is_repo = status.is_repo;
                    self.git_branch = status.branch;
                    self.git_staged = status.staged;
                    self.git_unstaged = status.unstaged;
                }
                Err(_) => {
                    self.git_is_repo = false;
                    self.git_branch = None;
                    self.git_staged = std::sync::Arc::new(Vec::new());
                    self.git_unstaged = std::sync::Arc::new(Vec::new());
                }
            }
        } else {
            self.git_is_repo = false;
            self.git_branch = None;
            self.git_staged = std::sync::Arc::new(Vec::new());
            self.git_unstaged = std::sync::Arc::new(Vec::new());
        }
    }

    /// 初始化 Git 仓库 (git init)
    pub fn git_init_repository(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::init_repo(&dir) {
                Ok(()) => {
                    self.refresh_git_status();
                    self.toast = Some(Toast::success(format!("已成功为 {} 初始化 Git 仓库", dir)));
                }
                Err(err) => {
                    self.toast = Some(Toast::error(format!("初始化 Git 仓库失败: {}", err)));
                }
            }
        } else {
            self.toast = Some(Toast::warning("请先在资源管理器中打开 Mod 项目目录"));
        }
    }

    /// 暂存单个文件
    pub fn git_stage_file(&mut self, rel_path: &str) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::stage_file(&dir, rel_path) {
                Ok(()) => {
                    self.refresh_git_status();
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("暂存文件失败: {}", e)));
                }
            }
        }
    }

    /// 暂存所有变更
    pub fn git_stage_all(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::stage_all(&dir) {
                Ok(()) => {
                    self.refresh_git_status();
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("暂存全部失败: {}", e)));
                }
            }
        }
    }

    /// 取消暂存单个文件
    pub fn git_unstage_file(&mut self, rel_path: &str) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::unstage_file(&dir, rel_path) {
                Ok(()) => {
                    self.refresh_git_status();
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("取消暂存失败: {}", e)));
                }
            }
        }
    }

    /// 取消暂存所有变更
    pub fn git_unstage_all(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::unstage_all(&dir) {
                Ok(()) => {
                    self.refresh_git_status();
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("取消全部暂存失败: {}", e)));
                }
            }
        }
    }

    /// 放弃单个文件的修改
    pub fn git_discard_file(&mut self, rel_path: &str, is_untracked: bool) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::discard_file(&dir, rel_path, is_untracked) {
                Ok(()) => {
                    self.refresh_git_status();
                    self.toast = Some(Toast::info(format!("已放弃更改: {}", rel_path)));
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("放弃更改失败: {}", e)));
                }
            }
        }
    }

    /// 放弃所有修改
    pub fn git_discard_all(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::discard_all(&dir) {
                Ok(()) => {
                    self.refresh_git_status();
                    self.toast = Some(Toast::info("已放弃所有未暂存更改"));
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("放弃所有更改失败: {}", e)));
                }
            }
        }
    }

    /// 提交暂存的修改 (如果只有工作区未暂存修改，且用户点击提交，自动暂存所有并提交)
    pub fn git_commit(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            let title = self.git_commit_title.trim().to_string();
            if title.is_empty() {
                self.toast = Some(Toast::warning("请输入提交信息 (Commit Title)"));
                self.git_commit_focus = GitCommitFocus::Title;
                return;
            }

            // 若暂存区为空，但工作区有改动，则先自动暂存所有
            if self.git_staged.is_empty() && !self.git_unstaged.is_empty() {
                if let Err(e) = crate::model::git::stage_all(&dir) {
                    self.toast = Some(Toast::error(format!("自动暂存失败: {}", e)));
                    return;
                }
            }

            let desc = self.git_commit_desc.trim().to_string();
            match crate::model::git::commit(&dir, &title, &desc) {
                Ok(msg) => {
                    self.git_commit_title.clear();
                    self.git_commit_desc.clear();
                    self.git_commit_focus = GitCommitFocus::None;
                    self.git_show_desc_field = false;
                    self.refresh_git_status();
                    let summary = msg.lines().next().unwrap_or("提交成功").to_string();
                    self.toast = Some(Toast::success(format!("Git 提交成功: {}", summary)));
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("Git 提交失败: {}", e)));
                }
            }
        } else {
            self.toast = Some(Toast::warning("请先打开 Mod 项目工作区"));
        }
    }

    /// 检查当前是否有任何输入框处于激活/聚焦状态
    pub fn has_active_text_input(&self) -> bool {
        if self.settings_modal_open && self.active_settings_field != ActiveSettingsField::None {
            return true;
        }
        if let Some(ref modal) = self.active_node_editor_modal {
            if modal.active_field != crate::ui::canvas::node_editor::ActiveEditField::None {
                return true;
            }
        }
        if self.global_search_focused {
            return true;
        }
        if self.git_commit_focus != GitCommitFocus::None {
            return true;
        }
        if self.is_adding_action || self.is_adding_query || self.is_adding_actor || self.is_adding_param {
            return true;
        }
        false
    }

    /// 获取当前激活的输入框文本内容切片
    pub fn active_text_input(&self) -> Option<&str> {
        if self.settings_modal_open {
            match self.active_settings_field {
                ActiveSettingsField::SwitchGameDir => return Some(&self.settings_draft.switch.game_dir),
                ActiveSettingsField::SwitchDlcDir => return Some(&self.settings_draft.switch.dlc_dir),
                ActiveSettingsField::SwitchExportDir => return Some(&self.settings_draft.switch.export_dir),
                ActiveSettingsField::WiiuGameDir => return Some(&self.settings_draft.wiiu.game_dir),
                ActiveSettingsField::WiiuUpdateDir => return Some(&self.settings_draft.wiiu.update_dir),
                ActiveSettingsField::WiiuDlcDir => return Some(&self.settings_draft.wiiu.dlc_dir),
                ActiveSettingsField::WiiuExportDir => return Some(&self.settings_draft.wiiu.export_dir),
                ActiveSettingsField::AiDumpDir => return Some(&self.settings_draft.custom_ai_dump_dir),
                ActiveSettingsField::None => {}
            }
        }
        if let Some(ref modal) = self.active_node_editor_modal {
            use crate::ui::canvas::node_editor::ActiveEditField;
            match modal.active_field {
                ActiveEditField::FlowchartName => return Some(&modal.res_flowchart_name),
                ActiveEditField::EntryPointName => return Some(&modal.entry_point_name),
                ActiveEditField::BranchValue(idx) => return modal.branches.get(idx).map(|b| b.value.as_str()),
                ActiveEditField::BranchTarget(idx) => return modal.branches.get(idx).map(|b| b.target_id.as_str()),
                ActiveEditField::ParamKey(idx) => return modal.properties.get(idx).map(|p| p.key.as_str()),
                ActiveEditField::ParamVal(idx) => return modal.properties.get(idx).map(|p| p.val.as_str()),
                ActiveEditField::None => {}
            }
        }
        if self.global_search_focused {
            return Some(&self.global_search_query);
        }
        match self.git_commit_focus {
            GitCommitFocus::Title => return Some(&self.git_commit_title),
            GitCommitFocus::Description => return Some(&self.git_commit_desc),
            GitCommitFocus::None => {}
        }
        if self.is_adding_action {
            return Some(&self.new_action_name);
        }
        if self.is_adding_query {
            return Some(&self.new_query_name);
        }
        if self.is_adding_actor {
            return Some(&self.new_actor_name);
        }
        if self.is_adding_param {
            return Some(&self.new_param_key);
        }
        None
    }

    /// 向当前聚焦的输入框插入文本 (支持中文 IME 提交、剪贴板粘贴等)
    pub fn insert_text_to_active_input(&mut self, text: &str) -> bool {
        let clean: String = text.chars().filter(|&c| c != '\r' && c != '\x08' && c != '\x1b').collect();
        if clean.is_empty() {
            return false;
        }

        // 1. 设置弹窗输入框
        if self.settings_modal_open {
            match self.active_settings_field {
                ActiveSettingsField::SwitchGameDir => {
                    self.settings_draft.switch.game_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::SwitchDlcDir => {
                    self.settings_draft.switch.dlc_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::SwitchExportDir => {
                    self.settings_draft.switch.export_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::WiiuGameDir => {
                    self.settings_draft.wiiu.game_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::WiiuUpdateDir => {
                    self.settings_draft.wiiu.update_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::WiiuDlcDir => {
                    self.settings_draft.wiiu.dlc_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::WiiuExportDir => {
                    self.settings_draft.wiiu.export_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::AiDumpDir => {
                    self.settings_draft.custom_ai_dump_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::None => {}
            }
        }

        // 2. 节点编辑器模态框输入框
        if let Some(ref mut modal) = self.active_node_editor_modal {
            use crate::ui::canvas::node_editor::ActiveEditField;
            match modal.active_field {
                ActiveEditField::FlowchartName => {
                    modal.res_flowchart_name.push_str(&clean);
                    return true;
                }
                ActiveEditField::EntryPointName => {
                    modal.entry_point_name.push_str(&clean);
                    return true;
                }
                ActiveEditField::BranchValue(idx) => {
                    if let Some(b) = modal.branches.get_mut(idx) {
                        b.value.push_str(&clean);
                        return true;
                    }
                }
                ActiveEditField::BranchTarget(idx) => {
                    if let Some(b) = modal.branches.get_mut(idx) {
                        b.target_id.push_str(&clean);
                        return true;
                    }
                }
                ActiveEditField::ParamKey(idx) => {
                    if let Some(p) = modal.properties.get_mut(idx) {
                        p.key.push_str(&clean);
                        return true;
                    }
                }
                ActiveEditField::ParamVal(idx) => {
                    if let Some(p) = modal.properties.get_mut(idx) {
                        p.val.push_str(&clean);
                        return true;
                    }
                }
                ActiveEditField::None => {}
            }
        }

        // 3. 全局搜索输入框
        if self.global_search_focused {
            self.global_search_query.push_str(&clean);
            self.update_global_search_results();
            return true;
        }

        // 4. Git 提交输入框
        match self.git_commit_focus {
            GitCommitFocus::Title => {
                self.git_commit_title.push_str(&clean);
                return true;
            }
            GitCommitFocus::Description => {
                self.git_commit_desc.push_str(&clean);
                return true;
            }
            GitCommitFocus::None => {}
        }

        // 5. 侧边栏/角色管理器快速输入
        if self.is_adding_action {
            self.new_action_name.push_str(&clean);
            return true;
        }
        if self.is_adding_query {
            self.new_query_name.push_str(&clean);
            return true;
        }
        if self.is_adding_actor {
            self.new_actor_name.push_str(&clean);
            return true;
        }
        if self.is_adding_param {
            self.new_param_key.push_str(&clean);
            return true;
        }

        false
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

    #[test]
    fn test_chinese_input_global_search() {
        let mut state = AppState::new();
        assert!(!state.has_active_text_input());

        state.global_search_focused = true;
        assert!(state.has_active_text_input());
        assert_eq!(state.active_text_input(), Some(""));

        // 模拟 Windows IME 提交中文字符
        let inserted = state.insert_text_to_active_input("林克");
        assert!(inserted);
        assert_eq!(state.global_search_query, "林克");
        assert_eq!(state.active_text_input(), Some("林克"));

        let inserted2 = state.insert_text_to_active_input(" 测试搜索");
        assert!(inserted2);
        assert_eq!(state.global_search_query, "林克 测试搜索");
    }

    #[test]
    fn test_chinese_input_git_commit() {
        let mut state = AppState::new();
        state.git_commit_focus = GitCommitFocus::Title;
        assert!(state.has_active_text_input());

        let inserted = state.insert_text_to_active_input("feat: 支持中文输入法");
        assert!(inserted);
        assert_eq!(state.git_commit_title, "feat: 支持中文输入法");

        state.git_commit_focus = GitCommitFocus::Description;
        let inserted_desc = state.insert_text_to_active_input("修复全局搜索和提交描述无法输入中文的问题");
        assert!(inserted_desc);
        assert_eq!(state.git_commit_desc, "修复全局搜索和提交描述无法输入中文的问题");
    }

    #[test]
    fn test_chinese_input_actor_manager() {
        let mut state = AppState::new();
        state.is_adding_action = true;
        assert!(state.has_active_text_input());

        let inserted = state.insert_text_to_active_input("动作_播放动画");
        assert!(inserted);
        assert_eq!(state.new_action_name, "动作_播放动画");
    }
}


