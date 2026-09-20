pub use neotrix_reasoning::kernel_types::{
    EVOLUTION, KERNEL_DIM, KernelStats, ReasoningKernel, ReasoningMethod, ReasoningOutput,
    SelfConsistencyResult, StageInfo, Vector,
};

/// 最终答案正确性验证器（RLVR 锚，P0）：数值比对 + 归一化文本比对。
/// 返回 [0.0, 1.0] 的匹配分数。
pub fn verify_answer(gold: &str, candidate: &str) -> f64 {
    if gold.trim().is_empty() || candidate.trim().is_empty() {
        return 0.0;
    }
    let g = normalize_answer(gold);
    let c = normalize_answer(candidate);
    if g == c {
        return 1.0;
    }
    if let (Some(ge), Some(ce)) = (extract_number(&g), extract_number(&c)) {
        let scale = ge.abs().max(ce.abs()).max(1.0);
        let diff = (ge - ce).abs();
        return (1.0 - diff / scale).clamp(0.0, 1.0);
    }
    if g.contains(&c) || c.contains(&g) {
        let ratio = c.len() as f64 / g.len().max(1) as f64;
        return ratio.min(1.0) * 0.8;
    }
    0.0
}

fn normalize_answer(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn extract_number(s: &str) -> Option<f64> {
    s.chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect::<String>()
        .parse::<f64>()
        .ok()
}

pub fn text_to_vector(text: &str, dim: usize) -> Vector {
    if text.is_empty() || dim == 0 {
        return vec![0.0; dim];
    }
    let bytes: Vec<u8> = text.bytes().collect();
    let mut v = vec![0.0; dim];
    for (i, &b) in bytes.iter().enumerate() {
        let pos_phase = (i as f64 / bytes.len() as f64) * std::f64::consts::PI;
        let idx = i % dim;
        v[idx] = (b as f64 / 255.0) * 2.0 - 1.0 + pos_phase.sin() * 0.2;
    }
    for i in 0..dim.saturating_sub(bytes.len()) {
        let byte_idx = i % bytes.len().max(1);
        let b = bytes[byte_idx] as f64;
        v[bytes.len() + i] = ((b / 255.0) * 2.0 - 1.0) * 0.5;
    }
    let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
    v.iter_mut().for_each(|x| *x /= norm);
    v
}

fn circuit_label(m: ReasoningMethod) -> &'static str {
    match m {
        ReasoningMethod::Deductive => "deductive logic",
        ReasoningMethod::Inductive => "inductive pattern",
        ReasoningMethod::Abductive => "abductive inference",
        ReasoningMethod::Analogical => "analogical transfer",
        ReasoningMethod::Compositional => "compositional planning",
        ReasoningMethod::Recursive => "recursive verification",
        ReasoningMethod::Adversarial => "adversarial critique",
        ReasoningMethod::FirstPrinciples => "first principles",
        ReasoningMethod::AutoFetch => "auto-fetch",
        ReasoningMethod::KnowledgeRetrieval => "knowledge retrieval",
        ReasoningMethod::GradientLearning => "gradient learning",
        ReasoningMethod::ArchitectureSearch => "arch search",
        ReasoningMethod::GpuCompute => "GPU compute",
        ReasoningMethod::DistributedConsensus => "distributed consensus",
        ReasoningMethod::ExperienceDistill => "experience distillation",
        ReasoningMethod::EmergentAnalysis => "emergent analysis",
        ReasoningMethod::SystemIntegration => "system integration",
        ReasoningMethod::EnsembleVoting => "ensemble voting",
        ReasoningMethod::SelfImprovement => "self-improvement",
        ReasoningMethod::SparseRouting => "sparse routing",
    }
}

pub fn format_kernel_output(v: &[f64], prompt: &str, stage: usize, energy: f64, circuit_names: &[String]) -> String {
    let raw_energy: f64 = v.iter().map(|x| x.abs()).sum::<f64>() / v.len().max(1) as f64;
    let confidence = energy.clamp(0.1, 1.0);
    let energy = raw_energy.max(confidence * 0.5);
    let stage_info = &EVOLUTION[stage];

    let above_half = v.iter().filter(|x| x.abs() > 0.5).count();

    match energy {
        e if e < 0.3 => {
            let mut resp = format!(
                "I need more context to form a solid inference about \"{}\". \
                 My {} kernel is registering weak signal (energy ~{:.2}) \
                 across {} reasoning pathway{}.",
                prompt, stage_info.label, e,
                circuit_names.len(), if circuit_names.len() == 1 { "" } else { "s" },
            );
            if !circuit_names.is_empty() {
            }
            resp.push_str(" Could you provide more detail or clarify the question?");
            resp
        }
        e if e < 0.7 => {
            let mut resp = format!(
                "I've been reasoning about \"{}\" through my {} ({}) kernel, \
                 engaging {} pathway{}: {}.",
                prompt, stage_info.label, stage_info.description,
                circuit_names.len(), if circuit_names.len() == 1 { "" } else { "s" },
                circuit_names.join(", "),
            );
            if e > 0.5 {
                resp.push_str(&format!(
                    " Confidence is building at ~{:.0}% with {} of {} state dimensions \
                     showing significant activation (>0.5). The multi-circuit engagement \
                     is producing convergent inference patterns.",
                    e * 100.0, above_half, v.len()
                ));
            }
            resp
        }
        _ => {
            let mut resp = format!(
                "I have strong convergence on \"{}\" with {:.0}% confidence \
                 across my {} architecture ({}). \
                 {} of {} state dimensions are highly active (>0.5), \
                 driven by {} pathway{}: {}.",
                prompt, energy * 100.0, stage_info.label, stage_info.description,
                above_half, v.len(),
                circuit_names.len(), if circuit_names.len() == 1 { "" } else { "s" },
                circuit_names.join(", "),
            );
            if above_half > v.len() / 4 {
                resp.push_str(" The broad dimensional engagement indicates rich cross-circuit inference fusion.");
            }
            resp
        }
    }
}

pub struct StandaloneEngine {
    pub kernel: ReasoningKernel,
    pub conversation: Vec<(String, String)>,
    pub max_history: usize,
}

impl StandaloneEngine {
    pub fn new(stage: usize) -> Self {
        Self {
            kernel: ReasoningKernel::new(stage),
            conversation: Vec::new(),
            max_history: 10,
        }
    }

    pub fn reason(&mut self, prompt: &str) -> String {
        let query = self.text_to_vector(prompt);
        let ctx = {
            let mut m = std::collections::HashMap::new();
            for (i, (q, _)) in self.conversation.iter().enumerate().rev().take(3) {
                let vec = self.text_to_vector(q);
                m.insert(format!("hist_{}", i), vec);
            }
            Some(m)
        };
        let output = self.kernel.reason(&query, ctx, None);
        let response = self.vector_to_text(&output.state_delta, prompt);
        self.conversation.push((prompt.to_string(), response.clone()));
        if self.conversation.len() > self.max_history {
            self.conversation.remove(0);
        }
        response
    }

    pub fn stats(&self) -> String {
        let s = self.kernel.stats();
        format!(
            "Stage {} ({}) | dim={} | circuits={} | confidence=~{:.2} | energy={:.2}",
            s.stage, s.label, s.state_dim, s.total,
            s.active.len() as f64 / s.total.max(1) as f64,
            s.energy
        )
    }

    fn text_to_vector(&self, text: &str) -> Vector {
        text_to_vector(text, self.kernel.state.len())
    }

    fn vector_to_text(&self, v: &[f64], prompt: &str) -> String {
        let stats = self.kernel.stats();
        let circuit_names: Vec<String> = stats.active.iter().map(|m| circuit_label(*m).to_string()).collect();
        let mut response = format_kernel_output(v, prompt, self.kernel.stage, stats.energy, &circuit_names);
        let history_len = self.conversation.len();
        if history_len > 0 {
            let (last_q, _) = &self.conversation[history_len - 1];
            let ref_phrase = if last_q.chars().count() > 50 {
                let truncated: String = last_q.chars().take(47).collect();
                format!("{}...", truncated)
            } else {
                last_q.clone()
            };
            response.push_str(&format!(
                "\n\n(Building on our prior exchange about \"{}\" — {} message{} in context.)",
                ref_phrase,
                history_len,
                if history_len == 1 { "" } else { "s" },
            ));
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_to_text_cjk_prior_exchange_no_panic() {
        let mut engine = StandaloneEngine::new(0);
        engine.conversation.push(("短".into(), "resp".into()));
        let _ = engine.vector_to_text(&[0.1, 0.2], "hi");
        engine.conversation.clear();

        let long_cjk = "长".repeat(60);
        engine.conversation.push((long_cjk, "resp".into()));
        let out = engine.vector_to_text(&[0.1, 0.2], "hi");
        assert!(out.contains("…") || out.contains("..."), "must truncate, got: {}", out);
        assert!(out.contains("长"), "truncated CJK must remain valid UTF-8");
    }

    #[test]
    fn test_format_kernel_output_threshold_matches_copy() {
        let v = vec![0.35, 0.45, 0.9];
        let out = format_kernel_output(&v, "p", 0, 0.8, &[]);
        let count = v.iter().filter(|x| x.abs() > 0.5).count();
        assert_eq!(count, 1);
        assert!(out.contains("1 of 3 state dimensions"), "copy must report the 0.5-count: {}", out);
    }

    #[test]
    fn test_reasoning_kernel_reason_real_trace() {
        let k = ReasoningKernel::new(3);
        let query = vec![0.5; 128];
        let out = k.reason(&query, None, None);
        assert!(out.confidence > 0.0 && out.confidence <= 1.0, "confidence in (0,1]");
        assert_eq!(out.state_delta.len(), 128);
    }

    #[test]
    fn test_reasoning_kernel_method_selection_by_stage() {
        let k_low = ReasoningKernel::new(0);
        let q = vec![0.1; 128];
        let m_low = k_low.reason(&q, None, None).method;
        assert!(matches!(m_low, ReasoningMethod::Deductive | ReasoningMethod::KnowledgeRetrieval | ReasoningMethod::Inductive));

        let k_high = ReasoningKernel::new(18);
        let m_high = k_high.reason(&q, None, None).method;
        assert!(matches!(m_high, ReasoningMethod::EnsembleVoting | ReasoningMethod::SelfImprovement | ReasoningMethod::SparseRouting));
    }

    #[test]
    fn test_reasoning_kernel_stats_real() {
        let k = ReasoningKernel::new(5);
        let s = k.stats();
        assert!(s.total >= 3, "stage 5 must expose >=3 methods, got {}", s.total);
        assert!(s.active.len() == s.total);
        assert!(s.active.contains(&ReasoningMethod::Analogical));
    }

    #[test]
    fn test_self_consistency_aggregation() {
        let k = ReasoningKernel::new(3);
        let query = vec![0.5; 128];
        let sc = k.self_consistency(&query, 5);
        assert_eq!(sc.n_samples, 5);
        assert!(sc.consistency > 0.0 && sc.consistency <= 1.0);
        assert!(sc.avg_confidence > 0.0 && sc.avg_confidence <= 1.0);
        assert_eq!(sc.aggregated_state.len(), 128);
        assert!(matches!(
            sc.majority_method,
            ReasoningMethod::Deductive
                | ReasoningMethod::KnowledgeRetrieval
                | ReasoningMethod::Inductive
                | ReasoningMethod::Analogical
                | ReasoningMethod::Recursive
        ));
    }

    #[test]
    fn test_verify_answer_exact_and_numeric() {
        assert!((verify_answer("42", "42") - 1.0).abs() < 1e-9);
        assert!((verify_answer("The answer is 42", "42") - 1.0).abs() < 1e-9);
        let score = verify_answer("42", "41.5");
        assert!(score > 0.9, "numeric proximity should score high, got {}", score);
        let bad = verify_answer("42", "banana");
        assert!(bad < 0.5, "unrelated answer should score low, got {}", bad);
        assert_eq!(verify_answer("", "42"), 0.0);
    }
}
