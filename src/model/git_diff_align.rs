use crate::model::git::{GitDiffLine, GitDiffLineType, GitFileDiff};

/// 差异单元格种类
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffCellKind {
    /// 正常未修改上下文
    Context,
    /// 新增代码行
    Addition,
    /// 删除代码行
    Deletion,
    /// 占位区域（对应侧有代码，本侧无代码，绘制斜纹对角阴影）
    Placeholder,
}

/// 双栏对比中的单侧单元格
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffSideCell {
    /// 对应真实行号（占位单元格为 None）
    pub line_num: Option<usize>,
    /// 行号显示标签（如 "529+"、"528-"、"511" 或 ""）
    pub line_label: String,
    /// 行前缀（"+"、"-" 或 " "）
    pub prefix: &'static str,
    /// 代码文本内容
    pub text: String,
    /// 单元格种类
    pub kind: DiffCellKind,
}

impl DiffSideCell {
    /// 创建占位对角阴影单元格
    pub fn placeholder() -> Self {
        Self {
            line_num: None,
            line_label: String::new(),
            prefix: "",
            text: String::new(),
            kind: DiffCellKind::Placeholder,
        }
    }

    /// 创建上下文单元格
    pub fn context(line_num: Option<usize>, text: String) -> Self {
        let line_label = line_num.map(|n| n.to_string()).unwrap_or_default();
        Self {
            line_num,
            line_label,
            prefix: " ",
            text,
            kind: DiffCellKind::Context,
        }
    }

    /// 创建删除行单元格 (左栏)
    pub fn deletion(line_num: Option<usize>, text: String) -> Self {
        let line_label = line_num.map(|n| format!("{}-", n)).unwrap_or_default();
        Self {
            line_num,
            line_label,
            prefix: "-",
            text,
            kind: DiffCellKind::Deletion,
        }
    }

    /// 创建新增行单元格 (右栏，图2中如 529+)
    pub fn addition(line_num: Option<usize>, text: String) -> Self {
        let line_label = line_num.map(|n| format!("{}+", n)).unwrap_or_default();
        Self {
            line_num,
            line_label,
            prefix: "+",
            text,
            kind: DiffCellKind::Addition,
        }
    }
}

/// Side-by-Side 双栏对比中的渲染行
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SideBySideRow {
    /// Hunk 头部信息（跨越两栏的分隔条）
    HunkHeader {
        /// Hunk 原始文本，如 "@@ -511,18 +511,25 @@ pub fn get_en..."
        header_text: String,
        /// 函数或代码块上下文说明
        context_hint: String,
        /// 旧版本起始行
        old_start: usize,
        /// 新版本起始行
        new_start: usize,
    },
    /// 对齐的代码行对（左栏与右栏）
    LinePair {
        left: DiffSideCell,
        right: DiffSideCell,
    },
}

/// 将 GitFileDiff 的一维 Unified Diff 转换为对齐的双栏行结构
pub fn align_unified_diff(diff: &GitFileDiff) -> Vec<SideBySideRow> {
    let mut rows = Vec::new();
    let mut pending_deletions: Vec<GitDiffLine> = Vec::new();
    let mut pending_additions: Vec<GitDiffLine> = Vec::new();

    for line in &diff.lines {
        match line.line_type {
            GitDiffLineType::Header => {
                flush_diff_block(&mut pending_deletions, &mut pending_additions, &mut rows);

                let header_raw = line.text.clone();
                let (old_start, new_start, context_hint) = parse_hunk_header(&header_raw);

                rows.push(SideBySideRow::HunkHeader {
                    header_text: header_raw,
                    context_hint,
                    old_start,
                    new_start,
                });
            }
            GitDiffLineType::Context => {
                flush_diff_block(&mut pending_deletions, &mut pending_additions, &mut rows);

                rows.push(SideBySideRow::LinePair {
                    left: DiffSideCell::context(line.old_line_num, line.text.clone()),
                    right: DiffSideCell::context(line.new_line_num, line.text.clone()),
                });
            }
            GitDiffLineType::Deletion => {
                pending_deletions.push(line.clone());
            }
            GitDiffLineType::Addition => {
                pending_additions.push(line.clone());
            }
        }
    }

    flush_diff_block(&mut pending_deletions, &mut pending_additions, &mut rows);
    rows
}

/// 刷新并对齐一组变更块（将若干删除行和新增行对齐填充占位单元格）
fn flush_diff_block(
    deletions: &mut Vec<GitDiffLine>,
    additions: &mut Vec<GitDiffLine>,
    rows: &mut Vec<SideBySideRow>,
) {
    if deletions.is_empty() && additions.is_empty() {
        return;
    }

    let del_count = deletions.len();
    let add_count = additions.len();
    let max_len = del_count.max(add_count);

    for i in 0..max_len {
        let left = if i < del_count {
            let item = &deletions[i];
            DiffSideCell::deletion(item.old_line_num, item.text.clone())
        } else {
            DiffSideCell::placeholder()
        };

        let right = if i < add_count {
            let item = &additions[i];
            DiffSideCell::addition(item.new_line_num, item.text.clone())
        } else {
            DiffSideCell::placeholder()
        };

        rows.push(SideBySideRow::LinePair { left, right });
    }

    deletions.clear();
    additions.clear();
}

/// 解析 Hunk 头部，提取起始行号与上下文提示
fn parse_hunk_header(header: &str) -> (usize, usize, String) {
    let mut old_start = 1;
    let mut new_start = 1;
    let mut context_hint = String::new();

    if let Some(pos) = header.find('-') {
        let rest = &header[pos + 1..];
        if let Some(comma_or_space) = rest.find([',', ' ']) {
            if let Ok(n) = rest[..comma_or_space].parse::<usize>() {
                old_start = n;
            }
        }
    }

    if let Some(pos) = header.find('+') {
        let rest = &header[pos + 1..];
        if let Some(comma_or_space) = rest.find([',', ' ']) {
            if let Ok(n) = rest[..comma_or_space].parse::<usize>() {
                new_start = n;
            }
        }
    }

    // 提取 @@ ... @@ 后面的函数声明或注释
    if let Some(first_at) = header.find("@@") {
        if let Some(second_at) = header[first_at + 2..].find("@@") {
            let after = &header[first_at + 2 + second_at + 2..];
            context_hint = after.trim().to_string();
        }
    }

    (old_start, new_start, context_hint)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_pure_addition() {
        let diff = GitFileDiff {
            file_path: "test.rs".to_string(),
            lines: vec![
                GitDiffLine {
                    line_type: GitDiffLineType::Context,
                    old_line_num: Some(1),
                    new_line_num: Some(1),
                    text: "line1".to_string(),
                },
                GitDiffLine {
                    line_type: GitDiffLineType::Addition,
                    old_line_num: None,
                    new_line_num: Some(2),
                    text: "line2_new".to_string(),
                },
                GitDiffLine {
                    line_type: GitDiffLineType::Context,
                    old_line_num: Some(2),
                    new_line_num: Some(3),
                    text: "line3".to_string(),
                },
            ],
            additions: 1,
            deletions: 0,
        };

        let rows = align_unified_diff(&diff);
        assert_eq!(rows.len(), 3);

        match &rows[1] {
            SideBySideRow::LinePair { left, right } => {
                assert_eq!(left.kind, DiffCellKind::Placeholder);
                assert_eq!(right.kind, DiffCellKind::Addition);
                assert_eq!(right.line_label, "2+");
                assert_eq!(right.text, "line2_new");
            }
            _ => panic!("Expected LinePair"),
        }
    }
}
