use std::collections::HashMap;
use std::str::FromStr;

use bdk_wallet::bitcoin::bip32::{DerivationPath, Xpriv};
use bdk_wallet::bitcoin::consensus::encode::serialize_hex;
use bdk_wallet::bitcoin::locktime::absolute;
use bdk_wallet::bitcoin::psbt::Psbt;
use bdk_wallet::bitcoin::secp256k1::Secp256k1;
use bdk_wallet::bitcoin::{Address, Amount, Network as BdkNetwork, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness};
use bdk_wallet::bip39::Mnemonic;
use bdk_wallet::{KeychainKind, Wallet as BdkWallet};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Network {
    Mainnet,
    Testnet,
    Signet,
}

impl Network {
    fn to_bdk(&self) -> BdkNetwork {
        match self {
            Network::Mainnet => BdkNetwork::Bitcoin,
            Network::Testnet => BdkNetwork::Testnet,
            Network::Signet => BdkNetwork::Signet,
        }
    }

    fn api_url(&self) -> &'static str {
        match self {
            Network::Mainnet => "https://blockstream.info/api",
            Network::Testnet => "https://blockstream.info/testnet/api",
            Network::Signet => "https://blockstream.info/signet/api",
        }
    }

    fn bip84_path(&self) -> &'static str {
        match self {
            Network::Mainnet => "m/84'/0'/0'",
            Network::Testnet | Network::Signet => "m/84'/1'/0'",
        }
    }
}

pub struct WalletInfo {
    pub balance: BalanceInfo,
    pub addresses: Vec<AddressInfo>,
    pub transaction_count: usize,
}

#[derive(Debug, Clone)]
pub struct BalanceInfo {
    pub confirmed: u64,
    pub spendable: u64,
    pub total: u64,
}

#[derive(Debug, Clone)]
pub struct AddressInfo {
    pub address: String,
    pub index: u32,
    pub is_internal: bool,
    pub used: bool,
}

#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub txid: String,
    pub amount: i64,
    pub fee: u64,
    pub confirmations: u32,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UtxoData {
    txid: String,
    vout: u32,
    value: u64,
    script_pubkey: ScriptBuf,
    #[allow(dead_code)]
    address_index: u32,
    confirmed: bool,
}

#[derive(Debug, Deserialize)]
struct AddressUtxo {
    txid: String,
    vout: u32,
    value: u64,
    status: StatusInfo,
}

#[derive(Debug, Deserialize)]
struct StatusInfo {
    confirmed: bool,
    block_height: Option<u32>,
    block_time: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct AddressTx {
    txid: String,
    fee: u64,
    status: StatusInfo,
    vin: Vec<TxVin>,
    vout: Vec<TxVout>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TxVin {
    #[allow(dead_code)]
    txid: String,
    #[allow(dead_code)]
    vout: u32,
    #[serde(default)]
    prevout: Option<TxPrevout>,
}

#[derive(Debug, Deserialize)]
struct TxPrevout {
    #[serde(default)]
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Debug, Deserialize)]
struct TxVout {
    value: u64,
    #[serde(default)]
    scriptpubkey_address: Option<String>,
}

pub struct BitcoinWallet {
    wallet: BdkWallet,
    network: Network,
    mnemonic: String,
}

impl BitcoinWallet {
    pub fn new(mnemonic: Option<&str>, network: Network) -> Result<Self, String> {
        let mnemonic_str = match mnemonic {
            Some(m) => m.to_string(),
            None => {
                let mut entropy = [0u8; 16];
                rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut entropy);
                Mnemonic::from_entropy(&entropy)
                    .map_err(|e| format!("Mnemonic generation failed: {}", e))?
                    .to_string()
            }
        };

        let mnemonic = Mnemonic::parse(&mnemonic_str).map_err(|e| format!("Invalid mnemonic: {}", e))?;
        let seed = mnemonic.to_seed("");
        let secp = Secp256k1::new();
        let bdk_net = network.to_bdk();

        let master_xpriv =
            Xpriv::new_master(bdk_net, &seed).map_err(|e| format!("Master key creation failed: {}", e))?;

        let path = DerivationPath::from_str(network.bip84_path())
            .map_err(|e| format!("Invalid derivation path: {}", e))?;
        let derived = master_xpriv
            .derive_priv(&secp, &path)
            .map_err(|e| format!("Key derivation failed: {}", e))?;

        let desc = format!("wpkh({}/0/*)", derived);
        let change_desc = format!("wpkh({}/1/*)", derived);

        let wallet = BdkWallet::create(desc, change_desc)
            .network(bdk_net)
            .create_wallet_no_persist()
            .map_err(|e| format!("Wallet creation failed: {}", e))?;

        Ok(BitcoinWallet {
            wallet,
            network,
            mnemonic: mnemonic_str,
        })
    }

    pub fn mnemonic_phrase(&self) -> &str {
        &self.mnemonic
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn generate_new_address(&mut self) -> Result<String, String> {
        let info = self.wallet.reveal_next_address(KeychainKind::External);
        Ok(info.address.to_string())
    }

    pub fn get_balance(&self) -> Result<BalanceInfo, String> {
        let revealed = self.get_revealed_info();
        let mut confirmed: u64 = 0;
        let mut total: u64 = 0;

        for (idx, is_internal) in &revealed {
            let keychain = if *is_internal { KeychainKind::Internal } else { KeychainKind::External };
            let addr_info = self.wallet.peek_address(keychain, *idx);
            let addr_str = addr_info.address.to_string();
            let utxos = self.fetch_utxos_for_address(&addr_str)?;
            for utxo in &utxos {
                total += utxo.value;
                if utxo.confirmed {
                    confirmed += utxo.value;
                }
            }
        }

        Ok(BalanceInfo {
            confirmed,
            spendable: confirmed,
            total,
        })
    }

    pub fn send_to(&mut self, address: &str, amount_sats: u64, fee_rate: f64) -> Result<String, String> {
        let recipient = Address::from_str(address)
            .map_err(|e| format!("Invalid address: {}", e))?
            .require_network(self.network.to_bdk())
            .map_err(|_| format!("Address not valid for {:?}", self.network))?;

        let all_utxos = self.fetch_all_utxos()?;
        if all_utxos.is_empty() {
            return Err("No UTXOs available to spend from".to_string());
        }

        let total_input: u64 = all_utxos.iter().map(|u| u.value).sum();
        let tx_vsize: u64 = 140 + (all_utxos.len() as u64 * 68) + 43;
        let estimated_fee = (tx_vsize as f64 * fee_rate).ceil() as u64;
        let total_needed = amount_sats + estimated_fee;

        if total_input < total_needed {
            return Err(format!(
                "Insufficient funds: have {} sats, need {} sats ({} + {} fee)",
                total_input, total_needed, amount_sats, estimated_fee
            ));
        }

        let mut remaining = total_needed;
        let mut selected_utxos: Vec<UtxoData> = Vec::new();
        for utxo in &all_utxos {
            if remaining == 0 {
                break;
            }
            selected_utxos.push(utxo.clone());
            if utxo.value >= remaining {
                remaining = 0;
            } else {
                remaining -= utxo.value;
            }
        }

        let selected_total: u64 = selected_utxos.iter().map(|u| u.value).sum();
        let actual_fee = (tx_vsize as f64 * fee_rate).ceil() as u64;
        let change = selected_total - amount_sats - actual_fee;

        let mut tx = Transaction {
            version: bdk_wallet::bitcoin::transaction::Version(2),
            lock_time: absolute::LockTime::ZERO,
            input: Vec::new(),
            output: vec![TxOut {
                value: Amount::from_sat(amount_sats),
                script_pubkey: recipient.script_pubkey(),
            }],
        };

        let mut psbt_inputs: Vec<bdk_wallet::bitcoin::psbt::Input> = Vec::new();

        for utxo in &selected_utxos {
            let prev_txid = bdk_wallet::bitcoin::Txid::from_str(&utxo.txid)
                .map_err(|e| format!("Invalid txid: {}", e))?;
            tx.input.push(TxIn {
                previous_output: OutPoint::new(prev_txid, utxo.vout),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            });
            psbt_inputs.push(bdk_wallet::bitcoin::psbt::Input {
                witness_utxo: Some(TxOut {
                    value: Amount::from_sat(utxo.value),
                    script_pubkey: utxo.script_pubkey.clone(),
                }),
                ..Default::default()
            });
        }

        if change > 546 {
            let change_addr = self.wallet.reveal_next_address(KeychainKind::Internal);
            tx.output.push(TxOut {
                value: Amount::from_sat(change),
                script_pubkey: change_addr.address.script_pubkey(),
            });
        }

        let mut psbt = Psbt::from_unsigned_tx(tx).map_err(|e| format!("PSBT creation failed: {}", e))?;
        psbt.inputs = psbt_inputs;

        let sign_opts = bdk_wallet::SignOptions {
            trust_witness_utxo: true,
            ..Default::default()
        };

        self.wallet
            .sign(&mut psbt, sign_opts)
            .map_err(|e| format!("Signing failed: {}", e))?;

        self.wallet
            .finalize_psbt(&mut psbt, Default::default())
            .map_err(|e| format!("Finalization failed: {}", e))?;

        let signed_tx = psbt
            .extract_tx()
            .map_err(|e| format!("Transaction extraction failed: {}", e))?;
        let txid = signed_tx.compute_txid().to_string();
        let tx_hex = serialize_hex(&signed_tx);

        self.broadcast_tx_hex(&tx_hex)?;

        Ok(txid)
    }

    pub fn get_transaction_history(&self, count: usize) -> Result<Vec<TransactionInfo>, String> {
        let revealed = self.get_revealed_info();
        let mut all_txs: HashMap<String, TransactionInfo> = HashMap::new();

        for (idx, is_internal) in &revealed {
            let keychain = if *is_internal { KeychainKind::Internal } else { KeychainKind::External };
            let addr_info = self.wallet.peek_address(keychain, *idx);
            let addr_str = addr_info.address.to_string();

            let txs: Vec<AddressTx> =
                self.api_get(&format!("{}/address/{}/txs", self.network.api_url(), addr_str))?;

            for tx_data in &txs {
                let mut our_receive: u64 = 0;
                let mut our_send: u64 = 0;

                for vout in &tx_data.vout {
                    if let Some(ref addr) = vout.scriptpubkey_address {
                        if self.is_our_address(addr) {
                            our_receive += vout.value;
                        }
                    }
                }

                for vin in &tx_data.vin {
                    if let Some(ref prevout) = vin.prevout {
                        if let Some(ref addr) = prevout.scriptpubkey_address {
                            if self.is_our_address(addr) {
                                our_send += prevout.value;
                            }
                        }
                    }
                }

                let net_amount = if our_receive > our_send {
                    our_receive as i64 - our_send as i64
                } else if our_send > 0 {
                    -(our_send as i64 - our_receive as i64)
                } else {
                    continue;
                };

                let confirmations = tx_data.status.block_height.unwrap_or(0);

                all_txs.entry(tx_data.txid.clone()).or_insert(TransactionInfo {
                    txid: tx_data.txid.clone(),
                    amount: net_amount,
                    fee: tx_data.fee,
                    confirmations,
                    timestamp: tx_data.status.block_time,
                });
            }
        }

        let mut result: Vec<TransactionInfo> = all_txs.into_values().collect();
        result.sort_by(|a, b| b.timestamp.unwrap_or(0).cmp(&a.timestamp.unwrap_or(0)));
        result.truncate(count);

        Ok(result)
    }

    pub fn wallet_info(&self) -> Result<WalletInfo, String> {
        let balance = self.get_balance()?;
        let revealed = self.get_revealed_info();

        let mut addresses: Vec<AddressInfo> = Vec::new();
        let mut tx_count = 0;

        for (idx, is_internal) in &revealed {
            let keychain = if *is_internal { KeychainKind::Internal } else { KeychainKind::External };
            let addr_info = self.wallet.peek_address(keychain, *idx);
            let addr_str = addr_info.address.to_string();
            let utxos = self.fetch_utxos_for_address(&addr_str)?;
            tx_count += utxos.len();

            addresses.push(AddressInfo {
                address: addr_str,
                index: *idx,
                is_internal: *is_internal,
                used: !utxos.is_empty(),
            });
        }

        Ok(WalletInfo {
            balance,
            addresses,
            transaction_count: tx_count,
        })
    }

    fn get_revealed_info(&self) -> Vec<(u32, bool)> {
        let mut revealed = Vec::new();
        for (keychain, _) in self.wallet.keychains() {
            let kc = if keychain == KeychainKind::Internal { KeychainKind::Internal } else { KeychainKind::External };
            if let Some(idx) = self.wallet.derivation_index(kc) {
                for i in 0..=idx {
                    revealed.push((i, kc == KeychainKind::Internal));
                }
            }
        }
        revealed
    }

    fn is_our_address(&self, addr: &str) -> bool {
        if let Ok(address) = Address::from_str(addr) {
            let script = address.assume_checked().script_pubkey();
            return self.wallet.is_mine(script);
        }
        false
    }

    fn fetch_utxos_for_address(&self, addr: &str) -> Result<Vec<UtxoData>, String> {
        let utxos: Vec<AddressUtxo> =
            self.api_get(&format!("{}/address/{}/utxo", self.network.api_url(), addr))?;

        let address = Address::from_str(addr).map_err(|e| format!("Invalid address: {}", e))?;
        let script = address.assume_checked().script_pubkey();

        let mut result = Vec::new();
        for utxo in &utxos {
            if let Some((_kc, idx)) = self.wallet.derivation_of_spk(script.clone()) {
                result.push(UtxoData {
                    txid: utxo.txid.clone(),
                    vout: utxo.vout,
                    value: utxo.value,
                    script_pubkey: script.clone(),
                    address_index: idx,
                    confirmed: utxo.status.confirmed,
                });
            }
        }
        Ok(result)
    }

    fn fetch_all_utxos(&self) -> Result<Vec<UtxoData>, String> {
        let revealed = self.get_revealed_info();
        let mut all = Vec::new();
        for (idx, is_internal) in &revealed {
            let keychain = if *is_internal { KeychainKind::Internal } else { KeychainKind::External };
            let addr_info = self.wallet.peek_address(keychain, *idx);
            let addr_str = addr_info.address.to_string();
            let utxos = self.fetch_utxos_for_address(&addr_str)?;
            all.extend(utxos);
        }
        all.sort_by(|a, b| b.value.cmp(&a.value));
        Ok(all)
    }

    fn broadcast_tx_hex(&self, tx_hex: &str) -> Result<String, String> {
        let url = format!("{}/tx", self.network.api_url());
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("HTTP client build failed: {}", e))?;

        let resp = client
            .post(&url)
            .body(tx_hex.to_string())
            .header("Content-Type", "text/plain")
            .send()
            .map_err(|e| format!("Broadcast request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            return Err(format!("Broadcast failed ({}): {}", status, body));
        }
        let txid = resp.text().map_err(|e| format!("Failed to read response: {}", e))?;
        Ok(txid.trim().to_string())
    }

    fn api_get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("HTTP client build failed: {}", e))?;

        let resp = client
            .get(url)
            .send()
            .map_err(|e| format!("API request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("API error {}: {}", resp.status(), url));
        }

        resp.json::<T>().map_err(|e| format!("JSON parse failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_mnemonic() -> &'static str {
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
    }

    #[test]
    fn test_wallet_from_mnemonic() {
        let wallet = BitcoinWallet::new(Some(test_mnemonic()), Network::Testnet).unwrap();
        assert!(wallet.mnemonic_phrase().contains("abandon"));
        assert!(wallet.mnemonic_phrase().contains("about"));
        assert_eq!(wallet.network(), Network::Testnet);
    }

    #[test]
    fn test_generate_new_mnemonic() {
        let wallet = BitcoinWallet::new(None, Network::Testnet).unwrap();
        let words: Vec<&str> = wallet.mnemonic_phrase().split_whitespace().collect();
        assert_eq!(words.len(), 12);
    }

    #[test]
    fn test_deterministic_addresses() {
        let mut w1 = BitcoinWallet::new(Some(test_mnemonic()), Network::Testnet).unwrap();
        let mut w2 = BitcoinWallet::new(Some(test_mnemonic()), Network::Testnet).unwrap();

        let a1 = w1.generate_new_address().unwrap();
        let a2 = w2.generate_new_address().unwrap();
        assert_eq!(a1, a2);

        let a3 = w1.generate_new_address().unwrap();
        let a4 = w2.generate_new_address().unwrap();
        assert_eq!(a3, a4);
        assert_ne!(a1, a3);
    }

    #[test]
    fn test_mainnet_vs_testnet_addresses() {
        let mut main = BitcoinWallet::new(Some(test_mnemonic()), Network::Mainnet).unwrap();
        let mut test = BitcoinWallet::new(Some(test_mnemonic()), Network::Testnet).unwrap();

        let main_addr = main.generate_new_address().unwrap();
        let test_addr = test.generate_new_address().unwrap();

        assert!(main_addr.starts_with("bc1"), "mainnet: {}", main_addr);
        assert!(test_addr.starts_with("tb1"), "testnet: {}", test_addr);
    }

    #[test]
    fn test_network_mapping() {
        assert_eq!(Network::Mainnet.to_bdk(), BdkNetwork::Bitcoin);
        assert_eq!(Network::Testnet.to_bdk(), BdkNetwork::Testnet);
        assert_eq!(Network::Signet.to_bdk(), BdkNetwork::Signet);
    }

    #[test]
    fn test_invalid_mnemonic() {
        let result = BitcoinWallet::new(Some("not a valid mnemonic phrase"), Network::Testnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_balance_api_url() {
        assert_eq!(Network::Mainnet.api_url(), "https://blockstream.info/api");
        assert_eq!(Network::Testnet.api_url(), "https://blockstream.info/testnet/api");
        assert_eq!(Network::Signet.api_url(), "https://blockstream.info/signet/api");
    }

    #[test]
    fn test_bip84_path() {
        assert_eq!(Network::Mainnet.bip84_path(), "m/84'/0'/0'");
        assert_eq!(Network::Testnet.bip84_path(), "m/84'/1'/0'");
        assert_eq!(Network::Signet.bip84_path(), "m/84'/1'/0'");
    }

    #[test]
    fn test_balance_parsing() {
        let json = r#"[
            {"txid":"a1b2","vout":0,"value":50000,"status":{"confirmed":true,"block_height":100,"block_time":123456}},
            {"txid":"c3d4","vout":1,"value":25000,"status":{"confirmed":false,"block_height":null,"block_time":null}}
        ]"#;
        let utxos: Vec<AddressUtxo> = serde_json::from_str(json).unwrap();
        assert_eq!(utxos.len(), 2);
        assert!(utxos[0].status.confirmed);
        assert!(!utxos[1].status.confirmed);
        assert_eq!(utxos[0].value, 50000);
        assert_eq!(utxos[1].value, 25000);
    }

    #[test]
    fn test_tx_parsing() {
        let json = r#"{
            "txid":"deadbeef",
            "fee":10000,
            "status":{"confirmed":true,"block_height":200,"block_time":123456789},
            "vin":[{"txid":"a1","vout":0,"prevout":{"scriptpubkey_address":"tb1qxyz","value":100000}}],
            "vout":[{"value":50000,"scriptpubkey_address":"tb1qabc"},{"value":40000,"scriptpubkey_address":"tb1qdef"}]
        }"#;
        let tx: AddressTx = serde_json::from_str(json).unwrap();
        assert_eq!(tx.txid, "deadbeef");
        assert_eq!(tx.fee, 10000);
        assert!(tx.status.confirmed);
        assert_eq!(tx.vout.len(), 2);
        assert_eq!(tx.vout[0].value, 50000);
        assert_eq!(tx.vin[0].prevout.as_ref().unwrap().value, 100000);
    }

    #[test]
    fn test_utxo_data_construction() {
        let script = Address::from_str("tb1q7kn55vf3mmd40gyj46r24l8h3j2v4gxqgjqg7g")
            .unwrap()
            .assume_checked()
            .script_pubkey();
        let utxo = UtxoData {
            txid: "feed0001".to_string(),
            vout: 0,
            value: 100000,
            script_pubkey: script,
            address_index: 0,
            confirmed: true,
        };
        assert_eq!(utxo.value, 100000);
        assert!(utxo.confirmed);
    }
}
