#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum ObservableCategory {
    Weight,
    Gradient,
    Representation,
    Attention,
    Loss,
    Architecture,
    Optimization,
    Memory,
    Behavior,
}

#[derive(Debug, Clone)]
pub struct ObservableId {
    pub name: &'static str,
    pub category: ObservableCategory,
    pub description: &'static str,
}

pub struct ObservablesRegistry {
    observables: Vec<ObservableId>,
}

impl ObservablesRegistry {
    pub fn new() -> Self {
        let mut o = Vec::new();

        // Weight (26)
        o.push(ObservableId {
            name: "weight_l1",
            category: ObservableCategory::Weight,
            description: "L1 norm of all weight parameters",
        });
        o.push(ObservableId {
            name: "weight_l2",
            category: ObservableCategory::Weight,
            description: "L2 norm of all weight parameters",
        });
        o.push(ObservableId {
            name: "weight_update_norm",
            category: ObservableCategory::Weight,
            description: "Norm of the weight update delta",
        });
        o.push(ObservableId {
            name: "weight_spectral_radius",
            category: ObservableCategory::Weight,
            description: "Spectral radius of weight matrices",
        });
        o.push(ObservableId {
            name: "weight_rank",
            category: ObservableCategory::Weight,
            description: "Effective rank of weight matrices",
        });
        o.push(ObservableId {
            name: "weight_sparsity",
            category: ObservableCategory::Weight,
            description: "Fraction of zero-valued weights",
        });
        o.push(ObservableId {
            name: "weight_frobenius",
            category: ObservableCategory::Weight,
            description: "Frobenius norm of weight tensors",
        });
        o.push(ObservableId {
            name: "weight_max",
            category: ObservableCategory::Weight,
            description: "Maximum weight value across all parameters",
        });
        o.push(ObservableId {
            name: "weight_min",
            category: ObservableCategory::Weight,
            description: "Minimum weight value across all parameters",
        });
        o.push(ObservableId {
            name: "weight_mean",
            category: ObservableCategory::Weight,
            description: "Mean of all weight values",
        });
        o.push(ObservableId {
            name: "weight_std",
            category: ObservableCategory::Weight,
            description: "Standard deviation of weight values",
        });
        o.push(ObservableId {
            name: "weight_final_layer_norm",
            category: ObservableCategory::Weight,
            description: "Norm of the final layer weights",
        });
        o.push(ObservableId {
            name: "weight_embedding_norm",
            category: ObservableCategory::Weight,
            description: "Norm of the embedding layer weights",
        });
        o.push(ObservableId {
            name: "weight_bias_norm",
            category: ObservableCategory::Weight,
            description: "Norm of all bias parameters",
        });
        o.push(ObservableId {
            name: "weight_gate_ratio",
            category: ObservableCategory::Weight,
            description: "Ratio of gate to update gate magnitudes",
        });
        o.push(ObservableId {
            name: "weight_kernel_norm",
            category: ObservableCategory::Weight,
            description: "Norm of convolutional kernel weights",
        });
        o.push(ObservableId {
            name: "weight_output_norm",
            category: ObservableCategory::Weight,
            description: "Norm of output projection weights",
        });
        o.push(ObservableId {
            name: "weight_condition_number",
            category: ObservableCategory::Weight,
            description: "Condition number of weight matrices",
        });
        o.push(ObservableId {
            name: "weight_sign_asymmetry",
            category: ObservableCategory::Weight,
            description: "Ratio of positive to negative weights",
        });
        o.push(ObservableId {
            name: "weight_magnitude_growth",
            category: ObservableCategory::Weight,
            description: "Rate of weight magnitude increase",
        });
        o.push(ObservableId {
            name: "weight_layer_imbalance",
            category: ObservableCategory::Weight,
            description: "Variance of norms across layers",
        });
        o.push(ObservableId {
            name: "weight_unit_norm_deviation",
            category: ObservableCategory::Weight,
            description: "Deviation from unit norm in normalized layers",
        });
        o.push(ObservableId {
            name: "weight_exponential_moving_average",
            category: ObservableCategory::Weight,
            description: "EMA of weight values over training steps",
        });
        o.push(ObservableId {
            name: "weight_svd_energy",
            category: ObservableCategory::Weight,
            description: "Fraction of singular value energy in top components",
        });
        o.push(ObservableId {
            name: "weight_quantization_error",
            category: ObservableCategory::Weight,
            description: "Error introduced by weight quantization",
        });
        o.push(ObservableId {
            name: "weight_pruning_sensitivity",
            category: ObservableCategory::Weight,
            description: "Loss increase per pruned weight magnitude",
        });

        // Gradient (16)
        o.push(ObservableId {
            name: "gradient_norm",
            category: ObservableCategory::Gradient,
            description: "Global gradient norm",
        });
        o.push(ObservableId {
            name: "gradient_noise_scale",
            category: ObservableCategory::Gradient,
            description: "Scale of gradient noise estimated across batches",
        });
        o.push(ObservableId {
            name: "gradient_variance",
            category: ObservableCategory::Gradient,
            description: "Variance of gradient estimates across batches",
        });
        o.push(ObservableId {
            name: "gradient_max",
            category: ObservableCategory::Gradient,
            description: "Maximum gradient element magnitude",
        });
        o.push(ObservableId {
            name: "gradient_min",
            category: ObservableCategory::Gradient,
            description: "Minimum gradient element magnitude",
        });
        o.push(ObservableId {
            name: "gradient_update_ratio",
            category: ObservableCategory::Gradient,
            description: "Ratio of update norm to parameter norm",
        });
        o.push(ObservableId {
            name: "gradient_sparsity",
            category: ObservableCategory::Gradient,
            description: "Fraction of zero-valued gradients",
        });
        o.push(ObservableId {
            name: "gradient_rank",
            category: ObservableCategory::Gradient,
            description: "Effective rank of the gradient matrix",
        });
        o.push(ObservableId {
            name: "gradient_conflict",
            category: ObservableCategory::Gradient,
            description: "Cosine similarity between conflicting gradient directions",
        });
        o.push(ObservableId {
            name: "gradient_orthogonality",
            category: ObservableCategory::Gradient,
            description: "Degree of orthogonality between gradient vectors",
        });
        o.push(ObservableId {
            name: "gradient_layer_norm",
            category: ObservableCategory::Gradient,
            description: "Per-layer gradient norm distribution",
        });
        o.push(ObservableId {
            name: "gradient_batch_variance",
            category: ObservableCategory::Gradient,
            description: "Variance of gradients within a batch",
        });
        o.push(ObservableId {
            name: "gradient_explosion_ratio",
            category: ObservableCategory::Gradient,
            description: "Ratio of gradient norms exceeding threshold",
        });
        o.push(ObservableId {
            name: "gradient_align_quality",
            category: ObservableCategory::Gradient,
            description: "Alignment of minibatch gradients with full batch",
        });
        o.push(ObservableId {
            name: "gradient_hessian_product",
            category: ObservableCategory::Gradient,
            description: "Gradient norm after Hessian-vector product",
        });
        o.push(ObservableId {
            name: "gradient_direction_consistency",
            category: ObservableCategory::Gradient,
            description: "Cosine similarity of gradients across consecutive steps",
        });

        // Representation (18)
        o.push(ObservableId {
            name: "representation_similarity",
            category: ObservableCategory::Representation,
            description: "CKA similarity between layers",
        });
        o.push(ObservableId {
            name: "representation_entropy",
            category: ObservableCategory::Representation,
            description: "Entropy of hidden state distributions",
        });
        o.push(ObservableId {
            name: "representation_rank",
            category: ObservableCategory::Representation,
            description: "Effective rank of hidden representations",
        });
        o.push(ObservableId {
            name: "representation_sparsity",
            category: ObservableCategory::Representation,
            description: "Sparsity of hidden layer activations",
        });
        o.push(ObservableId {
            name: "neuron_activation_rate",
            category: ObservableCategory::Representation,
            description: "Fraction of neurons with nonzero activation",
        });
        o.push(ObservableId {
            name: "neuron_dead_ratio",
            category: ObservableCategory::Representation,
            description: "Fraction of neurons that never activate",
        });
        o.push(ObservableId {
            name: "neuron_saturation",
            category: ObservableCategory::Representation,
            description: "Fraction of neurons in saturation regime",
        });
        o.push(ObservableId {
            name: "hidden_state_norm",
            category: ObservableCategory::Representation,
            description: "Norm of hidden state vectors",
        });
        o.push(ObservableId {
            name: "embedding_cosine_sim",
            category: ObservableCategory::Representation,
            description: "Average cosine similarity between embeddings",
        });
        o.push(ObservableId {
            name: "representation_dimensionality",
            category: ObservableCategory::Representation,
            description: "Intrinsic dimensionality of representations",
        });
        o.push(ObservableId {
            name: "representation_cluster_count",
            category: ObservableCategory::Representation,
            description: "Number of clusters in representation space",
        });
        o.push(ObservableId {
            name: "representation_mutual_information",
            category: ObservableCategory::Representation,
            description: "Mutual information between layers",
        });
        o.push(ObservableId {
            name: "representation_contrast",
            category: ObservableCategory::Representation,
            description: "Average distance between class prototypes",
        });
        o.push(ObservableId {
            name: "representation_evolution_speed",
            category: ObservableCategory::Representation,
            description: "Rate of change in representations over steps",
        });
        o.push(ObservableId {
            name: "representation_collapse",
            category: ObservableCategory::Representation,
            description: "Degree of representational collapse",
        });
        o.push(ObservableId {
            name: "representation_manifold_curvature",
            category: ObservableCategory::Representation,
            description: "Curvature of the representation manifold",
        });
        o.push(ObservableId {
            name: "representation_proto_stability",
            category: ObservableCategory::Representation,
            description: "Stability of class prototypes over time",
        });
        o.push(ObservableId {
            name: "representation_orthogonality",
            category: ObservableCategory::Representation,
            description: "Degree of feature axis orthogonality",
        });

        // Attention (15)
        o.push(ObservableId {
            name: "attention_entropy",
            category: ObservableCategory::Attention,
            description: "Entropy of attention distribution",
        });
        o.push(ObservableId {
            name: "attention_concentration",
            category: ObservableCategory::Attention,
            description: "Concentration of attention weights on few positions",
        });
        o.push(ObservableId {
            name: "attention_head_similarity",
            category: ObservableCategory::Attention,
            description: "Similarity between attention heads",
        });
        o.push(ObservableId {
            name: "attention_sparsity",
            category: ObservableCategory::Attention,
            description: "Fraction of near-zero attention weights",
        });
        o.push(ObservableId {
            name: "attention_induction_strength",
            category: ObservableCategory::Attention,
            description: "Strength of induction head patterns",
        });
        o.push(ObservableId {
            name: "attention_context_utilization",
            category: ObservableCategory::Attention,
            description: "Fraction of context positions attended to",
        });
        o.push(ObservableId {
            name: "attention_position_bias",
            category: ObservableCategory::Attention,
            description: "Bias toward recent or early positions",
        });
        o.push(ObservableId {
            name: "attention_layer_diversity",
            category: ObservableCategory::Attention,
            description: "Diversity of attention patterns across layers",
        });
        o.push(ObservableId {
            name: "attention_pattern_rank",
            category: ObservableCategory::Attention,
            description: "Rank of the attention pattern matrix",
        });
        o.push(ObservableId {
            name: "attention_kernel_smoothness",
            category: ObservableCategory::Attention,
            description: "Smoothness of the attention kernel",
        });
        o.push(ObservableId {
            name: "attention_rollback",
            category: ObservableCategory::Attention,
            description: "Rate of attention pattern reversion",
        });
        o.push(ObservableId {
            name: "attention_attribution",
            category: ObservableCategory::Attention,
            description: "Attribution of output to input positions",
        });
        o.push(ObservableId {
            name: "attention_vertical_pattern",
            category: ObservableCategory::Attention,
            description: "Vertical attention pattern strength across layers",
        });
        o.push(ObservableId {
            name: "attention_sink_strength",
            category: ObservableCategory::Attention,
            description: "Strength of attention sink on initial tokens",
        });
        o.push(ObservableId {
            name: "attention_compression_ratio",
            category: ObservableCategory::Attention,
            description: "Ratio of context compressed via attention",
        });

        // Loss (14)
        o.push(ObservableId {
            name: "training_loss",
            category: ObservableCategory::Loss,
            description: "Training loss value",
        });
        o.push(ObservableId {
            name: "validation_loss",
            category: ObservableCategory::Loss,
            description: "Validation loss value",
        });
        o.push(ObservableId {
            name: "generalization_gap",
            category: ObservableCategory::Loss,
            description: "Difference between validation and training loss",
        });
        o.push(ObservableId {
            name: "loss_curvature",
            category: ObservableCategory::Loss,
            description: "Curvature of the loss landscape",
        });
        o.push(ObservableId {
            name: "loss_landscape_sharpness",
            category: ObservableCategory::Loss,
            description: "Sharpness of the loss minimum",
        });
        o.push(ObservableId {
            name: "loss_zcr",
            category: ObservableCategory::Loss,
            description: "Zero-crossing rate of loss derivative",
        });
        o.push(ObservableId {
            name: "perplexity",
            category: ObservableCategory::Loss,
            description: "Model perplexity on validation data",
        });
        o.push(ObservableId {
            name: "loss_var",
            category: ObservableCategory::Loss,
            description: "Variance of loss across batches",
        });
        o.push(ObservableId {
            name: "loss_autocorrelation",
            category: ObservableCategory::Loss,
            description: "Autocorrelation of the loss sequence",
        });
        o.push(ObservableId {
            name: "loss_lyapunov",
            category: ObservableCategory::Loss,
            description: "Lyapunov exponent of loss dynamics",
        });
        o.push(ObservableId {
            name: "loss_batch_variance",
            category: ObservableCategory::Loss,
            description: "Variance of loss within a batch",
        });
        o.push(ObservableId {
            name: "loss_update_efficiency",
            category: ObservableCategory::Loss,
            description: "Ratio of loss reduction to gradient norm",
        });
        o.push(ObservableId {
            name: "loss_outlier_fraction",
            category: ObservableCategory::Loss,
            description: "Fraction of examples with anomalously high loss",
        });
        o.push(ObservableId {
            name: "loss_consistency",
            category: ObservableCategory::Loss,
            description: "Consistency of loss across data subgroups",
        });

        // Architecture (11)
        o.push(ObservableId {
            name: "effective_width",
            category: ObservableCategory::Architecture,
            description: "Effective width considering dead neurons",
        });
        o.push(ObservableId {
            name: "effective_depth",
            category: ObservableCategory::Architecture,
            description: "Effective depth considering gradient propagation",
        });
        o.push(ObservableId {
            name: "gradient_flow",
            category: ObservableCategory::Architecture,
            description: "Ratio of gradient norm at early vs late layers",
        });
        o.push(ObservableId {
            name: "skip_connection_utilization",
            category: ObservableCategory::Architecture,
            description: "Magnitude of skip connection contributions",
        });
        o.push(ObservableId {
            name: "layer_similarity",
            category: ObservableCategory::Architecture,
            description: "Representational similarity between consecutive layers",
        });
        o.push(ObservableId {
            name: "jacobian_rank",
            category: ObservableCategory::Architecture,
            description: "Rank of the input-output Jacobian",
        });
        o.push(ObservableId {
            name: "neural_network_entropy",
            category: ObservableCategory::Architecture,
            description: "Entropy of the network weight distribution",
        });
        o.push(ObservableId {
            name: "activation_statistics",
            category: ObservableCategory::Architecture,
            description: "Statistical moments of layer activations",
        });
        o.push(ObservableId {
            name: "path_norm",
            category: ObservableCategory::Architecture,
            description: "Norm of all paths through the network",
        });
        o.push(ObservableId {
            name: "module_utilization",
            category: ObservableCategory::Architecture,
            description: "Fraction of architectural modules actively used",
        });
        o.push(ObservableId {
            name: "expressivity_measure",
            category: ObservableCategory::Architecture,
            description: "Measure of network function complexity",
        });

        // Optimization (13)
        o.push(ObservableId {
            name: "learning_rate_effective",
            category: ObservableCategory::Optimization,
            description: "Effective learning rate after adaptive scaling",
        });
        o.push(ObservableId {
            name: "momentum_accumulation",
            category: ObservableCategory::Optimization,
            description: "Magnitude of momentum buffer",
        });
        o.push(ObservableId {
            name: "adaptive_rate_balance",
            category: ObservableCategory::Optimization,
            description: "Balance of per-parameter adaptive rates",
        });
        o.push(ObservableId {
            name: "weight_decay_impact",
            category: ObservableCategory::Optimization,
            description: "Magnitude of weight decay contribution",
        });
        o.push(ObservableId {
            name: "batch_size_utilization",
            category: ObservableCategory::Optimization,
            description: "Effective batch size after gradient accumulation",
        });
        o.push(ObservableId {
            name: "update_to_noise_ratio",
            category: ObservableCategory::Optimization,
            description: "Ratio of meaningful update to gradient noise",
        });
        o.push(ObservableId {
            name: "convergence_speed",
            category: ObservableCategory::Optimization,
            description: "Rate of convergence measured by loss decay",
        });
        o.push(ObservableId {
            name: "optimality_gap",
            category: ObservableCategory::Optimization,
            description: "Distance to estimated optimal loss",
        });
        o.push(ObservableId {
            name: "train_speed",
            category: ObservableCategory::Optimization,
            description: "Examples per second throughput",
        });
        o.push(ObservableId {
            name: "saturation_rate",
            category: ObservableCategory::Optimization,
            description: "Fraction of parameters in saturated regime",
        });
        o.push(ObservableId {
            name: "learning_rate_warmup_progress",
            category: ObservableCategory::Optimization,
            description: "Progress through LR warmup schedule",
        });
        o.push(ObservableId {
            name: "loss_plateau_detection",
            category: ObservableCategory::Optimization,
            description: "Number of steps since last loss improvement",
        });
        o.push(ObservableId {
            name: "gradient_step_quality",
            category: ObservableCategory::Optimization,
            description: "Cosine similarity between update and gradient",
        });

        // Memory (11)
        o.push(ObservableId {
            name: "memory_utilization",
            category: ObservableCategory::Memory,
            description: "Fraction of memory capacity in use",
        });
        o.push(ObservableId {
            name: "memory_fragmentation",
            category: ObservableCategory::Memory,
            description: "Degree of memory fragmentation",
        });
        o.push(ObservableId {
            name: "cache_hit_rate",
            category: ObservableCategory::Memory,
            description: "Rate of cache hits in memory system",
        });
        o.push(ObservableId {
            name: "retrieval_latency",
            category: ObservableCategory::Memory,
            description: "Average latency of memory retrieval",
        });
        o.push(ObservableId {
            name: "eviction_rate",
            category: ObservableCategory::Memory,
            description: "Rate of memory entry eviction",
        });
        o.push(ObservableId {
            name: "surprise_score",
            category: ObservableCategory::Memory,
            description: "Average surprise of stored experiences",
        });
        o.push(ObservableId {
            name: "novelty_ratio",
            category: ObservableCategory::Memory,
            description: "Fraction of novel entries in memory",
        });
        o.push(ObservableId {
            name: "forgetting_rate",
            category: ObservableCategory::Memory,
            description: "Rate at which stored memories decay",
        });
        o.push(ObservableId {
            name: "memory_compression_ratio",
            category: ObservableCategory::Memory,
            description: "Ratio of original to compressed memory size",
        });
        o.push(ObservableId {
            name: "memory_access_frequency",
            category: ObservableCategory::Memory,
            description: "Distribution of memory access frequencies",
        });
        o.push(ObservableId {
            name: "memory_consolidation_urgency",
            category: ObservableCategory::Memory,
            description: "Urgency score for memory consolidation",
        });

        // Behavior (13)
        o.push(ObservableId {
            name: "accuracy",
            category: ObservableCategory::Behavior,
            description: "Model accuracy on validation set",
        });
        o.push(ObservableId {
            name: "error_consistency",
            category: ObservableCategory::Behavior,
            description: "Consistency of errors across similar inputs",
        });
        o.push(ObservableId {
            name: "prediction_confidence",
            category: ObservableCategory::Behavior,
            description: "Average softmax confidence of predictions",
        });
        o.push(ObservableId {
            name: "calibration_error",
            category: ObservableCategory::Behavior,
            description: "Expected calibration error",
        });
        o.push(ObservableId {
            name: "uncertainty",
            category: ObservableCategory::Behavior,
            description: "Predictive uncertainty estimate",
        });
        o.push(ObservableId {
            name: "decision_speed",
            category: ObservableCategory::Behavior,
            description: "Average inference time per example",
        });
        o.push(ObservableId {
            name: "exploration_rate",
            category: ObservableCategory::Behavior,
            description: "Rate of exploratory vs exploitative actions",
        });
        o.push(ObservableId {
            name: "mode_collapse",
            category: ObservableCategory::Behavior,
            description: "Degree of output mode collapse",
        });
        o.push(ObservableId {
            name: "behavioral_diversity",
            category: ObservableCategory::Behavior,
            description: "Diversity of model behaviors across inputs",
        });
        o.push(ObservableId {
            name: "generalization_quality",
            category: ObservableCategory::Behavior,
            description: "Quality of generalization to unseen data",
        });
        o.push(ObservableId {
            name: "robustness",
            category: ObservableCategory::Behavior,
            description: "Robustness to input perturbations",
        });
        o.push(ObservableId {
            name: "fairness_metric",
            category: ObservableCategory::Behavior,
            description: "Demographic parity or equalized odds",
        });
        o.push(ObservableId {
            name: "distribution_shift_sensitivity",
            category: ObservableCategory::Behavior,
            description: "Sensitivity of outputs to distribution shift",
        });

        ObservablesRegistry { observables: o }
    }

    pub fn by_category(&self, cat: &ObservableCategory) -> Vec<&ObservableId> {
        self.observables
            .iter()
            .filter(|o| &o.category == cat)
            .collect()
    }

    pub fn by_name(&self, name: &str) -> Option<&ObservableId> {
        self.observables.iter().find(|o| o.name == name)
    }

    pub fn all(&self) -> &[ObservableId] {
        &self.observables
    }

    pub fn count(&self) -> usize {
        self.observables.len()
    }

    pub fn categories(&self) -> Vec<ObservableCategory> {
        use ObservableCategory::*;
        vec![
            Weight,
            Gradient,
            Representation,
            Attention,
            Loss,
            Architecture,
            Optimization,
            Memory,
            Behavior,
        ]
    }

    pub fn search(&self, query: &str) -> Vec<&ObservableId> {
        let q = query.to_lowercase();
        self.observables
            .iter()
            .filter(|o| {
                o.name.to_lowercase().contains(&q) || o.description.to_lowercase().contains(&q)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_at_least_100_entries() {
        let reg = ObservablesRegistry::new();
        assert!(
            reg.count() >= 100,
            "expected >= 100 observables, got {}",
            reg.count()
        );
    }

    #[test]
    fn by_category_returns_correct_count() {
        let reg = ObservablesRegistry::new();
        let weight_count = reg.by_category(&ObservableCategory::Weight).len();
        assert!(
            weight_count >= 15,
            "expected >= 15 weight observables, got {}",
            weight_count
        );
    }

    #[test]
    fn by_name_finds_valid_observable() {
        let reg = ObservablesRegistry::new();
        let found = reg.by_name("attention_entropy");
        assert!(found.is_some());
        assert_eq!(found.unwrap().category, ObservableCategory::Attention);
    }

    #[test]
    fn by_name_returns_none_for_invalid() {
        let reg = ObservablesRegistry::new();
        assert!(reg.by_name("nonexistent_observable").is_none());
    }

    #[test]
    fn search_matches_name() {
        let reg = ObservablesRegistry::new();
        let results = reg.search("gradient_norm");
        assert!(!results.is_empty());
        assert!(results.iter().any(|o| o.name == "gradient_norm"));
    }

    #[test]
    fn search_matches_description() {
        let reg = ObservablesRegistry::new();
        let results = reg.search("saturation");
        assert!(!results.is_empty());
        assert!(results.iter().any(|o| o.name.contains("saturation")));
    }

    #[test]
    fn categories_returns_all_nine() {
        let reg = ObservablesRegistry::new();
        let cats = reg.categories();
        assert_eq!(cats.len(), 9);
        let cat_set: std::collections::HashSet<ObservableCategory> = cats.into_iter().collect();
        assert_eq!(cat_set.len(), 9);
    }

    #[test]
    fn all_returns_all_observables() {
        let reg = ObservablesRegistry::new();
        assert_eq!(reg.all().len(), reg.count());
    }

    #[test]
    fn each_category_has_at_least_minimum_observables() {
        let reg = ObservablesRegistry::new();
        let min_counts: Vec<(ObservableCategory, usize)> = vec![
            (ObservableCategory::Weight, 15),
            (ObservableCategory::Gradient, 12),
            (ObservableCategory::Representation, 15),
            (ObservableCategory::Attention, 12),
            (ObservableCategory::Loss, 12),
            (ObservableCategory::Architecture, 8),
            (ObservableCategory::Optimization, 10),
            (ObservableCategory::Memory, 8),
            (ObservableCategory::Behavior, 10),
        ];
        for (cat, min) in min_counts {
            let count = reg.by_category(&cat).len();
            assert!(
                count >= min,
                "category {:?} has {} observables, expected at least {}",
                cat,
                count,
                min
            );
        }
    }

    #[test]
    fn search_empty_query_returns_all() {
        let reg = ObservablesRegistry::new();
        let results = reg.search("");
        assert_eq!(results.len(), reg.count());
    }

    #[test]
    fn by_category_returns_distinct_entries() {
        let reg = ObservablesRegistry::new();
        let total: usize = reg
            .categories()
            .iter()
            .map(|c| reg.by_category(c).len())
            .sum();
        assert_eq!(total, reg.count());
    }
}
