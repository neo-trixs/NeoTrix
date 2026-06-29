use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

use crate::obfuscation::jitter_sleep;
use crate::socket::tune_socket;
use crate::socks5::socks5_connect;
use crate::ssrf::is_private_target;
use crate::tls::{UpstreamInner, UpstreamStream};

const CONN_LIFETIME: Duration = Duration::from_secs(120);

pub(crate) fn try_upstream(upstream_url: &str, target: &str) -> Result<UpstreamStream, String> {
    if is_private_target(target) {
        return Err(format!("private target blocked (SSRF): {target}"));
    }
    let start = Instant::now();
    let sock = socks5_connect(upstream_url, target)?;
    let lat = start.elapsed().as_secs_f64() * 1000.0;
    log::error!("  connect {lat:.0}ms");
    Ok(sock)
}

pub(crate) fn relay_with_lifetime(a: &mut TcpStream, b: &mut TcpStream, start: Instant) {
    let remaining = CONN_LIFETIME.saturating_sub(start.elapsed());
    if remaining <= Duration::ZERO {
        return;
    }

    // Timing jitter: small random delay before relay starts
    jitter_sleep();

    let mut a_try = match a.try_clone() {
        Ok(c) => {
            tune_socket(&c);
            c
        }
        Err(_) => return,
    };
    let mut b_try = match b.try_clone() {
        Ok(c) => {
            tune_socket(&c);
            c
        }
        Err(_) => return,
    };

    let h1 = thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(move || {
            pipe_with_timeout(&mut a_try, &mut b_try, remaining);
        })
        .unwrap_or_else(|_| thread::spawn(|| {}));
    pipe_with_timeout(b, a, remaining);
    let _ = h1.join();
}

pub(crate) fn relay_upstream(client: &mut TcpStream, remote: &mut UpstreamStream, start: Instant) {
    match &mut remote.inner {
        UpstreamInner::Plain(s) => relay_with_lifetime(client, s, start),
        UpstreamInner::Tls(t) => relay_tls_stream(client, t, start),
    }
}

fn relay_tls_stream(client: &mut TcpStream, tls: &mut (impl Read + Write), start: Instant) {
    use crate::obfuscation::{jitter_sleep, padding_bytes, rand_range};

    let remaining = CONN_LIFETIME.saturating_sub(start.elapsed());
    if remaining <= Duration::ZERO {
        return;
    }
    let deadline = Instant::now() + remaining;

    jitter_sleep();
    let _ = client.set_read_timeout(Some(Duration::from_millis(50)));

    let mut buf = vec![0u8; 65536];
    let mut bytes_relayed = 0u64;
    loop {
        if Instant::now() >= deadline {
            break;
        }

        match client.read(&mut buf) {
            Ok(0) => {
                let _ = tls.flush();
                break;
            }
            Ok(n) => {
                bytes_relayed += n as u64;
                if tls.write_all(&buf[..n]).is_err() {
                    break;
                }
                if bytes_relayed % 65536 < n as u64 && rand_range(100) < 20 {
                    let pad = padding_bytes(16, 256);
                    let _ = tls.write_all(&pad);
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }

        match tls.read(&mut buf) {
            Ok(0) => {
                let _ = client.flush();
                break;
            }
            Ok(n) => {
                if client.write_all(&buf[..n]).is_err() {
                    break;
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }
    }
}

/// Relay between a TCP client and a MuxStream (HTTP/2 CONNECT tunnel).
///
/// Mirrors `relay_tls_stream` but works with any `Read + Write` peer, since
/// `MuxStream` internally bridges to an h2 stream via a Unix socket pair.
/// Includes the same padding obfuscation as the TLS relay path.
#[cfg(feature = "mux")]
pub(crate) fn relay_mux_stream(
    client: &mut TcpStream,
    mux: &mut (impl Read + Write),
    start: Instant,
) {
    use crate::obfuscation::{jitter_sleep, padding_bytes, rand_range};

    let remaining = CONN_LIFETIME.saturating_sub(start.elapsed());
    if remaining <= Duration::ZERO {
        return;
    }
    let deadline = Instant::now() + remaining;

    jitter_sleep();
    let _ = client.set_read_timeout(Some(Duration::from_millis(50)));

    let mut buf = vec![0u8; 65536];
    let mut bytes_relayed = 0u64;
    loop {
        if Instant::now() >= deadline {
            break;
        }

        // Client → Mux (downstream → upstream)
        match client.read(&mut buf) {
            Ok(0) => {
                let _ = mux.flush();
                break;
            }
            Ok(n) => {
                bytes_relayed += n as u64;
                if mux.write_all(&buf[..n]).is_err() {
                    break;
                }
                // Random padding injection (same as TLS relay)
                if bytes_relayed % 65536 < n as u64 && rand_range(100) < 20 {
                    let pad = padding_bytes(16, 256);
                    let _ = mux.write_all(&pad);
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }

        // Mux → Client (upstream → downstream)
        match mux.read(&mut buf) {
            Ok(0) => {
                let _ = client.flush();
                break;
            }
            Ok(n) => {
                if client.write_all(&buf[..n]).is_err() {
                    break;
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }
    }
}

fn pipe_with_timeout(r: &mut TcpStream, w: &mut TcpStream, max_duration: Duration) {
    use crate::obfuscation::{padding_bytes, rand_range};

    let deadline = Instant::now() + max_duration;
    let mut buf = vec![0u8; 65536];
    let mut last_to = max_duration;
    let mut bytes_relayed = 0u64;
    let _ = r.set_read_timeout(Some(max_duration));
    loop {
        if Instant::now() >= deadline {
            break;
        }
        let timeout_left = deadline.saturating_duration_since(Instant::now());
        if timeout_left.as_secs() < last_to.as_secs() - 1 {
            let _ = r.set_read_timeout(Some(timeout_left));
            last_to = timeout_left;
        }
        match r.read(&mut buf) {
            Ok(0) => {
                let _ = w.shutdown(std::net::Shutdown::Write);
                break;
            }
            Ok(n) => {
                bytes_relayed += n as u64;
                if w.write_all(&buf[..n]).is_err() {
                    break;
                }
                // Random padding injection: after every ~64KB, 20% chance to pad
                if bytes_relayed % 65536 < n as u64 && rand_range(100) < 20 {
                    let pad = padding_bytes(16, 256);
                    let _ = w.write_all(&pad);
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(_) => break,
        }
    }
}
