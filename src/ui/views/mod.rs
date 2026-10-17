pub mod actor_manager;
pub mod bottom_tabs;
pub mod code_editor;
pub mod diff_viewer;
pub mod event_manager;
pub mod json_viewer;
pub mod welcome;

pub use actor_manager::render_actor_manager_view;
pub use bottom_tabs::render_bottom_tabs;
pub use code_editor::render_code_editor_view;
pub use diff_viewer::render_diff_viewer_view;
pub use event_manager::render_event_manager_view;
pub use json_viewer::render_json_viewer_view;
pub use welcome::render_welcome_view;
