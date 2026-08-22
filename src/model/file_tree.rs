use rayon::prelude::*;
use roead::sarc::Sarc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 文件树节点定义 (支持物理文件夹、SARC 封包及叶子文件)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileNode {
    /// 节点显示名称
    pub name: String,
    /// 完整路径 (物理绝对路径或 SARC:pack//inner 虚拟路径)
    pub path: String,
    /// 是否为目录文件夹
    pub is_dir: bool,
    /// 是否为 SARC / Yaz0 归档封包 (如 .sbeventpack / .sbactorpack / .pack)
    pub is_sarc: bool,
    /// 子节点列表
    pub children: Option<Vec<FileNode>>,
}

/// 检查路径或扩展名是否属于 SARC 压缩包格式
pub fn is_sarc_extension(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".sbeventpack")
        || lower.ends_with(".sbactorpack")
        || lower.ends_with(".pack")
        || lower.ends_with(".sarc")
        || lower.ends_with(".ssarc")
        || lower.ends_with(".spack")
        || lower.ends_with(".bactorpack")
}

/// 从内存字节流中解析 SARC 内部文件树结构（支持多层嵌套解压）
pub fn get_sarc_nodes(data: &[u8], base_path: &str) -> Vec<FileNode> {
    #[derive(Default)]
    struct TempNode {
        children: HashMap<String, TempNode>,
        file_info: Option<(String, bool, Option<Vec<FileNode>>)>,
    }

    let mut root = TempNode::default();
    let decompressed = roead::yaz0::decompress(data).unwrap_or_else(|_| data.to_vec());

    if let Ok(sarc) = Sarc::new(&decompressed) {
        let file_list: Vec<(String, Vec<u8>)> = sarc
            .files()
            .filter_map(|f| f.name().map(|name| (name.to_string(), f.data().to_vec())))
            .collect();

        for (inner_name, file_bytes) in file_list {
            let parts: Vec<&str> = inner_name.split('/').collect();
            let mut current = &mut root;

            for (i, part) in parts.iter().enumerate() {
                let is_last = i == parts.len() - 1;
                if is_last {
                    let is_nested_sarc = is_sarc_extension(part);
                    let nested_children = if is_nested_sarc {
                        Some(get_sarc_nodes(&file_bytes, &format!("{}//{}", base_path, inner_name)))
                    } else {
                        None
                    };

                    let entry = current.children.entry(part.to_string()).or_default();
                    entry.file_info = Some((
                        format!("{}//{}", base_path, inner_name),
                        is_nested_sarc,
                        nested_children,
                    ));
                } else {
                    current = current.children.entry(part.to_string()).or_default();
                }
            }
        }
    }

    fn convert(temp: TempNode, current_virtual_dir: &str, base_path: &str) -> Vec<FileNode> {
        let mut nodes = Vec::new();
        for (name, child) in temp.children {
            if let Some((path, is_sarc, children)) = child.file_info {
                nodes.push(FileNode {
                    name,
                    path,
                    is_dir: false,
                    is_sarc,
                    children,
                });
            } else {
                let dir_virtual_path = if current_virtual_dir.is_empty() {
                    format!("{}//{}", base_path, name)
                } else {
                    format!("{}/{}", current_virtual_dir, name)
                };
                let mut dir_children = convert(child, &dir_virtual_path, base_path);
                sort_file_nodes(&mut dir_children);
                nodes.push(FileNode {
                    name,
                    path: dir_virtual_path,
                    is_dir: true,
                    is_sarc: false,
                    children: Some(dir_children),
                });
            }
        }
        sort_file_nodes(&mut nodes);
        nodes
    }

    convert(root, "", base_path)
}

/// 递归扫描物理目录，自动解构内部 SARC 封包并返回排序后的文件树
pub fn read_dir_recursive(dir_path: &Path) -> Result<Vec<FileNode>, String> {
    if !dir_path.is_dir() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(dir_path).map_err(|e| e.to_string())?;
    let paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    let mut nodes: Vec<FileNode> = paths
        .into_par_iter()
        .map(|path| -> Result<FileNode, String> {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let is_dir = path.is_dir();
            let is_sarc = !is_dir && is_sarc_extension(&name);

            let (path_str, children) = if is_dir {
                (path.to_string_lossy().to_string(), Some(read_dir_recursive(&path)?))
            } else if is_sarc {
                let data = fs::read(&path).map_err(|e| e.to_string())?;
                let sarc_prefix = format!("SARC:{}", path.to_string_lossy());
                (sarc_prefix.clone(), Some(get_sarc_nodes(&data, &sarc_prefix)))
            } else {
                (path.to_string_lossy().to_string(), None)
            };

            Ok(FileNode {
                name,
                path: path_str,
                is_dir,
                is_sarc,
                children,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    sort_file_nodes(&mut nodes);
    Ok(nodes)
}

/// 根据打开的目标路径（目录或 SARC 单文件）解析出项目名及完整文件树
pub fn scan_project_path(path: &str) -> Result<(String, Vec<FileNode>), String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    let name = p
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "Project".to_string());

    if p.is_dir() {
        let nodes = read_dir_recursive(p)?;
        Ok((name, nodes))
    } else if is_sarc_extension(&name) {
        let data = fs::read(p).map_err(|e| e.to_string())?;
        let sarc_prefix = format!("SARC:{}", path);
        let nodes = get_sarc_nodes(&data, &sarc_prefix);
        Ok((name, nodes))
    } else {
        let single_node = FileNode {
            name: name.clone(),
            path: path.to_string(),
            is_dir: false,
            is_sarc: false,
            children: None,
        };
        Ok((name, vec![single_node]))
    }
}

/// 根据物理或虚拟路径读取完整二进制字节流
pub fn read_file_from_path(path: &str) -> Result<Vec<u8>, String> {
    if path.starts_with("SARC:") {
        let parts: Vec<&str> = path.trim_start_matches("SARC:").split("//").collect();
        if parts.is_empty() {
            return Err("无效的 SARC 文件路径".to_string());
        }
        let mut data = fs::read(parts[0]).map_err(|e| e.to_string())?;

        for i in 1..parts.len() {
            let decompressed = roead::yaz0::decompress(&data).unwrap_or_else(|_| data.clone());
            let sarc = Sarc::new(&decompressed).map_err(|e| e.to_string())?;
            let inner_name = parts[i];
            let inner_norm = inner_name.replace('\\', "/");

            data = sarc
                .files()
                .find(|f| {
                    if let Some(name) = f.name() {
                        name == inner_name || name.replace('\\', "/") == inner_norm
                    } else {
                        false
                    }
                })
                .map(|f| f.data().to_vec())
                .ok_or_else(|| format!("未在封包中找到文件: {}", inner_name))?;
        }
        Ok(data)
    } else {
        fs::read(path).map_err(|e| e.to_string())
    }
}

/// 辅助排序：文件夹与 SARC 归档置顶，按文件名升序排列
fn sort_file_nodes(nodes: &mut [FileNode]) {
    nodes.sort_by(|a, b| {
        let a_is_container = a.is_dir || a.is_sarc;
        let b_is_container = b.is_dir || b.is_sarc;
        b_is_container
            .cmp(&a_is_container)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sarc_extension() {
        assert!(is_sarc_extension("FaroreFlow.sbeventpack"));
        assert!(is_sarc_extension("Actor.sbactorpack"));
        assert!(is_sarc_extension("Bootup.pack"));
        assert!(is_sarc_extension("Title.sarc"));
        assert!(is_sarc_extension("Resource.ssarc"));
        assert!(!is_sarc_extension("tes.bfevfl"));
        assert!(!is_sarc_extension("dialogue.msbt"));
    }

    #[test]
    fn test_scan_current_dir() {
        let res = scan_project_path(".");
        assert!(res.is_ok());
        let (name, nodes) = res.unwrap();
        assert!(!name.is_empty());
        assert!(!nodes.is_empty());
    }
}
