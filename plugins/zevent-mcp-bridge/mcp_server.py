#!/usr/bin/env python3
"""
ZEventStudio Model Context Protocol (MCP) Server
Bridge between AI assistants (Claude, Cursor, Antigravity) and ZEventStudio.
Communicates with ZEventStudio local API server via HTTP on http://127.0.0.1:31415.
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

SERVER_INFO = {
    "name": "zevent-mcp-bridge",
    "version": "1.0.0"
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
        "description": "Get opened project path, file tree count, and project details",
        "inputSchema": {
            "type": "object",
            "properties": {}
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
    }
]


def http_get(path: str) -> dict:
    url = f"{API_BASE_URL}{path}"
    req = urllib.request.Request(url, headers={"User-Agent": "ZEvent-MCP-Server"})
    try:
        with urllib.request.urlopen(req, timeout=3.0) as resp:
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
        with urllib.request.urlopen(req, timeout=5.0) as resp:
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
        if tool_name == "zevent_get_status":
            res = http_get("/api/status")
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_get_project":
            res = http_get("/api/project")
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

        elif tool_name == "zevent_get_graph":
            res = http_get("/api/graph")
            return {"content": [{"type": "text", "text": json.dumps(res, indent=2, ensure_ascii=False)}]}

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
        except Exception as e:
            print(f"Connection test failed: {e}")
            sys.exit(1)
        return

    run_stdio_server()


if __name__ == "__main__":
    main()
