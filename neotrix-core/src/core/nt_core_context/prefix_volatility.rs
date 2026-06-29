#![allow(dead_code)]
//! # PrefixVolatilityDetector — KV 缓存前缀稳定检测
//!
//! 借鉴 Kun 的 prefix-volatility.ts:
//!   不可变前缀 (system + few-shots) 字节级指纹
//!   检测前缀变异 → 重置缓存
//!   确保 KV 缓存复用率最大化

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 前缀指纹 — 用于检测不可变前缀的变异
#[derive(Debug, Clone)]
pub struct PrefixFingerprint {
    fingerprint: u64,
    content: String,
}

impl PrefixFingerprint {
    /// 从字符串计算指纹
    pub fn from_content(content: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Self {
            fingerprint: hasher.finish(),
            content: content.to_string(),
        }
    }

    /// 验证指纹是否匹配
    pub fn verify(&self, other: &str) -> bool {
        let mut hasher = DefaultHasher::new();
        other.hash(&mut hasher);
        self.fingerprint == hasher.finish()
    }
}

/// 前缀变异检测结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolatilityResult {
    Stable,   // 前缀无变化 → 缓存安全
    Volatile, // 前缀变化 → 缓存需要重置
    Unknown,  // 首次建立基线
}

/// 前缀易失性检测器
#[derive(Debug, Clone)]
pub struct PrefixVolatilityDetector {
    system_fingerprint: Option<PrefixFingerprint>,
    few_shot_fingerprint: Option<PrefixFingerprint>,
    volatility_count: u64,
    stable_count: u64,
}

impl Default for PrefixVolatilityDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PrefixVolatilityDetector {
    pub fn new() -> Self {
        Self {
            system_fingerprint: None,
            few_shot_fingerprint: None,
            volatility_count: 0,
            stable_count: 0,
        }
    }

    /// 检测 system 前缀是否稳定
    pub fn check_system(&mut self, system_prompt: &str) -> VolatilityResult {
        let fp = PrefixFingerprint::from_content(system_prompt);
        match &self.system_fingerprint {
            Some(existing) if existing.verify(system_prompt) => {
                self.stable_count += 1;
                VolatilityResult::Stable
            }
            Some(_) => {
                self.volatility_count += 1;
                self.system_fingerprint = Some(fp);
                VolatilityResult::Volatile
            }
            None => {
                self.system_fingerprint = Some(fp);
                VolatilityResult::Unknown
            }
        }
    }

    /// 检测 few-shot 前缀是否稳定
    pub fn check_few_shot(&mut self, few_shot: &str) -> VolatilityResult {
        let fp = PrefixFingerprint::from_content(few_shot);
        match &self.few_shot_fingerprint {
            Some(existing) if existing.verify(few_shot) => VolatilityResult::Stable,
            Some(_) => {
                self.volatility_count += 1;
                self.few_shot_fingerprint = Some(fp);
                VolatilityResult::Volatile
            }
            None => {
                self.few_shot_fingerprint = Some(fp);
                VolatilityResult::Unknown
            }
        }
    }

    /// 检查完整前缀是否稳定 (system + few_shot 组合)
    pub fn check_combined(&mut self, system: &str, few_shot: &str) -> VolatilityResult {
        let sys_result = self.check_system(system);
        let fs_result = self.check_few_shot(few_shot);
        if sys_result == VolatilityResult::Volatile || fs_result == VolatilityResult::Volatile {
            VolatilityResult::Volatile
        } else if sys_result == VolatilityResult::Unknown || fs_result == VolatilityResult::Unknown
        {
            VolatilityResult::Unknown
        } else {
            VolatilityResult::Stable
        }
    }

    /// 缓存是否可以被安全复用
    pub fn is_cache_safe(&self) -> bool {
        self.stable_count > 3 && self.volatility_count == 0
    }

    /// 缓存命中率估计
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.stable_count + self.volatility_count;
        if total == 0 {
            return 0.0;
        }
        if self.volatility_count == 0 {
            return 1.0; // 全命中
        }
        self.stable_count as f64 / total as f64
    }

    /// 重置
    pub fn reset(&mut self) {
        self.system_fingerprint = None;
        self.few_shot_fingerprint = None;
        self.volatility_count = 0;
        self.stable_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_unknown() {
        let mut detector = PrefixVolatilityDetector::new();
        assert_eq!(
            detector.check_system("system prompt"),
            VolatilityResult::Unknown
        );
    }

    #[test]
    fn test_same_system_prompt_is_stable() {
        let mut detector = PrefixVolatilityDetector::new();
        detector.check_system("system prompt"); // Unknown first time
        assert_eq!(
            detector.check_system("system prompt"),
            VolatilityResult::Stable
        );
    }

    #[test]
    fn test_different_system_prompt_is_volatile() {
        let mut detector = PrefixVolatilityDetector::new();
        detector.check_system("system prompt v1");
        assert_eq!(
            detector.check_system("system prompt v2"),
            VolatilityResult::Volatile
        );
    }

    #[test]
    fn test_combined_stable() {
        let mut detector = PrefixVolatilityDetector::new();
        detector.check_combined("system", "few_shot"); // Unknown
        assert_eq!(
            detector.check_combined("system", "few_shot"),
            VolatilityResult::Stable
        );
    }

    #[test]
    fn test_combined_volatile() {
        let mut detector = PrefixVolatilityDetector::new();
        detector.check_combined("system v1", "few_shot v1");
        assert_eq!(
            detector.check_combined("system v2", "few_shot v1"),
            VolatilityResult::Volatile
        );
    }

    #[test]
    fn test_cache_safety() {
        let mut detector = PrefixVolatilityDetector::new();
        assert!(!detector.is_cache_safe());
        detector.check_system("stable prompt"); // Unknown
        detector.check_system("stable prompt"); // Stable
        detector.check_system("stable prompt"); // Stable
        detector.check_system("stable prompt"); // Stable → now safe (3+ stables)
        assert!(detector.is_cache_safe());
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut detector = PrefixVolatilityDetector::new();
        assert_eq!(detector.cache_hit_rate(), 0.0);
        detector.check_system("a");
        detector.check_system("a");
        detector.check_system("a");
        assert!((detector.cache_hit_rate() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_volatility_count() {
        let mut detector = PrefixVolatilityDetector::new();
        detector.check_system("v1");
        detector.check_system("v2");
        detector.check_system("v3");
        assert_eq!(detector.volatility_count, 2);
    }

    #[test]
    fn test_reset() {
        let mut detector = PrefixVolatilityDetector::new();
        detector.check_system("prompt");
        detector.check_system("prompt");
        assert!(detector.is_cache_safe());
        detector.reset();
        assert!(!detector.is_cache_safe());
        assert_eq!(detector.volatility_count, 0);
    }

    #[test]
    fn test_different_few_shot_same_system() {
        let mut detector = PrefixVolatilityDetector::new();
        detector.check_combined("system", "fs_v1");
        let result = detector.check_combined("system", "fs_v2");
        assert_eq!(result, VolatilityResult::Volatile);
    }

    #[test]
    fn test_repeated_volatility() {
        let mut detector = PrefixVolatilityDetector::new();
        for i in 0..5 {
            detector.check_system(&format!("v{}", i));
        }
        assert_eq!(detector.volatility_count, 4);
        assert_eq!(detector.cache_hit_rate(), 0.0);
    }
}
