# ZEventStudio MCP Bridge Plugin

`zevent-mcp-bridge` 是为 **ZEventStudio**（塞尔达传说：旷野之息 / 王国之泪事件流编辑器与 Mod 开发套件）深度定制的 **Model Context Protocol (MCP)** 全功能桥接插件。

通过此插件，AI 智能助手（如 **Claude Desktop**, **Cursor**, **Google Antigravity**, **Cline** 等）可直接全面连接正在运行的 ZEventStudio 编辑器，实现：
1. **项目文件与 SARC 归档全量查询**：递归展开项目物理文件、多层嵌套 SARC / Yaz0 归档内虚拟文件树，支持按格式过滤与数量限制；
2. **多格式代码反编译与实时读取**：读取 BFEVFL 流程图（自动反编译为标准 JSON 拓扑）、BYML/BGYML（YAML）、AAMP/BAIPROG（YAML）、MSBT（MSYT 对话数据）及普通代码，并优先读取内存中未保存的编辑器标签页；
3. **全自动代码回编译与智能保存**：修改代码后自动编译回原生二进制、自动 Yaz0 压缩并写回磁盘或 SARC 归档，同时零延迟热重载正在编辑的画布与代码窗口；
4. **文件增删改查与目录管理**：支持创建文件/文件夹、删除文件/封包条目、重命名文件、全局代码检索；
5. **工作区标签页协同调度**：打开文件至新标签、切换标签、关闭标签、保存标签及工程加载。

---

## 🚀 完整 MCP 工具列表

### 1. 文件树与项目查询 (Project & Files Query)
| 工具名称 | 功能说明 | 关键参数 |
| :--- | :--- | :--- |
| `zevent_get_status` | 获取编辑器全局运行状态、活动文件、标签页数量及流程图摘要 | 无 |
| `zevent_get_project` | 获取已打开项目的物理路径、SARC 封包与文件列表统计、打开的标签页列表 | 无 |
| `zevent_list_files` | 递归查询项目目录及所有 SARC 封包内部的全部子文件 | `path`, `recursive`, `include_sarc`, `filter_ext`, `limit` |
| `zevent_search_code` | 在工程物理文件及 SARC 封包内部全文检索代码/文本关键字 | `query`, `case_sensitive`, `file_pattern`, `max_results` |

### 2. 代码查看与编辑修改 (Code Read & Write)
| 工具名称 | 功能说明 | 关键参数 |
| :--- | :--- | :--- |
| `zevent_read_file` | 查看指定文件的源码/反编译内容（优先读取活跃未保存标签） | `path`, `format` ("code" / "raw") |
| `zevent_write_file` | 写入/修改/创建物理文件或 SARC 封包内虚拟文件（自动编译与热重载） | `path`, `content`, `format` |
| `zevent_delete_file` | 删除物理文件/文件夹或 SARC 封包内虚拟子文件（自动关闭对应标签） | `path` |
| `zevent_rename_file` | 重命名物理文件/文件夹或 SARC 封包内虚拟子文件（自动同步对应标签） | `old_path`, `new_name` |
| `zevent_create_directory` | 递归创建物理文件夹目录 | `path` |

### 3. 工作区与标签页协同 (Workspace & Tabs)
| 工具名称 | 功能说明 | 关键参数 |
| :--- | :--- | :--- |
| `zevent_get_tabs` | 获取所有已打开文档标签页状态（ID、标题、路径、类型、修改标记） | 无 |
| `zevent_open_file` | 在 ZEventStudio UI 中打开指定物理文件或 SARC 条目为编辑器标签 | `path` |
| `zevent_save_file` | 保存指定或当前活跃的文档标签页至磁盘/封包 | `tab_id`（可选） |
| `zevent_close_file` | 关闭指定文档标签页 | `tab_id` 或 `path` |
| `zevent_reload_project` | 重新扫描项目文件树并刷新 Git 版本控制状态 | 无 |
| `zevent_open_project` | 加载新工程目录或打开独立 SARC 封包 | `path` |

### 4. 流程图节点与拓扑操作 (Event Flowchart Editing)
| 工具名称 | 功能说明 | 关键参数 |
| :--- | :--- | :--- |
| `zevent_get_graph` | 获取当前正在编辑的完整事件流图拓扑（节点、连线、角色、参数） | 无 |
| `zevent_add_node` | 在画布上新增事件节点（Action, Query, Switch, Fork, Join, SubFlow 等） | `name`, `kind`, `actor`, `x`, `y`, `params` |
| `zevent_connect_nodes` | 在节点出入口引脚之间建立定向事件连线 | `from_node`, `from_port`, `to_node`, `to_port` |
| `zevent_update_node` | 更新指定节点的名称、归属角色或属性参数字典 | `node_id`, `name`, `actor`, `params` |
| `zevent_delete_node` | 删除指定节点及其相连的全部输入输出连线 | `node_id` |
| `zevent_auto_layout` | 一键触发 Sugiyama 层次化拓扑自动对齐与画布居中适应 | 无 |
| `zevent_toast` | 在 ZEventStudio 编辑器界面弹出悬浮通知提示 | `message`, `level` ("info" / "success" / "error") |

---

## 🛠️ 快速接入指南

### 1. 启动 ZEventStudio
运行 ZEventStudio，本地 API 桥接服务会自动启动并在 `http://127.0.0.1:31415` 监听连接。您也可以随时在左侧 **🧩 插件中心** 启停服务或查看运行端口。

### 2. 配置 Claude Desktop
将以下内容加入到 Claude Desktop 配置文件 `claude_desktop_config.json` 中：

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
在 Cursor 或 Antigravity 的 MCP 设置中添加服务器：

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

在终端中可直接运行诊断命令验证连接与功能：
```bash
python plugins/zevent-mcp-bridge/mcp_server.py --test
```
若 ZEventStudio 已启动，将输出当前连接状态、工程概览以及前几个示例文件列表。
