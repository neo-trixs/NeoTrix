use super::key_exchange;
use super::rpc;
use super::session::Session;
use crate::neotrix::nt_act_gram::crypto;
use crate::neotrix::nt_act_gram::transport::{MtpTransport, TransportProtocol};

pub struct MtpServer {
    port: u16,
}

impl MtpServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn start(&self) -> Result<(), String> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("bind {}: {}", addr, e))?;
        log::info!("NeoTrix MTProto server listening on {}", addr);

        loop {
            let (stream, peer) = listener
                .accept()
                .await
                .map_err(|e| format!("accept: {}", e))?;
            log::info!("new connection from {}", peer);
            tokio::spawn(async move {
                let mut transport = MtpTransport::new(stream, TransportProtocol::Abridged);
                match transport.read_handshake().await {
                    Ok(_proto) => {
                        let mut session = Session::new();
                        if let Err(e) = handle_connection(transport, &mut session).await {
                            log::error!("connection error from {}: {}", peer, e);
                        }
                        log::info!("connection closed: {}", peer);
                    }
                    Err(e) => {
                        log::error!("transport init failed: {}", e);
                    }
                }
            });
        }
    }

    pub async fn start_with_session_store(
        &self,
        _store: std::sync::Arc<tokio::sync::Mutex<Vec<Session>>>,
    ) -> Result<(), String> {
        self.start().await
    }
}

async fn handle_connection(
    mut transport: MtpTransport,
    session: &mut Session,
) -> Result<(), String> {
    let auth_key = perform_key_exchange(&mut transport).await?;
    session.auth_key = Some(auth_key);

    loop {
        let packet = transport.read_packet().await?;

        if packet.len() < 24 {
            log::warn!("packet too short: {} bytes", packet.len());
            continue;
        }

        let (decrypted, _computed_msg_key) = {
            let auth_key = session.auth_key.as_ref().ok_or_else(|| "no auth key".to_string())?;
            let msg_key: [u8; 16] = {
                let mut k = [0u8; 16];
                k.copy_from_slice(&packet[..16]);
                k
            };
            let encrypted_data = &packet[16..];
            let (aes_key, aes_iv) =
                crypto::generate_aes_key_iv(auth_key, &msg_key, true);
            let decrypted =
                crypto::aes_ige_decrypt(&aes_key, &aes_iv, encrypted_data);
            let computed = crypto::compute_message_key(auth_key, &decrypted, true);
            if msg_key != computed {
                log::warn!("msg_key mismatch");
                continue;
            }
            (decrypted, computed)
        };

        let rpc_data = extract_rpc_data(&decrypted)?;
        let response = rpc::handle_rpc_call(session, &rpc_data);
        let plaintext = wrap_rpc_response(&response);

        let (resp_key, resp_iv, resp_msg_key) = {
            let auth_key = session.auth_key.as_ref().ok_or_else(|| "no auth key".to_string())?;
            let msg_key = crypto::compute_message_key(auth_key, &plaintext, false);
            let (k, iv) = crypto::generate_aes_key_iv(auth_key, &msg_key, false);
            (k, iv, msg_key)
        };
        let encrypted = crypto::aes_ige_encrypt(&resp_key, &resp_iv, &plaintext);

        let mut out = Vec::with_capacity(16 + encrypted.len());
        out.extend_from_slice(&resp_msg_key);
        out.extend_from_slice(&encrypted);
        transport.write_packet(&out).await?;
    }
}

async fn perform_key_exchange(
    transport: &mut MtpTransport,
) -> Result<[u8; 256], String> {
    let packet = transport.read_packet().await?;
    let nonce = key_exchange::parse_req_pq(&packet)?;

    let server_nonce = crypto::generate_server_nonce();
    let server_secret = crypto::generate_server_secret();
    let g_a = crypto::compute_g_a(&server_secret);

    let res_pq = key_exchange::build_res_pq(&nonce, &server_nonce);
    transport.write_packet(&res_pq).await?;

    let dh_packet = transport.read_packet().await?;
    let _dh_params = key_exchange::parse_req_dh_params(&dh_packet)?;

    let dh_ok = key_exchange::build_server_dh_params_ok(&nonce, &server_nonce, &g_a);
    transport.write_packet(&dh_ok).await?;

    let client_packet = transport.read_packet().await?;
    let set_params = key_exchange::parse_set_client_dh_params(&client_packet)?;

    let client_inner = key_exchange::decrypt_client_dh_inner_data(
        &server_nonce,
        &set_params.encrypted_data,
    )?;
    let g_b = key_exchange::extract_g_b_from_client_inner(&client_inner)?;
    let auth_key = crypto::compute_auth_key(&g_b, &server_secret);

    let dh_gen = key_exchange::build_dh_gen_ok(&nonce, &server_nonce, &auth_key);
    transport.write_packet(&dh_gen).await?;

    Ok(auth_key)
}

fn extract_rpc_data(decrypted: &[u8]) -> Result<Vec<u8>, String> {
    if decrypted.len() < 24 {
        return Err("decrypted data too short".to_string());
    }
    Ok(decrypted[24..].to_vec())
}

fn wrap_rpc_response(data: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(24 + data.len());
    let salt: u64 = rand::random();
    let session_id: u64 = rand::random();
    buf.extend_from_slice(&salt.to_le_bytes());
    buf.extend_from_slice(&session_id.to_le_bytes());
    buf.extend_from_slice(data);
    while buf.len() % 16 != 0 {
        buf.push(0u8);
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_auth_key() {
        let secret = crypto::generate_server_secret();
        let g_a = crypto::compute_g_a(&secret);
        assert_eq!(g_a.len(), 256);
        assert!(!g_a.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_unique_auth_keys() {
        let a = crypto::generate_server_secret();
        let b = crypto::generate_server_secret();
        assert_ne!(a, b);
    }

    #[test]
    fn test_wrap_response_aligned() {
        let data = b"\x01\x02\x03\x04";
        let wrapped = wrap_rpc_response(data);
        assert_eq!(wrapped.len() % 16, 0);
        assert_eq!(&wrapped[16..20], data);
    }

    #[test]
    fn test_extract_rpc_data_valid() {
        let d = [0u8; 32];
        let r = extract_rpc_data(&d).expect("value should be ok in test");
        assert_eq!(r.len(), 8);
    }

    #[test]
    fn test_extract_rpc_data_too_short() {
        let d = [0u8; 3];
        assert!(extract_rpc_data(&d).is_err());
    }
}
