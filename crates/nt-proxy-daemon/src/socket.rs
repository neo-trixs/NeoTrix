use std::net::TcpStream;
use std::time::{Duration, Instant};

use crate::dns::dns_cache;
use crate::socks5::parse_host_port;
use crate::ssrf::is_private_target;

const LISTEN_ADDR: &str = "127.0.0.1:11080";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const IO_TIMEOUT: Duration = Duration::from_secs(120);

pub(crate) fn create_listener() -> std::io::Result<std::net::TcpListener> {
    let listener = std::net::TcpListener::bind(LISTEN_ADDR)?;
    Ok(listener)
}

pub(crate) fn tune_socket(sock: &TcpStream) {
    let _ = sock.set_nodelay(true);
}

pub(crate) fn try_direct(target: &str) -> Result<TcpStream, String> {
    if is_private_target(target) {
        return Err(format!("private target blocked (SSRF): {target}"));
    }
    let (host, port) = parse_host_port(target).ok_or_else(|| format!("bad target: {target}"))?;
    let start = Instant::now();
    let sock_addr = dns_cache().resolve(&host, port)?;
    let sock = TcpStream::connect_timeout(&sock_addr, CONNECT_TIMEOUT)
        .map_err(|e| format!("direct connect: {e}"))?;
    tune_socket(&sock);
    let _ = sock.set_read_timeout(Some(IO_TIMEOUT));
    let _ = sock.set_write_timeout(Some(IO_TIMEOUT));
    let lat = start.elapsed().as_secs_f64() * 1000.0;
    log::error!("  direct → {target} ({lat:.0}ms)");
    Ok(sock)
}
