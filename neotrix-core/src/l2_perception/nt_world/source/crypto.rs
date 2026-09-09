use aes::Aes128;
use cipher::{BlockEncryptMut, KeyIvInit};
use md5::Md5;
use md5::Digest as _;

type Aes128CbcEnc = cbc::Encryptor<Aes128>;

/// Kugou Eapi 加密
pub fn kugou_eapi_encrypt(_url: &str, params: &str) -> String {
    let secret = format!("{:x}", Md5::digest(params.as_bytes()));
    let key = secret.as_bytes();
    let plaintext = params.as_bytes();
    let padded = pkcs7_pad(plaintext, 16);
    let mut buf = padded.clone();
    // ECB mode: encrypt each block independently using cbc with zero iv
    let zero_iv = [0u8; 16];
    let encryptor = Aes128CbcEnc::new(key.into(), &zero_iv.into());
    let _ = encryptor.encrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut buf, padded.len());
    let hex_str = hex::encode(&buf);
    let verify = format!("{:x}", Md5::digest(hex_str.as_bytes()));
    format!(
        "https://trackercdn.kugou.com/i/v2/?cmd=25&pid=1&behavior=play&eapi={}&verify={}",
        urlencoding::encode(&hex_str),
        verify
    )
}

/// Netease Weapi 加密
pub fn netease_weapi_encrypt(params: &str) -> String {
    let key = b"0CoJUm6Qyw8W8jud";
    let iv = b"0102030405060708";
    let pub_key = b"0QU9qcQlCblr0nGi";
    let encrypted1 = aes128_cbc_encrypt(params.as_bytes(), key, iv);
    let base64_1 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted1);
    let encrypted2 = aes128_cbc_encrypt(base64_1.as_bytes(), pub_key, iv);
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted2)
}

/// Netease Eapi 加密
pub fn netease_eapi_encrypt(url: &str, params: &str) -> String {
    let message = format!("nobody{}use{}", url, params);
    let digest = format!("{:x}", Md5::digest(message.as_bytes()));
    let text = format!("{}{}{}{}{}{}{}", "nobody", url, "use", digest, "md5", "forever", params);
    let key = b"e82ckenh8dichen8";
    let encrypted = aes128_ecb_encrypt(text.as_bytes(), key);
    hex::encode(encrypted)
}

/// Migu HMAC-SHA1 签名
pub fn migu_sign(params: &str) -> String {
    let message = format!("appid=music&{}&platid=0&userId=", params);
    use hmac::{Hmac, Mac};
    type HmacSha1 = Hmac<sha1::Sha1>;
    let mut mac = HmacSha1::new_from_slice(b"music").expect("HMAC accepts any key length");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

fn aes128_cbc_encrypt(plaintext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let padded = pkcs7_pad(plaintext, 16);
    let mut buf = padded.clone();
    let encryptor = Aes128CbcEnc::new(key.into(), iv.into());
    let _ = encryptor.encrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut buf, padded.len());
    buf
}

fn aes128_ecb_encrypt(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let padded = pkcs7_pad(plaintext, 16);
    let mut buf = padded.clone();
    // ECB mode: encrypt each block independently using cbc with zero iv
    let zero_iv = [0u8; 16];
    let encryptor = Aes128CbcEnc::new(key.into(), &zero_iv.into());
    let _ = encryptor.encrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut buf, padded.len());
    buf
}

fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
    let padding_len = block_size - (data.len() % block_size);
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat(padding_len as u8).take(padding_len));
    padded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkcs7_pad() {
        let data = b"hello";
        let padded = pkcs7_pad(data, 16);
        assert_eq!(padded.len(), 16);
        assert_eq!(padded[5], 11); // 16 - 5 = 11
    }

    #[test]
    fn test_kugou_eapi_encrypt() {
        let url = "https://trackercdn.kugou.com/i/v2/";
        let params = r#"{"appid":1005,"platid":4,"encode_album_audio_id":"test","token":""}"#;
        let encrypted = kugou_eapi_encrypt(url, params);
        assert!(encrypted.contains("eapi="));
        assert!(encrypted.contains("verify="));
    }

    #[test]
    fn test_netease_weapi_encrypt() {
        let params = r#"{"s":"test","type":1,"limit":30,"offset":0}"#;
        let encrypted = netease_weapi_encrypt(params);
        assert!(!encrypted.is_empty());
        // 应该是 base64 编码
        assert!(base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encrypted).is_ok());
    }

    #[test]
    fn test_netease_eapi_encrypt() {
        let url = "/api/song/enhance/player/url";
        let params = r#"{"ids":"[12345]","br":320000}"#;
        let encrypted = netease_eapi_encrypt(url, params);
        assert!(!encrypted.is_empty());
        // 应该是 hex 编码
        assert!(hex::decode(&encrypted).is_ok());
    }

    #[test]
    fn test_migu_sign() {
        let params = "songId=12345";
        let sign = migu_sign(params);
        assert!(!sign.is_empty());
        // 应该是 hex 编码
        assert!(hex::decode(&sign).is_ok());
    }
}
