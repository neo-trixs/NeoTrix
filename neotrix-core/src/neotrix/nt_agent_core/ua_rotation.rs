use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone)]
pub enum UaStrategy {
    RoundRobin,
    Random,
    Weighted(Vec<f64>),
}

#[derive(Debug)]
pub struct UserAgentRotation {
    agents: Vec<&'static str>,
    strategy: UaStrategy,
    counter: AtomicUsize,
    domain_memory: Option<std::collections::HashMap<String, usize>>,
}

impl Default for UserAgentRotation {
    fn default() -> Self {
        Self::new()
    }
}

impl UserAgentRotation {
    pub fn new() -> Self {
        Self {
            agents: Self::default_agents(),
            strategy: UaStrategy::RoundRobin,
            counter: AtomicUsize::new(0),
            domain_memory: None,
        }
    }

    pub fn with_agents(agents: Vec<&'static str>) -> Self {
        Self {
            agents,
            strategy: UaStrategy::RoundRobin,
            counter: AtomicUsize::new(0),
            domain_memory: None,
        }
    }

    pub fn with_strategy(mut self, strategy: UaStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn next(&self) -> &'static str {
        if self.agents.is_empty() {
            return DEFAULT_UA;
        }
        let idx = match self.strategy {
            UaStrategy::RoundRobin => {
                self.counter.fetch_add(1, Ordering::Relaxed) % self.agents.len()
            }
            UaStrategy::Random => {
                let seed = self.counter.fetch_add(1, Ordering::Relaxed);
                (seed.wrapping_mul(6364136223846793005)) % self.agents.len()
            }
            UaStrategy::Weighted(ref _weights) => {
                self.counter.fetch_add(1, Ordering::Relaxed) % self.agents.len()
            }
        };
        self.agents[idx]
    }

    pub fn next_for_domain(&mut self, domain: &str) -> &'static str {
        let ua = self.next();
        self.domain_memory
            .get_or_insert_with(std::collections::HashMap::new)
            .insert(domain.to_string(), self.counter.load(Ordering::Relaxed));
        ua
    }

    pub fn agents(&self) -> &[&'static str] {
        &self.agents
    }

    pub fn add_agent(&mut self, agent: &'static str) {
        self.agents.push(agent);
    }

    fn default_agents() -> Vec<&'static str> {
        vec![
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:126.0) Gecko/20100101 Firefox/126.0",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
            "Mozilla/5.0 (iPhone; CPU iPhone OS 17_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1",
            "Mozilla/5.0 (iPad; CPU OS 17_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1",
            "Mozilla/5.0 (Linux; Android 14; Pixel 8 Pro) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Mobile Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 Edg/125.0.0.0",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 Edg/125.0.0.0",
            "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
            "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Mobile Safari/537.36",
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:126.0) Gecko/20100101 Firefox/126.0",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
            "Mozilla/5.0 (iPhone; CPU iPhone OS 17_5_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:126.0) Gecko/20100101 Firefox/126.0",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15",
            "Mozilla/5.0 (Linux; Android 13; SM-S908B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Mobile Safari/537.36",
        ]
    }
}

const DEFAULT_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_returns_valid_ua() {
        let rotator = UserAgentRotation::new();
        let ua = rotator.next();
        assert!(ua.starts_with("Mozilla/"));
    }

    #[test]
    fn test_round_robin_cycles() {
        let rotator = UserAgentRotation::new();
        let first = rotator.next();
        let second = rotator.next();
        assert_ne!(first, second);
    }

    #[test]
    fn test_with_custom_agents() {
        let rotator = UserAgentRotation::with_agents(vec!["test-agent/1.0"]);
        assert_eq!(rotator.next(), "test-agent/1.0");
    }

    #[test]
    fn test_default_agents_count() {
        assert!(UserAgentRotation::default_agents().len() >= 10);
    }

    #[test]
    fn test_next_for_domain_does_not_panic() {
        let mut rotator = UserAgentRotation::new();
        let ua = rotator.next_for_domain("example.com");
        assert!(ua.starts_with("Mozilla/"));
    }
}
