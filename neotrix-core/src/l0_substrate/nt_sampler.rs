//! 采样器 — 温度 / top-k / top-p / 重复惩罚 的确定性实现。
//!
//! 公理：给定 seed 与 logits，采样结果**完全可复现**（无 wall-clock、无全局
//! RNG 状态、无 IO）。`TinyRng` 是 xorshift64*，刻意不引 `rand`——L0 基座不应
//! 为一个 90 行算法拖进外部依赖。
//!
//! 萃取说明（2026-09-28）：来自 `crates/neotrix-decision-engine` 的
//! `src/model/nt_gen_model.rs:468-621`（该 crate 已归档至
//! `~/Downloads/Neo/neotrix-archive/crates/neotrix-decision-engine/`）。
//! ⚠️ 源文件**从未参与编译**——批量抢救提交加了文件却漏了 `mod` 声明，
//! 且其 `Cargo.toml` 无 `[features]` 段却用 `cfg(feature="metal")`。
//! 这是「入库 ≠ 编译」，与 layer-map `_rule` 的「导出 ≠ 调用」同源。
//!
//! 为什么落 L0：主代码此前**只有采样参数、没有采样实现**——
//! `repetition_penalty` / `top_k` / `top_p` 全部是 `nt_io_inference` 里的
//! config 透传字段，从不在本地 logits 上生效；唯一类别采样器
//! `neotrix/nt_crystal_core/ctm.rs` 用 `rand::random()`，非确定、无 top-k、
//! 无 top-p、无重复惩罚。本模块补的就是这个空缺。

/// 采样参数（MiniMind Pattern 8 映射）
#[derive(Debug, Clone)]
pub struct NtSampleParams {
    pub temperature: f64,
    pub top_k: Option<usize>,
    pub top_p: Option<f64>,
    pub repetition_penalty: f32,
    pub seed: u64,
}

impl Default for NtSampleParams {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            top_k: Some(40),
            top_p: Some(0.9),
            repetition_penalty: 1.1,
            seed: 0x243F_6A88_85A3_08D3,
        }
    }
}

/// 小确定性 RNG（xorshift64*，免 rand 依赖）
struct TinyRng(u64);

impl TinyRng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn next_f64(&mut self) -> f64 {
        const DIV: f64 = (u64::MAX as f64) + 1.0;
        self.next_u64() as f64 / DIV
    }
}

pub struct NtSampler {
    params: NtSampleParams,
    rng: TinyRng,
}

impl NtSampler {
    pub fn new(params: NtSampleParams) -> Self {
        let seed = params.seed;
        Self {
            params,
            rng: TinyRng(seed),
        }
    }

    /// 采样：greedy（temp<=0）或 温度/top-k/top-p/重复惩罚
    pub fn sample(&mut self, logits: &[f32], seen: &[u32]) -> u32 {
        if logits.is_empty() {
            return 0;
        }
        if self.params.temperature <= 0.0 {
            return argmax(logits);
        }
        let mut adjusted: Vec<f32> = logits.to_vec();
        if (self.params.repetition_penalty - 1.0).abs() > f32::EPSILON {
            for s in seen {
                if let Some(v) = adjusted.get_mut(*s as usize) {
                    if *v > 0.0 {
                        *v /= self.params.repetition_penalty;
                    } else {
                        *v *= self.params.repetition_penalty;
                    }
                }
            }
        }
        let temp = self.params.temperature.max(1e-6) as f32;
        let mut probs: Vec<f32> = adjusted.iter().map(|l| l / temp).collect();
        // top-k：只留前 k（并列截断不影响正确性：多留的进不了采样即弃）
        if let Some(k) = self.params.top_k {
            if k < probs.len() {
                let mut idx: Vec<usize> = (0..probs.len()).collect();
                idx.sort_by(|a, b| {
                    probs[*b]
                        .partial_cmp(&probs[*a])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                for (i, p) in probs.iter_mut().enumerate() {
                    if !idx[..k.min(idx.len())].contains(&i) {
                        *p = f32::NEG_INFINITY;
                    }
                }
            }
        }
        // softmax
        let max = probs
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        let mut sum = 0.0f32;
        for p in probs.iter_mut() {
            *p = (*p - max).exp();
            sum += *p;
        }
        if sum > 0.0 {
            for p in probs.iter_mut() {
                *p /= sum;
            }
        }
        // top-p
        let mut order: Vec<usize> = (0..probs.len()).collect();
        order.sort_by(|a, b| {
            probs[*b]
                .partial_cmp(&probs[*a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut allowed = vec![false; probs.len()];
        if let Some(top_p) = self.params.top_p {
            let mut cum = 0.0f32;
            for i in &order {
                allowed[*i] = true;
                cum += probs[*i];
                if cum >= top_p as f32 {
                    break;
                }
            }
        } else {
            for i in &order {
                allowed[*i] = true;
            }
        }
        // 按过滤后分布采样
        let total: f32 = order.iter().filter(|i| allowed[**i]).map(|i| probs[*i]).sum();
        let mut r = (self.rng.next_f64() as f32) * total;
        for i in &order {
            if !allowed[*i] {
                continue;
            }
            r -= probs[*i];
            if r <= 0.0 {
                return *i as u32;
            }
        }
        order.first().copied().unwrap_or(0) as u32
    }
}

fn argmax(logits: &[f32]) -> u32 {
    let mut best = 0usize;
    for (i, v) in logits.iter().enumerate() {
        if *v > logits[best] {
            best = i;
        }
    }
    best as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampler_greedy_and_penalty() {
        let greedy = NtSampleParams {
            temperature: 0.0,
            top_k: None,
            top_p: None,
            repetition_penalty: 1.0,
            seed: 7,
        };
        assert_eq!(NtSampler::new(greedy.clone()).sample(&[1.0, 5.0, 3.0], &[]), 1);
        // 重复惩罚翻转 argmax（top_k=1 时采样退化为受罚 argmax，种子无关）
        let penalized = NtSampleParams {
            temperature: 1.0,
            top_k: Some(1),
            top_p: None,
            repetition_penalty: 2.0,
            seed: 7,
        };
        assert_eq!(
            NtSampler::new(penalized).sample(&[5.0, 4.0], &[0]),
            1,
            "token0 被惩罚后应选 token1"
        );
        // top-p 截断：尖峰分布只剩首位
        let nucleus = NtSampleParams {
            temperature: 1.0,
            top_k: None,
            top_p: Some(0.5),
            repetition_penalty: 1.0,
            seed: 7,
        };
        assert_eq!(NtSampler::new(nucleus).sample(&[10.0, 0.0, 0.0, 0.0], &[]), 0);
        // 空 logits 兜底
        assert_eq!(NtSampler::new(greedy).sample(&[], &[]), 0);
    }

    /// 萃取补充：确定性契约（同 seed 同输入必同输出），原 crate 未覆盖这条。
    #[test]
    fn same_seed_same_output() {
        let p = NtSampleParams {
            temperature: 1.0,
            top_k: None,
            top_p: None,
            repetition_penalty: 1.0,
            seed: 42,
        };
        let logits = [0.1f32, 3.0, -1.0, 2.2, 0.7];
        let a = NtSampler::new(p.clone()).sample(&logits, &[]);
        let b = NtSampler::new(p).sample(&logits, &[]);
        assert_eq!(a, b, "同 seed 同输入必须可复现");
    }

    /// 萃取补充：越界的 seen 下标不得 panic（`get_mut` 兜底路径）。
    #[test]
    fn out_of_range_seen_is_ignored() {
        let p = NtSampleParams {
            temperature: 1.0,
            top_k: None,
            top_p: None,
            repetition_penalty: 2.0,
            seed: 1,
        };
        let _ = NtSampler::new(p).sample(&[1.0, 2.0], &[9999]);
    }
}
