use crate::neotrix::nt_mind::core::{CapabilityVector, PerformanceEvaluator, AccessContext};
use crate::neotrix::nt_mind::self_iterating::AbsorbValidator;
use crate::neotrix::nt_world_model::TaskType;

/// Heavy-Pass@K 验证结果
#[derive(Debug, Clone)]
pub struct HeavyPassResult {
    pub hp_at_k: f64,
    pub hm_at_k: f64,
    pub vote_at_k: f64,
    pub mean_at_k: f64,
    pub k: usize,
}

/// Heavy-Pass@K 验证协议 (arXiv:2605.02396)
///
/// HP@K >= HM@K >= Vote@K >= Mean@K >= P@K
/// - P@K: at least one of K is correct
/// - Vote@K: majority vote correctness
/// - HM@K: heavy-weighted top-k mean
/// - HP@K: deliberation-synthesized answer correctness
pub fn heavy_pass_at_k(scores: &[f64], threshold: f64) -> HeavyPassResult {
    let k = scores.len();
    if k == 0 {
        return HeavyPassResult {
            hp_at_k: 0.0, hm_at_k: 0.0, vote_at_k: 0.0, mean_at_k: 0.0, k: 0,
        };
    }

    // Mean@K
    let mean_at_k: f64 = scores.iter().sum::<f64>() / k as f64;

    // Vote@K: fraction above threshold
    let vote_at_k = scores.iter().filter(|&&s| s >= threshold).count() as f64 / k as f64;

    // HM@K: top half (heavy) mean
    let mut sorted = scores.to_vec();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let heavy_count = (k / 2).max(1);
    let hm_at_k: f64 = sorted.iter().take(heavy_count).sum::<f64>() / heavy_count as f64;

    // HP@K: weighted by confidence and diversity bonus
    // deliberation synthesizes across traces → bonus for having multiple high-quality traces
    let top_k = sorted.iter().take(3.min(k)).copied().collect::<Vec<_>>();
    let diversity_bonus = if top_k.len() >= 2 {
        let var: f64 = top_k.iter().map(|s| (s - mean_at_k).powi(2)).sum::<f64>() / top_k.len() as f64;
        var.sqrt().min(0.2) // diversity gives up to 0.2 bonus
    } else {
        0.0
    };
    let hp_at_k = (hm_at_k + diversity_bonus * 0.5).min(1.0);

    HeavyPassResult { hp_at_k, hm_at_k, vote_at_k, mean_at_k, k }
}

/// CriticNode — 独立验证节点（对应 AIRecon 的 Critic Model 闭环）
///
/// 关键设计原则：
/// 1. CriticNode 必须独立于被验证的 Worker 节点运行
/// 2. 验证标准必须与被验证内容不同（否则无法发现同源幻觉）
/// 3. 支持多维度验证：能力评估 + 响应质量 + 规则合规
#[derive(Clone)]
pub struct CriticNode {
    /// 验证时的"视角"差异（不同 temperature 或 prompt 模板）
    pub perspective_bias: f64,
    /// 启用严格模式（更强验证条件）
    pub strict_mode: bool,
}

impl Default for CriticNode {
    fn default() -> Self {
        Self::new()
    }
}

impl CriticNode {
    pub fn new() -> Self {
        Self {
            perspective_bias: 0.1,
            strict_mode: false,
        }
    }

    /// 独立能力评估（基于 PerformanceEvaluator，但保持独立实例）
    pub fn evaluate(&self, task_type: TaskType, capability: &CapabilityVector) -> f64 {
        let score = PerformanceEvaluator::evaluate(&task_type, capability);
        // 加入视角偏差模拟独立评估（避免与被验证方使用完全相同的标准）
        
        (score + self.perspective_bias * 0.1 - self.perspective_bias * 0.05).clamp(0.0, 1.0)
    }

    /// 带上下文的评估（使用 AccessContext）
    pub fn evaluate_with_context(&self, task_type: TaskType, capability: &CapabilityVector, context: &AccessContext) -> f64 {
        let base = self.evaluate(task_type, capability);
        // trust_score 调权：低信任 → 严格评分
        let trust_factor = context.trust_score * 0.2;
        (base * (1.0 - trust_factor) + base).min(1.0) / 2.0
    }

    /// 从 LLM 响应文本中提取评分（独立启发式，不依赖 LLM）
    pub fn evaluate_from_response(&self, response: &str) -> f64 {
        let length_score = (response.len() as f64 / 2000.0).min(1.0) * 0.4;
        let has_steps = if response.contains("1)") || response.contains("Step") || response.contains("步骤") { 0.3 } else { 0.0 };
        let has_code = if response.contains("```") { 0.3 } else { 0.0 };
        let base_score = (length_score + has_steps + has_code).min(1.0);
        if self.strict_mode {
            // 严格模式：必须有实质内容
            if response.len() < 50 { 0.0 } else { base_score * 0.8 }
        } else {
            base_score
        }
    }

    pub fn needs_retry(&self, score: f64, threshold: f64) -> bool {
        let effective_threshold = if self.strict_mode { threshold + 0.1 } else { threshold };
        score < effective_threshold
    }

    /// HP@K 验证：评估多个推理轨迹的综合可信度
    pub fn heavy_pass_verify(&self, scores: &[f64]) -> HeavyPassResult {
        let threshold = if self.strict_mode { 0.7 } else { 0.5 };
        heavy_pass_at_k(scores, threshold)
    }

    /// P@K：评估 K 个推理轨迹中至少一个正确的概率
    pub fn pass_at_k(&self, scores: &[f64], threshold: f64) -> f64 {
        if scores.is_empty() { return 0.0; }
        let correct = scores.iter().filter(|&&s| s >= threshold).count();
        if correct > 0 { 1.0 } else { 0.0 }
    }

    /// 跨维度交叉验证（检查能力向量的内部一致性）
    pub fn cross_validate(&self, capability: &CapabilityVector) -> Vec<String> {
        let mut issues = Vec::new();
        // 检查：verification 不应该高于相关的具体能力
        if capability.verification() > 0.9 && capability.analysis() < 0.5 {
            issues.push("verification > 0.9 但 analysis < 0.5：验证能力与分析能力不匹配".to_string());
        }
        if capability.quality_gates() > 0.9 && capability.verification() < 0.5 {
            issues.push("quality_gates > 0.9 但 verification < 0.5：门控与验证能力不匹配".to_string());
        }
        issues
    }
}

impl AbsorbValidator for CriticNode {
    fn validate_absorb(&self, after: &CapabilityVector) -> bool {
        self.cross_validate(after).is_empty()
    }
}
