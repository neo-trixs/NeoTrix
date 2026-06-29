use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::pool::ProxyPool;

const IO_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug)]
pub(crate) enum HttpError {
    IoError,
    TooLong,
    InvalidMethod,
    InvalidTarget,
    NoRequest,
}

#[derive(Debug)]
pub(crate) struct HttpRequest {
    pub(crate) method: String,
    pub(crate) target: String,
}

pub(crate) fn parse_http_request(stream: &mut TcpStream) -> Result<HttpRequest, HttpError> {
    stream.set_read_timeout(Some(Duration::from_secs(3))).ok();

    let mut buf = Vec::with_capacity(256);
    let mut byte = [0u8; 1];
    loop {
        match stream.read(&mut byte) {
            Ok(0) => return Err(HttpError::NoRequest),
            Ok(_) => {
                if byte[0] == b'\n' {
                    break;
                }
                if buf.len() >= 512 {
                    return Err(HttpError::TooLong);
                }
                buf.push(byte[0]);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(HttpError::IoError),
        }
    }

    let line = std::str::from_utf8(&buf).map_err(|_| HttpError::InvalidMethod)?;
    let line = line.trim_end_matches('\r');
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or(HttpError::InvalidMethod)?.to_string();
    let target = parts.next().ok_or(HttpError::InvalidTarget)?.to_string();

    if method != "CONNECT" && method != "GET" {
        return Err(HttpError::InvalidMethod);
    }
    if method == "CONNECT" && !target.contains(':') {
        return Err(HttpError::InvalidTarget);
    }

    let mut state: u8 = 0;
    loop {
        match stream.read(&mut byte) {
            Ok(0) => break,
            Ok(_) => match state {
                0 => {
                    if byte[0] == b'\r' {
                        state = 1;
                    }
                }
                1 => {
                    if byte[0] == b'\n' {
                        state = 2;
                    } else if byte[0] == b'\r' { /* stay */
                    } else {
                        state = 0;
                    }
                }
                2 => {
                    if byte[0] == b'\r' {
                        state = 3;
                    } else {
                        state = 0;
                    }
                }
                3 => {
                    if byte[0] == b'\n' {
                        break;
                    } else if byte[0] == b'\r' {
                        state = 1;
                    } else {
                        state = 0;
                    }
                }
                _ => break,
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }

    stream
        .set_read_timeout(Some(IO_TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(IO_TIMEOUT)))
        .ok();

    Ok(HttpRequest { method, target })
}

pub(crate) fn send_status(client: &mut TcpStream, pool: &ProxyPool) {
    let body = pool.status_json();
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(), body
    );
    let _ = client.write_all(resp.as_bytes());
}

pub(crate) fn send_error(client: &mut TcpStream, status: u16, message: &str) {
    let resp = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status, message, message.len(), message
    );
    let _ = client.write_all(resp.as_bytes());
}
