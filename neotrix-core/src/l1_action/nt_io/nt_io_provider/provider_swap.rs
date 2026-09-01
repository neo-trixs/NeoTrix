use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::factory::LlmProviderType;
use super::types::LlmError;

#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub provider: LlmProviderType,
    pub last_check: Instant,
    pub consecutive_failures: u32,
    pub total_requests: u64,
    pub total_errors: u64,
    pub avg_latency_ms: f64,
    pub is_healthy: bool,
    pub last_error: Option<String>,
}

impl ProviderHealth {
    pub fn new(provider: LlmProviderType) -> Self {
        Self {
            provider,
            last_check: Instant::now(),
            consecutive_failures: 0,
            total_requests: 0,
            total_errors: 0,
            avg_latency_ms: 0.0,
            is_healthy: true,
            last_error: None,
        }
    }

    pub fn record_success(&mut self, latency_ms: f64) {
        self.total_requests += 1;
        self.consecutive_failures = 0;
        self.avg_latency_ms = self.avg_latency_ms * 0.9 + latency_ms * 0.1;
        self.is_healthy = true;
    }

    pub fn record_error(&mut self, error: &LlmError) {
        self.total_requests += 1;
        self.total_errors += 1;
        self.consecutive_failures += 1;
        self.last_error = Some(error.to_string());
        if self.consecutive_failures >= 3 {
            self.is_healthy = false;
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 1.0;
        }
        1.0 - (self.total_errors as f64 / self.total_requests as f64)
    }

    pub fn is_circuit_broken(&self) -> bool {
        !self.is_healthy && self.consecutive_failures >= 5
    }
}

#[derive(Debug, Clone)]
pub struct SwapRule {
    pub trigger_consecutive_failures: u32,
    pub trigger_success_rate_below: f64,
    pub prefer_same_category: bool,
    pub cooldown_secs: u64,
}

impl Default for SwapRule {
    fn default() -> Self {
        Self {
            trigger_consecutive_failures: 3,
            trigger_success_rate_below: 0.5,
            prefer_same_category: true,
            cooldown_secs: 300,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapRecord {
    pub from: LlmProviderType,
    pub to: LlmProviderType,
    pub reason: String,
    pub timestamp: Instant,
    pub success: bool,
}

pub struct ProviderSwapManager {
    pub health_map: HashMap<LlmProviderType, ProviderHealth>,
    pub swap_history: Vec<SwapRecord>,
    pub rules: SwapRule,
    pub cooldowns: HashMap<LlmProviderType, Instant>,
    pub fallback_chain: Vec<LlmProviderType>,
}

impl ProviderSwapManager {
    pub fn new(fallback_chain: Vec<LlmProviderType>) -> Self {
        let mut health_map = HashMap::new();
        for p in &fallback_chain {
            health_map.insert(*p, ProviderHealth::new(*p));
        }
        Self {
            health_map,
            swap_history: Vec::new(),
            rules: SwapRule::default(),
            cooldowns: HashMap::new(),
            fallback_chain,
        }
    }

    pub fn record_success(&mut self, provider: LlmProviderType, latency_ms: f64) {
        if let Some(health) = self.health_map.get_mut(&provider) {
            health.record_success(latency_ms);
        }
    }

    pub fn record_error(&mut self, provider: LlmProviderType, error: &LlmError) {
        if let Some(health) = self.health_map.get_mut(&provider) {
            health.record_error(error);
        }
    }

    pub fn should_swap(&self, provider: LlmProviderType) -> bool {
        if let Some(cooldown_end) = self.cooldowns.get(&provider) {
            if Instant::now() < *cooldown_end {
                return false;
            }
        }
        if let Some(health) = self.health_map.get(&provider) {
            if health.consecutive_failures >= self.rules.trigger_consecutive_failures {
                return true;
            }
            if health.total_requests >= 10 && health.success_rate() < self.rules.trigger_success_rate_below {
                return true;
            }
        }
        false
    }

    pub fn find_best_alternative(&self, failed: LlmProviderType) -> Option<LlmProviderType> {
        for candidate in &self.fallback_chain {
            if *candidate == failed {
                continue;
            }
            if let Some(health) = self.health_map.get(candidate) {
                if health.is_healthy && !health.is_circuit_broken() {
                    return Some(*candidate);
                }
            } else {
                return Some(*candidate);
            }
        }
        None
    }

    pub fn perform_swap(&mut self, from: LlmProviderType) -> Option<LlmProviderType> {
        if !self.should_swap(from) {
            return None;
        }

        let to = self.find_best_alternative(from)?;
        let reason = if let Some(health) = self.health_map.get(&from) {
            format!(
                "swap: {} (failures={}, rate={:.1}%) -> {}",
                from_name(from),
                health.consecutive_failures,
                health.success_rate() * 100.0,
                from_name(to),
            )
        } else {
            format!("swap: {} -> {} (unknown health)", from_name(from), from_name(to))
        };

        self.cooldowns.insert(from, Instant::now() + Duration::from_secs(self.rules.cooldown_secs));
        self.swap_history.push(SwapRecord {
            from,
            to,
            reason: reason.clone(),
            timestamp: Instant::now(),
            success: true,
        });
        log::info!("[provider_swap] {}", reason);
        Some(to)
    }

    pub fn health_report(&self) -> Vec<ProviderHealthSummary> {
        let mut report: Vec<ProviderHealthSummary> = self.health_map
            .values()
            .map(|h| ProviderHealthSummary {
                provider: from_name(h.provider).to_string(),
                healthy: h.is_healthy,
                success_rate: h.success_rate(),
                consecutive_failures: h.consecutive_failures,
                total_requests: h.total_requests,
                avg_latency_ms: h.avg_latency_ms,
            })
            .collect();
        report.sort_by(|a, b| a.provider.cmp(&b.provider));
        report
    }

    pub fn recent_swaps(&self, n: usize) -> &[SwapRecord] {
        let start = self.swap_history.len().saturating_sub(n);
        &self.swap_history[start..]
    }
}

#[derive(Debug, Clone)]
pub struct ProviderHealthSummary {
    pub provider: String,
    pub healthy: bool,
    pub success_rate: f64,
    pub consecutive_failures: u32,
    pub total_requests: u64,
    pub avg_latency_ms: f64,
}

pub static GLOBAL_SWAP_MANAGER: std::sync::LazyLock<Mutex<ProviderSwapManager>> =
    std::sync::LazyLock::new(|| {
        let fallback_chain = vec![
            LlmProviderType::OpenAI,
            LlmProviderType::Anthropic,
            LlmProviderType::Gemini,
            LlmProviderType::Groq,
            LlmProviderType::OpenRouter,
            LlmProviderType::Ollama,
        ];
        Mutex::new(ProviderSwapManager::new(fallback_chain))
    });

fn from_name(t: LlmProviderType) -> &'static str {
    match t {
        LlmProviderType::OpenAI => "OpenAI",
        LlmProviderType::Anthropic => "Anthropic",
        LlmProviderType::Gemini => "Gemini",
        LlmProviderType::Ollama => "Ollama",
        LlmProviderType::Groq => "Groq",
        LlmProviderType::OpenRouter => "OpenRouter",
        LlmProviderType::Cerebras => "Cerebras",
        LlmProviderType::SambaNova => "SambaNova",
        LlmProviderType::Pollinations => "Pollinations",
        LlmProviderType::BazaarLink => "BazaarLink",
        LlmProviderType::FreeTheAi => "FreeTheAi",
        LlmProviderType::ZeroLimit => "ZeroLimit",
        LlmProviderType::FreeApi => "FreeApi",
        LlmProviderType::CustomProxy => "CustomProxy",
        _ => "Other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_health_new_is_healthy() {
        let health = ProviderHealth::new(LlmProviderType::OpenAI);
        assert!(health.is_healthy);
        assert_eq!(health.consecutive_failures, 0);
        assert_eq!(health.total_requests, 0);
    }

    #[test]
    fn test_provider_health_record_success() {
        let mut health = ProviderHealth::new(LlmProviderType::OpenAI);
        health.record_success(100.0);
        assert_eq!(health.total_requests, 1);
        assert_eq!(health.consecutive_failures, 0);
        assert!(health.is_healthy);
    }

    #[test]
    fn test_provider_health_record_error() {
        let mut health = ProviderHealth::new(LlmProviderType::OpenAI);
        let error = LlmError::RateLimit("test".to_string());
        health.record_error(&error);
        assert_eq!(health.total_requests, 1);
        assert_eq!(health.consecutive_failures, 1);
        assert!(health.is_healthy);
    }

    #[test]
    fn test_provider_health_becomes_unhealthy_after_3_failures() {
        let mut health = ProviderHealth::new(LlmProviderType::OpenAI);
        let error = LlmError::RateLimit("test".to_string());
        for _ in 0..3 {
            health.record_error(&error);
        }
        assert!(!health.is_healthy);
    }

    #[test]
    fn test_success_rate() {
        let mut health = ProviderHealth::new(LlmProviderType::OpenAI);
        let error = LlmError::RateLimit("test".to_string());
        health.record_success(50.0);
        health.record_error(&error);
        health.record_success(60.0);
        assert!((health.success_rate() - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_swap_manager_initialization() {
        let chain = vec![LlmProviderType::OpenAI, LlmProviderType::Anthropic];
        let manager = ProviderSwapManager::new(chain);
        assert_eq!(manager.health_map.len(), 2);
        assert_eq!(manager.swap_history.len(), 0);
    }

    #[test]
    fn test_should_swap_after_failures() {
        let chain = vec![LlmProviderType::OpenAI, LlmProviderType::Anthropic];
        let mut manager = ProviderSwapManager::new(chain);
        let error = LlmError::RateLimit("test".to_string());
        for _ in 0..3 {
            manager.record_error(LlmProviderType::OpenAI, &error);
        }
        assert!(manager.should_swap(LlmProviderType::OpenAI));
    }

    #[test]
    fn test_should_not_swap_when_healthy() {
        let chain = vec![LlmProviderType::OpenAI];
        let manager = ProviderSwapManager::new(chain);
        assert!(!manager.should_swap(LlmProviderType::OpenAI));
    }

    #[test]
    fn test_find_best_alternative() {
        let chain = vec![LlmProviderType::OpenAI, LlmProviderType::Anthropic, LlmProviderType::Gemini];
        let manager = ProviderSwapManager::new(chain);
        let alt = manager.find_best_alternative(LlmProviderType::OpenAI);
        assert!(alt.is_some());
        assert_eq!(alt.unwrap(), LlmProviderType::Anthropic);
    }

    #[test]
    fn test_perform_swap() {
        let chain = vec![LlmProviderType::OpenAI, LlmProviderType::Anthropic];
        let mut manager = ProviderSwapManager::new(chain);
        let error = LlmError::RateLimit("test".to_string());
        for _ in 0..3 {
            manager.record_error(LlmProviderType::OpenAI, &error);
        }
        let swapped = manager.perform_swap(LlmProviderType::OpenAI);
        assert!(swapped.is_some());
        assert_eq!(swapped.unwrap(), LlmProviderType::Anthropic);
        assert_eq!(manager.swap_history.len(), 1);
    }

    #[test]
    fn test_perform_swap_no_alternative() {
        let chain = vec![LlmProviderType::OpenAI];
        let mut manager = ProviderSwapManager::new(chain);
        let error = LlmError::RateLimit("test".to_string());
        for _ in 0..3 {
            manager.record_error(LlmProviderType::OpenAI, &error);
        }
        let swapped = manager.perform_swap(LlmProviderType::OpenAI);
        assert!(swapped.is_none());
    }

    #[test]
    fn test_swap_cooldown() {
        let chain = vec![LlmProviderType::OpenAI, LlmProviderType::Anthropic];
        let mut manager = ProviderSwapManager::new(chain);
        let error = LlmError::RateLimit("test".to_string());
        for _ in 0..3 {
            manager.record_error(LlmProviderType::OpenAI, &error);
        }
        manager.cooldowns.insert(LlmProviderType::OpenAI, Instant::now() + Duration::from_secs(9999));
        assert!(!manager.should_swap(LlmProviderType::OpenAI));
    }

    #[test]
    fn test_health_report() {
        let chain = vec![LlmProviderType::OpenAI, LlmProviderType::Anthropic];
        let mut manager = ProviderSwapManager::new(chain);
        manager.record_success(LlmProviderType::OpenAI, 50.0);
        manager.record_success(LlmProviderType::Anthropic, 100.0);
        let report = manager.health_report();
        assert_eq!(report.len(), 2);
    }

    #[test]
    fn test_global_swap_manager() {
        let manager = GLOBAL_SWAP_MANAGER.lock().unwrap();
        assert_eq!(manager.fallback_chain.len(), 6);
    }
}
