pub mod types;
pub mod ops;
pub mod diff;

pub use types::*;
pub use ops::*;
pub use diff::*;
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