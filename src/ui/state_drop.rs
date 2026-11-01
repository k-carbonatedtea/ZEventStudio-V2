//! 窗口外部文件/目录拖拽接收与批量派发处理

use std::path::PathBuf;
use crate::ui::state::AppState;

impl AppState {
    /// 处理拖拽至窗口的外部文件或目录路径列表
    pub fn open_dropped_paths(&mut self, paths: &[PathBuf]) {
        if paths.is_empty() {
            return;
        }

        let mut directories = Vec::new();
        let mut pack_files = Vec::new();
        let mut normal_files = Vec::new();

        for path in paths {
            if path.is_dir() {
                directories.push(path);
                continue;
            }
            if !path.is_file() {
                continue;
            }

            let file_name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();

            if crate::model::is_sarc_extension(&file_name) {
                pack_files.push(path);
            } else {
                normal_files.push(path);
            }
        }

        self.apply_dropped_project_targets(&directories, &pack_files);
        self.apply_dropped_normal_files(&normal_files);
    }

    /// 应用拖拽的项目目标（优先打开目录，次选打开独立封包）
    fn apply_dropped_project_targets(
        &mut self,
        directories: &[&PathBuf],
        pack_files: &[&PathBuf],
    ) {
        if let Some(first_dir) = directories.first() {
            let path_string = first_dir.to_string_lossy().to_string();
            self.open_project_directory(&path_string);
            return;
        }

        if let Some(first_pack) = pack_files.first() {
            let path_string = first_pack.to_string_lossy().to_string();
            self.open_pack_file(&path_string);
        }
    }

    /// 打开拖拽的所有普通文件标签页
    fn apply_dropped_normal_files(&mut self, normal_files: &[&PathBuf]) {
        for file_path in normal_files {
            let path_string = file_path.to_string_lossy().to_string();
            let file_name = file_path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            self.load_file_by_path(&path_string);

            if !self.is_path_inside_project(&path_string) {
                let extension = file_path
                    .extension()
                    .map(|ext| ext.to_string_lossy().to_string())
                    .unwrap_or_else(|| "bfevfl".to_string());
                self.add_recent_project(file_name, path_string, extension);
            }
        }
    }
}

