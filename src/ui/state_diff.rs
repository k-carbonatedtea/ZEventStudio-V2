use std::path::Path;
use crate::model::binary_diff::{
    get_commit_file_bytes, get_working_tree_file_bytes, resolve_smart_code_diff,
};
use crate::model::file_tree::is_sarc_extension;
use crate::model::{
    align_unified_diff, DiffViewerTabState, OpenDocumentTab,
};
use crate::ui::components::Toast;
use crate::ui::state::AppState;

impl AppState {
    /// 打开指定提交中某个文件的差异对比标签页 (IDE 对比模式，参考图2)
    pub fn open_commit_file_diff_tab(&mut self, hash: &str, rel_path: &str) {
        let dir = match self.get_git_working_dir() {
            Some(d) => d,
            None => {
                self.toast = Some(Toast::warning("请先在资源管理器中打开项目目录"));
                return;
            }
        };

        let file_name = Path::new(rel_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(rel_path)
            .to_string();

        // 查找提交信息以获取其短哈希与父提交哈希
        let commit_item = self
            .git_commits
            .iter()
            .find(|c| c.hash == hash || c.short_hash == hash);

        let new_rev = commit_item
            .map(|c| c.short_hash.clone())
            .unwrap_or_else(|| hash[..7.min(hash.len())].to_string());

        let parent_rev = commit_item
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

        let old_rev = parent_rev
            .as_deref()
            .map(|p| p[..7.min(p.len())].to_string())
            .unwrap_or_else(|| "0000000".to_string());

        let tab_id = format!("diff:commit:{}:{}", new_rev, rel_path);

        // 如果该对比 Tab 已打开，直接切换至该 Tab
        if self.open_tabs.iter().any(|t| t.id == tab_id) {
            self.switch_document_tab(&tab_id);
            return;
        }

        // 1. 获取基础 git diff
        match crate::model::git::get_commit_file_diff(&dir, hash, rel_path) {
            Ok(raw_diff) => {
                let full_path = Path::new(&dir).join(rel_path).to_string_lossy().to_string();
                let tab_title = format!("{} ({}) ↔ {} ({})", file_name, old_rev, file_name, new_rev);

                // 2. 检查是否为二进制或 SARC 封包，尝试智能代码解析
                let is_binary = raw_diff.lines.is_empty()
                    || is_sarc_extension(rel_path)
                    || raw_diff.lines.iter().any(|l| l.text.contains("Binary files"));

                let mut sub_files = Vec::new();
                if is_binary {
                    let new_bytes = get_commit_file_bytes(&dir, hash, rel_path).ok();
                    let old_bytes = if let Some(ref p_hash) = parent_rev {
                        get_commit_file_bytes(&dir, p_hash, rel_path).ok()
                    } else {
                        None
                    };

                    if let Some(smart_subs) = resolve_smart_code_diff(
                        old_bytes.as_deref(),
                        new_bytes.as_deref(),
                        rel_path,
                    ) {
                        sub_files = smart_subs;
                    }
                }

                let (final_diff, final_rows) = if let Some(first) = sub_files.first() {
                    (first.diff.clone(), first.rows.clone())
                } else {
                    let rows = align_unified_diff(&raw_diff);
                    (raw_diff, rows)
                };

                let diff_tab_state = DiffViewerTabState {
                    title: tab_title.clone(),
                    file_path: rel_path.to_string(),
                    full_path,
                    old_rev,
                    new_rev,
                    diff: final_diff,
                    rows: final_rows,
                    sub_files,
                    active_sub_file_idx: 0,
                };

                let new_tab = OpenDocumentTab::new_diff(
                    tab_id.clone(),
                    tab_title,
                    rel_path.to_string(),
                    diff_tab_state,
                );

                self.open_tabs.push(new_tab);
                self.switch_document_tab(&tab_id);
            }
            Err(err) => {
                self.toast = Some(Toast::error(format!("获取提交对比差异失败: {}", err)));
            }
        }
    }

    /// 打开工作区或暂存区未提交文件的修改差异对比标签页 (IDE 对比模式，参考图2)
    pub fn open_working_file_diff_tab(&mut self, rel_path: &str, is_staged: bool) {
        let dir = match self.get_git_working_dir() {
            Some(d) => d,
            None => {
                self.toast = Some(Toast::warning("请先在资源管理器中打开项目目录"));
                return;
            }
        };

        let file_name = Path::new(rel_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(rel_path)
            .to_string();

        let (old_rev, new_rev, tab_id_prefix) = if is_staged {
            ("HEAD".to_string(), "Index".to_string(), "diff:staged")
        } else {
            ("Index".to_string(), "Working Tree".to_string(), "diff:working")
        };

        let tab_id = format!("{}:{}", tab_id_prefix, rel_path);

        // 如果该对比 Tab 已打开，直接切换至该 Tab
        if self.open_tabs.iter().any(|t| t.id == tab_id) {
            self.switch_document_tab(&tab_id);
            return;
        }

        // 1. 获取工作区/暂存区文件的统一差异数据
        match crate::model::git::get_working_tree_file_diff(&dir, rel_path, is_staged) {
            Ok(raw_diff) => {
                let full_path = Path::new(&dir).join(rel_path).to_string_lossy().to_string();
                let tab_title = format!("{} ({}) ↔ {} ({})", file_name, old_rev, file_name, new_rev);

                // 2. 检查是否为二进制或 SARC 封包，尝试智能代码解析
                let is_binary = raw_diff.lines.is_empty()
                    || is_sarc_extension(rel_path)
                    || raw_diff.lines.iter().any(|l| l.text.contains("Binary files"));

                let mut sub_files = Vec::new();
                if is_binary {
                    let new_bytes = get_working_tree_file_bytes(&dir, rel_path, is_staged).ok();
                    let old_bytes = if is_staged {
                        get_commit_file_bytes(&dir, "HEAD", rel_path).ok()
                    } else {
                        get_working_tree_file_bytes(&dir, rel_path, true).ok()
                            .or_else(|| get_commit_file_bytes(&dir, "HEAD", rel_path).ok())
                    };

                    if let Some(smart_subs) = resolve_smart_code_diff(
                        old_bytes.as_deref(),
                        new_bytes.as_deref(),
                        rel_path,
                    ) {
                        sub_files = smart_subs;
                    }
                }

                let (final_diff, final_rows) = if let Some(first) = sub_files.first() {
                    (first.diff.clone(), first.rows.clone())
                } else {
                    let rows = align_unified_diff(&raw_diff);
                    (raw_diff, rows)
                };

                let diff_tab_state = DiffViewerTabState {
                    title: tab_title.clone(),
                    file_path: rel_path.to_string(),
                    full_path,
                    old_rev,
                    new_rev,
                    diff: final_diff,
                    rows: final_rows,
                    sub_files,
                    active_sub_file_idx: 0,
                };

                let new_tab = OpenDocumentTab::new_diff(
                    tab_id.clone(),
                    tab_title,
                    rel_path.to_string(),
                    diff_tab_state,
                );

                self.open_tabs.push(new_tab);
                self.switch_document_tab(&tab_id);
            }
            Err(err) => {
                self.toast = Some(Toast::error(format!("获取文件修改差异失败: {}", err)));
            }
        }
    }

    /// 切换 SARC 封包对比模式中正在查看的内部子文件
    pub fn switch_diff_sub_file(&mut self, tab_id: &str, sub_idx: usize) {
        if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == tab_id) {
            if let Some(ref mut ds) = tab.diff_state {
                if sub_idx < ds.sub_files.len() {
                    ds.active_sub_file_idx = sub_idx;
                    if let Some(sub) = ds.sub_files.get(sub_idx) {
                        ds.diff = sub.diff.clone();
                        ds.rows = sub.rows.clone();
                    }
                }
            }
        }
    }
}
