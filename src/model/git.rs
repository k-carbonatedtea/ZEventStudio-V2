use std::path::Path;
use std::process::Command;

/// Git 变更状态种类
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitFileStatusKind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

impl GitFileStatusKind {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Renamed => "R",
            Self::Untracked => "U",
            Self::Conflicted => "!",
        }
    }
}

/// Git 变更文件条目
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitFileItem {
    pub rel_path: String,
    pub file_name: String,
    pub dir_name: String,
    pub full_path: String,
    pub status: GitFileStatusKind,
    pub is_staged: bool,
}

/// Git 仓库整体状态
#[derive(Clone, Debug, Default)]
pub struct GitRepoStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub staged: std::sync::Arc<Vec<GitFileItem>>,
    pub unstaged: std::sync::Arc<Vec<GitFileItem>>,
}

fn run_git_cmd(repo_dir: &str, args: &[&str]) -> Result<std::process::Output, String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    cmd.current_dir(repo_dir);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW: 避免黑色终端弹窗闪烁
    }
    cmd.output().map_err(|e| format!("执行 Git 命令失败: {}", e))
}

/// 检查指定目录是否为独立的 Git 仓库根目录 (必须在该目录下直接存在 .git)
/// 严格禁止向上层父级/祖先目录冒泡穿透，确保打开不同项目时 Git 状态相互独立 (与 VS Code 隔离策略一致)
pub fn check_is_repo(dir: &str) -> bool {
    let p = Path::new(dir);
    if !p.exists() || !p.is_dir() {
        return false;
    }
    p.join(".git").exists()
}

/// 初始化 Git 仓库 (git init)
pub fn init_repo(dir: &str) -> Result<(), String> {
    let output = run_git_cmd(dir, &["init"])?;
    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(err.trim().to_string())
    }
}

/// 获取当前 Git 分支名称
pub fn get_current_branch(dir: &str) -> Option<String> {
    if let Ok(output) = run_git_cmd(dir, &["branch", "--show-current"]) {
        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.is_empty() {
                return Some(branch);
            }
        }
    }
    if let Ok(output) = run_git_cmd(dir, &["rev-parse", "--short", "HEAD"]) {
        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !hash.is_empty() {
                return Some(hash);
            }
        }
    }
    Some("main".to_string())
}

/// 读取并解析当前仓库的状态 (git status --porcelain)
pub fn get_repo_status(dir: &str) -> Result<GitRepoStatus, String> {
    if !check_is_repo(dir) {
        return Ok(GitRepoStatus {
            is_repo: false,
            branch: None,
            staged: std::sync::Arc::new(Vec::new()),
            unstaged: std::sync::Arc::new(Vec::new()),
        });
    }

    let branch = get_current_branch(dir);

    let output = run_git_cmd(dir, &["status", "--porcelain=v1", "-u"])?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(err.trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();

    let root_path = Path::new(dir);

    for line in stdout.lines() {
        let line = line.trim_end();
        if line.len() < 3 {
            continue;
        }

        let x = line.chars().nth(0).unwrap_or(' ');
        let y = line.chars().nth(1).unwrap_or(' ');
        let raw_path = &line[3..].trim();
        let clean_path = raw_path.trim_matches('"');
        let actual_path = if let Some((_, new_p)) = clean_path.split_once(" -> ") {
            new_p.trim()
        } else {
            clean_path
        };

        let path_obj = Path::new(actual_path);
        let file_name = path_obj
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(actual_path)
            .to_string();
        let dir_name = path_obj
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .replace('\\', "/");
        let full_path = root_path.join(actual_path).to_string_lossy().to_string();

        // 1. 暂存区变更 (Staged Changes)
        if x != ' ' && x != '?' {
            let status = match x {
                'M' => GitFileStatusKind::Modified,
                'A' => GitFileStatusKind::Added,
                'D' => GitFileStatusKind::Deleted,
                'R' => GitFileStatusKind::Renamed,
                'U' => GitFileStatusKind::Conflicted,
                _ => GitFileStatusKind::Modified,
            };
            staged.push(GitFileItem {
                rel_path: actual_path.to_string(),
                file_name: file_name.clone(),
                dir_name: dir_name.clone(),
                full_path: full_path.clone(),
                status,
                is_staged: true,
            });
        }

        // 2. 工作区未暂存变更 (Unstaged Changes)
        if x == '?' {
            unstaged.push(GitFileItem {
                rel_path: actual_path.to_string(),
                file_name: file_name.clone(),
                dir_name: dir_name.clone(),
                full_path: full_path.clone(),
                status: GitFileStatusKind::Untracked,
                is_staged: false,
            });
        } else if y != ' ' {
            let status = match y {
                'M' => GitFileStatusKind::Modified,
                'D' => GitFileStatusKind::Deleted,
                'U' => GitFileStatusKind::Conflicted,
                _ => GitFileStatusKind::Modified,
            };
            unstaged.push(GitFileItem {
                rel_path: actual_path.to_string(),
                file_name,
                dir_name,
                full_path,
                status,
                is_staged: false,
            });
        }
    }

    Ok(GitRepoStatus {
        is_repo: true,
        branch,
        staged: std::sync::Arc::new(staged),
        unstaged: std::sync::Arc::new(unstaged),
    })
}

/// 暂存单个文件 (git add -A -- <path>)
pub fn stage_file(dir: &str, rel_path: &str) -> Result<(), String> {
    let output = run_git_cmd(dir, &["add", "-A", "--", rel_path])?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// 暂存所有变更 (git add -A)
pub fn stage_all(dir: &str) -> Result<(), String> {
    let output = run_git_cmd(dir, &["add", "-A"])?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// 取消暂存单个文件 (git restore --staged -- <path>)
pub fn unstage_file(dir: &str, rel_path: &str) -> Result<(), String> {
    let output = run_git_cmd(dir, &["restore", "--staged", "--", rel_path]);
    if let Ok(out) = output {
        if out.status.success() {
            return Ok(());
        }
    }
    let output = run_git_cmd(dir, &["reset", "HEAD", "--", rel_path])?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// 取消暂存所有文件 (git restore --staged .)
pub fn unstage_all(dir: &str) -> Result<(), String> {
    let output = run_git_cmd(dir, &["restore", "--staged", "."]);
    if let Ok(out) = output {
        if out.status.success() {
            return Ok(());
        }
    }
    let output = run_git_cmd(dir, &["reset", "HEAD"])?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// 放弃工作区文件变更 (git restore / git clean)
pub fn discard_file(dir: &str, rel_path: &str, is_untracked: bool) -> Result<(), String> {
    if is_untracked {
        let output = run_git_cmd(dir, &["clean", "-f", "-d", "--", rel_path])?;
        if output.status.success() {
            return Ok(());
        }
        let _ = std::fs::remove_file(Path::new(dir).join(rel_path));
        return Ok(());
    }

    let output = run_git_cmd(dir, &["restore", "--", rel_path]);
    if let Ok(out) = output {
        if out.status.success() {
            return Ok(());
        }
    }
    let output = run_git_cmd(dir, &["checkout", "--", rel_path])?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// 放弃所有工作区变更 (git restore . && git clean -fd)
pub fn discard_all(dir: &str) -> Result<(), String> {
    let _ = run_git_cmd(dir, &["restore", "."]);
    let _ = run_git_cmd(dir, &["clean", "-fd"]);
    Ok(())
}

/// 提交已暂存的变更 (git commit -m <title> [-m <desc>])
pub fn commit(dir: &str, title: &str, description: &str) -> Result<String, String> {
    let title_trimmed = title.trim();
    if title_trimmed.is_empty() {
        return Err("提交标题不能为空".to_string());
    }

    let desc_trimmed = description.trim();
    let output = if desc_trimmed.is_empty() {
        run_git_cmd(dir, &["commit", "-m", title_trimmed])?
    } else {
        run_git_cmd(dir, &["commit", "-m", title_trimmed, "-m", desc_trimmed])?
    };

    if output.status.success() {
        let out_str = String::from_utf8_lossy(&output.stdout);
        Ok(out_str.trim().to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        let msg = if !err.trim().is_empty() {
            err.trim()
        } else {
            out.trim()
        };
        Err(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_init_and_status() {
        let tmp_dir = std::env::temp_dir().join(format!("zevent_git_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let dir_str = tmp_dir.to_str().unwrap();

        // 1. Initial state: not a repo
        assert!(!check_is_repo(dir_str));

        // 2. Initialize repo
        let init_res = init_repo(dir_str);
        assert!(init_res.is_ok(), "init_repo failed: {:?}", init_res);
        assert!(check_is_repo(dir_str));

        // 3. Create a test file
        let file_path = tmp_dir.join("test.txt");
        std::fs::write(&file_path, "hello git").unwrap();

        // 4. Status should show unstaged file
        let status = get_repo_status(dir_str).unwrap();
        assert!(status.is_repo);
        assert_eq!(status.unstaged.len(), 1);
        assert_eq!(status.unstaged[0].file_name, "test.txt");
        assert_eq!(status.unstaged[0].status, GitFileStatusKind::Untracked);
        assert_eq!(status.staged.len(), 0);

        // 5. Stage file
        let stage_res = stage_file(dir_str, "test.txt");
        assert!(stage_res.is_ok(), "stage_file failed: {:?}", stage_res);

        let status2 = get_repo_status(dir_str).unwrap();
        assert_eq!(status2.staged.len(), 1);
        assert_eq!(status2.staged[0].file_name, "test.txt");
        assert_eq!(status2.staged[0].status, GitFileStatusKind::Added);
        assert_eq!(status2.unstaged.len(), 0);

        // 6. Unstage file
        let unstage_res = unstage_file(dir_str, "test.txt");
        assert!(unstage_res.is_ok(), "unstage_file failed: {:?}", unstage_res);

        let status3 = get_repo_status(dir_str).unwrap();
        assert_eq!(status3.staged.len(), 0);
        assert_eq!(status3.unstaged.len(), 1);

        // 7. Discard file
        let discard_res = discard_file(dir_str, "test.txt", true);
        assert!(discard_res.is_ok(), "discard_file failed: {:?}", discard_res);
        assert!(!file_path.exists());

        let status4 = get_repo_status(dir_str).unwrap();
        assert_eq!(status4.staged.len(), 0);
        assert_eq!(status4.unstaged.len(), 0);

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
