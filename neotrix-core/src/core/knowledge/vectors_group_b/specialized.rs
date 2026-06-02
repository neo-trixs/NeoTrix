use crate::core::CapabilityVector;
use crate::core::knowledge::KnowledgeSource;

pub(super) fn cap_vec_specialized(s: &KnowledgeSource) -> CapabilityVector {
    match s {
        KnowledgeSource::IntegratedInformationTheory => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.4, 0.3,
                0.5, 0.4, 0.3, 0.4,
                0.95, 0.9, 0.92, 0.93, 0.95,
                0.3, 0.3, 0.2,
                0.3, 0.4, 0.3,
                0.7, 0.9, 0.92, 0.85,
            );
            cv.extend_named(&[
                ("integrated_information_phi".into(), 0.98),
                ("causal_power".into(), 0.95),
                ("phi_calculation".into(), 0.93),
                ("pyphi_compatibility".into(), 0.9),
                ("consciousness_metrics".into(), 0.92),
                ("quale_space".into(), 0.88),
                ("cause_effect_structure".into(), 0.95),
            ]);
            cv
        }
        KnowledgeSource::GlobalWorkspaceTheory => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.3, 0.2,
                0.4, 0.3, 0.2, 0.5,
                0.93, 0.85, 0.9, 0.92, 0.93,
                0.3, 0.3, 0.2,
                0.2, 0.3, 0.2,
                0.8, 0.9, 0.92, 0.88,
            );
            cv.extend_named(&[
                ("global_workspace_broadcast".into(), 0.97),
                ("competition_for_consciousness".into(), 0.95),
                ("ignition_dynamics".into(), 0.93),
                ("dehaene_global_neuronal".into(), 0.92),
                ("baars_theater_metaphor".into(), 0.9),
                ("conscious_access".into(), 0.88),
                ("widespread_brain_coherence".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::ActiveInference => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.4, 0.3,
                0.5, 0.5, 0.3, 0.4,
                0.94, 0.92, 0.93, 0.9, 0.95,
                0.3, 0.4, 0.3,
                0.2, 0.3, 0.2,
                0.9, 0.95, 0.93, 0.9,
            );
            cv.extend_named(&[
                ("free_energy_minimization".into(), 0.98),
                ("active_inference_loop".into(), 0.97),
                ("friston_fep".into(), 0.95),
                ("expected_free_energy".into(), 0.93),
                ("policy_selection".into(), 0.92),
                ("prior_posterior_update".into(), 0.9),
                ("generative_model".into(), 0.95),
                ("epistemic_foraging".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::VSAHyperdim => {
            let mut cv = CapabilityVector::from_values(
                0.4, 0.4, 0.5, 0.4,
                0.5, 0.5, 0.4, 0.5,
                0.92, 0.88, 0.9, 0.85, 0.92,
                0.4, 0.4, 0.3,
                0.3, 0.4, 0.3,
                0.85, 0.9, 0.9, 0.88,
            );
            cv.extend_named(&[
                ("hyperdimensional_computing".into(), 0.98),
                ("kanerva_hdv".into(), 0.97),
                ("bundle_bind_permute".into(), 0.96),
                ("cosine_similarity_memory".into(), 0.95),
                ("random_projection".into(), 0.93),
                ("vsa_binding".into(), 0.92),
                ("record_keeping".into(), 0.9),
                ("noise_tolerance".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::JEPAWorldModel => {
            let mut cv = CapabilityVector::from_values(
                0.4, 0.4, 0.5, 0.4,
                0.6, 0.5, 0.4, 0.4,
                0.95, 0.9, 0.93, 0.92, 0.94,
                0.4, 0.5, 0.3,
                0.3, 0.4, 0.3,
                0.95, 0.93, 0.92, 0.9,
            );
            cv.extend_named(&[
                ("joint_embedding".into(), 0.98),
                ("lecun_world_model".into(), 0.97),
                ("abstract_representation".into(), 0.95),
                ("energy_based_model".into(), 0.93),
                ("latent_variable_prediction".into(), 0.92),
                ("hierarchical_planning".into(), 0.9),
                ("variance_regularization".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::PredictiveCoding => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.4, 0.3,
                0.5, 0.4, 0.3, 0.4,
                0.93, 0.9, 0.92, 0.9, 0.93,
                0.3, 0.4, 0.3,
                0.2, 0.3, 0.2,
                0.88, 0.9, 0.92, 0.85,
            );
            cv.extend_named(&[
                ("prediction_error_minimization".into(), 0.98),
                ("hierarchical_inference".into(), 0.95),
                ("precision_weighting".into(), 0.93),
                ("rao_ballard_model".into(), 0.92),
                ("top_down_bottom_up".into(), 0.9),
                ("explaining_away".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::OrchOR => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.3, 0.2,
                0.3, 0.3, 0.2, 0.2,
                0.85, 0.8, 0.88, 0.85, 0.9,
                0.2, 0.2, 0.1,
                0.2, 0.2, 0.1,
                0.4, 0.8, 0.85, 0.7,
            );
            cv.extend_named(&[
                ("orchestrated_reduction".into(), 0.96),
                ("penrose_microtubule".into(), 0.95),
                ("quantum_computing_brain".into(), 0.93),
                ("hameroff_model".into(), 0.92),
                ("tubulin_dimer".into(), 0.9),
                ("objective_reduction".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::AttentionSchema => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.3, 0.2,
                0.4, 0.3, 0.3, 0.4,
                0.9, 0.85, 0.88, 0.9, 0.9,
                0.2, 0.3, 0.2,
                0.2, 0.2, 0.2,
                0.7, 0.85, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("attention_schema".into(), 0.97),
                ("graziano_model".into(), 0.95),
                ("self_as_attention_schema".into(), 0.93),
                ("awareness_control".into(), 0.92),
                ("social_attention".into(), 0.9),
                ("illusion_of_consciousness".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::SiaHarnessUpdate => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.2, 0.1, 0.1,
                0.3, 0.2, 0.2, 0.1,
                0.6, 0.7, 0.5, 0.6, 0.5,
                0.2, 0.3, 0.2,
                0.1, 0.2, 0.1,
                0.88, 0.82, 0.5, 0.4,
            );
            cv.extend_named(&[
                ("scaffold_rewrite".into(), 0.90),
                ("prompt_engineering".into(), 0.85),
                ("retry_logic".into(), 0.80),
            ]);
            cv
        }
        KnowledgeSource::SiaWeightUpdate => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.3, 0.1, 0.1,
                0.4, 0.2, 0.2, 0.2,
                0.85, 0.6, 0.7, 0.5, 0.5,
                0.2, 0.2, 0.2,
                0.1, 0.2, 0.1,
                0.4, 0.5, 0.85, 0.8,
            );
            cv.extend_named(&[
                ("rl_training".into(), 0.88),
                ("lora_adaptation".into(), 0.82),
                ("weight_update".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::SiaFeedbackLoop => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.2, 0.2, 0.1,
                0.3, 0.2, 0.3, 0.1,
                0.5, 0.6, 0.5, 0.7, 0.6,
                0.2, 0.2, 0.2,
                0.1, 0.1, 0.1,
                0.9, 0.85, 0.6, 0.5,
            );
            cv.extend_named(&[
                ("trajectory_analysis".into(), 0.92),
                ("improvement_rationale".into(), 0.85),
                ("feedback_routing".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::HyperAgents => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.3, 0.2, 0.2,
                0.4, 0.3, 0.3, 0.2,
                0.7, 0.8, 0.7, 0.8, 0.7,
                0.2, 0.3, 0.2,
                0.2, 0.2, 0.2,
                0.7, 0.6, 0.7, 0.6,
            );
            cv.extend_named(&[
                ("self_referential_loop".into(), 0.95),
                ("meta_agent_ability".into(), 0.90),
                ("population_diversity".into(), 0.88),
                ("code_diff_generation".into(), 0.90),
                ("sandbox_isolation".into(), 0.85),
                ("cross_domain_strategy".into(), 0.87),
            ]);
            cv
        }
        _ => unreachable!("general sources handled by general helper"),
    }
}
