use gpui::*;
use crate::ui::sidebar::render_sidebar_resize_handle;
use crate::ui::state::AppState;

pub mod main_content;
pub mod quick_tab;
pub mod recent_tab;
pub mod sidebar_explorer;
pub mod walkthrough_col;

pub use main_content::render_welcome_main_content;
pub use sidebar_explorer::render_welcome_sidebar;

/// 渲染 VS Code 风格的完整启动欢迎界面 (左侧资源管理器 + 拖拽手柄 + 右侧启动/最近/演练区)
pub fn render_welcome_view(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    div()
        .id("welcome_view_root")
        .size_full()
        .bg(hsla(240.0 / 360.0, 0.06, 0.05, 1.0))
        .flex()
        // 左侧：资源管理器面板 (VS Code Explorer Sidebar)
        .child(render_welcome_sidebar(state, model))
        // 侧边栏宽度拖拽调整手柄
        .child(render_sidebar_resize_handle(state, model))
        // 右侧：欢迎主内容编辑区 (VS Code Welcome Editor Area)
        .child(render_welcome_main_content(state, model))
}


