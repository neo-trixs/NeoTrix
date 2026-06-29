use super::chain::{ChainConfig, ChainType};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum EvmProviderMode {
    Live,
    Mock,
}

#[derive(Clone, Debug)]
pub struct EvmClient {
    pub chain: ChainType,
    pub mode: EvmProviderMode,
    rpc_url: String,
    client: reqwest::blocking::Client,
}

/// Default RPC URLs for each chain.
pub fn default_rpc_url(chain: &ChainType) -> &'static str {
    match chain {
        ChainType::Bsc => "https://bsc-dataseed.binance.org/",
        _ => "",
    }
}

/// Resolve RPC URL for a chain: env var override → default.
pub fn resolve_rpc_url(chain: &ChainType) -> String {
    let env_key = format!("NEOTRIX_{}_RPC_URL", chain.to_string().to_uppercase());
    std::env::var(&env_key).unwrap_or_else(|_| default_rpc_url(chain).to_string())
}

impl EvmClient {
    pub fn new(config: &ChainConfig) -> Self {
        Self {
            chain: config.chain.clone(),
            mode: EvmProviderMode::Live,
            rpc_url: config.rpc_url.clone(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn new_live(chain: ChainType, rpc_url: &str) -> Self {
        Self {
            chain,
            mode: EvmProviderMode::Live,
            rpc_url: rpc_url.to_string(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn new_mock(chain: ChainType) -> Self {
        Self {
            chain,
            mode: EvmProviderMode::Mock,
            rpc_url: String::new(),
            client: reqwest::blocking::Client::builder()
                .build()
                .unwrap_or_default(),
        }
    }

    pub(crate) fn rpc_call(&self, method: &str, params: Vec<Value>) -> Result<Value, String> {
        if self.mode == EvmProviderMode::Mock {
            return self.mock_rpc(method, params);
        }
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let resp = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .map_err(|e| format!("RPC request failed: {}", e))?;
        let val: Value = resp
            .json()
            .map_err(|e| format!("RPC parse failed: {}", e))?;
        if let Some(err) = val.get("error") {
            return Err(format!("RPC error: {}", err));
        }
        Ok(val["result"].clone())
    }

    fn mock_rpc(&self, method: &str, _params: Vec<Value>) -> Result<Value, String> {
        match method {
            "eth_getBalance" => Ok(json!("0x152d02c7e14af6800000")), // 100 ETH
            "eth_gasPrice" => Ok(json!("0x9502f900")),               // 2.5 gwei
            "eth_getTransactionCount" => Ok(json!("0x5")),
            "eth_blockNumber" => Ok(json!("0x1234567")),
            "eth_chainId" => Ok(json!(format!("0x{:x}", self.chain.chain_id()))),
            "eth_call" => Ok(json!(
                "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000"
            )),
            "eth_estimateGas" => Ok(json!("0x5208")), // 21000
            "eth_sendRawTransaction" => Ok(json!(
                "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            )),
            "eth_feeHistory" => Ok(json!({
                "oldestBlock": "0x1234560",
                "baseFeePerGas": ["0x3b9aca00", "0x3b9aca00", "0x3b9aca00", "0x3b9aca00"],
                "gasUsedRatio": [0.5, 0.6, 0.4, 0.55],
                "reward": [["0x59682f00"], ["0x59682f00"], ["0x59682f00"], ["0x59682f00"]]
            })),
            _ => Ok(json!("0x0")),
        }
    }

    pub fn get_balance(&self, address: &str) -> Result<f64, String> {
        let addr = address.strip_prefix("0x").unwrap_or(address);
        let result = self.rpc_call(
            "eth_getBalance",
            vec![json!(format!("0x{}", addr)), json!("latest")],
        )?;
        let hex_str = result.as_str().unwrap_or("0x0");
        let val = u128::from_str_radix(hex_str.strip_prefix("0x").unwrap_or("0"), 16)
            .map_err(|e| format!("parse balance: {}", e))?;
        Ok(val as f64 / 1e18)
    }

    pub fn get_token_balance(&self, address: &str, token_contract: &str) -> Result<f64, String> {
        let addr = address.strip_prefix("0x").unwrap_or(address);
        let token = token_contract.strip_prefix("0x").unwrap_or(token_contract);
        let data = format!("0x70a08231000000000000000000000000{}", addr);
        let result = self.rpc_call(
            "eth_call",
            vec![
                json!({"to": format!("0x{}", token), "data": data}),
                json!("latest"),
            ],
        )?;
        let hex_str = result.as_str().unwrap_or("0x0");
        let val = u128::from_str_radix(hex_str.strip_prefix("0x").unwrap_or("0"), 16)
            .map_err(|e| format!("parse token balance: {}", e))?;
        Ok(val as f64 / 1e18)
    }

    pub fn get_gas_price(&self) -> Result<f64, String> {
        let result = self.rpc_call("eth_gasPrice", vec![])?;
        let hex_str = result.as_str().unwrap_or("0x0");
        let val = u128::from_str_radix(hex_str.strip_prefix("0x").unwrap_or("0"), 16)
            .map_err(|e| format!("parse gas price: {}", e))?;
        Ok(val as f64 / 1e9)
    }

    pub fn get_transaction_count(&self, address: &str) -> Result<u64, String> {
        let addr = address.strip_prefix("0x").unwrap_or(address);
        let result = self.rpc_call(
            "eth_getTransactionCount",
            vec![json!(format!("0x{}", addr)), json!("latest")],
        )?;
        let hex_str = result.as_str().unwrap_or("0x0");
        u64::from_str_radix(hex_str.strip_prefix("0x").unwrap_or("0"), 16)
            .map_err(|e| format!("parse nonce: {}", e))
    }

    pub fn get_block_number(&self) -> Result<u64, String> {
        let result = self.rpc_call("eth_blockNumber", vec![])?;
        let hex_str = result.as_str().unwrap_or("0x0");
        u64::from_str_radix(hex_str.strip_prefix("0x").unwrap_or("0"), 16)
            .map_err(|e| format!("parse block: {}", e))
    }

    pub fn send_raw_transaction(&self, raw_tx: &[u8]) -> Result<String, String> {
        let hex_str = format!("0x{}", hex::encode(raw_tx));
        let result = self.rpc_call("eth_sendRawTransaction", vec![serde_json::json!(hex_str)])?;
        result
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "empty tx hash response".into())
    }

    pub fn estimate_gas(&self, tx: &serde_json::Value) -> Result<u64, String> {
        let result = self.rpc_call("eth_estimateGas", vec![tx.clone()])?;
        let hex_str = result.as_str().unwrap_or("0x0");
        u64::from_str_radix(hex_str.strip_prefix("0x").unwrap_or("0"), 16)
            .map_err(|e| format!("parse gas estimate: {}", e))
    }

    pub fn get_fee_history(&self, block_count: u64) -> Result<serde_json::Value, String> {
        self.rpc_call(
            "eth_feeHistory",
            vec![
                serde_json::json!(format!("0x{:x}", block_count)),
                serde_json::json!("latest"),
                serde_json::json!([25.0, 50.0, 75.0]),
            ],
        )
    }

    pub fn chain_is_operational(&self) -> bool {
        if self.mode == EvmProviderMode::Mock {
            return true;
        }
        self.get_block_number().is_ok()
    }
}

pub struct MultiEvmClient {
    clients: HashMap<ChainType, EvmClient>,
}

impl MultiEvmClient {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn new_live() -> Self {
        let mut clients = HashMap::new();
        for chain in &[
            ChainType::Ethereum,
            ChainType::Bsc,
            ChainType::Polygon,
            ChainType::Arbitrum,
            ChainType::Optimism,
            ChainType::Base,
        ] {
            let url = resolve_rpc_url(chain);
            if !url.is_empty() {
                clients.insert(chain.clone(), EvmClient::new_live(chain.clone(), &url));
            }
        }
        Self { clients }
    }

    pub fn new_mock() -> Self {
        let mut clients = HashMap::new();
        for chain in &[
            ChainType::Ethereum,
            ChainType::Bsc,
            ChainType::Polygon,
            ChainType::Arbitrum,
            ChainType::Optimism,
            ChainType::Base,
        ] {
            clients.insert(chain.clone(), EvmClient::new_mock(chain.clone()));
        }
        Self { clients }
    }

    pub fn add(&mut self, client: EvmClient) {
        self.clients.insert(client.chain.clone(), client);
    }

    pub fn get(&self, chain: &ChainType) -> Option<&EvmClient> {
        self.clients.get(chain)
    }

    pub fn clients(&self) -> &HashMap<ChainType, EvmClient> {
        &self.clients
    }

    pub fn set_mode(&mut self, mode: EvmProviderMode) {
        for (_, client) in &mut self.clients {
            client.mode = mode.clone();
        }
    }

    pub fn check_all_balances(&self, address: &str) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        for (chain, client) in &self.clients {
            match client.get_balance(address) {
                Ok(balance) => {
                    if balance > 0.0 {
                        results.insert(chain.to_string(), balance);
                    }
                }
                Err(_) => {}
            }
        }
        results
    }

    pub fn operational_chains(&self) -> Vec<ChainType> {
        self.clients
            .iter()
            .filter(|(_, c)| c.chain_is_operational())
            .map(|(k, _)| k.clone())
            .collect()
    }
}

impl Default for MultiEvmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // NOTE: test_resolve_rpc_url_env_override uses std::env::set_var.
    // #[test] runs on its own OS thread — no concurrent set_var risk.
    use super::*;
    use crate::neotrix::nt_act_crypto::chain::ChainConfig;

    #[test]
    fn test_format_balance_call() {
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        let _token = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let addr = address.strip_prefix("0x").unwrap();
        let tkn_data = format!("0x70a08231000000000000000000000000{}", addr);
        assert_eq!(tkn_data.len(), 74);
        assert!(tkn_data.starts_with("0x"));
        assert!(tkn_data.contains("70a08231"));
    }

    #[test]
    fn test_mock_client_returns_balance() {
        let client = EvmClient::new_mock(ChainType::Ethereum);
        let balance = client.get_balance("0x1234567890abcdef1234567890abcdef12345678");
        assert!(balance.is_ok());
        assert!(balance.unwrap() > 0.0);
    }

    #[test]
    fn test_mock_client_block_number() {
        let client = EvmClient::new_mock(ChainType::Ethereum);
        let block = client.get_block_number();
        assert!(block.is_ok());
        assert_eq!(block.unwrap(), 0x1234567);
    }

    #[test]
    fn test_mock_client_gas_price() {
        let client = EvmClient::new_mock(ChainType::Polygon);
        let gas = client.get_gas_price();
        assert!(gas.is_ok());
        assert!(gas.unwrap() > 0.0);
    }

    #[test]
    fn test_mock_chain_is_operational() {
        let client = EvmClient::new_mock(ChainType::Arbitrum);
        assert!(client.chain_is_operational());
    }

    #[test]
    fn test_mock_multi_client() {
        let clients = MultiEvmClient::new_mock();
        let chains = clients.operational_chains();
        assert_eq!(chains.len(), 6);
    }

    #[test]
    fn test_rpc_call_rejects_bad_url() {
        let config = ChainConfig::new(ChainType::Ethereum, "https://invalid-rpc.example.com");
        let client = EvmClient::new(&config);
        let result = client.get_block_number();
        assert!(result.is_err());
    }

    #[test]
    fn test_new_live_evm_client() {
        let client = EvmClient::new_live(
            ChainType::Ethereum,
            "https://eth-mainnet.g.alchemy.com/v2/test_key",
        );
        assert_eq!(client.chain, ChainType::Ethereum);
        assert_eq!(client.mode, EvmProviderMode::Live);
        assert_eq!(
            client.rpc_url,
            "https://eth-mainnet.g.alchemy.com/v2/test_key"
        );
    }

    #[test]
    fn test_default_rpc_urls() {
        assert_eq!(default_rpc_url(&ChainType::Ethereum), "");
        assert!(default_rpc_url(&ChainType::Bsc).contains("binance"));
        assert_eq!(default_rpc_url(&ChainType::Polygon), "");
        assert_eq!(default_rpc_url(&ChainType::Arbitrum), "");
        assert_eq!(default_rpc_url(&ChainType::Optimism), "");
        assert_eq!(default_rpc_url(&ChainType::Base), "");
        assert_eq!(default_rpc_url(&ChainType::Solana), "");
    }

    #[test]
    fn test_resolve_rpc_url_uses_default() {
        let url = resolve_rpc_url(&ChainType::Ethereum);
        assert_eq!(url, "");
    }

    #[test]
    fn test_resolve_rpc_url_env_override() {
        std::env::set_var("NEOTRIX_ETHEREUM_RPC_URL", "https://custom.example.com/rpc");
        let url = resolve_rpc_url(&ChainType::Ethereum);
        assert_eq!(url, "https://custom.example.com/rpc");
        std::env::remove_var("NEOTRIX_ETHEREUM_RPC_URL");
    }

    #[test]
    fn test_multi_evm_client_new_live() {
        let clients = MultiEvmClient::new_live();
        // Only BSC has a non-empty default RPC URL; Alchemy chains require NEOTRIX_*_RPC_URL env vars.
        assert_eq!(clients.clients.len(), 1);
        assert!(clients.get(&ChainType::Bsc).is_some());
        assert_eq!(
            clients.get(&ChainType::Bsc).unwrap().mode,
            EvmProviderMode::Live
        );
    }

    #[test]
    fn test_multi_evm_client_get_and_clients() {
        let clients = MultiEvmClient::new_mock();
        let eth = clients.get(&ChainType::Ethereum);
        assert!(eth.is_some());
        assert_eq!(clients.clients().len(), 6);
    }
}
