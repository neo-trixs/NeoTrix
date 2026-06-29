use super::chain::ChainType;
use super::wallet::CryptoWallet;
use k256::ecdsa::signature::hazmat::PrehashSigner;
use k256::ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey};
use sha3::{Digest, Keccak256};

#[derive(Clone, Debug)]
pub enum TxType {
    Legacy,
    Eip1559,
}

#[derive(Clone, Debug)]
pub struct Tx1559 {
    pub chain_id: u64,
    pub nonce: u64,
    pub max_priority_fee: u128,
    pub max_fee: u128,
    pub gas_limit: u64,
    pub to: String,
    pub value: u128,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct TxLegacy {
    pub chain_id: u64,
    pub nonce: u64,
    pub gas_price: u128,
    pub gas_limit: u64,
    pub to: String,
    pub value: u128,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct SignedTx {
    pub raw: Vec<u8>,
    pub tx_hash: String,
    pub r: Vec<u8>,
    pub s: Vec<u8>,
    pub v: u8,
}

#[derive(Clone, Debug)]
pub struct TxReceipt {
    pub tx_hash: String,
    pub block_number: Option<u64>,
    pub status: Option<bool>,
    pub gas_used: Option<u64>,
    pub effective_gas_price: Option<u128>,
}

pub struct TxBuilder;

impl TxBuilder {
    #[allow(unused_variables)]
    pub fn build_1559(
        wallet: &CryptoWallet,
        chain: &ChainType,
        to: &str,
        value: u128,
        data: Vec<u8>,
        max_priority_fee: u128,
        max_fee: u128,
        gas_limit: u64,
        nonce: u64,
    ) -> Tx1559 {
        Tx1559 {
            chain_id: chain.chain_id(),
            nonce,
            max_priority_fee,
            max_fee,
            gas_limit,
            to: to.to_string(),
            value,
            data,
        }
    }

    pub fn sign_1559(wallet: &CryptoWallet, tx: &Tx1559) -> Result<SignedTx, String> {
        let signing_key = SigningKey::from_slice(wallet.private_key_bytes())
            .map_err(|e| format!("invalid key: {}", e))?;

        let items = encode_1559_items(tx);
        let payload = rlp_encode_list(&items);

        let mut msg = vec![0x02];
        msg.extend(&payload);

        let hash = Keccak256::digest(&msg);
        let hash_bytes: [u8; 32] = hash.into();

        let sig: Signature = signing_key
            .sign_prehash(&hash_bytes)
            .map_err(|e| format!("signing failed: {}", e))?;

        let (recid, r, s) = recovery_data(&signing_key, &hash_bytes, &sig)?;

        let signed_items = encode_1559_signed_items(tx, recid, &r, &s);
        let signed_payload = rlp_encode_list(&signed_items);

        let mut raw = vec![0x02];
        raw.extend(&signed_payload);

        let raw_hash = Keccak256::digest(&raw);

        Ok(SignedTx {
            raw,
            tx_hash: format!("0x{}", hex::encode(&raw_hash)),
            r,
            s,
            v: recid,
        })
    }

    pub fn sign_legacy(wallet: &CryptoWallet, tx: &TxLegacy) -> Result<SignedTx, String> {
        let signing_key = SigningKey::from_slice(wallet.private_key_bytes())
            .map_err(|e| format!("invalid key: {}", e))?;

        let items = encode_legacy_items(tx);
        let payload = rlp_encode_list(&items);

        let hash = Keccak256::digest(&payload);
        let hash_bytes: [u8; 32] = hash.into();

        let sig: Signature = signing_key
            .sign_prehash(&hash_bytes)
            .map_err(|e| format!("signing failed: {}", e))?;

        let (recid, r, s) = recovery_data(&signing_key, &hash_bytes, &sig)?;

        let v = tx.chain_id * 2 + 35 + recid as u64;

        let signed_items = encode_legacy_signed_items(tx, v, &r, &s);
        let raw = rlp_encode_list(&signed_items);

        let raw_hash = Keccak256::digest(&raw);

        Ok(SignedTx {
            raw,
            tx_hash: format!("0x{}", hex::encode(&raw_hash)),
            r,
            s,
            v: recid,
        })
    }

    pub fn encode_erc20_transfer(
        token_contract: &str,
        to: &str,
        amount: u128,
    ) -> (String, Vec<u8>) {
        let to_addr = addr_to_bytes(to);
        let amount_bytes = u256_padded(&amount.to_be_bytes());

        let mut data = Vec::with_capacity(4 + 32 + 32);
        data.extend_from_slice(
            &hex::decode("a9059cbb").expect("compile-time hex literal for transfer selector"),
        );
        data.extend_from_slice(&to_addr);
        data.extend_from_slice(&amount_bytes);

        (token_contract.to_string(), data)
    }

    pub fn encode_erc20_approve(spender: &str, amount: u128) -> Vec<u8> {
        let spender_bytes = addr_to_bytes(spender);
        let amount_bytes = u256_padded(&amount.to_be_bytes());

        let mut data = Vec::with_capacity(4 + 32 + 32);
        data.extend_from_slice(
            &hex::decode("095ea7b3").expect("compile-time hex literal for approve selector"),
        );
        data.extend_from_slice(&spender_bytes);
        data.extend_from_slice(&amount_bytes);

        data
    }
}

fn recovery_data(
    signing_key: &SigningKey,
    hash: &[u8; 32],
    sig: &Signature,
) -> Result<(u8, Vec<u8>, Vec<u8>), String> {
    let vk = signing_key.verifying_key();
    let r_bytes = sig.r().to_bytes().to_vec();
    let s_bytes = sig.s().to_bytes().to_vec();

    let recid = [0u8, 1u8]
        .into_iter()
        .find(|&rid| {
            let rec_id = RecoveryId::new(rid != 0, false);
            VerifyingKey::recover_from_prehash(hash.as_slice(), sig, rec_id)
                .map_or(false, |recovered| recovered == *vk)
        })
        .unwrap_or(0);

    Ok((recid, r_bytes, s_bytes))
}

fn rlp_encode_bytes(data: &[u8]) -> Vec<u8> {
    if data.len() == 1 && data[0] < 0x80 {
        return vec![data[0]];
    }
    if data.len() <= 55 {
        let mut result = vec![0x80 + data.len() as u8];
        result.extend(data);
        return result;
    }
    let len = data.len();
    let len_bytes = len.to_be_bytes();
    let start = len_bytes.iter().position(|&b| b != 0).unwrap_or(7);
    let len_data = len_bytes[start..].to_vec();
    let mut result = vec![0xb7 + len_data.len() as u8];
    result.extend(len_data);
    result.extend(data);
    result
}

fn rlp_encode_integer(val: u64) -> Vec<u8> {
    if val == 0 {
        return vec![0x80];
    }
    if val < 0x80 {
        return vec![val as u8];
    }
    let bytes = val.to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
    rlp_encode_bytes(&bytes[start..])
}

fn rlp_encode_u128(val: u128) -> Vec<u8> {
    if val == 0 {
        return vec![0x80];
    }
    let bytes = val.to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(15);
    if bytes.len() - start == 1 && bytes[start] < 0x80 {
        return vec![bytes[start]];
    }
    rlp_encode_bytes(&bytes[start..])
}

fn rlp_encode_address(addr: &str) -> Vec<u8> {
    let s = addr.strip_prefix("0x").unwrap_or(addr);
    if s.is_empty() || s == "0" {
        return vec![0x80];
    }
    let bytes = hex::decode(s).unwrap_or_default();
    if bytes.is_empty() {
        vec![0x80]
    } else if bytes.len() == 1 && bytes[0] < 0x80 {
        vec![bytes[0]]
    } else {
        rlp_encode_bytes(&bytes)
    }
}

fn rlp_encode_list(items: &[Vec<u8>]) -> Vec<u8> {
    let mut flat: Vec<u8> = Vec::new();
    for item in items {
        flat.extend(item);
    }
    if flat.len() <= 55 {
        let mut result = vec![0xc0 + flat.len() as u8];
        result.extend(flat);
        return result;
    }
    let len = flat.len();
    let len_bytes = len.to_be_bytes();
    let start = len_bytes.iter().position(|&b| b != 0).unwrap_or(7);
    let len_data = len_bytes[start..].to_vec();
    let mut result = vec![0xf7 + len_data.len() as u8];
    result.extend(len_data);
    result.extend(flat);
    result
}

fn encode_1559_items(tx: &Tx1559) -> Vec<Vec<u8>> {
    vec![
        rlp_encode_integer(tx.chain_id),
        rlp_encode_integer(tx.nonce),
        rlp_encode_u128(tx.max_priority_fee),
        rlp_encode_u128(tx.max_fee),
        rlp_encode_integer(tx.gas_limit),
        rlp_encode_address(&tx.to),
        rlp_encode_u128(tx.value),
        rlp_encode_bytes(&tx.data),
        rlp_encode_list(&[]),
    ]
}

fn encode_1559_signed_items(tx: &Tx1559, recid: u8, r: &[u8], s: &[u8]) -> Vec<Vec<u8>> {
    vec![
        rlp_encode_integer(tx.chain_id),
        rlp_encode_integer(tx.nonce),
        rlp_encode_u128(tx.max_priority_fee),
        rlp_encode_u128(tx.max_fee),
        rlp_encode_integer(tx.gas_limit),
        rlp_encode_address(&tx.to),
        rlp_encode_u128(tx.value),
        rlp_encode_bytes(&tx.data),
        rlp_encode_list(&[]),
        rlp_encode_integer(recid as u64),
        rlp_encode_bytes(r),
        rlp_encode_bytes(s),
    ]
}

fn encode_legacy_items(tx: &TxLegacy) -> Vec<Vec<u8>> {
    vec![
        rlp_encode_integer(tx.nonce),
        rlp_encode_u128(tx.gas_price),
        rlp_encode_integer(tx.gas_limit),
        rlp_encode_address(&tx.to),
        rlp_encode_u128(tx.value),
        rlp_encode_bytes(&tx.data),
        rlp_encode_integer(tx.chain_id),
        rlp_encode_integer(0),
        rlp_encode_integer(0),
    ]
}

fn encode_legacy_signed_items(tx: &TxLegacy, v: u64, r: &[u8], s: &[u8]) -> Vec<Vec<u8>> {
    vec![
        rlp_encode_integer(tx.nonce),
        rlp_encode_u128(tx.gas_price),
        rlp_encode_integer(tx.gas_limit),
        rlp_encode_address(&tx.to),
        rlp_encode_u128(tx.value),
        rlp_encode_bytes(&tx.data),
        rlp_encode_integer(v),
        rlp_encode_bytes(r),
        rlp_encode_bytes(s),
    ]
}

fn addr_to_bytes(addr: &str) -> [u8; 32] {
    let s = addr.strip_prefix("0x").unwrap_or(addr);
    let bytes = hex::decode(s).unwrap_or_default();
    let mut result = [0u8; 32];
    let start = 32_usize.saturating_sub(bytes.len());
    result[start..].copy_from_slice(&bytes[..bytes.len().min(32)]);
    result
}

fn u256_padded(bytes: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let start = 32_usize.saturating_sub(bytes.len());
    result[start..].copy_from_slice(&bytes[..bytes.len().min(32)]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_act_crypto::chain::ChainType;
    use crate::neotrix::nt_act_crypto::wallet::CryptoWallet;

    fn test_wallet() -> CryptoWallet {
        CryptoWallet::generate_evm("test_tx")
    }

    #[test]
    fn test_rlp_encode_single_byte() {
        let encoded = rlp_encode_bytes(&[0x05]);
        assert_eq!(encoded, vec![0x05]);
    }

    #[test]
    fn test_rlp_encode_string() {
        let encoded = rlp_encode_bytes(b"dog");
        assert_eq!(encoded, vec![0x83, b'd', b'o', b'g']);
    }

    #[test]
    fn test_rlp_encode_empty() {
        let encoded = rlp_encode_bytes(&[]);
        assert_eq!(encoded, vec![0x80]);
    }

    #[test]
    fn test_rlp_encode_list() {
        let items = vec![rlp_encode_bytes(b"cat"), rlp_encode_bytes(b"dog")];
        let encoded = rlp_encode_list(&items);
        assert_eq!(
            encoded,
            vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']
        );
    }

    #[test]
    fn test_rlp_encode_integer_zero() {
        let encoded = rlp_encode_integer(0);
        assert_eq!(encoded, vec![0x80]);
    }

    #[test]
    fn test_rlp_encode_integer_small() {
        let encoded = rlp_encode_integer(0x05);
        assert_eq!(encoded, vec![0x05]);
    }

    #[test]
    fn test_rlp_encode_integer_large() {
        let encoded = rlp_encode_integer(0x0100);
        assert_eq!(encoded, vec![0x82, 0x01, 0x00]);
    }

    #[test]
    fn test_rlp_encode_address() {
        let addr = "0x1234567890abcdef1234567890abcdef12345678";
        let encoded = rlp_encode_address(addr);
        assert_eq!(encoded.len(), 21); // 0x94 + 20 bytes
        assert_eq!(encoded[0], 0x94);
    }

    #[test]
    fn test_build_1559() {
        let wallet = test_wallet();
        let chain = ChainType::Ethereum;
        let tx = TxBuilder::build_1559(
            &wallet,
            &chain,
            "0x1234567890abcdef1234567890abcdef12345678",
            1000000000000000000,
            vec![],
            2000000000,
            50000000000,
            21000,
            0,
        );
        assert_eq!(tx.chain_id, 1);
        assert_eq!(tx.nonce, 0);
        assert_eq!(tx.value, 1000000000000000000);
        assert_eq!(tx.gas_limit, 21000);
    }

    #[test]
    fn test_sign_1559() {
        let wallet = test_wallet();
        let chain = ChainType::Ethereum;
        let tx = TxBuilder::build_1559(
            &wallet,
            &chain,
            "0x1234567890abcdef1234567890abcdef12345678",
            0,
            vec![],
            1000000000,
            20000000000,
            21000,
            0,
        );
        let signed = TxBuilder::sign_1559(&wallet, &tx).unwrap();
        assert!(signed.raw.len() > 100);
        assert!(signed.tx_hash.starts_with("0x"));
        assert_eq!(signed.tx_hash.len(), 66);
        assert_eq!(signed.r.len(), 32);
        assert_eq!(signed.s.len(), 32);
    }

    #[test]
    fn test_sign_legacy() {
        let wallet = test_wallet();
        let chain = ChainType::Ethereum;
        let tx = TxLegacy {
            chain_id: chain.chain_id(),
            nonce: 0,
            gas_price: 10000000000,
            gas_limit: 21000,
            to: "0x1234567890abcdef1234567890abcdef12345678".into(),
            value: 0,
            data: vec![],
        };
        let signed = TxBuilder::sign_legacy(&wallet, &tx).unwrap();
        assert!(signed.raw.len() > 100);
        assert!(signed.tx_hash.starts_with("0x"));
    }

    #[test]
    fn test_erc20_transfer_encoding() {
        let (contract, data) = TxBuilder::encode_erc20_transfer(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0x1234567890abcdef1234567890abcdef12345678",
            1000000,
        );
        assert!(contract.contains("A0b86991"));
        assert_eq!(data.len(), 68);
        assert_eq!(&data[..4], &hex::decode("a9059cbb").unwrap());
    }

    #[test]
    fn test_erc20_approve_encoding() {
        let data = TxBuilder::encode_erc20_approve(
            "0x1234567890abcdef1234567890abcdef12345678",
            u128::MAX,
        );
        assert_eq!(data.len(), 68);
        assert_eq!(&data[..4], &hex::decode("095ea7b3").unwrap());
    }

    #[test]
    fn test_sign_1559_bsc() {
        let wallet = test_wallet();
        let chain = ChainType::Bsc;
        let tx = Tx1559 {
            chain_id: chain.chain_id(),
            nonce: 0,
            max_priority_fee: 1000000000,
            max_fee: 5000000000,
            gas_limit: 21000,
            to: "0x1234567890abcdef1234567890abcdef12345678".into(),
            value: 1000000000000000000,
            data: vec![],
        };
        let signed = TxBuilder::sign_1559(&wallet, &tx).unwrap();
        assert_eq!(signed.r.len(), 32);
        assert_eq!(signed.s.len(), 32);
        assert!(signed.tx_hash.starts_with("0x"));
    }

    #[test]
    fn test_rlp_encode_u128() {
        let encoded = rlp_encode_u128(0);
        assert_eq!(encoded, vec![0x80]);

        let encoded = rlp_encode_u128(1);
        assert_eq!(encoded, vec![0x01]);

        let encoded = rlp_encode_u128(0x80);
        assert_eq!(encoded, vec![0x81, 0x80]);

        let val: u128 = 1_000_000_000_000_000_000;
        let encoded = rlp_encode_u128(val);
        assert!(encoded.len() > 1);
    }
}
