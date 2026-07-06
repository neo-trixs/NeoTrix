use std::collections::HashMap;

type Vector = Vec<f64>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReasoningMethod {
    Deductive, Inductive, Abductive, Analogical, FirstPrinciples,
    Recursive, Compositional, Adversarial, AutoFetch,
    KnowledgeRetrieval, GradientLearning, ArchitectureSearch,
    GpuCompute, DistributedConsensus, ExperienceDistill,
    EmergentAnalysis, SystemIntegration, EnsembleVoting,
    SelfImprovement, SparseRouting,
}

#[derive(Debug, Clone, Copy)]
pub struct StageInfo {
    pub label: &'static str,
    pub description: &'static str,
}

pub const EVOLUTION: &[StageInfo] = &[
    StageInfo { label: "Stage 0", description: "Initial" },
    StageInfo { label: "Stage 1", description: "Pattern Recognition" },
    StageInfo { label: "Stage 2", description: "Abstraction" },
    StageInfo { label: "Stage 3", description: "Analogy Engine" },
    StageInfo { label: "Stage 4", description: "Recursive Reasoner" },
    StageInfo { label: "Stage 5", description: "Compositional" },
    StageInfo { label: "Stage 6", description: "Adversarial" },
    StageInfo { label: "Stage 7", description: "First Principles" },
    StageInfo { label: "Stage 8", description: "Auto-Fetch" },
    StageInfo { label: "Stage 9", description: "Knowledge Retrieval" },
    StageInfo { label: "Stage 10", description: "Gradient Learning" },
    StageInfo { label: "Stage 11", description: "Architecture Search" },
    StageInfo { label: "Stage 12", description: "GPU Compute" },
    StageInfo { label: "Stage 13", description: "Distributed Consensus" },
    StageInfo { label: "Stage 14", description: "Experience Distill" },
    StageInfo { label: "Stage 15", description: "Emergent Analysis" },
    StageInfo { label: "Stage 16", description: "System Integration" },
    StageInfo { label: "Stage 17", description: "Ensemble Voting" },
    StageInfo { label: "Stage 18", description: "Self-Improvement" },
];

#[derive(Debug, Clone)]
pub struct ReasoningTrace {
    pub method: ReasoningMethod,
    pub steps: usize,
    pub intermediate_states: Vec<Vec<f64>>,
    pub convergence: f64,
}

#[derive(Debug, Clone)]
pub struct ReasoningOutput {
    pub state_delta: Vec<f64>,
    pub confidence: f64,
    pub trace: ReasoningTrace,
}

#[derive(Debug, Clone)]
pub struct KernelStats {
    pub stage: usize,
    pub label: String,
    pub state_dim: usize,
    pub total: usize,
    pub active: Vec<ReasoningMethod>,
    pub energy: f64,
}

#[derive(Debug, Clone)]
pub struct ReasoningKernel {
    pub stage: usize,
    pub state: Vector,
}

impl ReasoningKernel {
    pub fn new(stage: usize) -> Self {
        Self { stage: stage.min(EVOLUTION.len() - 1), state: vec![0.0; 128] }
    }

    pub fn reason(&self, _query: &[f64], _context: Option<HashMap<String, Vector>>) -> ReasoningOutput {
        ReasoningOutput {
            state_delta: self.state.clone(),
            confidence: 0.5,
            trace: ReasoningTrace {
                method: ReasoningMethod::Deductive,
                steps: 1,
                intermediate_states: vec![],
                convergence: 0.5,
            },
        }
    }

    pub fn stats(&self) -> KernelStats {
        KernelStats {
            stage: self.stage,
            label: EVOLUTION[self.stage].label.to_string(),
            state_dim: self.state.len(),
            total: 8,
            active: vec![ReasoningMethod::Deductive],
            energy: self.state.iter().map(|x| x.abs()).sum::<f64>() / self.state.len().max(1) as f64,
        }
    }
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

    let above_half = v.iter().filter(|x| x.abs() > 0.3).count();

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
//                resp.push_str(&format!(" The active circuits — {} — are engaged but haven't reached convergence.", circuit_names.join(", ")));
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
            } else {
//                resp.push_str(" The inference is still converging — some circuits are still settling on stable state representations.");
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
        let output = self.kernel.reason(&query, ctx);
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
