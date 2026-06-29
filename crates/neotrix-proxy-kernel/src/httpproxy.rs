use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use crate::node::ProxyNode;
use crate::telemetry::{ConnectStats, now_ms};

pub async fn connect_via_http(
    node: &ProxyNode,
    target: &str,
    target_port: u16,
) -> anyhow::Result<(TcpStream, ConnectStats)> {
    let start = now_ms();
    let addr = node.connect_addr();

    let mut stream = TcpStream::connect(&addr).await
        .map_err(|e| anyhow::anyhow!("http proxy connect to {} failed: {}", addr, e))?;
    let _ = stream.set_nodelay(true);

    let host_header = format!("{}:{}", target, target_port);
    let mut request = format!(
        "CONNECT {} HTTP/1.1\r\nHost: {}\r\n",
        host_header, host_header
    ).into_bytes();

    if node.username.is_some() && node.password.is_some() {
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        let auth = format!("{}:{}", node.username.as_deref().unwrap_or(""), node.password.as_deref().unwrap_or(""));
        let encoded = engine.encode(auth);
        request.extend_from_slice(format!("Proxy-Authorization: Basic {}\r\n", encoded).as_bytes());
    }
    request.extend_from_slice(b"\r\n");

    stream.write_all(&request).await
        .map_err(|e| anyhow::anyhow!("http proxy CONNECT write failed: {}", e))?;

    let mut resp = Vec::with_capacity(512);
    let mut buf = [0u8; 256];
    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 { break; }
        resp.extend_from_slice(&buf[..n]);
        if resp.len() >= 4 && resp[resp.len()-4..] == *b"\r\n\r\n" {
            break;
        }
        if resp.len() > 8192 {
            anyhow::bail!("http proxy response too large");
        }
    }

    let resp_str = String::from_utf8_lossy(&resp);
    let status_line = resp_str.lines().next().unwrap_or("");
    let status_code = status_line.split(' ').nth(1).unwrap_or("0");
    let code: u16 = status_code.parse().unwrap_or(0);

    if code != 200 {
        anyhow::bail!("http proxy CONNECT rejected: {} {}", code, status_line.trim());
    }

    let elapsed = now_ms() - start;
    let stat = ConnectStats {
        protocol: crate::node::ProtocolKind::Http,
        server: node.server.clone(),
        port: node.port,
        target: format!("{}:{}", target, target_port),
        success: true,
        latency_ms: elapsed,
        bytes_sent: 0,
        bytes_recv: 0,
        error: None,
        timestamp_ms: start,
    };

    Ok((stream, stat))
}
