use crate::model::search_project_files;
use crate::ui::state::AppState;

/// 全局文件搜索与快速检索状态扩展
impl AppState {
    /// 触发全局搜索结果增量计算并缓存 (毫秒级响应，避免每帧渲染反复遍历)
    pub fn update_global_search_results(&mut self) {
        let query = self.global_search_query.trim();
        if query.is_empty() {
            self.global_search_results.clear();
            return;
        }

        self.global_search_results = search_project_files(
            &self.file_tree,
            query,
            self.global_search_case_sensitive,
            self.global_search_whole_word,
            self.global_search_use_regex,
            self.global_search_include.trim(),
            self.global_search_exclude.trim(),
            self.project_path.as_deref(),
        );
    }
}
