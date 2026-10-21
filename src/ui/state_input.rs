use crate::ui::state::{ActiveSettingsField, AppState, GitCommitFocus};

/// 文本输入框焦点调度与 IME 输入法支持状态扩展
impl AppState {
    pub fn has_active_text_input(&self) -> bool {
        if self.settings_modal_open && self.active_settings_field != ActiveSettingsField::None {
            return true;
        }
        if let Some(ref modal) = self.active_node_editor_modal {
            if modal.active_field != crate::ui::canvas::node_editor::ActiveEditField::None {
                return true;
            }
        }
        if self.global_search_focused || self.file_search_focused || self.git_history_search_focused || self.plugin_search_focused {
            return true;
        }
        if self.git_commit_focus != GitCommitFocus::None {
            return true;
        }
        if self.is_adding_action || self.is_adding_query || self.is_adding_actor || self.is_adding_param {
            return true;
        }
        if self.rename_modal.as_ref().is_some_and(|m| m.focused) {
            return true;
        }
        false
    }

    /// 清除所有侧边栏及编辑输入框的激活状态，使焦点回归主工作区/代码编辑器
    pub fn clear_active_text_inputs(&mut self) {
        self.global_search_focused = false;
        self.file_search_focused = false;
        self.git_history_search_focused = false;
        self.plugin_search_focused = false;
        self.git_commit_focus = GitCommitFocus::None;
        self.is_adding_action = false;
        self.is_adding_query = false;
        self.is_adding_actor = false;
        self.is_adding_param = false;
        if let Some(ref mut m) = self.rename_modal {
            m.focused = false;
        }
    }

    /// 获取当前激活的输入框文本内容切片
    pub fn active_text_input(&self) -> Option<&str> {
        if self.settings_modal_open {
            match self.active_settings_field {
                ActiveSettingsField::SwitchGameDir => return Some(&self.settings_draft.switch.game_dir),
                ActiveSettingsField::SwitchDlcDir => return Some(&self.settings_draft.switch.dlc_dir),
                ActiveSettingsField::SwitchExportDir => return Some(&self.settings_draft.switch.export_dir),
                ActiveSettingsField::WiiuGameDir => return Some(&self.settings_draft.wiiu.game_dir),
                ActiveSettingsField::WiiuUpdateDir => return Some(&self.settings_draft.wiiu.update_dir),
                ActiveSettingsField::WiiuDlcDir => return Some(&self.settings_draft.wiiu.dlc_dir),
                ActiveSettingsField::WiiuExportDir => return Some(&self.settings_draft.wiiu.export_dir),
                ActiveSettingsField::AiDumpDir => return Some(&self.settings_draft.custom_ai_dump_dir),
                ActiveSettingsField::None => {}
            }
        }
        if let Some(ref modal) = self.active_node_editor_modal {
            use crate::ui::canvas::node_editor::ActiveEditField;
            match modal.active_field {
                ActiveEditField::FlowchartName => return Some(&modal.res_flowchart_name),
                ActiveEditField::EntryPointName => return Some(&modal.entry_point_name),
                ActiveEditField::BranchValue(idx) => return modal.branches.get(idx).map(|b| b.value.as_str()),
                ActiveEditField::BranchTarget(idx) => return modal.branches.get(idx).map(|b| b.target_id.as_str()),
                ActiveEditField::ParamKey(idx) => return modal.properties.get(idx).map(|p| p.key.as_str()),
                ActiveEditField::ParamVal(idx) => return modal.properties.get(idx).map(|p| p.val.as_str()),
                ActiveEditField::None => {}
            }
        }
        if self.global_search_focused {
            return Some(&self.global_search_query);
        }
        if self.git_history_search_focused {
            return Some(&self.git_history_search_query);
        }
        if self.plugin_search_focused {
            return Some(&self.plugin_search_query);
        }
        match self.git_commit_focus {
            GitCommitFocus::Title => return Some(&self.git_commit_title),
            GitCommitFocus::Description => return Some(&self.git_commit_desc),
            GitCommitFocus::None => {}
        }
        if self.is_adding_action {
            return Some(&self.new_action_name);
        }
        if self.is_adding_query {
            return Some(&self.new_query_name);
        }
        if self.is_adding_actor {
            return Some(&self.new_actor_name);
        }
        if self.is_adding_param {
            return Some(&self.new_param_key);
        }
        if let Some(ref m) = self.rename_modal {
            if m.focused {
                return Some(&m.new_name);
            }
        }
        None
    }

    /// 向当前聚焦的输入框插入文本 (支持中文 IME 提交、剪贴板粘贴等)
    pub fn insert_text_to_active_input(&mut self, text: &str) -> bool {
        let clean: String = text.chars().filter(|&c| c != '\r' && c != '\x08' && c != '\x1b').collect();
        if clean.is_empty() {
            return false;
        }

        // 1. 设置弹窗输入框
        if self.settings_modal_open {
            match self.active_settings_field {
                ActiveSettingsField::SwitchGameDir => {
                    self.settings_draft.switch.game_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::SwitchDlcDir => {
                    self.settings_draft.switch.dlc_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::SwitchExportDir => {
                    self.settings_draft.switch.export_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::WiiuGameDir => {
                    self.settings_draft.wiiu.game_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::WiiuUpdateDir => {
                    self.settings_draft.wiiu.update_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::WiiuDlcDir => {
                    self.settings_draft.wiiu.dlc_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::WiiuExportDir => {
                    self.settings_draft.wiiu.export_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::AiDumpDir => {
                    self.settings_draft.custom_ai_dump_dir.push_str(&clean);
                    return true;
                }
                ActiveSettingsField::None => {}
            }
        }

        // 2. 节点编辑器模态框输入框
        if let Some(ref mut modal) = self.active_node_editor_modal {
            use crate::ui::canvas::node_editor::ActiveEditField;
            match modal.active_field {
                ActiveEditField::FlowchartName => {
                    modal.res_flowchart_name.push_str(&clean);
                    return true;
                }
                ActiveEditField::EntryPointName => {
                    modal.entry_point_name.push_str(&clean);
                    return true;
                }
                ActiveEditField::BranchValue(idx) => {
                    if let Some(b) = modal.branches.get_mut(idx) {
                        b.value.push_str(&clean);
                        return true;
                    }
                }
                ActiveEditField::BranchTarget(idx) => {
                    if let Some(b) = modal.branches.get_mut(idx) {
                        b.target_id.push_str(&clean);
                        return true;
                    }
                }
                ActiveEditField::ParamKey(idx) => {
                    if let Some(p) = modal.properties.get_mut(idx) {
                        p.key.push_str(&clean);
                        return true;
                    }
                }
                ActiveEditField::ParamVal(idx) => {
                    if let Some(p) = modal.properties.get_mut(idx) {
                        p.val.push_str(&clean);
                        return true;
                    }
                }
                ActiveEditField::None => {}
            }
        }

        // 3. 全局搜索输入框
        if self.global_search_focused {
            self.global_search_query.push_str(&clean);
            self.update_global_search_results();
            return true;
        }

        // 3.1 Git 历史提交搜索框
        if self.git_history_search_focused {
            self.git_history_search_query.push_str(&clean);
            return true;
        }

        // 3.2 插件搜索输入框
        if self.plugin_search_focused {
            self.plugin_search_query.push_str(&clean);
            return true;
        }

        // 4. Git 提交输入框
        match self.git_commit_focus {
            GitCommitFocus::Title => {
                self.git_commit_title.push_str(&clean);
                return true;
            }
            GitCommitFocus::Description => {
                self.git_commit_desc.push_str(&clean);
                return true;
            }
            GitCommitFocus::None => {}
        }

        // 5. 侧边栏/角色管理器快速输入
        if self.is_adding_action {
            self.new_action_name.push_str(&clean);
            return true;
        }
        if self.is_adding_query {
            self.new_query_name.push_str(&clean);
            return true;
        }
        if self.is_adding_actor {
            self.new_actor_name.push_str(&clean);
            return true;
        }
        if self.is_adding_param {
            self.new_param_key.push_str(&clean);
            return true;
        }

        // 6. 重命名模态框输入
        if let Some(ref mut m) = self.rename_modal {
            if m.focused {
                m.new_name.push_str(&clean);
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_input_global_search() {
        let mut state = AppState::new();
        assert!(!state.has_active_text_input());

        state.global_search_focused = true;
        assert!(state.has_active_text_input());
        assert_eq!(state.active_text_input(), Some(""));

        // 模拟 Windows IME 提交中文字符
        let inserted = state.insert_text_to_active_input("林克");
        assert!(inserted);
        assert_eq!(state.global_search_query, "林克");
        assert_eq!(state.active_text_input(), Some("林克"));

        let inserted2 = state.insert_text_to_active_input(" 测试搜索");
        assert!(inserted2);
        assert_eq!(state.global_search_query, "林克 测试搜索");
    }

    #[test]
    fn test_chinese_input_git_commit() {
        let mut state = AppState::new();
        state.git_commit_focus = GitCommitFocus::Title;
        assert!(state.has_active_text_input());

        let inserted = state.insert_text_to_active_input("feat: 支持中文输入法");
        assert!(inserted);
        assert_eq!(state.git_commit_title, "feat: 支持中文输入法");

        state.git_commit_focus = GitCommitFocus::Description;
        let inserted_desc = state.insert_text_to_active_input("修复全局搜索和提交描述无法输入中文的问题");
        assert!(inserted_desc);
        assert_eq!(state.git_commit_desc, "修复全局搜索和提交描述无法输入中文的问题");
    }

    #[test]
    fn test_chinese_input_actor_manager() {
        let mut state = AppState::new();
        state.is_adding_action = true;
        assert!(state.has_active_text_input());

        let inserted = state.insert_text_to_active_input("动作_播放动画");
        assert!(inserted);
        assert_eq!(state.new_action_name, "动作_播放动画");
    }

    #[test]
    fn test_clear_active_text_inputs() {
        let mut state = AppState::new();
        state.git_commit_focus = GitCommitFocus::Title;
        assert!(state.has_active_text_input());
        state.clear_active_text_inputs();
        assert!(!state.has_active_text_input());

        state.global_search_focused = true;
        assert!(state.has_active_text_input());
        state.clear_active_text_inputs();
        assert!(!state.has_active_text_input());

        state.file_search_focused = true;
        assert!(state.has_active_text_input());
        state.clear_active_text_inputs();
        assert!(!state.has_active_text_input());

        state.is_adding_param = true;
        assert!(state.has_active_text_input());
        state.clear_active_text_inputs();
        assert!(!state.has_active_text_input());
    }
}