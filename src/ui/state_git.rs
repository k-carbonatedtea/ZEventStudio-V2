use crate::ui::components::Toast;
use crate::ui::state::{AppState, GitCommitFocus};

/// Git 源代码管理与仓库操作状态扩展
impl AppState {
    pub fn refresh_git_status(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::get_repo_status(&dir) {
                Ok(status) => {
                    self.git_is_repo = status.is_repo;
                    self.git_branch = status.branch;
                    self.git_staged = status.staged;
                    self.git_unstaged = status.unstaged;
                }
                Err(_) => {
                    self.git_is_repo = false;
                    self.git_branch = None;
                    self.git_staged = std::sync::Arc::new(Vec::new());
                    self.git_unstaged = std::sync::Arc::new(Vec::new());
                }
            }
            self.refresh_git_history();
        } else {
            self.git_is_repo = false;
            self.git_branch = None;
            self.git_staged = std::sync::Arc::new(Vec::new());
            self.git_unstaged = std::sync::Arc::new(Vec::new());
            self.git_commits = std::sync::Arc::new(Vec::new());
        }
    }

    /// 刷新 Git 提交历史与分支图谱
    pub fn refresh_git_history(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            if self.git_is_repo {
                if let Ok(commits) = crate::model::git::get_commit_history(&dir, self.git_history_limit) {
                    self.git_commits = std::sync::Arc::new(commits);
                    return;
                }
            }
        }
        self.git_commits = std::sync::Arc::new(Vec::new());
    }

    /// 加载更多历史提交
    pub fn load_more_git_history(&mut self) {
        self.git_history_limit += 50;
        self.refresh_git_history();
    }

    /// 展开或收起某次提交的文件列表
    pub fn toggle_expand_commit(&mut self, hash: &str) {
        if self.git_expanded_commit_hash.as_deref() == Some(hash) {
            self.git_expanded_commit_hash = None;
            self.git_expanded_commit_files.clear();
        } else {
            self.git_expanded_commit_hash = Some(hash.to_string());
            if let Some(dir) = self.get_git_working_dir() {
                if let Ok(files) = crate::model::git::get_commit_files(&dir, hash) {
                    self.git_expanded_commit_files = files;
                } else {
                    self.git_expanded_commit_files.clear();
                }
            }
        }
    }

    /// 展开或折叠指定提交中 SARC 封包文件的子文件列表
    pub fn toggle_expand_commit_sarc(&mut self, hash: &str, rel_path: &str) {
        let key = format!("commit:{}:{}", hash, rel_path);
        if self.git_expanded_sarc_keys.contains(&key) {
            self.git_expanded_sarc_keys.remove(&key);
        } else {
            self.git_expanded_sarc_keys.insert(key.clone());
            if !self.git_sarc_subfiles_cache.contains_key(&key) {
                if let Some(dir) = self.get_git_working_dir() {
                    let commit_item = self
                        .git_commits
                        .iter()
                        .find(|c| c.hash == hash || c.short_hash == hash);
                    let parent_hash = commit_item
                        .and_then(|c| c.parents.first())
                        .cloned()
                        .or_else(|| {
                            let mut cmd = std::process::Command::new("git");
                            cmd.args(["rev-parse", &format!("{}^", hash)]);
                            cmd.current_dir(&dir);
                            #[cfg(windows)]
                            {
                                use std::os::windows::process::CommandExt;
                                cmd.creation_flags(0x08000000);
                            }
                            cmd.output().ok().and_then(|out| {
                                if out.status.success() {
                                    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                                    if !s.is_empty() { Some(s) } else { None }
                                } else {
                                    None
                                }
                            })
                        });

                    let new_bytes = crate::model::binary_diff::get_commit_file_bytes(&dir, hash, rel_path).ok();
                    let old_bytes = parent_hash.and_then(|p| {
                        crate::model::binary_diff::get_commit_file_bytes(&dir, &p, rel_path).ok()
                    });

                    let sub_changes = crate::model::binary_diff::get_sarc_changed_subfiles(
                        old_bytes.as_deref(),
                        new_bytes.as_deref(),
                    );
                    self.git_sarc_subfiles_cache.insert(key, sub_changes);
                }
            }
        }
    }

    /// 展开或折叠工作区/暂存区 SARC 封包文件的子文件列表
    #[allow(dead_code)]
    pub fn toggle_expand_working_sarc(&mut self, rel_path: &str, is_staged: bool) {
        let key = if is_staged {
            format!("staged:{}", rel_path)
        } else {
            format!("working:{}", rel_path)
        };
        if self.git_expanded_sarc_keys.contains(&key) {
            self.git_expanded_sarc_keys.remove(&key);
        } else {
            self.git_expanded_sarc_keys.insert(key.clone());
            if !self.git_sarc_subfiles_cache.contains_key(&key) {
                if let Some(dir) = self.get_git_working_dir() {
                    let new_bytes = crate::model::binary_diff::get_working_tree_file_bytes(&dir, rel_path, is_staged).ok();
                    let old_bytes = if is_staged {
                        crate::model::binary_diff::get_commit_file_bytes(&dir, "HEAD", rel_path).ok()
                    } else {
                        crate::model::binary_diff::get_working_tree_file_bytes(&dir, rel_path, true).ok()
                            .or_else(|| crate::model::binary_diff::get_commit_file_bytes(&dir, "HEAD", rel_path).ok())
                    };

                    let sub_changes = crate::model::binary_diff::get_sarc_changed_subfiles(
                        old_bytes.as_deref(),
                        new_bytes.as_deref(),
                    );
                    self.git_sarc_subfiles_cache.insert(key, sub_changes);
                }
            }
        }
    }

    /// 撤回指定的提交记录 (git revert <hash> --no-edit)
    pub fn revert_commit_by_hash(&mut self, hash: &str) {
        self.git_commit_context_menu = None;
        let dir = match self.get_git_working_dir() {
            Some(d) => d,
            None => {
                self.toast = Some(crate::ui::components::Toast::warning("未找到有效 Git 工作区"));
                return;
            }
        };

        let short_h = hash[..7.min(hash.len())].to_string();
        match crate::model::git_actions::revert_commit(&dir, hash) {
            Ok(_) => {
                self.toast = Some(crate::ui::components::Toast::success(format!("✓ 已成功撤回提交: {}", short_h)));
                self.refresh_git_status();
                self.refresh_git_history();
            }
            Err(e) => {
                self.toast = Some(crate::ui::components::Toast::error(format!("❌ 撤回提交失败: {}", e)));
                self.refresh_git_status();
            }
        }
    }

    /// 重置分支至指定提交 (git reset <mode> <hash>)
    pub fn reset_to_commit_by_hash(&mut self, hash: &str, mode: &str) {
        self.git_commit_context_menu = None;
        let dir = match self.get_git_working_dir() {
            Some(d) => d,
            None => {
                self.toast = Some(crate::ui::components::Toast::warning("未找到有效 Git 工作区"));
                return;
            }
        };

        let short_h = hash[..7.min(hash.len())].to_string();
        let mode_desc = match mode {
            "--soft" => "软重置(保留暂存区)",
            "--mixed" => "混合重置(保留工作区)",
            "--hard" => "硬重置",
            _ => "重置",
        };

        match crate::model::git_actions::reset_to_commit(&dir, hash, mode) {
            Ok(_) => {
                self.toast = Some(crate::ui::components::Toast::success(format!("✓ 已{}至提交: {}", mode_desc, short_h)));
                self.refresh_git_status();
                self.refresh_git_history();
            }
            Err(e) => {
                self.toast = Some(crate::ui::components::Toast::error(format!("❌ 重置分支失败: {}", e)));
                self.refresh_git_status();
            }
        }
    }

    /// 检出指定提交 (git checkout <hash>)
    pub fn checkout_commit_by_hash(&mut self, hash: &str) {
        self.git_commit_context_menu = None;
        let dir = match self.get_git_working_dir() {
            Some(d) => d,
            None => {
                self.toast = Some(crate::ui::components::Toast::warning("未找到有效 Git 工作区"));
                return;
            }
        };

        let short_h = hash[..7.min(hash.len())].to_string();
        match crate::model::git_actions::checkout_commit(&dir, hash) {
            Ok(_) => {
                self.toast = Some(crate::ui::components::Toast::success(format!("✓ 已检出切换至提交: {}", short_h)));
                self.refresh_git_status();
                self.refresh_git_history();
            }
            Err(e) => {
                self.toast = Some(crate::ui::components::Toast::error(format!("❌ 检出提交失败: {}", e)));
                self.refresh_git_status();
            }
        }
    }

    pub fn git_init_repository(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::init_repo(&dir) {
                Ok(()) => {
                    self.refresh_git_status();
                    self.toast = Some(Toast::success(format!("已成功为 {} 初始化 Git 仓库", dir)));
                }
                Err(err) => {
                    self.toast = Some(Toast::error(format!("初始化 Git 仓库失败: {}", err)));
                }
            }
        } else {
            self.toast = Some(Toast::warning("请先在资源管理器中打开 Mod 项目目录"));
        }
    }

    /// 暂存单个文件 (先自动保存所有未保存修改)
    pub fn git_stage_file(&mut self, rel_path: &str) {
        self.save_all_modified_documents();
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::stage_file(&dir, rel_path) {
                Ok(()) => {
                    self.refresh_git_status();
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("暂存文件失败: {}", e)));
                }
            }
        }
    }

    /// 暂存所有变更 (先自动保存所有未保存修改)
    pub fn git_stage_all(&mut self) {
        self.save_all_modified_documents();
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::stage_all(&dir) {
                Ok(()) => {
                    self.refresh_git_status();
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("暂存全部失败: {}", e)));
                }
            }
        }
    }

    /// 取消暂存单个文件
    pub fn git_unstage_file(&mut self, rel_path: &str) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::unstage_file(&dir, rel_path) {
                Ok(()) => {
                    self.refresh_git_status();
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("取消暂存失败: {}", e)));
                }
            }
        }
    }

    /// 取消暂存所有变更
    pub fn git_unstage_all(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::unstage_all(&dir) {
                Ok(()) => {
                    self.refresh_git_status();
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("取消全部暂存失败: {}", e)));
                }
            }
        }
    }

    /// 放弃单个文件的修改
    pub fn git_discard_file(&mut self, rel_path: &str, is_untracked: bool) {
        if let Some(dir) = self.get_git_working_dir() {
            let is_open_mod = self.is_file_modified(rel_path);
            let res = crate::model::git::discard_file(&dir, rel_path, is_untracked);
            self.refresh_git_status();
            self.handle_git_discarded_file(rel_path, is_untracked);
            if res.is_ok() || is_open_mod {
                self.toast = Some(Toast::info(format!("已放弃更改并重新载入: {}", rel_path)));
            } else if let Err(e) = res {
                self.toast = Some(Toast::error(format!("放弃更改失败: {}", e)));
            }
        }
    }

    /// 放弃所有修改
    pub fn git_discard_all(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            match crate::model::git::discard_all(&dir) {
                Ok(()) => {
                    self.refresh_git_status();
                    self.handle_git_discard_all();
                    self.toast = Some(Toast::info("已放弃所有未暂存更改并刷新文档"));
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("放弃所有更改失败: {}", e)));
                }
            }
        }
    }

    /// 提交暂存的修改 (如果只有工作区未暂存修改，且用户点击提交，自动暂存所有并提交)
    pub fn git_commit(&mut self) {
        if let Some(dir) = self.get_git_working_dir() {
            let title = self.git_commit_title.trim().to_string();
            if title.is_empty() {
                self.toast = Some(Toast::warning("请输入提交信息 (Commit Title)"));
                self.git_commit_focus = GitCommitFocus::Title;
                return;
            }

            // 提交前先保存所有未保存修改
            self.save_all_modified_documents();

            // 若暂存区为空，但工作区有改动，则先自动暂存所有
            if self.git_staged.is_empty() && !self.git_unstaged.is_empty() {
                if let Err(e) = crate::model::git::stage_all(&dir) {
                    self.toast = Some(Toast::error(format!("自动暂存失败: {}", e)));
                    return;
                }
            }

            let desc = self.git_commit_desc.trim().to_string();
            match crate::model::git::commit(&dir, &title, &desc) {
                Ok(msg) => {
                    self.git_commit_title.clear();
                    self.git_commit_desc.clear();
                    self.git_commit_focus = GitCommitFocus::None;
                    self.git_show_desc_field = false;
                    self.refresh_git_status();
                    let summary = msg.lines().next().unwrap_or("提交成功").to_string();
                    self.toast = Some(Toast::success(format!("Git 提交成功: {}", summary)));
                }
                Err(e) => {
                    self.toast = Some(Toast::error(format!("Git 提交失败: {}", e)));
                }
            }
        } else {
            self.toast = Some(Toast::warning("请先打开 Mod 项目工作区"));
        }
    }

}