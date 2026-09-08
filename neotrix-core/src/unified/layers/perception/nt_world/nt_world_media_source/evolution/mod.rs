pub mod auto_discovery;
pub mod adaptive_quality;
pub mod smart_warmup;
pub mod self_healing;
pub mod skillglow;
pub mod tests;
pub mod ecosystem;
pub mod arch;

pub use auto_discovery::AutoSourceDiscovery;
pub use adaptive_quality::AdaptiveQualityManager;
pub use smart_warmup::SmartCacheWarmer;
pub use self_healing::SelfHealingManager;
pub use skillglow::{LocalSkill, ProceduralFamily, CommitGate, SkillConsolidator};
