use aes::Aes128;
use cipher::{BlockEncryptMut, KeyIvInit};
use md5::Digest;

type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128EcbEnc = ecb::Encryptor<Aes128>;

/// Kugou Eapi 加密
pub fn kugou_eapi_encrypt(url: &str, params: &str) -> String {
    let secret = format!("{:x}", md5::compute(params.as_bytes()));
    let key = secret.as_bytes();
    let plaintext = params.as_bytes();
    let padded = pkcs7_pad(plaintext, 16);
    let mut buf = padded.clone();
    let encryptor = Aes128EcbEnc::new(key.into());
    encryptor.encrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut buf, padded.len());
    let hex_str = hex::encode(&buf);
    let verify = format!("{:x}", md5::compute(hex_str.as_bytes()));
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
    let digest = format!("{:x}", md5::compute(message.as_bytes()));
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
    encryptor.encrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut buf, padded.len());
    buf
}

fn aes128_ecb_encrypt(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let padded = pkcs7_pad(plaintext, 16);
    let mut buf = padded.clone();
    let encryptor = Aes128EcbEnc::new(key.into());
    encryptor.encrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut buf, padded.len());
    buf
}

fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
    let padding_len = block_size - (data.len() % block_size);
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat(padding_len as u8).take(padding_len));
    padded
}
