use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use crate::dns::dns_cache;
use crate::obfuscation::{jitter_sleep, socks5_greeting_padded};
use crate::socket::tune_socket;
use crate::ssrf::is_private_ip;
use crate::tls::{tls_config_random, UpstreamInner, UpstreamScheme, UpstreamStream};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);
const IO_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Copy)]
pub(crate) enum SocksAuth {
    NoAuth,
    UserPass,
}

pub(crate) fn parse_socks5_url(url: &str) -> Option<(String, u16, bool, UpstreamScheme)> {
    let is_local = url.starts_with("socks5://");
    let is_tls = url.starts_with("socks5+tls://") || url.starts_with("socks5h+tls://");
    let stripped = url
        .strip_prefix("socks5+tls://")
        .or_else(|| url.strip_prefix("socks5h+tls://"))
        .or_else(|| url.strip_prefix("socks5://"))
        .or_else(|| url.strip_prefix("socks5h://"))?;
    let scheme = if is_tls {
        UpstreamScheme::Tls
    } else {
        UpstreamScheme::Plain
    };
    if stripped.is_empty() {
        return None;
    }
    if stripped.starts_with('[') {
        let close_bracket = stripped.find(']')?;
        if close_bracket == 1 {
            return None;
        }
        let host = stripped[1..close_bracket].to_string();
        let port = stripped[close_bracket + 1..]
            .strip_prefix(':')?
            .parse()
            .ok()?;
        if port == 0 {
            return None;
        }
        Some((host, port, is_local, scheme))
    } else {
        let (host, port_str) = stripped.rsplit_once(':')?;
        if host.is_empty() {
            return None;
        }
        let port: u16 = port_str.parse().ok()?;
        if port == 0 {
            return None;
        }
        Some((host.to_string(), port, is_local, scheme))
    }
}

pub(crate) fn parse_socks5_auth(url: &str) -> Option<(String, String)> {
    let stripped = url
        .strip_prefix("socks5+tls://")
        .or_else(|| url.strip_prefix("socks5h+tls://"))
        .or_else(|| url.strip_prefix("socks5://"))
        .or_else(|| url.strip_prefix("socks5h://"))?;
    let (auth, _hostport) = stripped.split_once('@')?;
    if auth.is_empty() {
        return None;
    }
    if let Some((user, pass)) = auth.split_once(':') {
        Some((user.to_string(), pass.to_string()))
    } else {
        Some((auth.to_string(), String::new()))
    }
}

pub(crate) fn parse_host_port(input: &str) -> Option<(String, u16)> {
    if input.is_empty() {
        return None;
    }
    if input.starts_with('[') {
        let close = input.find(']')?;
        if close == 1 {
            return None;
        }
        let host = input[1..close].to_string();
        let port = input[close + 1..].strip_prefix(':')?.parse().ok()?;
        if port == 0 {
            return None;
        }
        Some((host, port))
    } else {
        let (host, port_str) = input.rsplit_once(':')?;
        if host.is_empty() {
            return None;
        }
        let port: u16 = port_str.parse().ok()?;
        Some((host.to_string(), port))
    }
}

fn socks5_handshake(sock: &mut TcpStream, user: Option<&str>, pass: Option<&str>) -> Result<SocksAuth, String> {
    let has_auth = user.is_some();
    let greeting = if has_auth {
        vec![5, 2, 0, 2]
    } else {
        socks5_greeting_padded()
    };
    sock.write_all(&greeting)
        .map_err(|e| format!("handshake write: {e}"))?;
    let mut buf = [0u8; 2];
    sock.read_exact(&mut buf)
        .map_err(|e| format!("handshake read: {e}"))?;
    if buf[0] != 5 {
        return Err(format!("bad SOCKS version: {}", buf[0]));
    }
    match buf[1] {
        0 => Ok(SocksAuth::NoAuth),
        2 => {
            let u = user.unwrap_or("");
            let p = pass.unwrap_or("");
            if u.len() > 255 || p.len() > 255 {
                return Err("auth credentials too long".to_string());
            }
            let mut cred = Vec::with_capacity(3 + u.len() + p.len());
            cred.push(1);
            cred.push(u.len() as u8);
            cred.extend_from_slice(u.as_bytes());
            cred.push(p.len() as u8);
            cred.extend_from_slice(p.as_bytes());
            sock.write_all(&cred)
                .map_err(|e| format!("auth write: {e}"))?;
            let mut auth_resp = [0u8; 2];
            sock.read_exact(&mut auth_resp)
                .map_err(|e| format!("auth read: {e}"))?;
            if auth_resp[1] != 0 {
                return Err(format!("auth rejected: {:?}", auth_resp));
            }
            Ok(SocksAuth::UserPass)
        }
        0xff => Err("no acceptable auth method".to_string()),
        m => Err(format!("unexpected auth method: {m}")),
    }
}

fn socks5_read_response(sock: &mut TcpStream) -> Result<(), String> {
    let mut resp = [0u8; 4];
    sock.read_exact(&mut resp)
        .map_err(|e| format!("read header: {e}"))?;
    if resp[0] != 5 {
        return Err(format!("bad SOCKS version: {}", resp[0]));
    }
    if resp[1] != 0 {
        let reason = match resp[1] {
            1 => "general failure",
            2 => "not allowed",
            3 => "network unreachable",
            4 => "host unreachable",
            5 => "connection refused",
            6 => "TTL expired",
            7 => "command not supported",
            8 => "address type not supported",
            _ => "unknown",
        };
        return Err(format!("rejected ({reason})"));
    }
    if resp[2] != 0 {
        return Err(format!("bad reserved byte: {}", resp[2]));
    }

    let addr_type = resp[3];
    let rest_len = match addr_type {
        1 => 6,
        3 => {
            let mut len_byte = [0u8; 1];
            sock.read_exact(&mut len_byte)
                .map_err(|e| format!("read addr len: {e}"))?;
            (len_byte[0] as usize) + 2
        }
        4 => 18,
        _ => return Err(format!("unsupported address type: {addr_type}")),
    };
    let mut rest = vec![0u8; rest_len];
    sock.read_exact(&mut rest)
        .map_err(|e| format!("read addr: {e}"))?;
    Ok(())
}

fn socks5_connect_ip(sock: &mut TcpStream, addr: &SocketAddr) -> Result<(), String> {
    let msg = match addr {
        SocketAddr::V4(v4) => {
            let octets = v4.ip().octets();
            let mut m = Vec::with_capacity(10);
            m.extend_from_slice(&[5, 1, 0, 1]);
            m.extend_from_slice(&octets);
            m.extend_from_slice(&v4.port().to_be_bytes());
            m
        }
        SocketAddr::V6(v6) => {
            let segments = v6.ip().segments();
            let mut m = Vec::with_capacity(22);
            m.extend_from_slice(&[5, 1, 0, 4]);
            for s in &segments {
                m.extend_from_slice(&s.to_be_bytes());
            }
            m.extend_from_slice(&v6.port().to_be_bytes());
            m
        }
    };
    sock.write_all(&msg)
        .map_err(|e| format!("connect cmd write: {e}"))?;
    socks5_read_response(sock)
}

fn socks5_connect_cmd(sock: &mut TcpStream, target: &str) -> Result<(), String> {
    let (host, port) = parse_host_port(target).ok_or_else(|| format!("bad target: {target}"))?;
    let host_bytes = host.as_bytes();
    if host_bytes.len() > 255 {
        return Err("hostname too long".to_string());
    }
    let mut msg = Vec::with_capacity(7 + host_bytes.len());
    msg.extend_from_slice(&[5, 1, 0, 3, host_bytes.len() as u8]);
    msg.extend_from_slice(host_bytes);
    msg.extend_from_slice(&port.to_be_bytes());
    sock.write_all(&msg)
        .map_err(|e| format!("connect cmd write: {e}"))?;
    socks5_read_response(sock)
}

fn socks5_handshake_io(io: &mut (impl Read + Write), user: Option<&str>, pass: Option<&str>) -> Result<SocksAuth, String> {
    let has_auth = user.is_some();
    let greeting = if has_auth {
        vec![5, 2, 0, 2]
    } else {
        socks5_greeting_padded()
    };
    io.write_all(&greeting)
        .map_err(|e| format!("handshake write: {e}"))?;
    let mut buf = [0u8; 2];
    io.read_exact(&mut buf)
        .map_err(|e| format!("handshake read: {e}"))?;
    if buf[0] != 5 {
        return Err(format!("bad SOCKS version: {}", buf[0]));
    }
    match buf[1] {
        0 => Ok(SocksAuth::NoAuth),
        2 => {
            let u = user.unwrap_or("");
            let p = pass.unwrap_or("");
            if u.len() > 255 || p.len() > 255 {
                return Err("auth credentials too long".to_string());
            }
            let mut cred = Vec::with_capacity(3 + u.len() + p.len());
            cred.push(1);
            cred.push(u.len() as u8);
            cred.extend_from_slice(u.as_bytes());
            cred.push(p.len() as u8);
            cred.extend_from_slice(p.as_bytes());
            io.write_all(&cred)
                .map_err(|e| format!("auth write: {e}"))?;
            let mut auth_resp = [0u8; 2];
            io.read_exact(&mut auth_resp)
                .map_err(|e| format!("auth read: {e}"))?;
            if auth_resp[1] != 0 {
                return Err(format!("auth rejected: {:?}", auth_resp));
            }
            Ok(SocksAuth::UserPass)
        }
        0xff => Err("no acceptable auth method".to_string()),
        m => Err(format!("unexpected auth method: {m}")),
    }
}

fn socks5_read_response_io(io: &mut (impl Read + Write)) -> Result<(), String> {
    let mut resp = [0u8; 4];
    io.read_exact(&mut resp)
        .map_err(|e| format!("read header: {e}"))?;
    if resp[0] != 5 {
        return Err(format!("bad SOCKS version: {}", resp[0]));
    }
    if resp[1] != 0 {
        let reason = match resp[1] {
            1 => "general failure",
            2 => "not allowed",
            3 => "network unreachable",
            4 => "host unreachable",
            5 => "connection refused",
            6 => "TTL expired",
            7 => "command not supported",
            8 => "address type not supported",
            _ => "unknown",
        };
        return Err(format!("rejected ({reason})"));
    }
    if resp[2] != 0 {
        return Err(format!("bad reserved byte: {}", resp[2]));
    }
    let addr_type = resp[3];
    let rest_len = match addr_type {
        1 => 6,
        3 => {
            let mut len_byte = [0u8; 1];
            io.read_exact(&mut len_byte)
                .map_err(|e| format!("read addr len: {e}"))?;
            (len_byte[0] as usize) + 2
        }
        4 => 18,
        _ => return Err(format!("unsupported address type: {addr_type}")),
    };
    let mut rest = vec![0u8; rest_len];
    io.read_exact(&mut rest)
        .map_err(|e| format!("read addr: {e}"))?;
    Ok(())
}

fn socks5_connect_ip_io(io: &mut (impl Read + Write), addr: &SocketAddr) -> Result<(), String> {
    let msg = match addr {
        SocketAddr::V4(v4) => {
            let octets = v4.ip().octets();
            let mut m = Vec::with_capacity(10);
            m.extend_from_slice(&[5, 1, 0, 1]);
            m.extend_from_slice(&octets);
            m.extend_from_slice(&v4.port().to_be_bytes());
            m
        }
        SocketAddr::V6(v6) => {
            let segments = v6.ip().segments();
            let mut m = Vec::with_capacity(22);
            m.extend_from_slice(&[5, 1, 0, 4]);
            for s in &segments {
                m.extend_from_slice(&s.to_be_bytes());
            }
            m.extend_from_slice(&v6.port().to_be_bytes());
            m
        }
    };
    io.write_all(&msg)
        .map_err(|e| format!("connect cmd write: {e}"))?;
    socks5_read_response_io(io)
}

fn socks5_connect_cmd_io(io: &mut (impl Read + Write), target: &str) -> Result<(), String> {
    let (host, port) = parse_host_port(target).ok_or_else(|| format!("bad target: {target}"))?;
    let host_bytes = host.as_bytes();
    if host_bytes.len() > 255 {
        return Err("hostname too long".to_string());
    }
    let mut msg = Vec::with_capacity(7 + host_bytes.len());
    msg.extend_from_slice(&[5, 1, 0, 3, host_bytes.len() as u8]);
    msg.extend_from_slice(host_bytes);
    msg.extend_from_slice(&port.to_be_bytes());
    io.write_all(&msg)
        .map_err(|e| format!("connect cmd write: {e}"))?;
    socks5_read_response_io(io)
}

/// Resolve hostname to a single IP and pin it (DNS pinning for local DNS mode).
fn resolve_and_pin(target: &str) -> Result<(SocketAddr, std::net::IpAddr), String> {
    let (host, port) = parse_host_port(target).ok_or_else(|| format!("bad target: {target}"))?;
    let sock_addr = dns_cache().resolve(&host, port)?;
    let pinned_ip = sock_addr.ip();
    Ok((sock_addr, pinned_ip))
}

pub(crate) fn socks5_connect(upstream_url: &str, target: &str) -> Result<UpstreamStream, String> {
    let (proxy_host, proxy_port, is_local_dns, scheme) =
        parse_socks5_url(upstream_url).ok_or_else(|| format!("bad upstream: {}", upstream_url))?;

    let socket_addr = dns_cache().resolve(&proxy_host, proxy_port)?;

    let start = Instant::now();

    jitter_sleep();

    let mut sock = TcpStream::connect_timeout(&socket_addr, CONNECT_TIMEOUT)
        .map_err(|e| format!("connect: {e} ({proxy_host}:{proxy_port})"))?;

    let latency = start.elapsed().as_secs_f64() * 1000.0;
    tune_socket(&sock);
    sock.set_read_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|_| sock.set_write_timeout(Some(CONTROL_TIMEOUT)))
        .ok();

    let (auth_user, auth_pass) = parse_socks5_auth(upstream_url)
        .map(|(u, p)| (Some(u), Some(p)))
        .unwrap_or((None, None));

    if scheme == UpstreamScheme::Tls {
        let server_name = rustls::pki_types::ServerName::try_from(proxy_host.clone())
            .map_err(|_| format!("invalid hostname for TLS SNI: {proxy_host}"))?;
        // Use random config for TLS fingerprint diversity
        let conn = rustls::ClientConnection::new(tls_config_random().clone(), server_name)
            .map_err(|e| format!("TLS create: {e}"))?;
        let mut tls_stream = rustls::StreamOwned::new(conn, sock);
        tls_stream
            .write(&[])
            .map_err(|e| format!("TLS handshake: {e}"))?;
        socks5_handshake_io(&mut tls_stream, auth_user.as_deref(), auth_pass.as_deref())?;
        if is_local_dns {
            let (resolved_addr, pinned_ip) = resolve_and_pin(target)?;
            if is_private_ip(pinned_ip) {
                return Err(format!("private target blocked (SSRF): {target}"));
            }
            socks5_connect_ip_io(&mut tls_stream, &resolved_addr)?;
        } else {
            socks5_connect_cmd_io(&mut tls_stream, target)?;
        }
        log::error!("  SOCKS5+TLS {proxy_host}:{proxy_port} → {target} ({latency:.0}ms)");
        return Ok(UpstreamStream {
            inner: UpstreamInner::Tls(Box::new(tls_stream)),
        });
    }

    socks5_handshake(&mut sock, auth_user.as_deref(), auth_pass.as_deref())?;
    if is_local_dns {
        let (resolved_addr, pinned_ip) = resolve_and_pin(target)?;
        if is_private_ip(pinned_ip) {
            return Err(format!("private target blocked (SSRF): {target}"));
        }
        socks5_connect_ip(&mut sock, &resolved_addr)?;
    } else {
        socks5_connect_cmd(&mut sock, target)?;
    }
    sock.set_read_timeout(Some(IO_TIMEOUT))
        .and_then(|_| sock.set_write_timeout(Some(IO_TIMEOUT)))
        .ok();

    log::error!("  SOCKS5 {proxy_host}:{proxy_port} → {target} ({latency:.0}ms)");
    Ok(UpstreamStream {
        inner: UpstreamInner::Plain(sock),
    })
}
