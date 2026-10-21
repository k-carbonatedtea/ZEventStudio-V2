use gpui::*;
use crate::model::DocumentTabKind;
use crate::ui::components::Toast;
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

    /// 快速丢弃关联文件的打开标签页的未保存修改并关闭相关 Diff 视图 (零延迟瞬时恢复)
    pub fn fast_discard_open_tabs_for_file(&mut self, rel_path: &str, is_untracked: bool) {
        let norm_rel = rel_path.replace('\\', "/");
        let project_dir = self.get_git_working_dir();

        // 1. 关闭关联的 Working Tree 差异对比标签页
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

        // 2. 查找关联的代码编辑器或流程图标签页并还原未保存修改
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
            if is_untracked {
                self.close_document_tab(&tid);
            } else {
                self.discard_document_changes(&tid);
                let is_missing = !std::path::Path::new(&fpath).exists();
                if is_missing {
                    self.close_document_tab(&tid);
                }
            }
        }
    }

    /// 快速丢弃所有标签页的未保存修改并关闭所有工作区 Diff 视图
    pub fn fast_discard_all_open_tabs(&mut self) {
        let diff_tabs_to_close: Vec<String> = self
            .open_tabs
            .iter()
            .filter(|t| t.tab_kind == DocumentTabKind::DiffViewer && t.id.starts_with("diff:working:"))
            .map(|t| t.id.clone())
            .collect();

        for dt_id in diff_tabs_to_close {
            self.close_document_tab(&dt_id);
        }

        self.discard_all_modified_documents();
    }

    /// 兼容旧接口：处理 Git 放弃修改后的标签页同步
    #[allow(dead_code)]
    pub fn handle_git_discarded_file(&mut self, rel_path: &str, is_untracked: bool) {
        self.fast_discard_open_tabs_for_file(rel_path, is_untracked);
    }

    /// 兼容旧接口：处理 Git 放弃全部未暂存修改后的标签页同步
    #[allow(dead_code)]
    pub fn handle_git_discard_all(&mut self) {
        self.fast_discard_all_open_tabs();
    }
}

/// 异步放弃单个文件更改并在后台执行 Git checkout / 文件删除
pub fn git_discard_file_async(
    model: &Entity<AppState>,
    rel_path: String,
    is_untracked: bool,
    cx: &mut App,
) {
    // 1. 立即执行内存中快速放弃 (0ms 延迟，UI 线程无感响应)
    let git_dir = model.update(cx, |st, cx| {
        if st.git_operating_files.contains(&rel_path) {
            return None;
        }
        st.git_operating_files.insert(rel_path.clone());
        st.fast_discard_open_tabs_for_file(&rel_path, is_untracked);
        cx.notify();
        st.get_git_working_dir()
    });

    let dir = match git_dir {
        Some(d) => d,
        None => return,
    };

    let model = model.clone();
    let path_clone = rel_path.clone();

    // 2. 在后台线程池异步执行 Git 进程或物理文件删除
    cx.spawn(async move |cx| {
        let path_bg = path_clone.clone();
        let dir_bg = dir.clone();

        let (discard_res, new_status) = cx
            .background_executor()
            .spawn(async move {
                let res = crate::model::git::discard_file(&dir_bg, &path_bg, is_untracked);
                let status = crate::model::git::get_repo_status(&dir_bg).ok();
                (res, status)
            })
            .await;

        // 3. 回到主线程更新状态与通知 UI
        cx.update(|cx| {
            model.update(cx, |st, cx| {
                st.git_operating_files.remove(&path_clone);
                if let Some(status) = new_status {
                    st.apply_git_status(status);
                }
                match discard_res {
                    Ok(()) => {
                        st.toast = Some(Toast::info(format!(
                            "{} 「{}」",
                            st.t("source_control.discard_success"),
                            path_clone
                        )));
                    }
                    Err(e) => {
                        st.toast = Some(Toast::error(format!(
                            "{}: {}",
                            st.t("source_control.discarding"),
                            e
                        )));
                    }
                }
                cx.notify();
            });
        });
    })
    .detach();
}

/// 异步放弃所有未暂存更改并在后台执行 Git checkout / clean
pub fn git_discard_all_async(model: &Entity<AppState>, cx: &mut App) {
    let git_dir = model.update(cx, |st, cx| {
        if st.git_is_operating {
            return None;
        }
        st.git_is_operating = true;
        st.fast_discard_all_open_tabs();
        cx.notify();
        st.get_git_working_dir()
    });

    let dir = match git_dir {
        Some(d) => d,
        None => return,
    };

    let model = model.clone();
    cx.spawn(async move |cx| {
        let dir_bg = dir.clone();
        let (discard_res, new_status) = cx
            .background_executor()
            .spawn(async move {
                let res = crate::model::git::discard_all(&dir_bg);
                let status = crate::model::git::get_repo_status(&dir_bg).ok();
                (res, status)
            })
            .await;

        cx.update(|cx| {
            model.update(cx, |st, cx| {
                st.git_is_operating = false;
                if let Some(status) = new_status {
                    st.apply_git_status(status);
                }
                match discard_res {
                    Ok(()) => {
                        st.toast = Some(Toast::info(st.t("source_control.discard_success")));
                    }
                    Err(e) => {
                        st.toast = Some(Toast::error(format!(
                            "{}: {}",
                            st.t("source_control.discarding"),
                            e
                        )));
                    }
                }
                cx.notify();
            });
        });
    })
    .detach();
}
