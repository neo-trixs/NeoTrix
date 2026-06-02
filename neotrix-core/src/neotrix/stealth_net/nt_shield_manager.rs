use rand::Rng;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorProfile {
    Human,
    Automated,
    Mixed,
}

impl BehaviorProfile {
    pub fn typos_per_100_chars(&self) -> f64 {
        match self {
            BehaviorProfile::Human => 2.5,
            BehaviorProfile::Automated => 0.02,
            BehaviorProfile::Mixed => 0.8,
        }
    }

    pub fn action_delay_ms(&self) -> (u64, u64) {
        match self {
            BehaviorProfile::Human => (80, 350),
            BehaviorProfile::Automated => (5, 20),
            BehaviorProfile::Mixed => (30, 120),
        }
    }

    pub fn scroll_pattern(&self) -> &str {
        match self {
            BehaviorProfile::Human => "variable",
            BehaviorProfile::Automated => "linear",
            BehaviorProfile::Mixed => "semi_variable",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub id: usize,
    pub name: String,
    pub created_at: SystemTime,
    pub last_used: SystemTime,
    pub use_count: usize,
    pub success_rate: f64,
    pub confidence: f64,
    pub behavior: BehaviorProfile,
    pub user_agent: String,
    pub screen_resolution: (u32, u32),
    pub language: String,
    pub canvas_fingerprint: String,
    pub audio_fingerprint: String,
    pub age_seconds: f64,
    pub tags: Vec<String>,
}

impl Identity {
    fn new(id: usize, name: &str, behavior: BehaviorProfile) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            name: name.to_string(),
            created_at: now,
            last_used: now,
            use_count: 0,
            success_rate: 0.5,
            confidence: 0.6,
            behavior,
            user_agent: String::new(),
            screen_resolution: (1920, 1080),
            language: "en-US".to_string(),
            canvas_fingerprint: String::new(),
            audio_fingerprint: String::new(),
            age_seconds: 0.0,
            tags: Vec::new(),
        }
    }

    pub fn age_days(&self) -> f64 {
        self.age_seconds / 86400.0
    }

    pub fn record_use(&mut self, success: bool) {
        self.use_count += 1;
        self.last_used = SystemTime::now();
        let total = self.use_count as f64;
        self.success_rate = ((self.success_rate * (total - 1.0)) + if success { 1.0 } else { 0.0 }) / total;
    }

    pub fn decay(&mut self) {
        let now = SystemTime::now();
        let elapsed = now.duration_since(self.last_used).unwrap_or_default();
        self.age_seconds = elapsed.as_secs_f64();
        if elapsed > Duration::from_secs(86400) {
            self.confidence *= 0.95;
        }
    }
}

#[derive(Debug, Clone)]
pub struct IdentityPool {
    pub identities: Vec<Identity>,
    pub active_ids: Vec<usize>,
    pub retired_ids: Vec<usize>,
    pub pool_size: usize,
    pub rotation_strategy: RotationStrategy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RotationStrategy {
    LeastUsed,
    RoundRobin,
    HighestConfidence,
    Random,
    Targeted,
}

impl IdentityPool {
    pub fn new(pool_size: usize, strategy: RotationStrategy) -> Self {
        let mut pool = Self {
            identities: Vec::with_capacity(pool_size),
            active_ids: Vec::new(),
            retired_ids: Vec::new(),
            pool_size,
            rotation_strategy: strategy,
        };
        pool.initialize();
        pool
    }

    fn initialize(&mut self) {
        for i in 0..self.pool_size {
            let profile_idx = i % 3;
            let profile = match profile_idx {
                0 => BehaviorProfile::Human,
                1 => BehaviorProfile::Mixed,
                _ => BehaviorProfile::Automated,
            };
            let mut identity = Identity::new(i, &format!("identity_{}", i), profile);
            identity.user_agent = match i % 4 {
                0 => "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
                1 => "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
                2 => "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
                _ => "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15".to_string(),
            };
            identity.screen_resolution = match i % 3 {
                0 => (1920, 1080),
                1 => (2560, 1440),
                _ => (1440, 900),
            };
            identity.language = match i % 5 {
                0 => "en-US".to_string(),
                1 => "en-GB".to_string(),
                2 => "de-DE".to_string(),
                3 => "ja-JP".to_string(),
                _ => "fr-FR".to_string(),
            };
            identity.tags.push(format!("pool_{}", i % 3));
            self.identities.push(identity);
            self.active_ids.push(i);
        }
    }

    pub fn select(&mut self, target_tags: &[String]) -> Option<&mut Identity> {
        let candidates: Vec<usize> = if target_tags.is_empty() {
            self.active_ids.clone()
        } else {
            self.active_ids.iter()
                .filter(|id| {
                    if let Some(id) = self.identities.get(**id) {
                        target_tags.iter().any(|t| id.tags.contains(t))
                    } else { false }
                })
                .copied()
                .collect()
        };

        if candidates.is_empty() {
            return self.active_ids.first()
                .and_then(|id| self.identities.get_mut(*id));
        }

        let idx = match self.rotation_strategy {
            RotationStrategy::LeastUsed => {
                candidates.into_iter()
                    .min_by_key(|id| self.identities[*id].use_count)
                    .unwrap_or(0)
            }
            RotationStrategy::RoundRobin => {
                let next = candidates.iter()
                    .min_by_key(|id| {
                        self.identities[**id].last_used
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    })
                    .copied()
                    .unwrap_or(0);
                next
            }
            RotationStrategy::HighestConfidence => {
                candidates.into_iter()
                    .max_by(|a, b| {
                        self.identities[*a].confidence
                            .partial_cmp(&self.identities[*b].confidence)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap_or(0)
            }
            RotationStrategy::Random => {
                let i = rand::thread_rng().gen_range(0..candidates.len());
                candidates[i]
            }
            RotationStrategy::Targeted => {
                candidates.into_iter()
                    .max_by(|a, b| {
                        let sa = self.identities[*a].success_rate * self.identities[*a].confidence;
                        let sb = self.identities[*b].success_rate * self.identities[*b].confidence;
                        sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap_or(0)
            }
        };

        self.identities.get_mut(idx)
    }

    pub fn retire(&mut self, id: usize) -> bool {
        if let Some(pos) = self.active_ids.iter().position(|x| *x == id) {
            self.active_ids.remove(pos);
            self.retired_ids.push(id);
            true
        } else {
            false
        }
    }

    pub fn replenish(&mut self) -> usize {
        let needed = self.pool_size.saturating_sub(self.active_ids.len());
        let start = self.identities.len();
        for i in 0..needed {
            let profile = match i % 3 {
                0 => BehaviorProfile::Human,
                1 => BehaviorProfile::Mixed,
                _ => BehaviorProfile::Automated,
            };
            let mut identity = Identity::new(start + i, &format!("identity_{}", start + i), profile);
            identity.confidence = 0.4;
            identity.success_rate = 0.5;
            self.identities.push(identity);
            self.active_ids.push(start + i);
        }
        needed
    }

    pub fn stats(&self) -> PoolStats {
        let active = self.active_ids.len();
        let avg_success: f64 = self.active_ids.iter()
            .filter_map(|id| self.identities.get(*id))
            .map(|i| i.success_rate)
            .sum::<f64>() / active.max(1) as f64;
        let avg_conf: f64 = self.active_ids.iter()
            .filter_map(|id| self.identities.get(*id))
            .map(|i| i.confidence)
            .sum::<f64>() / active.max(1) as f64;

        PoolStats {
            total_identities: self.identities.len(),
            active_count: active,
            retired_count: self.retired_ids.len(),
            avg_success_rate: avg_success,
            avg_confidence: avg_conf,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_identities: usize,
    pub active_count: usize,
    pub retired_count: usize,
    pub avg_success_rate: f64,
    pub avg_confidence: f64,
}

#[derive(Debug, Clone)]
pub struct BehaviorSimulator {
    pub base_delay_ms: (u64, u64),
    pub typo_rate: f64,
    pub scroll_variance: f64,
    pub click_jitter_px: f64,
}

impl Default for BehaviorSimulator {
    fn default() -> Self { Self::new() }
}

impl BehaviorSimulator {
    pub fn new() -> Self {
        Self { base_delay_ms: (80, 350), typo_rate: 0.025, scroll_variance: 0.3, click_jitter_px: 3.0 }
    }

    pub fn delay_ms(&self, profile: &BehaviorProfile) -> u64 {
        let base = profile.action_delay_ms();
        let jitter = rand::thread_rng().gen_range(0..50u64);
        rand::thread_rng().gen_range(base.0..base.1) + jitter
    }

    pub fn simulate_typing(&self, text: &str, profile: &BehaviorProfile) -> Vec<(char, u64)> {
        let mut keystrokes = Vec::with_capacity(text.len());
        let typo_rate = profile.typos_per_100_chars() / 100.0;

        for (_i, c) in text.chars().enumerate() {
            let delay = self.delay_ms(profile);
            keystrokes.push((c, delay));

            if rand::thread_rng().gen_bool(typo_rate) {
                let typo_delay = delay + rand::thread_rng().gen_range(100..400);
                keystrokes.push((self.random_char(), typo_delay));
                let fix_delay = rand::thread_rng().gen_range(200..600);
                keystrokes.push(('\u{7f}', fix_delay));
                keystrokes.push((c, self.delay_ms(profile)));
            }
        }

        keystrokes
    }

    fn random_char(&self) -> char {
        let chars = "abcdefghijklmnopqrstuvwxyz";
        let idx = rand::thread_rng().gen_range(0..chars.len());
        chars.as_bytes()[idx] as char
    }

    pub fn total_typing_time(&self, text: &str, profile: &BehaviorProfile) -> u64 {
        self.simulate_typing(text, profile).iter().map(|(_, d)| d).sum()
    }
}

#[derive(Debug, Clone)]
pub struct StealthManager {
    pub pool: IdentityPool,
    pub simulator: BehaviorSimulator,
    pub last_rotation: SystemTime,
    pub rotation_interval: Duration,
}

impl StealthManager {
    pub fn new(pool_size: usize) -> Self {
        Self {
            pool: IdentityPool::new(pool_size, RotationStrategy::Targeted),
            simulator: BehaviorSimulator::new(),
            last_rotation: SystemTime::now(),
            rotation_interval: Duration::from_secs(3600),
        }
    }

    pub fn get_identity(&mut self, tags: &[String]) -> Option<&mut Identity> {
        let elapsed = self.last_rotation.elapsed().unwrap_or_default();
        if elapsed > self.rotation_interval {
            self.rotate_pool();
            self.last_rotation = SystemTime::now();
        }
        self.pool.select(tags)
    }

    pub fn get_scored_identity(&mut self, tags: &[String], min_confidence: f64) -> Option<&mut Identity> {
        let idx = self.pool.select(tags)
            .filter(|id| id.confidence >= min_confidence)
            .map(|id| id.id);
        match idx {
            Some(id) => self.pool.identities.get_mut(id),
            None => self.pool.select(tags),
        }
    }

    pub fn record_interaction(&mut self, id: usize, success: bool) {
        if let Some(identity) = self.pool.identities.get_mut(id) {
            identity.record_use(success);
        }
        self.decay_all();
        self.maybe_replenish();
    }

    fn decay_all(&mut self) {
        for identity in &mut self.pool.identities {
            identity.decay();
        }
    }

    fn maybe_replenish(&mut self) {
        let active = self.pool.active_ids.len();
        if active < self.pool.pool_size / 2 {
            let replenished = self.pool.replenish();
            log::info!("StealthManager: replenished {} identities", replenished);
        }
    }

    fn rotate_pool(&mut self) {
        let low_confidence: Vec<usize> = self.pool.active_ids.iter()
            .filter_map(|id| self.pool.identities.get(*id))
            .filter(|i| i.confidence < 0.2 || i.success_rate < 0.1)
            .map(|i| i.id)
            .collect();

        for id in low_confidence {
            self.pool.retire(id);
        }
        self.pool.replenish();
    }

    pub fn stats(&self) -> StealthManagerStats {
        let pool_stats = self.pool.stats();
        StealthManagerStats {
            total_identities: pool_stats.total_identities,
            active_count: pool_stats.active_count,
            retired_count: pool_stats.retired_count,
            avg_success_rate: pool_stats.avg_success_rate,
            avg_confidence: pool_stats.avg_confidence,
            simulator_typo_rate: self.simulator.typo_rate,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StealthManagerStats {
    pub total_identities: usize,
    pub active_count: usize,
    pub retired_count: usize,
    pub avg_success_rate: f64,
    pub avg_confidence: f64,
    pub simulator_typo_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_pool_init() {
        let pool = IdentityPool::new(5, RotationStrategy::LeastUsed);
        assert_eq!(pool.identities.len(), 5);
        assert_eq!(pool.active_ids.len(), 5);
    }

    #[test]
    fn test_select_identity() {
        let mut pool = IdentityPool::new(3, RotationStrategy::Random);
        let tags = vec!["pool_0".to_string()];
        let identity = pool.select(&tags);
        assert!(identity.is_some());
        assert!(identity.unwrap().tags.contains(&"pool_0".to_string()));
    }

    #[test]
    fn test_retire_and_replenish() {
        let mut pool = IdentityPool::new(4, RotationStrategy::RoundRobin);
        assert!(pool.retire(0));
        assert_eq!(pool.active_ids.len(), 3);

        let replenished = pool.replenish();
        assert_eq!(replenished, 1);
        assert_eq!(pool.active_ids.len(), 4);
    }

    #[test]
    fn test_behavior_simulator_typing() {
        let sim = BehaviorSimulator::new();
        let keystrokes = sim.simulate_typing("hello", &BehaviorProfile::Human);
        assert!(!keystrokes.is_empty());
        assert_eq!(keystrokes[0].0, 'h');
    }

    #[test]
    fn test_nt_shield_manager_get_identity() {
        let mut sm = StealthManager::new(3);
        let tags = vec![];
        let identity = sm.get_identity(&tags);
        assert!(identity.is_some());
    }

    #[test]
    fn test_record_interaction() {
        let mut sm = StealthManager::new(2);
        sm.record_interaction(0, true);
        let stats = sm.stats();
        assert_eq!(stats.total_identities, 2);
        assert!(stats.avg_success_rate > 0.0);
    }

    #[test]
    fn test_strategy_rotation() {
        let mut a = IdentityPool::new(3, RotationStrategy::LeastUsed);
        let mut b = IdentityPool::new(3, RotationStrategy::HighestConfidence);
        let tags = vec![];

        let id_a = a.select(&tags);
        let id_b = b.select(&tags);
        assert!(id_a.is_some());
        assert!(id_b.is_some());
    }

    #[test]
    fn test_pool_stats() {
        let pool = IdentityPool::new(4, RotationStrategy::Random);
        let stats = pool.stats();
        assert_eq!(stats.total_identities, 4);
        assert_eq!(stats.active_count, 4);
    }
}
