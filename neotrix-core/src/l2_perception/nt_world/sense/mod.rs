pub mod omniscient_view;
pub mod visual_cortex;
pub mod auditory_cortex;
pub mod nt_world_sense_hub;
pub mod world_consciousness;
pub mod real_sensors;
pub mod types;
pub mod encoder;
pub mod predictor;
pub mod loss;
pub mod sigreg;
pub mod td_jepa;
pub mod rgm_jepa;
pub mod vit;
pub mod masking;
pub mod action_predictor;
pub mod world_model;
pub mod world_model_v2;
pub mod nt_world_model_types;
pub mod nt_world_model_predict;
pub mod nt_world_model_td_jepa;
pub mod nt_world_model_rgm_jepa;
pub mod nt_world_model_object_partition;

pub use omniscient_view::*;
pub use visual_cortex::*;
pub use auditory_cortex::*;
pub use nt_world_sense_hub::*;
pub use world_consciousness::*;

pub use types::{
    JEPA_LATENT_DIM, JEPA_HIDDEN_DIM, JEPA_EMA_MOMENTUM, JEPA_VARIANCE_TARGET,
    JEPA_VAR_WEIGHT, JEPA_COV_WEIGHT, JEPA_INV_WEIGHT, JEPA_LEARNING_RATE,
    JEPA_GAUSS_STD_TARGET, JEPA_GAUSS_WEIGHT,
    MultiScalePrediction, LatentState, TDExperience, WorldModelState,
};
pub use encoder::JepaEncoder;
pub use predictor::JepaPredictor;
pub use loss::{EnergyModel, VicRegLoss, NonContrastiveLoss, levl_jepa_loss};
pub use sigreg::SIGReg;
pub use td_jepa::{TDTarget, TDDynamics};
pub use rgm_jepa::{CGBlock, MultiScaleJEPA, RGMLatent};
pub use world_model::{JepaWorldModel, WorldModel, ContextEncoder, WifiStatus};
pub use vit::JepaViTEncoder;
pub use masking::{MaskingStrategy, MaskInfo};
pub use action_predictor::{ActionConditionedPredictor, FusionMode};
pub use world_model_v2::JepaWorldModelV2;

pub use nt_world_model_types::*;
pub use nt_world_model_predict::*;
pub use nt_world_model_td_jepa::*;
pub use nt_world_model_rgm_jepa::*;
pub use nt_world_model_object_partition::*;

// WorldModel / ContextEncoder / WifiStatus 统一定义在 sense::world_model (单一事实源)
// 此处仅做 re-export，不再内联定义

pub use crate::neotrix::nt_world_model_v2::WorldModelV2;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod model_tests {
    use super::*;

    #[test]
    fn test_latent_state_similarity() {
        let dim = 32;
        let s1 = LatentState::zero(dim);
        let s2 = LatentState::zero(dim);
        let sim = s1.similarity(&s2);
        assert!(sim > 0.9);
    }

    #[test]
    fn test_context_from_description() {
        let ctx = Context::from_task_description("设计一个iOS原型");
        assert_eq!(ctx.task_type as usize, TaskType::Design as usize);

        let ctx2 = Context::from_task_description("分析这段代码的性能");
        assert_eq!(ctx2.task_type as usize, TaskType::CodeAnalysis as usize);
    }

    #[test]
    fn test_wifi_integration() {
        let mut wm = WorldModel::new(4);
        assert_eq!(wm.occupant_count, 0);
        assert!(wm.zone_heatmap.is_empty());
        assert!(!wm.wifi_enabled);

        let mut heatmap = std::collections::HashMap::new();
        heatmap.insert("zone_1".to_string(), 0.85);
        let status = WifiStatus {
            enabled: true,
            occupant_count: 2,
            zone_count: 3,
            heatmap,
        };
        wm.update_from_wifi(&status);
        assert_eq!(wm.occupant_count, 2);
        assert!((wm.zone_heatmap.get("zone_1").expect("value should be ok in test") - 0.85).abs() < 1e-6);
        assert!(wm.wifi_enabled);
    }

    #[test]
    fn test_reward_from_knowledge_quality_empty_cube() {
        let wm = WorldModel::new(4);
        let cube = KnowledgeHyperCube::new();
        let reward = wm.reward_from_knowledge_quality(&cube);
        assert!(reward >= 0.0 && reward <= 1.0);
    }

    #[test]
    fn test_reward_from_knowledge_quality_partially_filled() {
        let wm = WorldModel::new(4);
        let mut cube = KnowledgeHyperCube::new();
        let coord = crate::core::nt_core_hcube::coord::HyperCoord::new();
        cube.insert(&coord, "test-source", "test-entry");
        let reward = wm.reward_from_knowledge_quality(&cube);
        assert!(reward >= 0.0 && reward <= 1.0);
    }
}
