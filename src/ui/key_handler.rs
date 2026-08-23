use gpui::KeyDownEvent;
use crate::ui::canvas::node_editor::ActiveEditField;
use crate::ui::components::handle_text_key_event;
use crate::ui::state::AppState;

/// 处理全局键盘事件与快捷键，返回是否需要触发重绘通知
pub fn handle_global_key_event(event: &KeyDownEvent, state: &mut AppState) -> bool {
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

    // 5. 标签页右键菜单拦截 (Esc 关闭)
    if state.tab_context_menu.is_some() && event.keystroke.key == "escape" {
        state.tab_context_menu = None;
        return true;
    }

    // 5.1 首选项设置弹窗拦截 (Esc 关闭)
    if state.settings_modal_open && event.keystroke.key == "escape" {
        state.settings_modal_open = false;
        return true;
    }

    // 6. 未保存修改确认模态框键盘事件拦截 (Esc 取消, Enter 保存并关闭)
    if state.unsaved_changes_modal.is_some() {
        if event.keystroke.key == "escape" {
            state.cancel_unsaved_changes_modal();
            return true;
        } else if event.keystroke.key == "enter" {
            let should_quit = state.confirm_save_and_close_tab();
            if should_quit {
                state.should_quit_app = true;
            }
            return true;
        }
        return false;
    }

    // 6.1. 首选项设置弹窗文本框输入拦截
    if state.settings_modal_open && state.active_settings_field != crate::ui::state::ActiveSettingsField::None {
        use crate::ui::state::ActiveSettingsField;
        let mut should_notify = false;
        
        // 当点击文本框之外的地方或按下 Escape / Enter 时取消焦点
        if event.keystroke.key == "escape" || event.keystroke.key == "enter" {
            state.active_settings_field = ActiveSettingsField::None;
            return true;
        }

        match state.active_settings_field {
            ActiveSettingsField::LlmBaseUrl => {
                let (changed, _, _) = handle_text_key_event(&mut state.settings_draft.llm.base_url, event);
                should_notify = changed;
            }
            ActiveSettingsField::LlmModelName => {
                let (changed, _, _) = handle_text_key_event(&mut state.settings_draft.llm.model_name, event);
                should_notify = changed;
            }
            ActiveSettingsField::LlmApiKey => {
                let (changed, _, _) = handle_text_key_event(&mut state.settings_draft.llm.api_key, event);
                should_notify = changed;
            }
            ActiveSettingsField::SwitchGameDir => {
                let (changed, _, _) = handle_text_key_event(&mut state.settings_draft.switch.game_dir, event);
                should_notify = changed;
            }
            ActiveSettingsField::SwitchDlcDir => {
                let (changed, _, _) = handle_text_key_event(&mut state.settings_draft.switch.dlc_dir, event);
                should_notify = changed;
            }
            ActiveSettingsField::SwitchExportDir => {
                let (changed, _, _) = handle_text_key_event(&mut state.settings_draft.switch.export_dir, event);
                should_notify = changed;
            }
            ActiveSettingsField::WiiuGameDir => {
                let (changed, _, _) = handle_text_key_event(&mut state.settings_draft.wiiu.game_dir, event);
                should_notify = changed;
            }
            ActiveSettingsField::WiiuUpdateDir => {
                let (changed, _, _) = handle_text_key_event(&mut state.settings_draft.wiiu.update_dir, event);
                should_notify = changed;
            }
            ActiveSettingsField::WiiuDlcDir => {
                let (changed, _, _) = handle_text_key_event(&mut state.settings_draft.wiiu.dlc_dir, event);
                should_notify = changed;
            }
            ActiveSettingsField::WiiuExportDir => {
                let (changed, _, _) = handle_text_key_event(&mut state.settings_draft.wiiu.export_dir, event);
                should_notify = changed;
            }
            ActiveSettingsField::AiDumpDir => {
                let (changed, _, _) = handle_text_key_event(&mut state.settings_draft.custom_ai_dump_dir, event);
                should_notify = changed;
            }
            ActiveSettingsField::None => {}
        }
        
        return should_notify;
    }

    // 6. 节点属性与分支编辑模态框键盘输入拦截
    if let Some(ref mut modal) = state.active_node_editor_modal {
        let mut should_notify = false;
        match modal.active_field {
            ActiveEditField::FlowchartName => {
                let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.res_flowchart_name, event);
                if is_esc { modal.active_field = ActiveEditField::None; }
                if is_enter { modal.active_field = ActiveEditField::EntryPointName; }
                should_notify = changed || is_enter || is_esc;
            }
            ActiveEditField::EntryPointName => {
                let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.entry_point_name, event);
                if is_esc || is_enter { modal.active_field = ActiveEditField::None; }
                should_notify = changed || is_enter || is_esc;
            }
            ActiveEditField::BranchValue(idx) => {
                if idx < modal.branches.len() {
                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.branches[idx].value, event);
                    if is_esc || is_enter { modal.active_field = ActiveEditField::None; }
                    should_notify = changed || is_enter || is_esc;
                }
            }
            ActiveEditField::BranchTarget(idx) => {
                if idx < modal.branches.len() {
                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.branches[idx].target_id, event);
                    if is_esc || is_enter { modal.active_field = ActiveEditField::None; }
                    should_notify = changed || is_enter || is_esc;
                }
            }
            ActiveEditField::ParamKey(idx) => {
                if idx < modal.properties.len() {
                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.properties[idx].key, event);
                    if is_esc { modal.active_field = ActiveEditField::None; }
                    if is_enter { modal.active_field = ActiveEditField::ParamVal(idx); }
                    should_notify = changed || is_enter || is_esc;
                }
            }
            ActiveEditField::ParamVal(idx) => {
                if idx < modal.properties.len() {
                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.properties[idx].val, event);
                    if is_esc || is_enter { modal.active_field = ActiveEditField::None; }
                    should_notify = changed || is_enter || is_esc;
                }
            }
            ActiveEditField::None => {
                if event.keystroke.key == "escape" {
                    state.active_node_editor_modal = None;
                    should_notify = true;
                } else if event.keystroke.key == "enter" {
                    state.save_node_editor();
                    should_notify = true;
                }
            }
        }
        return should_notify;
    }

    // 4. 角色与事件快速输入框键盘事件
    handle_sidebar_quick_input(event, state)
}

/// 侧边栏及管理器快速输入字段键盘事件响应
fn handle_sidebar_quick_input(event: &KeyDownEvent, state: &mut AppState) -> bool {
    if state.is_adding_action {
        let (changed, is_enter, is_esc) = handle_text_key_event(&mut state.new_action_name, event);
        if is_enter {
            let act_name = if state.new_action_name.trim().is_empty() {
                "Demo_WaitFrame".to_string()
            } else {
                state.new_action_name.trim().to_string()
            };
            if let Some(act) = state.graph.actors.get_mut(state.selected_actor_idx) {
                if !act.actions.contains(&act_name) {
                    act.actions.push(act_name);
                }
            }
            state.is_adding_action = false;
        } else if is_esc {
            state.is_adding_action = false;
        }
        return changed || is_enter || is_esc;
    }

    if state.is_adding_query {
        let (changed, is_enter, is_esc) = handle_text_key_event(&mut state.new_query_name, event);
        if is_enter {
            let qry_name = if state.new_query_name.trim().is_empty() {
                "CheckFlag".to_string()
            } else {
                state.new_query_name.trim().to_string()
            };
            if let Some(act) = state.graph.actors.get_mut(state.selected_actor_idx) {
                if !act.queries.contains(&qry_name) {
                    act.queries.push(qry_name);
                }
            }
            state.is_adding_query = false;
        } else if is_esc {
            state.is_adding_query = false;
        }
        return changed || is_enter || is_esc;
    }

    if state.is_adding_actor {
        let (changed, is_enter, is_esc) = handle_text_key_event(&mut state.new_actor_name, event);
        if is_enter {
            let name = if state.new_actor_name.trim().is_empty() {
                format!("Actor_{}", state.graph.actors.len())
            } else {
                state.new_actor_name.trim().to_string()
            };
            let mut new_a = crate::model::actor::ActorData::new(name);
            new_a.load_default_params();
            state.graph.actors.push(new_a);
            state.selected_actor_idx = state.graph.actors.len() - 1;
            state.is_adding_actor = false;
        } else if is_esc {
            state.is_adding_actor = false;
        }
        return changed || is_enter || is_esc;
    }

    if state.is_adding_param {
        let (changed, is_enter, is_esc) = handle_text_key_event(&mut state.new_param_key, event);
        if is_enter {
            if let Some(act) = state.graph.actors.get_mut(state.selected_actor_idx) {
                act.params.push(crate::model::node::PropertyItem {
                    key: if state.new_param_key.trim().is_empty() {
                        format!("Param_{}", act.params.len())
                    } else {
                        state.new_param_key.trim().to_string()
                    },
                    val: if state.new_param_val.trim().is_empty() {
                        "0".to_string()
                    } else {
                        state.new_param_val.trim().to_string()
                    },
                    val_type: state.new_param_type.clone(),
                });
            }
            state.is_adding_param = false;
        } else if is_esc {
            state.is_adding_param = false;
        }
        return changed || is_enter || is_esc;
    }

    false
}
