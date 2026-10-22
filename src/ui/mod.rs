pub mod app;
pub mod app_layout;
pub mod canvas;
pub mod components;
pub mod inspector;
pub mod key_handler;
pub mod quick_add;
pub mod settings;
pub mod sidebar;
pub mod state;
pub mod state_canvas;
pub mod state_diff;
pub mod state_editor;
pub mod state_file_ops;
pub mod state_node_editor;
pub mod state_git;
pub mod state_git_sync;
pub mod state_init;
pub mod state_input;
pub mod plugin_ops;
pub mod state_plugin_bridge;
pub mod state_plugins;
pub mod state_project;
pub mod state_recent;
pub mod state_reveal;
pub mod state_save;
pub mod state_search;
pub mod state_settings;
pub mod state_tabs;
pub mod state_theme;
pub mod state_types;
pub mod tab_bar;
pub mod theme;
pub mod title_bar;
pub mod toolbar;
pub mod views;

pub use app::AppView;
pub use settings::render_settings_modal;
#[allow(unused_imports)]
pub use state_types::*;
#[allow(unused_imports)]
pub use state_settings::{EdgeCurveStyle, GridStyle, SettingsTab, TargetEngine};
pub use tab_bar::{render_document_tab_bar, render_tab_context_menu, render_unsaved_changes_modal};
#[allow(unused_imports)]
pub use theme::{AppTheme, ThemePalette};
pub use title_bar::render_title_bar;
