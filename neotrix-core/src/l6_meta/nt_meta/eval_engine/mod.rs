#![deny(clippy::unwrap_used)]

pub mod dataset_manager;
pub mod experiment_tracker;
pub mod llm_judge;

pub use dataset_manager::{filter_by_tag, load_json, save_json, Dataset, DatasetEntry};
pub use experiment_tracker::{compare_variants, mean, std_dev, Experiment, ExperimentVariant};
pub use llm_judge::{evaluate_response, Criterion, CriterionScore, JudgeConfig, JudgeResult};
