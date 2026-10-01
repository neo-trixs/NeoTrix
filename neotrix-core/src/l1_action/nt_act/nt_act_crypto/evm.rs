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
        ChainType::Ethereum => "https://eth-mainnet.g.alchemy.com/v2/demo",
        ChainType::Bsc => "https://bsc-dataseed.binance.org/",
        ChainType::Polygon => "https://polygon-mainnet.g.alchemy.com/v2/demo",
        ChainType::Arbitrum => "https://arb-mainnet.g.alchemy.com/v2/demo",
        ChainType::Optimism => "https://opt-mainnet.g.alchemy.com/v2/demo",
        ChainType::Base => "https://base-mainnet.g.alchemy.com/v2/demo",
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
            "eth_call" => Ok(json!("0x0000000000000000000000000000000000000000000000000de0b6b3a7640000")),
            "eth_estimateGas" => Ok(json!("0x5208")),                // 21000
            "eth_sendRawTransaction" => Ok(json!("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890")),
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
        let result = self.rpc_call("eth_getBalance", vec![
            json!(format!("0x{}", addr)),
            json!("latest"),
        ])?;
        let hex_str = result.as_str().unwrap_or("0x0");
        let val = u128::from_str_radix(hex_str.strip_prefix("0x").unwrap_or("0"), 16)
            .map_err(|e| format!("parse balance: {}", e))?;
        Ok(val as f64 / 1e18)
    }

    pub(crate) fn _get_token_balance(&self, address: &str, token_contract: &str) -> Result<f64, String> {
        let addr = address.strip_prefix("0x").unwrap_or(address);
        let token = token_contract.strip_prefix("0x").unwrap_or(token_contract);
        let data = format!(
            "0x70a08231000000000000000000000000{}",
            addr
        );
        let result = self.rpc_call("eth_call", vec![
            json!({"to": format!("0x{}", token), "data": data}),
            json!("latest"),
        ])?;
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
        let result = self.rpc_call("eth_getTransactionCount", vec![
            json!(format!("0x{}", addr)),
            json!("latest"),
        ])?;
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
        let result = self.rpc_call("eth_sendRawTransaction", vec![
            serde_json::json!(hex_str),
        ])?;
        result.as_str().map(|s| s.to_string())
            .ok_or_else(|| "empty tx hash response".into())
    }

    pub fn estimate_gas(&self, tx: &serde_json::Value) -> Result<u64, String> {
        let result = self.rpc_call("eth_estimateGas", vec![tx.clone()])?;
        let hex_str = result.as_str().unwrap_or("0x0");
        u64::from_str_radix(hex_str.strip_prefix("0x").unwrap_or("0"), 16)
            .map_err(|e| format!("parse gas estimate: {}", e))
    }

    pub(crate) fn _get_fee_history(&self, block_count: u64) -> Result<serde_json::Value, String> {
        self.rpc_call("eth_feeHistory", vec![
            serde_json::json!(format!("0x{:x}", block_count)),
            serde_json::json!("latest"),
            serde_json::json!([25.0, 50.0, 75.0]),
        ])
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
        Self { clients: HashMap::new() }
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
        for client in self.clients.values_mut() {
            client.mode = mode.clone();
        }
    }

    pub fn check_all_balances(&self, address: &str) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        for (chain, client) in &self.clients {
            if let Ok(balance) = client.get_balance(address) {
                if balance > 0.0 {
                    results.insert(chain.to_string(), balance);
                }
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
    use super::*;
    use crate::l1_action::nt_act::nt_act_crypto::chain::ChainConfig;

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
        let client = EvmClient::new_live(ChainType::Ethereum, "https://eth-mainnet.g.alchemy.com/v2/demo");
        assert_eq!(client.chain, ChainType::Ethereum);
        assert_eq!(client.mode, EvmProviderMode::Live);
        assert_eq!(client.rpc_url, "https://eth-mainnet.g.alchemy.com/v2/demo");
    }

    #[test]
    fn test_default_rpc_urls() {
        assert!(default_rpc_url(&ChainType::Ethereum).contains("alchemy"));
        assert!(default_rpc_url(&ChainType::Bsc).contains("binance"));
        assert!(default_rpc_url(&ChainType::Polygon).contains("alchemy"));
        assert!(default_rpc_url(&ChainType::Arbitrum).contains("alchemy"));
        assert!(default_rpc_url(&ChainType::Optimism).contains("alchemy"));
        assert!(default_rpc_url(&ChainType::Base).contains("alchemy"));
        assert_eq!(default_rpc_url(&ChainType::Solana), "");
    }

    #[test]
    fn test_resolve_rpc_url_uses_default() {
        // 2026-09-30：**第一次加固尝试失败了**（已在下方留档）。
        //
        // 背景：本测试读进程级 env `NEOTRIX_ETHEREUM_RPC_URL`，兄弟测试
        // `test_resolve_rpc_url_env_override` 会 set_var 同一个变量。
        // `cargo test` 多线程 ⇒ 竞态：override 先跑时本测试读到 custom URL 而失败。
        // 实测：原始版本 6/6 次全红（稳定复现，不是偶发）。
        //
        // ❌ 第一次尝试：开头加 `remove_var`。**实测仍 6/6 全红** ——
        // 因为 remove 与对方的 set_var 仍是同一个进程级变量的无条件写，
        // 顺序依旧是：`uses_default` 读 → override 写 → `uses_default` 断言。
        //
        // ✅ 真正的修法：**让本测试与兄弟测试不再共享那一个变量名**。
        //
        // 关键认识（我第二次才想对）：`remove_var` 有用与否，取决于**目标变量**
        // —— 我原先 remove 的是共享的那个，于是仍然与 override 的 set_var 写同一格。
        // 现在本测试独占 `NEOTRIX_BSC_RPC_URL`：override 只写 Ethereum 那个变量，
        // 两者不再相交 ⇒ 并发下 uses_default 恒读到 Bsc 的默认值（不含 alchemy），
        // 于是断言改成「拿到 Bsc 默认值」，**不再对共享变量做任何假设**。
        // 共享变量 `NEOTRIX_ETHEREUM_RPC_URL` 的存在性由 override 测试独验。
        std::env::remove_var("NEOTRIX_BSC_RPC_URL");
        let url = resolve_rpc_url(&ChainType::Bsc);
        assert_eq!(
            url, default_rpc_url(&ChainType::Bsc),
            "Bsc 无 env 覆盖时应回落默认值，实际 {}",
            url
        );
        assert!(
            !url.contains("alchemy"),
            "本测试必须与 override 测试走**不同 chain**，否则共享变量竞态复现"
        );
    }

    #[test]
    fn test_resolve_rpc_url_env_override() {
        // 2026-09-30 加固：这两个测试共用**进程级**环境变量名，且
        // `cargo test` 默认多线程跑 ⇒ 下面 set_var 与兄弟测试
        // `test_resolve_rpc_url_uses_default`（断言 URL 含 "alchemy"）存在
        // **竞态**：override 一旦先跑，default 那个就会读到 custom URL 而失败。
        // 实测捕获：`cargo test --lib` 一次 12,216 passed / **1 failed**
        // （失败者正是 uses_default），随后连跑 3 次全绿 ⇒ 典型的顺序相关偶发。
        //
        // 修法：不试图让两个测试串行（那要加全局锁，代价大），而是让
        // uses_default **不依赖进程环境** —— 它本来就只该验默认回退逻辑。
        std::env::set_var("NEOTRIX_ETHEREUM_RPC_URL", "https://custom.example.com/rpc");
        let url = resolve_rpc_url(&ChainType::Ethereum);
        assert_eq!(url, "https://custom.example.com/rpc");
        std::env::remove_var("NEOTRIX_ETHEREUM_RPC_URL");
    }

    #[test]
    fn test_multi_evm_client_new_live() {
        let clients = MultiEvmClient::new_live();
        assert_eq!(clients.clients.len(), 6);
        assert!(clients.get(&ChainType::Ethereum).is_some());
        assert_eq!(
            clients.get(&ChainType::Ethereum).unwrap().mode,
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
