use super::api_types::ApiCommand;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// 本地插件 API HTTP 桥接服务器句柄
pub struct PluginApiServer {
    /// 运行状态标志
    running: Arc<AtomicBool>,
    /// 监听端口
    port: u16,
}

impl PluginApiServer {
    /// 启动本地 API 桥接服务
    pub fn start(
        port: u16,
        command_sender: smol::channel::Sender<ApiCommand>,
    ) -> Result<Self, String> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr)
            .map_err(|e| format!("无法绑定本地 API 端口 {}: {}", port, e))?;

        // 设置非阻塞或短超时，以便在停止服务时能够退出循环
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("设置套接字模式失败: {}", e))?;

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        std::thread::Builder::new()
            .name("zevent-plugin-api".to_string())
            .spawn(move || {
                Self::server_loop(listener, command_sender, running_clone);
            })
            .map_err(|e| format!("启动 API 服务器线程失败: {}", e))?;

        Ok(Self { running, port })
    }

    #[allow(dead_code)]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 停止本地 API 服务器
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// 服务器主监听循环
    fn server_loop(
        listener: TcpListener,
        sender: smol::channel::Sender<ApiCommand>,
        running: Arc<AtomicBool>,
    ) {
        while running.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let sender_clone = sender.clone();
                    // 处理单个请求 (短连接)
                    Self::handle_connection(stream, sender_clone);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // 没有新连接，休眠 30 毫秒后重试
                    std::thread::sleep(Duration::from_millis(30));
                }
                Err(_) => {
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }
    }

    /// 处理单个 HTTP 客户端连接
    fn handle_connection(mut stream: TcpStream, sender: smol::channel::Sender<ApiCommand>) {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));

        let mut buffer = [0u8; 8192];
        let bytes_read = match stream.read(&mut buffer) {
            Ok(n) if n > 0 => n,
            _ => return,
        };

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut lines = request.lines();
        let request_line = match lines.next() {
            Some(line) => line,
            None => return,
        };

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return;
        }

        let method = parts[0];
        let path = parts[1];

        // 处理 OPTIONS 跨域预检请求
        if method == "OPTIONS" {
            Self::write_cors_response(&mut stream);
            return;
        }

        // 提取 HTTP Body
        let body = if let Some(pos) = request.find("\r\n\r\n") {
            &request[pos + 4..]
        } else {
            ""
        };

        let response = Self::dispatch_api_route(method, path, body, &sender);
        Self::write_json_response(&mut stream, response);
    }

    /// 路由并分发 API 请求
    fn dispatch_api_route(
        method: &str,
        path: &str,
        body: &str,
        sender: &smol::channel::Sender<ApiCommand>,
    ) -> (u16, serde_json::Value) {
        match (method, path) {
            ("GET", "/api/ping") => (
                200,
                serde_json::json!({
                    "status": "ok",
                    "app": "ZEventStudio",
                    "version": "0.1.0"
                }),
            ),
            ("GET", "/api/status") => {
                let (resp_tx, resp_rx) = smol::channel::bounded(1);
                if sender
                    .send_blocking(ApiCommand::GetStatus { responder: resp_tx })
                    .is_ok()
                {
                    if let Ok(val) = resp_rx.recv_blocking() {
                        return (200, val);
                    }
                }
                (500, serde_json::json!({ "error": "Internal state error" }))
            }
            ("GET", "/api/project") => {
                let (resp_tx, resp_rx) = smol::channel::bounded(1);
                if sender
                    .send_blocking(ApiCommand::GetProject { responder: resp_tx })
                    .is_ok()
                {
                    if let Ok(val) = resp_rx.recv_blocking() {
                        return (200, val);
                    }
                }
                (500, serde_json::json!({ "error": "Internal state error" }))
            }
            ("GET", "/api/graph") => {
                let (resp_tx, resp_rx) = smol::channel::bounded(1);
                if sender
                    .send_blocking(ApiCommand::GetGraph { responder: resp_tx })
                    .is_ok()
                {
                    if let Ok(val) = resp_rx.recv_blocking() {
                        return (200, val);
                    }
                }
                (500, serde_json::json!({ "error": "Internal state error" }))
            }
            ("POST", "/api/action") => {
                let parsed_body: serde_json::Value = match serde_json::from_str(body) {
                    Ok(val) => val,
                    Err(e) => {
                        return (
                            400,
                            serde_json::json!({ "error": format!("Invalid JSON payload: {}", e) }),
                        );
                    }
                };

                let (resp_tx, resp_rx) = smol::channel::bounded(1);
                if sender
                    .send_blocking(ApiCommand::ExecuteAction {
                        payload: parsed_body,
                        responder: resp_tx,
                    })
                    .is_ok()
                {
                    match resp_rx.recv_blocking() {
                        Ok(Ok(val)) => (200, val),
                        Ok(Err(err)) => (400, serde_json::json!({ "error": err })),
                        Err(_) => (500, serde_json::json!({ "error": "Timeout or failed execution" })),
                    }
                } else {
                    (500, serde_json::json!({ "error": "Failed to dispatch action" }))
                }
            }
            _ => (404, serde_json::json!({ "error": "Not Found" })),
        }
    }

    /// 响应跨域 OPTIONS 预检请求
    fn write_cors_response(stream: &mut TcpStream) {
        let response = "HTTP/1.1 200 OK\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
            Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
            Content-Length: 0\r\n\
            Connection: close\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
    }

    /// 输出 JSON 格式 HTTP 响应
    fn write_json_response(stream: &mut TcpStream, (status, json_data): (u16, serde_json::Value)) {
        let body_str = json_data.to_string();
        let status_text = match status {
            200 => "200 OK",
            400 => "400 Bad Request",
            404 => "404 Not Found",
            _ => "500 Internal Server Error",
        };

        let response = format!(
            "HTTP/1.1 {}\r\n\
            Content-Type: application/json; charset=utf-8\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n{}",
            status_text,
            body_str.len(),
            body_str
        );
        let _ = stream.write_all(response.as_bytes());
    }
}
