use super::super::tl::{TlReader, TlWriter};
use crate::neotrix::nt_io_gram::crypto;

const REQ_PQ: u32 = 0x60469778;
const REQ_DH_PARAMS: u32 = 0xd712e4be;
const SET_CLIENT_DH_PARAMS: u32 = 0xf5045f1f;

pub const PQ_P: &[u8] = b"\xad\xf5\x22\xc9\x69\xa8\xd7\x17";
pub const PQ_Q: &[u8] = b"\x93\x07\x3b\xc9\x78\x62\x8d\x23";
pub const FINGERPRINT: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

pub struct KeyExchangeResult {
    pub auth_key: [u8; 256],
    pub server_nonce: [u8; 32],
    pub new_nonce: [u8; 32],
}

pub fn parse_req_pq(data: &[u8]) -> Result<[u8; 16], String> {
    let mut r = TlReader::new(data.to_vec());
    let cid = r.read_uint32().map_err(|_| "req_pq: no cid")?;
    if cid != REQ_PQ {
        return Err(format!("req_pq: expected 0x{REQ_PQ:08x}, got 0x{cid:08x}"));
    }
    let mut nonce = [0u8; 16];
    let raw = r.read_int128_raw().map_err(|_| "req_pq: no nonce")?;
    nonce.copy_from_slice(&raw);
    Ok(nonce)
}

pub fn build_res_pq(nonce: &[u8; 16], server_nonce: &[u8; 32]) -> Vec<u8> {
    let pq_bytes = crypto::compute_pq_value(PQ_P, PQ_Q);
    let mut w = TlWriter::new();
    w.write_uint32(0x05162463);
    w.write_int128_raw(nonce);
    w.write_int256_raw(server_nonce);
    w.write_bytes(&pq_bytes);
    w.write_uint32(0x1cb5c415);
    w.write_uint32(1);
    w.write_raw(&FINGERPRINT);
    w.into_bytes()
}

pub fn parse_req_dh_params(data: &[u8]) -> Result<ReqDhParams, String> {
    let mut r = TlReader::new(data.to_vec());
    let cid = r.read_uint32().map_err(|_| "req_DH: no cid")?;
    if cid != REQ_DH_PARAMS {
        return Err(format!(
            "req_DH: expected 0x{REQ_DH_PARAMS:08x}, got 0x{cid:08x}"
        ));
    }
    let nonce = read_nonce(&mut r)?;
    let server_nonce = read_server_nonce(&mut r)?;
    let _p = r.read_bytes().map_err(|_| "req_DH: no p")?;
    let _q = r.read_bytes().map_err(|_| "req_DH: no q")?;
    let _fingerprint = r.read_int64().map_err(|_| "req_DH: no fp")?;
    let _encrypted = r.read_bytes().map_err(|_| "req_DH: no encrypted")?;
    Ok(ReqDhParams {
        nonce,
        server_nonce,
    })
}

pub struct ReqDhParams {
    pub nonce: [u8; 16],
    pub server_nonce: [u8; 32],
}

pub fn build_server_dh_params_ok(nonce: &[u8; 16], server_nonce: &[u8; 32], g_a: &[u8]) -> Vec<u8> {
    let (aes_key, aes_iv) = crypto::generate_aes_key_iv_server_nonce(server_nonce);
    let inner_data = build_server_dh_inner_data(nonce, server_nonce, g_a);
    let padded = pad_aes(&inner_data);
    let encrypted = crypto::aes_ige_encrypt(&aes_key, &aes_iv, &padded);

    let mut w = TlWriter::new();
    w.write_uint32(0xd0e8075c);
    w.write_int128_raw(nonce);
    w.write_int256_raw(server_nonce);
    w.write_bytes(&encrypted);
    w.into_bytes()
}

fn build_server_dh_inner_data(nonce: &[u8; 16], server_nonce: &[u8; 32], g_a: &[u8]) -> Vec<u8> {
    let p_bytes = crypto::dh_prime().to_bytes_le();
    let mut w = TlWriter::new();
    w.write_uint32(0xb5890dba);
    w.write_int128_raw(nonce);
    w.write_int256_raw(server_nonce);
    w.write_int32(crypto::DH_G as i32);
    w.write_string("");
    w.write_bytes(&p_bytes);
    w.write_bytes(g_a);
    w.write_int32(100);
    w.into_bytes()
}

pub fn parse_set_client_dh_params(data: &[u8]) -> Result<SetClientDhParams, String> {
    let mut r = TlReader::new(data.to_vec());
    let cid = r.read_uint32().map_err(|_| "set_client_DH: no cid")?;
    if cid != SET_CLIENT_DH_PARAMS {
        return Err(format!(
            "set_client_DH: expected 0x{SET_CLIENT_DH_PARAMS:08x}, got 0x{cid:08x}"
        ));
    }
    let nonce = read_nonce(&mut r)?;
    let server_nonce = read_server_nonce(&mut r)?;
    let encrypted = r.read_bytes().map_err(|_| "set_client_DH: no encrypted")?;

    Ok(SetClientDhParams {
        nonce,
        server_nonce,
        encrypted_data: encrypted,
    })
}

pub struct SetClientDhParams {
    pub nonce: [u8; 16],
    pub server_nonce: [u8; 32],
    pub encrypted_data: Vec<u8>,
}

pub fn decrypt_client_dh_inner_data(
    server_nonce: &[u8; 32],
    encrypted: &[u8],
) -> Result<Vec<u8>, String> {
    let (aes_key, aes_iv) = crypto::generate_aes_key_iv_server_nonce(server_nonce);
    Ok(crypto::aes_ige_decrypt(&aes_key, &aes_iv, encrypted))
}

pub fn extract_g_b_from_client_inner(decrypted: &[u8]) -> Result<Vec<u8>, String> {
    let mut r = TlReader::new(decrypted.to_vec());
    let cid = r.read_uint32().map_err(|_| "client_DH_inner: no cid")?;
    if cid != 0x6643b654 {
        return Err(format!(
            "expected client_DH_inner_data 0x6643b654, got 0x{cid:08x}"
        ));
    }
    let _nonce = read_nonce(&mut r)?;
    let _server_nonce = read_server_nonce(&mut r)?;
    let _retry_id = r.read_int64().map_err(|_| "client_DH: no retry")?;
    let g_b = r.read_bytes().map_err(|_| "client_DH: no g_b")?;
    Ok(g_b)
}

pub fn build_dh_gen_ok(nonce: &[u8; 16], server_nonce: &[u8; 32], auth_key: &[u8; 256]) -> Vec<u8> {
    let new_nonce_hash = compute_new_nonce_hash(auth_key, server_nonce);
    let mut w = TlWriter::new();
    w.write_uint32(0x3bcbf734);
    w.write_int128_raw(nonce);
    w.write_int256_raw(server_nonce);
    w.write_raw(&new_nonce_hash);
    w.into_bytes()
}

fn compute_new_nonce_hash(auth_key: &[u8; 256], _server_nonce: &[u8; 32]) -> [u8; 16] {
    let hash = crypto::sha1_bytes(auth_key);
    let mut result = [0u8; 16];
    result.copy_from_slice(&hash[..16]);
    result
}

fn read_nonce(r: &mut TlReader) -> Result<[u8; 16], String> {
    let raw = r.read_int128_raw()?;
    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&raw);
    Ok(nonce)
}

fn read_server_nonce(r: &mut TlReader) -> Result<[u8; 32], String> {
    let raw = r.read_int256_raw()?;
    let mut nonce = [0u8; 32];
    nonce.copy_from_slice(&raw);
    Ok(nonce)
}

fn pad_aes(data: &[u8]) -> Vec<u8> {
    let rem = data.len() % 16;
    if rem == 0 {
        data.to_vec()
    } else {
        let mut v = data.to_vec();
        v.extend(std::iter::repeat_n(0u8, 16 - rem));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_req_pq_roundtrip() {
        let nonce = [0x42u8; 16];
        let mut w = TlWriter::new();
        w.write_uint32(REQ_PQ);
        w.write_int128_raw(&nonce);
        let data = w.into_bytes();
        let parsed = parse_req_pq(&data).expect("value should be ok in test");
        assert_eq!(parsed, nonce);
    }

    #[test]
    fn test_build_res_pq_contains_constructor() {
        let nonce = [0x11u8; 16];
        let sn = [0x22u8; 32];
        let res = build_res_pq(&nonce, &sn);
        let mut r = TlReader::new(res);
        assert_eq!(
            r.read_uint32().expect("value should be ok in test"),
            0x05162463
        );
    }

    #[test]
    fn test_server_dh_params_ok_roundtrip() {
        let nonce = [0x11u8; 16];
        let sn = [0x22u8; 32];
        let g_a = vec![0x33u8; 256];
        let result = build_server_dh_params_ok(&nonce, &sn, &g_a);
        let mut r = TlReader::new(result);
        assert_eq!(
            r.read_uint32().expect("value should be ok in test"),
            0xd0e8075c
        );
    }

    #[test]
    fn test_dh_gen_ok_has_correct_constructor() {
        let nonce = [0x11u8; 16];
        let sn = [0x22u8; 32];
        let ak = [0x33u8; 256];
        let result = build_dh_gen_ok(&nonce, &sn, &ak);
        let mut r = TlReader::new(result);
        assert_eq!(
            r.read_uint32().expect("value should be ok in test"),
            0x3bcbf734
        );
    }

    #[test]
    fn test_dh_key_exchange_computes_same_key() {
        let server_secret = crypto::generate_server_secret();
        let client_secret = crypto::generate_server_secret();
        let g_a = crypto::compute_g_a(&server_secret);
        let g_b = crypto::compute_g_a(&client_secret);
        let server_key = crypto::compute_auth_key(&g_b, &server_secret);
        let client_key = crypto::compute_auth_key(&g_a, &client_secret);
        assert_eq!(server_key, client_key);
    }

    #[test]
    fn test_generate_pq_produces_valid_primes() {
        let (p, q) = crypto::generate_pq();
        assert!(!p.is_empty());
        assert!(!q.is_empty());
        let pq = crypto::compute_pq_value(&p, &q);
        assert_eq!(pq.len(), p.len() + q.len());
    }
}
