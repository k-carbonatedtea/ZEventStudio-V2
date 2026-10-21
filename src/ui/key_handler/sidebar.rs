use gpui::KeyDownEvent;
use crate::ui::components::handle_text_key_event;
use crate::ui::state::{AppState, GitCommitFocus};

/// 处理侧边栏搜索框与快速录入字段键盘事件
pub fn handle_sidebar_inputs(event: &KeyDownEvent, state: &mut AppState, clipboard: Option<&str>) -> Option<bool> {
    // 1. 全局搜索输入框键盘输入拦截
    if state.global_search_focused {
        if event.keystroke.key == "escape" || event.keystroke.key == "enter" {
            state.global_search_focused = false;
            return Some(true);
        }
        let (changed, _, _) = handle_text_key_event(&mut state.global_search_query, event, clipboard);
        if changed {
            state.update_global_search_results();
        }
        return Some(changed);
    }

    // 2. Git 历史提交搜索框键盘输入拦截
    if state.git_history_search_focused {
        if event.keystroke.key == "escape" || event.keystroke.key == "enter" {
            state.git_history_search_focused = false;
            return Some(true);
        }
        let (changed, _, _) = handle_text_key_event(&mut state.git_history_search_query, event, clipboard);
        return Some(changed);
    }

    // 2.1 插件列表搜索框键盘输入拦截
    if state.plugin_search_focused {
        if event.keystroke.key == "escape" || event.keystroke.key == "enter" {
            state.plugin_search_focused = false;
            return Some(true);
        }
        let (changed, _, _) = handle_text_key_event(&mut state.plugin_search_query, event, clipboard);
        return Some(changed);
    }

    // 3. Git 源代码管理提交输入框键盘输入拦截
    match state.git_commit_focus {
        GitCommitFocus::Title => {
            if (event.keystroke.modifiers.control && event.keystroke.key == "enter")
                || (!event.keystroke.modifiers.shift && !event.keystroke.modifiers.control && event.keystroke.key == "enter")
            {
                state.git_commit();
                return Some(true);
            }
            if event.keystroke.key == "escape" {
                state.git_commit_focus = GitCommitFocus::None;
                return Some(true);
            }
            if event.keystroke.key == "tab" {
                state.git_show_desc_field = true;
                state.git_commit_focus = GitCommitFocus::Description;
                return Some(true);
            }
            let (changed, _, _) = handle_text_key_event(&mut state.git_commit_title, event, clipboard);
            return Some(changed);
        }
        GitCommitFocus::Description => {
            if event.keystroke.modifiers.control && event.keystroke.key == "enter" {
                state.git_commit();
                return Some(true);
            }
            if event.keystroke.key == "escape" {
                state.git_commit_focus = GitCommitFocus::None;
                return Some(true);
            }
            if event.keystroke.key == "tab" {
                state.git_commit_focus = GitCommitFocus::Title;
                return Some(true);
            }
            let (changed, is_enter, _) = handle_text_key_event(&mut state.git_commit_desc, event, clipboard);
            if is_enter {
                state.git_commit_desc.push('\n');
                return Some(true);
            }
            return Some(changed);
        }
        GitCommitFocus::None => {}
    }

    // 4. 角色与动作/查询快速输入
    if state.is_adding_action {
        let (changed, is_enter, is_esc) = handle_text_key_event(&mut state.new_action_name, event, clipboard);
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
        return Some(changed || is_enter || is_esc);
    }

    if state.is_adding_query {
        let (changed, is_enter, is_esc) = handle_text_key_event(&mut state.new_query_name, event, clipboard);
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
        return Some(changed || is_enter || is_esc);
    }

    if state.is_adding_actor {
        let (changed, is_enter, is_esc) = handle_text_key_event(&mut state.new_actor_name, event, clipboard);
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
        return Some(changed || is_enter || is_esc);
    }

    if state.is_adding_param {
        let (changed, is_enter, is_esc) = handle_text_key_event(&mut state.new_param_key, event, clipboard);
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
        return Some(changed || is_enter || is_esc);
    }

    None
}
