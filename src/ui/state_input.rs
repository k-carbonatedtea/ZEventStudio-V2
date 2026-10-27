use crate::ui::state::{ActiveSettingsField, AppState, GitCommitFocus, QuestCreatorFocusField};

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
        if self.global_search_focused || self.file_search_focused || self.git_history_search_focused || self.plugin_search_focused || self.quests_search_focused {
            return true;
        }
        if self.quest_creator_modal.as_ref().is_some_and(|m| m.focused_field != QuestCreatorFocusField::None) {
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
        if self.msbt_search_focused || self.msbt_new_entry_focused {
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
        self.quests_search_focused = false;
        if let Some(ref mut m) = self.quest_creator_modal {
            m.focused_field = QuestCreatorFocusField::None;
        }
        self.git_commit_focus = GitCommitFocus::None;
        self.is_adding_action = false;
        self.is_adding_query = false;
        self.is_adding_actor = false;
        self.is_adding_param = false;
        if let Some(ref mut m) = self.rename_modal {
            m.focused = false;
        }
        self.msbt_search_focused = false;
        self.msbt_text_editor_focused = false;
        self.msbt_new_entry_focused = false;
        self.text_input_cursor = 0;
        self.text_input_select_anchor = None;
        self.text_input_is_dragging = false;
        self.text_input_marked_range = None;
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
        if self.quests_search_focused {
            return Some(&self.quests_search_query);
        }
        if let Some(ref modal) = self.quest_creator_modal {
            if let Some(s) = modal.active_input() {
                return Some(s);
            }
        }
        if let Some(ref m) = self.rename_modal {
            if m.focused {
                return Some(&m.new_name);
            }
        }
        if self.msbt_search_focused {
            return Some(&self.msbt_search_query);
        }
        if self.msbt_new_entry_focused {
            return Some(&self.msbt_new_entry_name);
        }
        None
    }

    /// 获取当前激活输入框的字符级选区范围 [start, end)
    pub fn text_input_selection_range(&self) -> Option<(usize, usize)> {
        let anchor = self.text_input_select_anchor?;
        let cursor = self.text_input_cursor;
        if anchor == cursor {
            None
        } else {
            let total = self.active_text_input().map(|s| s.chars().count()).unwrap_or(0);
            let start = anchor.min(cursor).min(total);
            let end = anchor.max(cursor).min(total);
            if start < end {
                Some((start, end))
            } else {
                None
            }
        }
    }

    /// 获取当前激活输入框的选中文本切片 (仅当存在非空有效选区时返回 Some)
    pub fn active_selected_text(&self) -> Option<String> {
        let text = self.active_text_input()?;
        let (start, end) = self.text_input_selection_range()?;
        if start >= end {
            return None;
        }
        let chars: Vec<char> = text.chars().collect();
        let s = start.min(chars.len());
        let e = end.min(chars.len());
        if s < e {
            Some(chars[s..e].iter().collect())
        } else {
            None
        }
    }

    /// 删除当前选区中的文本并重置选区锚点 (若无选区则返回 false，严禁误清空全文)
    pub fn delete_active_selection(&mut self) -> bool {
        let (start, end) = match self.text_input_selection_range() {
            Some(r) => r,
            None => return false,
        };
        if start >= end {
            self.text_input_select_anchor = None;
            return false;
        }
        self.apply_to_active_input(|target| {
            let mut chars: Vec<char> = target.chars().collect();
            let total = chars.len();
            let s = start.min(total);
            let e = end.min(total);
            chars.drain(s..e);
            *target = chars.into_iter().collect();
            s
        })
    }

    /// 清空当前激活输入框的所有内容
    #[allow(dead_code)]
    pub fn clear_active_input_content(&mut self) -> bool {
        self.apply_to_active_input(|target| {
            target.clear();
            0
        })
    }

    /// 对当前激活的输入框字符串执行就地修改闭包，并自动同步 cursor 与 anchor
    pub fn apply_to_active_input<F: FnOnce(&mut String) -> usize>(&mut self, f: F) -> bool {
        // 1. 设置弹窗
        if self.settings_modal_open {
            let target = match self.active_settings_field {
                ActiveSettingsField::SwitchGameDir => Some(&mut self.settings_draft.switch.game_dir),
                ActiveSettingsField::SwitchDlcDir => Some(&mut self.settings_draft.switch.dlc_dir),
                ActiveSettingsField::SwitchExportDir => Some(&mut self.settings_draft.switch.export_dir),
                ActiveSettingsField::WiiuGameDir => Some(&mut self.settings_draft.wiiu.game_dir),
                ActiveSettingsField::WiiuUpdateDir => Some(&mut self.settings_draft.wiiu.update_dir),
                ActiveSettingsField::WiiuDlcDir => Some(&mut self.settings_draft.wiiu.dlc_dir),
                ActiveSettingsField::WiiuExportDir => Some(&mut self.settings_draft.wiiu.export_dir),
                ActiveSettingsField::AiDumpDir => Some(&mut self.settings_draft.custom_ai_dump_dir),
                ActiveSettingsField::None => None,
            };
            if let Some(target) = target {
                let new_cursor = f(target);
                self.text_input_cursor = new_cursor;
                self.text_input_select_anchor = None;
                return true;
            }
        }

        // 2. 节点编辑器
        if let Some(ref mut modal) = self.active_node_editor_modal {
            use crate::ui::canvas::node_editor::ActiveEditField;
            let target = match modal.active_field {
                ActiveEditField::FlowchartName => Some(&mut modal.res_flowchart_name),
                ActiveEditField::EntryPointName => Some(&mut modal.entry_point_name),
                ActiveEditField::BranchValue(idx) => modal.branches.get_mut(idx).map(|b| &mut b.value),
                ActiveEditField::BranchTarget(idx) => modal.branches.get_mut(idx).map(|b| &mut b.target_id),
                ActiveEditField::ParamKey(idx) => modal.properties.get_mut(idx).map(|p| &mut p.key),
                ActiveEditField::ParamVal(idx) => modal.properties.get_mut(idx).map(|p| &mut p.val),
                ActiveEditField::None => None,
            };
            if let Some(target) = target {
                let new_cursor = f(target);
                self.text_input_cursor = new_cursor;
                self.text_input_select_anchor = None;
                return true;
            }
        }

        // 3. 任务向导模态框
        if let Some(ref mut modal) = self.quest_creator_modal {
            if modal.focused_field != QuestCreatorFocusField::None {
                let field = modal.focused_field;
                if let Some(target) = modal.get_mut_focused() {
                    let new_cursor = f(target);
                    if field == QuestCreatorFocusField::Name {
                        modal.sync_flags_from_name();
                    } else if field == QuestCreatorFocusField::ReadyFlag
                        || field == QuestCreatorFocusField::ActivatedFlag
                        || field == QuestCreatorFocusField::FinishFlag
                    {
                        modal.custom_flags = true;
                    }
                    self.text_input_cursor = new_cursor;
                    self.text_input_select_anchor = None;
                    return true;
                }
            }
        }

        // 4. 常规输入框
        let new_cursor = if self.global_search_focused {
            let c = f(&mut self.global_search_query);
            self.update_global_search_results();
            c
        } else if self.file_search_focused {
            f(&mut self.file_search_query)
        } else if self.git_history_search_focused {
            f(&mut self.git_history_search_query)
        } else if self.plugin_search_focused {
            f(&mut self.plugin_search_query)
        } else if self.quests_search_focused {
            f(&mut self.quests_search_query)
        } else if self.msbt_search_focused {
            f(&mut self.msbt_search_query)
        } else if self.msbt_new_entry_focused {
            f(&mut self.msbt_new_entry_name)
        } else if self.rename_modal.as_ref().is_some_and(|m| m.focused) {
            if let Some(ref mut m) = self.rename_modal {
                f(&mut m.new_name)
            } else {
                return false;
            }
        } else if self.is_adding_action {
            f(&mut self.new_action_name)
        } else if self.is_adding_query {
            f(&mut self.new_query_name)
        } else if self.is_adding_actor {
            f(&mut self.new_actor_name)
        } else if self.is_adding_param {
            f(&mut self.new_param_key)
        } else {
            match self.git_commit_focus {
                GitCommitFocus::Title => f(&mut self.git_commit_title),
                GitCommitFocus::Description => f(&mut self.git_commit_desc),
                GitCommitFocus::None => return false,
            }
        };

        self.text_input_cursor = new_cursor;
        self.text_input_select_anchor = None;
        true
    }

    /// 向当前聚焦的输入框插入文本 (支持中文 IME 提交、剪贴板粘贴等)
    #[allow(dead_code)]
    pub fn insert_text_to_active_input(&mut self, text: &str) -> bool {
        let clean: String = text.chars().filter(|&c| c != '\r' && c != '\x08' && c != '\x1b').collect();
        if clean.is_empty() {
            return false;
        }

        // 若当前处于 IME 标记组合态，优先替换 marked_range
        if let Some(m) = self.text_input_marked_range.take() {
            self.replace_range_in_active_input(m, &clean);
            return true;
        }

        let cursor = self.text_input_cursor;
        let anchor = self.text_input_select_anchor;
        self.apply_to_active_input(|target| {
            let mut cur = cursor;
            let mut anc = anchor;
            crate::ui::components::splice_string_at(target, &mut cur, &mut anc, &clean);
            cur
        })
    }

    /// 替换当前激活输入框中指定字符区间的文本，并同步光标与选区
    pub fn replace_range_in_active_input(&mut self, char_range: (usize, usize), new_text: &str) -> (usize, usize) {
        let clean: String = new_text.chars().filter(|&c| c != '\r' && c != '\x08' && c != '\x1b').collect();
        let new_len = clean.chars().count();
        let start = char_range.0;
        let end = char_range.1;

        self.apply_to_active_input(|target| {
            let mut chars: Vec<char> = target.chars().collect();
            let total = chars.len();
            let s = start.min(total);
            let e = end.min(total).max(s);
            chars.drain(s..e);
            for (idx, ch) in clean.chars().enumerate() {
                chars.insert(s + idx, ch);
            }
            *target = chars.into_iter().collect();
            s + new_len
        });

        (start, start + new_len)
    }
}