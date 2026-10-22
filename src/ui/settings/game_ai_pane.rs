use gpui::*;
use crate::ui::state::AppState;
use super::game_ai_helpers::{render_ai_autofill_info_card, render_ai_dump_dir_card, render_ai_status_card};

/// 渲染游戏 AI 定义与自动填充配置面板
pub fn render_game_ai_pane(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    div()
        .id("settings_game_ai_scroll")
        .flex_1()
        .flex()
        .flex_col()
        .gap_4()
        .overflow_y_scroll()
        .p_1()
        // 1. 本地 AI 定义文件夹 (Game AI Parameter Dump Dir)
        .child(render_ai_dump_dir_card(state, model))
        // 2. 游戏 AI 状态卡片 (Status Card)
        .child(render_ai_status_card(state, model))
        // 3. 自动填充机制说明卡片
        .child(render_ai_autofill_info_card(state))
}
