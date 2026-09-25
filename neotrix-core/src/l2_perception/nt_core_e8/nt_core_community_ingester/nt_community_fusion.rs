//! Community fusion: `fuse_*` priors + task-type heuristic.
//! Pure move from `nt_core_community_ingester.rs`; behavior unchanged.

use super::nt_community_data::{CommunityDataIngester, CommunityDataset};
use crate::l2_perception::nt_core_e8::E8TransitionMatrix;
use crate::l2_perception::nt_core_e8::domain_transition::{E8DomainTransitionModel, E8TaskType};

impl CommunityDataset {
    /// Fuse this dataset's transition patterns into a transition matrix.
    /// Each (from, to, count) triple adds `weight * count` virtual observations.
    pub fn fuse_into(&self, tm: &mut E8TransitionMatrix) {
        for &(from, to, count) in &self.transitions {
            let virtual_count = (self.weight * count as f64).round() as u64;
            for _ in 0..virtual_count {
                tm.record_transition(from, to);
            }
        }
    }
}

impl CommunityDataIngester {
    /// Fuse all community datasets into a single transition matrix.
    /// Returns a matrix containing the cumulative community-derived
    /// transition counts across all datasets.
    pub fn fuse_all(&self) -> E8TransitionMatrix {
        let mut fused = E8TransitionMatrix::new();
        for ds in &self.datasets {
            ds.fuse_into(&mut fused);
        }
        fused
    }

    /// Fuse community data into the domain transition model.
    /// Each dataset's transitions are injected into the appropriate
    /// domain sub-matrix based on the dataset's task type affinity.
    pub fn fuse_into_domain(&self, dtm: &mut E8DomainTransitionModel) {
        for ds in &self.datasets {
            for &(from, to, count) in &ds.transitions {
                // Clamp weight so a pathological deserialized config cannot
                // expand into a billion-iteration injection loop.
                let weight = ds.weight.clamp(0.0, 10.0);
                let virtual_count = (weight * count as f64).round() as u64;
                // Record into the general matrix
                for _ in 0..virtual_count {
                    dtm.general_matrix.record_transition(from, to);
                }
                // Also record into task-type-specific matrices (skip General:
                // all transitions already go into general_matrix above)
                for task_type in &E8TaskType::ALL {
                    if *task_type == E8TaskType::General {
                        continue;
                    }
                    if ds.name.contains(task_type.label())
                        || matches_task_type(ds.name.as_str(), *task_type)
                    {
                        for _ in 0..(virtual_count / 2).max(1) {
                            dtm.record_transition(*task_type, from, to);
                        }
                    }
                }
            }
        }
    }

    /// Total number of virtual observations across all datasets.
    pub fn total_virtual_observations(&self) -> u64 {
        let mut total = 0u64;
        for ds in &self.datasets {
            for &(_, _, count) in &ds.transitions {
                total += (ds.weight * count as f64).round() as u64;
            }
        }
        total
    }
}

/// Heuristic mapping from dataset name to E8TaskType for domain injection.
fn matches_task_type(name: &str, task_type: E8TaskType) -> bool {
    let key = name.to_lowercase();
    match task_type {
        E8TaskType::General => false,
        E8TaskType::Reasoning => {
            key.contains("reason")
                || key.contains("fable")
                || key.contains("distill")
                || key.contains("fuse")
                || key.contains("thought")
                || key.contains("scaler")
                || key.contains("god_seed")
                || key.contains("thinking")
                || key.contains("self_rewriting")
                || key.contains("chronos")
                || key.contains("nova")
                || key.contains("ouroboros")
                || key.contains("stratos")
                || key.contains("noesis")
        }
        E8TaskType::Math => key.contains("math"),
        E8TaskType::Coding => {
            key.contains("code")
                || key.contains("coding")
                || key.contains("swe")
                || key.contains("algorithmic")
        }
        E8TaskType::Agentic => {
            key.contains("agent") || key.contains("edge_agent") || key.contains("tool_use")
        }
        E8TaskType::Creative => {
            key.contains("creative") || key.contains("story") || key.contains("character")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_perception::nt_core_e8::nt_core_community_ingester::{
        CommunityDataIngester, CommunityDataset,
    };
    use crate::l2_perception::nt_core_e8::domain_transition::E8TaskType;

    #[test]
    fn test_community_dataset_fuse_into() {
        let ds = CommunityDataset {
            name: "test".into(),
            source_url: "https://example.com".into(),
            weight: 1.0,
            description: "test".into(),
            transitions: vec![(56, 48, 100), (48, 40, 50)],
        };
        let mut tm = E8TransitionMatrix::new();
        ds.fuse_into(&mut tm);
        assert!(tm.row_totals.0[56] >= 100);
        assert!(tm.row_totals.0[48] >= 50);
    }

    #[test]
    fn test_fuse_all_accumulates() {
        let ingester = CommunityDataIngester::default();
        let fused = ingester.fuse_all();
        let total: u64 = fused.visit_counts.0.iter().sum();
        assert!(
            total > 1000,
            "should fuse many virtual observations, got {}",
            total
        );
    }

    #[test]
    fn test_matches_task_type() {
        assert!(matches_task_type(
            "Complete-FABLE.5-traces-2M",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type("agentic_coding_15k", E8TaskType::Coding));
        assert!(matches_task_type(
            "agentic_distill_10k",
            E8TaskType::Agentic
        ));
        assert!(matches_task_type(
            "fable5_distillation_25k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "gemini_3_1_pro_reasoning_5_6m",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type("deepseek_r1_math_12k", E8TaskType::Math));
        assert!(matches_task_type(
            "creative_story_writing_5k",
            E8TaskType::Creative
        ));
        assert!(matches_task_type(
            "nvidia_open_swe_traces_207k",
            E8TaskType::Coding
        ));
        assert!(matches_task_type(
            "fable5_sft_traces_kelexine_4k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "priming_hybrid_ssm_fable",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "retrieval_aware_distill_ssm",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "ansulev_fable_sft_combined_v2",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "fable5_glm52_expanded_10k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "king3djbl_fable5_multisource_11k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "claude_mythos_distilled_25k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type("openr1_math_220k", E8TaskType::Math));
        assert!(matches_task_type(
            "fuseo1_deepseek_qwq_sft",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type("openmath_instruct_2", E8TaskType::Math));
        assert!(matches_task_type(
            "thoughts_v0_5_cot",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "scaler_r1_data_17k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "magpie_deepseek_reasoning_1m",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "helioai_deepreason_462x105m",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "kelexine_fable5_sft_traces_4k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "snow_opus47_reasoning_8k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "shijunhao_fable5_pi_traces",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "helioai_mythos_v2_full_distill",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "opus47_god_seed_reasoning_25k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "self_rewriting_metalean_25k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "grok44_god_seed_truth_25k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "chronos_thinking_v1_mini",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "nova_reasoning_correction_4k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "ouroboros_self_improving",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "bespoke_stratos_17k",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "noesis_50k_multilingual_moe",
            E8TaskType::Reasoning
        ));
        assert!(matches_task_type(
            "algorithmic_sft_distill_24k",
            E8TaskType::Coding
        ));
        assert!(matches_task_type(
            "edge_agent_websearch_260k",
            E8TaskType::Agentic
        ));
        assert!(matches_task_type(
            "numinamath_cot_distill_100k",
            E8TaskType::Math
        ));
        assert!(matches_task_type(
            "gemma4_fable5_distilled",
            E8TaskType::Reasoning
        ));
    }

    #[test]
    fn test_fuse_into_domain_adds_to_general() {
        let mut dtm = E8DomainTransitionModel::new(0.3);
        let ingester = CommunityDataIngester::default();
        let before: u64 = dtm.general_matrix.visit_counts.0.iter().sum();
        ingester.fuse_into_domain(&mut dtm);
        let after: u64 = dtm.general_matrix.visit_counts.0.iter().sum();
        assert!(after > before, "general matrix should gain observations");
    }
}
