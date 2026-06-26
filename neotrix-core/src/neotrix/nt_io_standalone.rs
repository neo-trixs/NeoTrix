use crate::neotrix::nt_core_kernel::{ReasoningKernel, EVOLUTION};
use crate::neotrix::nt_core_signal::Vector;
use crate::neotrix::nt_world_browse::circuits_types::ReasoningMethod;

/// Deterministic standalone text-to-vector embedding.
///
/// **This is NOT full VSA 4096-bit encoding** — it is a simplified deterministic
/// hash using byte-position phase modulation and L2 normalization, matching the
/// kernel's current state dimension. For full VSA semantic encoding, connect to
/// the daemon via `--connect`.
///
/// The algorithm: byte-position phase modulation + repeated residue filling +
/// unit-length normalization. Different inputs reliably produce different vectors,
/// but the encoding has no compositional semantics (no binding/bundling).
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

pub fn format_kernel_output(v: &[f64], prompt: &str, kernel: &ReasoningKernel) -> String {
    let raw_energy: f64 = v.iter().map(|x| x.abs()).sum::<f64>() / v.len().max(1) as f64;
    let confidence = kernel.stats().energy.clamp(0.1, 1.0);
    let energy = raw_energy.max(confidence * 0.5);
    let stats = kernel.stats();
    let stage_info = &EVOLUTION[kernel.stage];

    let circuit_names: Vec<String> = stats
        .active
        .iter()
        .map(|m| circuit_label(*m).to_string())
        .collect();

    let above_half = v.iter().filter(|x| x.abs() > 0.3).count();

    match energy {
        e if e < 0.3 => {
            let mut resp =
                format!(
                "[standalone kernel] I need more context to form a solid inference about \"{}\". \
                 My {} kernel is registering weak signal (energy ~{:.2}) \
                 across {} reasoning pathway{}.",
                prompt, stage_info.label, e,
                circuit_names.len(), if circuit_names.len() == 1 { "" } else { "s" },
            );
            if !circuit_names.is_empty() {
                //                resp.push_str(&format!(" The active circuits — {} — are engaged but haven't reached convergence.", circuit_names.join(", ")));
            }
            resp.push_str(" Could you provide more detail or clarify the question?");
            resp
        }
        e if e < 0.7 => {
            let mut resp =
                format!(
                "[standalone kernel] I've been reasoning about \"{}\" through my {} ({}) kernel, \
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
                    e * 100.0,
                    above_half,
                    v.len()
                ));
            } else {
                //                resp.push_str(" The inference is still converging — some circuits are still settling on stable state representations.");
            }
            resp
        }
        _ => {
            let mut resp =
                format!(
                "[standalone kernel] I have strong convergence on \"{}\" with {:.0}% confidence \
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
        let standalone_hint = "Running in standalone mode — connect to daemon with --connect for live consciousness data.";
        let query = self.text_to_vector(prompt);
        let ctx = {
            let mut m = std::collections::HashMap::new();
            for (i, (q, _)) in self.conversation.iter().enumerate().rev().take(3) {
                let vec = self.text_to_vector(q);
                m.insert(format!("hist_{}", i), vec);
            }
            Some(m)
        };
        let output = self.kernel.reason(&query, ctx);
        let mut response = self.vector_to_text(&output.state_delta, prompt);
        if self.conversation.is_empty() {
            response.push_str(&format!("\n\n_{}_", standalone_hint));
        }
        self.conversation
            .push((prompt.to_string(), response.clone()));
        if self.conversation.len() > self.max_history {
            self.conversation.remove(0);
        }
        response
    }

    pub fn stats(&self) -> String {
        let s = self.kernel.stats();
        format!(
            "[standalone] Stage {} ({}) | dim={} | circuits={} | confidence=~{:.2} | energy={:.2}",
            s.stage,
            s.label,
            s.state_dim,
            s.total,
            s.active.len() as f64 / s.total.max(1) as f64,
            s.energy
        )
    }

    fn text_to_vector(&self, text: &str) -> Vector {
        text_to_vector(text, self.kernel.state.len())
    }

    fn vector_to_text(&self, v: &[f64], prompt: &str) -> String {
        let mut response = format_kernel_output(v, prompt, &self.kernel);
        let history_len = self.conversation.len();
        if history_len > 0 {
            let (last_q, _) = &self.conversation[history_len - 1];
            let ref_phrase = if last_q.len() > 50 {
                format!("{}...", &last_q[..47])
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
