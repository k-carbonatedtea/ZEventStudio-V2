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
if hasattr(sys.stdout, 'reconfigure'):
    try:
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
        "description": "Get current status, opened project, active document tab, and graph summary of ZEventStudio",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_get_project",
        "description": "Get opened project path, file tree summary, open tabs, and project details",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_list_files",
        "description": "Query and list all files and SARC archives in the opened project (supports filtering by extension, recursive traversal, and SARC subfile extraction)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Subdirectory path or SARC file path to list (defaults to project root)"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Recursively list all subdirectories and nested SARCs (default: true)"
                },
                "include_sarc": {
                    "type": "boolean",
                    "description": "Whether to unpack and list virtual files inside SARC/Yaz0 archives (default: true)"
                },
                "filter_ext": {
                    "type": "string",
                    "description": "Filter by file extension, e.g. '.bfevfl', '.byml', '.msbt', '.aamp'"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of file entries to return (default: 2000)"
                }
            }
        }
    },
    {
        "name": "zevent_read_file",
        "description": "Read the source code or formatted text representation of any file in the opened project or inside SARC archives (automatically decompiles BFEVFL to JSON graph, BYML/AAMP/MSBT to YAML/MSYT, and reads plain text or active in-memory editor tabs)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Target file path (can be absolute physical path, relative path in project, or SARC path like 'SARC:path.sbeventpack//Event/Demo.bfevfl')"
                },
                "format": {
                    "type": "string",
                    "enum": ["code", "raw"],
                    "description": "Format mode: 'code' (default, parses to editable code/YAML/JSON) or 'raw' (UTF-8 text)"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_write_file",
        "description": "Write, modify, or create a file or code in the opened project or inside SARC archives (automatically recompiles JSON graphs to BFEVFL, YAML to BYML/AAMP/MSBT, compresses with Yaz0 if needed, writes to disk or SARC pack, and live-synchronizes any open tabs in ZEventStudio)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Target file path (can be physical file or SARC virtual path like 'SARC:path.sbeventpack//Event/Demo.bfevfl')"
                },
                "content": {
                    "type": "string",
                    "description": "Source code, JSON graph, YAML, or text content to write"
                },
                "format": {
                    "type": "string",
                    "enum": ["bfevfl", "byml", "aamp", "msbt", "json", "yaml", "text"],
                    "description": "Format override. If omitted, auto-detected from path extension"
                },
                "endian": {
                    "type": "string",
                    "enum": ["big", "little"],
                    "description": "Byte endianness ('big' or 'little'). If omitted, automatically detected from original file or format convention"
                }
            },
            "required": ["path", "content"]
        }
    },
    {
        "name": "zevent_delete_file",
        "description": "Delete a physical file/directory or remove a virtual subfile from within a SARC archive, and close any corresponding open tabs in ZEventStudio",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path of physical file/directory or SARC virtual entry to delete"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_rename_file",
        "description": "Rename or move a physical file/directory or a subfile within a SARC archive, updating all open document tabs in ZEventStudio",
        "inputSchema": {
            "type": "object",
            "properties": {
                "old_path": {
                    "type": "string",
                    "description": "Original file or SARC virtual entry path"
                },
                "new_name": {
                    "type": "string",
                    "description": "New file name or new inner path"
                }
            },
            "required": ["old_path", "new_name"]
        }
    },
    {
        "name": "zevent_create_directory",
        "description": "Create a new directory recursively in the opened project",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to create (relative to project root or absolute)"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_search_code",
        "description": "Search for text or keywords across all files and SARC archives in the opened project (returns matching files, line numbers, and code snippets)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Keyword or string to search for"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Case sensitive search (default: false)"
                },
                "file_pattern": {
                    "type": "string",
                    "description": "Filter by file path or extension pattern, e.g. '.bfevfl' or 'Actor'"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum matching lines to return (default: 50)"
                }
            },
            "required": ["query"]
        }
    },
    {
        "name": "zevent_get_tabs",
        "description": "Get all currently opened document tabs in ZEventStudio (with tab ID, title, file path, type, active status, and modification state)",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_open_file",
        "description": "Open a file or SARC entry into an active document tab in ZEventStudio UI (supports BFEVFL flowchart view and code editor view)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path to open in editor tab"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_save_file",
        "description": "Save the currently active document tab or a specified document tab to disk or SARC archive",
        "inputSchema": {
            "type": "object",
            "properties": {
                "tab_id": {
                    "type": "string",
                    "description": "Tab ID to save. If omitted, saves the currently active document"
                }
            }
        }
    },
    {
        "name": "zevent_close_file",
        "description": "Close an opened document tab by tab ID or file path in ZEventStudio",
        "inputSchema": {
            "type": "object",
            "properties": {
                "tab_id": {
                    "type": "string",
                    "description": "Tab ID to close"
                },
                "path": {
                    "type": "string",
                    "description": "File path of tab to close"
                }
            }
        }
    },
    {
        "name": "zevent_reload_project",
        "description": "Rescan and refresh the current project file tree and Git version control status in ZEventStudio",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_open_project",
        "description": "Open and load a new project directory (Mod root) or SARC pack file (.sbeventpack, .sbactorpack, .pack) into ZEventStudio",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Project directory path or pack file path to open"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_get_graph",
        "description": "Get the complete event flowchart graph (nodes, edges, actors, properties) for the active document",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_add_node",
        "description": "Add a new node (Action, Query, Switch, Fork, Join, SubFlow, EntryPoint, Comment) to the active flowchart in ZEventStudio",
        "inputSchema": {
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Node title/action name (e.g. 'Demo_PlayAnimation')"
                },
                "kind": {
                    "type": "string",
                    "enum": ["Action", "Query", "Switch", "Fork", "Join", "SubFlow", "EntryPoint", "Comment"],
                    "description": "Node type. Default is 'Action'"
                },
                "actor": {
                    "type": "string",
                    "description": "Actor name for this node (e.g. 'Link', 'WorldManager')"
                },
                "x": {
                    "type": "number",
                    "description": "Canvas X position"
                },
                "y": {
                    "type": "number",
                    "description": "Canvas Y position"
                },
                "params": {
                    "type": "object",
                    "description": "Parameters dictionary { key: value }"
                }
            },
            "required": ["name"]
        }
    },
    {
        "name": "zevent_connect_nodes",
        "description": "Connect two nodes with a directed edge in the active flowchart",
        "inputSchema": {
            "type": "object",
            "properties": {
                "from_node": {
                    "type": "string",
                    "description": "Source node ID"
                },
                "from_port": {
                    "type": "string",
                    "description": "Source port name (default: 'out')"
                },
                "to_node": {
                    "type": "string",
                    "description": "Target node ID"
                },
                "to_port": {
                    "type": "string",
                    "description": "Target port name (default: 'in')"
                }
            },
            "required": ["from_node", "to_node"]
        }
    },
    {
        "name": "zevent_update_node",
        "description": "Update properties, title, or actor of an existing node in the active flowchart",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node_id": {
                    "type": "string",
                    "description": "Target node ID"
                },
                "name": {
                    "type": "string",
                    "description": "New title/action name"
                },
                "actor": {
                    "type": "string",
                    "description": "New actor name"
                },
                "params": {
                    "type": "object",
                    "description": "Parameters dictionary { key: value }"
                }
            },
            "required": ["node_id"]
        }
    },
    {
        "name": "zevent_delete_node",
        "description": "Delete a node and its connected edges from the active flowchart",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node_id": {
                    "type": "string",
                    "description": "ID of node to delete"
                }
            },
            "required": ["node_id"]
        }
    },
    {
        "name": "zevent_auto_layout",
        "description": "Automatically calculate Sugiyama hierarchical layout for the active flowchart and fit view",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    },
    {
        "name": "zevent_toast",
        "description": "Display a popup toast notification inside the ZEventStudio UI",
        "inputSchema": {
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to show"
                },
                "level": {
                    "type": "string",
                    "enum": ["info", "success", "error"],
                    "description": "Notification level"
                }
            },
            "required": ["message"]
        }
    },
    {
        "name": "zevent_query_byml",
        "description": "Query or inspect BYML data structures using JSON pointer without transferring massive YAML files (supports BotW QuestProduct, GameData, SaveData, Mubin, etc.)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Target BYML file path (physical or SARC virtual path like 'SARC:Pack/TitleBG.pack//Quest/QuestProduct.sbquestpack')"
                },
                "query_path": {
                    "type": "string",
                    "description": "JSON pointer to query, e.g. '/Quest', '/Quest/170', '/data/bool_data' (defaults to root '')"
                },
                "summary_only": {
                    "type": "boolean",
                    "description": "If true, returns array length or object keys instead of full recursive tree"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_patch_byml",
        "description": "Perform high-performance targeted modification (append, set, delete) on a BYML file without rewriting the entire file (supports Big/Little Endian, auto Yaz0 compression, and SARC packing)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Target BYML file path (physical or SARC virtual path like 'SARC:Pack/TitleBG.pack//Quest/QuestProduct.sbquestpack')"
                },
                "patch_op": {
                    "type": "string",
                    "enum": ["append", "set", "delete"],
                    "description": "Patch operation: 'append' to push to an array, 'set' to update/assign a value, 'delete' to remove a key/index"
                },
                "query_path": {
                    "type": "string",
                    "description": "JSON pointer path where operation applies, e.g. '/Quest' for append, or '/Quest/170/Name' for set"
                },
                "value": {
                    "description": "JSON value to set or append (can be string, number, boolean, array, or object)"
                },
                "endian": {
                    "type": "string",
                    "enum": ["big", "little"],
                    "description": "Target endianness ('big' or 'little'). If omitted, automatically detected from original file or format convention"
                }
            },
            "required": ["path", "patch_op"]
        }
    },
    {
        "name": "zevent_read_msbt",
        "description": "Read dialogue and text entries from an MSBT localization file as a key-value dictionary (supports plain text extraction, SARC archives, and auto Yaz0 decompression)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Target MSBT file path (physical or SARC virtual path like 'SARC:Pack/Bootup_CNzh.pack//QuestMsg/QL_Test.msbt')"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "zevent_write_msbt",
        "description": "Write or batch update text entries into an MSBT localization file (automatically creates new MSBT or updates existing entries, packages into SARC, and compresses with Yaz0 if needed)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Target MSBT file path (physical or SARC virtual path like 'SARC:Pack/Bootup_CNzh.pack//QuestMsg/QL_Test.msbt')"
                },
                "entries": {
                    "type": "object",
                    "description": "Dictionary of label-text pairs to write, e.g. {'QL_Test_Name': '测试任务', 'QL_Test_Desc': '任务描述'}"
                },
                "endian": {
                    "type": "string",
                    "enum": ["big", "little"],
                    "description": "Target endianness ('little' for Switch, 'big' for WiiU). If omitted, automatically detected from original file or defaults to little"
                }
            },
            "required": ["path", "entries"]
        }
    },
    {
        "name": "zevent_pack_sarc",
        "description": "Recursively pack a physical directory of files into a SARC (.pack / .sarc / .ssarc) archive (supports Yaz0 compression and Big/Little Endian)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "source_dir": {
                    "type": "string",
                    "description": "Source physical directory path containing files to pack"
                },
                "target_file": {
                    "type": "string",
                    "description": "Target SARC pack file path (e.g. 'Pack/TitleBG.pack')"
                },
                "endian": {
                    "type": "string",
                    "enum": ["big", "little"],
                    "description": "Endianness of SARC header and filenames table (default: 'big')"
                },
                "compress_yaz0": {
                    "type": "boolean",
                    "description": "Whether to compress the resulting SARC archive with Yaz0 (default: true for .s... extensions)"
                }
            },
            "required": ["source_dir", "target_file"]
        }
    },
    {
        "name": "zevent_unpack_sarc",
        "description": "Unpack all files from a SARC (.pack / .sarc / .ssarc) archive or virtual sub-directory into a destination physical directory",
        "inputSchema": {
            "type": "object",
            "properties": {
                "source_file": {
                    "type": "string",
                    "description": "Path of SARC archive to extract (physical file or SARC virtual entry)"
                },
                "target_dir": {
                    "type": "string",
                    "description": "Destination physical directory path"
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
