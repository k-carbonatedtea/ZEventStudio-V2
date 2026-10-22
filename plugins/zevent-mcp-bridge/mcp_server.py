#!/usr/bin/env python3
"""
ZEventStudio Model Context Protocol (MCP) Server
Bridge between AI assistants (Claude, Cursor, Antigravity) and ZEventStudio.
Communicates with ZEventStudio local API server via HTTP on http://127.0.0.1:31415.
Provides comprehensive querying, modifying, and deleting operations for files,
code, flowcharts, and editor workspace tabs.
"""

import sys
import json
import urllib.request
import urllib.error

# Ensure UTF-8 streams for stdio JSON-RPC on Windows
if hasattr(sys.stdin, 'reconfigure'):
    try:
        sys.stdin.reconfigure(encoding='utf-8')
        sys.stdout.reconfigure(encoding='utf-8')
        sys.stderr.reconfigure(encoding='utf-8')
    except Exception:
        pass

API_BASE_URL = "http://127.0.0.1:31415"
PROTOCOL_VERSION = "2024-11-05"

# Bypass any environment/system proxies for local communication
proxy_handler = urllib.request.ProxyHandler({})
opener = urllib.request.build_opener(proxy_handler)
urllib.request.install_opener(opener)

SERVER_INFO = {
    "name": "zevent-mcp-bridge",
    "version": "1.1.0"
}

TOOLS = [
    {
        "name": "zevent_get_status",
        "description": "获取 ZEventStudio 当前运行状态、已打开项目、当前激活的文件标签页以及事件图概览信息",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_get_project",
        "description": "获取当前已打开项目的路径、文件树概要、打开的所有标签页以及项目详细配置",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_list_files",
        "description": "查询并列出当前项目中的所有物理文件与 SARC 虚拟归档文件（支持按扩展名过滤、递归遍历以及解包展示 SARC 内部子文件）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要列出的子目录路径或 SARC 文件路径（默认为项目根目录）"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "是否递归遍历所有子目录及嵌套 SARC 封包（默认: true）"
                },
                "include_sarc": {
                    "type": "boolean",
                    "description": "是否解包并展开 SARC/Yaz0 封包内的虚拟文件（默认: true）"
                },
                "filter_ext": {
                    "type": "string",
                    "description": "按文件扩展名过滤，例如 '.bfevfl', '.byml', '.msbt', '.aamp'"
                },
                "limit": {
                    "type": "integer",
                    "description": "返回的最大文件条目数量（默认: 2000）"
                }
            }
        }
    },
    {
        "name": "zevent_read_file",
        "description": "读取当前项目或 SARC 封包内任意文件的源码或格式化文本（自动将 BFEVFL 反编译为 JSON 图结构，BYML/AAMP/MSBT 转为 YAML/MSYT，支持读取内存中已修改但未保存的激活标签页）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "目标文件路径（支持物理绝对路径、项目相对路径或 SARC 虚拟路径，如 'SARC:path.sbeventpack//Event/Demo.bfevfl'）"
                },
                "format": {
                    "type": "string",
                    "enum": ["code", "raw"],
                    "description": "格式模式：'code'（默认，解析为可编辑的结构代码/YAML/JSON）或 'raw'（原始 UTF-8 文本）"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_write_file",
        "description": "写入、修改或创建项目中的文件或代码（自动将 JSON 图结构重编译为 BFEVFL，YAML 转为 BYML/AAMP/MSBT，按需自动进行 Yaz0 压缩并写回磁盘或 SARC 封包，同时实时热同步编辑器中打开的标签页）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "目标文件路径（可为物理文件或 SARC 内部虚拟路径）"
                },
                "content": {
                    "type": "string",
                    "description": "要写入的源代码、JSON 流程图结构、YAML 或纯文本内容"
                },
                "format": {
                    "type": "string",
                    "enum": ["bfevfl", "byml", "aamp", "msbt", "json", "yaml", "text"],
                    "description": "指定覆盖格式类型（若省略则自动根据文件扩展名推断）"
                },
                "endian": {
                    "type": "string",
                    "enum": ["big", "little"],
                    "description": "字节端序（'big' 大端序或 'little' 小端序，若省略则自动沿用原文件或格式规范）"
                }
            },
            "required": ["path", "content"]
        }
    },
    {
        "name": "zevent_delete_file",
        "description": "删除物理文件/目录，或从 SARC 封包内移除指定的虚拟子文件，并同步关闭编辑器中对应的标签页",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要删除的物理文件/目录路径，或 SARC 内部虚拟子文件路径"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_rename_file",
        "description": "重命名或移动物理文件/目录，或重命名 SARC 封包内的虚拟子文件，并自动更新编辑器中已打开的标签页",
        "inputSchema": {
            "type": "object",
            "properties": {
                "old_path": {
                    "type": "string",
                    "description": "原始文件路径或 SARC 虚拟条目路径"
                },
                "new_name": {
                    "type": "string",
                    "description": "新文件名或新的内部相对路径"
                }
            },
            "required": ["old_path", "new_name"]
        }
    },
    {
        "name": "zevent_create_directory",
        "description": "在当前项目中递归创建新文件夹目录",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要创建的目录路径（相对于项目根目录或绝对路径）"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_search_code",
        "description": "在当前项目所有文件及 SARC 封包中全文搜索文本或关键字（返回匹配的文件路径、行号及代码片段）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "要搜索的关键字或文本字符串"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "是否区分大小写（默认: false）"
                },
                "file_pattern": {
                    "type": "string",
                    "description": "按文件路径或后缀通配过滤，如 '.bfevfl' 或 'Actor'"
                },
                "max_results": {
                    "type": "integer",
                    "description": "返回的最大匹配行数（默认: 50）"
                }
            },
            "required": ["query"]
        }
    },
    {
        "name": "zevent_get_tabs",
        "description": "获取 ZEventStudio 当前所有已打开的文档标签页信息（包含标签 ID、标题、文件路径、类型、激活状态与修改状态）",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_open_file",
        "description": "在 ZEventStudio 编辑器界面中打开指定文件或 SARC 条目为新标签页（支持 BFEVFL 流程图视图与代码编辑器视图）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要在编辑器中打开的文件路径"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_save_file",
        "description": "将当前激活的文档标签页或指定标签页保存到磁盘或 SARC 封包中",
        "inputSchema": {
            "type": "object",
            "properties": {
                "tab_id": {
                    "type": "string",
                    "description": "要保存的标签页 ID（若省略则保存当前激活的文档）"
                }
            }
        }
    },
    {
        "name": "zevent_close_file",
        "description": "在 ZEventStudio 中根据标签页 ID 或文件路径关闭已打开的文档标签页",
        "inputSchema": {
            "type": "object",
            "properties": {
                "tab_id": {
                    "type": "string",
                    "description": "要关闭的标签页 ID"
                },
                "path": {
                    "type": "string",
                    "description": "要关闭的标签页文件路径"
                }
            }
        }
    },
    {
        "name": "zevent_reload_project",
        "description": "重新扫描并刷新 ZEventStudio 当前项目文件树及 Git 版本控制状态",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_open_project",
        "description": "在 ZEventStudio 中加载并打开新项目目录（Mod 根目录）或 SARC 封包文件（.sbeventpack、.sbactorpack、.pack 等）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "要打开的项目文件夹路径或封包文件路径"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_get_graph",
        "description": "获取当前激活文档的完整事件流程图数据（包含所有节点、连线、角色 Actor 与属性列表）",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_add_node",
        "description": "在 ZEventStudio 当前激活的流程图中添加新节点（Action、Query、Switch、Fork、Join、SubFlow、EntryPoint、Comment，支持一键自动填充参数）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "节点标题 / 动作名称（如 'Demo_PlayAnimation'）"
                },
                "kind": {
                    "type": "string",
                    "enum": ["Action", "Query", "Switch", "Fork", "Join", "SubFlow", "EntryPoint", "Comment"],
                    "description": "节点类型（Action、Query、Switch、Fork、Join、SubFlow、EntryPoint、Comment，默认: 'Action'）"
                },
                "actor": {
                    "type": "string",
                    "description": "绑定的角色名称（如 'Link', 'WorldManager', 'EventSystemActor'）"
                },
                "x": {
                    "type": "number",
                    "description": "画布 X 轴像素坐标"
                },
                "y": {
                    "type": "number",
                    "description": "画布 Y 轴像素坐标"
                },
                "params": {
                    "type": "object",
                    "description": "自定义参数字典键值对 { key: value }"
                },
                "autofill": {
                    "type": "boolean",
                    "description": "是否自动根据《旷野之息》游戏 AI 规范补全标准参数项（默认: false）"
                }
            },
            "required": ["name"]
        }
    },
    {
        "name": "zevent_connect_nodes",
        "description": "在当前流程图的两个节点之间创建有向连接边",
        "inputSchema": {
            "type": "object",
            "properties": {
                "from_node": {
                    "type": "string",
                    "description": "源节点 ID"
                },
                "from_port": {
                    "type": "string",
                    "description": "源端口名称（默认: 'out'）"
                },
                "to_node": {
                    "type": "string",
                    "description": "目标节点 ID"
                },
                "to_port": {
                    "type": "string",
                    "description": "目标端口名称（默认: 'in'）"
                }
            },
            "required": ["from_node", "to_node"]
        }
    },
    {
        "name": "zevent_update_node",
        "description": "更新当前流程图中指定节点的属性、标题、角色或触发参数自动填充/排序",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node_id": {
                    "type": "string",
                    "description": "目标节点 ID"
                },
                "name": {
                    "type": "string",
                    "description": "新的节点标题 / 动作名称"
                },
                "actor": {
                    "type": "string",
                    "description": "新的角色名称"
                },
                "params": {
                    "type": "object",
                    "description": "要更新或追加的参数字典 { key: value }"
                },
                "autofill": {
                    "type": "boolean",
                    "description": "是否自动根据游戏 AI 补全标准参数（默认: false）"
                },
                "reorder": {
                    "type": "boolean",
                    "description": "是否按任天堂官方规范重新排序参数（默认: false）"
                }
            },
            "required": ["node_id"]
        }
    },
    {
        "name": "zevent_delete_node",
        "description": "从当前激活的流程图中删除指定节点及其所有关联连线",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node_id": {
                    "type": "string",
                    "description": "要删除的节点 ID"
                }
            },
            "required": ["node_id"]
        }
    },
    {
        "name": "zevent_auto_layout",
        "description": "对当前流程图自动执行 Sugiyama 分层布局算法并自适应居中缩放画布",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_toast",
        "description": "在 ZEventStudio 编辑器界面右上角弹出 Toast 消息提示气泡",
        "inputSchema": {
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "提示文本内容"
                },
                "level": {
                    "type": "string",
                    "enum": ["info", "success", "error"],
                    "description": "消息级别（'info', 'success', 'error'）"
                }
            },
            "required": ["message"]
        }
    },
    {
        "name": "zevent_autofill_parameters",
        "description": "利用本地《旷野之息》游戏 AI 参数库（支持 Switch RomFS 与 Wii U Content dump），为指定 Action / Query / Switch 节点自动填充标准参数并规范排布",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node_id": {
                    "type": "string",
                    "description": "要执行参数自动填充的目标节点 ID"
                }
            },
            "required": ["node_id"]
        }
    },
    {
        "name": "zevent_reorder_parameters",
        "description": "根据《旷野之息》官方游戏 AI 定义的先后顺序，对指定节点的参数项进行规范重排",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node_id": {
                    "type": "string",
                    "description": "要规范排序参数的目标节点 ID"
                }
            },
            "required": ["node_id"]
        }
    },
    {
        "name": "zevent_get_ai_parameters",
        "description": "查询《旷野之息》官方游戏 AI 参数定义（名称、类型、默认值），无需修改画布流程图即可直接获取角色规范",
        "inputSchema": {
            "type": "object",
            "properties": {
                "actor": {
                    "type": "string",
                    "description": "角色名称（如 'EventSystemActor', 'GameROMPlayer', 'Npc_Zora001'）"
                },
                "name": {
                    "type": "string",
                    "description": "动作或查询名称（如 'Demo_OpenMessageDialog', 'GeneralTalk'）"
                },
                "is_query": {
                    "type": "boolean",
                    "description": "是否为 Query 查询节点（默认为 false 即 Action）"
                }
            },
            "required": ["actor", "name"]
        }
    },
    {
        "name": "zevent_query_byml",
        "description": "使用 JSON 指针高效查询与透视 BYML 数据结构，无需反编译传输超大 YAML（支持任务产品 QuestProduct、GameData、SaveData、Mubin 地图等）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "目标 BYML 文件路径（物理路径或 SARC 虚拟路径，如 'SARC:Pack/TitleBG.pack//Quest/QuestProduct.sbquestpack'）"
                },
                "query_path": {
                    "type": "string",
                    "description": "要检索的 JSON 指针路径，如 '/Quest', '/Quest/170', '/data/bool_data'（留空默认根目录）"
                },
                "summary_only": {
                    "type": "boolean",
                    "description": "为 true 时仅返回数组长度或对象键列表，避免传输庞大递归树"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_patch_byml",
        "description": "对 BYML 文件执行高性能局部修改（追加 append、设置 set、删除 delete），无需全量重写（支持大/小端序、自动 Yaz0 压缩与 SARC 封装）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "目标 BYML 文件路径（物理路径或 SARC 虚拟路径）"
                },
                "patch_op": {
                    "type": "string",
                    "enum": ["append", "set", "delete"],
                    "description": "修改操作类型：'append'（向数组追加）、'set'（设置/覆盖值）、'delete'（删除键或索引）"
                },
                "query_path": {
                    "type": "string",
                    "description": "操作生效的 JSON 指针路径，如 '/Quest' 或 '/Quest/170/Name'"
                },
                "value": {
                    "description": "要设置或追加的 JSON 数据值（支持字符串、数字、布尔、数组或对象）"
                },
                "endian": {
                    "type": "string",
                    "enum": ["big", "little"],
                    "description": "目标端序（'big' 或 'little'，若省略则自动检测）"
                }
            },
            "required": ["path", "patch_op"]
        }
    },
    {
        "name": "zevent_read_msbt",
        "description": "将 MSBT 本地化文本文件读取为键值对字典格式（支持文本提取、SARC 封包解析与自动 Yaz0 解压）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "目标 MSBT 文件路径（物理路径或 SARC 虚拟路径，如 'SARC:Pack/Bootup_CNzh.pack//QuestMsg/QL_Test.msbt'）"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_write_msbt",
        "description": "向 MSBT 本地化文件中写入或批量更新文本条目（自动创建或就地更新、打包进 SARC 并按需 Yaz0 压缩）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "目标 MSBT 文件路径（物理路径或 SARC 虚拟路径）"
                },
                "entries": {
                    "type": "object",
                    "description": "要写入的标签-文本键值对字典，如 {'QL_Test_Name': '测试任务', 'QL_Test_Desc': '任务描述'}"
                },
                "endian": {
                    "type": "string",
                    "enum": ["big", "little"],
                    "description": "端序（Switch 为 'little'，Wii U 为 'big'，默认自动检测）"
                }
            },
            "required": ["path", "entries"]
        }
    },
    {
        "name": "zevent_pack_sarc",
        "description": "将物理文件夹递归打包为 SARC（.pack / .sarc / .ssarc）封包文件（支持 Yaz0 压缩与大/小端序）",
        "inputSchema": {
            "type": "object",
            "properties": {
                "source_dir": {
                    "type": "string",
                    "description": "包含待封包文件的物理源目录路径"
                },
                "target_file": {
                    "type": "string",
                    "description": "目标 SARC 封包文件路径（如 'Pack/TitleBG.pack'）"
                },
                "endian": {
                    "type": "string",
                    "enum": ["big", "little"],
                    "description": "SARC 文件头与文件名表的字节端序（默认: 'big'）"
                },
                "compress_yaz0": {
                    "type": "boolean",
                    "description": "是否使用 Yaz0 压缩生成的 SARC 封包（默认: 对 .s... 扩展名自动启用）"
                }
            },
            "required": ["source_dir", "target_file"]
        }
    },
    {
        "name": "zevent_unpack_sarc",
        "description": "将 SARC（.pack / .sarc / .ssarc）封包或其虚拟子目录解压提取至目标物理文件夹",
        "inputSchema": {
            "type": "object",
            "properties": {
                "source_file": {
                    "type": "string",
                    "description": "要解压的 SARC 封包路径（物理文件或 SARC 虚拟条目）"
                },
                "target_dir": {
                    "type": "string",
                    "description": "解压释放的目标物理文件夹路径"
                }
            },
            "required": ["source_file", "target_dir"]
        }
    }
]


def http_get(path: str) -> dict:
    url = f"{API_BASE_URL}{path}"
    req = urllib.request.Request(url, headers={"User-Agent": "ZEvent-MCP-Server"})
    try:
        with urllib.request.urlopen(req, timeout=10.0) as resp:
            content = resp.read().decode("utf-8")
            return json.loads(content)
    except urllib.error.URLError as e:
        raise ConnectionError(
            f"Unable to connect to ZEventStudio local API server at {url}. "
            f"Please ensure ZEventStudio is running and the Local API Server is started (Click '启动服务' in Plugins Center). Error: {e}"
        )


def http_post(path: str, data: dict) -> dict:
    url = f"{API_BASE_URL}{path}"
    body = json.dumps(data).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=body,
        headers={"Content-Type": "application/json", "User-Agent": "ZEvent-MCP-Server"},
        method="POST"
    )
    try:
        with urllib.request.urlopen(req, timeout=15.0) as resp:
            content = resp.read().decode("utf-8")
            return json.loads(content)
    except urllib.error.HTTPError as e:
        err_msg = e.read().decode("utf-8")
        try:
            err_json = json.loads(err_msg)
            raise RuntimeError(err_json.get("error", err_msg))
        except Exception:
            raise RuntimeError(f"HTTP Error {e.code}: {err_msg}")
    except urllib.error.URLError as e:
        raise ConnectionError(
            f"Unable to connect to ZEventStudio local API server at {url}. "
            f"Please ensure ZEventStudio is running and the Local API Server is started. Error: {e}"
        )


def handle_tool_call(tool_name: str, arguments: dict) -> dict:
    try:
        # 1. 状态与概览查询
        if tool_name == "zevent_get_status":
            res = http_get("/api/status")
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_get_project":
            res = http_get("/api/project")
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_get_graph":
            res = http_get("/api/graph")
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        # 2. 文件与代码查询
        elif tool_name == "zevent_list_files":
            payload = {"action": "list_files"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_read_file":
            payload = {"action": "read_file"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_search_code":
            payload = {"action": "search_code"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        # 3. 文件修改、创建、删除、重命名
        elif tool_name == "zevent_write_file":
            payload = {"action": "write_file"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_delete_file":
            payload = {"action": "delete_file"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_rename_file":
            payload = {"action": "rename_file"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_create_directory":
            payload = {"action": "create_directory"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name in (
            "zevent_query_byml",
            "zevent_patch_byml",
            "zevent_read_msbt",
            "zevent_write_msbt",
            "zevent_pack_sarc",
            "zevent_unpack_sarc",
        ):
            action = tool_name.replace("zevent_", "")
            payload = {"action": action}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        # 4. 工作区与标签页操作
        elif tool_name == "zevent_get_tabs":
            res = http_post("/api/action", {"action": "get_tabs"})
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_open_file":
            payload = {"action": "open_file"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_save_file":
            payload = {"action": "save_file"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_close_file":
            payload = {"action": "close_file"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_reload_project":
            res = http_post("/api/action", {"action": "reload_project"})
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_open_project":
            payload = {"action": "open_project"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        # 5. 流程图节点与拓扑操作
        elif tool_name == "zevent_add_node":
            payload = {"action": "add_node"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_connect_nodes":
            payload = {"action": "connect_nodes"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_update_node":
            payload = {"action": "update_node"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_delete_node":
            payload = {"action": "delete_node"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_auto_layout":
            res = http_post("/api/action", {"action": "auto_layout"})
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name in ("zevent_autofill_parameters", "zevent_reorder_parameters", "zevent_get_ai_parameters"):
            action = tool_name.replace("zevent_", "")
            payload = {"action": action}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_toast":
            payload = {"action": "toast"}
            payload.update(arguments)
            res = http_post("/api/action", payload)
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        else:
            return {
                "isError": True,
                "content": [{"type": "text", "text": f"Unknown tool: {tool_name}"}]
            }

    except Exception as e:
        return {
            "isError": True,
            "content": [{"type": "text", "text": f"Error executing tool '{tool_name}': {str(e)}"}]
        }


def run_stdio_server():
    """Main JSON-RPC 2.0 loop over stdin/stdout for MCP clients."""
    while True:
        line = sys.stdin.readline()
        if not line:
            break
        line_clean = line.strip()
        if not line_clean:
            continue

        req = None
        # Handle Content-Length framing if present
        if line_clean.lower().startswith("content-length:"):
            try:
                length = int(line_clean.split(":", 1)[1].strip())
                while True:
                    header = sys.stdin.readline()
                    if not header or header.strip() == "":
                        break
                raw_data = sys.stdin.read(length)
                req = json.loads(raw_data)
            except Exception:
                continue
        else:
            try:
                req = json.loads(line_clean)
            except json.JSONDecodeError:
                continue

        if not req:
            continue

        msg_id = req.get("id")
        method = req.get("method", "")
        params = req.get("params", {})

        if method == "initialize":
            resp = {
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {
                    "protocolVersion": PROTOCOL_VERSION,
                    "serverInfo": SERVER_INFO,
                    "capabilities": {
                        "tools": {}
                    }
                }
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()

        elif method == "notifications/initialized":
            # Client notification acknowledgment, no response required
            pass

        elif method == "ping":
            resp = {
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {}
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()

        elif method == "tools/list":
            resp = {
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {
                    "tools": TOOLS
                }
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()

        elif method == "tools/call":
            tool_name = params.get("name", "")
            args = params.get("arguments", {})
            result = handle_tool_call(tool_name, args)

            resp = {
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": result
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()

        elif msg_id is not None:
            # Unsupported method response
            resp = {
                "jsonrpc": "2.0",
                "id": msg_id,
                "error": {
                    "code": -32601,
                    "message": f"Method not found: {method}"
                }
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()


def main():
    if len(sys.argv) > 1 and sys.argv[1] in ("--test", "--ping"):
        print("Testing connection to ZEventStudio local API server...")
        try:
            res = http_get("/api/ping")
            print(f"Connected successfully: {res}")
            status = http_get("/api/status")
            print(f"Current Status: {json.dumps(status, indent=2, ensure_ascii=False)}")
            project = http_get("/api/project")
            print(f"Current Project: {json.dumps(project, indent=2, ensure_ascii=False)}")
            files = http_post("/api/action", {"action": "list_files", "limit": 10})
            print(f"Sample Files: {json.dumps(files, indent=2, ensure_ascii=False)}")
        except Exception as e:
            print(f"Connection test failed: {e}")
            sys.exit(1)
        return

    run_stdio_server()


if __name__ == "__main__":
    main()
