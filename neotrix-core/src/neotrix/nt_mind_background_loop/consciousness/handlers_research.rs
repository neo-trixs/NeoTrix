#![allow(unused_imports)]
use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_experience::meta_cog_mera::ReasoningStep;

// RESEARCH handlers extracted from modules_core.rs
// 7 handlers

impl ConsciousnessIntegration {
    // ── Null Drift Memory ──

    pub fn handle_research_tick(&mut self) -> String {
        let result = self.research_engine.tick(self.stats().c_score);
        log::debug!("MODULES: research_tick {}", result);
        result
    }

    pub fn handle_research_propose_tick(&mut self) -> String {
        // S1-DeepResearch inspired: generate research tasks with constraint injection
        use std::time::Instant;
        let start = Instant::now();

        let mut constraints = Vec::new();
        constraints.push("source:web+academic".to_string());
        constraints.push("reasoning:multi_hop".to_string());
        constraints.push(format!(
            "output:report_{}_words",
            500 + (self.cycle % 1500) as u64
        ));

        let task_count = self.job_queue.pending_count();
        let kg_stats = self.research_kg.stats();
        let proposal = if task_count > 0 {
            format!(
                "research_propose:{}_tasks_queued_{}_constraints_kg:{}",
                task_count,
                constraints.len(),
                kg_stats,
            )
        } else {
            let result =
                self.research_engine
                    .propose_research("auto", "discovery", 20, constraints.clone());
            result
        };

        // Log trajectory step for the propose action
        let step = TrajectoryStep {
            step_type: "reasoning".to_string(),
            input: format!("propose_research cycle={}", self.cycle),
            output: proposal.clone(),
            tool_used: Some("research_engine+job_queue".to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
        };
        let _ = self.record_research_trajectory(
            format!("propose_cycle_{}", self.cycle),
            constraints,
            vec![step],
            proposal.clone(),
        );

        log::debug!("MODULES: research_propose_tick {}", proposal);
        format!("{}_took_{:?}", proposal, start.elapsed())
    }

    pub fn handle_research_stats_tick(&mut self) -> String {
        self.research_engine.stats()
    }

    // ── Research Knowledge Graph pipeline ──

    pub fn handle_research_kg_tick(&mut self) -> String {
        let stats = self.research_kg.stats();
        log::debug!("MODULES: research_kg_tick {}", stats);
        stats
    }

    pub fn handle_research_kg_submit_tick(&mut self) -> String {
        let content = format!("research_kg cycle {} auto-collect", self.cycle);
        let id = self.research_kg.submit_document("consciousness", &content);
        format!("kg:submitted|{}", id)
    }

    // ── S1-DeepResearch Trajectory Pipeline ──

    /// Collect research state into a trajectory step and run verification.
    /// Wires together: research_engine → research_kg → job_queue → trajectory_log → verification.

    pub fn handle_research_trajectory_tick(&mut self) -> String {
        let kg_stats = self.research_kg.stats();
        let queue_stats = self.job_queue.stats();
        let engine_stats = self.research_engine.stats();

        // Snapshot verifier stats before mutable borrow
        let (v_total, v_passed, v_failed) = {
            let ver = &self.trajectory_verification;
            (ver.total_verified, ver.passed, ver.failed)
        };

        let step = TrajectoryStep {
            step_type: "verification".to_string(),
            input: format!(
                "kg={}|queue={}|engine={}",
                kg_stats, queue_stats, engine_stats
            ),
            output: format!(
                "trajectory:verified={}_passed={}_failed={}",
                v_total, v_passed, v_failed,
            ),
            tool_used: Some("verifier".to_string()),
            duration_ms: 0,
        };

        let _ = self.record_research_trajectory(
            format!("pipeline_snapshot_cycle_{}", self.cycle),
            vec!["source:research_pipeline".to_string()],
            vec![step],
            format!("snapshot_at_cycle_{}", self.cycle),
        );

        format!(
            "trajectory:log={}_vtotal={}_vpassed={}_vfailed={}",
            self.research_trajectory_log.len(),
            v_total,
            v_passed,
            v_failed,
        )
    }
}
