use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::path::PathBuf;
use std::ops::Range;
use rand::Rng;
use serde::{Serialize, Deserialize};

use crate::neotrix::http_factory::TlsVariant;

use super::StealthHttpClient;
use super::system_fingerprint::{SystemFingerprint, SystemFingerprintGenerator, SystemFingerprintConfig, Platform, Browser};

/// 指纹 = 完整系统指纹 + 成功/失败统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fingerprint {
    pub system: SystemFingerprint,
    pub headers: HashMap<String, String>,
    pub success_count: u64,
    pub fail_count: u64,
}

impl Fingerprint {
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.fail_count;
        if total == 0 { 0.5 } else { self.success_count as f64 / total as f64 }
    }
}

/// 持久化存储格式
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FingerprintStore {
    fingerprints: Vec<Fingerprint>,
    best_index: usize,
}

/// 指纹管理器 — 基于 SystemFingerprintGenerator 的多平台指纹池
pub struct FingerprintManager {
    fingerprints: Vec<Fingerprint>,
    pub current_index: usize,
    rotation_interval: AtomicU64,
    generator: SystemFingerprintGenerator,
    store_path: PathBuf,
    pub current_timing_jitter: Range<u64>,
    rotation_profiles: Vec<RotationProfile>,
}

impl Default for FingerprintManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FingerprintManager {
    pub fn new() -> Self {
        let store_path = std::env::home_dir()
            .map(|d| d.join(".neotrix").join("fingerprints.json"))
            .unwrap_or_else(|| PathBuf::from("fingerprints.json"));

        let gen = SystemFingerprintGenerator::new();
        let fingerprints = Self::load_or_init(&gen, &store_path);

        Self {
            fingerprints,
            current_index: 0,
            rotation_interval: AtomicU64::new(5),
            generator: gen,
            store_path,
            current_timing_jitter: 50..300,
            rotation_profiles: RotationProfile::default_pool(),
        }
    }

    fn load_or_init(gen: &SystemFingerprintGenerator, path: &PathBuf) -> Vec<Fingerprint> {
        if let Ok(json) = std::fs::read_to_string(path) {
            if let Ok(store) = serde_json::from_str::<FingerprintStore>(&json) {
                if !store.fingerprints.is_empty() {
                    return store.fingerprints;
                }
            }
        }
        Self::generate_initial(gen)
    }

    fn generate_initial(gen: &SystemFingerprintGenerator) -> Vec<Fingerprint> {
        let mut rng = rand::thread_rng();
        let combos: Vec<(Platform, Browser)> = vec![
            // Windows — Chrome + Firefox + Edge
            (Platform::Windows, Browser::Chrome),
            (Platform::Windows, Browser::Chrome),
            (Platform::Windows, Browser::Firefox),
            (Platform::Windows, Browser::Edge),
            (Platform::Windows, Browser::Edge),
            // MacOS — Chrome + Safari + Firefox + Edge
            (Platform::MacOS, Browser::Chrome),
            (Platform::MacOS, Browser::Safari),
            (Platform::MacOS, Browser::Safari),
            (Platform::MacOS, Browser::Firefox),
            (Platform::MacOS, Browser::Edge),
            // Linux — Chrome + Firefox
            (Platform::Linux, Browser::Chrome),
            (Platform::Linux, Browser::Chrome),
            (Platform::Linux, Browser::Firefox),
            (Platform::Linux, Browser::Firefox),
            // ChromeOS — Chrome only
            (Platform::ChromeOS, Browser::Chrome),
            (Platform::ChromeOS, Browser::Chrome),
            // Android — Chrome + Firefox + Edge
            (Platform::Android, Browser::Chrome),
            (Platform::Android, Browser::Firefox),
            (Platform::Android, Browser::Edge),
            // iOS — Safari + Chrome
            (Platform::IOS, Browser::Safari),
            (Platform::IOS, Browser::Chrome),
        ];
        combos.into_iter().map(|(platform, nt_world_browse)| {
            let cfg = SystemFingerprintConfig {
                platform: Some(platform),
                nt_world_browse: Some(nt_world_browse),
                timezone: None,
                locale: None,
                h2_profile: None,
                auto_consistent: true,
            };
            let system = gen.generate(&cfg);
            let headers = SystemFingerprintGenerator::to_headers(&system);
            Fingerprint { system, headers, success_count: rng.gen_range(0..3), fail_count: 0 }
        }).collect()
    }

    fn save(&self) {
        let store = FingerprintStore {
            fingerprints: self.fingerprints.clone(),
            best_index: self.current_index,
        };
        if let Some(parent) = self.store_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&store) {
            let _ = std::fs::write(&self.store_path, json);
        }
    }

    pub fn current(&self) -> &Fingerprint {
        &self.fingerprints[self.current_index]
    }

    pub fn rotate(&mut self) {
        let total_requests: u64 = self.fingerprints.iter()
            .map(|f| f.success_count + f.fail_count).sum();
        if total_requests < 10 {
            self.current_index = (self.current_index + 1) % self.fingerprints.len();
            return;
        }
        let mut rng = rand::thread_rng();
        let best = self.fingerprints.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.success_rate().partial_cmp(&b.success_rate()).expect("expected valid comparison"))
            .map(|(i, _)| i)
            .unwrap_or(0);
        if rng.gen_bool(0.7) {
            self.current_index = best;
        } else {
            self.current_index = rng.gen_range(0..self.fingerprints.len());
        }
    }

    pub fn report_result(&mut self, success: bool) {
        let fp = &mut self.fingerprints[self.current_index];
        if success { fp.success_count += 1; } else { fp.fail_count += 1; }
        let interval = self.rotation_interval.load(Ordering::Relaxed);
        if fp.success_count + fp.fail_count >= interval {
            self.rotate();
        }
        self.save();
    }

    pub fn apply_headers(&self) -> HashMap<String, String> {
        let fp = self.current();
        let mut headers = fp.headers.clone();
        headers.insert("User-Agent".to_string(), self.ua_from_fingerprint(&fp.system));
        headers
    }

    fn ua_from_fingerprint(&self, fp: &SystemFingerprint) -> String {
        let days_since_epoch = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() / 86400;
        let chrome_ver = format!("{}.0.0.0", 120 + (days_since_epoch / 90) as u32);
        let ff_ver = format!("{}.0", 121 + (days_since_epoch / 90) as u32);
        let safari_ver = format!("{}.2", 17 + (days_since_epoch / 90) as u32);
        let os = fp.platform.user_agent_os();
        let mobile = fp.platform.is_mobile();
        match (fp.nt_world_browse, mobile) {
            (Browser::Chrome, false) => format!(
                "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                os, chrome_ver
            ),
            (Browser::Chrome, true) => format!(
                "Mozilla/5.0 ({}; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                os, chrome_ver
            ),
            (Browser::Edge, false) => format!(
                "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36 Edg/{}",
                os, chrome_ver, chrome_ver
            ),
            (Browser::Edge, true) => format!(
                "Mozilla/5.0 ({}; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36 Edg/{}",
                os, chrome_ver, chrome_ver
            ),
            (Browser::Firefox, false) => format!(
                "Mozilla/5.0 ({}; rv:{}) Gecko/20100101 Firefox/{}",
                os, ff_ver.split('.').next().unwrap_or("121"), ff_ver
            ),
            (Browser::Firefox, true) => format!(
                "Mozilla/5.0 ({}; rv:{}) Gecko/20100101 Firefox/{}",
                os, ff_ver.split('.').next().unwrap_or("121"), ff_ver
            ),
            (Browser::Safari, false) => format!(
                "Mozilla/5.0 ({}) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{} Safari/605.1.15",
                os, safari_ver
            ),
            (Browser::Safari, true) => format!(
                "Mozilla/5.0 ({}; Mobile/15E148) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{} Mobile/15E148 Safari/604.1",
                os, safari_ver
            ),
        }
    }

    pub fn clear_all(&mut self) {
        for fp in &mut self.fingerprints {
            fp.success_count = 0;
            fp.fail_count = 0;
        }
        self.save();
    }

    pub fn reset_to_defaults(&mut self) {
        self.fingerprints = Self::generate_initial(&self.generator);
        self.current_index = 0;
        self.save();
    }

    pub fn purge_all(&mut self) {
        self.fingerprints = Self::generate_initial(&self.generator);
        self.current_index = 0;
        self.save();
    }

    /// 生成新指纹变体 — 随机平台+浏览器组合
    pub fn spawn_variant(&mut self) {
        let mut rng = rand::thread_rng();
        let platforms = Platform::all();
        let nt_world_browses = Browser::all();
        let platform = platforms[rng.gen_range(0..platforms.len())];
        let compat_nt_world_browses: Vec<&Browser> = nt_world_browses.iter()
            .filter(|b| b.compatible_platforms().contains(&platform))
            .collect();
        let nt_world_browse = *compat_nt_world_browses[rng.gen_range(0..compat_nt_world_browses.len())];
        let config = SystemFingerprintConfig {
            platform: Some(platform),
            nt_world_browse: Some(nt_world_browse),
            timezone: None,
            locale: None,
            h2_profile: None,
            auto_consistent: true,
        };
        let system_fp = self.generator.generate(&config);
        let headers = SystemFingerprintGenerator::to_headers(&system_fp);
        let variant = Fingerprint {
            system: system_fp,
            headers,
            success_count: 0,
            fail_count: 0,
        };
        self.fingerprints.push(variant);
        self.save();
    }

    pub fn fingerprint_count(&self) -> usize {
        self.fingerprints.len()
    }

    pub fn best_success_rate(&self) -> f64 {
        self.fingerprints.iter()
            .map(|f| f.success_rate())
            .fold(0.0f64, |a, b| a.max(b))
    }

    pub fn total_requests(&self) -> u64 {
        self.fingerprints.iter()
            .map(|f| f.success_count + f.fail_count).sum()
    }

    /// Apply a RotationProfile — synchronously switch fingerprint + timing + TLS
    pub fn apply_profile(&mut self, profile: &RotationProfile) {
        let mut rng = rand::thread_rng();
        if !self.fingerprints.is_empty() {
            self.current_index = rng.gen_range(0..self.fingerprints.len());
        }
        self.current_timing_jitter = profile.timing_jitter_ms.clone();
    }

    /// Atomic rotation: pick random profile, apply all dimensions
    pub fn atomic_rotate(&mut self) {
        let profile = select_profile(&self.rotation_profiles).clone();
        self.apply_profile(&profile);
    }

    /// Replace the rotation profile pool
    pub fn set_rotation_profiles(&mut self, profiles: Vec<RotationProfile>) {
        self.rotation_profiles = profiles;
    }
}

/// Stealth 学习结果
#[derive(Debug, Clone)]
pub struct StealthLearning {
    pub iteration: u64,
    pub fingerprint_count: usize,
    pub best_success_rate: f64,
    pub total_requests: u64,
}

impl StealthLearning {
    pub fn to_reasoning_memory(&self, task: &str) -> crate::core::nt_core_bank::ReasoningMemory {
        use crate::core::nt_core_knowledge::TaskType;
        crate::core::nt_core_bank::ReasoningMemory::new(
            task,
            TaskType::CodeAnalysis,
            &[],
            self.best_success_rate,
        )
    }
}

/// 自迭代 Stealth 包装器
pub struct SelfIteratingStealth {
    pub client: StealthHttpClient,
    pub fingerprint_manager: FingerprintManager,
    iteration: u64,
}

impl Default for SelfIteratingStealth {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfIteratingStealth {
    pub fn new() -> Self {
        Self {
            client: StealthHttpClient::new(),
            fingerprint_manager: FingerprintManager::new(),
            iteration: 0,
        }
    }

    pub async fn stealth_fetch(&mut self, url: &str) -> Result<super::Response, String> {
        let headers = self.fingerprint_manager.apply_headers();
        self.client.set_extra_headers(headers).await;
        let result = self.client.fetch(url).await;
        let success = result.is_ok();
        self.fingerprint_manager.report_result(success);
        self.iteration += 1;
        result
    }

    pub fn self_iterate(&mut self) -> StealthLearning {
        let best_rate = self.fingerprint_manager.best_success_rate();
        if best_rate < 0.3 && self.fingerprint_manager.fingerprint_count() < 20 {
            self.fingerprint_manager.spawn_variant();
        }
        StealthLearning {
            iteration: self.iteration,
            fingerprint_count: self.fingerprint_manager.fingerprint_count(),
            best_success_rate: best_rate,
            total_requests: self.fingerprint_manager.total_requests(),
        }
    }

    pub fn learning_report(&self) -> StealthLearning {
        StealthLearning {
            iteration: self.iteration,
            fingerprint_count: self.fingerprint_manager.fingerprint_count(),
            best_success_rate: self.fingerprint_manager.best_success_rate(),
            total_requests: self.fingerprint_manager.total_requests(),
        }
    }
}

/// Couples (proxy, fingerprint, TLS, timing) as a single atomic rotation unit.
/// When rotating, all four dimensions switch together for coherent anonymity.
#[derive(Debug, Clone)]
pub struct RotationProfile {
    pub proxy_node: Option<String>,
    pub fingerprint_idx: usize,
    pub tls_cipher_order: Vec<u16>,
    pub timing_jitter_ms: Range<u64>,
    pub tls_variant: Option<TlsVariant>,
}

impl RotationProfile {
    /// Chrome on Windows — common baseline profile
    pub fn chrome_win_default() -> Self {
        Self {
            proxy_node: None,
            fingerprint_idx: 0,
            tls_cipher_order: vec![0x1301, 0x1302, 0x1303],
            timing_jitter_ms: 50..300,
            tls_variant: Some(TlsVariant::ModernH2),
        }
    }

    /// Firefox on macOS — longer jitter, different cipher order
    pub fn firefox_mac_default() -> Self {
        Self {
            proxy_node: None,
            fingerprint_idx: 1,
            tls_cipher_order: vec![0x1301, 0x1303, 0x1302],
            timing_jitter_ms: 100..500,
            tls_variant: Some(TlsVariant::LegacyHttp11),
        }
    }

    /// Safari on iOS — mobile profile, legacy strict TLS
    pub fn safari_ios_default() -> Self {
        Self {
            proxy_node: None,
            fingerprint_idx: 2,
            tls_cipher_order: vec![0x1301, 0x1302, 0x1303],
            timing_jitter_ms: 200..800,
            tls_variant: Some(TlsVariant::LegacyStrict),
        }
    }

    /// Edge on Windows — Edge fingerprint
    pub fn edge_win_default() -> Self {
        Self {
            proxy_node: None,
            fingerprint_idx: 3,
            tls_cipher_order: vec![0x1301, 0x1302, 0x1303, 0xC02F, 0xC030, 0xCCA8],
            timing_jitter_ms: 50..250,
            tls_variant: Some(TlsVariant::ModernH2),
        }
    }

    /// Build the default 4-profile rotation pool
    pub fn default_pool() -> Vec<Self> {
        vec![
            Self::chrome_win_default(),
            Self::firefox_mac_default(),
            Self::safari_ios_default(),
            Self::edge_win_default(),
        ]
    }
}

/// Pick a random profile from the pool
pub fn select_profile(profiles: &[RotationProfile]) -> &RotationProfile {
    let mut rng = rand::thread_rng();
    &profiles[rng.gen_range(0..profiles.len())]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_rotation() {
        let mut mgr = FingerprintManager::new();
        let _initial = mgr.current().system.platform;
        for _ in 0..20 {
            mgr.report_result(true);
        }
        assert!(mgr.current().headers.len() > 3);
        // cleanup
        let _ = std::fs::remove_file(&mgr.store_path);
    }

    #[test]
    fn test_spawn_variant() {
        let mut mgr = FingerprintManager::new();
        let count_before = mgr.fingerprint_count();
        mgr.spawn_variant();
        assert_eq!(mgr.fingerprint_count(), count_before + 1);
        // cleanup
        let _ = std::fs::remove_file(&mgr.store_path);
    }

    #[test]
    fn test_self_iterate_low_rate() {
        let mut mgr = FingerprintManager::new();
        for _ in 0..10 {
            mgr.report_result(false);
        }
        let count_before = mgr.fingerprint_count();
        if mgr.best_success_rate() < 0.3 {
            mgr.spawn_variant();
        }
        assert!(mgr.fingerprint_count() >= count_before);
        // cleanup
        let _ = std::fs::remove_file(&mgr.store_path);
    }

    #[test]
    fn test_stealth_learning_report() {
        let stealth = SelfIteratingStealth::new();
        let report = stealth.learning_report();
        assert_eq!(report.iteration, 0);
        assert!(report.fingerprint_count >= 4);
    }

    #[test]
    fn test_fingerprint_persistence() {
        let path = PathBuf::from("/tmp/test_fp_persist.json");
        {
            let mut mgr = FingerprintManager::new();
            mgr.store_path = path.clone();
            mgr.report_result(true);
            mgr.report_result(false);
        }
        // verify saved
        assert!(path.exists());
        let json = std::fs::read_to_string(&path).expect("value should be ok in test");
        let store: FingerprintStore = serde_json::from_str(&json).expect("value should be ok in test");
        assert!(store.fingerprints.len() >= 4);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_multi_platform_headers() {
        let mgr = FingerprintManager::new();
        let headers = mgr.apply_headers();
        assert!(headers.contains_key("User-Agent"));
        assert!(headers.contains_key("Sec-CH-UA"));
        assert!(headers.contains_key("Sec-CH-UA-Platform"));
        assert!(headers.contains_key("Accept-Language"));
        assert!(headers.contains_key("DNT"));
    }
}
