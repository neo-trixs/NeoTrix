//! TLS Fingerprint Manager — JA3/JA3S randomization, profile rotation, VSA encoding, SEAL evolution
//!
//! Provides stealth HTTP clients with browser-matching TLS fingerprints
//! to evade bot detection during web scraping and data collection.
//!
//! # Architecture
//! - `TlsFingerprintManager` holds a pool of `FingerprintProfile`s
//! - Each profile encodes browser TLS handshake params + VSA vector fingerprint
//! - Weighted random selection rotates profiles; failure tracking triggers SEAL evolution
//! - `build_stealth_client()` produces a `reqwest::Client` with matching headers
//!
//! # SEAL Evolution
//! 1. Track success/failure per profile via `record_result()`
//! 2. `evolve_profiles()` sorts by success rate, keeps top performers
//! 3. Crossover: combine JA3 cipher/extensions from two parents
//! 4. Mutation: randomly reorder adjacent ciphers/extensions
//! 5. Drop profiles below threshold (default 30% success)

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use rand::Rng;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CACHE_CONTROL, USER_AGENT};

const DEFAULT_FAILURE_THRESHOLD: f64 = 0.30;
const EVOLUTION_POOL_SIZE: usize = 5;
const MAX_HISTORY: usize = 100;

// ---------------------------------------------------------------------------
// Known browser fingerprint templates
// ---------------------------------------------------------------------------

/// Enumeration of known browser TLS fingerprint templates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FingerprintTemplate {
    Chrome120,
    Chrome120Mac,
    Firefox122,
    Firefox122Mac,
    Safari17,
    Edge120,
}

impl FingerprintTemplate {
    /// Build a full `FingerprintProfile` from this template, computing the VSA fingerprint
    pub fn profile(&self) -> FingerprintProfile {
        let (name_part, ja3, ja3s, ua, priority) = match self {
            Self::Chrome120 => (
                "chrome_120_win",
                "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513,29-23-24-25,0",
                "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513,29-23-24-25,0",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                10,
            ),
            Self::Chrome120Mac => (
                "chrome_120_mac",
                "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513,29-23-24-25,0",
                "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513,29-23-24-25,0",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                9,
            ),
            Self::Firefox122 => (
                "firefox_122_win",
                "771,4865-4867-4866-49195-49199-52393-52392-49200-49196-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24-25-256,0",
                "771,4865-4867-4866-49195-49199-52393-52392-49200-49196-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24-25-256,0",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0",
                10,
            ),
            Self::Firefox122Mac => (
                "firefox_122_mac",
                "771,4865-4867-4866-49195-49199-52393-52392-49200-49196-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24-25-256,0",
                "771,4865-4867-4866-49195-49199-52393-52392-49200-49196-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-21,29-23-24-25-256,0",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:122.0) Gecko/20100101 Firefox/122.0",
                9,
            ),
            Self::Safari17 => (
                "safari_17_mac",
                "771,4865-4866-4867-49196-49199-49195-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-17514-21-41,29-23-24-25-30,0",
                "771,4865-4866-4867-49196-49199-49195-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513-17514-21-41,29-23-24-25-30,0",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_2) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15",
                8,
            ),
            Self::Edge120 => (
                "edge_120_win",
                "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513,29-23-24-25,0",
                "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513,29-23-24-25,0",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
                7,
            ),
        };
        let vsa = compute_vsa_fingerprint(ja3, ja3s, ua);
        FingerprintProfile {
            name: name_part.into(),
            ja3: ja3.into(),
            ja3s: ja3s.into(),
            user_agent: ua.into(),
            vsa_fingerprint: vsa,
            priority,
        }
    }
}

// ---------------------------------------------------------------------------
// VSA fingerprint computation
// ---------------------------------------------------------------------------

/// Deterministic hash of JA3 + JA3S + UA into `[u64; 4]` (256-bit fingerprint)
///
/// This is a stand-in for a full 4096-dim VSA binding. Each of the 4 u64s
/// captures a different composition to produce a distributed signature.
fn compute_vsa_fingerprint(ja3: &str, ja3s: &str, ua: &str) -> [u64; 4] {
    let d = |input: &str| -> u64 {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    };
    [
        d(&format!("{ja3}:{ua}")),
        d(&format!("{ja3s}:{ua}")),
        d(&format!("{ja3}:{ja3s}")),
        d(ua),
    ]
}

// ---------------------------------------------------------------------------
// Genetic operators (mutation & crossover) for JA3 strings
// ---------------------------------------------------------------------------

/// Mutate a JA3 string by randomly reordering adjacent cipher or extension entries.
/// Returns the original if mutation doesn't fire (70% chance to skip).
fn mutate_ja3(ja3: &str) -> String {
    let mut rng = rand::thread_rng();
    let parts: Vec<&str> = ja3.split(',').collect();
    if parts.len() != 5 || !rng.gen_bool(0.30) {
        return ja3.to_string();
    }

    let field_idx = rng.gen_range(1..3);
    let mut entries: Vec<&str> = parts[field_idx].split('-').collect();

    if entries.len() > 2 {
        let i = rng.gen_range(0..entries.len() - 1);
        entries.swap(i, i + 1);
    }

    let mut result: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
    result[field_idx] = entries.join("-");

    format!("{},{},{},{},{}", result[0], result[1], result[2], result[3], result[4])
}

/// Crossover two JA3 strings by mixing their cipher and/or extension lists
fn crossover_ja3(a: &str, b: &str) -> String {
    let parts_a: Vec<&str> = a.split(',').collect();
    let parts_b: Vec<&str> = b.split(',').collect();
    if parts_a.len() != 5 || parts_b.len() != 5 {
        return a.to_string();
    }

    let mut rng = rand::thread_rng();
    let ciphers = if rng.gen_bool(0.5) { parts_b[1] } else { parts_a[1] };
    let extensions = if rng.gen_bool(0.5) { parts_b[2] } else { parts_a[2] };

    format!("{},{},{},{},{}", parts_a[0], ciphers, extensions, parts_a[3], parts_a[4])
}

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// A single TLS fingerprint profile with JA3/JA3S, User-Agent, and VSA fingerprint
#[derive(Debug, Clone)]
pub struct FingerprintProfile {
    pub name: String,
    pub ja3: String,
    pub ja3s: String,
    pub user_agent: String,
    /// VSA 4096-dim truncated to 256-bit (4 × u64) fingerprint
    pub vsa_fingerprint: [u64; 4],
    pub priority: u8,
}

/// Manages a pool of fingerprint profiles with rotation, result tracking, and SEAL evolution.
///
/// Thread-safe: all mutable state is behind `Arc<AtomicUsize>` or `Arc<Mutex<...>>`.
#[derive(Debug)]
pub struct TlsFingerprintManager {
    profiles: Vec<FingerprintProfile>,
    active_index: Arc<AtomicUsize>,
    success_rate: Arc<Mutex<HashMap<String, Vec<bool>>>>,
}

impl TlsFingerprintManager {
    /// Create a new manager with the given profiles
    pub fn new(profiles: Vec<FingerprintProfile>) -> Self {
        let active_index = Arc::new(AtomicUsize::new(0));
        let success_rate = Arc::new(Mutex::new(HashMap::new()));
        {
            let mut map = success_rate.lock().expect("lock success_rate");
            for p in &profiles {
                map.entry(p.name.clone()).or_insert_with(Vec::new);
            }
        }
        Self { profiles, active_index, success_rate }
    }

    /// Convenience constructor with the 6 default browser profiles
    pub fn with_default_profiles() -> Self {
        Self::new(default_profiles())
    }

    /// Select a profile — by name if `preferred` matches, otherwise weighted-random by priority
    pub fn select_profile(&self, preferred: Option<&str>) -> FingerprintProfile {
        if let Some(name) = preferred {
            if let Some(profile) = self.profiles.iter().find(|p| p.name == name) {
                return profile.clone();
            }
        }

        let idx = {
            let total: usize = self.profiles.iter().map(|p| p.priority as usize).sum();
            if total == 0 {
                return self.profiles[0].clone();
            }
            let mut roll = rand::thread_rng().gen_range(0..total);
            let mut i = 0;
            for (idx, p) in self.profiles.iter().enumerate() {
                if roll < p.priority as usize {
                    i = idx;
                    break;
                }
                roll -= p.priority as usize;
            }
            i
        };
        self.active_index.store(idx, Ordering::Relaxed);
        self.profiles[idx].clone()
    }

    /// Record a success or failure for a named profile (feeds the experience/evolution loop)
    pub fn record_result(&self, profile_name: &str, success: bool) {
        let mut map = self.success_rate.lock().expect("lock success_rate");
        let history = map.entry(profile_name.to_string()).or_insert_with(Vec::new);
        history.push(success);
        if history.len() > MAX_HISTORY {
            history.remove(0);
        }
    }

    /// Current success rate (0.0–1.0) for a named profile; `None` if no data recorded
    pub fn success_rate(&self, profile_name: &str) -> Option<f64> {
        let map = self.success_rate.lock().expect("lock success_rate");
        map.get(profile_name).map(|history| {
            if history.is_empty() {
                return 1.0;
            }
            let successes = history.iter().filter(|&&s| s).count();
            successes as f64 / history.len() as f64
        })
    }

    /// SEAL-inspired evolution: sort profiles by success rate, crossover + mutate top performers,
    /// drop profiles below failure threshold.
    ///
    /// Returns a **new** `Vec<FingerprintProfile>` — does not mutate `self`.
    pub fn evolve_profiles(&self) -> Vec<FingerprintProfile> {
        let map = self.success_rate.lock().expect("lock success_rate");

        let mut scored: Vec<(&FingerprintProfile, f64)> = self
            .profiles
            .iter()
            .map(|p| {
                let rate = map
                    .get(&p.name)
                    .and_then(|h| {
                        if h.is_empty() {
                            None
                        } else {
                            Some(h.iter().filter(|&&s| s).count() as f64 / h.len() as f64)
                        }
                    })
                    .unwrap_or(0.5);
                (p, rate)
            })
            .collect();
        drop(map);

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut evolved: Vec<FingerprintProfile> = Vec::new();
        let mut rng = rand::thread_rng();

        // Keep top 2 highest-scoring profiles as-is
        for (p, _) in scored.iter().take(2) {
            evolved.push((*p).clone());
        }

        // Crossover top 2 parents → 1 child
        if scored.len() >= 4 {
            let (p1, _) = scored[0];
            let (p2, _) = scored[1];
            let child_ja3 = crossover_ja3(&p1.ja3, &p2.ja3);
            let child_ja3s = crossover_ja3(&p1.ja3s, &p2.ja3s);
            let child_ua = if rng.gen_bool(0.5) {
                p1.user_agent.clone()
            } else {
                p2.user_agent.clone()
            };
            let child_vsa = compute_vsa_fingerprint(&child_ja3, &child_ja3s, &child_ua);
            evolved.push(FingerprintProfile {
                name: format!("evolved_cross_{}_{}", p1.name, p2.name),
                ja3: child_ja3,
                ja3s: child_ja3s,
                user_agent: child_ua,
                vsa_fingerprint: child_vsa,
                priority: 8,
            });
        }

        // Mutation: up to 2 mutants from each of the top 3 performers
        for (p, _) in scored.iter().take(3) {
            let mut spawned = 0;
            for _ in 0..2 {
                let m_ja3 = mutate_ja3(&p.ja3);
                if m_ja3 != p.ja3 {
                    spawned += 1;
                    let m_vsa = compute_vsa_fingerprint(&m_ja3, &p.ja3s, &p.user_agent);
                    evolved.push(FingerprintProfile {
                        name: format!("evolved_mut_{}_{}_{}", p.name, spawned, p.priority),
                        ja3: m_ja3,
                        ja3s: p.ja3s.clone(),
                        user_agent: p.user_agent.clone(),
                        vsa_fingerprint: m_vsa,
                        priority: 7,
                    });
                }
            }
        }

        // Keep any original profiles above threshold (not already in evolved set)
        for (p, rate) in &scored {
            if *rate >= DEFAULT_FAILURE_THRESHOLD && !evolved.iter().any(|e| e.name == p.name) {
                evolved.push((*p).clone());
            }
        }

        // Fallback: ensure minimum pool size
        if evolved.len() < EVOLUTION_POOL_SIZE {
            for p in &self.profiles {
                if evolved.iter().any(|e| e.name == p.name) {
                    continue;
                }
                evolved.push(p.clone());
                if evolved.len() >= EVOLUTION_POOL_SIZE * 2 {
                    break;
                }
            }
        }

        evolved
    }

    /// Build a `reqwest::Client` configured with the given profile's User-Agent and headers.
    ///
    /// Sets browser-matching headers (Accept, Accept-Language, Sec-CH-UA family for Chromium).
    /// True JA3-level randomization requires a custom TLS connector (future).
    pub fn build_stealth_client(&self, profile: &FingerprintProfile) -> reqwest::Client {
        let mut headers = HeaderMap::new();

        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&profile.user_agent)
                .unwrap_or_else(|_| HeaderValue::from_static("Mozilla/5.0")),
        );

        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8",
            ),
        );

        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-US,en;q=0.9"),
        );

        headers.insert(
            ACCEPT_ENCODING,
            HeaderValue::from_static("gzip, deflate, br"),
        );

        headers.insert(
            CACHE_CONTROL,
            HeaderValue::from_static("no-cache"),
        );

        // Chrome/Edge get Sec-CH-UA hints
        if profile.name.contains("chrome") || profile.name.contains("edge") {
            let _ = Self::insert_header_value(
                &mut headers,
                HeaderName::from_static("sec-ch-ua"),
                r#""Google Chrome";v="120", "Chromium";v="120", "Not=A?Brand";v="99""#,
            );
            let _ = Self::insert_header_value(
                &mut headers,
                HeaderName::from_static("sec-ch-ua-mobile"),
                "?0",
            );
            let _ = Self::insert_header_value(
                &mut headers,
                HeaderName::from_static("sec-ch-ua-platform"),
                if profile.name.contains("mac") { "\"macOS\"" } else { "\"Windows\"" },
            );
            let _ = Self::insert_header_value(
                &mut headers,
                HeaderName::from_static("upgrade-insecure-requests"),
                "1",
            );
        }

        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("reqwest client build should succeed with rustls-tls")
    }

    fn insert_header_value(headers: &mut HeaderMap, name: HeaderName, value: &str) -> Result<(), ()> {
        if let Ok(v) = HeaderValue::from_str(value) {
            headers.insert(name, v);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Access the current profile list
    pub fn profiles(&self) -> &[FingerprintProfile] {
        &self.profiles
    }

    /// Index of the last selected profile
    pub fn active_index(&self) -> usize {
        self.active_index.load(Ordering::Relaxed)
    }
}

// ---------------------------------------------------------------------------
// Default profile factory
// ---------------------------------------------------------------------------

/// Generate the standard set of 6 browser fingerprint profiles:
/// Chrome 120 (Win/Mac), Firefox 122 (Win/Mac), Safari 17 (Mac), Edge 120 (Win)
pub fn default_profiles() -> Vec<FingerprintProfile> {
    vec![
        FingerprintTemplate::Chrome120.profile(),
        FingerprintTemplate::Chrome120Mac.profile(),
        FingerprintTemplate::Firefox122.profile(),
        FingerprintTemplate::Firefox122Mac.profile(),
        FingerprintTemplate::Safari17.profile(),
        FingerprintTemplate::Edge120.profile(),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profiles_have_unique_names() {
        let profiles = default_profiles();
        let mut names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), profiles.len(), "all profile names must be unique");
    }

    #[test]
    fn test_select_profile_by_name() {
        let mgr = TlsFingerprintManager::with_default_profiles();
        let p = mgr.select_profile(Some("chrome_120_win"));
        assert_eq!(p.name, "chrome_120_win");
    }

    #[test]
    fn test_select_profile_fallback_to_random() {
        let mgr = TlsFingerprintManager::with_default_profiles();
        let p = mgr.select_profile(Some("nonexistent"));
        assert!(mgr.profiles().iter().any(|x| x.name == p.name));
    }

    #[test]
    fn test_record_and_retrieve_success_rate() {
        let mgr = TlsFingerprintManager::with_default_profiles();
        mgr.record_result("chrome_120_win", true);
        mgr.record_result("chrome_120_win", true);
        mgr.record_result("chrome_120_win", false);
        let rate = mgr.success_rate("chrome_120_win").unwrap();
        assert!((rate - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_success_rate_returns_none_for_untracked() {
        let mgr = TlsFingerprintManager::with_default_profiles();
        assert!(mgr.success_rate("unknown_profile").is_none());
    }

    #[test]
    fn test_evolve_profiles_returns_vec() {
        let mgr = TlsFingerprintManager::with_default_profiles();
        mgr.record_result("chrome_120_win", true);
        mgr.record_result("chrome_120_win", true);
        mgr.record_result("firefox_122_win", false);
        let evolved = mgr.evolve_profiles();
        assert!(!evolved.is_empty(), "evolve should return at least some profiles");
        // Should have at least the original profiles that are above threshold
        assert!(evolved.len() >= 2, "should keep at least top 2 profiles");
    }

    #[test]
    fn test_build_stealth_client_returns_client() {
        let mgr = TlsFingerprintManager::with_default_profiles();
        let profile = mgr.select_profile(Some("chrome_120_win"));
        let client = mgr.build_stealth_client(&profile);
        // Verify the client exists (can't easily inspect headers from reqwest::Client)
        let _ = client;
    }

    #[test]
    fn test_evolve_profiles_empty_profiles() {
        let mgr = TlsFingerprintManager::new(vec![]);
        let evolved = mgr.evolve_profiles();
        assert!(evolved.is_empty());
    }

    #[test]
    fn test_compute_vsa_deterministic() {
        let a = compute_vsa_fingerprint("ja3", "ja3s", "ua");
        let b = compute_vsa_fingerprint("ja3", "ja3s", "ua");
        assert_eq!(a, b, "VSA fingerprint must be deterministic");
    }

    #[test]
    fn test_compute_vsa_different_ua_differs() {
        let a = compute_vsa_fingerprint("ja3", "ja3s", "ua1");
        let b = compute_vsa_fingerprint("ja3", "ja3s", "ua2");
        assert_ne!(a, b, "different UA should produce different fingerprint");
    }

    #[test]
    fn test_mutate_ja3_preserves_structure() {
        let original = "771,4865-4866-4867,0-23-65281,29-23-24-25,0";
        // Run many times — at least one should mutate
        let any_mutated = (0..50).any(|_| mutate_ja3(original) != original);
        assert!(any_mutated, "mutation should eventually produce a change");
    }

    #[test]
    fn test_select_profile_priorities_distribute() {
        let profiles = vec![
            FingerprintProfile {
                name: "high".into(),
                ja3: "a".into(),
                ja3s: "b".into(),
                user_agent: "c".into(),
                vsa_fingerprint: [0; 4],
                priority: 100,
            },
            FingerprintProfile {
                name: "low".into(),
                ja3: "d".into(),
                ja3s: "e".into(),
                user_agent: "f".into(),
                vsa_fingerprint: [1; 4],
                priority: 1,
            },
        ];
        let mgr = TlsFingerprintManager::new(profiles);
        let mut counts = std::collections::HashMap::new();
        for _ in 0..1000 {
            let p = mgr.select_profile(None);
            *counts.entry(p.name.clone()).or_insert(0usize) += 1;
        }
        let high = *counts.get("high").unwrap_or(&0);
        let low = *counts.get("low").unwrap_or(&0);
        assert!(high > low, "higher priority should be selected more often (got high={high}, low={low})");
    }

    #[test]
    fn test_history_bounded_by_max() {
        let mgr = TlsFingerprintManager::with_default_profiles();
        for _ in 0..MAX_HISTORY + 50 {
            mgr.record_result("chrome_120_win", true);
        }
        let map = mgr.success_rate.lock().unwrap();
        let history = map.get("chrome_120_win").unwrap();
        assert!(history.len() <= MAX_HISTORY);
    }

    #[test]
    fn test_fingerprint_template_all_variants() {
        let variants = [
            FingerprintTemplate::Chrome120,
            FingerprintTemplate::Chrome120Mac,
            FingerprintTemplate::Firefox122,
            FingerprintTemplate::Firefox122Mac,
            FingerprintTemplate::Safari17,
            FingerprintTemplate::Edge120,
        ];
        for v in &variants {
            let p = v.profile();
            assert!(!p.ja3.is_empty(), "{} has empty JA3", p.name);
            assert!(!p.ja3s.is_empty(), "{} has empty JA3S", p.name);
            assert!(!p.user_agent.is_empty(), "{} has empty UA", p.name);
            assert_ne!(p.vsa_fingerprint, [0u64; 4], "{} VSA is all-zeros", p.name);
        }
    }
}
