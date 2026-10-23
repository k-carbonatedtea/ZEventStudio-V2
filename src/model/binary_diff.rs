use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::process::Command;
use roead::sarc::Sarc;
use crate::model::document_tab::SubFileDiffItem;
use crate::model::file_converter::parse_file_to_code;
use crate::model::file_tree::is_sarc_extension;
use crate::model::git::{parse_unified_diff, GitFileDiff, GitFileStatusKind};
use crate::model::git_diff_align::align_unified_diff;

/// 获取指定提交中某个文件的二进制字节流 (git show <hash>:<path>)
pub fn get_commit_file_bytes(dir: &str, commit_hash: &str, rel_path: &str) -> Result<Vec<u8>, String> {
    let norm_path = rel_path.replace('\\', "/");
    let spec = format!("{}:{}", commit_hash, norm_path);
    let mut cmd = Command::new("git");
    cmd.arg("-c").arg("core.quotepath=false");
    cmd.args(["show", &spec]);
    cmd.current_dir(dir);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let output = cmd.output().map_err(|e| format!("执行 Git 命令失败: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(output.stdout)
}

/// 获取工作区或暂存区中某个文件的二进制字节流
pub fn get_working_tree_file_bytes(dir: &str, rel_path: &str, is_staged: bool) -> Result<Vec<u8>, String> {
    let norm_path = rel_path.replace('\\', "/");
    if is_staged {
        let spec = format!(":{}", norm_path);
        let mut cmd = Command::new("git");
        cmd.arg("-c").arg("core.quotepath=false");
        cmd.args(["show", &spec]);
        cmd.current_dir(dir);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let output = cmd.output().map_err(|e| format!("执行 Git 命令失败: {}", e))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }
        Ok(output.stdout)
    } else {
        let full_path = Path::new(dir).join(&norm_path);
        std::fs::read(&full_path).map_err(|e| format!("读取工作区文件失败: {}", e))
    }
}

/// 递归解包 SARC 归档内部所有文件 (支持最多 4 层嵌套 SARC 展开)
fn unpack_sarc_files_recursive(
    data: &[u8],
    prefix: &str,
    depth: usize,
    max_depth: usize,
    map: &mut HashMap<String, Vec<u8>>,
) -> bool {
    let decompressed = roead::yaz0::decompress(data).unwrap_or_else(|_| data.to_vec());
    let sarc = match Sarc::new(&decompressed) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let files: Vec<(String, Vec<u8>)> = sarc
        .files()
        .filter_map(|f| f.name().map(|n| (n.to_string(), f.data().to_vec())))
        .collect();

    if files.is_empty() {
        return false;
    }

    for (name, file_data) in files {
        let full_path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", prefix, name)
        };

        let is_nested = is_sarc_extension(&name);
        if is_nested && depth < max_depth {
            let unpacked = unpack_sarc_files_recursive(
                &file_data,
                &full_path,
                depth + 1,
                max_depth,
                map,
            );
            if !unpacked {
                map.insert(full_path, file_data);
            }
        } else {
            map.insert(full_path, file_data);
        }
    }

    true
}

/// 解包 SARC 归档，提取内部所有文件的路径与原始字节流 (支持多层嵌套 SARC 递归展开)
pub fn unpack_sarc_files(data: &[u8]) -> Option<HashMap<String, Vec<u8>>> {
    let mut map = HashMap::new();
    if unpack_sarc_files_recursive(data, "", 0, 4, &mut map) {
        Some(map)
    } else {
        None
    }
}

/// 计算两份文本字符串之间的 Unified Diff (基于 git diff --no-index 算法引擎)
pub fn diff_two_texts(old_text: &str, new_text: &str, file_name: &str) -> Result<GitFileDiff, String> {
    let temp_root = std::env::temp_dir().join(format!("zevent_diff_{}", uuid::Uuid::new_v4()));
    let dir_a = temp_root.join("a");
    let dir_b = temp_root.join("b");

    let clean_file_name = Path::new(file_name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("diff_target.txt");
    let file_a = dir_a.join(clean_file_name);
    let file_b = dir_b.join(clean_file_name);

    let _ = std::fs::create_dir_all(&dir_a);
    let _ = std::fs::create_dir_all(&dir_b);
    let _ = std::fs::write(&file_a, old_text);
    let _ = std::fs::write(&file_b, new_text);

    let mut cmd = Command::new("git");
    cmd.arg("-c").arg("core.quotepath=false");
    cmd.args([
        "diff",
        "--no-index",
        "--",
        &file_a.to_string_lossy(),
        &file_b.to_string_lossy(),
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let output = cmd.output();

    let _ = std::fs::remove_dir_all(&temp_root);

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            Ok(parse_unified_diff(&stdout, file_name))
        }
        Err(e) => Err(format!("计算文本差异失败: {}", e)),
    }
}

/// 智能解析二进制或 SARC 封包文件的内部代码差异 (把二进制用代码编辑方法解析出来)
pub fn resolve_smart_code_diff(
    old_bytes_opt: Option<&[u8]>,
    new_bytes_opt: Option<&[u8]>,
    file_path: &str,
) -> Option<Vec<SubFileDiffItem>> {
    let lower_path = file_path.to_lowercase();

    // 1. 若为 SARC 归档封包 (如 .sbactorpack / .sbeventpack / .pack)
    if is_sarc_extension(&lower_path) {
        let old_files = old_bytes_opt.and_then(unpack_sarc_files).unwrap_or_default();
        let new_files = new_bytes_opt.and_then(unpack_sarc_files).unwrap_or_default();

        let mut all_keys: BTreeSet<String> = BTreeSet::new();
        for k in old_files.keys() {
            all_keys.insert(k.clone());
        }
        for k in new_files.keys() {
            all_keys.insert(k.clone());
        }

        let mut sub_diffs = Vec::new();

        for key in all_keys {
            let old_data = old_files.get(&key);
            let new_data = new_files.get(&key);

            // 比较二进制是否相同，相同则无变动跳过
            if old_data == new_data {
                continue;
            }

            // 尝试将内部文件反编译解析为代码 (AAMP / BYML / MSBT / BFEVFL 等)
            let old_code_res = old_data.and_then(|d| parse_file_to_code(d, &key).ok());
            let new_code_res = new_data.and_then(|d| parse_file_to_code(d, &key).ok());

            let (old_text, new_text, format_name) = match (old_code_res, new_code_res) {
                (Some((ot, ofmt, _)), Some((nt, _, _))) => {
                    if ot == nt {
                        continue;
                    }
                    (ot, nt, ofmt.display_name().to_string())
                }
                (Some((ot, ofmt, _)), None) => (ot, String::new(), ofmt.display_name().to_string()),
                (None, Some((nt, nfmt, _))) => (String::new(), nt, nfmt.display_name().to_string()),
                (None, None) => {
                    let old_size = old_data.map(|d| d.len()).unwrap_or(0);
                    let new_size = new_data.map(|d| d.len()).unwrap_or(0);
                    let bin_old = format!("# 二进制文件 (旧版本): {} ({} 字节)", key, old_size);
                    let bin_new = format!("# 二进制文件 (新版本): {} ({} 字节)", key, new_size);
                    (bin_old, bin_new, "Binary".to_string())
                }
            };

            let clean_name = Path::new(&key)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&key);

            if let Ok(diff) = diff_two_texts(&old_text, &new_text, clean_name) {
                if !diff.lines.is_empty() {
                    let rows = align_unified_diff(&diff);
                    sub_diffs.push(SubFileDiffItem {
                        sub_path: key,
                        format_name,
                        diff,
                        rows,
                    });
                }
            }
        }

        if !sub_diffs.is_empty() {
            return Some(sub_diffs);
        }
    }

    // 2. 若为独立的单二进制文件 (如 .byml, .bgyml, .aamp, .bfevfl, .msbt 等)
    let old_code_res = old_bytes_opt.and_then(|d| parse_file_to_code(d, file_path).ok());
    let new_code_res = new_bytes_opt.and_then(|d| parse_file_to_code(d, file_path).ok());

    if old_code_res.is_some() || new_code_res.is_some() {
        let format_name = new_code_res
            .as_ref()
            .map(|(_, fmt, _)| fmt.display_name())
            .or_else(|| old_code_res.as_ref().map(|(_, fmt, _)| fmt.display_name()))
            .unwrap_or("Code")
            .to_string();

        let old_text = old_code_res.map(|(t, _, _)| t).unwrap_or_default();
        let new_text = new_code_res.map(|(t, _, _)| t).unwrap_or_default();
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(file_path);

        if let Ok(diff) = diff_two_texts(&old_text, &new_text, file_name) {
            let rows = align_unified_diff(&diff);
            return Some(vec![SubFileDiffItem {
                sub_path: file_name.to_string(),
                format_name,
                diff,
                rows,
            }]);
        }
    }

    None
}

/// SARC 归档封包内部子文件单项变更
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SarcSubFileChange {
    pub sub_path: String,
    pub file_name: String,
    pub dir_name: String,
    pub format_name: String,
    pub status: GitFileStatusKind,
}

impl SarcSubFileChange {
    /// 获取子文件对应的图标符号
    pub fn icon(&self) -> &'static str {
        let lower = self.file_name.to_lowercase();
        if lower.ends_with(".bfevfl") {
            "⚡"
        } else if lower.ends_with(".msbt") {
            "💬"
        } else if [".bxml", ".baev", ".baischedule", ".genvb", ".bactparam"].iter().any(|ext| lower.ends_with(ext)) {
            "⚙️"
        } else if [".byml", ".bgyml", ".bgdata", ".sbgdata", ".sbquestpack"].iter().any(|ext| lower.ends_with(ext)) {
            "{ }"
        } else {
            "📄"
        }
    }
}

/// 提取并比对 SARC 封包前后两版本的差异子文件列表
pub fn get_sarc_changed_subfiles(
    old_bytes_opt: Option<&[u8]>,
    new_bytes_opt: Option<&[u8]>,
) -> Vec<SarcSubFileChange> {
    let old_files = old_bytes_opt.and_then(unpack_sarc_files).unwrap_or_default();
    let new_files = new_bytes_opt.and_then(unpack_sarc_files).unwrap_or_default();

    let mut all_keys: BTreeSet<String> = BTreeSet::new();
    for k in old_files.keys() {
        all_keys.insert(k.clone());
    }
    for k in new_files.keys() {
        all_keys.insert(k.clone());
    }

    let mut changes = Vec::new();

    for key in all_keys {
        let old_data = old_files.get(&key);
        let new_data = new_files.get(&key);

        if old_data == new_data {
            continue;
        }

        // 若旧数据与新数据皆存在，且两者经反编译为规范代码后完全一致（如忽略画布排版坐标后的事件流），则在逻辑上视为无变动
        if let (Some(od), Some(nd)) = (old_data, new_data) {
            if let (Ok((ot, _, _)), Ok((nt, _, _))) = (parse_file_to_code(od, &key), parse_file_to_code(nd, &key)) {
                if ot == nt {
                    continue;
                }
            }
        }

        let status = match (old_data.is_some(), new_data.is_some()) {
            (true, true) => GitFileStatusKind::Modified,
            (false, true) => GitFileStatusKind::Added,
            (true, false) => GitFileStatusKind::Deleted,
            (false, false) => continue,
        };

        let norm_key = key.replace('\\', "/");
        let (dir_name, file_name) = match norm_key.rfind('/') {
            Some(idx) => (norm_key[..idx].to_string(), norm_key[idx + 1..].to_string()),
            None => (String::new(), norm_key.clone()),
        };

        let format_name = new_data
            .or(old_data)
            .and_then(|d| parse_file_to_code(d, &key).ok())
            .map(|(_, fmt, _)| fmt.display_name().to_string())
            .unwrap_or_else(|| {
                let lk = key.to_lowercase();
                if [".bxml", ".baev", ".baischedule", ".genvb", ".bactparam"].iter().any(|ext| lk.ends_with(ext)) {
                    "AAMP".to_string()
                } else if [".byml", ".bgyml", ".bgdata", ".sbgdata", ".bquestpack", ".sbquestpack"].iter().any(|ext| lk.ends_with(ext)) {
                    "BYML".to_string()
                } else if lk.ends_with(".bfevfl") {
                    "BFEVFL".to_string()
                } else if lk.ends_with(".msbt") {
                    "MSBT".to_string()
                } else {
                    "Binary".to_string()
                }
            });

        changes.push(SarcSubFileChange {
            sub_path: norm_key,
            file_name,
            dir_name,
            format_name,
            status,
        });
    }

    changes
}

