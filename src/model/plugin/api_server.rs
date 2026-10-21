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

    /// 处理单个 HTTP 客户端连接 (支持大文件流与 Content-Length 完整帧)
    fn handle_connection(mut stream: TcpStream, sender: smol::channel::Sender<ApiCommand>) {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));

        let mut buffer = Vec::with_capacity(16384);
        let mut chunk = [0u8; 8192];
        let mut header_end = None;
        let mut content_length: usize = 0;

        // 1. 读取 HTTP 请求头部
        while header_end.is_none() {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                        header_end = Some((pos, pos + 4));
                    } else if let Some(pos) = buffer.windows(2).position(|w| w == b"\n\n") {
                        header_end = Some((pos, pos + 2));
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => break,
                Err(_) => return,
            }
        }

        let (header_pos, body_start) = match header_end {
            Some(pair) => pair,
            None => return,
        };

        // 2. 解析 Content-Length
        {
            let header_str = String::from_utf8_lossy(&buffer[..header_pos]);
            for line in header_str.lines() {
                if line.to_ascii_lowercase().starts_with("content-length:") {
                    if let Some(val_str) = line.split(':').nth(1) {
                        if let Ok(len) = val_str.trim().parse::<usize>() {
                            content_length = len;
                        }
                    }
                }
            }
        }

        // 3. 读取剩余未完成的 Body 字节 (所有 buffer 可变借用在此结束)
        let current_body_len = buffer.len().saturating_sub(body_start);
        if content_length > current_body_len {
            let mut remaining = content_length - current_body_len;
            while remaining > 0 {
                match stream.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        buffer.extend_from_slice(&chunk[..n]);
                        remaining = remaining.saturating_sub(n);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => break,
                    Err(_) => return,
                }
            }
        }

        // 4. 解析请求首行与路径
        let header_str = String::from_utf8_lossy(&buffer[..header_pos]);
        let mut lines = header_str.lines();
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

        // 提取完整 HTTP Body
        let body = if buffer.len() >= body_start {
            String::from_utf8_lossy(&buffer[body_start..])
        } else {
            std::borrow::Cow::Borrowed("")
        };

        let response = Self::dispatch_api_route(method, path, &body, &sender);
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
        let _ = stream.flush();
        let _ = stream.shutdown(std::net::Shutdown::Both);
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
Connection: close\r\n\r\n\
{}",
            status_text,
            body_str.len(),
            body_str
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
        let _ = stream.shutdown(std::net::Shutdown::Both);
    }
}
