//! 全局浮动模态框与上下文菜单图层集合渲染

use gpui::*;
use crate::ui::canvas::render_node_editor_modal;
use crate::ui::state::AppState;

/// 渲染主窗口全局浮层弹窗、右键菜单与确认模态框
pub fn render_app_modals_and_menus(
    state: &AppState,
    model: &Entity<AppState>,
) -> Vec<AnyElement> {
    let mut elements: Vec<AnyElement> = Vec::new();

    // 1. 标签页右键上下文菜单
    if let Some((tid, pos)) = state.tab_context_menu.as_ref() {
        elements.push(crate::ui::render_tab_context_menu(state, tid, *pos, model).into_any_element());
    }

    // 2. 文件树右键上下文菜单
    if let Some(menu) = state.file_tree_context_menu.as_ref() {
        elements.push(crate::ui::sidebar::render_file_tree_context_menu(state, menu, model).into_any_element());
    }

    // 3. 文件重命名模态框
    if let Some(modal) = state.rename_modal.as_ref() {
        elements.push(crate::ui::sidebar::render_rename_modal(state, modal, model).into_any_element());
    }

    // 4. 删除确认模态框
    if let Some(modal) = state.delete_confirm_modal.as_ref() {
        elements.push(crate::ui::sidebar::render_delete_confirm_modal(state, modal, model).into_any_element());
    }

    // 5. Git 提交记录右键上下文菜单
    if let Some((hash, pos)) = state.git_commit_context_menu.as_ref() {
        elements.push(crate::ui::sidebar::render_commit_context_menu(state, hash, *pos, model).into_any_element());
    }

    // 6. 节点属性与分支编辑模态框
    if state.active_node_editor_modal.is_some() {
        elements.push(render_node_editor_modal(state, model).into_any_element());
    }

    // 7. 未保存修改关闭确认模态框
    if state.unsaved_changes_modal.is_some() {
        elements.push(crate::ui::render_unsaved_changes_modal(state, model).into_any_element());
    }

    // 8. 首选项与系统设置模态框
    if state.settings_modal_open {
        elements.push(crate::ui::render_settings_modal(state, model).into_any_element());
    }

    // 9. Git 代码修改与差异对比模态框
    if state.active_diff_modal.is_some() {
        elements.push(crate::ui::sidebar::render_git_diff_modal(state, model).into_any_element());
    }

    // 10. 新建与编辑任务向导模态框
    if let Some(modal) = state.quest_creator_modal.as_ref() {
        elements.push(crate::ui::sidebar::render_quest_creator_modal(modal, state, model).into_any_element());
    }

    // 11. 任务原始数据与拓扑查看模态框
    if let Some(modal) = state.quest_raw_data_modal.as_ref() {
        elements.push(crate::ui::sidebar::render_quest_raw_data_modal(modal, state, model).into_any_element());
    }

    elements
}

