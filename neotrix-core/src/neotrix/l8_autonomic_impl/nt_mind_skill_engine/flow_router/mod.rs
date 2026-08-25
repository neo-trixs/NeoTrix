//! Flow Router — mattpocock/skills ask-matt 模式吸收
//! 
//! 实现技能流路由: main flow (grill→spec→tickets→implement) + on-ramps (triage, diagnosing-bugs, wayfinder) 
//! + vocabulary layer (domain-modeling, codebase-design) + phase boundaries (continue/clear/handoff/subagent/compact)

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 流阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FlowPhase {
    GrillWithDocs,
    ToSpec,
    ToTickets,
    Implement,
    CodeReview,
}

/// On-ramp 入口
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OnRamp {
    Triage,
    DiagnosingBugs,
    Wayfinder,
}

/// 词汇层技能
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VocabularySkill {
    DomainModeling,
    CodebaseDesign,
}

/// 独立技能
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StandaloneSkill {
    GrillMe,
    Grilling,
    ResolvingMergeConflicts,
    Prototype,
    Research,
    ToQuestionnaire,
    Wizard,
    WaitWhat,
    Teach,
    WritingForAgents,
}

/// 相边界决策
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PhaseBoundaryDecision {
    Continue,
    Clear,
    Handoff,
    Subagent,
    Compact,
}

/// 流路由决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowDecision {
    pub phase: FlowPhase,
    pub on_ramp: Option<OnRamp>,
    pub vocabulary: Vec<VocabularySkill>,
    pub standalone: Vec<StandaloneSkill>,
    pub boundary: PhaseBoundaryDecision,
    pub rationale: String,
}

/// 智能区上下文
#[derive(Debug, Clone)]
pub struct SmartZoneContext {
    pub token_budget: usize,
    pub used_tokens: usize,
    pub threshold: f64, // 0.8 = 80% 触发 compact
}

impl Default for SmartZoneContext {
    fn default() -> Self {
        Self {
            token_budget: 150_000,
            used_tokens: 0,
            threshold: 0.8,
        }
    }
}

impl SmartZoneContext {
    pub fn utilization(&self) -> f64 {
        if self.token_budget == 0 {
            0.0
        } else {
            self.used_tokens as f64 / self.token_budget as f64
        }
    }

    pub fn should_compact(&self) -> bool {
        self.utilization() >= self.threshold
    }

    pub fn record_usage(&mut self, tokens: usize) {
        self.used_tokens += tokens;
    }
}

/// Flow Router 核心
#[derive(Debug)]
pub struct FlowRouter {
    smart_zone: SmartZoneContext,
    skill_registry: HashMap<String, SkillMetadata>,
    active_flow: Option<FlowPhase>,
    completed_phases: Vec<FlowPhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub triggers: Vec<String>,
    pub e8_modes: Vec<u8>,
    pub category: String,
    pub priority: u8,
    pub verified: bool,
}

impl FlowRouter {
    pub fn new() -> Self {
        Self {
            smart_zone: SmartZoneContext::default(),
            skill_registry: HashMap::new(),
            active_flow: None,
            completed_phases: Vec::new(),
        }
    }

    /// 注册技能元数据 (来自 skill engine)
    pub fn register_skill(&mut self, name: String, meta: SkillMetadata) {
        self.skill_registry.insert(name, meta);
    }

    /// 根据用户输入路由到合适的流/技能
    pub fn route(&mut self, input: &str, _context: Option<&str>) -> FlowDecision {
        let _context = _context;
        let input_lower = input.to_lowercase();
        
        // 检查智能区
        if self.smart_zone.should_compact() {
            return FlowDecision {
                phase: FlowPhase::GrillWithDocs,
                on_ramp: None,
                vocabulary: vec![],
                standalone: vec![],
                boundary: PhaseBoundaryDecision::Compact,
                rationale: format!("Smart zone utilization {:.0}% ≥ threshold, compact required", 
                    self.smart_zone.utilization() * 100.0),
            };
        }

        // On-ramps 优先匹配
        if input_lower.contains("triage") || input_lower.contains("issue") || input_lower.contains("bug report") {
            return self.route_on_ramp(OnRamp::Triage, "Issue triage detected");
        }
        if input_lower.contains("diagnose") || input_lower.contains("flake") || input_lower.contains("regression") {
            return self.route_on_ramp(OnRamp::DiagnosingBugs, "Bug diagnosis requested");
        }
        if input_lower.contains("greenfield") || input_lower.contains("huge") || input_lower.contains("foggy") {
            return self.route_on_ramp(OnRamp::Wayfinder, "Large foggy effort detected");
        }

        // Vocabulary layer
        let mut vocab = Vec::new();
        if input_lower.contains("domain") || input_lower.contains("context.md") || input_lower.contains("adr") {
            vocab.push(VocabularySkill::DomainModeling);
        }
        if input_lower.contains("module") || input_lower.contains("interface") || input_lower.contains("seam") {
            vocab.push(VocabularySkill::CodebaseDesign);
        }

        // Standalone skills
        let mut standalone = Vec::new();
        if input_lower.contains("grill me") || input_lower.contains("stateless") {
            standalone.push(StandaloneSkill::GrillMe);
        }
        if input_lower.contains("prototype") || input_lower.contains("throwaway") {
            standalone.push(StandaloneSkill::Prototype);
        }
        if input_lower.contains("research") || input_lower.contains("background") {
            standalone.push(StandaloneSkill::Research);
        }
        if input_lower.contains("wait what") || input_lower.contains("didn't land") {
            standalone.push(StandaloneSkill::WaitWhat);
        }

        // Main flow routing
        let phase = self.determine_main_flow_phase(&input_lower);
        
        // Phase boundary decision
        let boundary = if self.smart_zone.should_compact() {
            PhaseBoundaryDecision::Compact
        } else if self.completed_phases.contains(&phase) {
            PhaseBoundaryDecision::Continue
        } else {
            PhaseBoundaryDecision::Continue
        };

        self.active_flow = Some(phase);
        
        FlowDecision {
            phase,
            on_ramp: None,
            vocabulary: vocab,
            standalone,
            boundary,
            rationale: format!("Routed to {:?} based on input analysis", phase),
        }
    }

    fn route_on_ramp(&mut self, ramp: OnRamp, rationale: &str) -> FlowDecision {
        FlowDecision {
            phase: FlowPhase::GrillWithDocs,
            on_ramp: Some(ramp),
            vocabulary: vec![],
            standalone: vec![],
            boundary: PhaseBoundaryDecision::Continue,
            rationale: rationale.to_string(),
        }
    }

    fn determine_main_flow_phase(&self, input: &str) -> FlowPhase {
        if input.contains("grill") || input.contains("sharpen") || input.contains("idea") {
            FlowPhase::GrillWithDocs
        } else if input.contains("spec") || input.contains("specification") {
            FlowPhase::ToSpec
        } else if input.contains("ticket") || input.contains("split") || input.contains("tracer") {
            FlowPhase::ToTickets
        } else if input.contains("implement") || input.contains("build") || input.contains("code") {
            FlowPhase::Implement
        } else if input.contains("review") || input.contains("pr") || input.contains("diff") {
            FlowPhase::CodeReview
        } else {
            FlowPhase::GrillWithDocs // Default entry
        }
    }

    /// 记录阶段完成
    pub fn complete_phase(&mut self, phase: FlowPhase) {
        if !self.completed_phases.contains(&phase) {
            self.completed_phases.push(phase);
        }
    }

    /// 检查是否需要 compact
    pub fn check_smart_zone(&mut self, estimated_tokens: usize) -> Option<PhaseBoundaryDecision> {
        self.smart_zone.record_usage(estimated_tokens);
        if self.smart_zone.should_compact() {
            Some(PhaseBoundaryDecision::Compact)
        } else {
            None
        }
    }

    /// 获取当前智能区状态
    pub fn smart_zone_status(&self) -> SmartZoneContext {
        self.smart_zone.clone()
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let mut router = FlowRouter::new();
        
        // Test 1: On-ramp routing
        let decision = router.route("triage the issues", None);
        assert!(decision.on_ramp == Some(OnRamp::Triage), "Should route to triage");
        
        // Test 2: Main flow routing
        let decision = router.route("implement the feature", None);
        assert_eq!(decision.phase, FlowPhase::Implement);
        
        // Test 3: Vocabulary detection
        let decision = router.route("update CONTEXT.md with new term", None);
        assert!(decision.vocabulary.contains(&VocabularySkill::DomainModeling));
        
        // Test 4: Smart zone compact
        let mut router = FlowRouter::new();
        router.smart_zone.used_tokens = 140_000;
        let decision = router.route("anything", None);
        assert_eq!(decision.boundary, PhaseBoundaryDecision::Compact);
        
        // Test 5: Phase completion tracking
        let mut router = FlowRouter::new();
        router.complete_phase(FlowPhase::GrillWithDocs);
        router.complete_phase(FlowPhase::ToSpec);
        assert!(router.completed_phases.contains(&FlowPhase::GrillWithDocs));
        assert!(router.completed_phases.contains(&FlowPhase::ToSpec));
        
        Ok(())
    }
}

impl Default for FlowRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_router_on_ramp_triage() {
        let mut router = FlowRouter::new();
        let decision = router.route("triage the incoming issues", None);
        assert_eq!(decision.on_ramp, Some(OnRamp::Triage));
    }

    #[test]
    fn test_flow_router_on_ramp_diagnosing() {
        let mut router = FlowRouter::new();
        let decision = router.route("diagnose this flaky test", None);
        assert_eq!(decision.on_ramp, Some(OnRamp::DiagnosingBugs));
    }

    #[test]
    fn test_flow_router_on_ramp_wayfinder() {
        let mut router = FlowRouter::new();
        let decision = router.route("this is a huge greenfield project", None);
        assert_eq!(decision.on_ramp, Some(OnRamp::Wayfinder));
    }

    #[test]
    fn test_flow_router_vocabulary_domain_modeling() {
        let mut router = FlowRouter::new();
        let decision = router.route("sharpen the domain model in CONTEXT.md", None);
        assert!(decision.vocabulary.contains(&VocabularySkill::DomainModeling));
    }

    #[test]
    fn test_flow_router_vocabulary_codebase_design() {
        let mut router = FlowRouter::new();
        let decision = router.route("design the module interface and seams", None);
        assert!(decision.vocabulary.contains(&VocabularySkill::CodebaseDesign));
    }

    #[test]
    fn test_flow_router_main_flow_grill() {
        let mut router = FlowRouter::new();
        let decision = router.route("grill this idea with docs", None);
        assert_eq!(decision.phase, FlowPhase::GrillWithDocs);
    }

    #[test]
    fn test_flow_router_main_flow_implement() {
        let mut router = FlowRouter::new();
        let decision = router.route("implement the feature now", None);
        assert_eq!(decision.phase, FlowPhase::Implement);
    }

    #[test]
    fn test_flow_router_smart_zone_compact() {
        let mut router = FlowRouter::new();
        router.smart_zone.used_tokens = 140_000;
        let decision = router.route("any input", None);
        assert_eq!(decision.boundary, PhaseBoundaryDecision::Compact);
    }

    #[test]
    fn test_flow_router_phase_completion() {
        let mut router = FlowRouter::new();
        router.complete_phase(FlowPhase::GrillWithDocs);
        router.complete_phase(FlowPhase::ToSpec);
        assert!(router.completed_phases.contains(&FlowPhase::GrillWithDocs));
        assert!(router.completed_phases.contains(&FlowPhase::ToSpec));
        assert!(!router.completed_phases.contains(&FlowPhase::Implement));
    }

    #[test]
    fn test_skill_registration() {
        let mut router = FlowRouter::new();
        router.register_skill("rust-analyzer".to_string(), SkillMetadata {
            name: "rust-analyzer".to_string(),
            triggers: vec!["rust".to_string()],
            e8_modes: vec![12, 13, 14],
            category: "coding".to_string(),
            priority: 80,
            verified: true,
        });
        assert!(router.skill_registry.contains_key("rust-analyzer"));
    }

    #[test]
    fn test_self_test_passes() {
        assert!(FlowRouter::self_test().is_ok());
    }
}