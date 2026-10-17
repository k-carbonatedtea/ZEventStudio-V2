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

/// Git 提交记录条目
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitCommitItem {
    pub hash: String,
    pub short_hash: String,
    pub parents: Vec<String>,
    pub author: String,
    pub author_email: String,
    pub relative_time: String,
    pub subject: String,
    pub refs: Vec<String>,
    pub graph_prefix: String,
}

/// 某次提交中发生变更的文件
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitCommitFileChange {
    pub status: GitFileStatusKind,
    pub path: String,
    pub old_path: Option<String>,
}

/// 代码差异行类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitDiffLineType {
    Context,
    Addition,
    Deletion,
    Header,
}

/// 单行差异信息
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitDiffLine {
    pub line_type: GitDiffLineType,
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
    pub text: String,
}

/// 单个文件的完整差异对比数据
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GitFileDiff {
    pub file_path: String,
    pub lines: Vec<GitDiffLine>,
    pub additions: usize,
    pub deletions: usize,
}

fn run_git_cmd(repo_dir: &str, args: &[&str]) -> Result<std::process::Output, String> {
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

/// 解码 Git 路径中可能包含的八进制转义序列（例如 "\346\266\205"）及 C 风格转义字符
pub fn unquote_git_path(raw: &str) -> String {
    let s = raw.trim();
    if !s.starts_with('"') || !s.ends_with('"') || s.len() < 2 {
        return s.to_string();
    }
    let inner = &s[1..s.len() - 1];
    let bytes = inner.as_bytes();
    let mut out_bytes = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 1;
            match bytes[i] {
                b'\\' => {
                    out_bytes.push(b'\\');
                    i += 1;
                }
                b'"' => {
                    out_bytes.push(b'"');
                    i += 1;
                }
                b'n' => {
                    out_bytes.push(b'\n');
                    i += 1;
                }
                b't' => {
                    out_bytes.push(b'\t');
                    i += 1;
                }
                b'r' => {
                    out_bytes.push(b'\r');
                    i += 1;
                }
                b'a' => {
                    out_bytes.push(0x07);
                    i += 1;
                }
                b'b' => {
                    out_bytes.push(0x08);
                    i += 1;
                }
                b'v' => {
                    out_bytes.push(0x0B);
                    i += 1;
                }
                b'f' => {
                    out_bytes.push(0x0C);
                    i += 1;
                }
                b'0'..=b'7' => {
                    let mut val = bytes[i] - b'0';
                    i += 1;
                    for _ in 0..2 {
                        if i < bytes.len() && (b'0'..=b'7').contains(&bytes[i]) {
                            val = (val << 3) | (bytes[i] - b'0');
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    out_bytes.push(val);
                }
                other => {
                    out_bytes.push(b'\\');
                    out_bytes.push(other);
                    i += 1;
                }
            }
        } else {
            out_bytes.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out_bytes).to_string()
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

/// 解析统一差异格式 (Unified Diff) 输出
pub fn parse_unified_diff(diff_output: &str, file_path: &str) -> GitFileDiff {
    let mut lines = Vec::new();
    let mut additions = 0;
    let mut deletions = 0;
    let mut old_num = 1;
    let mut new_num = 1;

    for raw_line in diff_output.lines() {
        if raw_line.starts_with("diff --git")
            || raw_line.starts_with("index ")
            || raw_line.starts_with("--- ")
            || raw_line.starts_with("+++ ")
        {
            continue;
        }

        if raw_line.starts_with("@@") {
            // 解析 Hunk 头部，提取起始行号例如 @@ -12,4 +15,6 @@
            if let Some(pos) = raw_line.find('-') {
                let rest = &raw_line[pos + 1..];
                if let Some(comma_or_space) = rest.find([',', ' ']) {
                    if let Ok(n) = rest[..comma_or_space].parse::<usize>() {
                        old_num = n;
                    }
                }
            }
            if let Some(pos) = raw_line.find('+') {
                let rest = &raw_line[pos + 1..];
                if let Some(comma_or_space) = rest.find([',', ' ']) {
                    if let Ok(n) = rest[..comma_or_space].parse::<usize>() {
                        new_num = n;
                    }
                }
            }
            lines.push(GitDiffLine {
                line_type: GitDiffLineType::Header,
                old_line_num: None,
                new_line_num: None,
                text: raw_line.to_string(),
            });
            continue;
        }

        if let Some(stripped) = raw_line.strip_prefix('+') {
            additions += 1;
            lines.push(GitDiffLine {
                line_type: GitDiffLineType::Addition,
                old_line_num: None,
                new_line_num: Some(new_num),
                text: stripped.to_string(),
            });
            new_num += 1;
        } else if let Some(stripped) = raw_line.strip_prefix('-') {
            deletions += 1;
            lines.push(GitDiffLine {
                line_type: GitDiffLineType::Deletion,
                old_line_num: Some(old_num),
                new_line_num: None,
                text: stripped.to_string(),
            });
            old_num += 1;
        } else if let Some(stripped) = raw_line.strip_prefix(' ') {
            lines.push(GitDiffLine {
                line_type: GitDiffLineType::Context,
                old_line_num: Some(old_num),
                new_line_num: Some(new_num),
                text: stripped.to_string(),
            });
            old_num += 1;
            new_num += 1;
        } else if raw_line.starts_with('\\') {
            // \ No newline at end of file
            lines.push(GitDiffLine {
                line_type: GitDiffLineType::Context,
                old_line_num: None,
                new_line_num: None,
                text: raw_line.to_string(),
            });
        } else {
            lines.push(GitDiffLine {
                line_type: GitDiffLineType::Context,
                old_line_num: Some(old_num),
                new_line_num: Some(new_num),
                text: raw_line.to_string(),
            });
            old_num += 1;
            new_num += 1;
        }
    }

    GitFileDiff {
        file_path: file_path.to_string(),
        lines,
        additions,
        deletions,
    }
}

/// 获取提交历史与分支图谱 (git log --graph)
pub fn get_commit_history(dir: &str, max_count: usize) -> Result<Vec<GitCommitItem>, String> {
    let count_str = max_count.to_string();
    let output = run_git_cmd(
        dir,
        &[
            "log",
            "-n",
            &count_str,
            "--graph",
            "--date=relative",
            "--pretty=format:__COMMIT__%H\x1f%h\x1f%P\x1f%an\x1f%ar\x1f%s\x1f%D",
        ],
    )?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut items = Vec::new();

    for line in stdout.lines() {
        if let Some(pos) = line.find("__COMMIT__") {
            let graph_part = line[..pos].trim_end();
            let commit_part = &line[pos + "__COMMIT__".len()..];
            let fields: Vec<&str> = commit_part.split('\x1f').collect();
            if fields.len() >= 6 {
                let hash = fields[0].trim().to_string();
                let short_hash = fields[1].trim().to_string();
                let parents = fields[2]
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                let author = fields[3].trim().to_string();
                let relative_time = fields[4].trim().to_string();
                let subject = fields[5].trim().to_string();
                let refs_str = fields.get(6).unwrap_or(&"").trim();
                let refs = if !refs_str.is_empty() {
                    refs_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else {
                    Vec::new()
                };

                let graph_prefix = if graph_part.is_empty() {
                    "*".to_string()
                } else {
                    graph_part.to_string()
                };

                items.push(GitCommitItem {
                    hash,
                    short_hash,
                    parents,
                    author,
                    author_email: String::new(),
                    relative_time,
                    subject,
                    refs,
                    graph_prefix,
                });
            }
        }
    }

    Ok(items)
}

/// 获取指定提交中修改的文件列表 (git show --name-status)
pub fn get_commit_files(dir: &str, commit_hash: &str) -> Result<Vec<GitCommitFileChange>, String> {
    let output = run_git_cmd(
        dir,
        &["show", "--name-status", "--format=", commit_hash],
    )?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(err.trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut list = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split('\t').collect();
        if parts.len() >= 2 {
            let status_char = parts[0].chars().next().unwrap_or('M');
            let status = match status_char {
                'A' => GitFileStatusKind::Added,
                'D' => GitFileStatusKind::Deleted,
                'R' => GitFileStatusKind::Renamed,
                'U' => GitFileStatusKind::Untracked,
                _ => GitFileStatusKind::Modified,
            };
            if status == GitFileStatusKind::Renamed && parts.len() >= 3 {
                list.push(GitCommitFileChange {
                    status,
                    path: unquote_git_path(parts[2]).replace('\\', "/"),
                    old_path: Some(unquote_git_path(parts[1]).replace('\\', "/")),
                });
            } else {
                list.push(GitCommitFileChange {
                    status,
                    path: unquote_git_path(parts[1]).replace('\\', "/"),
                    old_path: None,
                });
            }
        }
    }

    Ok(list)
}

/// 获取指定提交中某个文件的差异代码 (git show <hash> -- <path>)
pub fn get_commit_file_diff(dir: &str, commit_hash: &str, file_path: &str) -> Result<GitFileDiff, String> {
    let norm_path = file_path.replace('\\', "/");
    let output = run_git_cmd(
        dir,
        &["show", "--format=", commit_hash, "--", &norm_path],
    )?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(err.trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_unified_diff(&stdout, &norm_path))
}

/// 获取当前工作区或暂存区中某个文件的修改差异
pub fn get_working_tree_file_diff(dir: &str, file_path: &str, is_staged: bool) -> Result<GitFileDiff, String> {
    let norm_path = file_path.replace('\\', "/");
    let output = if is_staged {
        run_git_cmd(dir, &["diff", "--cached", "--", &norm_path])?
    } else {
        run_git_cmd(dir, &["diff", "--", &norm_path])?
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        return Ok(parse_unified_diff(&stdout, &norm_path));
    }

    // 若 Git diff 为空，但文件是新添加或未跟踪的，直接读取文件内容展示全行新增
    let full_path = Path::new(dir).join(&norm_path);
    if full_path.exists() && full_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&full_path) {
            let mut lines = Vec::new();
            let mut additions = 0;
            for (idx, line) in content.lines().enumerate() {
                additions += 1;
                lines.push(GitDiffLine {
                    line_type: GitDiffLineType::Addition,
                    old_line_num: None,
                    new_line_num: Some(idx + 1),
                    text: line.to_string(),
                });
            }
            return Ok(GitFileDiff {
                file_path: norm_path,
                lines,
                additions,
                deletions: 0,
            });
        }
    }

    Ok(parse_unified_diff(&stdout, &norm_path))
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

    #[test]
    fn test_unquote_git_path() {
        assert_eq!(unquote_git_path("simple.txt"), "simple.txt");
        assert_eq!(unquote_git_path("\"simple.txt\""), "simple.txt");
        assert_eq!(unquote_git_path("\"path with spaces/file.txt\""), "path with spaces/file.txt");
        // "\346\266\205\346\231\256\351\241\277" is "涅普顿"
        let quoted_chinese = "\"0187_\\346\\266\\205\\346\\231\\256\\351\\241\\277/file.txt\"";
        assert_eq!(unquote_git_path(quoted_chinese), "0187_涅普顿/file.txt");
    }
}
