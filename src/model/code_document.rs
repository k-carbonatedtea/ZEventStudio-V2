use serde::{Deserialize, Serialize};

/// 代码文档格式类型枚举 (对应 BOTW 及通用游戏数据格式)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeFormat {
    /// 二进制参数对象格式 (AAMP -> YAML)
    Aamp,
    /// 二进制字典与数据表格格式 (BYML/BGYML -> YAML)
    Byml,
    /// 二进制本地化消息与对话文本 (MSBT -> MSYT/YAML)
    Msbt,
    /// 事件流定义文件 (BFEVFL -> JSON/YAML 源码)
    Bfevfl,
    /// 标准 JSON 数据
    Json,
    /// 标准 YAML 数据
    Yaml,
    /// 纯文本 / 日志 / CSV 等通用文本
    Text,
    /// 通用未识别数据
    Generic,
}

impl CodeFormat {
    /// 获取格式的可读显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            CodeFormat::Aamp => "AAMP (YAML)",
            CodeFormat::Byml => "BYML (YAML)",
            CodeFormat::Msbt => "MSBT (MSYT)",
            CodeFormat::Bfevfl => "BFEVFL (JSON)",
            CodeFormat::Json => "JSON",
            CodeFormat::Yaml => "YAML",
            CodeFormat::Text => "PLAIN TEXT",
            CodeFormat::Generic => "GENERIC",
        }
    }

    /// 获取格式对应的高亮语言标识
    pub fn language_id(&self) -> &'static str {
        match self {
            CodeFormat::Aamp | CodeFormat::Byml | CodeFormat::Msbt | CodeFormat::Yaml => "yaml",
            CodeFormat::Bfevfl | CodeFormat::Json => "json",
            CodeFormat::Text | CodeFormat::Generic => "text",
        }
    }

    /// 获取代表该格式的展示图标 (如 💬, ⚙️, 📜, ⚡)
    pub fn icon(&self) -> &'static str {
        match self {
            CodeFormat::Msbt => "💬",
            CodeFormat::Byml | CodeFormat::Aamp => "⚙️",
            CodeFormat::Bfevfl => "⚡",
            CodeFormat::Json => "{}",
            CodeFormat::Yaml => "📜",
            CodeFormat::Text => "≡",
            CodeFormat::Generic => "📄",
        }
    }
}

/// 目标文件大小端模式 (Endianness)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeEndian {
    /// 小端序 (Nintendo Switch / PC / LE)
    Little,
    /// 大端序 (Nintendo Wii U / BE)
    Big,
}

impl CodeEndian {
    /// 获取大小端显示标签
    pub fn display_label(&self) -> &'static str {
        match self {
            CodeEndian::Little => "Switch (LE)",
            CodeEndian::Big => "WiiU (BE)",
        }
    }
}

/// 代码文档状态模型
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeDocumentState {
    /// 当前编辑代码文本
    pub content: String,
    /// 原始加载时的代码快照 (用于精确比对是否已修改)
    pub original_content: String,
    /// 文件格式
    pub format: CodeFormat,
    /// 大小端模式
    pub endian: CodeEndian,
    /// 关联文件物理或虚拟路径
    pub file_path: String,
    /// 数据版本号 (每次重新加载、放弃更改或外部更新时自增，通知 UI 同步)
    pub version: usize,
}

impl CodeDocumentState {
    /// 创建新代码文档状态实例
    pub fn new(
        content: String,
        format: CodeFormat,
        endian: CodeEndian,
        file_path: String,
    ) -> Self {
        Self {
            original_content: content.clone(),
            content,
            format,
            endian,
            file_path,
            version: 0,
        }
    }

    /// 检查当前内容是否相对于原始加载发生变更
    pub fn is_modified(&self) -> bool {
        self.content != self.original_content
    }

    /// 重置内容为原始状态并自增版本号
    pub fn revert(&mut self) {
        self.content = self.original_content.clone();
        self.version = self.version.wrapping_add(1);
    }

    /// 标记当前内容为已保存（更新基线快照）
    pub fn mark_saved(&mut self) {
        self.original_content = self.content.clone();
    }

    /// 从磁盘或外部重新加载新内容并自增版本号
    pub fn reload(&mut self, content: String, format: CodeFormat, endian: CodeEndian) {
        self.original_content = content.clone();
        self.content = content;
        self.format = format;
        self.endian = endian;
        self.version = self.version.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_document_state_dirty_tracking() {
        let mut doc = CodeDocumentState::new(
            "key: value".to_string(),
            CodeFormat::Byml,
            CodeEndian::Little,
            "test.byml".to_string(),
        );

        assert!(!doc.is_modified());
        doc.content = "key: new_value".to_string();
        assert!(doc.is_modified());

        doc.revert();
        assert!(!doc.is_modified());
        assert_eq!(doc.content, "key: value");

        doc.content = "key: changed".to_string();
        doc.mark_saved();
        assert!(!doc.is_modified());
        assert_eq!(doc.original_content, "key: changed");
    }
}
