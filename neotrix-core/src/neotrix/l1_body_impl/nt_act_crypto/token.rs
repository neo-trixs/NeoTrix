use super::chain::ChainType;
use super::evm::EvmClient;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub chain: ChainType,
    pub price_usd: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct TokenBalance {
    pub token: TokenInfo,
    pub raw_balance: u128,
    pub balance: f64,
    pub value_usd: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct ApprovalInfo {
    pub token_address: String,
    pub spender: String,
    pub current_allowance: u128,
    pub chain: ChainType,
}

pub struct Erc20Abi;

impl Erc20Abi {
    pub fn balance_of(owner: &str) -> Vec<u8> {
        let owner_bytes = addr_to_bytes(owner);
        let mut data = Vec::with_capacity(36);
        data.extend_from_slice(&hex::decode("70a08231").unwrap());
        data.extend_from_slice(&owner_bytes);
        data
    }

    pub fn transfer(to: &str, amount: u128) -> Vec<u8> {
        let to_bytes = addr_to_bytes(to);
        let amount_bytes = u256_padded(&amount.to_be_bytes());
        let mut data = Vec::with_capacity(68);
        data.extend_from_slice(&hex::decode("a9059cbb").unwrap());
        data.extend_from_slice(&to_bytes);
        data.extend_from_slice(&amount_bytes);
        data
    }

    pub fn approve(spender: &str, amount: u128) -> Vec<u8> {
        let spender_bytes = addr_to_bytes(spender);
        let amount_bytes = u256_padded(&amount.to_be_bytes());
        let mut data = Vec::with_capacity(68);
        data.extend_from_slice(&hex::decode("095ea7b3").unwrap());
        data.extend_from_slice(&spender_bytes);
        data.extend_from_slice(&amount_bytes);
        data
    }

    pub fn allowance(owner: &str, spender: &str) -> Vec<u8> {
        let owner_bytes = addr_to_bytes(owner);
        let spender_bytes = addr_to_bytes(spender);
        let mut data = Vec::with_capacity(68);
        data.extend_from_slice(&hex::decode("dd62ed3e").unwrap());
        data.extend_from_slice(&owner_bytes);
        data.extend_from_slice(&spender_bytes);
        data
    }

    pub fn decode_u256(hex_data: &str) -> Result<u128, String> {
        let s = hex_data.strip_prefix("0x").unwrap_or(hex_data);
        if s.is_empty() || s == "0" {
            return Ok(0);
        }
        let padded = if s.len() % 2 == 1 {
            format!("0{s}")
        } else {
            s.to_string()
        };
        let bytes = hex::decode(&padded).map_err(|e| format!("hex decode: {e}"))?;
        if bytes.is_empty() {
            return Ok(0);
        }
        let start = bytes.len().saturating_sub(16);
        let mut arr = [0u8; 16];
        arr.copy_from_slice(&bytes[start..]);
        Ok(u128::from_be_bytes(arr))
    }

    pub fn decode_address(hex_data: &str) -> Result<String, String> {
        let s = hex_data.strip_prefix("0x").unwrap_or(hex_data);
        let bytes = hex::decode(s).map_err(|e| format!("hex decode: {}", e))?;
        if bytes.len() < 32 {
            return Err("data too short for address".into());
        }
        let addr_bytes = &bytes[bytes.len() - 20..];
        Ok(format!("0x{}", hex::encode(addr_bytes)))
    }
}

pub struct TokenRegistry {
    tokens: HashMap<(ChainType, String), TokenInfo>,
    known_tokens: Vec<TokenInfo>,
}

impl TokenRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            tokens: HashMap::new(),
            known_tokens: Vec::new(),
        };
        reg.register_known();
        reg
    }

    fn register_known(&mut self) {
        let known = vec![
            TokenInfo {
                address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".into(),
                symbol: "WETH".into(),
                name: "Wrapped Ether".into(),
                decimals: 18,
                chain: ChainType::Ethereum,
                price_usd: None,
            },
            TokenInfo {
                address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(),
                symbol: "USDC".into(),
                name: "USD Coin".into(),
                decimals: 6,
                chain: ChainType::Ethereum,
                price_usd: None,
            },
            TokenInfo {
                address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".into(),
                symbol: "USDT".into(),
                name: "Tether USD".into(),
                decimals: 6,
                chain: ChainType::Ethereum,
                price_usd: None,
            },
            TokenInfo {
                address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".into(),
                symbol: "WBTC".into(),
                name: "Wrapped Bitcoin".into(),
                decimals: 8,
                chain: ChainType::Ethereum,
                price_usd: None,
            },
        ];
        for t in known {
            self.known_tokens.push(t.clone());
            let key = (t.chain.clone(), t.address.clone());
            self.tokens.insert(key, t);
        }
    }

    pub fn register(&mut self, token: TokenInfo) {
        let key = (token.chain.clone(), token.address.clone());
        self.tokens.insert(key, token);
    }

    pub fn get(&self, chain: &ChainType, address: &str) -> Option<&TokenInfo> {
        let addr = address.strip_prefix("0x").unwrap_or(address);
        let addr_normalized = if addr.len() < 40 {
            format!("{:0>40}", addr)
        } else {
            addr.to_string()
        };
        let full_addr = format!("0x{}", addr_normalized);
        self.tokens.get(&(chain.clone(), full_addr))
    }

    pub fn get_balance(
        &self,
        token: &TokenInfo,
        owner: &str,
        client: &EvmClient,
    ) -> Result<TokenBalance, String> {
        let data = Erc20Abi::balance_of(owner);
        let result = client.rpc_call("eth_call", vec![
            serde_json::json!({"to": &token.address, "data": format!("0x{}", hex::encode(&data))}),
            serde_json::json!("latest"),
        ])?;
        let hex_str = result.as_str().unwrap_or("0x0");
        let raw = Erc20Abi::decode_u256(hex_str).unwrap_or(0);
        let divisor = 10u128.pow(token.decimals as u32);
        let balance = raw as f64 / divisor as f64;

        Ok(TokenBalance {
            token: token.clone(),
            raw_balance: raw,
            balance,
            value_usd: token.price_usd.map(|p| balance * p),
        })
    }

    pub fn all_tokens(&self) -> &[TokenInfo] {
        &self.known_tokens
    }

    pub fn tokens_by_chain(&self, chain: &ChainType) -> Vec<&TokenInfo> {
        self.known_tokens
            .iter()
            .filter(|t| t.chain == *chain)
            .collect()
    }
}

impl Default for TokenRegistry {
    fn default() -> Self {
        Self::new()
    }
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

    #[test]
    fn test_balance_of_encoding() {
        let data = Erc20Abi::balance_of("0x1234567890abcdef1234567890abcdef12345678");
        assert_eq!(data.len(), 36);
        assert_eq!(&data[..4], &hex::decode("70a08231").unwrap());
    }

    #[test]
    fn test_transfer_encoding() {
        let data = Erc20Abi::transfer(
            "0x1234567890abcdef1234567890abcdef12345678",
            1000000,
        );
        assert_eq!(data.len(), 68);
        assert_eq!(&data[..4], &hex::decode("a9059cbb").unwrap());
    }

    #[test]
    fn test_approve_encoding() {
        let data = Erc20Abi::approve(
            "0x1234567890abcdef1234567890abcdef12345678",
            u128::MAX,
        );
        assert_eq!(data.len(), 68);
        assert_eq!(&data[..4], &hex::decode("095ea7b3").unwrap());
    }

    #[test]
    fn test_allowance_encoding() {
        let data = Erc20Abi::allowance(
            "0x1111111111111111111111111111111111111111",
            "0x2222222222222222222222222222222222222222",
        );
        assert_eq!(data.len(), 68);
        assert_eq!(&data[..4], &hex::decode("dd62ed3e").unwrap());
    }

    #[test]
    fn test_decode_u256_zero() {
        let val = Erc20Abi::decode_u256("0x0").unwrap();
        assert_eq!(val, 0);
    }

    #[test]
    fn test_decode_u256_value() {
        let val = Erc20Abi::decode_u256("0x0000000000000000000000000000000000000000000000000de0b6b3a7640000").unwrap();
        assert_eq!(val, 1000000000000000000);
    }

    #[test]
    fn test_decode_u256_small() {
        let val = Erc20Abi::decode_u256("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        assert_eq!(val, 1);
    }

    #[test]
    fn test_decode_address() {
        let addr = Erc20Abi::decode_address(
            "0x0000000000000000000000001234567890abcdef1234567890abcdef12345678",
        )
        .unwrap();
        assert_eq!(addr, "0x1234567890abcdef1234567890abcdef12345678");
    }

    #[test]
    fn test_token_registry_known() {
        let reg = TokenRegistry::new();
        assert_eq!(reg.all_tokens().len(), 4);
    }

    #[test]
    fn test_token_registry_get() {
        let reg = TokenRegistry::new();
        let weth = reg.get(
            &ChainType::Ethereum,
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        );
        assert!(weth.is_some());
        assert_eq!(weth.unwrap().symbol, "WETH");
    }

    #[test]
    fn test_token_by_chain() {
        let reg = TokenRegistry::new();
        let eth_tokens = reg.tokens_by_chain(&ChainType::Ethereum);
        assert_eq!(eth_tokens.len(), 4);
    }

    #[test]
    fn test_addr_to_bytes_padding() {
        let bytes = addr_to_bytes("0x1234567890abcdef1234567890abcdef12345678");
        assert_eq!(bytes.len(), 32);
        assert_eq!(bytes[12], 0x12);
        assert_eq!(bytes[31], 0x78);
    }

    #[test]
    fn test_u256_padded() {
        let bytes = u256_padded(&1u128.to_be_bytes());
        assert_eq!(bytes[31], 1);
        assert_eq!(bytes[0..31].to_vec(), vec![0u8; 31]);
    }
}
