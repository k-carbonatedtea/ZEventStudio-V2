pub mod clip_item;
pub mod inspector;
pub mod modals;
pub mod ruler;
pub mod toolbar;
pub mod tracks;

use gpui::*;
use crate::ui::state::AppState;
pub use toolbar::render_timeline_toolbar;
pub use ruler::render_timeline_ruler;
pub use tracks::render_timeline_tracks;
pub use inspector::render_timeline_inspector;
pub use modals::render_timeline_modals;

/// 渲染过场动画时间线编辑器全屏视图 (BFEVTM Timeline Editor View)
pub fn render_timeline_editor_view(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();

    div()
        .id("timeline-editor-view")
        .size_full()
        .flex()
        .flex_col()
        .bg(pal.app_bg)
        .overflow_hidden()
        // 1. 顶部回放与控制工具栏
        .child(render_timeline_toolbar(state, model))
        // 2. 主体工作区：时间轴轨道区域与右侧属性检查器
        .child(
            div()
                .flex_1()
                .w_full()
                .flex()
                .overflow_hidden()
                // 左侧多轨道时间线主体
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .child(render_timeline_ruler(state, model))
                        .child(render_timeline_tracks(state, model)),
                )
                // 右侧属性检查器面板
                .child(render_timeline_inspector(state, model)),
        )
        // 3. 浮动弹窗层 (新建角色 / 剪辑 / 分镜)
        .child(render_timeline_modals(state, model))
}
