use super::chain::ChainType;
use super::evm::EvmClient;
use super::token::TokenInfo;

#[derive(Clone, Debug)]
pub struct SwapQuote {
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub amount_in: u128,
    pub amount_out: u128,
    pub route: Vec<String>,
    pub price_impact: f64,
    pub estimated_gas: u64,
}

#[derive(Clone, Debug)]
pub struct DexPool {
    pub address: String,
    pub dex_name: String,
    pub protocol: DexProtocol,
    pub token0: String,
    pub token1: String,
    pub reserve0: u128,
    pub reserve1: u128,
    pub chain: ChainType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DexProtocol {
    UniswapV2,
    UniswapV3,
}

impl DexProtocol {
    pub fn name(&self) -> &str {
        match self {
            DexProtocol::UniswapV2 => "Uniswap V2",
            DexProtocol::UniswapV3 => "Uniswap V3",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DexConfig {
    pub chain: ChainType,
    pub factory_v2: Option<String>,
    pub router_v2: Option<String>,
    pub router_v3: Option<String>,
    pub quoter_v3: Option<String>,
}

pub struct DexRegistry {
    pub dexes: Vec<DexConfig>,
}

impl DexRegistry {
    pub fn new() -> Self {
        Self { dexes: vec![] }
    }

    pub fn register_defaults(&mut self) {
        let defaults = vec![
            DexConfig {
                chain: ChainType::Ethereum,
                factory_v2: Some("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".into()),
                router_v2: Some("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".into()),
                router_v3: Some("0xE592427A0AEce92De3Edee1F18E0157C05861564".into()),
                quoter_v3: Some("0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".into()),
            },
            DexConfig {
                chain: ChainType::Bsc,
                factory_v2: Some("0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73".into()),
                router_v2: Some("0x10ED43C718714eb63d5aA57B78B54704E256024E".into()),
                router_v3: Some("0x1b81D678ffb9C0263b24A97847620C99d213eB14".into()),
                quoter_v3: Some("0x78D78E420Da98ad378D7799bE8f4AF69033EB077".into()),
            },
            DexConfig {
                chain: ChainType::Polygon,
                factory_v2: Some("0x5757371414417b8C6CAad45bAeF941aBc7d3Ab32".into()),
                router_v2: Some("0xa5E0829CaCEd8fFDD4De3c43696c57F7D7A678ff".into()),
                router_v3: Some("0xE592427A0AEce92De3Edee1F18E0157C05861564".into()),
                quoter_v3: Some("0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".into()),
            },
        ];
        for dex in defaults {
            let chain = dex.chain.clone();
            if !self.dexes.iter().any(|d| d.chain == chain) {
                self.dexes.push(dex);
            }
        }
    }

    pub fn router_v2(&self, chain: &ChainType) -> Option<&str> {
        self.dexes
            .iter()
            .find(|d| d.chain == *chain)
            .and_then(|d| d.router_v2.as_deref())
    }

    pub fn router_v3(&self, chain: &ChainType) -> Option<&str> {
        self.dexes
            .iter()
            .find(|d| d.chain == *chain)
            .and_then(|d| d.router_v3.as_deref())
    }
}

impl Default for DexRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DexSwapper;

impl DexSwapper {
    pub fn encode_swap_exact_tokens_for_tokens_v2(
        amount_in: u128,
        amount_out_min: u128,
        path: &[String],
        to: &str,
        deadline: u64,
    ) -> Vec<u8> {
        let deadline_bytes = u256_padded(&deadline.to_be_bytes());

        let mut data = Vec::with_capacity(4 + 32 + 32 + 32 + 32 + 32);
        data.extend_from_slice(
            &hex::decode("38ed1739")
                .expect("compile-time hex literal for swapExactTokensForTokens selector"),
        );
        data.extend_from_slice(&u256_padded(&amount_in.to_be_bytes()));
        data.extend_from_slice(&u256_padded(&amount_out_min.to_be_bytes()));

        let path_offset: u64 = 160;
        data.extend_from_slice(&u256_padded(&path_offset.to_be_bytes()));
        data.extend_from_slice(&addr_to_bytes(to));
        data.extend_from_slice(&deadline_bytes);

        let path_len = path.len() as u64;
        data.extend_from_slice(&u256_padded(&path_len.to_be_bytes()));
        for addr in path {
            let bytes = hex::decode(addr.strip_prefix("0x").unwrap_or(addr)).unwrap_or_default();
            data.extend_from_slice(&bytes);
        }

        data
    }

    pub fn encode_add_liquidity_v2(
        token_a: &str,
        token_b: &str,
        amount_a_desired: u128,
        amount_b_desired: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        to: &str,
        deadline: u64,
    ) -> Vec<u8> {
        let a = hex::decode(token_a.strip_prefix("0x").unwrap_or(token_a)).unwrap_or_default();
        let b = hex::decode(token_b.strip_prefix("0x").unwrap_or(token_b)).unwrap_or_default();

        let mut data = Vec::with_capacity(4 + 20 + 20 + 32 + 32 + 32 + 32 + 20 + 32);
        data.extend_from_slice(
            &hex::decode("e8e33700").expect("compile-time hex literal for addLiquidityV2 selector"),
        );
        data.extend_from_slice(&a);
        data.extend_from_slice(&b);
        data.extend_from_slice(&u256_padded(&amount_a_desired.to_be_bytes()));
        data.extend_from_slice(&u256_padded(&amount_b_desired.to_be_bytes()));
        data.extend_from_slice(&u256_padded(&amount_a_min.to_be_bytes()));
        data.extend_from_slice(&u256_padded(&amount_b_min.to_be_bytes()));
        data.extend_from_slice(&addr_to_bytes(to));
        data.extend_from_slice(&u256_padded(&deadline.to_be_bytes()));
        data
    }

    pub fn compute_v2_quote(amount_in: u128, reserve_in: u128, reserve_out: u128) -> u128 {
        if reserve_in == 0 || amount_in == 0 {
            return 0;
        }
        let amount_in_with_fee = amount_in * 997;
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * 1000 + amount_in_with_fee;
        numerator / denominator
    }

    pub fn compute_price_impact(
        amount_in: u128,
        reserve_in: u128,
        reserve_out: u128,
        amount_out: u128,
    ) -> f64 {
        if reserve_in == 0 || reserve_out == 0 {
            return 1.0;
        }
        let mid_price = reserve_out as f64 / reserve_in as f64;
        let expected_out = amount_in as f64 * mid_price;
        if expected_out == 0.0 {
            return 0.0;
        }
        ((expected_out - amount_out as f64) / expected_out).max(0.0)
    }

    pub fn get_reserves_v2(
        pair_address: &str,
        client: &EvmClient,
    ) -> Result<(u128, u128, u32), String> {
        let data =
            hex::decode("0902f1ac").expect("compile-time hex literal for getReserves selector");
        let result = client.rpc_call("eth_call", vec![
            serde_json::json!({"to": pair_address, "data": format!("0x{}", hex::encode(&data))}),
            serde_json::json!("latest"),
        ])?;
        let hex_str = result.as_str().unwrap_or("0x");
        let bytes = hex::decode(hex_str.strip_prefix("0x").unwrap_or("0")).unwrap_or_default();
        if bytes.len() < 64 {
            return Err("short reserves data".into());
        }
        let mut r0 = [0u8; 16];
        let mut r1 = [0u8; 16];
        r0.copy_from_slice(&bytes[..16]);
        r1.copy_from_slice(&bytes[32..48]);
        let reserve0 = u128::from_be_bytes(r0);
        let reserve1 = u128::from_be_bytes(r1);
        Ok((reserve0, reserve1, 0))
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
    fn test_dex_registry_defaults() {
        let mut reg = DexRegistry::new();
        reg.register_defaults();
        assert!(reg.router_v2(&ChainType::Ethereum).is_some());
        assert!(reg.router_v3(&ChainType::Ethereum).is_some());
        assert!(reg.router_v2(&ChainType::Bsc).is_some());
        assert!(reg.router_v2(&ChainType::Polygon).is_some());
    }

    #[test]
    fn test_compute_v2_quote() {
        let amount = 1000u128;
        let reserve_in = 100_000u128;
        let reserve_out = 200_000u128;
        let out = DexSwapper::compute_v2_quote(amount, reserve_in, reserve_out);
        assert!(out > 0);
        let expected = (amount * 997 * reserve_out) / (reserve_in * 1000 + amount * 997);
        assert_eq!(out, expected);
    }

    #[test]
    fn test_compute_v2_quote_no_liquidity() {
        let out = DexSwapper::compute_v2_quote(1000, 0, 1000);
        assert_eq!(out, 0);
    }

    #[test]
    fn test_price_impact() {
        let impact = DexSwapper::compute_price_impact(1000, 100000, 200000, 1990);
        assert!(impact >= 0.0 && impact <= 1.0);
    }

    #[test]
    fn test_price_impact_no_liquidity() {
        let impact = DexSwapper::compute_price_impact(1000, 0, 0, 0);
        assert!((impact - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_encode_swap_v2() {
        let path = vec![
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(),
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".into(),
        ];
        let data = DexSwapper::encode_swap_exact_tokens_for_tokens_v2(
            1000000,
            1,
            &path,
            "0x1234567890abcdef1234567890abcdef12345678",
            9999999999,
        );
        assert_eq!(&data[..4], &hex::decode("38ed1739").unwrap());
        assert!(data.len() > 100);
    }

    #[test]
    fn test_encode_add_liquidity_v2() {
        let data = DexSwapper::encode_add_liquidity_v2(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            1000000,
            1000000000000000000,
            900000,
            900000000000000000,
            "0x1234567890abcdef1234567890abcdef12345678",
            9999999999,
        );
        assert_eq!(&data[..4], &hex::decode("e8e33700").unwrap());
    }

    #[test]
    fn test_dex_protocol_names() {
        assert_eq!(DexProtocol::UniswapV2.name(), "Uniswap V2");
        assert_eq!(DexProtocol::UniswapV3.name(), "Uniswap V3");
    }

    #[test]
    fn test_get_reserves_error_on_bad_rpc() {
        let chain = ChainType::Ethereum;
        let config = crate::neotrix::nt_act_crypto::chain::ChainConfig::new(
            chain,
            "https://invalid-rpc.example.com",
        );
        let client = EvmClient::new(&config);
        let result =
            DexSwapper::get_reserves_v2("0x0000000000000000000000000000000000000000", &client);
        assert!(result.is_err());
    }
}
