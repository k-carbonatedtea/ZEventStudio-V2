use std::path::Path;
use crate::ui::state::{AppState, RecentProjectItem};

/// 判断 child 是否严格位于 parent 目录内部 (非自身且是其后代)
fn is_strict_subpath(child: &Path, parent: &Path) -> bool {
    if let (Ok(can_child), Ok(can_parent)) = (child.canonicalize(), parent.canonicalize()) {
        can_child.starts_with(&can_parent) && can_child != can_parent
    } else {
        let child_norm = child.to_string_lossy().replace('\\', "/").to_lowercase();
        let mut parent_norm = parent.to_string_lossy().replace('\\', "/").to_lowercase();
        if !parent_norm.ends_with('/') {
            parent_norm.push('/');
        }
        child_norm.starts_with(&parent_norm) && child_norm != parent_norm.trim_end_matches('/')
    }
}

impl AppState {
    /// 检查指定路径是否属于某个已打开的项目或已知项目根目录的子文件
    pub fn is_path_inside_project(&self, target_path: &str) -> bool {
        let norm_target = target_path.replace('\\', "/").to_lowercase();

        // 1. 虚拟路径或封包内部子文件 (包含 "//" 或以 "SARC:" 开头) 属于封包内容，不可作为独立项目
        if norm_target.starts_with("sarc:") || norm_target.contains("//") {
            return true;
        }

        // 2. 版本控制与临时元数据文件 (.git, .vscode, 等) 绝不能作为独立项目
        if norm_target.contains("/.git/")
            || norm_target.ends_with("/.git")
            || norm_target.contains("/.vscode/")
            || norm_target.ends_with("/.vscode")
            || norm_target.contains("/.idea/")
            || norm_target.ends_with("/.idea")
        {
            return true;
        }

        let target_p = Path::new(target_path);

        // 3. 检查是否在当前打开的项目目录下
        if let Some(ref current_proj) = self.project_path {
            let current_proj_p = Path::new(current_proj);
            if current_proj_p.is_dir() && is_strict_subpath(target_p, current_proj_p) {
                return true;
            }
        }

        // 4. 检查是否在任一已知最近项目文件夹内部
        for item in &self.recent_projects {
            if item.project_type == "mod" || Path::new(&item.path).is_dir() {
                let proj_p = Path::new(&item.path);
                if is_strict_subpath(target_p, proj_p) {
                    return true;
                }
            }
        }

        // 5. 检查祖先目录是否包含 .git 或 romfs/content（识别属于 Mod/Git 仓库内部的子文件）
        if let Some(parent) = target_p.parent() {
            for ancestor in parent.ancestors() {
                if ancestor.join(".git").is_dir() {
                    return true;
                }
                if let Some(name) = ancestor.file_name() {
                    let name_lower = name.to_string_lossy().to_lowercase();
                    if name_lower == "romfs" || name_lower == "content" {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// 清理并净化最近打开记录：移除所有属于项目内部的子文件、版本控制临时文件或已不存在的路径
    pub fn clean_invalid_recent_projects(&mut self) {
        let mut known_projects: Vec<std::path::PathBuf> = Vec::new();

        // 收集当前已打开的项目路径
        if let Some(ref current_proj) = self.project_path {
            let p = std::path::PathBuf::from(current_proj);
            if p.is_dir() {
                known_projects.push(p);
            }
        }

        // 收集历史记录中所有 Mod 项目文件夹
        for item in &self.recent_projects {
            if item.project_type == "mod" || Path::new(&item.path).is_dir() {
                let p = std::path::PathBuf::from(&item.path);
                if !known_projects.contains(&p) {
                    known_projects.push(p);
                }
            }
        }

        let original_len = self.recent_projects.len();

        self.recent_projects.retain(|item| {
            let norm = item.path.replace('\\', "/").to_lowercase();

            // 过滤虚拟/封包内部路径
            if norm.starts_with("sarc:") || norm.contains("//") {
                return false;
            }

            // 过滤 .git, .vscode, .idea 等内部文件
            if norm.contains("/.git/")
                || norm.ends_with("/.git")
                || norm.contains("/.vscode/")
                || norm.ends_with("/.vscode")
                || norm.contains("/.idea/")
                || norm.ends_with("/.idea")
            {
                return false;
            }

            let item_path = Path::new(&item.path);

            // 如果路径已不存在，剔除
            if !item_path.exists() {
                return false;
            }

            // 如果是文件夹本身，保留
            if item_path.is_dir() {
                return true;
            }

            // 如果是文件，检查其是否位于任何已知 Mod 项目根目录下
            for proj_dir in &known_projects {
                if is_strict_subpath(item_path, proj_dir) {
                    return false;
                }
            }

            // 检查祖先目录是否包含 .git 或 romfs/content
            if let Some(parent) = item_path.parent() {
                for ancestor in parent.ancestors() {
                    if ancestor.join(".git").is_dir() {
                        return false;
                    }
                    if let Some(name) = ancestor.file_name() {
                        let name_lower = name.to_string_lossy().to_lowercase();
                        if name_lower == "romfs" || name_lower == "content" {
                            return false;
                        }
                    }
                }
            }

            true
        });

        // 若有无效项被剔除，同步回持久化配置并落盘保存
        if self.recent_projects.len() != original_len {
            self.game_paths.recent_projects = self.recent_projects.clone();
            let _ = self.game_paths.save_to_disk();
        }
    }

    /// 辅助添加最近项目项并持久化保存 (自动拦截项目内部子文件)
    pub fn add_recent_project(&mut self, name: String, path: String, project_type: String) {
        // 如果目标路径是某个项目内部的文件或版本控制临时文件，严禁加入最近项目列表
        if self.is_path_inside_project(&path) {
            return;
        }

        self.recent_projects.retain(|p| p.path != path);
        self.recent_projects.insert(
            0,
            RecentProjectItem {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                path,
                project_type,
            },
        );
        self.recent_projects.truncate(20);
        self.clean_invalid_recent_projects();
        self.game_paths.recent_projects = self.recent_projects.clone();
        let _ = self.game_paths.save_to_disk();
    }

    /// 清空所有最近打开记录并持久化
    pub fn clear_recent_projects(&mut self) {
        self.recent_projects.clear();
        self.game_paths.recent_projects.clear();
        let _ = self.game_paths.save_to_disk();
    }

    /// 删除单条最近打开记录并持久化
    pub fn remove_recent_project(&mut self, index: usize) {
        if index < self.recent_projects.len() {
            self.recent_projects.remove(index);
            self.game_paths.recent_projects = self.recent_projects.clone();
            let _ = self.game_paths.save_to_disk();
        }
    }

    /// 智能打开任意类型最近记录 (文件夹/SARC/单文件)
    pub fn open_recent_item(&mut self, item: &RecentProjectItem) {
        let path = item.path.clone();
        if path.is_empty() {
            return;
        }

        let p = Path::new(&path);
        if p.is_dir() {
            self.open_project_directory(&path);
        } else if crate::model::is_sarc_extension(&item.name)
            || item.project_type == "pack"
            || item.project_type == "sbeventpack"
            || item.project_type == "sbactorpack"
        {
            self.open_pack_file(&path);
        } else {
            self.load_file_by_path(&path);
            self.add_recent_project(item.name.clone(), path, item.project_type.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_strict_subpath() {
        let parent = Path::new("C:/mods/9999_BCML");
        let child = Path::new("C:/mods/9999_BCML/.git/config");
        assert!(is_strict_subpath(child, parent));

        // 相同目录不属于其自身子路径
        assert!(!is_strict_subpath(parent, parent));

        // 具有相同名称前缀的同级目录不属于其子路径
        let sibling = Path::new("C:/mods/9999_BCML_extra/config");
        assert!(!is_strict_subpath(sibling, parent));

        // 外部无关路径
        let unrelated = Path::new("D:/other/test.bfevfl");
        assert!(!is_strict_subpath(unrelated, parent));
    }
}
