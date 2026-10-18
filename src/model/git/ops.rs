use std::path::Path;
use std::process::Command;
use super::types::*;
use super::diff::unquote_git_path;
pub(crate) fn run_git_cmd(repo_dir: &str, args: &[&str]) -> Result<std::process::Output, String> {
    let mut cmd = Command::new("git");
    cmd.arg("-c").arg("core.quotepath=false");
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

        let mut chars = line.chars();
        let x = chars.next().unwrap_or(' ');
        let y = chars.next().unwrap_or(' ');
        let raw_path = line[3..].trim();
        let actual_path = if let Some((_, new_p)) = raw_path.split_once(" -> ") {
            unquote_git_path(new_p.trim())
        } else {
            unquote_git_path(raw_path)
        };
        let actual_path = actual_path.replace('\\', "/");

        let path_obj = Path::new(&actual_path);
        let file_name = path_obj
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&actual_path)
            .to_string();
        let dir_name = path_obj
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .replace('\\', "/");
        let full_path = root_path.join(&actual_path).to_string_lossy().to_string();

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
    let norm_path = rel_path.replace('\\', "/");
    let output = run_git_cmd(dir, &["add", "-A", "--", &norm_path])?;
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
    let norm_path = rel_path.replace('\\', "/");
    let output = run_git_cmd(dir, &["restore", "--staged", "--", &norm_path]);
    if let Ok(out) = output {
        if out.status.success() {
            return Ok(());
        }
    }
    let output = run_git_cmd(dir, &["reset", "HEAD", "--", &norm_path])?;
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
    let norm_path = rel_path.replace('\\', "/");
    if is_untracked {
        let output = run_git_cmd(dir, &["clean", "-f", "-d", "--", &norm_path])?;
        if output.status.success() {
            return Ok(());
        }
        let p = Path::new(dir).join(&norm_path);
        if p.is_dir() {
            let _ = std::fs::remove_dir_all(&p);
        } else {
            let _ = std::fs::remove_file(&p);
        }
        return Ok(());
    }

    let output = run_git_cmd(dir, &["restore", "--", &norm_path]);
    if let Ok(out) = output {
        if out.status.success() {
            return Ok(());
        }
    }
    let output = run_git_cmd(dir, &["checkout", "--", &norm_path])?;
    if output.status.success() {
        Ok(())
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

/// 放弃所有工作区变更 (git restore . && git clean -fd)
pub fn discard_all(dir: &str) -> Result<(), String> {
    let _ = run_git_cmd(dir, &["restore", "."]);
    let _ = run_git_cmd(dir, &["checkout", "--", "."]);
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
