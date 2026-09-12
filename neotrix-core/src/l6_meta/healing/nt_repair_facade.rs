//! NT-REPAIR Facade — re-export all public types from nt_repair submodules.

pub use super::nt_repair_self_heal::{HealableDetector, SelfHealLoop};

pub use super::nt_repair_causal_trace::{
    CausalChainWalker, CausalNode, CausalTrace, CausalTraceSelfTest, EvidenceGate,
    EvidenceRule, SourceAdjudicator, SourceDetector, SourceVerdict, Warning,
    default_adjudicator,
};

pub use super::nt_repair_hanzi_video::{
    HanziChaiziEngine, HanziChar, HanziComponent, HanziRepairResult, HanziVideoSelfTest,
    RepairHarness, VideoClip, VideoClipPlan, VideoClipPlanner,
};

pub use super::nt_mind_consciousness_monitor::{
    AwarenessReport, AwarenessTrends, ConsciousnessMonitor,
};

pub use super::nt_mind_consciousness_gold_standard::{
    ConsciousnessGoldStandard, ConsciousnessLevel, DetectionTrend, E8HexagramState,
    GoldStandardReport, derive_level, DEFAULT_COHERENCE_THRESHOLD, DEFAULT_MAX_HISTORY,
    DEFAULT_PHI_THRESHOLD, HIGH_COHERENCE_THRESHOLD, HIGH_PHI_THRESHOLD,
};

pub use super::nt_mind_eval_harness::{
    ComplianceGate, DatasetSpec, EvalError, EvalHarness, EvalPoint, EvalQuery, EvalReport,
    GalaxyComparison, HdaAttribution, HdaAttributionReport, HdaComponent,
    HyperparamSensitivity, InstructionPlane, LadderReport, ModelQualityCurve, ModelSpec,
    OracleLadder, OracleLadderHealer, OracleRung, ParetoPoint, PlaneConflictCase,
    RegressionCase, RegressionResult, RewardSignal, RungOracle, RungResult,
    SelfVerifiableReward, SelfVerifiableRewardSelfTest, SmallScaleMethod, SmallScaleWarning,
    UnifiedVerifyRequest, UnifiedVerifyResult, VerificationChannel, WithholdingResult,
    ap_acc_score, hda_attribution, verify_constraint, verify_deterministic,
    verify_extractable, verify_unified, verify_unified_batch, DEFAULT_BUDGET_GRID,
};
