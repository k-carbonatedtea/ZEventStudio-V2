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
}
