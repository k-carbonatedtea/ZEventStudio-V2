pub mod modals;
pub mod sidebar;

use gpui::KeyDownEvent;
use crate::ui::components::handle_text_key_event;
use crate::ui::state::AppState;
use modals::handle_modal_key_event;
use sidebar::handle_sidebar_inputs;

/// 处理全局键盘事件与快捷键，返回是否需要触发重绘通知
pub fn handle_global_key_event(event: &KeyDownEvent, state: &mut AppState, clipboard: Option<&str>) -> bool {
    // 1. 快捷键: Ctrl+S 保存当前活跃文档 (流程图或代码文件)
    if event.keystroke.modifiers.control && event.keystroke.key == "s" {
        state.save_active_document();
        return true;
    }

    // 2. 快捷键: Ctrl+N 新建空白流程图
    if event.keystroke.modifiers.control && event.keystroke.key == "n" {
        state.new_blank_flow();
        return true;
    }

    // 3. 快捷键: Ctrl+W 关闭当前标签页
    if event.keystroke.modifiers.control && event.keystroke.key == "w" {
        if let Some(id) = state.active_tab_id.clone() {
            state.request_close_document_tab(&id);
            return true;
        }
    }

    // 4. 快捷键: Ctrl+Q 退出软件
    if event.keystroke.modifiers.control && event.keystroke.key == "q" {
        let allow_exit = state.request_exit_application();
        if allow_exit {
            state.should_quit_app = true;
        }
        return true;
    }

    // 4.1 快捷键: Ctrl+Shift+E 切换至资源管理器
    if event.keystroke.modifiers.control && event.keystroke.modifiers.shift && event.keystroke.key == "e" {
        state.active_activity_tab = crate::ui::state::ActivityBarTab::Explorer;
        state.sidebar_open = true;
        return true;
    }

    // 4.2 快捷键: Ctrl+Shift+F 切换至全局搜索并自动聚焦
    if event.keystroke.modifiers.control && event.keystroke.modifiers.shift && event.keystroke.key == "f" {
        state.active_activity_tab = crate::ui::state::ActivityBarTab::Search;
        state.sidebar_open = true;
        state.global_search_focused = true;
        return true;
    }

    // 4.3 快捷键: Ctrl+Shift+G 切换至源文件管理
    if event.keystroke.modifiers.control && event.keystroke.modifiers.shift && event.keystroke.key == "g" {
        state.active_activity_tab = crate::ui::state::ActivityBarTab::SourceControl;
        state.sidebar_open = true;
        state.refresh_git_status();
        state.sync_active_tab_modified();
        return true;
    }

    // 4.4 快捷键: Ctrl+F 在当前激活面板中唤起搜索 (资源管理器中搜索文件，搜索面板中自动聚焦)
    if event.keystroke.modifiers.control && event.keystroke.key == "f" {
        match state.active_activity_tab {
            crate::ui::state::ActivityBarTab::Explorer => {
                state.file_search_focused = true;
                state.sidebar_open = true;
                return true;
            }
            crate::ui::state::ActivityBarTab::Search => {
                state.global_search_focused = true;
                state.sidebar_open = true;
                return true;
            }
            crate::ui::state::ActivityBarTab::SourceControl => {
                state.git_history_search_focused = true;
                state.sidebar_open = true;
                return true;
            }
        }
    }

    // 4.5 文件树排序下拉菜单拦截 (Esc 关闭)
    if state.file_tree_sort_menu_open && event.keystroke.key == "escape" {
        state.file_tree_sort_menu_open = false;
        return true;
    }

    // 5. 模态框、弹窗与右键菜单拦截处理
    if let Some(handled) = handle_modal_key_event(event, state, clipboard) {
        return handled;
    }

    // 6. 资源管理器文件过滤搜索框输入拦截
    if state.file_search_focused {
        if event.keystroke.key == "escape" || event.keystroke.key == "enter" {
            state.file_search_focused = false;
            return true;
        }
        let (changed, _, _) = handle_text_key_event(&mut state.file_search_query, event, clipboard);
        return changed;
    }

    // 7. 侧边栏搜索与快速录入拦截处理
    if let Some(handled) = handle_sidebar_inputs(event, state, clipboard) {
        return handled;
    }

    // 8. 在源文件管理面板激活时按下 Ctrl+Enter 直接提交
    if state.active_activity_tab == crate::ui::state::ActivityBarTab::SourceControl
        && event.keystroke.modifiers.control
        && event.keystroke.key == "enter"
    {
        state.git_commit();
        return true;
    }

    false
}
