use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use super::analyzer::TrafficAnalyzer;

#[cfg(feature = "stealth-net")]
use crate::neotrix::l1_body_impl::nt_shield_stealth_net::ca_cert::{
    build_client_tls_config, build_mitm_server_config, parse_sni_from_client_hello, CaCertManager,
};

struct PeekedStream<T> {
    peeked: Vec<u8>,
    pos: usize,
    inner: T,
}

impl<T: AsyncRead + Unpin> AsyncRead for PeekedStream<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        if this.pos < this.peeked.len() {
            let n = std::cmp::min(buf.remaining(), this.peeked.len() - this.pos);
            buf.put_slice(&this.peeked[this.pos..this.pos + n]);
            this.pos += n;
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut this.inner).poll_read(cx, buf)
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for PeekedStream<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.get_mut().inner).poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}

#[derive(Debug, Clone)]
pub struct MitmProxyConfig {
    pub listen_addr: String,
    pub upstream_connect_timeout: Duration,
    pub capture_request_body: bool,
    pub capture_response_body: bool,
    pub max_body_capture: usize,
}

impl Default for MitmProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:11081".into(),
            upstream_connect_timeout: Duration::from_secs(10),
            capture_request_body: true,
            capture_response_body: true,
            max_body_capture: 65536,
        }
    }
}

pub struct MitmProxy {
    config: MitmProxyConfig,
    analyzer: Arc<Mutex<TrafficAnalyzer>>,
    mitm_enabled: bool,
}

impl MitmProxy {
    pub fn new(config: MitmProxyConfig) -> Self {
        Self {
            config,
            analyzer: Arc::new(Mutex::new(TrafficAnalyzer::new())),
            mitm_enabled: cfg!(feature = "stealth-net"),
        }
    }

    pub fn with_mitm(mut self, enabled: bool) -> Self {
        self.mitm_enabled = enabled;
        self
    }

    pub fn analyzer(&self) -> Arc<Mutex<TrafficAnalyzer>> {
        self.analyzer.clone()
    }

    pub async fn start(&self) -> Result<(), String> {
        let listener = TcpListener::bind(&self.config.listen_addr)
            .await
            .map_err(|e| format!("bind {}: {}", self.config.listen_addr, e))?;

        println!(
            "[traffic] proxy listening on {} (MITM: {})",
            self.config.listen_addr,
            if self.mitm_enabled { "enabled" } else { "passthrough" }
        );

        let analyzer = self.analyzer.clone();
        let mitm = self.mitm_enabled;
        let cfg = self.config.clone();

        loop {
            let (stream, addr) = listener.accept().await
                .map_err(|e| format!("accept: {}", e))?;

            let a = analyzer.clone();
            let c = cfg.clone();

            tokio::spawn(async move {
                let now = Instant::now();
                #[cfg(feature = "stealth-net")]
                let result = if mitm {
                    handle_one_mitm(stream, a, &c).await
                } else {
                    handle_one(stream, a, &c).await
                };
                #[cfg(not(feature = "stealth-net"))]
                let result = handle_one(stream, a, &c).await;
                if let Err(e) = result {
                    log::warn!("[traffic] {} -> {} ({:?})", addr, e, now.elapsed());
                }
            });
        }
    }
}

fn parse_connect_request(data: &[u8]) -> Option<(String, u16)> {
    let text = std::str::from_utf8(data).ok()?;
    let rest = text.strip_prefix("CONNECT ")?;
    let target = rest.split(' ').next()?;
    if let Some(pos) = target.rfind(':') {
        let port: u16 = target[pos + 1..].parse().ok()?;
        Some((target[..pos].to_string(), port))
    } else {
        Some((target.to_string(), 443))
    }
}

fn parse_http_request(data: &[u8]) -> Option<(String, String, Vec<(String, String)>)> {
    let text = std::str::from_utf8(data).ok()?;
    let mut lines = text.lines();
    let first = lines.next()?;
    let parts: Vec<&str> = first.splitn(3, ' ').collect();
    if parts.len() < 2 { return None; }
    let method = parts[0].to_string();
    let url = parts[1].to_string();
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() { break; }
        if let Some(pos) = line.find(':') {
            headers.push((line[..pos].trim().to_string(), line[pos + 1..].trim().to_string()));
        }
    }
    Some((method, url, headers))
}

fn extract_body(data: &[u8]) -> Vec<u8> {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Some(pos) = text.find("\r\n\r\n") {
            return data[(pos + 4).min(data.len())..].to_vec();
        }
    }
    Vec::new()
}

fn capture_headers(data: &[u8]) -> Vec<(String, String)> {
    let text = String::from_utf8_lossy(data);
    let mut headers = Vec::new();
    for line in text.lines().skip(1) {
        if line.is_empty() { break; }
        if let Some(pos) = line.find(':') {
            headers.push((line[..pos].trim().to_string(), line[pos + 1..].trim().to_string()));
        }
    }
    headers
}

fn parse_status_line(data: &[u8]) -> Option<(u16, String)> {
    let text = String::from_utf8_lossy(data);
    let parts: Vec<&str> = text.lines().next()?.splitn(3, ' ').collect();
    if parts.len() >= 2 {
        Some((parts[1].parse().ok()?, parts.get(2).unwrap_or(&"").to_string()))
    } else {
        None
    }
}

async fn handle_one(
    mut stream: TcpStream,
    analyzer: Arc<Mutex<TrafficAnalyzer>>,
    config: &MitmProxyConfig,
) -> Result<(), String> {
    let mut buf = vec![0u8; 16384];
    let n = tokio::time::timeout(Duration::from_secs(15), stream.read(&mut buf))
        .await.map_err(|_| "read timeout".to_string())?
        .map_err(|e| format!("read: {}", e))?;
    if n == 0 { return Ok(()); }
    let data = buf[..n].to_vec();

    if data.starts_with(b"CONNECT ") {
        connect_passthrough(stream, &data, analyzer, config).await
    } else if let Some((method, url, headers)) = parse_http_request(&data) {
        let body = extract_body(&data);
        handle_http(stream, data, &method, &url, &headers, &body, analyzer, config).await
    } else {
        Err("unknown request".to_string())
    }
}

#[cfg(feature = "stealth-net")]
async fn handle_one_mitm(
    mut stream: TcpStream,
    analyzer: Arc<Mutex<TrafficAnalyzer>>,
    config: &MitmProxyConfig,
) -> Result<(), String> {
    let mut buf = vec![0u8; 16384];
    let n = tokio::time::timeout(Duration::from_secs(15), stream.read(&mut buf))
        .await.map_err(|_| "read timeout".to_string())?
        .map_err(|e| format!("read: {}", e))?;
    if n == 0 { return Ok(()); }
    let data = buf[..n].to_vec();

    if data.starts_with(b"CONNECT ") {
        handle_connect_mitm(stream, data, analyzer, config).await
    } else if let Some((method, url, headers)) = parse_http_request(&data) {
        let body = extract_body(&data);
        handle_http(stream, data, &method, &url, &headers, &body, analyzer, config).await
    } else {
        Err("unknown request".to_string())
    }
}

async fn connect_passthrough(
    mut stream: TcpStream,
    data: &[u8],
    analyzer: Arc<Mutex<TrafficAnalyzer>>,
    config: &MitmProxyConfig,
) -> Result<(), String> {
    let (host, port) = parse_connect_request(data)
        .ok_or_else(|| "parse CONNECT".to_string())?;

    let mut upstream = tokio::time::timeout(
        config.upstream_connect_timeout,
        TcpStream::connect(format!("{}:{}", host, port)),
    ).await.map_err(|_| format!("upstream timeout {}:{}", host, port))?
     .map_err(|e| format!("upstream: {}", e))?;

    stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        .await.map_err(|e| format!("write 200: {}", e))?;

    let _sid = {
        let mut a = analyzer.lock().await;
        a.capture_request(&host, port, "TUNNEL", &format!("tcp://{}:{}", host, port), &[], b"")
    };

    let _ = tokio::io::copy_bidirectional(&mut stream, &mut upstream).await;
    Ok(())
}

#[cfg(feature = "stealth-net")]
async fn handle_connect_mitm(
    mut stream: TcpStream,
    data: Vec<u8>,
    analyzer: Arc<Mutex<TrafficAnalyzer>>,
    config: &MitmProxyConfig,
) -> Result<(), String> {
    let (host, port) = parse_connect_request(&data)
        .ok_or_else(|| "parse CONNECT".to_string())?;

    let ca = CaCertManager::new();
    if !ca.ca_cert_path.exists() {
        return connect_passthrough(stream, &data, analyzer, config).await;
    }

    stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        .await.map_err(|e| format!("write 200: {}", e))?;

    let mut buf = vec![0u8; 4096];
    let n = tokio::time::timeout(Duration::from_secs(10), stream.read(&mut buf))
        .await.map_err(|_| "tls read timeout".to_string())?
        .map_err(|e| format!("tls read: {}", e))?;
    if n == 0 { return Err("closed before ClientHello".into()); }

    let client_hello = buf[..n].to_vec();
    let sni = parse_sni_from_client_hello(&client_hello)
        .unwrap_or_else(|| host.clone());

    let server_config = build_mitm_server_config(&ca, &sni)
        .map_err(|e| format!("cert: {}", e))?;
    let client_tls_cfg = build_client_tls_config(&[])
        .map_err(|e| format!("client tls: {}", e))?;

    let peeked = PeekedStream { peeked: client_hello, pos: 0, inner: stream };
    let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
    let mut client_tls = acceptor.accept(peeked)
        .await.map_err(|e| format!("tls accept: {}", e))?;

    let upstream = tokio::time::timeout(
        config.upstream_connect_timeout,
        TcpStream::connect(format!("{}:{}", host, port)),
    ).await.map_err(|_| format!("upstream timeout {}:{}", host, port))?
     .map_err(|e| format!("upstream connect: {}", e))?;

    let domain = rustls::ServerName::try_from(host.as_str())
        .map_err(|e| format!("bad domain: {}", e))?;
    let connector = tokio_rustls::TlsConnector::from(client_tls_cfg);
    let mut upstream_tls = connector.connect(domain, upstream)
        .await.map_err(|e| format!("upstream tls: {}", e))?;

    let _sid = {
        let mut a = analyzer.lock().await;
        a.capture_request(&host, port, "CONNECT", &format!("https://{}/", host), &[], b"")
    };

    let _ = tokio::io::copy_bidirectional(&mut client_tls, &mut upstream_tls).await;
    Ok(())
}

async fn handle_http(
    mut stream: TcpStream,
    data: Vec<u8>,
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: &[u8],
    analyzer: Arc<Mutex<TrafficAnalyzer>>,
    config: &MitmProxyConfig,
) -> Result<(), String> {
    let host = headers.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("host"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();

    let mut upstream = tokio::time::timeout(
        config.upstream_connect_timeout,
        TcpStream::connect(format!("{}:{}", host, 80)),
    ).await.map_err(|_| format!("upstream timeout {}:80", host))?
     .map_err(|e| format!("upstream: {}", e))?;

    let session_id = {
        let mut a = analyzer.lock().await;
        a.capture_request(&host, 80, method, url, headers,
            if config.capture_request_body { body } else { b"" })
    };

    upstream.write_all(&data).await
        .map_err(|e| format!("write upstream: {}", e))?;

    let mut resp = Vec::new();
    let mut buf = vec![0u8; 16384];
    loop {
        match tokio::time::timeout(Duration::from_secs(30), upstream.read(&mut buf)).await {
            Ok(Ok(0)) | Err(_) => break,
            Ok(Ok(n)) => resp.extend_from_slice(&buf[..n]),
            Ok(Err(e)) => return Err(format!("resp read: {}", e)),
        }
    }

    if let Some((code, status)) = parse_status_line(&resp) {
        let rh = capture_headers(&resp);
        let rb = if config.capture_response_body {
            let b = extract_body(&resp);
            if b.len() > config.max_body_capture { b[..config.max_body_capture].to_vec() } else { b }
        } else { Vec::new() };
        let mut a = analyzer.lock().await;
        a.capture_response(session_id, code, &status, &rh, &rb);
    }

    stream.write_all(&resp).await
        .map_err(|e| format!("write response: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_connect() {
        let (h, p) = parse_connect_request(b"CONNECT api.anthropic.com:443 HTTP/1.1\r\n\r\n").unwrap();
        assert_eq!(h, "api.anthropic.com"); assert_eq!(p, 443);
    }

    #[test]
    fn test_parse_connect_no_port() {
        let (h, p) = parse_connect_request(b"CONNECT example.com HTTP/1.1\r\n\r\n").unwrap();
        assert_eq!(h, "example.com"); assert_eq!(p, 443);
    }

    #[test]
    fn test_parse_http_get() {
        let d = b"GET /index.html HTTP/1.1\r\nHost: x\r\n\r\n";
        let (m, u, _) = parse_http_request(d).unwrap();
        assert_eq!(m, "GET"); assert_eq!(u, "/index.html");
    }

    #[test]
    fn test_extract_body() {
        assert_eq!(&extract_body(b"POST / HTTP/1.1\r\nHost: x\r\nContent-Length: 5\r\n\r\nhello")[..], b"hello");
    }

    #[test]
    fn test_parse_status() {
        let (c, s) = parse_status_line(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
        assert_eq!(c, 200); assert_eq!(s, "OK");
    }

    #[test]
    fn test_capture_headers() {
        let h = capture_headers(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n");
        assert!(h.contains(&("Content-Type".into(), "application/json".into())));
    }

    #[test]
    fn test_non_connect_rejected() {
        assert!(parse_connect_request(b"GET / HTTP/1.1\r\n\r\n").is_none());
    }

    #[test]
    fn test_mitm_config_default() {
        let cfg = MitmProxyConfig::default();
        assert_eq!(cfg.listen_addr, "127.0.0.1:11081");
        assert!(cfg.capture_request_body);
    }
}
