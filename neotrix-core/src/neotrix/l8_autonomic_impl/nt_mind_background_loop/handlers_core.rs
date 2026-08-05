use super::*;
use crate::neotrix::nt_mind::MemoryAgentCapability;

impl BackgroundLoopHandle {
    pub(crate) async fn handle_save(&mut self) {
        let b = self.brain.read().await;
        if let Err(e) = b.brain.save() { eprintln!("[bg] save: {}", e); }
        if let Err(e) = self.goal_loop.save() { log::warn!("[background] save goal_loop: {}", e); }
    }

    pub(crate) async fn handle_consolidate(&mut self) {
        let mut b = self.brain.write().await;
        let r = b.consolidate_memories();
        eprintln!("[bg] consolidated: {} merge, {} prune, {} replay",
            r.merged_count, r.pruned_count, r.replayed_count);
        // Persist any pending memory orchestrator entries to KB.
        // Currently a no-op until a DualTrackEntry producer is wired in;
        // the connector is established here for future OMP integration.
        let kb = b.reasoning_engine.as_mut().and_then(|e| e.kb.take());
        if let Some(kb_inner) = kb {
            let count = b.persist_pending_entries(&kb_inner);
            if count > 0 {
                eprintln!("[bg] persisted {} memory orchestrator entries", count);
            }
            if let Some(ref mut engine) = b.reasoning_engine {
                engine.kb = Some(kb_inner);
            }
        }

        // ── Dreaming 巩固 (P0-3, skales 三阶段: 重组→提纯→巩固) ──
        // 用主库近期节点内容构造 VSA 事件 (seeded_random 确定性向量),
        // 跑 run_consolidation_cycle + prune_low_coherence。这是记忆大脑的
        // 夜间整理 — 此前 dream_consolidation 零生产调用。
        use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
        use crate::core::nt_core_hcube::dream_consolidation::{DreamConfig};
        let dream_inputs: Vec<(String, f64)> = {
            if let Some(ref kb) = self.kb {
                if let Ok(nodes) = kb.all_nodes() {
                    nodes
                        .into_iter()
                        .take(64)
                        .filter_map(|n| {
                            let text = n.title;
                            let salience = n.importance.max(0.0).min(1.0);
                            Some((text, salience))
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };
        if !dream_inputs.is_empty() {
            for (label, salience) in dream_inputs {
                let mut h = std::collections::hash_map::DefaultHasher::new();
                use std::hash::Hasher;
                h.write(label.as_bytes());
                let seed = h.finish();
                let vector = QuantizedVSA::seeded_random(seed, VSA_DIM);
                self.dream.record_event(vector, &label, salience);
            }
            let report = self.dream.run_consolidation_cycle();
            self.dream.prune_low_coherence(DreamConfig::default().merge_threshold);
            eprintln!(
                "[bg-dream] replayed={} merged={} abstracted={} pred={} novelty={:.2} coherence_gain={:.2}",
                report.sequences_replayed, report.patterns_merged,
                report.abstractions_formed, report.predictions_generated,
                report.novelty_score, report.coherence_gain,
            );
        }
    }

    pub(crate) async fn handle_goal(&mut self) {
        // ── 门控前置检查: 爆炸半径分级 × 护栏 × 评审组 ──
        if let Some(ref registry) = self.gate_registry {
            let input = crate::core::nt_core_gate::JudgeInput {
                candidate: "autonomous goal pursuit".to_string(),
                claims: vec![crate::core::nt_core_gate::Claim::new("pursue next goal", &["internal:goal_loop"])],
                evidence_ids: vec!["internal:goal_loop".to_string()],
                trajectory: None,
                grounding_failures: self.tool_grounding.grounding_failures,
                schema_failures: vec![],
                producer_family: crate::core::nt_core_gate::JudgeFamily::None,
            };
            let panel = crate::core::nt_core_gate::JudgePanel::default_panel();
            let decision = GateDecision::check_path(&registry.cloned_specs(), &input, &panel);
            if !decision.allows_autonomous() {
                eprintln!("[bg] gate blocked goal pursuit: level={:?} action={:?} verdict={:?} reason={}",
                    decision.level, decision.action, decision.verdict, decision.reason);
                return; // 阻断本轮 goal pursuit
            }
        }

        let mut b = self.brain.write().await;
        // Full autonomous cycle: circuit-breaker guard, terminal-goal
        // reward/dequeue, auto-generation of new goals when none is active,
        // then one pursue_iteration. pursue_all(.., 1) bails early when no
        // active goal exists and never exercises the auto-generation path.
        self.goal_loop.pursue_auto_iteration(&mut b);

        // ── 记忆大脑 agent 能力接线 (R-P42/R-P79) ──
        // goal 迭代后顺带跑一次记忆能力面: 巩固规模信号 + 证据计数,
        // 让 meta_agent 不是死代码而是生产路径上的真实消费者。
        if let Some(ref agent) = self.meta_agent {
            if let Ok(outcome) = agent.capability_consolidate() {
                match outcome {
                    crate::neotrix::nt_mind::CapabilityOutcome::Count(n) => {
                        eprintln!("[bg-agent] memory consolidate: nodes={}", n);
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) async fn handle_prediction(&mut self) {
        if let Some(ref mut pano) = self.panorama {
            let mut brain = self.brain.write().await;
            if let Some(ref mut wm) = self.nt_world_model {
                let r = pano.run_cycle(&mut brain, &mut self.goal_loop, wm);
                eprintln!("[bg] prediction: cycle={}, anomaly={}", r.cycle, r.anomaly);
            }
        }
    }

    pub(crate) async fn handle_exploration(&mut self) {
        let p = dirs::home_dir().unwrap_or_default().join(".neotrix").join("exploration_sources.txt");
        let urls: Vec<String> = std::fs::read_to_string(&p).unwrap_or_default()
            .lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
        if !urls.is_empty() {
            if let Some(ref mut ev) = self.self_evolver {
                for url in &urls { let _ = ev.evolve_from_url(url); }
            }
            let _ = std::fs::write(&p, "");
        }
        if let Some(ref mut gd) = self.gap_detector {
            use crate::core::nt_core_meta::scanner::CodeScanner;
            use crate::core::nt_core_meta::weakness::WeaknessAnalyzer;
            let m = CodeScanner::new(".").scan();
            let w = WeaknessAnalyzer::new().analyze(&m);
            let r = gd.detect_gaps(&m, &w.weaknesses);
            if r.high_priority_count > 0 {
                eprintln!("[bg] gaps: {} total, {} high", r.total_gaps, r.high_priority_count);
            }
        }
    }

    pub(crate) async fn handle_world_sense(&mut self) {
        if let Some(ref mut wc) = self.world_consciousness {
            wc.refresh_self_awareness();
            eprintln!("[bg] world_sense: active={} status={}", wc.active, wc.consciousness_status().len());
        }
    }

    pub(crate) async fn handle_always_on(&mut self) {
        if self.always_on.enabled {
            if let Ok(r) = self.always_on.full_cycle() {
                if r.tasks_executed > 0 {
                    eprintln!("[bg] always_on: scanned={}, done={}", r.scan_count, r.tasks_executed);
                    if let Err(e) = self.always_on.save() { log::warn!("[background] save always_on: {}", e); }
                }
            }
        }
    }

    pub(crate) async fn handle_nt_act_voice_tick(&mut self) {
        if let Some(ref mut vi) = self.nt_act_voice_input {
            if vi.is_active() && vi.is_continuous() {
                if let Some(t) = vi.poll_transcription() { eprintln!("[voice] {}", t); }
            }
        }
    }

    pub(crate) async fn handle_plugin_tick(&mut self) {
        use crate::neotrix::nt_io_plugin::PluginEvent;
        self.plugin_registry.dispatch(&PluginEvent::BrainTick).await;
    }

    pub(crate) async fn handle_agent_discovery(&mut self) {
        if let Some(ref mut d) = self.agent_discovery {
            if let Err(e) = d.listen() { log::warn!("[bg] discovery: {}", e); }
        }
    }

    pub(crate) async fn handle_curiosity(&mut self) {
        use crate::neotrix::nt_mind::hypercube_bridge::HyperCubeBridge;
        // 用真实皮层数据构建桥接，而非空桥 — 好奇心 gap 检测必须基于实际知识分布。
        let mut bridge = HyperCubeBridge::new();
        let cortex_traces = self.panorama.as_ref()
            .map(|p| p.cortex.all_traces().len())
            .unwrap_or(0);
        if let Some(ref pano) = self.panorama {
            bridge.ingest_from_cortex(&pano.cortex);
        }
        let gaps = bridge.analyze_gaps();
        self.curiosity_drive.ingest_gap_reports(&gaps);
        for q in self.curiosity_drive.drain_queries().iter().take(2) {
            if let Some(ref mut ev) = self.self_evolver {
                let url = format!("https://en.wikipedia.org/wiki/{}", q.replace(' ', "_"));
                let _ = ev.evolve_from_url(&url);
            }
        }
        log::debug!("[bg] curiosity: cortex_traces={} gaps={} queries={}",
            cortex_traces, gaps.len(), self.curiosity_drive.top_signals(3).len());
    }

    /// Log a session event to KB (obsidian-mind SessionStart/Stop pattern).
    /// Creates a node with type=Session with event type and summary.
    #[allow(dead_code)]
    pub(crate) async fn log_session_event(&self, event_type: &str, summary: &str) {
        let Some(ref kb) = self.kb else { return };
        let title = format!("session-{}-{}", event_type, chrono::Utc::now().timestamp());
        let _ = kb.insert_or_get_node(&title, crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType::Session, Some(summary), None, Some("neotrix"));
    }

    pub(crate) async fn handle_knowledge_chain(&mut self) {
        if let Some(ref mut chain) = self.knowledge_chain {
            if !chain.has_pending() { chain.init_default_discovery(); }
            let mut brain = crate::neotrix::nt_mind::self_iterating::ReasoningBrain::new();
            let mut bank = crate::neotrix::nt_mind::memory::ReasoningBank::new(100);
            if let Ok(r) = chain.run_chain(&mut brain, &mut bank) {
                eprintln!("[bg] knowledge chain: discovered={}, mined={}, absorbed={}",
                    r.discovered, r.mined, r.absorbed);
            }
        }
    }

    pub(crate) async fn handle_knowledge_aging(&mut self) {
        let r = self.knowledge_aging.run_aging_cycle();
        if r.stale_count > 0 {
            eprintln!("[bg] aging: {} stale, {} expired", r.stale_count, r.expired_count);
            for url in r.rescans_needed.iter().take(3) {
                if let Some(ref mut ev) = self.self_evolver {
                    if crate::neotrix::nt_mind::self_evolver::SelfEvolver::is_url(url) {
                        let _ = ev.evolve_from_url(url);
                    }
                }
            }
        }
    }

}
