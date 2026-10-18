
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
