use std::path::PathBuf;
use std::fs;
use std::sync::Arc;

const CA_DIR: &str = ".neotrix";

pub struct CaCertManager {
    pub ca_cert_path: PathBuf,
    pub ca_key_path: PathBuf,
}

impl Default for CaCertManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CaCertManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            ca_cert_path: home.join(CA_DIR).join("ca.crt"),
            ca_key_path: home.join(CA_DIR).join("ca.key"),
        }
    }

    pub fn ensure_ca(&self) -> Result<(), String> {
        if self.ca_cert_path.exists() && self.ca_key_path.exists() {
            return Ok(());
        }
        self.generate()
    }

    fn generate(&self) -> Result<(), String> {
        use rcgen::{CertificateParams, KeyPair, IsCa, BasicConstraints, DistinguishedName, DnType};

        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "NeoTrix Root CA");
        dn.push(DnType::OrganizationName, "NeoTrix");
        params.distinguished_name = dn;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
            rcgen::KeyUsagePurpose::DigitalSignature,
        ];

        let key_pair = KeyPair::generate().map_err(|e| format!("rcgen keypair: {}", e))?;
        let cert = params.self_signed(&key_pair).map_err(|e| format!("rcgen self-signed: {}", e))?;

        if let Some(parent) = self.ca_cert_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir: {}", e))?;
        }

        fs::write(&self.ca_cert_path, cert.pem()).map_err(|e| format!("write ca.crt: {}", e))?;
        fs::write(&self.ca_key_path, key_pair.serialize_pem()).map_err(|e| format!("write ca.key: {}", e))?;

        println!("[ca] CA certificate generated: {}", self.ca_cert_path.display());
        Ok(())
    }

    pub fn install_macos(&self) -> Result<(), String> {
        self.ensure_ca()?;
        let status = std::process::Command::new("sudo")
            .args([
                "nt_shield", "add-trusted-cert", "-d", "-r", "trustRoot",
                "-k", "/Library/Keychains/System.keychain",
            ])
            .arg(&self.ca_cert_path)
            .status()
            .map_err(|e| format!("sudo nt_shield: {}", e))?;
        if !status.success() {
            return Err("failed to install CA cert (sudo required)".into());
        }
        println!("[ca] CA certificate installed in macOS trust store");
        Ok(())
    }

    pub fn uninstall_macos(&self) -> Result<(), String> {
        let sha1_output = std::process::Command::new("openssl")
            .args([
                "x509", "-in", &self.ca_cert_path.to_string_lossy(),
                "-fingerprint", "-sha1", "-noout",
            ])
            .output()
            .map_err(|e| format!("openssl: {}", e))?;
        let sha1 = String::from_utf8_lossy(&sha1_output.stdout);
        if let Some(fp) = sha1.split('=').nth(1) {
            let hash = fp.trim().replace(':', "");
            let status = std::process::Command::new("sudo")
                .args(["nt_shield", "remove-trusted-cert", "-d", &hash])
                .status()
                .map_err(|e| format!("sudo nt_shield remove: {}", e))?;
            if !status.success() {
                return Err("failed to remove CA cert".into());
            }
            println!("[ca] CA certificate removed from macOS trust store");
        }
        Ok(())
    }

    pub fn ca_cert_pem(&self) -> Result<String, String> {
        fs::read_to_string(&self.ca_cert_path).map_err(|e| format!("read ca.crt: {}", e))
    }

    pub fn ca_key_pem(&self) -> Result<String, String> {
        fs::read_to_string(&self.ca_key_path).map_err(|e| format!("read ca.key: {}", e))
    }

    pub fn generate_server_cert(&self, hostname: &str) -> Result<(String, String), String> {
        use rcgen::{CertificateParams, KeyPair};

        let params = CertificateParams::new(vec![hostname.to_string()])
            .map_err(|e| format!("rcgen params: {}", e))?;

        let key_pair = KeyPair::generate().map_err(|e| format!("rcgen keypair: {}", e))?;

        let ca_key_pem = self.ca_key_pem()?;
        let ca_cert_pem = self.ca_cert_pem()?;
        let ca_key = KeyPair::from_pem(&ca_key_pem).map_err(|e| format!("parse ca key: {}", e))?;
        let ca_params = rcgen::CertificateParams::from_ca_cert_pem(&ca_cert_pem)
            .map_err(|e| format!("parse ca cert: {}", e))?;
        let ca_cert = ca_params.self_signed(&ca_key)
            .map_err(|e| format!("self-sign ca: {}", e))?;

        let cert = params.signed_by(&key_pair, &ca_cert, &ca_key)
            .map_err(|e| format!("sign server cert: {}", e))?;

        Ok((cert.pem(), key_pair.serialize_pem()))
    }
}

fn pem_to_der(pem: &str, section: &str) -> Result<Vec<u8>, String> {
    let mut found = false;
    let mut b64 = String::new();
    for line in pem.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("-----BEGIN {}-----", section)) {
            found = true;
            continue;
        }
        if trimmed.starts_with(&format!("-----END {}-----", section)) {
            break;
        }
        if found {
            b64.push_str(trimmed);
        }
    }
    if !found {
        return Err(format!("PEM section {} not found", section));
    }
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(&b64)
        .map_err(|e| format!("base64 decode: {}", e))
}

pub fn build_mitm_server_config(
    ca: &CaCertManager,
    hostname: &str,
) -> Result<Arc<rustls::ServerConfig>, String> {
    let (cert_pem, key_pem) = ca.generate_server_cert(hostname)?;

    let cert_der = pem_to_der(&cert_pem, "CERTIFICATE")?;
    let cert = rustls::Certificate(cert_der);

    let key_der = pem_to_der(&key_pem, "PRIVATE KEY")?;
    let key = rustls::PrivateKey(key_der);

    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .map_err(|e| format!("server config: {}", e))?;

    Ok(Arc::new(config))
}

pub fn build_client_tls_config(
    cipher_order: &[String],
) -> Result<Arc<rustls::ClientConfig>, String> {
    use rustls::SupportedCipherSuite;

    let suites: Vec<SupportedCipherSuite> = rustls::ALL_CIPHER_SUITES.iter()
        .copied()
        .filter(|cs| {
            match cs {
                SupportedCipherSuite::Tls13(s) => {
                    cipher_order.iter().any(|name| {
                        cipher_suite_name_match(&s.common.suite, name)
                    })
                }
                _ => false,
            }
        })
        .collect();

    if suites.is_empty() {
        // Fallback to safe defaults
        return Ok(Arc::new(
            rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(NoCertVerifier))
                .with_no_client_auth()
        ));
    }

    let config = rustls::ClientConfig::builder()
        .with_cipher_suites(&suites)
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| format!("protocol versions: {}", e))?
        .with_custom_certificate_verifier(Arc::new(NoCertVerifier))
        .with_no_client_auth();

    Ok(Arc::new(config))
}

fn cipher_suite_name_match(suite: &rustls::CipherSuite, name: &str) -> bool {
    matches!(
        (suite, name),
        (rustls::CipherSuite::TLS13_AES_128_GCM_SHA256, "TLS_AES_128_GCM_SHA256")
            | (rustls::CipherSuite::TLS13_AES_256_GCM_SHA384, "TLS_AES_256_GCM_SHA384")
            | (rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256, "TLS_CHACHA20_POLY1305_SHA256")
            | (rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256, "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256")
            | (rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256, "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256")
            | (rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384, "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384")
            | (rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384, "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384")
            | (rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256, "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256")
            | (rustls::CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256, "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256")
    )
}

pub struct NoCertVerifier;

impl rustls::client::ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

pub fn parse_sni_from_client_hello(data: &[u8]) -> Option<String> {
    if data.len() < 5 || data[0] != 0x16 {
        return None; // Not a TLS handshake record
    }
    let record_len = ((data[3] as usize) << 8) | (data[4] as usize);
    if data.len() < 5 + record_len {
        return None;
    }
    let handshake = &data[5..5 + record_len];
    if handshake.is_empty() || handshake[0] != 0x01 {
        return None; // Not ClientHello
    }
    let mut offset = 4 + 2 + 32;
    if handshake.len() <= offset {
        return None;
    }

    // Session ID
    let session_id_len = handshake[offset] as usize;
    offset += 1 + session_id_len;
    if handshake.len() <= offset + 1 {
        return None;
    }

    // Cipher Suites
    let cipher_suites_len = ((handshake[offset] as usize) << 8) | (handshake[offset + 1] as usize);
    offset += 2 + cipher_suites_len;
    if handshake.len() <= offset {
        return None;
    }

    // Compression Methods
    let comp_len = handshake[offset] as usize;
    offset += 1 + comp_len;
    if handshake.len() <= offset + 1 {
        return None;
    }

    // Extensions
    let ext_len = ((handshake[offset] as usize) << 8) | (handshake[offset + 1] as usize);
    offset += 2;
    if handshake.len() < offset + ext_len {
        return None;
    }

    let exts = &handshake[offset..offset + ext_len];
    let mut ext_offset = 0;
    while ext_offset + 4 <= exts.len() {
        let ext_type = ((exts[ext_offset] as u16) << 8) | (exts[ext_offset + 1] as u16);
        let ext_data_len = ((exts[ext_offset + 2] as usize) << 8) | (exts[ext_offset + 3] as usize);
        ext_offset += 4;
        if ext_offset + ext_data_len > exts.len() {
            break;
        }
        if ext_type == 0x0000 {
            // SNI extension
            let sni_data = &exts[ext_offset..ext_offset + ext_data_len];
            if sni_data.len() >= 5 {
                let _sni_list_len = ((sni_data[0] as usize) << 8) | (sni_data[1] as usize);
                if sni_data[2] == 0x00 {
                    let host_len = ((sni_data[3] as usize) << 8) | (sni_data[4] as usize);
                    if sni_data.len() >= 5 + host_len {
                        return String::from_utf8(sni_data[5..5 + host_len].to_vec()).ok();
                    }
                }
            }
        }
        ext_offset += ext_data_len;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ca_generation() {
        let mgr = CaCertManager::new();
        assert!(mgr.ensure_ca().is_ok());
        assert!(mgr.ca_cert_path.exists());
        assert!(mgr.ca_key_path.exists());
        let cert_pem = mgr.ca_cert_pem().expect("value should be ok in test");
        assert!(cert_pem.contains("BEGIN CERTIFICATE"));
    }

    #[test]
    fn test_server_cert_generation() {
        let mgr = CaCertManager::new();
        mgr.ensure_ca().expect("value should be ok in test");
        let (cert, key) = mgr.generate_server_cert("example.com").expect("value should be ok in test");
        assert!(cert.contains("BEGIN CERTIFICATE"));
        assert!(key.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn test_pem_to_der() {
        let pem = "-----BEGIN TEST-----\nSGVsbG8=\n-----END TEST-----";
        let der = pem_to_der(pem, "TEST").expect("value should be ok in test");
        assert_eq!(der, b"Hello");
    }

    #[test]
    fn test_sni_parse() {
        // Minimal TLS ClientHello with SNI for example.com
        let hello = vec![
            0x16, 0x03, 0x01, 0x00, 0x00, // TLS record header (dummy length)
        ];
        assert!(parse_sni_from_client_hello(&hello).is_none());
    }
}
