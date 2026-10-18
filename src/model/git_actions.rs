use std::process::Command;

/// 执行特定提交的还原撤销 (git revert <commit_hash> --no-edit)
pub fn revert_commit(repo_dir: &str, commit_hash: &str) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.arg("-c").arg("core.quotepath=false");
    cmd.args(["revert", commit_hash, "--no-edit"]);
    cmd.current_dir(repo_dir);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let output = cmd.output().map_err(|e| format!("执行 Git 命令失败: {}", e))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(if err.trim().is_empty() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            err.trim().to_string()
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// 重置当前分支 HEAD 至指定提交 (git reset <mode> <commit_hash>)
/// mode: "--soft" (保留暂存区与工作区), "--mixed" (撤销暂存区，保留工作区), "--hard" (放弃所有修改)
pub fn reset_to_commit(repo_dir: &str, commit_hash: &str, mode: &str) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.arg("-c").arg("core.quotepath=false");
    cmd.args(["reset", mode, commit_hash]);
    cmd.current_dir(repo_dir);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let output = cmd.output().map_err(|e| format!("执行 Git 命令失败: {}", e))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(if err.trim().is_empty() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            err.trim().to_string()
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// 检出指定提交 (git checkout <commit_hash>)
pub fn checkout_commit(repo_dir: &str, commit_hash: &str) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.arg("-c").arg("core.quotepath=false");
    cmd.args(["checkout", commit_hash]);
    cmd.current_dir(repo_dir);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let output = cmd.output().map_err(|e| format!("执行 Git 命令失败: {}", e))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(if err.trim().is_empty() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            err.trim().to_string()
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
