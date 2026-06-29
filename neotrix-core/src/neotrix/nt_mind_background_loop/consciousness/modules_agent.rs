#![allow(unused_imports)]
use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_agent::cdp_session::CDPSessionManager;
use crate::core::nt_core_agent::factor_miner::FactorMiner;
use crate::core::nt_core_agent::permission::{
    PermissionDecision, PermissionGate, PermissionMode, PermissionOverrides,
};
use crate::core::nt_core_agent::quant_data::QuantDataIngestion;
use crate::core::nt_core_agent::remote_host::RemoteAgentHost;
use crate::core::nt_core_experience::osint_tools::OsintToolLayer;
use crate::core::nt_core_experience::LoadTier;
use crate::core::nt_core_protect::honeypot::SecurityGate;

// AGENT handlers extracted from modules.rs
// 29 handlers

impl ConsciousnessIntegration {
    pub fn safety_check_mutation(
        &mut self,
        mutation: &crate::core::nt_core_experience::self_evolution_loop::MutationOp,
    ) -> Result<(), String> {
        match mutation {
            crate::core::nt_core_experience::self_evolution_loop::MutationOp::TuneParam {
                target,
                delta,
            } => {
                self.ball_verifier
                    .check_modification(target, *delta, None)?;
                self.pcc_safety
                    .evaluate_edit(target, *delta, "self_evolution")
                    .map_err(|_| format!("PccSafetyGate rejected {}", target))?;
                Ok(())
            }
            _ => {
                let label = mutation.label();
                self.pcc_safety
                    .evaluate_edit(label, 0.0, "self_evolution")
                    .map_err(|_| format!("PccSafetyGate rejected mutation {}", label))?;
                Ok(())
            }
        }
    }

    pub fn handle_meta_agent_tick(&mut self) -> String {
        // Skip if cognitive load is too high
        if self.cognitive_load > 0.8 {
            return format!(
                "meta_agent:skipped:cognitive_load_{:.2}",
                self.cognitive_load
            );
        }

        // Record recent base edits from self-evolution archive
        if let Some(ref evo) = self.self_evolution {
            let archive = &evo.archive;
            let top: Vec<&crate::core::nt_core_experience::self_evolution_loop::SelfEvolutionStep> =
                archive
                    .steps
                    .iter()
                    .filter(|s| s.score_after.unwrap_or(0.0) > 0.0)
                    .rev()
                    .take(10)
                    .collect();
            for step in top {
                self.meta_agent.record_base_edit(
                    crate::core::nt_core_experience::hyperagent::EditRecord {
                        description: step.mutation.summary(),
                        target_subsystem: match &step.mutation {
                            crate::core::nt_core_experience::self_evolution_loop::MutationOp::TuneParam { target, .. } => target.clone(),
                            _ => "handler".to_string(),
                        },
                        negentropy_gain: step.score_after.unwrap_or(0.0) - step.score_before,
                        applied: step.accepted,
                    },
                );
            }
        }

        // Feed goal drift signal before reflecting (SAHOO pattern #5)
        self.meta_agent
            .set_goal_drift(self.goal_drift.gdi(), self.goal_drift.drift_detected());

        // Run v2 MetaAgent evolution every cycle (moved outside if/else)
        let v2_report: String;
        let v2_best_score: f64;
        let v2_has_entries: bool;
        {
            let (r, s, h) = if let Some(ref mut ma2) = self.meta_agent_v2 {
                if ma2.should_continue() {
                    let gen = ma2.run_generation();
                    if gen.proposals_accepted > 0 {
                        log::info!(
                            "META_AGENT_V2: generated={} accepted={} best={:.3}",
                            gen.proposals_generated,
                            gen.proposals_accepted,
                            gen.best_score
                        );
                    }
                    let report = format!(
                        "v2:gen={}_acc={}_best={:.3}_arc={}_roll={}",
                        gen.proposals_generated,
                        gen.proposals_accepted,
                        gen.best_score,
                        gen.archive_size,
                        gen.rollbacks
                    );
                    (report, gen.best_score, true)
                } else {
                    ("v2:budget_exhausted".to_string(), 0.0, false)
                }
            } else {
                ("v2:unavailable".to_string(), 0.0, false)
            };
            v2_report = r;
            v2_best_score = s;
            v2_has_entries = h;
        }

        // DGM-H bridge v2→v1: feed v2's archive best entry into v1's pattern detection
        if v2_has_entries {
            let v2_entry = self
                .meta_agent_v2
                .as_ref()
                .and_then(|ma2| ma2.archive.latest_node())
                .map(|n| (n.id.clone(), n.score));
            if let Some((v2_id, v2_score_v)) = v2_entry {
                self.meta_agent.record_base_edit(
                    crate::core::nt_core_experience::hyperagent::EditRecord {
                        description: format!("v2_archive:{}_score={:.3}", v2_id, v2_score_v),
                        target_subsystem: "meta_agent_v2".to_string(),
                        negentropy_gain: v2_score_v,
                        applied: true,
                    },
                );
            }
        }

        // Reflect: detect meta-level patterns
        if let Some(proposal) = self.meta_agent.reflect() {
            // Gate 1: Safety evaluation (same-context)
            if !self.meta_agent.evaluate_safety(&proposal) {
                log::warn!(
                    "META_AGENT: rejected unsafe proposal [{}]",
                    proposal.description
                );
                self.meta_agent.reject(&proposal);
                return "meta_agent:rejected_unsafe".to_string();
            }
            // Gate 2: Fresh-context independent review
            let (review_approved, review_reason) =
                self.meta_agent.review_with_fresh_context(&proposal);
            if !review_approved {
                log::warn!(
                    "META_AGENT: rejected by fresh-context reviewer [{}]: {}",
                    proposal.description,
                    review_reason
                );
                self.meta_agent.reject(&proposal);
                return format!("meta_agent:rejected_by_reviewer_{}", review_reason);
            }
            log::info!(
                "META_AGENT: applying [{}] {}",
                proposal.description,
                proposal.change
            );
            let _action = self.meta_agent.apply(&proposal);
            let v1_score = proposal.predicted_impact * proposal.confidence;

            // Metacognitive wisdom gate: know when NOT to act
            let wisdom_threshold = if self.wisdom_score_history.len() > 5 {
                let sum: f64 = self.wisdom_score_history.iter().rev().take(20).sum();
                let count = self.wisdom_score_history.len().min(20) as f64;
                (sum / count) * 0.5
            } else {
                0.3
            };

            let cycle_max_score = v1_score.max(v2_best_score);
            self.wisdom_score_history.push(cycle_max_score);
            if self.wisdom_score_history.len() > 100 {
                self.wisdom_score_history.remove(0);
            }

            if cycle_max_score < wisdom_threshold && self.wisdom_score_history.len() > 3 {
                log::info!(
                    "WISDOM_GATE: skipping apply — max_score={:.3} < threshold={:.3} (history={})",
                    cycle_max_score,
                    wisdom_threshold,
                    self.wisdom_score_history.len()
                );
                return format!(
                    "meta_agent:wisdom_gate_score={:.3}_thresh={:.3}",
                    cycle_max_score, wisdom_threshold
                );
            }

            // DGM-H bridge v1→v2: record v1's applied proposal into v2's archive
            if let Some(ref mut ma2) = self.meta_agent_v2 {
                use std::collections::HashMap;
                let entry = crate::neotrix::nt_mind::meta_agent::ArchiveEntry {
                    id: format!("v1_{}_{}", self.cycle, proposal.id),
                    parent_id: ma2.archive.latest_node().map(|n| n.id.clone()),
                    score: v1_score,
                    diversity_score: 0.0,
                    diffs: Vec::new(),
                    generation: ma2.archive.config.min_generations,
                    timestamp: self.cycle,
                    lineage: vec!["v1_meta_agent".to_string()],
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("target".to_string(), format!("{:?}", proposal.target));
                        m.insert("description".to_string(), proposal.description.clone());
                        m.insert(
                            "predicted_impact".to_string(),
                            format!("{:.3}", proposal.predicted_impact),
                        );
                        m.insert(
                            "confidence".to_string(),
                            format!("{:.3}", proposal.confidence),
                        );
                        m
                    },
                };
                ma2.archive.push_entry(entry);
            }

            // Tournament: compare v1 proposal with v2 evolution
            let tournament = if v2_best_score > v1_score && v2_has_entries {
                log::info!("TOURNAMENT: v2 wins (v2={:.3} > v1={:.3}) — v1 applied by default until v2 apply is wired", v2_best_score, v1_score);
                format!("|v2_wins_{:.3}_vs_{:.3}", v2_best_score, v1_score)
            } else if v2_has_entries {
                format!("|v1_wins_{:.3}_vs_{:.3}", v1_score, v2_best_score)
            } else {
                String::new()
            };
            format!("meta_agent:applied_{:?}{}", proposal.target, tournament)
        } else {
            // Track idle cycle score (0.0 since no v1 proposal)
            self.wisdom_score_history.push(v2_best_score);
            if self.wisdom_score_history.len() > 100 {
                self.wisdom_score_history.remove(0);
            }

            // — Autonomous evolution: detect underperforming handlers —
            let low_performers = self.handler_registry.worst_handlers(0.3);
            let mut evo_parts: Vec<String> = Vec::new();
            for hname in &low_performers {
                match self.handler_registry.success_rate(hname) {
                    Some(rate) if rate == 0.0 => {
                        self.handler_registry.mark_unloaded(hname);
                        evo_parts.push(format!("pruned:{}:zero_success", hname));
                    }
                    Some(rate) => {
                        self.handler_registry.register(hname, LoadTier::Warm);
                        self.handler_registry.record_success(hname);
                        evo_parts.push(format!("repaired:{}:rate_{:.2}", hname, rate));
                    }
                    None => {}
                }
            }

            let feedback = self.evaluate_pending_mutations();
            let feedback_part = if feedback.is_empty() {
                String::new()
            } else {
                format!("|{}", feedback)
            };
            let report = self.meta_agent.report();
            let evo_summary = if evo_parts.is_empty() {
                String::new()
            } else {
                format!("|evo:{}", evo_parts.join(","))
            };
            format!(
                "meta_agent:idle_applied={}_rejected={}_enabled={}_{}{}{}",
                report.applied_meta_edits,
                report.base_rejection_rate > 0.3,
                report.meta_enabled,
                v2_report,
                evo_summary,
                feedback_part,
            )
        }
    }

    // ── Skill health monitor (MOLTRON-style self-healing) ──

    pub fn handle_skill_health_tick(&mut self) -> String {
        let diag = self.skill_health_monitor.diagnostic();
        let needs_repair = self.skill_health_monitor.needs_repair();
        if !needs_repair.is_empty() {
            log::warn!(
                "SKILLHEALTH: {} skills need repair: {:?}",
                needs_repair.len(),
                needs_repair
            );
            for skill_name in &needs_repair {
                let report = self
                    .skill_health_monitor
                    .attempt_repair(skill_name, self.cycle);
                log::warn!("SKILLHEALTH: {}", report);
                if let Some(ref mut ev) = self.ne_evaluator {
                    let skill_path = format!("{}/{}.ne", self.ne_source_dir, skill_name);
                    if let Ok(source) = std::fs::read_to_string(&skill_path) {
                        match ev.eval_file(&source) {
                            Ok(_) => {
                                self.skill_health_monitor.record_success(skill_name);
                                log::info!(
                                    "SKILLHEALTH: re-loaded skill '{}' successfully",
                                    skill_name
                                );
                            }
                            Err(e) => {
                                log::error!(
                                    "SKILLHEALTH: re-load of skill '{}' also failed: {}",
                                    skill_name,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
        format!("skill_health:{}", diag)
    }

    // ── AutoResearchEngine (Karpathy-style fixed-budget experiment loop) ──

    pub fn handle_quant_data_tick(&mut self) -> String {
        if self.quant_data.is_none() {
            self.quant_data = Some(QuantDataIngestion::new());
            return "qdata:init".into();
        }
        let qdata = match self.quant_data.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };
        let snapshot = qdata.ingest_tick("NEOTRIX", 100.0 + self.cycle as f64, 1000.0);
        let n = qdata.source_count();
        format!("qdata:sym={},srcs={}", snapshot.symbol, n)
    }

    // ── CDP Session Manager (P1.25) ──

    pub fn handle_cdp_session_tick(&mut self) -> String {
        if self.cdp_session.is_none() {
            self.cdp_session = Some(CDPSessionManager::new());
            return "cdp:init".into();
        }
        let cdp_session = match self.cdp_session.as_ref() {
            Some(s) => s,
            None => {
                log::error!("[modules_agent] cdp_session not initialized after init");
                return "cdp:unavailable".into();
            }
        };
        let stats = cdp_session.stats();
        format!(
            "cdp:sessions={},cmds={}",
            stats.active_sessions, stats.total_commands
        )
    }

    // ── Fringe Mix Strategy (P1.26) ──

    pub fn handle_factor_miner_tick(&mut self) -> String {
        if self.factor_miner.is_none() {
            self.factor_miner = Some(FactorMiner::new());
            return "fmin:init".into();
        }
        let fmin = match self.factor_miner.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };
        let candidates = fmin.generate_candidates(5);
        format!("fmin:candidates={}", candidates.len())
    }

    // ── OSINT Tool Layer (P2.18) ──

    /// Run a full OSINT round: username search → identity correlation → evidence ingest.
    pub fn handle_osint_tick(&mut self, target: &str) -> String {
        let osint = match self.osint_tools.as_mut() {
            Some(e) => e,
            None => return "osint:unavailable".into(),
        };

        // 1. Username search (basic + advanced)
        let basic = osint.search_username(target);
        let advanced = osint.search_username_advanced(target);
        let mut all_evidence: Vec<_> = basic.into_iter().chain(advanced).collect();

        // 2. If target looks like an email, run email + breach checks
        if target.contains('@') {
            all_evidence.extend(osint.search_email(target));
            all_evidence.extend(osint.search_breach(target));
        }

        // 3. Ingest evidence into correlator
        if let Some(correlator) = self.identity_correlator.as_mut() {
            for ev in &all_evidence {
                correlator.register_alias(&ev.target, ev.source_type.name(), ev.confidence, None);
            }
        }

        let n_sources: usize = all_evidence
            .iter()
            .map(|e| e.source_type)
            .collect::<std::collections::HashSet<_>>()
            .len();
        format!(
            "osint:target={},sources={},hits={}",
            target,
            n_sources,
            all_evidence.len()
        )
    }

    /// Run IP reconnaissance.
    pub fn handle_ip_osint(&mut self, ip: &str) -> String {
        let osint = match self.osint_tools.as_mut() {
            Some(e) => e,
            None => return "ip:unavailable".into(),
        };
        let results = osint.search_ip(ip);
        format!("ip:{} hits={}", ip, results.len())
    }

    /// Run domain WHOIS lookup.
    pub fn handle_domain_osint(&mut self, domain: &str) -> String {
        let osint = match self.osint_tools.as_mut() {
            Some(e) => e,
            None => return "whois:unavailable".into(),
        };
        let results = osint.search_whois(domain);
        format!("whois:{} hits={}", domain, results.len())
    }

    /// Correlate aliases into identities using IdentityCorrelator.
    pub fn handle_identity_correlation(&mut self, aliases: &[&str]) -> String {
        let correlator = match self.identity_correlator.as_mut() {
            Some(c) => c,
            None => return "corr:unavailable".into(),
        };
        let results = correlator.correlate(aliases);
        format!("corr:n_identities={}", results.len())
    }

    // ── Native VSA Capability Dispatch ──
    // Replaces MCPIntelligenceServer with direct VSA-native capability routing.

    pub fn handle_native_capability_tick(&mut self) -> String {
        let n_capabilities = self.capability_synthesizer.stats().total_capabilities;
        if n_capabilities == 0 {
            if self.cycle % 50 == 0 {
                return "ncap:no_capabilities".into();
            }
        }
        format!("ncap:count={}", n_capabilities)
    }

    // ── Hubness Detector (P2.21) ──

    pub fn handle_remote_host_tick(&mut self) -> String {
        if self.remote_host.is_none() {
            self.remote_host = Some(RemoteAgentHost::new());
            return "rhost:init".into();
        }
        let host = match self.remote_host.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };
        if host.active_session_count() == 0 && self.cycle % 50 == 0 {
            let configs = host.discover_from_ssh_config();
            format!("rhost:discovered={}", configs.len())
        } else {
            format!("rhost:sessions={}", host.active_session_count())
        }
    }

    // ── Security Gate (P2.24) ──

    pub fn handle_security_gate_tick(&mut self) -> String {
        if self.security_gate.is_none() {
            self.security_gate = Some(SecurityGate::new());
            return "secg:init".into();
        }
        let sec = match self.security_gate.as_mut() {
            Some(e) => e,
            None => {
                log::error!("MODULES: engine not init");
                return "engine:unavailable".into();
            }
        };
        if !self.attractor_state.is_empty() {
            let result = sec.check_threat("127.0.0.1", &self.attractor_state);
            let rules = sec.rule_count();
            format!(
                "secg:anomaly={},z={:.2},rules={}",
                result.is_anomaly, result.z_score, rules
            )
        } else {
            format!("secg:rules={}", sec.rule_count())
        }
    }

    // ── Native CDP Browser Tick (replaces BrowserMCP) ──
    // Uses CDPSessionManager directly — no MCP protocol layer.

    pub fn handle_native_browser_tick(&mut self) -> String {
        if self.cdp_session.is_none() {
            self.cdp_session = Some(CDPSessionManager::new());
            return "nbrw:init".into();
        }
        let cdp = match self.cdp_session.as_ref() {
            Some(s) => s,
            None => return "nbrw:unavailable".into(),
        };
        let stats = cdp.stats();
        format!(
            "nbrw:sessions={},cmds={}",
            stats.active_sessions, stats.total_commands
        )
    }

    // ── Koopman Operator (P2.22) ──

    pub fn handle_sub_agent_spawn_tick(&mut self) -> String {
        match self.lead_agent.as_mut() {
            Some(la) => {
                let count = la.state.active_subagents.len();
                format!("sub_agent:{}_active", count)
            }
            None => "sub_agent:unwired".to_string(),
        }
    }

    pub fn handle_sub_agent_tick(&mut self) -> String {
        let _ = self.handle_sub_agent_spawn_tick();
        let count = self.lead_agent.as_ref().map_or(0, |a| a.active_count());
        format!("sub_agent:active={}", count)
    }

    pub fn handle_sub_agent_collect_tick(&mut self) -> String {
        match self.lead_agent.as_mut() {
            Some(la) => {
                let done = la.state.completed_tasks.len();
                format!("sub_agent:{}_completed", done)
            }
            None => "sub_agent:unwired".to_string(),
        }
    }

    // ── LeadAgent handlers ──

    pub fn handle_lead_agent_plan_tick(&mut self) -> String {
        match self.lead_agent.as_mut() {
            Some(la) => {
                let id = format!("plan-{}", self.cycle);
                la.plan("default goal");
                format!("lead_agent:planned_{}", id)
            }
            None => "lead_agent:unwired".to_string(),
        }
    }

    pub fn handle_lead_agent_execute_tick(&mut self) -> String {
        match self.lead_agent.as_ref() {
            Some(la) => {
                let tasks = la.state.active_subagents.len();
                let done = la.state.completed_tasks.len();
                format!("lead_agent:{}_tasks_{}_done", tasks, done)
            }
            None => "lead_agent:unwired".to_string(),
        }
    }

    // ── Preview engine handler ──

    pub fn handle_ultra_review_tick(&mut self) -> String {
        use crate::neotrix::nt_act_code::ultra_review::{
            default_review_dimensions, UltraReviewEngine,
        };
        let result = UltraReviewEngine::review("", "", &default_review_dimensions());
        let total = result.total_issues;
        format!("ultra_review:{}_issues", total)
    }

    // ── PersistentGoalManager handlers ──

    pub fn handle_goal_manager_create_tick(&mut self) -> String {
        match self.goal_manager.as_mut() {
            Some(gm) => {
                use crate::neotrix::nt_mind::goal_loop::types::GoalPriority;
                match gm.create_goal("consciousness-driven goal", GoalPriority::Medium) {
                    Ok(id) => format!("goal:created_{}", id),
                    Err(e) => format!("goal:error_{}", e),
                }
            }
            None => "goal:unwired".to_string(),
        }
    }

    pub fn handle_goal_manager_execute_tick(&mut self) -> String {
        match self.goal_manager.as_mut() {
            Some(gm) => {
                let ids: Vec<String> = gm.active_goals().iter().map(|g| g.id.clone()).collect();
                if ids.is_empty() {
                    return "goal:no_active".to_string();
                }
                use crate::core::nt_core_agent::lead_agent::PlanEffort;
                let mut summary = String::new();
                for id in &ids {
                    if let Ok(results) = gm.execute_goal(id, PlanEffort::Balanced) {
                        summary.push_str(&format!("{}:{};", id, results.len()));
                    }
                }
                format!("goal:executed_{}", summary)
            }
            None => "goal:unwired".to_string(),
        }
    }

    pub fn handle_goal_manager_status_tick(&mut self) -> String {
        match self.goal_manager.as_ref() {
            Some(gm) => gm.summary(),
            None => "goal:unwired".to_string(),
        }
    }

    pub fn handle_goal_manager_pause_tick(&mut self) -> String {
        match self.goal_manager.as_mut() {
            Some(gm) => {
                let ids: Vec<String> = gm.active_goals().iter().map(|g| g.id.clone()).collect();
                let mut paused = 0usize;
                for id in &ids {
                    if gm.pause_goal(id).is_ok() {
                        paused += 1;
                    }
                }
                format!("goal:paused_{}", paused)
            }
            None => "goal:unwired".to_string(),
        }
    }

    pub fn handle_goal_manager_resume_tick(&mut self) -> String {
        match self.goal_manager.as_mut() {
            Some(gm) => {
                let all: Vec<String> = gm.goals.keys().cloned().collect();
                let mut resumed = 0usize;
                for id in &all {
                    if gm.resume_paused_goal(id).is_ok() {
                        resumed += 1;
                    }
                }
                format!("goal:resumed_{}", resumed)
            }
            None => "goal:unwired".to_string(),
        }
    }

    pub fn handle_goal_manager_cancel_tick(&mut self) -> String {
        match self.goal_manager.as_mut() {
            Some(gm) => {
                let ids: Vec<String> = gm.active_goals().iter().map(|g| g.id.clone()).collect();
                let mut cancelled = 0usize;
                for id in &ids {
                    if gm.cancel_goal(id).is_ok() {
                        cancelled += 1;
                    }
                }
                format!("goal:cancelled_{}", cancelled)
            }
            None => "goal:unwired".to_string(),
        }
    }

    // ── KnowledgeBase tick ──

    pub fn handle_permission_set_mode_tick(&mut self) -> String {
        let mode = match self.permission_gate.mode {
            PermissionMode::AllowAll => "allow-all",
            PermissionMode::DenyAll => "deny-all",
            PermissionMode::AskHuman => "ask-human",
            PermissionMode::AutoClassify => "auto-classify",
        };
        format!("permission_mode:{}", mode)
    }

    pub fn handle_permission_check_tick(&mut self) -> String {
        let mode = self.permission_gate.mode.name();
        let allow_count = self.permission_gate.allow_list.len();
        let deny_count = self.permission_gate.deny_list.len();
        format!(
            "permission:mode={},allow={},deny={}",
            mode, allow_count, deny_count
        )
    }

    pub fn handle_permission_override_tick(&mut self) -> String {
        let count = self.permission_overrides.agent_overrides.len();
        let global = self
            .permission_overrides
            .global_override
            .map(|m| m.name())
            .unwrap_or("none");
        format!(
            "permission_overrides:agent_count={},global={}",
            count, global
        )
    }

    pub fn handle_verify_check_tick(&mut self) -> String {
        let pass_rate = self.verify_loop.pass_rate();
        let history = self.verify_loop.history.len();
        let enabled = self.verify_loop.enabled;
        format!(
            "verify:pass_rate={:.2},history={},enabled={}",
            pass_rate, history, enabled
        )
    }

    pub fn handle_verify_toggle_tick(&mut self) -> String {
        self.verify_loop.enabled = !self.verify_loop.enabled;
        format!("verify:toggled={}", self.verify_loop.enabled)
    }

    pub fn handle_dispatch_pipeline_mode_tick(&mut self) -> String {
        format!("dispatch_pipeline:mode=standard_cycle={}", self.cycle)
    }

    // ─── Transcript handlers ─────────────────────────────────────────
}
