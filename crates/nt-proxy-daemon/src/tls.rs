use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::obfuscation::rand_u64_splitmix64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum UpstreamScheme {
    Plain,
    Tls,
}

pub(crate) struct UpstreamStream {
    pub(crate) inner: UpstreamInner,
}

pub(crate) enum UpstreamInner {
    Plain(TcpStream),
    Tls(Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>),
}

impl Read for UpstreamStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            UpstreamInner::Plain(s) => s.read(buf),
            UpstreamInner::Tls(s) => s.read(buf),
        }
    }
}

impl Write for UpstreamStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            UpstreamInner::Plain(s) => s.write(buf),
            UpstreamInner::Tls(s) => s.write(buf),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.inner {
            UpstreamInner::Plain(s) => s.flush(),
            UpstreamInner::Tls(s) => s.flush(),
        }
    }
}

fn build_config(root_store: rustls::RootCertStore, alpn: Vec<Vec<u8>>) -> Arc<rustls::ClientConfig> {
    let mut c = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    if !alpn.is_empty() {
        c.alpn_protocols = alpn;
    }
    Arc::new(c)
}

#[allow(dead_code)]
pub(crate) fn tls_config() -> &'static Arc<rustls::ClientConfig> {
    static POOL: std::sync::OnceLock<Vec<Arc<rustls::ClientConfig>>> = std::sync::OnceLock::new();
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let pool = POOL.get_or_init(|| {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        // Generate 8 configs with varied ALPN fingerprints
        let mut configs = Vec::with_capacity(8);
        let alpn_variants = vec![
            vec![],                                    // 0: no ALPN
            vec![b"http/1.1".to_vec()],               // 1: http/1.1 only
            vec![b"h2".to_vec(), b"http/1.1".to_vec()], // 2: h2 + http/1.1
            vec![b"http/1.1".to_vec(), b"h2".to_vec()], // 3: http/1.1 + h2 (reversed)
            vec![b"h2".to_vec()],                      // 4: h2 only
            vec![b"http/1.1".to_vec(), b"h2c".to_vec()], // 5: http/1.1 + h2c
            vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"h2c".to_vec()], // 6: 3 ALPN
            vec![b"http/1.1".to_vec(), b"h2".to_vec(), b"http/1.0".to_vec()], // 7: rare order
        ];
        for alpn in alpn_variants {
            configs.push(build_config(root_store.clone(), alpn));
        }
        configs
    });
    let idx = COUNTER.fetch_add(1, Ordering::Relaxed) as usize % pool.len();
    &pool[idx]
}

/// Return a random TLS config using splitmix64, bypassing round-robin predictability.
pub(crate) fn tls_config_random() -> &'static Arc<rustls::ClientConfig> {
    static POOL: std::sync::OnceLock<Vec<Arc<rustls::ClientConfig>>> = std::sync::OnceLock::new();
    let pool = POOL.get_or_init(|| {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        // 12 configs: 8 from above + 4 with same ALPN but different provider state
        let mut configs = Vec::with_capacity(12);
        let alpn_variants = vec![
            vec![],
            vec![b"http/1.1".to_vec()],
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
            vec![b"http/1.1".to_vec(), b"h2".to_vec()],
            vec![b"h2".to_vec()],
            vec![b"http/1.1".to_vec(), b"h2c".to_vec()],
            vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"h2c".to_vec()],
            vec![b"http/1.1".to_vec(), b"h2".to_vec(), b"http/1.0".to_vec()],
            vec![b"http/1.1".to_vec()],
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
            vec![b"http/1.1".to_vec(), b"h2c".to_vec()],
            vec![],
        ];
        for alpn in alpn_variants {
            configs.push(build_config(root_store.clone(), alpn));
        }
        configs
    });
    let idx = (rand_u64_splitmix64() as usize) % pool.len();
    &pool[idx]
}
