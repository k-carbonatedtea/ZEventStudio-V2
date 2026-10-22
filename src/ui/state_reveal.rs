use crate::ui::state::AppState;

/// 文件树高亮聚焦与平滑滚动定位扩展
impl AppState {
    /// 在左侧文件树中定位并高亮显示指定路径的文件/目录，并自动平滑滚动至视口中央
    pub fn reveal_file_in_tree(&mut self, target_path: &str, switch_to_explorer: bool) {
        if target_path.is_empty() {
            return;
        }

        let target_norm = target_path.replace('\\', "/").to_lowercase();

        // 1. 递归查找到该节点的真实 node.path，以及沿途所有祖先容器路径
        let mut ancestors = Vec::new();
        let mut matched_exact_path = None;
        let mut is_container = false;

        fn search_node(
            nodes: &[crate::model::FileNode],
            target_norm: &str,
            ancestors: &mut Vec<String>,
            matched_exact_path: &mut Option<String>,
            is_container: &mut bool,
        ) -> bool {
            for node in nodes {
                let node_norm = node.path.replace('\\', "/").to_lowercase();
                if node_norm == target_norm {
                    *matched_exact_path = Some(node.path.clone());
                    *is_container = node.is_dir || node.is_sarc;
                    return true;
                }
                if let Some(children) = &node.children {
                    ancestors.push(node.path.clone());
                    if search_node(children, target_norm, ancestors, matched_exact_path, is_container) {
                        return true;
                    }
                    ancestors.pop();
                }
            }
            false
        }

        search_node(
            &self.file_tree,
            &target_norm,
            &mut ancestors,
            &mut matched_exact_path,
            &mut is_container,
        );

        let final_path = match matched_exact_path {
            Some(p) => p,
            None => return, // 如果目标不在当前项目文件树中，不执行树展开与跳转
        };

        // 2. 确保所有祖先目录及 SARC 封包全部处于展开状态
        for anc in ancestors {
            self.expanded_tree_paths.insert(anc);
        }
        if is_container {
            self.expanded_tree_paths.insert(final_path.clone());
        }

        // 3. 选中该文件路径
        self.selected_file_path = Some(final_path.clone());

        // 4. 若请求切换至资源管理器，切换活动栏标签并展开侧边栏
        if switch_to_explorer {
            self.active_activity_tab = crate::ui::state::ActivityBarTab::Explorer;
            self.sidebar_open = true;
        }

        // 5. 清理文件树过滤干扰，确保目标文件能在列表中正常呈现
        self.file_search_query.clear();
        self.file_tree_filter_type = crate::ui::state::FileFilterType::All;

        // 6. 精确计算目标在虚拟列表中的扁平行索引，并触发平滑滚动到居中位置
        let mut flat_entries = Vec::new();
        crate::ui::components::file_tree::flatten_visible_nodes(
            &self.file_tree,
            &self.expanded_tree_paths,
            false,
            &std::collections::HashSet::new(),
            &final_path,
            self,
            0,
            &mut flat_entries,
        );

        if let Some(ix) = flat_entries.iter().position(|e| e.path == final_path) {
            self.file_tree_scroll_handle
                .scroll_to_item_strict(ix, gpui::ScrollStrategy::Center);
        }
    }
}
