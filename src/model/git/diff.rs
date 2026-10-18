use std::path::Path;
use super::types::*;
use super::ops::run_git_cmd;
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
