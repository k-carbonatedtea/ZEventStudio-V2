use crate::ui::components::Toast;
use crate::ui::state::AppState;
use crate::ui::views::msbt_editor::parser::{parse_msyt_to_document, serialize_document_to_msyt};
use crate::ui::views::msbt_editor::types::MsbtEntry;

/// MSBT 可视化气泡编辑器数据同步与操作扩展
impl AppState {
    /// 将当前选中的 MSBT 条目原始富文本同步到编辑草稿中
    pub fn sync_msbt_selected_entry_to_draft(&mut self) {
        let active_id = match self.active_tab_id.as_deref() {
            Some(id) => id,
            None => return,
        };

        let raw_yaml = match self.open_tabs.iter().find(|t| t.id == active_id) {
            Some(tab) => match tab.code_state.as_ref() {
                Some(code) => code.content.clone(),
                None => return,
            },
            None => return,
        };

        if let Ok(doc) = parse_msyt_to_document(&raw_yaml) {
            if let Some(target_key) = self.msbt_selected_entry_key.as_deref() {
                if let Some(entry) = doc.entries.iter().find(|e| e.key == target_key) {
                    self.msbt_text_editor_draft = entry.raw_text.clone();
                    self.msbt_text_editor_cursor = self.msbt_text_editor_draft.chars().count();
                    self.msbt_text_editor_select_anchor = None;
                }
            } else if let Some(first_entry) = doc.entries.first() {
                self.msbt_selected_entry_key = Some(first_entry.key.clone());
                self.msbt_text_editor_draft = first_entry.raw_text.clone();
                self.msbt_text_editor_cursor = self.msbt_text_editor_draft.chars().count();
                self.msbt_text_editor_select_anchor = None;
            }
        }
    }

    /// 提交对白草稿修改并实时回写至当前激活标签页的 MSYT 文本与版本快照
    pub fn commit_msbt_draft_to_active_tab(&mut self) {
        let active_id = match self.active_tab_id.clone() {
            Some(id) => id,
            None => return,
        };

        let target_key = match self.msbt_selected_entry_key.clone() {
            Some(k) => k,
            None => return,
        };

        let draft = self.msbt_text_editor_draft.clone();

        if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == active_id) {
            if let Some(ref mut code) = tab.code_state {
                if let Ok(mut doc) = parse_msyt_to_document(&code.content) {
                    if let Some(entry) = doc.entries.iter_mut().find(|e| e.key == target_key) {
                        entry.raw_text = draft.clone();
                    }
                    if let Ok(new_yaml) = serialize_document_to_msyt(&doc) {
                        code.content = new_yaml;
                        code.version = code.version.wrapping_add(1);
                        tab.recompute_modified();
                    }
                }
            }
        }
    }

    /// 使用前缀与后缀标签包裹当前编辑文本或选中文本 (如 <colour:red>...</colour>)
    pub fn wrap_msbt_draft_with_tag(&mut self, prefix: &str, suffix: &str) {
        let (sel_start, sel_end) = match self.text_input_selection_range() {
            Some((s, e)) => (Some(s), Some(e)),
            None => (None, None),
        };

        if let (Some(start), Some(end)) = (sel_start, sel_end) {
            let chars: Vec<char> = self.msbt_text_editor_draft.chars().collect();
            let before: String = chars[..start].iter().collect();
            let selected: String = chars[start..end].iter().collect();
            let after: String = chars[end..].iter().collect();
            self.msbt_text_editor_draft = format!("{}{}{}{}{}", before, prefix, selected, suffix, after);
            self.msbt_text_editor_cursor = start + prefix.chars().count() + selected.chars().count() + suffix.chars().count();
            self.msbt_text_editor_select_anchor = None;
        } else {
            let cursor = self.msbt_text_editor_cursor;
            let chars: Vec<char> = self.msbt_text_editor_draft.chars().collect();
            let total = chars.len();
            let cur = cursor.min(total);
            let before: String = chars[..cur].iter().collect();
            let after: String = chars[cur..].iter().collect();
            self.msbt_text_editor_draft = format!("{}{}{}{}", before, prefix, suffix, after);
            self.msbt_text_editor_cursor = cur + prefix.chars().count();
            self.msbt_text_editor_select_anchor = None;
        }
        self.commit_msbt_draft_to_active_tab();
    }

    /// 向当前编辑文本光标处注入控制片段 (如 <pause:short>) 并替代选区
    pub fn insert_snippet_to_msbt_draft(&mut self, snippet: &str) {
        let (start, end) = self.text_input_selection_range().unwrap_or((self.msbt_text_editor_cursor, self.msbt_text_editor_cursor));
        let chars: Vec<char> = self.msbt_text_editor_draft.chars().collect();
        let total = chars.len();
        let s = start.min(total);
        let e = end.min(total);
        let before: String = chars[..s].iter().collect();
        let after: String = chars[e..].iter().collect();
        self.msbt_text_editor_draft = format!("{}{}{}", before, snippet, after);
        self.msbt_text_editor_cursor = s + snippet.chars().count();
        self.msbt_text_editor_select_anchor = None;
        self.commit_msbt_draft_to_active_tab();
    }

    /// 确认新建 MSBT 条目
    pub fn confirm_create_msbt_entry(&mut self) {
        let new_key = self.msbt_new_entry_name.trim().to_string();
        if new_key.is_empty() {
            self.toast = Some(Toast::error("条目键名不能为空"));
            return;
        }

        let active_id = match self.active_tab_id.clone() {
            Some(id) => id,
            None => return,
        };

        let mut success = false;
        if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == active_id) {
            if let Some(ref mut code) = tab.code_state {
                if let Ok(mut doc) = parse_msyt_to_document(&code.content) {
                    if doc.entries.iter().any(|e| e.key == new_key) {
                        self.toast = Some(Toast::error(format!("条目「{}」已存在", new_key)));
                        return;
                    }
                    doc.entries.push(MsbtEntry {
                        key: new_key.clone(),
                        raw_text: String::new(),
                        clean_text: String::new(),
                        spans: Vec::new(),
                        animation: "none".to_string(),
                        sound: "none".to_string(),
                        line_count: 1,
                        char_count: 0,
                    });
                    doc.group_count = doc.group_count.max(doc.entries.len() as u32);
                    if let Ok(new_yaml) = serialize_document_to_msyt(&doc) {
                        code.content = new_yaml;
                        code.version = code.version.wrapping_add(1);
                        tab.recompute_modified();
                        success = true;
                    }
                }
            }
        }

        if success {
            self.msbt_selected_entry_key = Some(new_key.clone());
            self.msbt_text_editor_draft.clear();
            self.msbt_text_editor_cursor = 0;
            self.msbt_text_editor_select_anchor = None;
            self.msbt_new_entry_modal_open = false;
            self.msbt_new_entry_name.clear();
            self.msbt_new_entry_focused = false;
            self.toast = Some(Toast::success(format!("已新建条目「{}」", new_key)));
        }
    }

    /// 删除指定 MSBT 消息条目
    pub fn delete_msbt_entry(&mut self, target_key: &str) {
        let active_id = match self.active_tab_id.clone() {
            Some(id) => id,
            None => return,
        };

        let mut next_key = None;
        let mut success = false;

        if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == active_id) {
            if let Some(ref mut code) = tab.code_state {
                if let Ok(mut doc) = parse_msyt_to_document(&code.content) {
                    if let Some(idx) = doc.entries.iter().position(|e| e.key == target_key) {
                        doc.entries.remove(idx);
                        if !doc.entries.is_empty() {
                            let new_idx = idx.min(doc.entries.len().saturating_sub(1));
                            next_key = Some(doc.entries[new_idx].key.clone());
                        }
                        if let Ok(new_yaml) = serialize_document_to_msyt(&doc) {
                            code.content = new_yaml;
                            code.version = code.version.wrapping_add(1);
                            tab.recompute_modified();
                            success = true;
                        }
                    }
                }
            }
        }

        if success {
            self.msbt_selected_entry_key = next_key;
            self.sync_msbt_selected_entry_to_draft();
            self.toast = Some(Toast::success(format!("已删除条目「{}」", target_key)));
        }
    }
}
