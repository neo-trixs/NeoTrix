pub mod node;
pub mod socks;
pub mod httpproxy;
pub mod telemetry;

use node::ProxyNode;
use telemetry::ConnectStats;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait ProxyStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> ProxyStream for T {}

pub type BoxedStream = Box<dyn ProxyStream>;

pub async fn connect_through(
    node: &ProxyNode,
    target: &str,
    target_port: u16,
) -> anyhow::Result<(BoxedStream, ConnectStats)> {
    match node.protocol {
        node::ProtocolKind::Socks5 => {
            let (stream, stat) = socks::connect_via_socks5(node, target, target_port).await?;
            Ok((Box::new(stream) as BoxedStream, stat))
        }
        node::ProtocolKind::Http => {
            let (stream, stat) = httpproxy::connect_via_http(node, target, target_port).await?;
            Ok((Box::new(stream) as BoxedStream, stat))
        }
        node::ProtocolKind::Shadowsocks => {
            ss::connect_via_shadowsocks(node, target, target_port).await
        }
        node::ProtocolKind::Trojan => {
            trojan::connect_via_trojan(node, target, target_port).await
        }
        node::ProtocolKind::Hysteria2 => {
            hysteria2::connect_via_hysteria2(node, target, target_port).await
        }
        node::ProtocolKind::VLess => {
            vless::connect_via_vless(node, target, target_port).await
        }
        node::ProtocolKind::VMess => {
            vmess::connect_via_vmess(node, target, target_port).await
        }
        _ => anyhow::bail!("protocol {:?} not natively supported", node.protocol),
    }
}

pub mod ss {
    use crate::node::ProxyNode;
    use crate::telemetry::{ConnectStats, now_ms};
    use crate::BoxedStream;
    use shadowsocks::crypto::CipherKind;


    pub async fn connect_via_shadowsocks(
        node: &ProxyNode,
        target: &str,
        target_port: u16,
    ) -> anyhow::Result<(BoxedStream, ConnectStats)> {
        use shadowsocks::config::ServerConfig;
        use shadowsocks::context::Context;

        let method = node.method.as_deref().unwrap_or("aes-256-gcm");
        let cipher = match method {
            "aes-128-gcm" => CipherKind::AES_128_GCM,
            "aes-256-gcm" => CipherKind::AES_256_GCM,
            "chacha20-ietf-poly1305" | "chacha20-poly1305" => CipherKind::CHACHA20_POLY1305,
            _ => CipherKind::AES_256_GCM,
        };

        let svr_addrs = tokio::net::lookup_host(node.connect_addr()).await
            .map_err(|e| anyhow::anyhow!("ss lookup failed: {}", e))?;
        let svr_addr = svr_addrs.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("no addresses for {}", node.connect_addr()))?;

        let pwd = node.password.as_deref().unwrap_or("");
        let svr = ServerConfig::new((svr_addr.ip().to_string(), svr_addr.port()), pwd, cipher)
            .map_err(|e| anyhow::anyhow!("shadowsocks config error: {}", e))?;

        let ctx = Context::new_shared(shadowsocks::config::ServerType::Local);
        let start = now_ms();
        let target_addr: shadowsocks::relay::Address = format!("{}:{}", target, target_port)
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid target addr: {}", e))?;

        let stream = shadowsocks::relay::tcprelay::proxy_stream::ProxyClientStream::connect(ctx, &svr, target_addr).await
            .map_err(|e| anyhow::anyhow!("shadowsocks connect failed: {}", e))?;

        let elapsed = now_ms() - start;
        let stat = ConnectStats {
            protocol: crate::node::ProtocolKind::Shadowsocks,
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

        Ok((Box::new(stream) as BoxedStream, stat))
    }
}

pub mod trojan {
    use tokio::io::AsyncWriteExt;
    use tokio_rustls::TlsConnector;
    use rustls::ClientConfig;
    use std::sync::Arc;
    use crate::node::ProxyNode;
    use crate::telemetry::{ConnectStats, now_ms};
    use crate::BoxedStream;
    use tokio::net::TcpStream;

    #[derive(Debug)]
    pub(crate) struct NoVerifier;

    impl rustls::client::danger::ServerCertVerifier for NoVerifier {
        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            vec![
                rustls::SignatureScheme::RSA_PKCS1_SHA256,
                rustls::SignatureScheme::RSA_PKCS1_SHA384,
                rustls::SignatureScheme::RSA_PKCS1_SHA512,
                rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
                rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
                rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
                rustls::SignatureScheme::RSA_PSS_SHA256,
                rustls::SignatureScheme::RSA_PSS_SHA384,
                rustls::SignatureScheme::RSA_PSS_SHA512,
                rustls::SignatureScheme::ED25519,
            ]
        }

        fn verify_server_cert(
            &self,
            _end_entity: &rustls_pki_types::CertificateDer<'_>,
            _intermediates: &[rustls_pki_types::CertificateDer<'_>],
            _server_name: &rustls_pki_types::ServerName<'_>,
            _ocsp_response: &[u8],
            _now: rustls_pki_types::UnixTime,
        ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &rustls_pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &rustls_pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }
    }

    pub async fn connect_via_trojan(
        node: &ProxyNode,
        target: &str,
        target_port: u16,
    ) -> anyhow::Result<(BoxedStream, ConnectStats)> {
        use sha2::{Sha224, Digest};

        let start = now_ms();
        let addr = node.connect_addr();

        let raw = TcpStream::connect(&addr).await
            .map_err(|e| anyhow::anyhow!("trojan tcp connect failed: {}", e))?;
        let _ = raw.set_nodelay(true);

        let tls_config = if node.skip_cert_verify {
            ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoVerifier))
                .with_no_client_auth()
        } else {
            let mut root_store = rustls::RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth()
        };

        let sni = node.sni.clone().unwrap_or_else(|| node.server.clone());
        let connector = TlsConnector::from(Arc::new(tls_config));
        let domain = rustls_pki_types::ServerName::try_from(sni)
            .map_err(|e| anyhow::anyhow!("invalid trojan SNI: {}", e))?;
        let tls_stream = connector.connect(domain, raw).await
            .map_err(|e| anyhow::anyhow!("trojan tls handshake failed: {}", e))?;
        let mut tls = tokio_rustls::TlsStream::Client(tls_stream);

        let pass = node.password.as_deref().unwrap_or("");
        let mut hasher = Sha224::new();
        hasher.update(pass.as_bytes());
        let hash = hasher.finalize();
        let hex_hash: String = hash.iter().map(|b| format!("{:02x}", b)).collect();

        let target_bytes = target.as_bytes();
        let req = if target_bytes.contains(&0x00) || target_bytes.is_empty() {
            anyhow::bail!("trojan requires domain target");
        } else {
            let mut buf = vec![0x03, target_bytes.len() as u8];
            buf.extend(target_bytes);
            buf.extend(&target_port.to_be_bytes());
            buf
        };

        let mut trojan_request = format!("{}\r\n", hex_hash).into_bytes();
        trojan_request.extend_from_slice(&req);
        tls.write_all(&trojan_request).await
            .map_err(|e| anyhow::anyhow!("trojan auth+request write failed: {}", e))?;

        let stat = ConnectStats {
            protocol: crate::node::ProtocolKind::Trojan,
            server: node.server.clone(),
            port: node.port,
            target: format!("{}:{}", target, target_port),
            success: true,
            latency_ms: now_ms() - start,
            bytes_sent: 0,
            bytes_recv: 0,
            error: None,
            timestamp_ms: start,
        };

        Ok((Box::new(tls) as BoxedStream, stat))
    }
}

pub mod hysteria2 {
    use crate::node::ProxyNode;
    use crate::telemetry::{ConnectStats, now_ms};
    use crate::BoxedStream;

    pub async fn connect_via_hysteria2(
        node: &ProxyNode,
        target: &str,
        target_port: u16,
    ) -> anyhow::Result<(BoxedStream, ConnectStats)> {
        let start = now_ms();
        let cfg = hysteria2::config::Config {
            auth: node.password.clone().unwrap_or_default(),
            server_addr: node.connect_addr(),
            server_name: node.sni.clone().unwrap_or_else(|| node.server.clone()),
            insecure: node.skip_cert_verify,
            port_hopping_range: None,
        };

        let client = hysteria2::network::connect(&cfg).await
            .map_err(|e| anyhow::anyhow!("hysteria2 connect failed: {}", e))?;

        let target_addr = format!("{}:{}", target, target_port);
        let stream = client.tcp_connect(&target_addr).await
            .map_err(|e| anyhow::anyhow!("hysteria2 tcp_connect failed: {}", e))?;

        let elapsed = now_ms() - start;
        let stat = ConnectStats {
            protocol: crate::node::ProtocolKind::Hysteria2,
            server: node.server.clone(),
            port: node.port,
            target: target_addr,
            success: true,
            latency_ms: elapsed,
            bytes_sent: 0,
            bytes_recv: 0,
            error: None,
            timestamp_ms: start,
        };

        Ok((Box::new(stream) as BoxedStream, stat))
    }
}

/// Helper: build VLESS/VMess target address bytes
fn build_addr_bytes(target: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(1 + target.len() + 2);
    if let Ok(v4) = target.parse::<std::net::Ipv4Addr>() {
        buf.push(0x01);
        buf.extend_from_slice(&v4.octets());
    } else if let Ok(v6) = target.parse::<std::net::Ipv6Addr>() {
        buf.push(0x04);
        buf.extend_from_slice(&v6.octets());
    } else {
        buf.push(0x03);
        buf.push(target.len() as u8);
        buf.extend_from_slice(target.as_bytes());
    }
    buf
}

pub mod vless {
    use crate::node::ProxyNode;
    use crate::telemetry::{ConnectStats, now_ms};
    use crate::BoxedStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    pub async fn connect_via_vless(
        node: &ProxyNode,
        target: &str,
        target_port: u16,
    ) -> anyhow::Result<(BoxedStream, ConnectStats)> {
        let start = now_ms();
        let addr = node.connect_addr();

        let raw = TcpStream::connect(&addr).await
            .map_err(|e| anyhow::anyhow!("vless tcp connect failed: {}", e))?;
        let _ = raw.set_nodelay(true);

        let uuid_str = node.uuid.as_deref().unwrap_or("");
        let uuid = uuid::Uuid::parse_str(uuid_str)
            .map_err(|e| anyhow::anyhow!("invalid vless uuid '{}': {}", uuid_str, e))?
            .into_bytes();

        let mut stream: BoxedStream = if node.tls {
            let tls_config = if node.skip_cert_verify {
                let mut cfg = rustls::ClientConfig::builder()
                    .dangerous()
                    .with_custom_certificate_verifier(std::sync::Arc::new(super::trojan::NoVerifier))
                    .with_no_client_auth();
                cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
                cfg
            } else {
                let mut root_store = rustls::RootCertStore::empty();
                root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                let mut cfg = rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();
                cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
                cfg
            };
            let connector = tokio_rustls::TlsConnector::from(std::sync::Arc::new(tls_config));
            let sni = node.sni.clone().unwrap_or_else(|| node.server.clone());
            let domain = rustls_pki_types::ServerName::try_from(sni)
                .map_err(|e| anyhow::anyhow!("invalid vless SNI: {}", e))?;
            let tls_stream = connector.connect(domain, raw).await
                .map_err(|e| anyhow::anyhow!("vless tls handshake failed: {}", e))?;
            Box::new(tokio_rustls::TlsStream::Client(tls_stream)) as BoxedStream
        } else {
            Box::new(raw) as BoxedStream
        };

        // VLESS request: version(0) + uuid(16) + addons_len(0) + cmd(1) + port(2) + addr_type(1) + addr
        let mut req = Vec::with_capacity(512);
        req.push(0x00);
        req.extend_from_slice(&uuid);
        req.push(0x00);
        req.push(0x01);
        req.extend_from_slice(&target_port.to_be_bytes());
        req.extend_from_slice(&crate::build_addr_bytes(target));

        stream.write_all(&req).await
            .map_err(|e| anyhow::anyhow!("vless request write failed: {}", e))?;

        // Response: version(1) + addons_len(1) + [addons]
        let mut resp = [0u8; 2];
        stream.read_exact(&mut resp).await
            .map_err(|e| anyhow::anyhow!("vless response read failed: {}", e))?;
        let addons_len = resp[1] as usize;
        if addons_len > 0 {
            let mut addons = vec![0u8; addons_len];
            stream.read_exact(&mut addons).await
                .map_err(|e| anyhow::anyhow!("vless addons read failed: {}", e))?;
        }

        let elapsed = now_ms() - start;
        let stat = ConnectStats {
            protocol: crate::node::ProtocolKind::VLess,
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
}

pub mod vmess {
    use crate::node::ProxyNode;
    use crate::telemetry::{ConnectStats, now_ms};
    use crate::BoxedStream;
    use aes_gcm::{Aes128Gcm, KeyInit, Nonce};
    use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
    use tokio::net::TcpStream;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    fn md5(data: &[u8]) -> [u8; 16] {
        use md5::Md5;
        use hmac::digest::Digest;
        let mut h = Md5::new();
        h.update(data);
        let r = h.finalize();
        let mut out = [0u8; 16];
        out.copy_from_slice(&r);
        out
    }

    fn hmac_md5(key: &[u8], msg: &[u8]) -> [u8; 16] {
        use hmac::Mac;
        use md5::Md5;
        let mut mac = <hmac::Hmac<Md5> as Mac>::new_from_slice(key).expect("HMAC-MD5 key");
        mac.update(msg);
        let r = mac.finalize().into_bytes();
        let mut out = [0u8; 16];
        out.copy_from_slice(&r);
        out
    }

    fn fnv1a(data: &[u8]) -> u32 {
        use fnv::FnvHasher;
        use std::hash::Hasher;
        let mut h = FnvHasher::default();
        h.write(data);
        h.finish() as u32
    }

    fn aes128_cfb_encrypt(key: &[u8; 16], iv: &[u8; 16], data: &[u8]) -> Vec<u8> {
        use aes::cipher::{AsyncStreamCipher, KeyIvInit};
        type Aes128CfbEnc = cfb_mode::Encryptor<aes::Aes128>;
        let cipher = Aes128CfbEnc::new_from_slices(key, iv).expect("AES-128-CFB");
        let mut out = data.to_vec();
        cipher.encrypt(&mut out);
        out
    }

    fn aes128_cfb_decrypt(key: &[u8; 16], iv: &[u8; 16], data: &[u8]) -> Vec<u8> {
        use aes::cipher::{AsyncStreamCipher, KeyIvInit};
        type Aes128CfbDec = cfb_mode::Decryptor<aes::Aes128>;
        let cipher = Aes128CfbDec::new_from_slices(key, iv).expect("AES-128-CFB");
        let mut out = data.to_vec();
        cipher.decrypt(&mut out);
        out
    }

    fn derive_cmd_key(uuid: &[u8; 16]) -> [u8; 16] {
        let mut input = uuid.to_vec();
        input.extend_from_slice(b"c48619fe-8f02-49e0-b9e9-edf763e17e21");
        md5(&input)
    }

    fn derive_cmd_iv(ts_bytes: &[u8; 8]) -> [u8; 16] {
        let mut input = ts_bytes.to_vec();
        input.extend_from_slice(ts_bytes);
        input.extend_from_slice(ts_bytes);
        input.extend_from_slice(ts_bytes);
        md5(&input)
    }

    fn pick_encryption(cipher_name: Option<&str>) -> u8 {
        match cipher_name {
            Some("aes-128-gcm") | Some("auto") | None => 0x02,
            Some("chacha20-poly1305") | Some("chacha20-ietf-poly1305") => 0x03,
            Some("none") | Some("aes-128-cfb") => 0x00,
            _ => 0x02, // default to AES-128-GCM
        }
    }

    /// VMess encrypted data section stream wrapper.
    /// Handles chunked format with AES-128-GCM per-chunk encryption.
    struct VmessStream<S> {
        inner: S,
        enc_key: [u8; 16],
        enc_iv: [u8; 16],
        enc_type: u8,
        write_count: u16,
        read_count: u16,
        read_buf: Vec<u8>,
        read_pos: usize,
    }

    impl<S: AsyncRead + AsyncWrite + Unpin> VmessStream<S> {
        fn new(inner: S, key: [u8; 16], iv: [u8; 16], enc_type: u8) -> Self {
            Self {
                inner,
                enc_key: key,
                enc_iv: iv,
                enc_type,
                write_count: 0,
                read_count: 0,
                read_buf: Vec::new(),
                read_pos: 0,
            }
        }
    }

    impl<S: AsyncRead + AsyncWrite + Unpin + Send> AsyncRead for VmessStream<S> {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let this = self.get_mut();

            // If we have buffered data, serve from it
            if this.read_pos < this.read_buf.len() {
                let avail = std::cmp::min(buf.remaining(), this.read_buf.len() - this.read_pos);
                buf.put_slice(&this.read_buf[this.read_pos..this.read_pos + avail]);
                this.read_pos += avail;
                if this.read_pos >= this.read_buf.len() {
                    this.read_buf.clear();
                    this.read_pos = 0;
                }
                return Poll::Ready(Ok(()));
            }

            // Read a VMess chunk: 2-byte length
            let mut len_buf = [0u8; 2];
            let pin = Pin::new(&mut this.inner);
            match pin.poll_read(cx, &mut tokio::io::ReadBuf::new(&mut len_buf)) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
            let l = u16::from_be_bytes(len_buf) as usize;

            if l == 0 {
                return Poll::Ready(Ok(()));
            }

            // Read the data portion
            let read_len = l;
            let mut chunk = vec![0u8; read_len];
            let pin = Pin::new(&mut this.inner);
            match pin.poll_read(cx, &mut tokio::io::ReadBuf::new(&mut chunk)) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }

            // Decrypt based on encryption type
            let plain = match this.enc_type {
                0x02 => {
                    // AES-128-GCM: content = ciphertext(actual) + tag(16) = full AEAD payload
                    use aes_gcm::aead::Aead;
                    if chunk.len() < 16 {
                        this.read_buf = chunk;
                        this.read_pos = 0;
                        Vec::new()
                    } else {
                        let nonce_bytes = &this.enc_iv[2..12];
                        let mut iv = this.read_count.to_be_bytes().to_vec();
                        iv.extend_from_slice(nonce_bytes);
                        let nonce = Nonce::from_slice(&iv);
                        let key = aes_gcm::Key::<Aes128Gcm>::from_slice(&this.enc_key);
                        let cipher = Aes128Gcm::new(key);
                        match cipher.decrypt(nonce, chunk.as_ref()) {
                            Ok(plain) => plain,
                            Err(_) => chunk.to_vec(),
                        }
                    }
                }
                _ => chunk, // unsupported/unknown: pass through
            };

            // Update read count for next chunk's nonce
            this.read_count = this.read_count.wrapping_add(1);

            // Serve from newly decrypted buffer
            this.read_buf = plain;
            this.read_pos = 0;
            let avail = std::cmp::min(buf.remaining(), this.read_buf.len());
            buf.put_slice(&this.read_buf[..avail]);
            this.read_pos += avail;
            if this.read_pos >= this.read_buf.len() {
                this.read_buf.clear();
                this.read_pos = 0;
            }
            Poll::Ready(Ok(()))
        }
    }

    impl<S: AsyncRead + AsyncWrite + Unpin + Send> AsyncWrite for VmessStream<S> {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            let this = self.get_mut();

            // Encrypt data based on encryption type
            let encrypted = match this.enc_type {
                0x02 => {
                    let nonce_bytes = &this.enc_iv[2..12];
                    let mut iv = this.write_count.to_be_bytes().to_vec();
                    iv.extend_from_slice(nonce_bytes);
                    let nonce = Nonce::from_slice(&iv);
                    let key = aes_gcm::Key::<Aes128Gcm>::from_slice(&this.enc_key);
                    let cipher = Aes128Gcm::new(key);
                    
                    {
                        // A basic GCM encrypt without the full AEAD interface
                        // We'll manually construct: ciphertext + tag
                        use aes_gcm::aead::Aead;
                        cipher.encrypt(nonce, buf).unwrap_or_else(|_| buf.to_vec())
                    }
                }
                _ => buf.to_vec(),
            };

            this.write_count = this.write_count.wrapping_add(1);

            // Write: 2-byte length + encrypted data
            let len = std::cmp::min(encrypted.len(), 16384) as u16;
            let mut header = len.to_be_bytes().to_vec();
            header.extend_from_slice(&encrypted[..len as usize]);

            let pin = Pin::new(&mut this.inner);
            pin.poll_write(cx, &header)
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            Pin::new(&mut self.get_mut().inner).poll_flush(cx)
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            // Send empty chunk to signal end
            let this = self.get_mut();
            let header = [0u8; 2];
            let pin = Pin::new(&mut this.inner);
            let _ = pin.poll_write(cx, &header);
            Pin::new(&mut this.inner).poll_shutdown(cx)
        }
    }

    pub async fn connect_via_vmess(
        node: &ProxyNode,
        target: &str,
        target_port: u16,
    ) -> anyhow::Result<(BoxedStream, ConnectStats)> {
        use std::time::SystemTime;

        let start = now_ms();
        let addr = node.connect_addr();

        let raw = TcpStream::connect(&addr).await
            .map_err(|e| anyhow::anyhow!("vmess tcp connect failed: {}", e))?;
        let _ = raw.set_nodelay(true);

        // Optionally wrap in TLS
        let stream: BoxedStream = if node.tls {
            let tls_config = if node.skip_cert_verify {
                let mut cfg = rustls::ClientConfig::builder()
                    .dangerous()
                    .with_custom_certificate_verifier(std::sync::Arc::new(super::trojan::NoVerifier))
                    .with_no_client_auth();
                cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
                cfg
            } else {
                let mut root_store = rustls::RootCertStore::empty();
                root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                let mut cfg = rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();
                cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
                cfg
            };
            let connector = tokio_rustls::TlsConnector::from(std::sync::Arc::new(tls_config));
            let sni = node.sni.clone().unwrap_or_else(|| node.server.clone());
            let domain = rustls_pki_types::ServerName::try_from(sni)
                .map_err(|e| anyhow::anyhow!("invalid vmess SNI: {}", e))?;
            let tls_stream = connector.connect(domain, raw).await
                .map_err(|e| anyhow::anyhow!("vmess tls handshake failed: {}", e))?;
            Box::new(tokio_rustls::TlsStream::Client(tls_stream)) as BoxedStream
        } else {
            Box::new(raw) as BoxedStream
        };

        // Step 1: Generate timestamp with jitter
        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("time error: {}", e))?
            .as_secs();
        let jitter: u64 = (rand::random::<u64>() % 61) as u64; // 0-60 seconds
        let ts = if jitter < 30 { now_secs + jitter } else { now_secs - (jitter - 30) };
        let ts_bytes = ts.to_be_bytes();

        // Step 2: Authentication info - HMAC-MD5(UUID, timestamp)
        let uuid_str = node.uuid.as_deref().unwrap_or("");
        let uuid = uuid::Uuid::parse_str(uuid_str)
            .map_err(|e| anyhow::anyhow!("invalid vmess uuid '{}': {}", uuid_str, e))?
            .into_bytes();
        let auth = hmac_md5(&uuid, &ts_bytes);

        // Step 3: Command section encryption
        let cmd_key = derive_cmd_key(&uuid);
        let cmd_iv = derive_cmd_iv(&ts_bytes);

        // Generate random command section fields
        let req_iv: [u8; 16] = rand::random();
        let req_key: [u8; 16] = rand::random();
        let resp_v: u8 = rand::random();
        let enc_type = pick_encryption(node.cipher.as_deref());
        let opt: u8 = 0x01; // Standard format only

        // Build command section plaintext
        let mut cmd = Vec::with_capacity(256);
        cmd.push(0x01); // Version
        cmd.extend_from_slice(&req_iv); // Request IV
        cmd.extend_from_slice(&req_key); // Request Key
        cmd.push(resp_v); // Response Auth V
        cmd.push(opt); // Option
        cmd.push(0x00); // Padding length P = 0
        cmd.push(enc_type); // Encryption type
        cmd.push(0x00); // Reserved
        cmd.push(0x01); // Command: TCP
        cmd.extend_from_slice(&target_port.to_be_bytes());
        cmd.extend_from_slice(&crate::build_addr_bytes(target));
        // Padding (P=0 so none)
        // FNV1a checksum
        let checksum = fnv1a(&cmd);
        cmd.extend_from_slice(&checksum.to_be_bytes());

        // Encrypt command section
        let encrypted_cmd = aes128_cfb_encrypt(&cmd_key, &cmd_iv, &cmd);

        // Step 4: Write auth + encrypted command
        let mut writer = stream;
        writer.write_all(&auth).await
            .map_err(|e| anyhow::anyhow!("vmess auth write failed: {}", e))?;
        writer.write_all(&encrypted_cmd).await
            .map_err(|e| anyhow::anyhow!("vmess cmd write failed: {}", e))?;
        writer.flush().await.ok();

        // Step 5: Read and decrypt response (4 bytes)
        let resp_key = md5(&req_key);
        let resp_iv = md5(&req_iv);
        let mut resp_enc = [0u8; 4];
        // Use read_exact on writer (which is the stream)
        use tokio::io::AsyncReadExt;
        let reader = &mut writer;
        reader.read_exact(&mut resp_enc).await
            .map_err(|e| anyhow::anyhow!("vmess response read failed: {}", e))?;
        let resp_plain = aes128_cfb_decrypt(&resp_key, &resp_iv, &resp_enc);
        let _resp_v = resp_plain[0]; // Should match our resp_v

        // Step 6: Wrap in VmessStream for data section handling
        let vmess_stream = VmessStream::new(writer, req_key, req_iv, enc_type);

        let elapsed = now_ms() - start;
        let stat = ConnectStats {
            protocol: crate::node::ProtocolKind::VMess,
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

        Ok((Box::new(vmess_stream) as BoxedStream, stat))
    }
}
