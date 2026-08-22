use gpui::*;
use crate::ui::state::AppState;
use super::actions_col::render_actions_column;
use super::params_col::render_params_column;
use super::queries_col::render_queries_column;

/// 渲染下半区所选角色的三栏面板 (动作 / 查询 / 参数)
pub fn render_actor_detail_columns(
    state: &AppState,
    actor: &crate::model::actor::ActorData,
    actor_idx: usize,
    model: &Entity<AppState>,
) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        // 栏 1: 动作列表
        .child(render_actions_column(state, actor, actor_idx, model))
        // 栏 2: 查询列表
        .child(render_queries_column(state, actor, actor_idx, model))
        // 栏 3: 参数属性列表
        .child(render_params_column(state, actor, actor_idx, model))
}
