use crate::ui::state::{AppState, GitCommitFocus};

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

    #[test]
    fn test_quest_creator_input_and_delete() {
        let mut state = AppState::new();
        state.open_quest_creator();
        assert!(state.has_active_text_input());
        assert_eq!(state.active_text_input(), Some("Quest_CustomMiniQuest"));
        assert_eq!(state.text_input_cursor, "Quest_CustomMiniQuest".chars().count());

        // 无选区时，active_selected_text 返回 None，delete_active_selection 不误删
        assert_eq!(state.active_selected_text(), None);
        assert!(!state.delete_active_selection());
        assert_eq!(state.active_text_input(), Some("Quest_CustomMiniQuest"));

        // 选定全文字符 [0, 22)
        state.text_input_select_anchor = Some(0);
        assert_eq!(state.active_selected_text(), Some("Quest_CustomMiniQuest".to_string()));

        // 删除选区
        assert!(state.delete_active_selection());
        assert_eq!(state.active_text_input(), Some(""));
        assert_eq!(state.text_input_cursor, 0);
        assert_eq!(state.active_selected_text(), None);

        // 测试输入新名称
        assert!(state.insert_text_to_active_input("MyCustomQuest"));
        assert_eq!(state.active_text_input(), Some("MyCustomQuest"));
        assert_eq!(state.text_input_cursor, "MyCustomQuest".chars().count());

        // 测试删除字符
        state.apply_to_active_input(|target| {
            target.pop();
            target.chars().count()
        });
        assert_eq!(state.active_text_input(), Some("MyCustomQues"));
        assert_eq!(state.text_input_cursor, "MyCustomQues".chars().count());
    }

    #[test]
    fn test_ime_preedit_composition_flow() {
        let mut state = AppState::new();
        state.global_search_focused = true;
        assert_eq!(state.active_text_input(), Some(""));

        // 1. 输入拼音 "w" (Preedit 组合态开始)
        let (s, e) = state.replace_range_in_active_input((0, 0), "w");
        state.text_input_marked_range = Some((s, e));
        assert_eq!(state.global_search_query, "w");
        assert_eq!(state.text_input_cursor, 1);
        assert_eq!(state.text_input_marked_range, Some((0, 1)));

        // 2. 追加拼音至 "wo" (更新组合态文本)
        let (s2, e2) = state.replace_range_in_active_input((0, 1), "wo");
        state.text_input_marked_range = Some((s2, e2));
        assert_eq!(state.global_search_query, "wo");
        assert_eq!(state.text_input_cursor, 2);
        assert_eq!(state.text_input_marked_range, Some((0, 2)));

        // 3. 用户确认候选词 "我" (Commit 组合态结束)
        state.replace_range_in_active_input((0, 2), "我");
        state.text_input_marked_range = None;
        assert_eq!(state.global_search_query, "我");
        assert_eq!(state.text_input_cursor, 1);
        assert_eq!(state.text_input_marked_range, None);
    }
}
