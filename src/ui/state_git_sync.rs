use crate::model::DocumentTabKind;
use crate::ui::state::AppState;

/// Git 变更同步与标签页多源状态计算扩展
impl AppState {
    /// 计算所有发生变更的文件总数 (合并 Git 已暂存、Git 未暂存以及内存中未保存的已打开文档)
    pub fn total_changed_files_count(&self) -> usize {
        if !self.git_is_repo {
            return self.modified_tabs_count();
        }

        let mut changed_paths = std::collections::HashSet::new();
        for item in self.git_staged.iter() {
            changed_paths.insert(item.rel_path.replace('\\', "/"));
        }
        for item in self.git_unstaged.iter() {
            changed_paths.insert(item.rel_path.replace('\\', "/"));
        }

        let project_dir = self.get_git_working_dir();
        for tab in self.open_tabs.iter() {
            if tab.is_modified && tab.tab_kind != DocumentTabKind::DiffViewer {
                let norm_path = tab.file_path.replace('\\', "/");
                let rel_path = if let Some(ref root) = project_dir {
                    let norm_root = root.replace('\\', "/").trim_end_matches('/').to_string();
                    if let Some(stripped) = norm_path.strip_prefix(&norm_root) {
                        stripped.trim_start_matches('/').to_string()
                    } else {
                        norm_path
                    }
                } else {
                    norm_path
                };
                changed_paths.insert(rel_path);
            }
        }

        changed_paths.len()
    }

    /// 处理 Git 放弃修改后的标签页同步：关闭对应的 Diff 标签页，并重新从磁盘加载常规文档
    pub fn handle_git_discarded_file(&mut self, rel_path: &str, is_untracked: bool) {
        let norm_rel = rel_path.replace('\\', "/");
        let project_dir = self.get_git_working_dir();

        // 1. 关闭关联的 Working Tree 差异对比标签页 (因为工作区修改已放弃，diff 不复存在)
        let diff_tabs_to_close: Vec<String> = self
            .open_tabs
            .iter()
            .filter(|t| t.tab_kind == DocumentTabKind::DiffViewer)
            .filter(|t| {
                t.id.starts_with("diff:working:")
                    && (t.file_path.replace('\\', "/").ends_with(&norm_rel)
                        || t.file_path.replace('\\', "/") == norm_rel)
            })
            .map(|t| t.id.clone())
            .collect();

        for dt_id in diff_tabs_to_close {
            self.close_document_tab(&dt_id);
        }

        // 2. 查找关联的代码编辑器或流程图标签页并从磁盘重新加载
        let tabs_to_process: Vec<(String, String)> = self
            .open_tabs
            .iter()
            .filter(|t| t.tab_kind != DocumentTabKind::DiffViewer)
            .filter(|t| {
                let norm_tab = t.file_path.replace('\\', "/");
                if norm_tab == norm_rel {
                    return true;
                }
                if let Some(ref d) = project_dir {
                    let full = std::path::Path::new(d)
                        .join(rel_path)
                        .to_string_lossy()
                        .replace('\\', "/");
                    if norm_tab.eq_ignore_ascii_case(&full) {
                        return true;
                    }
                }
                norm_tab.ends_with(&format!("/{}", norm_rel))
            })
            .map(|t| (t.id.clone(), t.file_path.clone()))
            .collect();

        for (tid, fpath) in tabs_to_process {
            let is_missing = !std::path::Path::new(&fpath).exists();
            if is_untracked {
                self.close_document_tab(&tid);
            } else {
                let reload_failed = self.reload_tab_from_disk(&tid).is_err();
                if reload_failed && is_missing {
                    self.close_document_tab(&tid);
                }
            }
        }
    }

    /// 处理 Git 放弃全部未暂存修改后的标签页同步
    pub fn handle_git_discard_all(&mut self) {
        // 1. 关闭所有工作区差异对比标签页
        let diff_tabs_to_close: Vec<String> = self
            .open_tabs
            .iter()
            .filter(|t| t.tab_kind == DocumentTabKind::DiffViewer && t.id.starts_with("diff:working:"))
            .map(|t| t.id.clone())
            .collect();

        for dt_id in diff_tabs_to_close {
            self.close_document_tab(&dt_id);
        }

        // 2. 从磁盘重新加载所有常规代码与流程图标签页
        let tabs_to_reload: Vec<String> = self
            .open_tabs
            .iter()
            .filter(|t| t.tab_kind != DocumentTabKind::DiffViewer)
            .map(|t| t.id.clone())
            .collect();

        for tid in tabs_to_reload {
            let _ = self.reload_tab_from_disk(&tid);
        }
    }
}
