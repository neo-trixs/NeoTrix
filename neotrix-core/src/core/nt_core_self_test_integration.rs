use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

pub fn register_absorbed_modules(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(AnswerEngineSelfTest));
    registry.register(Box::new(AnswerBridgeSelfTest));
    registry.register(Box::new(AgentTeamSelfTest));
    registry.register(Box::new(AgenticScanSelfTest));
    registry.register(Box::new(DigitalHumanSelfTest));
    registry.register(Box::new(LeannStoreSelfTest));
    registry.register(Box::new(VideoPipelineSelfTest));
}

struct AnswerEngineSelfTest;

impl SelfTest for AnswerEngineSelfTest {
    fn name(&self) -> &str {
        "nt_core_answer_engine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let engine = crate::core::nt_core_answer_engine::AnswerEngine::with_mode(
            crate::core::nt_core_answer_engine::AnswerMode::Balanced
        );
        let config = engine.config();
        if config.max_sources == 0 {
            return Err(vec!["config not initialized".into()]);
        }
        Ok(())
    }
}

struct AnswerBridgeSelfTest;

impl SelfTest for AnswerBridgeSelfTest {
    fn name(&self) -> &str {
        "nt_core_answer_bridge"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mode = crate::core::nt_core_answer_engine::AnswerMode::Speed;
        match crate::core::nt_core_answer_bridge::AnswerGwtBridge::route_for_mode(mode) {
            crate::core::nt_core_answer_bridge::RoutingMode::Direct(_) => Ok(()),
            _ => Err(vec!["Speed should route Direct".into()]),
        }
    }
}

struct AgentTeamSelfTest;

impl SelfTest for AgentTeamSelfTest {
    fn name(&self) -> &str {
        "nt_act_agent_team"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::neotrix::l1_body_impl::nt_agent_agent_team::*;
        let mut team = AgentTeam::new("test-team");
        team.add_member(AgentProfile::new(AgentRole::Lead, "alice"));
        team.add_member(AgentProfile::new(AgentRole::Coder, "bob"));
        if team.member_count() != 2 {
            return Err(vec!["expected 2 members".into()]);
        }
        let t1 = team.create_task("implement feature", AgentRole::Coder, 1);
        let assigned = team.assign_tasks();
        if assigned.is_empty() {
            return Err(vec!["no tasks assigned".into()]);
        }
        team.complete_task(t1);
        if (team.progress() - 1.0).abs() > 0.01 {
            return Err(vec!["progress should be 1.0".into()]);
        }
        Ok(())
    }
}

struct AgenticScanSelfTest;

impl SelfTest for AgenticScanSelfTest {
    fn name(&self) -> &str {
        "nt_shield_agentic_scan"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::neotrix::l1_body_impl::nt_shield_agentic_scan::*;
        let scanner = AgenticScanner::new(ScanConfig::default());
        if scanner.current_stage() != ScanStage::Recon {
            return Err(vec!["initial stage should be Recon".into()]);
        }
        let report = scanner.recon_scan("http://test.com");
        if report.estimated_files == 0 {
            return Err(vec!["recon should estimate files".into()]);
        }
        Ok(())
    }
}

struct DigitalHumanSelfTest;

impl SelfTest for DigitalHumanSelfTest {
    fn name(&self) -> &str {
        "nt_io_digital_human"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::neotrix::l1_body_impl::nt_io_digital_human::*;
        let pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        if !pipeline.generate_reply("hello").contains("Hello") {
            return Err(vec!["reply should handle hello".into()]);
        }
        Ok(())
    }
}

struct LeannStoreSelfTest;

impl SelfTest for LeannStoreSelfTest {
    fn name(&self) -> &str {
        "nt_memory_leann_store"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::neotrix::l3_memory_impl::nt_memory_leann_store::*;
        let store = LeannGraphStore::new(LeannConfig::default());
        if store.node_count() != 0 {
            return Err(vec!["fresh store should have 0 nodes".into()]);
        }
        Ok(())
    }
}

struct VideoPipelineSelfTest;

impl SelfTest for VideoPipelineSelfTest {
    fn name(&self) -> &str {
        "nt_world_video_pipeline"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::neotrix::l2_world_impl::nt_world_video_pipeline::*;
        let orch = VideoOrchestrator::new(TranscodeConfig::default());
        orch.self_test()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_all() {
        let mut registry = SelfTestRegistry::new();
        register_absorbed_modules(&mut registry);
        assert_eq!(registry.count(), 7);
    }

    #[test]
    fn test_answer_engine_st() {
        assert!(AnswerEngineSelfTest.self_test().is_ok());
    }

    #[test]
    fn test_answer_bridge_st() {
        assert!(AnswerBridgeSelfTest.self_test().is_ok());
    }

    #[test]
    fn test_all_have_names() {
        let tests: [&dyn SelfTest; 7] = [
            &AnswerEngineSelfTest, &AnswerBridgeSelfTest, &AgentTeamSelfTest,
            &AgenticScanSelfTest, &DigitalHumanSelfTest, &LeannStoreSelfTest,
            &VideoPipelineSelfTest,
        ];
        for t in &tests { assert!(!t.name().is_empty()); }
    }
}
