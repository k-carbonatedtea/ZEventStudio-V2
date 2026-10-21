use crate::ui::components::Toast;
use crate::ui::state::AppState;
use crate::ui::state_types::{DeleteConfirmModalState, FileTreeContextMenu, RenameModalState};

/// 资源管理器文件与封包条目增删改查交互扩展
impl AppState {
    /// 打开文件树右键上下文菜单
    pub fn open_file_tree_context_menu(
        &mut self,
        path: String,
        name: String,
        is_dir: bool,
        is_sarc: bool,
        position: (f32, f32),
    ) {
        self.clear_active_text_inputs();
        let is_inside_sarc = path.starts_with("SARC:") && path.contains("//");
        self.file_tree_context_menu = Some(FileTreeContextMenu {
            path,
            name,
            is_dir,
            is_sarc,
            is_inside_sarc,
            position,
        });
    }

    /// 关闭文件树右键上下文菜单
    pub fn close_file_tree_context_menu(&mut self) {
        self.file_tree_context_menu = None;
    }

    /// 触发重命名对话框
    pub fn trigger_rename_item(&mut self, path: &str, name: &str, is_dir: bool, is_inside_sarc: bool) {
        self.close_file_tree_context_menu();
        self.clear_active_text_inputs();
        self.rename_modal = Some(RenameModalState {
            target_path: path.to_string(),
            old_name: name.to_string(),
            new_name: name.to_string(),
            is_inside_sarc,
            is_dir,
            focused: true,
        });
    }

    /// 执行重命名提交
    pub fn execute_rename_item(&mut self) {
        let (old_path, new_name, old_name) = match &self.rename_modal {
            Some(m) => (m.target_path.clone(), m.new_name.trim().to_string(), m.old_name.clone()),
            None => return,
        };

        if new_name.is_empty() || new_name == old_name {
            self.rename_modal = None;
            return;
        }

        match crate::model::rename_file_in_target(&old_path, &new_name) {
            Ok(new_path) => {
                // 若被重命名的文件处于打开标签页中，同步更新其路径与标题
                for tab in &mut self.open_tabs {
                    if tab.file_path == old_path {
                        tab.file_path = new_path.clone();
                        tab.title = new_name.clone();
                    }
                }
                if self.selected_file_path.as_deref() == Some(&old_path) {
                    self.selected_file_path = Some(new_path);
                }
                self.rename_modal = None;
                self.reload_project();
                self.toast = Some(Toast::success(format!("{} → {}", old_name, new_name)));
            }
            Err(e) => {
                self.toast = Some(Toast::error(format!("{}: {}", self.t("toast.rename_error"), e)));
            }
        }
    }

    /// 触发删除确认对话框
    pub fn trigger_delete_item(&mut self, path: &str, name: &str, is_dir: bool, is_inside_sarc: bool) {
        self.close_file_tree_context_menu();
        self.clear_active_text_inputs();
        self.delete_confirm_modal = Some(DeleteConfirmModalState {
            target_path: path.to_string(),
            name: name.to_string(),
            is_inside_sarc,
            is_dir,
        });
    }

    /// 执行确认删除
    pub fn execute_delete_item(&mut self) {
        let (target_path, name) = match &self.delete_confirm_modal {
            Some(m) => (m.target_path.clone(), m.name.clone()),
            None => return,
        };

        match crate::model::delete_file_from_target(&target_path) {
            Ok(()) => {
                // 若被删除的文件在标签页打开，直接关闭它
                let tabs_to_close: Vec<String> = self
                    .open_tabs
                    .iter()
                    .filter(|t| t.file_path == target_path || t.file_path.starts_with(&format!("{}/", target_path)))
                    .map(|t| t.id.clone())
                    .collect();

                for tid in tabs_to_close {
                    self.close_document_tab(&tid);
                }

                if self.selected_file_path.as_deref() == Some(&target_path) {
                    self.selected_file_path = None;
                }

                self.delete_confirm_modal = None;
                self.reload_project();
                self.toast = Some(Toast::success(format!("{} 「{}」", self.t("toast.delete_success"), name)));
            }
            Err(e) => {
                self.toast = Some(Toast::error(format!("{}: {}", self.t("toast.delete_error"), e)));
            }
        }
    }

    /// 在系统原生资源管理器中定位并高亮显示目标文件
    pub fn reveal_item_in_file_explorer(&mut self, path: &str) {
        let physical = if path.starts_with("SARC:") {
            let clean = path.trim_start_matches("SARC:");
            clean.split("//").next().unwrap_or(clean)
        } else {
            path
        };

        // 处理相对路径，转换为绝对路径
        let mut target_path = std::path::PathBuf::from(physical);
        if !target_path.is_absolute() {
            if let Some(ref project_root) = self.project_path {
                let candidate = std::path::Path::new(project_root).join(&target_path);
                if candidate.exists() {
                    target_path = candidate;
                }
            }
        }
        if !target_path.is_absolute() {
            if let Ok(abs) = std::path::absolute(&target_path) {
                target_path = abs;
            }
        }

        // 规范化路径字符串 (去除 Windows 下 UNC \\?\ 前缀，转为标准反斜杠)
        let mut path_str = target_path.to_string_lossy().to_string();
        if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
            path_str = stripped.to_string();
        }

        #[cfg(target_os = "windows")]
        let normalized = path_str.replace('/', "\\").trim_end_matches('\\').to_string();
        #[cfg(not(target_os = "windows"))]
        let normalized = path_str;

        let check_path = std::path::Path::new(&normalized);
        let (final_path, is_file) = if check_path.exists() {
            (normalized.clone(), !check_path.is_dir() || path.starts_with("SARC:"))
        } else if let Some(parent) = check_path.parent() {
            if parent.exists() {
                let parent_str = parent.to_string_lossy().to_string();
                #[cfg(target_os = "windows")]
                let p_norm = parent_str.replace('/', "\\").trim_end_matches('\\').to_string();
                #[cfg(not(target_os = "windows"))]
                let p_norm = parent_str;
                (p_norm, false)
            } else {
                self.toast = Some(Toast::error(format!("{}: {}", self.t("toast.file_not_found"), normalized)));
                return;
            }
        } else {
            self.toast = Some(Toast::error(format!("{}: {}", self.t("toast.file_not_found"), normalized)));
            return;
        };

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let mut cmd = std::process::Command::new("explorer");
            if is_file {
                cmd.raw_arg(format!("/select,\"{}\"", final_path));
            } else {
                cmd.raw_arg(format!("\"{}\"", final_path));
            }
            if let Err(e) = cmd.spawn() {
                self.toast = Some(Toast::error(format!("{}: {}", self.t("toast.explorer_open_failed"), e)));
            }
        }
        #[cfg(target_os = "macos")]
        {
            let mut cmd = std::process::Command::new("open");
            if is_file {
                cmd.arg("-R").arg(&final_path);
            } else {
                cmd.arg(&final_path);
            }
            let _ = cmd.spawn();
        }
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(&final_path)
                .spawn();
        }
    }
}
