use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes256;
use num_bigint::BigUint;
use sha1::Sha1;
use sha2::{Digest, Sha256};

const SHA1_DIGEST_LEN: usize = 20;
const SHA256_DIGEST_LEN: usize = 32;
const AES_BLOCK: usize = 16;
pub const AUTH_KEY_LEN: usize = 256;
pub const DH_G: u32 = 3;
pub const DH_PRIME: &str = "c71caeb9c6b1c9048e6c522f70f13f73980d40238e3e21c14934d037563d930f48198a0aa7c14058229493d22530f4dbfa336f6e0ac925139543aed44cce7c3720fd51f69458705ac68cd4fe6b6b13abdc9746512969328454f18faf8c595f642477fe96bb2a941d5bcd1d4ac8cc49880708fa9b378e3c4f3a9060bee67cf9a4a4a695811051907e162753b56b0f6b410dba74d8a84b2a14b3144e0ef1284754fd17ed950d5965b4b9dd46582db1178d169c6bc465b0d6ff9ca3928fef5b9ae4e418fc15e83ebea0f87fa9ff5eed70050ded2849f47bf959d956850ce929851f0d8115f635b105ee2e4e15d04b2454bf6f4fadf034b10403119cd8e3b92fcc5b";

pub fn sha1_bytes(data: &[u8]) -> [u8; SHA1_DIGEST_LEN] {
    Sha1::digest(data).into()
}

pub fn sha256_bytes(data: &[u8]) -> [u8; SHA256_DIGEST_LEN] {
    Sha256::digest(data).into()
}

pub fn compute_message_key(auth_key: &[u8; AUTH_KEY_LEN], plaintext: &[u8], is_server: bool) -> [u8; 16] {
    let x = if is_server { 8u8 } else { 0u8 };
    let mut msg_key_large = Vec::with_capacity(32 + plaintext.len());
    msg_key_large.extend_from_slice(&auth_key[88 + x as usize..104 + x as usize]);
    msg_key_large.extend_from_slice(plaintext);
    let hash = sha256_bytes(&msg_key_large);
    let mut msg_key = [0u8; 16];
    msg_key.copy_from_slice(&hash[8..24]);
    msg_key
}

pub fn aes_ige_encrypt(key: &[u8; 32], iv: &[u8; 32], data: &[u8]) -> Vec<u8> {
    let cipher = Aes256::new_from_slice(key).expect("valid AES key");
    let blocks = data.chunks_exact(AES_BLOCK);
    let mut output = Vec::with_capacity(data.len());
    let mut prev_c = [0u8; AES_BLOCK];
    let mut prev_p = [0u8; AES_BLOCK];
    prev_c.copy_from_slice(&iv[..AES_BLOCK]);
    prev_p.copy_from_slice(&iv[AES_BLOCK..]);

    for chunk in blocks {
        let plain_block = <&[u8; AES_BLOCK]>::try_from(chunk).expect("result");
        let mut block = *plain_block;
        xor_block(&mut block, &prev_c);
        cipher.encrypt_block((&mut block).into());
        xor_block(&mut block, &prev_p);
        prev_c = block;
        prev_p = *plain_block;
        output.extend_from_slice(&block);
    }
    output
}

pub fn aes_ige_decrypt(key: &[u8; 32], iv: &[u8; 32], data: &[u8]) -> Vec<u8> {
    let cipher = Aes256::new_from_slice(key).expect("valid AES key");
    let blocks = data.chunks_exact(AES_BLOCK);
    let mut output = Vec::with_capacity(data.len());
    let mut prev_c = [0u8; AES_BLOCK];
    let mut prev_p = [0u8; AES_BLOCK];
    prev_c.copy_from_slice(&iv[..AES_BLOCK]);
    prev_p.copy_from_slice(&iv[AES_BLOCK..]);

    for chunk in blocks {
        let original_c = *<&[u8; AES_BLOCK]>::try_from(chunk).expect("result");
        let mut block = original_c;
        xor_block(&mut block, &prev_p);
        cipher.decrypt_block((&mut block).into());
        xor_block(&mut block, &prev_c);
        prev_c = original_c;
        prev_p = block;
        output.extend_from_slice(&block);
    }
    output
}

pub fn generate_aes_key_iv(auth_key: &[u8; AUTH_KEY_LEN], msg_key: &[u8; 16], is_server: bool) -> ([u8; 32], [u8; 32]) {
    let x = if is_server { 8 } else { 0 };
    let mut sha1_a = Vec::with_capacity(16 + 32);
    sha1_a.extend_from_slice(msg_key);
    sha1_a.extend_from_slice(&auth_key[x as usize..x as usize + 32]);
    let a = sha1_bytes(&sha1_a);

    let mut sha1_b = Vec::with_capacity(16 + 16 + 32);
    sha1_b.extend_from_slice(&auth_key[32 + x as usize..48 + x as usize]);
    sha1_b.extend_from_slice(msg_key);
    sha1_b.extend_from_slice(&auth_key[48 + x as usize..64 + x as usize]);
    let b = sha1_bytes(&sha1_b);

    let mut sha1_c = Vec::with_capacity(16 + 64);
    sha1_c.extend_from_slice(&auth_key[64 + x as usize..96 + x as usize]);
    sha1_c.extend_from_slice(msg_key);
    sha1_c.extend_from_slice(&auth_key[96 + x as usize..128 + x as usize]);
    let c = sha1_bytes(&sha1_c);

    let mut sha1_d = Vec::with_capacity(16 + 32);
    sha1_d.extend_from_slice(msg_key);
    sha1_d.extend_from_slice(&auth_key[128 + x as usize..160 + x as usize]);
    let d = sha1_bytes(&sha1_d);

    let mut aes_key = [0u8; 32];
    aes_key[..8].copy_from_slice(&a[..8]);
    aes_key[8..20].copy_from_slice(&b[8..20]);
    aes_key[20..32].copy_from_slice(&c[4..16]);

    let mut aes_iv = [0u8; 32];
    aes_iv[..12].copy_from_slice(&a[8..20]);
    aes_iv[12..20].copy_from_slice(&b[..8]);
    aes_iv[20..24].copy_from_slice(&c[16..20]);
    aes_iv[24..32].copy_from_slice(&d[..8]);

    (aes_key, aes_iv)
}

pub fn parse_biguint_hex(hex_str: &str) -> BigUint {
    BigUint::parse_bytes(hex_str.as_bytes(), 16).expect("valid hex")
}

pub fn dh_prime() -> BigUint {
    parse_biguint_hex(DH_PRIME)
}

pub fn dh_generator() -> u32 {
    DH_G
}

pub fn generate_server_secret() -> Vec<u8> {
    let mut secret = vec![0u8; 256];
    for chunk in secret.chunks_mut(8) {
        let val: u64 = rand::random();
        chunk.copy_from_slice(&val.to_le_bytes());
    }
    while secret.len() > 256 {
        secret.pop();
    }
    secret[..256].to_vec()
}

pub fn compute_g_a(secret_a: &[u8]) -> Vec<u8> {
    let p = dh_prime();
    let g = BigUint::from(DH_G as u64);
    let a = BigUint::from_bytes_le(secret_a);
    let g_a = g.modpow(&a, &p);
    let mut bytes = g_a.to_bytes_le();
    while bytes.len() < 256 {
        bytes.push(0u8);
    }
    bytes[..256].to_vec()
}

pub fn compute_auth_key(g_b: &[u8], secret_a: &[u8]) -> [u8; 256] {
    let p = dh_prime();
    let a = BigUint::from_bytes_le(secret_a);
    let gb = BigUint::from_bytes_le(g_b);
    let auth_key_bn = gb.modpow(&a, &p);
    let mut auth_key = [0u8; 256];
    let bytes = auth_key_bn.to_bytes_le();
    let copy_len = bytes.len().min(256);
    auth_key[..copy_len].copy_from_slice(&bytes[..copy_len]);
    auth_key
}

pub fn sha1_bytes_biguint(val: &BigUint) -> [u8; 20] {
    let bytes = val.to_bytes_le();
    sha1_bytes(&bytes)
}

pub fn generate_server_nonce() -> [u8; 32] {
    let mut nonce = [0u8; 32];
    for chunk in nonce.chunks_mut(8) {
        let v: u64 = rand::random();
        chunk.copy_from_slice(&v.to_le_bytes());
    }
    nonce
}

pub fn generate_nonce() -> [u8; 16] {
    let mut nonce = [0u8; 16];
    for chunk in nonce.chunks_mut(8) {
        let v: u64 = rand::random();
        chunk.copy_from_slice(&v.to_le_bytes());
    }
    nonce
}

pub fn generate_pq() -> (Vec<u8>, Vec<u8>) {
    loop {
        let p_candidate: u64 = rand::random::<u64>() | 0x8000000000000001u64;
        if !is_prime(p_candidate) {
            continue;
        }
        let q_candidate: u64 = rand::random::<u64>() | 0x8000000000000001u64;
        if !is_prime(q_candidate) || q_candidate == p_candidate {
            continue;
        }
        let p_bn = BigUint::from(p_candidate);
        let q_bn = BigUint::from(q_candidate);
        let pq_bn = &p_bn * &q_bn;
        let pq_bytes = pq_bn.to_bytes_le();
        let p_bytes = p_bn.to_bytes_le();
        let q_bytes = q_bn.to_bytes_le();
        if pq_bytes.len() > 128 {
            continue;
        }
        return (p_bytes, q_bytes);
    }
}

pub fn compute_pq_value(p_bytes: &[u8], q_bytes: &[u8]) -> Vec<u8> {
    let p = BigUint::from_bytes_le(p_bytes);
    let q = BigUint::from_bytes_le(q_bytes);
    let pq = &p * &q;
    pq.to_bytes_le()
}

pub fn compute_fingerprint(p_bytes: &[u8], q_bytes: &[u8]) -> [u8; 8] {
    let pq = compute_pq_value(p_bytes, q_bytes);
    let mut data = Vec::new();
    data.extend_from_slice(&pq);
    data.extend_from_slice(p_bytes);
    data.extend_from_slice(q_bytes);
    let hash = sha1_bytes(&data);
    let mut fp = [0u8; 8];
    fp.copy_from_slice(&hash[12..20]);
    fp
}

fn is_prime(n: u64) -> bool {
    // Deterministic Miller-Rabin for u64: bases [2, 3, 5, 7, 11, 13]
    if n < 2 { return false; }
    if n == 2 || n == 3 { return true; }
    if n.is_multiple_of(2) { return false; }
    let mut d = n - 1;
    let mut s = 0;
    while d.is_multiple_of(2) { d /= 2; s += 1; }
    'witness: for &a in &[2u64, 3, 5, 7, 11, 13] {
        if a >= n { continue; }
        let mut x = mod_pow(a, d, n);
        if x == 1 || x == n - 1 { continue; }
        for _ in 0..s - 1 {
            x = mod_mul(x, x, n);
            if x == n - 1 { continue 'witness; }
        }
        return false;
    }
    true
}

fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result = 1u64;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 { result = mod_mul(result, base, modulus); }
        base = mod_mul(base, base, modulus);
        exp >>= 1;
    }
    result
}

fn mod_mul(a: u64, b: u64, modulus: u64) -> u64 {
    ((a as u128) * (b as u128) % modulus as u128) as u64
}

pub fn generate_aes_key_iv_server_nonce(server_nonce: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let hash = sha256_bytes(server_nonce);
    let mut aes_key = [0u8; 32];
    aes_key.copy_from_slice(&hash[..32]);
    let hash2 = sha256_bytes(&server_nonce[..28]);
    let mut aes_iv = [0u8; 32];
    aes_iv[..28].copy_from_slice(&hash2[..28]);
    (aes_key, aes_iv)
}

fn xor_block(block: &mut [u8; 16], xor_with: &[u8; 16]) {
    for (b, x) in block.iter_mut().zip(xor_with.iter()) {
        *b ^= x;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_known() {
        let result = sha1_bytes(b"");
        assert_eq!(hex::encode(result), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn test_sha256_known() {
        let result = sha256_bytes(b"");
        assert_eq!(
            hex::encode(result),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_aes_ige_roundtrip() {
        let key = [0x42u8; 32];
        let iv = [0x24u8; 32];
        let plaintext = b"Hello NeoTrix MTProto!";
        let padded = pad_to_16(plaintext);
        let ciphertext = aes_ige_encrypt(&key, &iv, &padded);
        let decrypted = aes_ige_decrypt(&key, &iv, &ciphertext);
        assert_eq!(decrypted, padded);
    }

    fn pad_to_16(data: &[u8]) -> Vec<u8> {
        let rem = data.len() % 16;
        if rem == 0 { data.to_vec() }
        else {
            let mut v = data.to_vec();
            v.extend(std::iter::repeat(0u8).take(16 - rem));
            v
        }
    }

    #[test]
    fn test_aes_key_iv_deterministic() {
        let auth_key = [0x11u8; 256];
        let msg_key = [0x22u8; 16];
        let (k1, i1) = generate_aes_key_iv(&auth_key, &msg_key, false);
        let (k2, i2) = generate_aes_key_iv(&auth_key, &msg_key, false);
        assert_eq!(k1, k2);
        assert_eq!(i1, i2);
    }

    #[test]
    fn test_message_key_nonzero() {
        let auth_key = [0x11u8; 256];
        let msg_key = compute_message_key(&auth_key, b"test message", false);
        assert!(!msg_key.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_server_client_key_differs() {
        let mut auth_key = [0x11u8; 256];
        auth_key[0..8].copy_from_slice(b"DIFFDATA");
        auth_key[8..16].copy_from_slice(b"ANOTHER0");
        let msg_key = [0x22u8; 16];
        let (sk, _) = generate_aes_key_iv(&auth_key, &msg_key, true);
        let (ck, _) = generate_aes_key_iv(&auth_key, &msg_key, false);
        // With differentiated auth_key, server and client keys must differ
        // (x offset 8 vs 0 changes which 32-byte windows are hashed)
        assert_ne!(sk, ck, "server/client keys must differ with non-uniform auth_key");
    }
}
