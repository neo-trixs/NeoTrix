pub mod crypto;
pub mod earn;
pub mod network;
pub mod crawl;
pub mod social;
pub mod browse;
pub mod vision;

pub use crypto::CryptoBridge;
pub use earn::EarnBridge;
pub use network::NetworkBridge;
pub use crawl::CrawlBridge;
pub use social::SocialBridge;
pub use browse::BrowseBridge;
pub use vision::VisionBridge;

use crate::types::*;

/// Facade over all 7 ConsciousnessAbility bridges.
/// Provides unified sense_all() / curiosity_all() / health_all()
/// for the BackgroundLoop to call in a single tick.
pub struct BridgeRegistry {
    pub crypto: CryptoBridge,
    pub earn: EarnBridge,
    pub network: NetworkBridge,
    pub crawl: CrawlBridge,
    pub social: SocialBridge,
    pub browse: BrowseBridge,
    pub vision: VisionBridge,
}

impl BridgeRegistry {
    pub fn new() -> Self {
        Self {
            crypto: CryptoBridge::new(),
            earn: EarnBridge::new(),
            network: NetworkBridge::new(),
            crawl: CrawlBridge::new(),
            social: SocialBridge::new(),
            browse: BrowseBridge::new(false),
            vision: VisionBridge::new(),
        }
    }

    /// All bridges emit their current VSA-tagged perception events.
    /// Filters near-zero negentropy events to reduce noise.
    pub fn sense_all(&mut self) -> Vec<VsaTagged> {
        let mut all = Vec::new();
        all.extend(self.crypto.sense());
        all.extend(self.earn.sense());
        all.extend(self.network.sense());
        all.extend(self.crawl.sense());
        all.extend(self.social.sense());
        all.extend(self.browse.sense());
        all.extend(self.vision.sense());
        all.retain(|e| e.negentropy_contribution > 0.01);
        all
    }

    /// Collect curiosity signals from all bridges, sorted by novelty descending.
    pub fn curiosity_all(&self) -> Vec<CuriositySignal> {
        let mut all = Vec::new();
        all.extend(self.crypto.curiosity_signals());
        all.extend(self.earn.curiosity_signals());
        all.extend(self.network.curiosity_signals());
        all.extend(self.crawl.curiosity_signals());
        all.extend(self.social.curiosity_signals());
        all.extend(self.browse.curiosity_signals());
        all.extend(self.vision.curiosity_signals());
        all.sort_by(|a, b| {
            b.novelty_estimate
                .partial_cmp(&a.novelty_estimate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all.truncate(20);
        all
    }

    /// Health snapshot of every bridge.
    pub fn health_all(&self) -> Vec<BridgeHealth> {
        vec![
            self.crypto.health(),
            self.earn.health(),
            self.network.health(),
            self.crawl.health(),
            self.social.health(),
            self.browse.health(),
            self.vision.health(),
        ]
    }
}

impl Default for BridgeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_registry_new() {
        let r = BridgeRegistry::new();
        assert_eq!(r.crypto.domain(), Domain::Crypto);
        assert_eq!(r.earn.domain(), Domain::Earn);
        assert_eq!(r.network.domain(), Domain::Network);
        assert_eq!(r.crawl.domain(), Domain::Crawl);
        assert_eq!(r.social.domain(), Domain::Social);
        assert_eq!(r.browse.domain(), Domain::Browse);
        assert_eq!(r.vision.domain(), Domain::Vision);
    }

    #[test]
    fn test_sense_all_returns_tagged_events() {
        let mut r = BridgeRegistry::new();
        let events = r.sense_all();
        for e in &events {
            assert_eq!(e.vector.len(), VSA_DIM);
            assert!(e.negentropy_contribution > 0.01);
        }
    }

    #[test]
    fn test_curiosity_all_sorted() {
        let r = BridgeRegistry::new();
        let signals = r.curiosity_all();
        assert!(!signals.is_empty());
        assert!(signals.len() <= 20);
        for w in signals.windows(2) {
            assert!(w[0].novelty_estimate >= w[1].novelty_estimate);
        }
    }

    #[test]
    fn test_health_all_covers_seven() {
        let r = BridgeRegistry::new();
        let health = r.health_all();
        assert_eq!(health.len(), 7);
        let domains: Vec<Domain> = health.iter().map(|h| h.domain).collect();
        assert!(domains.contains(&Domain::Crypto));
        assert!(domains.contains(&Domain::Earn));
        assert!(domains.contains(&Domain::Network));
        assert!(domains.contains(&Domain::Crawl));
        assert!(domains.contains(&Domain::Social));
        assert!(domains.contains(&Domain::Browse));
        assert!(domains.contains(&Domain::Vision));
    }
}
