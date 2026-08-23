pub mod app;
pub mod canvas;
pub mod components;
pub mod inspector;
pub mod key_handler;
pub mod quick_add;
pub mod sidebar;
pub mod state;
pub mod state_editor;
pub mod state_project;
pub mod state_theme;
pub mod tab_bar;
pub mod theme;
pub mod title_bar;
pub mod toolbar;
pub mod views;

pub use app::AppView;
pub use tab_bar::{render_document_tab_bar, render_unsaved_changes_modal};
#[allow(unused_imports)]
pub use theme::{AppTheme, ThemePalette};
pub use title_bar::render_title_bar;
