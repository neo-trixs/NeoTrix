// P3: BayesianExperimentDesign (吸收 arXiv 2608.09696 MDA — Model Discovery Agent)
// 机制: LLM 提议候选模型结构 → 贝叶斯机制 (SMC/SBI) 计算后验 →
// 价值信息 (VoI) 选择下一个实验; 当真实模型落在当前假设类之外 (M-open),
// predictive check 标记欠拟合并扩展假设空间。
//
// 实现要点:
// - 确定性伪随机 (i*2654435761 % 2^32)/2^32 + Box-Muller → N(0.5, noise) 候选结果
// - VoI ≈ 对候选观测结果取期望的 prior→posterior KL 散度
// - M-open predictive check: adequacy = posterior > 1/n 的假设占比
// - 假设对实验 e 的判别结构: 按 split 阈值将假设分为两组 (探针区分度 = 张力)

use serde::{Deserialize, Serialize};

const ADEQUACY_THRESHOLD: f64 = 0.75;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VoIConfig {
    pub hypotheses: usize,
    pub experiments: usize,
    pub samples_per_experiment: usize,
    pub noise: f64,
}

impl Default for VoIConfig {
    fn default() -> Self {
        Self {
            hypotheses: 8,
            experiments: 16,
            samples_per_experiment: 64,
            noise: 0.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: usize,
    pub log_likelihood: f64,
    pub prior: f64,
    pub posterior: f64,
}

impl Hypothesis {
    /// posterior = prior * exp(log_likelihood), 对给定假设集合归一化
    pub fn posterior(&self, hypotheses: &[Hypothesis]) -> f64 {
        let raw: Vec<f64> = hypotheses
            .iter()
            .map(|h| h.prior * h.log_likelihood.exp())
            .collect();
        let sum: f64 = raw.iter().sum();
        let self_raw = self.prior * self.log_likelihood.exp();
        if sum > 0.0 {
            self_raw / sum
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Experiment {
    pub id: usize,
    pub outcome: f64,
    pub information_gain: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MOpenCheck {
    pub adequacy: f64,
    pub expand_hypothesis_space: bool,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianExperimentDesign {
    pub config: VoIConfig,
    pub hypotheses: Vec<Hypothesis>,
    pub experiments: Vec<Experiment>,
}

impl BayesianExperimentDesign {
    pub fn new(config: VoIConfig, hypotheses: Vec<Hypothesis>) -> Self {
        let experiments = (0..config.experiments)
            .map(|id| Experiment {
                id,
                outcome: 0.0,
                information_gain: 0.0,
            })
            .collect();
        Self {
            config,
            hypotheses,
            experiments,
        }
    }

    /// posterior = prior * exp(log_likelihood), 再除以总和
    pub fn normalize_posteriors(&mut self) {
        let n = self.hypotheses.len();
        if n == 0 {
            return;
        }
        let raw: Vec<f64> = self
            .hypotheses
            .iter()
            .map(|h| h.prior * h.log_likelihood.exp())
            .collect();
        let sum: f64 = raw.iter().sum();
        if sum > 0.0 {
            for (h, r) in self.hypotheses.iter_mut().zip(raw.iter()) {
                h.posterior = r / sum;
            }
        }
    }

    /// 假设 h 的隐含结果预测均值 (0,1) 内, 由 id 确定性导出
    fn hypothesis_mean(&self, h: &Hypothesis) -> f64 {
        let n = self.hypotheses.len().max(1) as f64;
        (h.id as f64 + 1.0) / (n + 1.0)
    }

    /// 实验 e 的判别 split 阈值 (0,1) 内: 假设按预测均值落入两组
    fn experiment_split(&self, experiment: usize) -> f64 {
        let e = self.config.experiments.max(1) as f64;
        (experiment as f64 + 1.0) / (e + 1.0)
    }

    /// 确定性伪随机: (i*2654435761 % 2^32)/2^32 ∈ [0,1)
    fn pseudo_uniform(i: usize) -> f64 {
        let u = ((i as u64).wrapping_mul(2654435761) % (1u64 << 32)) as f64
            / (1u64 << 32) as f64;
        u.max(1e-12)
    }

    /// Box-Muller: 候选观测结果 ~ N(0.5, noise)
    fn sample_outcome(&self, i: usize) -> f64 {
        let u1 = Self::pseudo_uniform(i);
        let u2 = Self::pseudo_uniform(i.wrapping_add(1));
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        0.5 + self.config.noise * z
    }

    /// VoI ≈ E_y[ KL(prior || posterior(y)) ], 对 samples_per_experiment 个候选观测取平均
    pub fn value_of_information(&self, experiment: usize) -> f64 {
        let n = self.hypotheses.len();
        if n == 0 {
            return 0.0;
        }
        let split = self.experiment_split(experiment);
        let prior: Vec<f64> = self.hypotheses.iter().map(|h| h.posterior).collect();
        let (mut ga, mut gb) = (Vec::new(), Vec::new());
        for h in &self.hypotheses {
            let m = self.hypothesis_mean(h);
            if m < split {
                ga.push(m);
            } else {
                gb.push(m);
            }
        }
        let pred_a = if ga.is_empty() {
            split
        } else {
            ga.iter().sum::<f64>() / ga.len() as f64
        };
        let pred_b = if gb.is_empty() {
            split
        } else {
            gb.iter().sum::<f64>() / gb.len() as f64
        };
        let samples = self.config.samples_per_experiment.max(1);
        let sigma = self.config.noise.max(1e-9);
        let mut total_kl = 0.0;
        for i in 0..samples {
            let y = self.sample_outcome(i);
            let mut q = Vec::with_capacity(n);
            let mut sum = 0.0;
            for (idx, h) in self.hypotheses.iter().enumerate() {
                let m = self.hypothesis_mean(h);
                let pred = if m < split { pred_a } else { pred_b };
                let added = -0.5 * ((y - pred) / sigma).powi(2);
                let w = prior[idx] * added.exp();
                sum += w;
                q.push(w);
            }
            if sum > 0.0 {
                for k in 0..n {
                    let pk = prior[k].max(1e-12);
                    let qk = q[k] / sum;
                    if qk > 0.0 {
                        total_kl += pk * (pk / qk).ln();
                    }
                }
            }
        }
        // VoI 语义非负: KL 分量在浮点下可产生 ~-1e-16 舍入负值, 截断避免
        // 污染后续决策 (select_next_experiment 按 VoI 排序) 与 SelfTest 判定。
        (total_kl / samples as f64).max(0.0)
    }
    pub fn select_next_experiment(&self) -> Option<usize> {
        self.experiments
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.information_gain
                    .partial_cmp(&b.information_gain)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
    }

    /// M-open predictive check: adequacy = posterior > 1/n 的假设占比
    pub fn m_open_predictive_check(&self) -> MOpenCheck {
        let n = self.hypotheses.len();
        let adequacy = if n == 0 {
            0.0
        } else {
            let cutoff = 1.0 / n as f64;
            let count = self
                .hypotheses
                .iter()
                .filter(|h| h.posterior > cutoff)
                .count();
            (count as f64 / n as f64).max(0.0).min(1.0)
        };
        MOpenCheck {
            adequacy,
            expand_hypothesis_space: adequacy < ADEQUACY_THRESHOLD,
            threshold: ADEQUACY_THRESHOLD,
        }
    }

    /// 扩展假设空间: 压入新假设并重归一化
    pub fn expand_hypothesis_space(&mut self, new_hypothesis: Hypothesis) {
        self.hypotheses.push(new_hypothesis);
        self.normalize_posteriors();
    }
}

impl crate::core::nt_core_self_test::SelfTest for BayesianExperimentDesign {
    fn name(&self) -> &str {
        "nt_core_hcube_bayesian_experiment"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let config = VoIConfig::default();
        let hypotheses: Vec<Hypothesis> = (0..config.hypotheses)
            .map(|id| Hypothesis {
                id,
                log_likelihood: (id as f64 - 3.5) * 0.1,
                prior: 1.0,
                posterior: 0.0,
            })
            .collect();
        let mut bed = BayesianExperimentDesign::new(config, hypotheses);
        bed.normalize_posteriors();
        let total: f64 = bed.hypotheses.iter().map(|h| h.posterior).sum();
        if (total - 1.0).abs() > 1e-9 {
            return Err(vec![format!("posteriors must sum to 1, got {}", total)]);
        }
        let voi = bed.value_of_information(0);
        if !voi.is_finite() || voi < 0.0 {
            return Err(vec![format!(
                "VoI must be finite and non-negative, got {}",
                voi
            )]);
        }
        if bed.select_next_experiment().is_none() {
            return Err(vec![
                "select_next_experiment must return Some with a non-empty pool".into(),
            ]);
        }
        let check = bed.m_open_predictive_check();
        if !check.adequacy.is_finite() {
            return Err(vec!["M-open adequacy must be finite".into()]);
        }
        bed.expand_hypothesis_space(Hypothesis {
            id: config.hypotheses,
            log_likelihood: 1.0,
            prior: 1.0,
            posterior: 0.0,
        });
        let total2: f64 = bed.hypotheses.iter().map(|h| h.posterior).sum();
        if (total2 - 1.0).abs() > 1e-9 {
            return Err(vec![
                "posteriors must re-normalize after hypothesis expansion".into(),
            ]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    /// 均匀先验 + 零对数似然 → 归一化后均匀后验
    fn sample_design(n: usize) -> BayesianExperimentDesign {
        let config = VoIConfig {
            hypotheses: n,
            experiments: 16,
            samples_per_experiment: 64,
            noise: 0.1,
        };
        let hypotheses: Vec<Hypothesis> = (0..n)
            .map(|id| Hypothesis {
                id,
                log_likelihood: 0.0,
                prior: 1.0,
                posterior: 0.0,
            })
            .collect();
        let mut bed = BayesianExperimentDesign::new(config, hypotheses);
        bed.normalize_posteriors();
        bed
    }

    #[test]
    fn test_normalize_posteriors_sums_to_one() {
        let mut bed = sample_design(8);
        bed.normalize_posteriors();
        let total: f64 = bed.hypotheses.iter().map(|h| h.posterior).sum();
        assert!((total - 1.0).abs() < 1e-9, "posteriors must sum to 1");
        for h in &bed.hypotheses {
            assert!(h.posterior > 0.0, "all posteriors must be positive");
        }
    }

    #[test]
    fn test_best_hypothesis_highest_posterior() {
        let config = VoIConfig::default();
        let hypotheses: Vec<Hypothesis> = (0..8)
            .map(|id| Hypothesis {
                id,
                log_likelihood: 0.1 * (id as f64 - 3.5),
                prior: 1.0,
                posterior: 0.0,
            })
            .collect();
        let mut bed = BayesianExperimentDesign::new(config, hypotheses);
        bed.normalize_posteriors();
        let best = bed
            .hypotheses
            .iter()
            .max_by(|a, b| a.posterior.partial_cmp(&b.posterior).unwrap())
            .unwrap();
        assert_eq!(best.id, 7, "highest log-likelihood hypothesis must win");
        let via_method = best.posterior(&bed.hypotheses);
        assert!(
            (best.posterior - via_method).abs() < 1e-12,
            "Hypothesis::posterior() must match the stored normalized posterior"
        );
    }

    #[test]
    fn test_voi_tension_ordering_and_determinism() {
        let bed = sample_design(8);
        // e=0: split 1/17, 所有假设预测均值都在 split 之上 → 同组, 无张力 → VoI ≈ 0
        let degenerate = bed.value_of_information(0);
        // e=7: split 8/17 ≈ 0.47, 假设被 4/4 分开 → 判别张力高 → VoI > 0
        let balanced = bed.value_of_information(7);
        let repeat = bed.value_of_information(7);
        assert!(
            degenerate < 1e-12,
            "all-on-one-side experiment should yield ~0 VoI, got {}",
            degenerate
        );
        assert!(
            balanced > degenerate,
            "experiment with more hypotheses-tension must have higher VoI: {} vs {}",
            balanced,
            degenerate
        );
        assert_eq!(balanced, repeat, "VoI must be deterministic");
    }

    #[test]
    fn test_select_next_experiment_max_information_gain() {
        let mut bed = sample_design(8);
        // 16 experiments: 给 idx3 唯一最高 IG, 其余低值 → 无并列, 稳定选中 idx3
        for (i, e) in bed.experiments.iter_mut().enumerate() {
            e.information_gain = if i == 3 { 2.0 } else { 0.1 + (i as f64) * 0.01 };
        }
        let idx = bed.select_next_experiment().unwrap();
        assert_eq!(idx, 3, "must pick experiment with max information_gain (2.0)");
        assert_eq!(bed.experiments[idx].information_gain, 2.0);
        let empty_pool = BayesianExperimentDesign {
            config: VoIConfig::default(),
            hypotheses: Vec::new(),
            experiments: Vec::new(),
        };
        assert!(empty_pool.select_next_experiment().is_none());
    }

    #[test]
    fn test_m_open_check_expands_when_inadequate() {
        // 单一假设支配 → 只有 1/8 的假设 posterior > 1/8 → adequacy 低 → 扩展
        let config = VoIConfig::default();
        let hypotheses: Vec<Hypothesis> = (0..8)
            .map(|id| Hypothesis {
                id,
                log_likelihood: if id == 0 { 3.0 } else { -3.0 },
                prior: 1.0,
                posterior: 0.0,
            })
            .collect();
        let mut bed = BayesianExperimentDesign::new(config, hypotheses);
        bed.normalize_posteriors();
        let check = bed.m_open_predictive_check();
        assert!(
            check.expand_hypothesis_space,
            "dominated posterior must trigger M-open expansion"
        );
        assert!(check.adequacy < check.threshold);
    }

    #[test]
    fn test_m_open_check_adequate_without_expansion() {
        // 7/8 假设 posterior > 1/8 → adequacy 0.875 ≥ 0.75 → 不扩展
        let config = VoIConfig::default();
        let hypotheses: Vec<Hypothesis> = (0..8)
            .map(|id| Hypothesis {
                id,
                log_likelihood: if id == 0 { -10.0 } else { 1.0 },
                prior: 1.0,
                posterior: 0.0,
            })
            .collect();
        let mut bed = BayesianExperimentDesign::new(config, hypotheses);
        bed.normalize_posteriors();
        let check = bed.m_open_predictive_check();
        assert!(!check.expand_hypothesis_space);
        assert!(check.adequacy >= 0.75, "adequacy = {}", check.adequacy);
    }

    #[test]
    fn test_expand_hypothesis_space_renormalizes() {
        let mut bed = sample_design(8);
        let before: f64 = bed.hypotheses.iter().map(|h| h.posterior).sum();
        bed.expand_hypothesis_space(Hypothesis {
            id: 8,
            log_likelihood: 2.0,
            prior: 1.0,
            posterior: 0.0,
        });
        assert_eq!(bed.hypotheses.len(), 9);
        let total: f64 = bed.hypotheses.iter().map(|h| h.posterior).sum();
        assert!((total - 1.0).abs() < 1e-9, "must re-normalize after expansion");
        let best = bed
            .hypotheses
            .iter()
            .max_by(|a, b| a.posterior.partial_cmp(&b.posterior).unwrap())
            .unwrap();
        assert_eq!(best.id, 8, "new best-fitting hypothesis must dominate");
        let old_mass: f64 = bed.hypotheses.iter().take(8).map(|h| h.posterior).sum();
        assert!(
            old_mass < before,
            "prior hypotheses must shrink after absorbing a better model"
        );
    }

    #[test]
    fn test_selftest() {
        let bed = BayesianExperimentDesign::new(VoIConfig::default(), Vec::new());
        match bed.self_test() {
            Ok(()) => {}
            Err(f) => panic!("self_test failed: {:?}", f),
        }
    }

    #[test]
    fn dbg_selftest() {
        let config = VoIConfig::default();
        let hypotheses: Vec<Hypothesis> = (0..config.hypotheses)
            .map(|id| Hypothesis {
                id,
                log_likelihood: (id as f64 - 3.5) * 0.1,
                prior: 1.0,
                posterior: 0.0,
            })
            .collect();
        let mut bed = BayesianExperimentDesign::new(config, hypotheses);
        bed.normalize_posteriors();
        let total: f64 = bed.hypotheses.iter().map(|h| h.posterior).sum();
        println!("total={} diff={}", total, (total - 1.0).abs());
        let voi = bed.value_of_information(0);
        println!("voi={} finite={} lt0={}", voi, voi.is_finite(), voi < 0.0);
        println!("selnext={:?}", bed.select_next_experiment());
        let check = bed.m_open_predictive_check();
        println!(
            "adequacy={} finite={} expand={}",
            check.adequacy,
            check.adequacy.is_finite(),
            check.expand_hypothesis_space
        );
        bed.expand_hypothesis_space(Hypothesis {
            id: config.hypotheses,
            log_likelihood: 1.0,
            prior: 1.0,
            posterior: 0.0,
        });
        let total2: f64 = bed.hypotheses.iter().map(|h| h.posterior).sum();
        println!("total2={} diff={}", total2, (total2 - 1.0).abs());
    }
}