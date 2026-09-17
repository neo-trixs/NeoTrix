use std::sync::{Mutex, OnceLock};

use super::AffectiveFeedback;

/// 情感观测旁路槽 (Q2 P3, 设计: docs/1-DESIGN/affective-reward-context.md)。
///
/// 跨域数据流桥梁: NT-IO 数字人 (process_audio_input) 把最近一次情感观测写入,
/// NT-MIND SEAL RewardCalculationStage 读取消费 (经 RewardSource::External 引导)。
/// 语义: 单槽覆盖 (latest-wins) + 消费即取走 (take) — 与 `global_shield` 全局共享先例一致。
static SLOT: OnceLock<Mutex<Option<AffectiveFeedback>>> = OnceLock::new();

fn slot() -> &'static Mutex<Option<AffectiveFeedback>> {
    SLOT.get_or_init(|| Mutex::new(None))
}

/// 写入最近一次情感观测 (数字人旁路事件)。
pub fn publish(fb: Option<AffectiveFeedback>) {
    if let Ok(mut guard) = slot().lock() {
        *guard = fb;
    }
}

/// 读取并取走最近一次情感观测 (SEAL 奖励计算消费)。
pub fn take() -> Option<AffectiveFeedback> {
    slot().lock().ok().and_then(|mut guard| guard.take())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_take_roundtrip() {
        let fb = AffectiveFeedback {
            valence: 0.9,
            arousal: 0.2,
            stage: 4,
            interactions: 10,
            signal_weight: 0.3,
        };
        publish(Some(fb));
        let got = take().expect("consumer receives published observation");
        assert!((got.valence - 0.9).abs() < 1e-9);
        assert_eq!(got.stage, 4);
        assert_eq!(got.interactions, 10);
        // 消费即取走 — 二次读取为空
        assert!(take().is_none());
    }

    #[test]
    fn test_take_empty_returns_none() {
        publish(None);
        assert!(take().is_none());
    }

    #[test]
    fn test_publish_none_clears_slot() {
        let fb = AffectiveFeedback {
            valence: 0.5,
            arousal: 0.5,
            stage: 2,
            interactions: 5,
            signal_weight: 0.2,
        };
        publish(Some(fb));
        publish(None);
        assert!(take().is_none());
    }
}