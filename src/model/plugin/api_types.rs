#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// 插件本地 API 请求命令通道枚举 (在后台服务与 UI 主线程间传递)
#[derive(Debug)]
pub enum ApiCommand {
    /// 获取应用运行状态及当前概览
    GetStatus {
        responder: smol::channel::Sender<serde_json::Value>,
    },
    /// 获取当前打开的工程项目及文件树信息
    GetProject {
        responder: smol::channel::Sender<serde_json::Value>,
    },
    /// 获取当前活动事件流图拓扑 (Nodes, Edges, Actors)
    GetGraph {
        responder: smol::channel::Sender<serde_json::Value>,
    },
    /// 执行图编辑或工作区动作 (增删改查、布局、Toast 提示等)
    ExecuteAction {
        payload: serde_json::Value,
        responder: smol::channel::Sender<Result<serde_json::Value, String>>,
    },
}

/// 添加节点动作参数载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNodePayload {
    /// 节点名称 (例如 "Demo_PlayAnimation")
    pub name: String,
    /// 节点类型 (Action, Query, Fork, Join, Switch, Subflow)
    #[serde(default = "default_action_kind")]
    pub kind: String,
    /// 归属角色 (例如 "Link", "WorldManager")
    #[serde(default)]
    pub actor: Option<String>,
    /// 初始横向坐标 (可选，缺省自动放置)
    #[serde(default)]
    pub x: Option<f32>,
    /// 初始纵向坐标 (可选，缺省自动放置)
    #[serde(default)]
    pub y: Option<f32>,
    /// 初始属性参数键值字典
    #[serde(default)]
    pub params: std::collections::HashMap<String, String>,
}

fn default_action_kind() -> String {
    "Action".to_string()
}

/// 节点参数更新动作载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNodePayload {
    /// 目标节点 ID
    pub node_id: String,
    /// 新节点名称 (可选)
    #[serde(default)]
    pub name: Option<String>,
    /// 新归属角色 (可选)
    #[serde(default)]
    pub actor: Option<String>,
    /// 属性更新键值字典
    #[serde(default)]
    pub params: std::collections::HashMap<String, String>,
}

/// 节点引脚连线动作载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectNodesPayload {
    /// 源节点 ID
    pub from_node: String,
    /// 源端口 ID (如 "out", "out_True", "out_0")
    #[serde(default = "default_out_port")]
    pub from_port: String,
    /// 目标节点 ID
    pub to_node: String,
    /// 目标端口 ID (如 "in")
    #[serde(default = "default_in_port")]
    pub to_port: String,
}

fn default_out_port() -> String {
    "out".to_string()
}

fn default_in_port() -> String {
    "in".to_string()
}

/// 界面提示通知 Toast 载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastPayload {
    /// 提示文本信息
    pub message: String,
    /// 提示级别 ("info", "success", "error")
    #[serde(default = "default_info_level")]
    pub level: String,
}

fn default_info_level() -> String {
    "info".to_string()
}
