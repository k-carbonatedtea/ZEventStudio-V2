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
    LlmBaseUrl,
    LlmModelName,
    LlmApiKey,
    SwitchGameDir,
    SwitchDlcDir,
    SwitchExportDir,
    WiiuGameDir,
    WiiuUpdateDir,
    WiiuDlcDir,
    WiiuExportDir,
    AiDumpDir,
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
    pub performance_mode: bool,
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
    pub file_search_query: String,

    // 多文档标签页状态 (Document Tabs)
    pub open_tabs: Vec<crate::model::OpenDocumentTab>,
    pub active_tab_id: Option<String>,
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
            performance_mode: false,
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

    pub fn toggle_performance_mode(&mut self) {
        self.performance_mode = !self.performance_mode;
        if self.performance_mode {
            self.toast = Some(Toast::success(self.t("toast.perf_on")));
        } else {
            self.toast = Some(Toast::info(self.t("toast.perf_off")));
        }
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


