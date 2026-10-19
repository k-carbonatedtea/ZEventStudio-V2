use gpui::KeyDownEvent;
use crate::ui::canvas::node_editor::ActiveEditField;
use crate::ui::components::handle_text_key_event;
use crate::ui::state::{ActiveSettingsField, AppState};

/// 处理所有弹窗、模态对话框与上下文菜单相关的按键事件
pub fn handle_modal_key_event(event: &KeyDownEvent, state: &mut AppState, clipboard: Option<&str>) -> Option<bool> {
    // 1. 文件树右键菜单拦截 (Esc 关闭)
    if state.file_tree_context_menu.is_some() && event.keystroke.key == "escape" {
        state.file_tree_context_menu = None;
        return Some(true);
    }

    // 2. 标签页右键菜单拦截 (Esc 关闭)
    if state.tab_context_menu.is_some() && event.keystroke.key == "escape" {
        state.tab_context_menu = None;
        return Some(true);
    }

    // 3. Git 提交记录右键菜单拦截 (Esc 关闭)
    if state.git_commit_context_menu.is_some() && event.keystroke.key == "escape" {
        state.git_commit_context_menu = None;
        return Some(true);
    }

    // 4. 删除确认模态框拦截 (Esc 取消, Enter 确认删除)
    if state.delete_confirm_modal.is_some() {
        if event.keystroke.key == "escape" {
            state.delete_confirm_modal = None;
            return Some(true);
        }
        if event.keystroke.key == "enter" {
            state.execute_delete_item();
            return Some(true);
        }
        return Some(false);
    }

    // 5. 重命名模态框键盘输入拦截 (Esc 取消, Enter 提交, 文本键入)
    if let Some(ref mut modal) = state.rename_modal {
        if event.keystroke.key == "escape" {
            state.rename_modal = None;
            return Some(true);
        }
        if event.keystroke.key == "enter" {
            state.execute_rename_item();
            return Some(true);
        }
        let (changed, _, _) = handle_text_key_event(&mut modal.new_name, event, clipboard);
        return Some(changed);
    }

    // 6. 首选项设置弹窗拦截 (Esc 关闭)
    if state.settings_modal_open && event.keystroke.key == "escape" {
        state.settings_modal_open = false;
        return Some(true);
    }

    // 7. Git 差异对比模态框拦截 (Esc 关闭)
    if state.active_diff_modal.is_some() && event.keystroke.key == "escape" {
        state.close_diff_modal();
        return Some(true);
    }

    // 8. 未保存修改确认模态框键盘事件拦截 (Esc 取消, Enter 保存并关闭)
    if state.unsaved_changes_modal.is_some() {
        if event.keystroke.key == "escape" {
            state.cancel_unsaved_changes_modal();
            return Some(true);
        } else if event.keystroke.key == "enter" {
            let should_quit = state.confirm_save_and_close_tab();
            if should_quit {
                state.should_quit_app = true;
            }
            return Some(true);
        }
        return Some(false);
    }

    // 9. 首选项设置弹窗文本框输入拦截
    if state.settings_modal_open && state.active_settings_field != ActiveSettingsField::None {
        if event.keystroke.key == "escape" || event.keystroke.key == "enter" {
            state.active_settings_field = ActiveSettingsField::None;
            return Some(true);
        }

        let target = match state.active_settings_field {
            ActiveSettingsField::SwitchGameDir => &mut state.settings_draft.switch.game_dir,
            ActiveSettingsField::SwitchDlcDir => &mut state.settings_draft.switch.dlc_dir,
            ActiveSettingsField::SwitchExportDir => &mut state.settings_draft.switch.export_dir,
            ActiveSettingsField::WiiuGameDir => &mut state.settings_draft.wiiu.game_dir,
            ActiveSettingsField::WiiuUpdateDir => &mut state.settings_draft.wiiu.update_dir,
            ActiveSettingsField::WiiuDlcDir => &mut state.settings_draft.wiiu.dlc_dir,
            ActiveSettingsField::WiiuExportDir => &mut state.settings_draft.wiiu.export_dir,
            ActiveSettingsField::AiDumpDir => &mut state.settings_draft.custom_ai_dump_dir,
            ActiveSettingsField::None => return Some(false),
        };
        let (changed, _, _) = handle_text_key_event(target, event, clipboard);
        return Some(changed);
    }

    // 10. 节点属性与分支编辑模态框
    if let Some(ref mut modal) = state.active_node_editor_modal {
        let should_notify = match modal.active_field {
            ActiveEditField::FlowchartName => {
                let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.res_flowchart_name, event, clipboard);
                if is_esc || is_enter { modal.active_field = ActiveEditField::None; }
                changed || is_enter || is_esc
            }
            ActiveEditField::EntryPointName => {
                let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.entry_point_name, event, clipboard);
                if is_esc || is_enter { modal.active_field = ActiveEditField::None; }
                changed || is_enter || is_esc
            }
            ActiveEditField::BranchValue(idx) => {
                if idx < modal.branches.len() {
                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.branches[idx].value, event, clipboard);
                    if is_esc { modal.active_field = ActiveEditField::None; }
                    if is_enter { modal.active_field = ActiveEditField::BranchTarget(idx); }
                    changed || is_enter || is_esc
                } else { false }
            }
            ActiveEditField::BranchTarget(idx) => {
                if idx < modal.branches.len() {
                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.branches[idx].target_id, event, clipboard);
                    if is_esc || is_enter { modal.active_field = ActiveEditField::None; }
                    changed || is_enter || is_esc
                } else { false }
            }
            ActiveEditField::ParamKey(idx) => {
                if idx < modal.properties.len() {
                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.properties[idx].key, event, clipboard);
                    if is_esc { modal.active_field = ActiveEditField::None; }
                    if is_enter { modal.active_field = ActiveEditField::ParamVal(idx); }
                    changed || is_enter || is_esc
                } else { false }
            }
            ActiveEditField::ParamVal(idx) => {
                if idx < modal.properties.len() {
                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.properties[idx].val, event, clipboard);
                    if is_esc || is_enter { modal.active_field = ActiveEditField::None; }
                    changed || is_enter || is_esc
                } else { false }
            }
            ActiveEditField::None => {
                if event.keystroke.key == "escape" {
                    state.active_node_editor_modal = None;
                    true
                } else if event.keystroke.key == "enter" {
                    state.save_node_editor();
                    true
                } else { false }
            }
        };
        return Some(should_notify);
    }

    None
}
