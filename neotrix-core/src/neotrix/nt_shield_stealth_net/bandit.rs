//! Thompson Sampling Bandit — 自适应指纹选择 (TLS × Platform × Geo)
//!
//! 每个 (TlsVariant, Platform, GeoRegion) 组合维护 Beta(α, β) 后验,
//! 每次 fetch() 从匹配当前出口代理地理位置的后验采样, 选最大采样值的组合.
//!
//! 4 TLS × 3 Platform × 5 Geo = 60 组合臂.

pub const GEO_REGIONS: &[&str] = &["", "US", "EU", "ASIA", "OTHER"];

use rand::Rng;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::neotrix::nt_io_http_factory::{TlsVariant, H2SettingsProfile};
use super::system_fingerprint::Platform;
use super::config::load as cfg;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ComboArm {
    pub tls: TlsVariant,
    pub platform: Platform,
    pub h2_profile: H2SettingsProfile,
    #[serde(default)]
    pub geo_tag: String,
}

impl ComboArm {
    pub fn all() -> Vec<ComboArm> {
        let tls_variants = TlsVariant::all();
        let platforms = Platform::all();
        let h2_profiles = H2SettingsProfile::all();
        let mut arms = Vec::new();
        for tls in tls_variants {
            for platform in platforms.iter().take(3) {
                for h2 in h2_profiles {
                    for geo in GEO_REGIONS {
                        arms.push(ComboArm {
                            tls: *tls,
                            platform: *platform,
                            h2_profile: *h2,
                            geo_tag: geo.to_string(),
                        });
                    }
                }
            }
        }
        arms
    }
}

struct ArmStats {
    success: AtomicU64,
    fail: AtomicU64,
}

pub struct FingerprintBandit {
    arms: Vec<(ComboArm, ArmStats)>,
}

impl Default for FingerprintBandit {
    fn default() -> Self {
        Self::new()
    }
}

impl FingerprintBandit {
    /// 加载持久化的 bandit（如果存在），否则新建
    pub fn load() -> Self {
        let bandit = Self::new();
        if let Ok(path) = bandit_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(loaded) = serde_json::from_str::<Vec<(ComboArm, u64, u64)>>(&content) {
                        for (arm, success, fail) in loaded {
                            for (a, stats) in &bandit.arms {
                                if *a == arm {
                                    stats.success.store(success, Ordering::Relaxed);
                                    stats.fail.store(fail, Ordering::Relaxed);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        bandit
    }

    pub fn new() -> Self {
        let arms = ComboArm::all().iter().map(|arm| {
            (arm.clone(), ArmStats { success: AtomicU64::new(1), fail: AtomicU64::new(1) })
        }).collect();
        Self { arms }
    }

    /// 保存当前臂参数到 ~/.neotrix/bandit.json
    pub fn save(&self) {
        let path = match bandit_path() {
            Ok(p) => p,
            Err(_) => return,
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let data: Vec<(ComboArm, u64, u64)> = self.arms.iter().map(|(a, s)| {
            (a.clone(), s.success.load(Ordering::Relaxed), s.fail.load(Ordering::Relaxed))
        }).collect();
        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = fs::write(&path, json);
        }
    }

    /// 连续 reward 更新 — reward ∈ [0, 1]
    /// 每个 reward 产生 10 个虚拟观测，按 reward 比例分配 α/β
    pub fn update(&self, arm: ComboArm, reward: f64) {
        let reward = reward.clamp(0.0, 1.0);
        let k = 10u64;
        for (a, stats) in &self.arms {
            if *a == arm {
                stats.success.fetch_add((reward * k as f64).round() as u64, Ordering::Relaxed);
                stats.fail.fetch_add(((1.0 - reward) * k as f64).round() as u64, Ordering::Relaxed);
                self.save();
                return;
            }
        }
    }

    /// Thompson Sampling: 从匹配 geo 的臂中采样, 选最大
    /// geo=None 时从所有臂采样
    pub fn select_arm(&self, geo: Option<&str>) -> ComboArm {
        let mut rng = rand::thread_rng();
        let mut best_arm = self.arms[0].0.clone();
        let mut best_sample = f64::NEG_INFINITY;
        for (arm, stats) in &self.arms {
            if let Some(geo) = geo {
                if arm.geo_tag != geo { continue; }
            }
            let alpha = stats.success.load(Ordering::Relaxed) as f64;
            let beta = stats.fail.load(Ordering::Relaxed) as f64;
            let sample = sample_beta(&mut rng, alpha, beta);
            if sample > best_sample {
                best_sample = sample;
                best_arm = arm.clone();
            }
        }
        best_arm
    }

    /// 置信度: 最优臂 vs 次优臂的期望值差距 / 0.5
    /// 1.0 = 明确最优, 0.0 = 所有臂相当
    pub fn confidence(&self) -> f64 {
        let mut rates: Vec<f64> = self.arms.iter().map(|(_, s)| {
            let a = s.success.load(Ordering::Relaxed) as f64;
            let b = s.fail.load(Ordering::Relaxed) as f64;
            a / (a + b)
        }).collect();
        rates.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        if rates.len() < 2 { return 1.0; }
        let gap = rates[0] - rates[1];
        (gap / 0.5).clamp(0.0, 1.0)
    }

    pub fn stats(&self) -> Vec<(ComboArm, u64, u64, f64)> {
        self.arms.iter().map(|(a, s)| {
            let alpha = s.success.load(Ordering::Relaxed);
            let beta = s.fail.load(Ordering::Relaxed);
            let rate = alpha as f64 / (alpha + beta) as f64;
            (a.clone(), alpha, beta, rate)
        }).collect()
    }

    /// 计算与另一个 bandit 的余弦相似度 (基于臂成功率分布)
    pub fn similarity(&self, other: &FingerprintBandit) -> f64 {
        let self_rates: Vec<f64> = self.arms.iter().map(|(_, s)| {
            s.success.load(Ordering::Relaxed) as f64 / (s.success.load(Ordering::Relaxed) + s.fail.load(Ordering::Relaxed)) as f64
        }).collect();
        let other_rates: Vec<f64> = other.arms.iter().map(|(_, s)| {
            s.success.load(Ordering::Relaxed) as f64 / (s.success.load(Ordering::Relaxed) + s.fail.load(Ordering::Relaxed)) as f64
        }).collect();
        let dot: f64 = self_rates.iter().zip(other_rates.iter()).map(|(a, b)| a * b).sum();
        let norm_self: f64 = self_rates.iter().map(|v| v * v).sum::<f64>().sqrt();
        let norm_other: f64 = other_rates.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm_self < 1e-12 || norm_other < 1e-12 { return 0.0; }
        (dot / (norm_self * norm_other)).clamp(0.0, 1.0)
    }

    /// 将另一个 bandit 的经验合并到此 bandit (加权平均, other_weight=0.3)
    pub fn migrate_from(&self, other: &FingerprintBandit, other_weight: f64) {
        let w = other_weight.clamp(0.0, 0.5);
        for (arm, stats) in &self.arms {
            if let Some((_, other_stats)) = other.arms.iter().find(|(a, _)| *a == *arm) {
                let o_s = other_stats.success.load(Ordering::Relaxed);
                let o_f = other_stats.fail.load(Ordering::Relaxed);
                if o_s + o_f > 5 { // 至少有意义的观测
                    let self_s = stats.success.load(Ordering::Relaxed);
                    let self_f = stats.fail.load(Ordering::Relaxed);
                    let merged_s = (self_s as f64 * (1.0 - w) + o_s as f64 * w).round() as u64;
                    let merged_f = (self_f as f64 * (1.0 - w) + o_f as f64 * w).round() as u64;
                    stats.success.store(merged_s, Ordering::Relaxed);
                    stats.fail.store(merged_f, Ordering::Relaxed);
                }
            }
        }
        self.save();
    }

    /// 遍历所有 per-host bandits, 找到相似度 > 0.9 的自动迁移
    pub fn auto_migrate(bandits: &[(String, FingerprintBandit)]) -> usize {
        let mut migration_count = 0;
        for i in 0..bandits.len() {
            for j in (i + 1)..bandits.len() {
                let sim = bandits[i].1.similarity(&bandits[j].1);
                if sim > 0.9 {
                    log::info!("[bandit] auto-migrate {} → {} (sim={:.3})", bandits[i].0, bandits[j].0, sim);
                    bandits[i].1.migrate_from(&bandits[j].1, 0.3);
                    migration_count += 1;
                }
            }
        }
        migration_count
    }
}

fn bandit_path() -> Result<PathBuf, String> {
    let c = cfg();
    let path_str = shellexpand::tilde(&c.bandit.persistence_path).to_string();
    Ok(PathBuf::from(path_str))
}

/// Beta(α, β) 采样 — Marsaglia-Tsang 法
fn sample_beta(rng: &mut impl Rng, alpha: f64, beta: f64) -> f64 {
    if alpha <= 0.0 || beta <= 0.0 { return 0.5; }
    // Beta(a,b) = Gamma(a) / (Gamma(a) + Gamma(b))
    let x = sample_gamma(rng, alpha);
    let y = sample_gamma(rng, beta);
    if (x + y).abs() < 1e-12 { 0.5 } else { x / (x + y) }
}

/// Gamma(shape, 1) 采样 — Marsaglia-Tsang (shape >= 1)
fn sample_gamma(rng: &mut impl Rng, shape: f64) -> f64 {
    if shape < 1.0 {
        return sample_gamma(rng, shape + 1.0) * rng.gen::<f64>().powf(1.0 / shape);
    }
    let d = shape - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();
    loop {
        // Box-Muller 标准正态采样 (避免 rand::distributions::StandardNormal 版本问题)
        let (u1, u2): (f64, f64) = (rng.gen(), rng.gen());
        let x = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let v = 1.0 + c * x;
        if v <= 0.0 { continue; }
        let v = v * v * v;
        if v <= 0.0 { continue; }
        let u: f64 = rng.gen();
        if u < 1.0 - 0.0331 * (x * x) * (x * x) { return d * v; }
        if u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln()) { return d * v; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_io_http_factory::{TlsVariant, H2SettingsProfile};
    use crate::neotrix::nt_shield_stealth_net::system_fingerprint::Platform;

    fn test_arm(tls: TlsVariant, platform: Platform) -> ComboArm {
        ComboArm { tls, platform, h2_profile: H2SettingsProfile::ChromeDefault, geo_tag: String::new() }
    }

    #[test]
    fn test_bandit_creation() {
        let bandit = FingerprintBandit::new();
        let stats = bandit.stats();
        assert_eq!(stats.len(), ComboArm::all().len());
        assert!(stats.len() >= 240);
    }

    #[test]
    fn test_select_arm_returns_valid() {
        let bandit = FingerprintBandit::new();
        let all_arms = ComboArm::all();
        for _ in 0..20 {
            let arm = bandit.select_arm(None);
            assert!(all_arms.contains(&arm));
        }
    }

    #[test]
    fn test_update_affects_stats() {
        let bandit = FingerprintBandit::new();
        let before = bandit.stats();
        let arm = test_arm(TlsVariant::ModernH2, Platform::Windows);
        bandit.update(arm.clone(), 0.9);
        let after = bandit.stats();
        for (a, alpha, _beta, _) in &after {
            if *a == arm {
                assert!(*alpha > before[0].1);
                return;
            }
        }
        panic!("arm not found");
    }

    #[test]
    fn test_bandit_converges_to_best_arm() {
        let bandit = FingerprintBandit::new();
        let best = test_arm(TlsVariant::LegacyHttp11, Platform::MacOS);
        let worst = test_arm(TlsVariant::ModernH2, Platform::Windows);
        use crate::neotrix::nt_io_http_factory::H2SettingsProfile;
        // Train all 4 h2 variants of the best/worst pattern so they dominate
        let mut train_arms = Vec::new();
        for h2 in H2SettingsProfile::all() {
            let mut b = best.clone(); b.h2_profile = *h2;
            let mut w = worst.clone(); w.h2_profile = *h2;
            train_arms.push(b);
            train_arms.push(w);
        }
        // Also train all TLS/platform combos to push them down
        for tls in crate::neotrix::nt_io_http_factory::TlsVariant::all() {
            for plat in crate::neotrix::nt_shield_stealth_net::system_fingerprint::Platform::all().iter().take(3) {
                for h2 in H2SettingsProfile::all() {
                    let cnt = if *tls == TlsVariant::LegacyHttp11 { train_arms.iter().filter(|a| a.tls == *tls).count() as u64 } else { 0 };
                    if cnt == 0 {
                        let a = ComboArm { tls: *tls, platform: *plat, h2_profile: *h2, geo_tag: String::new() };
                        train_arms.push(a);
                    }
                }
            }
        }
        for a in &train_arms {
            let reward = if a.tls == TlsVariant::LegacyHttp11 { 0.9 } else { 0.1 };
            for _ in 0..50 { bandit.update(a.clone(), reward); }
        }
        let counts: std::collections::HashMap<_, _> = (0..500)
            .map(|_| bandit.select_arm(Some("")))
            .fold(std::collections::HashMap::new(), |mut acc, v| {
                *acc.entry(v).or_insert(0) += 1; acc
            });
        let champ = counts.iter().max_by_key(|(_, c)| *c).map(|(v, _)| v).expect("bandit should have a winner");
        assert_eq!(champ.tls, TlsVariant::LegacyHttp11, "best tls should be LegacyHttp11");
        assert_eq!(champ.platform, Platform::MacOS, "best platform should be MacOS");
    }

    #[test]
    fn test_save_load_roundtrip() {
        // 使用临时路径避免依赖 OnceLock 中的持久化文件
        let tmp = std::env::temp_dir().join("neotrix_bandit_test.json");
        let bandit = FingerprintBandit::new();
        let arm = test_arm(TlsVariant::LegacyHttp11, Platform::Linux);
        bandit.update(arm.clone(), 0.9);
        bandit.update(arm.clone(), 0.9);
        bandit.update(arm.clone(), 0.1);

        // 手工保存到临时路径
        let data: Vec<(ComboArm, u64, u64)> = bandit.stats().into_iter().map(|(a, s, f, _)| (a, s, f)).collect();
        let json = serde_json::to_string_pretty(&data).expect("stats should serialize to json");
        std::fs::write(&tmp, &json).expect("failed to write bandit data to temp file");

        // 反序列化验证
        let content = std::fs::read_to_string(&tmp).expect("failed to read back bandit data");
        let loaded: Vec<(ComboArm, u64, u64)> = serde_json::from_str(&content).expect("bandit data should deserialize");
        for (a, alpha, beta) in &loaded {
            if *a == arm {
                assert!(*alpha > 2);
                assert!(*beta > 1);
                let _ = std::fs::remove_file(&tmp);
                return;
            }
        }
        let _ = std::fs::remove_file(&tmp);
        panic!("arm not found after load");
    }

    #[test]
    fn test_beta_sampling_bounds() {
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let s = sample_beta(&mut rng, 2.0, 5.0);
            assert!(s > 0.0 && s < 1.0);
        }
    }
}
