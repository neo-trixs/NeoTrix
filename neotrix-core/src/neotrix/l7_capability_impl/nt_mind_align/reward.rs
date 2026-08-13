//! nt_mind::align::reward — 偏好对奖励校准
//!
//! 节点: nt_mind::align::reward (L0)
//! Provides: preference_alignment, reward_calibration

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

/// 偏好对: (chosen, rejected)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PreferencePair {
    pub chosen: String,
    pub rejected: String,
}

/// 奖励校准器 — 按偏好对更新奖励偏置 (DPO 风格隐式奖励)
#[derive(Debug, Clone, Default)]
pub struct RewardCalibrator {
    pairs: Vec<PreferencePair>,
    bias: f32,
    updates: u64,
}

impl RewardCalibrator {
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录偏好对并更新偏置 (chosen 与 rejected 差异贡献校准信号)
    pub fn add_pair(&mut self, chosen: &str, rejected: &str) -> Result<(), NeoTrixError> {
        if chosen.is_empty() || rejected.is_empty() {
            return Err(NeoTrixError::InvalidInput("偏好对不能为空".into()));
        }
        if chosen == rejected {
            return Err(NeoTrixError::InvalidInput(
                "chosen 与 rejected 不能相同".into(),
            ));
        }
        let sig = (chosen.len() as f32 - rejected.len() as f32)
            / (chosen.len().max(rejected.len()).max(1) as f32);
        self.bias += sig * 0.1;
        self.pairs.push(PreferencePair {
            chosen: chosen.into(),
            rejected: rejected.into(),
        });
        self.updates += 1;
        Ok(())
    }

    pub fn bias(&self) -> f32 {
        self.bias
    }

    pub fn pair_count(&self) -> usize {
        self.pairs.len()
    }

    pub fn updates(&self) -> u64 {
        self.updates
    }
}

impl CapabilityNode for RewardCalibrator {
    fn node_id(&self) -> &str {
        "nt_mind::align::reward"
    }
    fn provides(&self) -> Vec<String> {
        vec!["preference_alignment".into(), "reward_calibration".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec![]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Indigo]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for RewardCalibrator {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut r = RewardCalibrator::new();
        r.add_pair("long correct answer", "short")
            .map_err(|e| vec![e.to_string()])?;
        assert_eq!(r.pair_count(), 1);
        assert!(r.bias() > 0.0, "chosen 更长应推高偏置");
        assert!(r.add_pair("same", "same").is_err());
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_mind_align_reward"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_pair_updates_bias() {
        let mut r = RewardCalibrator::new();
        r.add_pair("chosen answer", "rejected").unwrap();
        assert_eq!(r.pair_count(), 1);
        assert!(r.bias() != 0.0);
    }

    #[test]
    fn test_identical_pair_rejected() {
        let mut r = RewardCalibrator::new();
        assert!(r.add_pair("x", "x").is_err());
    }

    #[test]
    fn test_empty_pair_rejected() {
        let mut r = RewardCalibrator::new();
        assert!(r.add_pair("", "y").is_err());
    }

    #[test]
    fn test_updates_counter() {
        let mut r = RewardCalibrator::new();
        r.add_pair("a", "b").unwrap();
        r.add_pair("c", "d").unwrap();
        assert_eq!(r.updates(), 2);
    }
}
