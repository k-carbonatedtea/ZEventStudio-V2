/// 扁平化展示项 (用于虚拟化列表快速分片渲染)
#[derive(Clone)]
pub struct FlatFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_sarc: bool,
    pub level: usize,
    pub is_expanded: bool,
    pub is_selected: bool,
    pub is_modified: bool,
}
