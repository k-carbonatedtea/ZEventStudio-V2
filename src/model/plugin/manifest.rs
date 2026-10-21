use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 插件类型分类枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PluginKind {
    /// 默认通用插件
    #[default]
    General,
    /// 模型上下文协议 (Model Context Protocol) 插件
    Mcp,
    /// 脚本类插件 (Python, Lua, Node.js 等)
    Script,
    /// 外部独立进程/可执行文件插件
    Process,
    /// 内置/原生功能插件
    Internal,
}

/// 插件贡献的命令项定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginCommand {
    /// 命令唯一标识符 (例如 "mcp.start_server")
    pub id: String,
    /// 命令用户可读标题
    pub title: String,
    /// 命令详细说明
    #[serde(default)]
    pub description: Option<String>,
}

/// 插件贡献的工具能力项定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginTool {
    /// 工具名称 (例如 "get_graph")
    pub name: String,
    /// 工具说明
    pub description: String,
    /// 工具输入参数 JSON Schema 定义 (可选)
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
}

/// 插件贡献项集合 (扩展点清单)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PluginContributes {
    /// 声明贡献的命令列表
    #[serde(default)]
    pub commands: Vec<PluginCommand>,
    /// 声明贡献的工具能力列表
    #[serde(default)]
    pub tools: Vec<PluginTool>,
}

/// 插件外部服务启动与通信配置
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PluginServerConfig {
    /// 通信传输模式 ("stdio", "http", "websocket")
    #[serde(default = "default_transport")]
    pub transport: String,
    /// 本地监听端口 (如使用 HTTP)
    #[serde(default)]
    pub port: Option<u16>,
    /// 启动命令 (例如 "python", "node", "./bin/my_plugin")
    #[serde(default)]
    pub command: Option<String>,
    /// 启动命令行参数列表
    #[serde(default)]
    pub args: Vec<String>,
}

fn default_transport() -> String {
    "stdio".to_string()
}

/// 插件清单结构体 (对应 plugin.json)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    /// 插件全局唯一标识符 (例如 "zevent-mcp-bridge")
    pub id: String,
    /// 插件显示名称
    pub name: String,
    /// 语义化版本号 (例如 "1.0.0")
    pub version: String,
    /// 插件功能描述
    pub description: String,
    /// 作者或组织名称
    pub author: String,
    /// 插件图标 (Emoji 或 图标标识，例如 "🤖")
    #[serde(default = "default_icon")]
    pub icon: String,
    /// 插件类型分类
    #[serde(rename = "type", default)]
    pub kind: PluginKind,
    /// 入口文件路径 (相对于插件根目录，例如 "mcp_server.py")
    #[serde(default)]
    pub entry: Option<String>,
    /// 是否默认启用
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 项目主页或仓库链接
    #[serde(default)]
    pub homepage: Option<String>,
    /// 外部服务端配置
    #[serde(default)]
    pub server: Option<PluginServerConfig>,
    /// 插件声明贡献项
    #[serde(default)]
    pub contributes: PluginContributes,
}

fn default_icon() -> String {
    "🧩".to_string()
}

fn default_true() -> bool {
    true
}

/// 插件运行状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    /// 已加载就绪
    Loaded,
    /// 正在后台运行中
    #[allow(dead_code)]
    Running,
    /// 已禁用
    Disabled,
    /// 加载或运行出错
    Error(String),
}

/// 插件运行时实例对象
#[derive(Debug, Clone, PartialEq)]
pub struct PluginInstance {
    /// 静态清单定义
    pub manifest: PluginManifest,
    /// 插件所在文件系统绝对路径
    pub dir_path: PathBuf,
    /// 当前运行时生命周期状态
    pub status: PluginStatus,
    /// 异常信息 (若有)
    pub error_message: Option<String>,
}

impl PluginInstance {
    /// 创建新插件实例
    pub fn new(manifest: PluginManifest, dir_path: PathBuf) -> Self {
        let status = if manifest.enabled {
            PluginStatus::Loaded
        } else {
            PluginStatus::Disabled
        };
        Self {
            manifest,
            dir_path,
            status,
            error_message: None,
        }
    }

    /// 获取插件唯一标识符
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    /// 获取插件显示名称
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    /// 判断插件是否处于激活启用状态
    pub fn is_enabled(&self) -> bool {
        self.manifest.enabled && self.status != PluginStatus::Disabled
    }
}
