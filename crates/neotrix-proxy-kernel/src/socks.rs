use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use crate::node::ProxyNode;
use crate::telemetry::{ConnectStats, now_ms};

pub async fn connect_via_socks5(
    node: &ProxyNode,
    target: &str,
    target_port: u16,
) -> anyhow::Result<(TcpStream, ConnectStats)> {
    let start = now_ms();
    let addr = node.connect_addr();

    let mut stream = TcpStream::connect(&addr).await
        .map_err(|e| anyhow::anyhow!("socks5 connect to {} failed: {}", addr, e))?;
    let _ = stream.set_nodelay(true);

    let has_auth = node.username.is_some() && node.password.is_some();
    let methods = if has_auth { vec![0x00, 0x02] } else { vec![0x00] };

    let mut msg = vec![0x05, methods.len() as u8];
    msg.extend(&methods);
    stream.write_all(&msg).await
        .map_err(|e| anyhow::anyhow!("socks5 handshake write failed: {}", e))?;

    let mut resp = [0u8; 2];
    stream.read_exact(&mut resp).await
        .map_err(|e| anyhow::anyhow!("socks5 handshake read failed: {}", e))?;

    if resp[0] != 0x05 {
        anyhow::bail!("socks5 invalid version: {}", resp[0]);
    }

    if resp[1] == 0x02 {
        let user = node.username.as_deref().unwrap_or("").as_bytes();
        let pass = node.password.as_deref().unwrap_or("").as_bytes();
        let mut auth_msg = vec![0x01, user.len() as u8];
        auth_msg.extend(user);
        auth_msg.push(pass.len() as u8);
        auth_msg.extend(pass);
        stream.write_all(&auth_msg).await
            .map_err(|e| anyhow::anyhow!("socks5 auth write failed: {}", e))?;
        let mut auth_resp = [0u8; 2];
        stream.read_exact(&mut auth_resp).await
            .map_err(|e| anyhow::anyhow!("socks5 auth read failed: {}", e))?;
        if auth_resp[1] != 0x00 {
            anyhow::bail!("socks5 auth rejected: {}", auth_resp[1]);
        }
    } else if resp[1] != 0x00 {
        anyhow::bail!("socks5 no acceptable auth method: {}", resp[1]);
    }

    let target_bytes = target.as_bytes();
    let mut connect_msg = vec![0x05, 0x01, 0x00, 0x03, target_bytes.len() as u8];
    connect_msg.extend(target_bytes);
    connect_msg.extend(&target_port.to_be_bytes());
    stream.write_all(&connect_msg).await
        .map_err(|e| anyhow::anyhow!("socks5 connect write failed: {}", e))?;

    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await
        .map_err(|e| anyhow::anyhow!("socks5 connect read header failed: {}", e))?;

    if header[1] != 0x00 {
        anyhow::bail!("socks5 connect rejected: reply={}", header[1]);
    }

    let addr_type = header[3];
    let addr_len = match addr_type {
        0x01 => 4,
        0x03 => {
            let mut len_byte = [0u8; 1];
            stream.read_exact(&mut len_byte).await?;
            len_byte[0] as usize
        }
        0x04 => 16,
        _ => anyhow::bail!("socks5 unknown address type: {}", addr_type),
    };
    let mut addr_buf = vec![0u8; addr_len + 2];
    stream.read_exact(&mut addr_buf).await
        .map_err(|e| anyhow::anyhow!("socks5 read address failed: {}", e))?;

    let elapsed = now_ms() - start;
    let stat = ConnectStats {
        protocol: crate::node::ProtocolKind::Socks5,
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
