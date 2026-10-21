# ZEventStudio MCP Bridge Plugin

`zevent-mcp-bridge` 是为 **ZEventStudio**（塞尔达荒野之息 / 王国之泪事件流编辑器）定制的 **Model Context Protocol (MCP)** 桥接插件。

通过此插件，AI 智能助手（如 **Claude Desktop**, **Cursor**, **Google Antigravity**, **Cline** 等）可以直接实时理解当前打开的事件流程图结构，并协助您完成节点增删、连线构建、参数填写与拓扑重排。

---

## 🚀 核心功能与 MCP 工具列表

| 工具名称 | 功能说明 |
| :--- | :--- |
| `zevent_get_status` | 获取编辑器全局状态、当前项目名、活动标签页及图拓扑摘要 |
| `zevent_get_project` | 获取已打开项目的物理路径、SARC 封包与文件列表统计 |
| `zevent_get_graph` | 获取当前正在编辑的完整事件流图（节点、引脚、连线、角色、参数） |
| `zevent_add_node` | 在画布上新增节点（支持 Action, Query, Switch, Fork, Join, SubFlow, EntryPoint, Comment） |
| `zevent_connect_nodes` | 在指定节点的出入口端口之间建立定向事件连线 |
| `zevent_update_node` | 更新指定节点的名称、归属角色或属性参数键值 |
| `zevent_delete_node` | 删除指定节点及其相连的全部输入输出连线 |
| `zevent_auto_layout` | 一键触发 Sugiyama 层次化拓扑自动对齐与画布居中适应 |
| `zevent_toast` | 在 ZEventStudio 界面弹出通知提示信息 |

---

## 🛠️ 快速接入指南

### 1. 启动 ZEventStudio 本地桥接服务
1. 打开 ZEventStudio。
2. 点击左侧活动栏的 **🧩 插件中心** 图标（或快捷键 `Ctrl+Shift+X`）。
3. 找到 **ZEvent MCP Bridge** 插件卡片。
4. 点击 **「启动服务」** 按钮（默认监听 `http://127.0.0.1:31415`）。

### 2. 配置 Claude Desktop
点击插件卡片上的 **「📋 Claude Desktop 配置」** 复制配置，并将其粘贴到 Claude Desktop 配置文件 `claude_desktop_config.json` 中：

```json
{
  "mcpServers": {
    "zevent-studio": {
      "command": "python",
      "args": [
        "c:/Users/Administrator/Desktop/Zevent/ZEventStudio/plugins/zevent-mcp-bridge/mcp_server.py"
      ]
    }
  }
}
```

### 3. 配置 Cursor / Antigravity
点击插件卡片上的 **「📋 Cursor / AGY 配置」** 复制配置，并加入到 MCP 服务器配置中：

```json
{
  "mcpServers": {
    "zevent-studio": {
      "command": "python",
      "args": [
        "c:/Users/Administrator/Desktop/Zevent/ZEventStudio/plugins/zevent-mcp-bridge/mcp_server.py"
      ]
    }
  }
}
```

---

## 🧪 本地测试验证

在终端中可直接运行诊断命令验证连接：
```bash
python plugins/zevent-mcp-bridge/mcp_server.py --test
```
若 ZEventStudio 已启动且开启了本地服务，将输出当前的连接与状态 JSON 信息。
