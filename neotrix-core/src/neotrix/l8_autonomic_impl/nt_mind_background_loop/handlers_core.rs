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

        // ── 对话吸收桥 (DialogueAbsorbBridge) ──
        // 把 KB 近期 session/experience 蒸馏出的能力向量反哺 SelfIteratingBrain:
        // 对话经验从"只落 KB"升级为"参与脑能力进化"。释放写锁前消费。
        if let Some(ref bridge) = self.dialogue_bridge {
            let outcome = bridge.absorb_pending(&mut b);
            if outcome.absorbed > 0 {
                eprintln!(
                    "[bg-agent] dialogue absorb: {} experiences -> brain (critic_accepted={}, score_delta={:+.4})",
                    outcome.absorbed, outcome.critic_accepted, outcome.score_delta
                );
            }
        }
        // ── 元认知 agent 外壳 (MetaAgentShell) + 派单执行桥 (P0) ──
        // 多域刺激: 依据当前活动目标的内容派发不同注意力域 (不再永远只刺激
        // SelfReflection)。派单结果经 AgentExecutor 接到真实子系统并执行,
        // 把实测执行成败喂回 RouteLearner — 星系派单从仪式变控制面。
        if let Some(ref mut shell) = self.meta_shell {
            let task_hint = self.goal_loop.active_goal.as_ref()
                .map(|g| g.description.clone())
                .unwrap_or_default();
            // 依据目标语义刺激对应注意力域 (research→PatternMatch, 编码→Code, ...)
            for (domain, amount) in crate::neotrix::nt_mind::evolution::agent_capability::domains_for_goal(&task_hint) {
                shell.stimulate(domain, amount);
            }
            // P2: 星系能力网络 → 控制面 — 树的分支薄弱信号驱动派单。
            // ConsciousnessTree 的 branch health/fog/constellation 此前是纯观测,
            // 现转为真实派单输入: 薄弱分支刺激对应注意力域, 派单去强化它。
            if let Some(ref tree) = self.consciousness_tree {
                let branch_stimuli = crate::neotrix::nt_mind::evolution::agent_capability::tree_branch_stimuli(tree);
                if !branch_stimuli.is_empty() {
                    eprintln!(
                        "[bg-meta] tree-control: {} branch signals -> attention",
                        branch_stimuli.len(),
                    );
                }
                for (domain, amount) in branch_stimuli {
                    shell.stimulate(domain, amount);
                }
            }
            let exec_task = if task_hint.is_empty() {
                "general_dialogue_tick"
            } else {
                task_hint.as_str()
            };
            if let Some(executor) = self.agent_executor.as_ref() {
                if let Some((agent, outcome)) = shell.dispatch_and_execute(executor, exec_task) {
                    eprintln!(
                        "[bg-meta] dispatched={} -> {}",
                        agent,
                        outcome.summary(),
                    );
                    // P1: 每次派单后把行为统计落盘 KB — 学习跨会话累积。
                    if let Some(ref kb_ref) = self.kb {
                        if let Err(e) = shell.learner.persist(kb_ref) {
                            eprintln!("[bg-meta] route_learner persist failed: {}", e);
                        }
                    }
                    // P4: MAGE 四子图共进化循环同步落盘 — 同一 reward 驱动的图谱 +
                    // 任务级搜索 bandit 跨会话存活 (append-only, 重启后继续累积)。
                    if let Some(ref kb_ref) = self.kb {
                        if let Err(e) = shell.persist_coevo(kb_ref) {
                            eprintln!("[bg-meta] coevolution loop persist failed: {}", e);
                        }
                    }
                    // P6: 派单经验回读 — coevo 经验子图 → 大脑吸收闭环 (R-P79)。
                    // 水位之上的新经验 (成败双索引) 经 EDV 批评器反哺 SelfIteratingBrain,
                    // 同一 coevo 图谱: 写 (record_reward) 在派单时, 读 (吸收) 在派单后。
                    if let Some(ref bridge) = self.dialogue_bridge {
                        let absorb = bridge.absorb_dispatch_experiences(&mut b, &mut shell.coevo);
                        if absorb.absorbed > 0 {
                            eprintln!(
                                "[bg-agent] dispatch absorb: {} experiences -> brain (critic_accepted={}, score_delta={:+.4})",
                                absorb.absorbed, absorb.critic_accepted, absorb.score_delta
                            );
                        }
                        // 水位推进后落盘, 防止重启后重复吸收同一批经验。
                        if let Some(ref kb_ref) = self.kb {
                            if let Err(e) = shell.persist_coevo(kb_ref) {
                                eprintln!("[bg-meta] coevolution watermark persist failed: {}", e);
                            }
                        }
                    }
                    // P3: MANTA trace 审计 + 有界结构修复 — 派单后依据行为 trace
                    // 检查派单拓扑, 当前组织不足则改边 (域→档案), 并落盘 playbook。
                    let repairs = shell.audit_and_repair_topology();
                    if !repairs.is_empty() {
                        eprintln!(
                            "[bg-meta] topology repair x{} (revision={}): {}",
                            repairs.len(),
                            shell.topology.revision,
                            repairs.iter().map(|r| r.summary()).collect::<Vec<_>>().join("; "),
                        );
                        if let Some(ref kb_ref) = self.kb {
                            if let Err(e) = shell.persist_topology(kb_ref) {
                                eprintln!("[bg-meta] dispatch_topology persist failed: {}", e);
                            }
                        }
                    }
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
            match d.listen() {
                Ok(discovered) if discovered > 0 => {
                    // D17: listen 排空返回新增数 — 有新增即触发可观测信号 (日志 + 状态), 供感知层响应
                    eprintln!("[bg] discovery: +{} agents ({} known)", discovered, d.known_agents.len());
                }
                Ok(_) => {}
                Err(e) => log::warn!("[bg] discovery: {}", e),
            }
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
            // ── 研究结论吸收闭环 (R-P79: 搜索结论 → KB → 脑能力进化) ──
            // 好奇心查询经统一搜索 (DDG→Wikipedia 有序后端) 产出结论, 由
            // DialogueAbsorbBridge 落 KB + 蒸馏反哺 SelfIteratingBrain。
            // 与 evolve_from_url 并列: 前者进化 self_evolver 爬取, 后者进化脑能力面。
            if let Some(ref bridge) = self.dialogue_bridge {
                let mut b = self.brain.write().await;
                let outcome = bridge.absorb_research_query(&mut b, q, 5);
                if outcome.absorbed > 0 {
                    eprintln!(
                        "[bg-research] query={} absorbed={} nodes -> brain (critic_accepted={}, score_delta={:+.4})",
                        q, outcome.absorbed, outcome.critic_accepted, outcome.score_delta
                    );
                }
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
                // ── SEAL 微迭代 (R-P79: 吸收即进化, 禁止延期死代码) ──
                // 爬取数据吸收进 KB 后, 触发一次 SelfIteratingBrain 微迭代,
                // 让新知识立即参与能力进化而非仅仅落盘。门控: 有实际吸收才跑。
                if r.absorbed > 0 {
                    let mut b = self.brain.write().await;
                    let task = format!("knowledge_chain_absorb_d{}_m{}_a{}", r.discovered, r.mined, r.absorbed);
                    match b.run_seal_loop_pipeline(&task, None, Some(r.total_reward)) {
                        Ok(reward) => {
                            eprintln!("[bg] knowledge chain -> seal micro-iteration: reward={:.3}", reward);
                        }
                        Err(e) => {
                            log::debug!("[bg] knowledge chain seal micro-iteration skipped: {}", e);
                        }
                    }
                }
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
