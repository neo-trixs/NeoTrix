use crate::types::*;
use chrono::Utc;

const SIMULATED_CHAINS: &[&str] = &[
    "ethereum",
    "bsc",
    "polygon",
    "arbitrum",
    "optimism",
    "base",
    "avalanche",
    "fantom",
    "solana",
];

const SIMULATED_TOKENS: &[(&str, f64)] = &[
    ("ETH", 3512.40),
    ("BTC", 98745.00),
    ("SOL", 168.32),
    ("BNB", 612.80),
    ("MATIC", 0.54),
    ("ARB", 0.89),
    ("OP", 1.76),
    ("AVAX", 36.21),
];

const SIMULATED_GAS_GWEI: &[(&str, f64)] = &[
    ("ethereum", 12.8),
    ("bsc", 3.2),
    ("polygon", 42.5),
    ("arbitrum", 0.08),
    ("optimism", 0.006),
    ("base", 0.12),
    ("avalanche", 25.4),
    ("fantom", 72.1),
    ("solana", 0.0003),
];

#[derive(Debug)]
pub struct CryptoBridge {
    pub vsa: VsaLight,
    pub wallet_configured: bool,
    pub known_chains: Vec<String>,
    pub last_balance_check_ms: i64,
    pub total_actuations: u64,
    pub error_count: u64,
}

impl CryptoBridge {
    pub fn new() -> Self {
        Self {
            vsa: VsaLight::new(VSA_DIM),
            wallet_configured: false,
            known_chains: SIMULATED_CHAINS.iter().map(|s| s.to_string()).collect(),
            last_balance_check_ms: 0,
            total_actuations: 0,
            error_count: 0,
        }
    }

    pub fn configure_wallet(&mut self) {
        self.wallet_configured = true;
    }

    fn simulate_balance_usd(&self) -> f64 {
        if !self.wallet_configured {
            return 0.0;
        }
        let base: f64 = 1250.0;
        let jitter: f64 = (Utc::now().timestamp() % 100) as f64 * 0.31;
        base + jitter
    }

    fn simulate_portfolio_diversity(&self) -> usize {
        if self.wallet_configured {
            4
        } else {
            0
        }
    }

    fn simulate_market_volatility(&self) -> f64 {
        let seed = Utc::now().timestamp_subsec_millis() as f64;
        0.15 + (seed * 0.007).sin().abs() * 0.25
    }

    fn simulate_price_ticks(&self) -> Vec<(String, f64, f64)> {
        let now = Utc::now();
        let ms = now.timestamp_subsec_millis() as f64;
        SIMULATED_TOKENS
            .iter()
            .map(|(sym, base)| {
                let drift = (ms * 0.013).sin() * base * 0.02;
                let price = base + drift;
                let change_pct = drift / base;
                ((*sym).to_string(), price, change_pct)
            })
            .collect()
    }

    fn simulate_gas_prices(&self) -> Vec<(String, f64)> {
        let ms = Utc::now().timestamp_subsec_millis() as f64;
        SIMULATED_GAS_GWEI
            .iter()
            .map(|(chain, base)| {
                let drift = (ms * 0.021 + chain.len() as f64).sin() * base * 0.15;
                ((*chain).to_string(), base + drift)
            })
            .collect()
    }
}

impl Default for CryptoBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsciousnessAbility for CryptoBridge {
    fn domain(&self) -> Domain {
        Domain::Crypto
    }

    fn sense(&mut self) -> Vec<VsaTagged> {
        let now = Utc::now();
        let now_ms = now.timestamp_millis();
        self.last_balance_check_ms = now_ms;
        let volatility = self.simulate_market_volatility();
        let balance = self.simulate_balance_usd();
        let mut signals = Vec::new();

        let balance_seed = (now_ms as u64).wrapping_mul(0x9e3779b97f4a7c15);
        signals.push(VsaTagged {
            vector: self.vsa.seeded_vector(balance_seed),
            origin: VsaOrigin::Bridge(Domain::Crypto),
            timestamp_ms: now_ms,
            negentropy_contribution: volatility * 0.3,
        });

        for (i, (_token, price, _)) in self.simulate_price_ticks().iter().enumerate() {
            let price_seed = (now_ms as u64)
                .wrapping_mul(0x9e3779b97f4a7c16)
                .wrapping_add(i as u64);
            let token_vol = (price * 0.001).min(1.0);
            signals.push(VsaTagged {
                vector: self.vsa.seeded_vector(price_seed),
                origin: VsaOrigin::World(Sensory::PriceTick),
                timestamp_ms: now_ms,
                negentropy_contribution: token_vol * volatility,
            });
        }

        for (i, (_chain, gwei)) in self.simulate_gas_prices().iter().enumerate() {
            let gas_seed = (now_ms as u64)
                .wrapping_mul(0x9e3779b97f4a7c17)
                .wrapping_add(i as u64);
            let gas_vol = (gwei * 0.02).min(1.0);
            signals.push(VsaTagged {
                vector: self.vsa.seeded_vector(gas_seed),
                origin: VsaOrigin::World(Sensory::PriceTick),
                timestamp_ms: now_ms,
                negentropy_contribution: gas_vol * volatility * 0.5,
            });
        }

        let wallet_signal_seed = (now_ms as u64).wrapping_mul(0x9e3779b97f4a7c18);
        signals.push(VsaTagged {
            vector: self.vsa.seeded_vector(wallet_signal_seed),
            origin: VsaOrigin::Bridge(Domain::Crypto),
            timestamp_ms: now_ms,
            negentropy_contribution: if balance > 0.0 {
                volatility * (balance / 10000.0).min(1.0) * 0.4
            } else {
                0.0
            },
        });

        signals
    }

    fn actuate(&mut self, intention: &IntentionVsa) -> Result<WorldEffect, String> {
        let start = Utc::now();
        self.total_actuations += 1;

        if !self.wallet_configured {
            let now = Utc::now();
            self.error_count += 1;
            return Ok(WorldEffect {
                domain: Domain::Crypto,
                description: "no wallet configured — call configure_wallet() first".into(),
                success: false,
                latency_ms: Utc::now().signed_duration_since(now).num_milliseconds().max(1) as u64,
            });
        }

        let result = match intention.action.as_str() {
            "transfer" => {
                let to = intention
                    .parameters
                    .get("to")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let amount = intention
                    .parameters
                    .get("amount")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                if amount <= 0.0 {
                    self.error_count += 1;
                    Err("transfer amount must be positive".into())
                } else if to.len() < 10 {
                    self.error_count += 1;
                    Err("invalid recipient address".into())
                } else {
                    Ok(format!("transferred {} to {}", amount, to))
                }
            }
            "swap" => {
                let from = intention
                    .parameters
                    .get("from_token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let to = intention
                    .parameters
                    .get("to_token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let amount = intention
                    .parameters
                    .get("amount")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                if amount <= 0.0 {
                    self.error_count += 1;
                    Err("swap amount must be positive".into())
                } else if from == to {
                    self.error_count += 1;
                    Err("cannot swap token to itself".into())
                } else {
                    Ok(format!("swapped {} {} → {}", amount, from, to))
                }
            }
            "bridge" => {
                let chain = intention
                    .parameters
                    .get("target_chain")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let amount = intention
                    .parameters
                    .get("amount")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                if !self.known_chains.iter().any(|c| c == chain) {
                    self.error_count += 1;
                    Err(format!("unsupported target chain: {}", chain))
                } else if amount <= 0.0 {
                    self.error_count += 1;
                    Err("bridge amount must be positive".into())
                } else {
                    Ok(format!("bridged {} to {}", amount, chain))
                }
            }
            "check_balance" => {
                let bal = self.simulate_balance_usd();
                Ok(format!("wallet balance: ${:.2}", bal))
            }
            _ => {
                self.error_count += 1;
                Err(format!("unknown crypto action: {}", intention.action))
            }
        };

        let latency = Utc::now()
            .signed_duration_since(start)
            .num_milliseconds()
            .max(0) as u64;

        match result {
            Ok(desc) => Ok(WorldEffect {
                domain: Domain::Crypto,
                description: desc,
                success: true,
                latency_ms: latency.max(1),
            }),
            Err(e) => Ok(WorldEffect {
                domain: Domain::Crypto,
                description: e.clone(),
                success: false,
                latency_ms: latency.max(1),
            }),
        }
    }

    fn curiosity_signals(&self) -> Vec<CuriositySignal> {
        let vol = self.simulate_market_volatility();
        let _now_ms = Utc::now().timestamp_millis() as u64;

        vec![
            CuriositySignal {
                domain: Domain::Crypto,
                query: "explore unlisted L2 chains — Linea, Scroll, ZkSync Eco".into(),
                novelty_estimate: 0.75,
                potential_negentropy: 0.6 * vol,
            },
            CuriositySignal {
                domain: Domain::Crypto,
                query: "monitor new DEX pools on Base for early liquidity opportunities".into(),
                novelty_estimate: 0.65,
                potential_negentropy: 0.55 * vol,
            },
            CuriositySignal {
                domain: Domain::Crypto,
                query: "track whale movements across Ethereum and Solana mempools".into(),
                novelty_estimate: 0.60,
                potential_negentropy: 0.50 * vol,
            },
            CuriositySignal {
                domain: Domain::Crypto,
                query: "identify arbitrage routes across CEX-DEX price gaps".into(),
                novelty_estimate: 0.70,
                potential_negentropy: 0.65 * vol,
            },
            CuriositySignal {
                domain: Domain::Crypto,
                query: "scan airdrop eligibility for new protocols on Arbitrum and Optimism".into(),
                novelty_estimate: 0.80,
                potential_negentropy: 0.70 * vol,
            },
            CuriositySignal {
                domain: Domain::Crypto,
                query: "evaluate yield farming strategies with restaking tokens (LRT)".into(),
                novelty_estimate: 0.55,
                potential_negentropy: 0.45 * vol,
            },
        ]
    }

    fn grace_mode(&self) -> GraceMode {
        if self.wallet_configured {
            GraceMode::SkipSilently
        } else {
            GraceMode::FallbackDefault
        }
    }

    fn health(&self) -> BridgeHealth {
        BridgeHealth {
            domain: Domain::Crypto,
            available: self.wallet_configured,
            last_seen_ms: self.last_balance_check_ms,
            error_count: self.error_count,
            total_actuations: self.total_actuations,
        }
    }

    fn probe_available(&self) -> bool {
        if !self.wallet_configured {
            return false;
        }
        let wallet_dir = std::path::PathBuf::from(
            format!(
                "{}/.neotrix/wallets",
                std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
            ),
        );
        wallet_dir.exists()
            && std::fs::read_dir(&wallet_dir)
                .ok()
                .map(|entries| {
                    entries.filter_map(|e| e.ok()).any(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "json")
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
    }

    fn negentropy_estimate(&self) -> f64 {
        let vol = self.simulate_market_volatility();
        let diversity = self.simulate_portfolio_diversity();
        let balance = self.simulate_balance_usd();

        let vol_component = vol * 0.4;
        let diversity_component = (diversity as f64 / 10.0) * 0.3;
        let balance_component = (balance / 10000.0).min(1.0) * 0.3;

        vol_component + diversity_component + balance_component
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn bridge_with_wallet() -> CryptoBridge {
        let mut b = CryptoBridge::new();
        b.configure_wallet();
        b
    }

    #[test]
    fn test_domain() {
        let b = CryptoBridge::new();
        assert_eq!(b.domain(), Domain::Crypto);
    }

    #[test]
    fn test_sense_returns_signals() {
        let mut b = bridge_with_wallet();
        let signals = b.sense();
        assert!(!signals.is_empty());
        assert!(signals.len() >= 10);

        let price_ticks: Vec<_> = signals
            .iter()
            .filter(|s| matches!(s.origin, VsaOrigin::World(Sensory::PriceTick)))
            .collect();
        assert!(!price_ticks.is_empty());
        assert!(price_ticks[0].vector.len() == VSA_DIM);
        assert!(price_ticks[0].timestamp_ms > 0);
    }

    #[test]
    fn test_sense_no_wallet_still_returns_market_data() {
        let mut b = CryptoBridge::new();
        let signals = b.sense();
        assert!(!signals.is_empty());
        let bridge_signals: Vec<_> = signals
            .iter()
            .filter(|s| matches!(s.origin, VsaOrigin::Bridge(Domain::Crypto)))
            .collect();
        assert!(!bridge_signals.is_empty());
    }

    #[test]
    fn test_actuate_transfer_success() {
        let mut b = bridge_with_wallet();
        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "transfer".into(),
            parameters: json!({"to": "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18", "amount": 0.5}),
            confidence: 0.9,
            urgency: 0.5,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(effect.success);
        assert_eq!(effect.domain, Domain::Crypto);
        assert!(b.total_actuations == 1);
    }

    #[test]
    fn test_actuate_transfer_fails_no_wallet() {
        let mut b = CryptoBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "transfer".into(),
            parameters: json!({"to": "0xabc", "amount": 1.0}),
            confidence: 0.9,
            urgency: 0.5,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(!effect.success);
        assert!(b.error_count == 1);
    }

    #[test]
    fn test_actuate_swap_success() {
        let mut b = bridge_with_wallet();
        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "swap".into(),
            parameters: json!({"from_token": "ETH", "to_token": "USDC", "amount": 0.1}),
            confidence: 0.85,
            urgency: 0.6,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(effect.success);
    }

    #[test]
    fn test_actuate_swap_self_fails() {
        let mut b = bridge_with_wallet();
        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "swap".into(),
            parameters: json!({"from_token": "ETH", "to_token": "ETH", "amount": 1.0}),
            confidence: 0.8,
            urgency: 0.3,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(!effect.success);
        assert!(b.error_count == 1);
    }

    #[test]
    fn test_actuate_swap_zero_amount_fails() {
        let mut b = bridge_with_wallet();
        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "swap".into(),
            parameters: json!({"from_token": "ETH", "to_token": "USDC", "amount": 0.0}),
            confidence: 0.9,
            urgency: 0.5,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(!effect.success);
    }

    #[test]
    fn test_actuate_bridge_success() {
        let mut b = bridge_with_wallet();
        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "bridge".into(),
            parameters: json!({"target_chain": "arbitrum", "amount": 0.5}),
            confidence: 0.75,
            urgency: 0.4,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(effect.success);
    }

    #[test]
    fn test_actuate_bridge_unknown_chain_fails() {
        let mut b = bridge_with_wallet();
        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "bridge".into(),
            parameters: json!({"target_chain": "nonexistent", "amount": 1.0}),
            confidence: 0.7,
            urgency: 0.5,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(!effect.success);
    }

    #[test]
    fn test_actuate_check_balance() {
        let mut b = bridge_with_wallet();
        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "check_balance".into(),
            parameters: json!({}),
            confidence: 1.0,
            urgency: 0.2,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(effect.success);
        assert!(effect.description.contains("balance"));
    }

    #[test]
    fn test_actuate_unknown_action() {
        let mut b = bridge_with_wallet();
        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "mine_bitcoin".into(),
            parameters: json!({}),
            confidence: 0.1,
            urgency: 0.0,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(!effect.success);
        assert!(b.error_count == 1);
    }

    #[test]
    fn test_curiosity_signals_domain() {
        let b = bridge_with_wallet();
        let signals = b.curiosity_signals();
        assert!(!signals.is_empty());
        for s in &signals {
            assert_eq!(s.domain, Domain::Crypto);
            assert!(s.novelty_estimate > 0.0);
            assert!(s.potential_negentropy > 0.0);
        }
    }

    #[test]
    fn test_curiosity_signals_scaled_by_volatility() {
        let b1 = CryptoBridge::new();
        let s1 = b1.curiosity_signals();
        let vol_high = 0.4;
        for s in &s1 {
            assert!(s.potential_negentropy <= vol_high);
        }
    }

    #[test]
    fn test_grace_mode_no_wallet() {
        let b = CryptoBridge::new();
        assert_eq!(b.grace_mode(), GraceMode::FallbackDefault);
    }

    #[test]
    fn test_grace_mode_with_wallet() {
        let mut b = CryptoBridge::new();
        b.configure_wallet();
        assert_eq!(b.grace_mode(), GraceMode::SkipSilently);
    }

    #[test]
    fn test_health_reflects_state() {
        let mut b = CryptoBridge::new();
        let h = b.health();
        assert_eq!(h.domain, Domain::Crypto);
        assert!(!h.available);
        assert_eq!(h.error_count, 0);
        assert_eq!(h.total_actuations, 0);

        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "transfer".into(),
            parameters: json!({"to": "0xabc", "amount": 1.0}),
            confidence: 0.9,
            urgency: 0.5,
        };
        let _ = b.actuate(&intention);
        let h = b.health();
        assert_eq!(h.error_count, 1);
        assert_eq!(h.total_actuations, 1);
    }

    #[test]
    fn test_probe_available_no_wallet_dir() {
        let b = CryptoBridge::new();
        assert!(!b.probe_available());
    }

    #[test]
    fn test_negentropy_estimate_range() {
        let b = bridge_with_wallet();
        let ne = b.negentropy_estimate();
        assert!(ne >= 0.0);
        assert!(ne <= 1.0);
    }

    #[test]
    fn test_negentropy_estimate_zero_when_no_wallet() {
        let b = CryptoBridge::new();
        let ne = b.negentropy_estimate();
        assert!(ne >= 0.0);
    }

    #[test]
    fn test_seeded_vectors_differ() {
        let vsa = VsaLight::new(VSA_DIM);
        let a = vsa.seeded_vector(42);
        let b = vsa.seeded_vector(99);
        assert_ne!(a, b);
    }

    #[test]
    fn test_case_insensitive_known_chains() {
        let b = CryptoBridge::new();
        assert!(b.known_chains.iter().any(|c| c == "ethereum"));
        assert!(b.known_chains.iter().any(|c| c == "solana"));
    }

    #[test]
    fn test_total_actuations_increments_on_failure() {
        let mut b = bridge_with_wallet();
        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "transfer".into(),
            parameters: json!({"to": "0xabc", "amount": -1.0}),
            confidence: 0.9,
            urgency: 0.5,
        };
        let _ = b.actuate(&intention);
        assert_eq!(b.total_actuations, 1);
        assert_eq!(b.error_count, 1);
    }

    #[test]
    fn test_configure_wallet_idempotent() {
        let mut b = CryptoBridge::new();
        assert!(!b.wallet_configured);
        b.configure_wallet();
        assert!(b.wallet_configured);
        b.configure_wallet();
        assert!(b.wallet_configured);
    }

    #[test]
    fn test_actuate_latency_nonzero() {
        let mut b = bridge_with_wallet();
        let intention = IntentionVsa {
            domain: Domain::Crypto,
            action: "check_balance".into(),
            parameters: json!({}),
            confidence: 1.0,
            urgency: 0.0,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(effect.latency_ms >= 1, "latency should be at least 1ms");
    }
}
