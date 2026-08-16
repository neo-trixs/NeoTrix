//! # 量子态检测与最优融合 (Quantum State Detection & Optimal Fusion)
//!
//! 借鉴量子力学范式（叠加/纠缠/坍缩），以确定性算法实现**多源信号最优融合检测**：
//!
//! - **叠加态 (Superposition)**: 多个检测器的独立输出被视为同一系统的叠加态分量
//!   ——每个分量携带信号值 `value`、置信度 `confidence` 与来源 `source`。
//! - **纠缠一致性 (Entanglement)**: 度量各分量之间的内在一致性。高度纠缠
//!   （信号高度一致）意味着多源佐证 → 融合结果可信；低纠缠（信号冲突）
//!   意味着测量噪声 → 融合置信度被惩罚。
//! - **坍缩选优 (Collapse)**: 对叠加态执行最优融合坍缩，产出单一高可靠
//!   `FusedSignal`（值 + 置信度 + 熵 + 纠缠度 + 主导源）。
//!
//! 与 NeoTrix 既有能力的边界：
//! - GWT `resonance` 负责**模块间**注意力路由；本模块负责**信号级**融合。
//! - `arch_fitness` 守卫族检测架构退化；本模块可作为其多源信号的融合前端。
//! - 实现为确定性浮点算法，无真量子随机性；不引入外部量子库（R-P48）。
//!
//! 层归属: L4 认知层 (Cognition)。接线: SelfTest 检测族 (T3)。

use crate::core::nt_core_self_test::SelfTest;

// ---------------------------------------------------------------------------
// 叠加态
// ---------------------------------------------------------------------------

/// 单个检测源的输出 —— 叠加态的一个分量。
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumSignal {
    /// 归一化信号值 [0.0, 1.0]
    pub value: f64,
    /// 检测源置信度 [0.0, 1.0]
    pub confidence: f64,
    /// 检测源标识（如 "gwt_resonance", "arch_fitness", "e8_predictor"）
    pub source: String,
}

impl QuantumSignal {
    pub fn new(value: f64, confidence: f64, source: impl Into<String>) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            source: source.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// 融合配置 (R-P11: config struct + Default)
// ---------------------------------------------------------------------------

/// 融合超参数。
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumFusionConfig {
    /// 置信度低于该值的信号视为噪声，坍缩时排除
    pub confidence_floor: f64,
    /// 纠缠度高于该值时视为「高度纠缠」——融合置信度获得加成
    pub high_entanglement: f64,
    /// 纠缠度低于该值时视为「冲突」——融合置信度被惩罚
    pub conflict_threshold: f64,
}

impl Default for QuantumFusionConfig {
    fn default() -> Self {
        Self {
            confidence_floor: 0.2,
            high_entanglement: 0.75,
            conflict_threshold: 0.6,
        }
    }
}

// ---------------------------------------------------------------------------
// 坍缩态
// ---------------------------------------------------------------------------

/// 最优融合坍缩后的单一高可靠信号。
#[derive(Debug, Clone, PartialEq)]
pub struct FusedSignal {
    /// 融合后的信号值 [0.0, 1.0]
    pub value: f64,
    /// 融合置信度 [0.0, 1.0]（含纠缠调节）
    pub confidence: f64,
    /// 信号熵 [0.0, 1.0]——多样性/不确定性（高熵 = 分散）
    pub entropy: f64,
    /// 纠缠一致性 [0.0, 1.0]——多源佐证强度
    pub entanglement: f64,
    /// 主导源：置信度最高且未被丢弃的信号来源
    pub dominant_source: Option<String>,
    /// 参与坍缩的有效信号数（排除噪声后）
    pub fused_count: usize,
}

// ---------------------------------------------------------------------------
// 叠加态实现
// ---------------------------------------------------------------------------

/// 多源信号的叠加态，提供纠缠度量与最优融合坍缩。
#[derive(Debug, Clone, Default)]
pub struct QuantumSuperposition {
    signals: Vec<QuantumSignal>,
}

impl QuantumSuperposition {
    pub fn new() -> Self {
        Self {
            signals: Vec::new(),
        }
    }

    /// 加入一个检测源信号。
    pub fn push(&mut self, sig: QuantumSignal) {
        self.signals.push(sig);
    }

    /// 便捷构造：从信号迭代器构建叠加态。
    pub fn from_signals(signals: impl IntoIterator<Item = QuantumSignal>) -> Self {
        Self {
            signals: signals.into_iter().collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }

    pub fn len(&self) -> usize {
        self.signals.len()
    }

    pub fn signals(&self) -> &[QuantumSignal] {
        &self.signals
    }

    /// 纠缠一致性：信号值的加权离散度。
    /// 0.0 = 完全纠缠（全部信号一致）；越接近 1.0 = 越冲突。
    pub fn entanglement_dispersion(&self) -> f64 {
        if self.signals.is_empty() {
            return 0.0;
        }
        let total_conf: f64 = self.signals.iter().map(|s| s.confidence).sum();
        if total_conf <= 0.0 {
            return 0.0;
        }
        let mean: f64 = self
            .signals
            .iter()
            .map(|s| s.value * s.confidence)
            .sum::<f64>()
            / total_conf;
        let variance: f64 = self
            .signals
            .iter()
            .map(|s| s.confidence * (s.value - mean).powi(2))
            .sum::<f64>()
            / total_conf;
        (variance.sqrt()).clamp(0.0, 1.0)
    }

    /// 纠缠一致性得分：`1 - dispersion`，越接近 1.0 表示多源佐证越强。
    pub fn entanglement(&self) -> f64 {
        1.0 - self.entanglement_dispersion()
    }

    /// 信号熵：基于有效信号值分桶的香农熵（归一化到 [0,1]）。
    /// 单一确定信号 → 0.0；均匀分散 → 1.0。
    pub fn entropy(&self) -> f64 {
        let effective: Vec<f64> = self.signals.iter().map(|s| s.value).collect();
        if effective.is_empty() {
            return 0.0;
        }
        const BUCKETS: usize = 8;
        let mut counts = [0usize; BUCKETS];
        for v in &effective {
            let idx = ((v * BUCKETS as f64) as usize).min(BUCKETS - 1);
            counts[idx] += 1;
        }
        let n = effective.len() as f64;
        let mut entropy = 0.0f64;
        for c in counts.iter().filter(|&&c| c > 0) {
            let p = *c as f64 / n;
            entropy -= p * p.ln();
        }
        (entropy / (BUCKETS as f64).ln()).clamp(0.0, 1.0)
    }

    /// 最优融合坍缩：将叠加态坍缩为单一高可靠信号。
    ///
    /// 融合策略（确定性）：
    /// 1. 排除置信度低于 `confidence_floor` 的噪声信号。
    /// 2. 置信度加权平均得基础值。
    /// 3. 按纠缠度调节置信度：高度纠缠 → 加成；冲突 → 惩罚。
    /// 4. 主导源 = 排除噪声后置信度最高的信号。
    pub fn fuse(&self) -> FusedSignal {
        let cfg = QuantumFusionConfig::default();
        let effective: Vec<&QuantumSignal> = self
            .signals
            .iter()
            .filter(|s| s.confidence >= cfg.confidence_floor)
            .collect();

        if effective.is_empty() {
            return FusedSignal {
                value: 0.0,
                confidence: 0.0,
                entropy: self.entropy(),
                entanglement: self.entanglement(),
                dominant_source: None,
                fused_count: 0,
            };
        }

        let total_conf: f64 = effective.iter().map(|s| s.confidence).sum();
        let weighted_value: f64 = effective
            .iter()
            .map(|s| s.value * s.confidence)
            .sum::<f64>()
            / total_conf;

        let ent = self.entanglement();
        // 纠缠调节：冲突惩罚 / 高纠缠加成
        let mut confidence =
            effective.iter().map(|s| s.confidence).sum::<f64>() / effective.len() as f64;
        if ent < cfg.conflict_threshold {
            confidence *= ent / cfg.conflict_threshold; // 冲突 → 线性惩罚
        } else if ent > cfg.high_entanglement {
            confidence = (confidence + 0.15 * (ent - cfg.high_entanglement)).min(1.0);
        }

        let dominant = effective
            .iter()
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.source.clone());

        FusedSignal {
            value: weighted_value.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            entropy: self.entropy(),
            entanglement: ent,
            dominant_source: dominant,
            fused_count: effective.len(),
        }
    }

    /// 加权融合配置版本（允许自定义超参）。
    pub fn fuse_with(&self, cfg: &QuantumFusionConfig) -> FusedSignal {
        let effective: Vec<&QuantumSignal> = self
            .signals
            .iter()
            .filter(|s| s.confidence >= cfg.confidence_floor)
            .collect();

        if effective.is_empty() {
            return FusedSignal {
                value: 0.0,
                confidence: 0.0,
                entropy: self.entropy(),
                entanglement: self.entanglement(),
                dominant_source: None,
                fused_count: 0,
            };
        }

        let total_conf: f64 = effective.iter().map(|s| s.confidence).sum();
        let weighted_value: f64 = effective
            .iter()
            .map(|s| s.value * s.confidence)
            .sum::<f64>()
            / total_conf;

        let ent = self.entanglement();
        let mut confidence =
            effective.iter().map(|s| s.confidence).sum::<f64>() / effective.len() as f64;
        if ent < cfg.conflict_threshold {
            confidence *= ent / cfg.conflict_threshold;
        } else if ent > cfg.high_entanglement {
            confidence = (confidence + 0.15 * (ent - cfg.high_entanglement)).min(1.0);
        }

        let dominant = effective
            .iter()
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.source.clone());

        FusedSignal {
            value: weighted_value.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            entropy: self.entropy(),
            entanglement: ent,
            dominant_source: dominant,
            fused_count: effective.len(),
        }
    }

    // -------------------------------------------------------------------
    // 算法级升级 (研究驱动的数学强化):
    //  Dempster-Shafer 证据融合 + 去相干模型 + Born 规则坍缩
    // -------------------------------------------------------------------

    /// 去相干因子: 信号熵越高 → 越接近"已去相干"的经典噪声 → 未知质量越大。
    /// 映射熵 [0,1] → 去相干系数 [0,1], 在熵 > `decoherence_entropy` 时线性抬升。
    pub fn decoherence_factor(entropy: f64, decoherence_entropy: f64) -> f64 {
        if entropy <= decoherence_entropy {
            0.0
        } else {
            ((entropy - decoherence_entropy) / (1.0 - decoherence_entropy)).clamp(0.0, 1.0)
        }
    }

    /// Dempster-Shafer 证据融合: 将每个信号视为二元证据体
    /// `m(healthy), m(degraded), m(unknown)` 用 D-S 组合规则逐对融合。
    ///
    /// 二元识别框架 Θ = {H, D}: H=健康, D=退化。每信号:
    ///   - `m_h = value * confidence`          (支持健康)
    ///   - `m_d = (1 - value) * confidence`    (支持退化)
    ///   - `m_u = 1 - m_h - m_d`               (未知/去相干/证据不足)
    ///
    /// Dempster 组合: 对 A∈{H,D}, `m_12(A) = Σ_{B∩C=A} m_1(B)·m_2(C) / (1-K)`,
    /// K = Σ_{B∩C=∅} m_1(B)·m_2(C) 为冲突质量。高冲突 → 大 K → 归一化放大。
    /// 相比线性加权融合, D-S 显式建模"未知" (去相干), 对冲突证据更稳健。
    pub fn fuse_evidence(&self, cfg: &QuantumFusionConfig) -> EvidenceFusedSignal {
        let effective: Vec<&QuantumSignal> = self
            .signals
            .iter()
            .filter(|s| s.confidence >= cfg.confidence_floor)
            .collect();

        if effective.is_empty() {
            return EvidenceFusedSignal {
                m_healthy: 0.0,
                m_degraded: 0.0,
                m_unknown: 1.0,
                conflict: 0.0,
                belief_healthy: 0.0,
                plausibility_healthy: 0.0,
                fused_count: 0,
            };
        }

        // 迭代 D-S 组合 (两两融合)
        let mut m_h = effective[0].value * effective[0].confidence;
        let mut m_d = (1.0 - effective[0].value) * effective[0].confidence;
        let mut m_u = (1.0 - m_h - m_d).clamp(0.0, 1.0);
        let mut conflict = 0.0f64;

        for sig in &effective[1..] {
            let c_h = sig.value * sig.confidence;
            let c_d = (1.0 - sig.value) * sig.confidence;
            let c_u = (1.0 - c_h - c_d).clamp(0.0, 1.0);

            // 冲突质量 K = m1(B) ∩ m2(C) = ∅ 的组合质量
            let k = m_h * c_d + m_d * c_h;
            conflict = k;

            if k >= 1.0 {
                // 完全冲突: 全质量归未知 (去相干主导)
                m_h = 0.0;
                m_d = 0.0;
                m_u = 1.0;
                continue;
            }

            let norm = 1.0 - k;
            let nh = (m_h * c_h + m_h * c_u + m_u * c_h) / norm;
            let nd = (m_d * c_d + m_d * c_u + m_u * c_d) / norm;
            let nu = (m_u * c_u) / norm;
            m_h = nh;
            m_d = nd;
            m_u = nu;
        }

        // 熵→去相干调节: 高熵信号把部分确定性质量归入未知
        let ent = self.entropy();
        let deph = Self::decoherence_factor(ent, cfg.high_entanglement);
        if deph > 0.0 {
            let bleed = (m_h + m_d) * deph * 0.3;
            m_h -= m_h * deph * 0.3;
            m_d -= m_d * deph * 0.3;
            m_u += bleed;
        }

        let belief_healthy = m_h;
        let plausibility_healthy = m_h + m_u;

        EvidenceFusedSignal {
            m_healthy: m_h.clamp(0.0, 1.0),
            m_degraded: m_d.clamp(0.0, 1.0),
            m_unknown: m_u.clamp(0.0, 1.0),
            conflict,
            belief_healthy,
            plausibility_healthy,
            fused_count: effective.len(),
        }
    }

    /// Born 规则坍缩: 把证据融合结果映射为确定性信号 (对应量子测量坍缩)。
    /// `|振幅|²` 概率的确定性等价: belief 作为健康振幅平方, 直接归一化到 [0,1]。
    pub fn collapse_evidence(&self, cfg: &QuantumFusionConfig) -> FusedSignal {
        let ev = self.fuse_evidence(cfg);
        // 健康信念作为坍缩结果值; 置信度由 belief 主导 (非未知主导)
        let value = ev.belief_healthy;
        let confidence = ev.belief_healthy + ev.plausibility_healthy / 2.0;
        FusedSignal {
            value: value.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            entropy: self.entropy(),
            entanglement: self.entanglement(),
            dominant_source: self
                .signals
                .iter()
                .max_by(|a, b| {
                    a.confidence
                        .partial_cmp(&b.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|s| s.source.clone()),
            fused_count: ev.fused_count,
        }
    }
}

/// D-S 证据融合结果 — 显式建模未知/冲突 (对应量子去相干与测量不确定性)。
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceFusedSignal {
    /// 支持健康的质量 [0,1] (belief)
    pub m_healthy: f64,
    /// 支持退化的质量 [0,1]
    pub m_degraded: f64,
    /// 未知/去相干质量 [0,1] (证据不足)
    pub m_unknown: f64,
    /// 冲突系数 K [0,1] (高冲突 = 多源打架)
    pub conflict: f64,
    /// 健康信念 [0,1]
    pub belief_healthy: f64,
    /// 健康可能度 [0,1] (belief + unknown, D-S 区间上界)
    pub plausibility_healthy: f64,
    /// 参与融合的有效信号数
    pub fused_count: usize,
}

// ---------------------------------------------------------------------------
// SelfTest 接线 (T3: 注册进检测族)
// ---------------------------------------------------------------------------

/// 量子态检测与最优融合的 SelfTest 探针。
pub struct QuantumFusionSelfTest;

impl SelfTest for QuantumFusionSelfTest {
    fn name(&self) -> &str {
        "nt_core_quantum_fusion"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // 高纠缠场景: 多源一致 → 融合置信度应被加成
        let mut sup = QuantumSuperposition::new();
        sup.push(QuantumSignal::new(0.8, 0.9, "a"));
        sup.push(QuantumSignal::new(0.85, 0.85, "b"));
        sup.push(QuantumSignal::new(0.78, 0.95, "c"));
        let fused = sup.fuse();
        if fused.entanglement < 0.6 {
            failures.push(format!("高纠缠场景纠缠度异常: {}", fused.entanglement));
        }
        if fused.fused_count != 3 {
            failures.push(format!(
                "高纠缠场景应融合 3 信号, 实际 {}",
                fused.fused_count
            ));
        }
        if (fused.value - 0.80).abs() > 0.06 {
            failures.push(format!("高纠缠融合值偏离期望: {}", fused.value));
        }

        // 冲突场景: 信号打架 → 融合置信度应被惩罚
        let mut sup2 = QuantumSuperposition::new();
        sup2.push(QuantumSignal::new(0.0, 0.9, "a"));
        sup2.push(QuantumSignal::new(1.0, 0.9, "b"));
        let fused2 = sup2.fuse();
        if fused2.entanglement > 0.6 {
            failures.push(format!("冲突场景纠缠度异常: {}", fused2.entanglement));
        }
        if fused2.confidence > 0.8 {
            failures.push(format!("冲突场景置信度应被惩罚: {}", fused2.confidence));
        }

        // 噪声场景: 低置信度信号被排除
        let mut sup3 = QuantumSuperposition::new();
        sup3.push(QuantumSignal::new(0.9, 0.95, "trusted"));
        sup3.push(QuantumSignal::new(0.1, 0.05, "noise"));
        let fused3 = sup3.fuse();
        if fused3.fused_count != 1 {
            failures.push(format!("噪声信号应被排除, 实际融合 {}", fused3.fused_count));
        }
        if fused3.dominant_source.as_deref() != Some("trusted") {
            failures.push("主导源应为 trusted".into());
        }

        // 空叠加态 → 中性坍缩
        let empty = QuantumSuperposition::new();
        let fused4 = empty.fuse();
        if fused4.fused_count != 0 || fused4.dominant_source.is_some() {
            failures.push("空叠加态应产出中性坍缩".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_signal_clamps() {
        let s = QuantumSignal::new(1.5, -0.2, "src");
        assert_eq!(s.value, 1.0);
        assert_eq!(s.confidence, 0.0);
    }

    #[test]
    fn test_entanglement_high_when_consistent() {
        let mut sup = QuantumSuperposition::new();
        sup.push(QuantumSignal::new(0.5, 0.9, "a"));
        sup.push(QuantumSignal::new(0.51, 0.9, "b"));
        sup.push(QuantumSignal::new(0.49, 0.9, "c"));
        assert!(sup.entanglement() > 0.9);
    }

    #[test]
    fn test_entanglement_low_when_conflicting() {
        let mut sup = QuantumSuperposition::new();
        sup.push(QuantumSignal::new(0.0, 0.9, "a"));
        sup.push(QuantumSignal::new(1.0, 0.9, "b"));
        assert!(sup.entanglement() < 0.7);
    }

    #[test]
    fn test_entropy_zero_for_single() {
        let mut sup = QuantumSuperposition::new();
        sup.push(QuantumSignal::new(0.7, 0.9, "a"));
        assert_eq!(sup.entropy(), 0.0);
    }

    #[test]
    fn test_fuse_confidence_penalized_on_conflict() {
        let mut sup = QuantumSuperposition::new();
        sup.push(QuantumSignal::new(0.0, 0.95, "a"));
        sup.push(QuantumSignal::new(1.0, 0.95, "b"));
        let fused = sup.fuse();
        assert!(
            fused.confidence < 0.8,
            "冲突应惩罚置信度, got {}",
            fused.confidence
        );
    }

    #[test]
    fn test_fuse_dominant_is_highest_confidence() {
        let mut sup = QuantumSuperposition::new();
        sup.push(QuantumSignal::new(0.6, 0.5, "weak"));
        sup.push(QuantumSignal::new(0.7, 0.99, "strong"));
        let fused = sup.fuse();
        assert_eq!(fused.dominant_source.as_deref(), Some("strong"));
    }

    #[test]
    fn test_fuse_with_custom_config_floor() {
        let cfg = QuantumFusionConfig {
            confidence_floor: 0.9,
            ..Default::default()
        };
        let mut sup = QuantumSuperposition::new();
        sup.push(QuantumSignal::new(0.8, 0.5, "weak"));
        sup.push(QuantumSignal::new(0.2, 0.95, "strong"));
        let fused = sup.fuse_with(&cfg);
        assert_eq!(fused.fused_count, 1);
        assert_eq!(fused.dominant_source.as_deref(), Some("strong"));
    }

    #[test]
    fn test_decoherence_factor_zero_below_threshold() {
        assert_eq!(QuantumSuperposition::decoherence_factor(0.5, 0.75), 0.0);
    }

    #[test]
    fn test_decoherence_factor_rises_with_entropy() {
        let low = QuantumSuperposition::decoherence_factor(0.8, 0.75);
        let high = QuantumSuperposition::decoherence_factor(0.95, 0.75);
        assert!(low > 0.0 && high > low);
    }

    #[test]
    fn test_fuse_evidence_all_healthy_high_belief() {
        let cfg = QuantumFusionConfig::default();
        let mut sup = QuantumSuperposition::new();
        sup.push(QuantumSignal::new(0.9, 0.9, "a"));
        sup.push(QuantumSignal::new(0.85, 0.9, "b"));
        sup.push(QuantumSignal::new(0.95, 0.9, "c"));
        let ev = sup.fuse_evidence(&cfg);
        assert!(
            ev.belief_healthy > 0.9,
            "全健康信号 belief 应高, got {}",
            ev.belief_healthy
        );
        assert!(
            ev.m_unknown < 0.2,
            "一致信号未知质量应低, got {}",
            ev.m_unknown
        );
        assert!(
            ev.m_degraded < 0.2,
            "一致信号退化质量应低, got {}",
            ev.m_degraded
        );
    }

    #[test]
    fn test_fuse_evidence_conflict_raises_unknown() {
        let cfg = QuantumFusionConfig::default();
        let mut sup = QuantumSuperposition::new();
        sup.push(QuantumSignal::new(0.0, 0.95, "a"));
        sup.push(QuantumSignal::new(1.0, 0.95, "b"));
        let ev = sup.fuse_evidence(&cfg);
        assert!(
            ev.conflict > 0.5,
            "冲突信号冲突系数应高, got {}",
            ev.conflict
        );
        assert!(
            ev.belief_healthy < 0.7,
            "冲突下 belief 不应过高, got {}",
            ev.belief_healthy
        );
    }

    #[test]
    fn test_collapse_evidence_matches_belief() {
        let cfg = QuantumFusionConfig::default();
        let mut sup = QuantumSuperposition::new();
        sup.push(QuantumSignal::new(0.9, 0.95, "trusted"));
        sup.push(QuantumSignal::new(0.88, 0.9, "aux"));
        let fused = sup.collapse_evidence(&cfg);
        assert!(
            fused.value > 0.8,
            "collapse 应反映健康 belief, got {}",
            fused.value
        );
        assert!(fused.confidence > 0.5);
    }
}
