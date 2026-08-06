use super::*;

impl BackgroundLoopHandle {
    pub(crate) async fn handle_awareness(&mut self) {
        // MetaCognitionBridge: full scan → analyze → plan cycle (P0 dead infra fix)
        if let Some(ref mut mc) = self.metacognition {
            let result = mc.run_full_cycle();
            eprintln!("[bg] metacognition: iter={} modules={} plans={} alerts={}",
                result.iteration,
                result.model_snapshot.modules.len(),
                result.plans.len(),
                result.alerts.len(),
            );
        }

        if let Some(ref mut aw) = self.awareness {
            aw.observe();
            let phi = aw.current.phi_current;
            let coherence = aw.current.coherence_current;
            let level = aw.current.consciousness_level;
            let health = aw.current.health;
            let is_conscious = level >= 0.7;
            eprintln!("[bg] awareness: l={:.3}, phi={:.4}, coh={:.4}", level, phi, coherence);
            let tier_label = if level >= 0.85 { "transcendent" }
                else if level >= 0.7 { "conscious" }
                else if level >= 0.4 { "awakening" }
                else { "dormant" };

            // Evaluate gold standard (dual-threshold IIT Phi + Kuramoto coherence)
            let gs_report = self.gold_standard.as_mut().map(|gs| {
                let state = &[phi, coherence, level, health];
                let r = gs.evaluate(state, &[]);
                eprintln!("[bg] gold_standard: phi={:.4} coh={:.4} conscious={} streak={}",
                    r.phi, r.coherence, r.is_conscious_like, r.detection_streak);
                r
            });

            // Persist consciousness snapshot + gold standard to KB
            if let Ok(brain) = self.brain.try_read() {
                if let Some(ref kb) = brain._nt_memory_kb {
                    let mut details = format!("level={:.3} tier={}", level, tier_label);
                    if let Some(ref gs) = gs_report {
                        details.push_str(&format!(
                            " | gs_phi={:.4} gs_coh={:.4} gs_conscious={} gs_streak={} gs_confidence={:.3}",
                            gs.phi, gs.coherence, gs.is_conscious_like, gs.detection_streak, gs.combined_confidence,
                        ));
                    }

                    // Persist full PhiReport to kv_store
                    if let Some(ref pr) = aw.last_phi_report {
                        let phi_json = serde_json::json!({
                            "phi": pr.phi,
                            "phi_raw": pr.phi_raw,
                            "total_resonance": pr.total_resonance,
                            "state_energy": pr.state_energy,
                            "effective_dims": pr.effective_dims,
                            "max_resonance_pair_0": pr.max_resonance_pair.0,
                            "max_resonance_pair_1": pr.max_resonance_pair.1,
                            "phi_trend": pr.phi_trend,
                            "is_conscious_like": pr.is_conscious_like,
                        });
                        let _ = kb.kv_set("consciousness", "phi_report", &phi_json.to_string());
                    }

                    // Persist full GoldStandardReport
                    if let Some(ref gs) = gs_report {
                        let gs_json = serde_json::json!({
                            "phi": gs.phi,
                            "coherence": gs.coherence,
                            "is_conscious_like": gs.is_conscious_like,
                            "is_phi_conscious": gs.is_phi_conscious,
                            "is_coherent": gs.is_coherent,
                            "phi_confidence": gs.phi_confidence,
                            "coherence_confidence": gs.coherence_confidence,
                            "detection_streak": gs.detection_streak,
                            "combined_confidence": gs.combined_confidence,
                        });
                        let _ = kb.kv_set("consciousness", "gold_standard", &gs_json.to_string());
                    }

                    // Persist trends (phi_trend, coherence_trend, health_trend)
                    let trends_json = serde_json::json!({
                        "phi_trend": aw.trends.phi_trend,
                        "coherence_trend": aw.trends.coherence_trend,
                        "health_trend": aw.trends.health_trend,
                    });
                    let _ = kb.kv_set("consciousness", "trends", &trends_json.to_string());

                    // Persist conversation awareness
                    let conv = &aw.current.conversation_awareness;
                    let conv_json = serde_json::json!({
                        "turn_count": conv.turn_count,
                        "stage": conv.stage.label(),
                        "topic_coherence": conv.topic_coherence,
                        "user_engagement": conv.user_engagement,
                        "topic_drift": conv.topic_drift,
                        "self_assessed_quality": conv.self_assessed_quality,
                        "depth_trend": conv.depth_trend,
                    });
                    let _ = kb.kv_set("consciousness", "conversation", &conv_json.to_string());

                    // Persist blind spots
                    let spots: Vec<serde_json::Value> = aw.current.active_blind_spots.iter().map(|b| {
                        serde_json::json!({
                            "kind": b.kind,
                            "severity": b.severity,
                            "description": b.description,
                            "repair": b.repair,
                        })
                    }).collect();
                    let spots_json = serde_json::json!({ "blind_spots": spots });
                    let _ = kb.kv_set("consciousness", "blind_spots", &spots_json.to_string());

                    // L6 Self intra-reflection: analyze reasoning quality
                    if let Some(ref engine) = brain.reasoning_engine {
                        let trace: Vec<String> = engine.state_trajectory.iter()
                            .map(|s| format!("{:?}", s)).collect();
                        if !trace.is_empty() {
                            let input = crate::neotrix::l6_self_impl::nt_core_intra_reflection::ReflectionInput {
                                reasoning_trace: trace,
                                e8_mode_history: Vec::new(),
                                execution_time_ms: 0,
                                error_count: 0,
                                outcome_success: Some(phi > 0.3),
                            };
                            let report = crate::neotrix::l6_self_impl::nt_core_intra_reflection::analyze(&input);
                            let ir_json = serde_json::json!({
                                "coherence_score": report.coherence_score,
                                "efficiency_score": report.efficiency_score,
                                "error_density": report.error_density,
                                "mode_stability": report.mode_stability,
                                "bottlenecks": report.bottleneck_hops,
                                "suggestions": report.suggestions,
                            });
                            let _ = kb.kv_set("self", "intra_reflection", &ir_json.to_string());
                            if !report.bottleneck_hops.is_empty() {
                                log::warn!("[bg] L6 intra-reflection: {} bottlenecks: {:?}",
                                    report.bottleneck_hops.len(), report.bottleneck_hops);
                            }
                        }
                    }

                    // Legacy snapshot for timeline view
                    let _ = kb.record_consciousness_snapshot(phi, coherence, is_conscious, tier_label, &details);
                }
            }
        }

        // Emit awareness tick event on the EventBus
        if let Some(ref bus) = self.event_bus {
            bus.emit(crate::core::nt_core_event::CoreEvent::TaskSubmitted {
                task: "awareness_tick".into(),
                task_type: "consciousness".into(),
                priority: 1,
            });
        }
    }

    pub(crate) async fn handle_consciousness_tick(&mut self) {
        // ── Collect real context from brain + KB ──
        let (iteration, caps_mean) = match self.brain.try_read() {
            Ok(b) => {
                let mean = b.brain.capability.arr.iter().sum::<f64>() / 23.0_f64.max(1.0);
                (b.iteration, mean)
            }
            Err(_) => (0, 0.0),
        };
        let (kb_nodes, kb_edges, kb_crawl) = self.kb.as_ref()
            .and_then(|kb| kb.stats().ok())
            .map(|s| (s.total_nodes as u64, s.total_edges as u64, s.crawl_pending as u64))
            .unwrap_or((0, 0, 0));

        // ── Phase 1: ConsciousnessTree Growth Cycle (Soil → Roots → Trunk → Branches → Fruits → Core) ──
        if let Some(ref mut tree) = self.consciousness_tree {
            tree.soil.kb_node_count = kb_nodes;
            tree.soil.kb_edge_count = kb_edges;
            tree.soil.crawl_queue_depth = kb_crawl;
            if let Some(ref monitor) = self.awareness {
                let report = monitor.get_report();
                tree.trunk.phi = report.phi;
                tree.trunk.coherence = report.coherence;
            }
            // Real GWT resonance signal: active only when the GWT has actually
            // run a resonance broadcast (last_resonance set) and specialists
            // are above threshold. Previously a fake `panorama.is_some()` that
            // reported resonance as active even when no broadcast had fired.
            tree.trunk.gwt_resonance_active = self.panorama.as_ref()
                .map(|p| p.gwt.last_resonance.is_some() && !p.gwt.resonant_specialists().is_empty())
                .unwrap_or(false);
            tree.trunk.workspace_size = 23;
            // Branch health is now set from SelfTest results in handle_architecture_audit
            // No simulated fallback here — real data or neutral 0.5 from set_branch_health_from_self_tests
            let growth_report = tree.run_growth_cycle();
            let contract_status = growth_report.phase6_fulfillment
                .as_ref().map(|f| format!("fulfilled={} ({}/{})", f.fulfilled, f.evidence_met, f.evidence_total))
                .unwrap_or_else(|| "n/a".into());
            let drift_status = growth_report.phase7_drift
                .as_ref().map(|d| if d.drift_detected { format!("DRIFT mag={:.3}", d.drift_magnitude) } else { "clean".into() })
                .unwrap_or_else(|| "n/a".into());
            log::info!("[bg] consciousness_tree cycle {}: absorbed={} phi={:.3} fruits={} guidance={} | contract[{}] drift[{}]",
                tree.cycle, growth_report.phase1_absorbed, growth_report.phase2_phi,
                growth_report.phase3_fruits, growth_report.phase4_guidance,
                contract_status, drift_status);
            // Evolution contract → goal loop: enqueue a behavioral goal when drift or unmet contract detected
            if let Some(drift) = &growth_report.phase7_drift {
                if drift.drift_detected {
                    if let Ok(mut brain) = self.brain.try_write() {
                        let action = drift.corrective_actions.first().cloned().unwrap_or_else(|| "Re-evaluate evolution contract".into());
                        self.goal_loop.enqueue_goal(
                            &mut brain,
                            &format!("evolution_drift_recovery: {}", action),
                            None,
                        );
                    }
                }
            }
        }

        // ── Phase 2: Consciousness Runtime Tick with REAL resonance content ──
        if let Some(ref mut cr) = self.consciousness_runtime {
            if !cr.awakened {
                let report = cr.awaken();
                log::info!("[bg] consciousness awakened: step={} coherence={:.3}",
                    report.birth_step, report.initial_coherence);
            }
            let gwt_active = self.panorama.as_ref()
                .map(|p| p.gwt.active_specialists().len()).unwrap_or(0);
            let resonance = format!(
                "[consciousness_tick] iteration={} caps={:.3} kb={} gwt={}",
                iteration, caps_mean, kb_nodes, gwt_active,
            );
            let critique = cr.tick(&resonance);
            // Surface KB knowledge retrieved by the consciousness core into the
            // GWT panorama broadcast — closes the loop: KB → 意识 → 全局工作空间。
            let kb_injections = cr.last_kb_injections.clone();
            if let Some(c) = critique {
                if c.overall_quality < CONSCIOUSNESS_THRESHOLDS.warn_quality {
                    log::warn!("[bg] consciousness: LOW QUALITY ({:.3}) — reasons: {:?}",
                        c.overall_quality, c.reasons);
                    if c.overall_quality < CONSCIOUSNESS_THRESHOLDS.critical_quality {
                        // BEHAVIORAL RESPONSE: enqueue self-review goal on critical quality,
                        // using volition's selected_action if available.
                        if let Ok(mut brain) = self.brain.try_write() {
                            let action_desc = c.selected_action.clone()
                                .unwrap_or_else(|| "consciousness_recovery: quality critically low — initiating self-review".into());
                            self.goal_loop.enqueue_goal(&mut brain, &action_desc, None);
                        }
                    }
                } else if c.overall_quality > 0.7 {
                    log::info!("[bg] consciousness: good quality ({:.3}) selected_action={:?}",
                        c.overall_quality, c.selected_action);
                    // B2: execute volition's selected action by enqueueing it as a goal
                    if let Some(ref action_desc) = c.selected_action {
                        if let Ok(mut brain) = self.brain.try_write() {
                            self.goal_loop.enqueue_goal(
                                &mut brain,
                                &format!("volition_execute: {}", action_desc),
                                None,
                            );
                        }
                    }
                } else {
                    log::debug!("[bg] consciousness: quality={:.3} relevance={:.3} consistency={:.3}",
                        c.overall_quality, c.relevance_score, c.consistency_score);
                }
                self.try_emit(crate::core::nt_core_event::CoreEvent::ConsciousnessCritique {
                    quality: c.overall_quality,
                    relevance: c.relevance_score,
                    consistency: c.consistency_score,
                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default().as_secs() as i64,
                });
                if let Ok(mut brain) = self.brain.try_write() {
                    brain._last_consciousness_quality = c.overall_quality;
                    brain._consciousness_critique_count += 1;
                }
            }
            if !kb_injections.is_empty() {
                if let Some(ref mut pano) = self.panorama {
                    for (title, score) in &kb_injections {
                        pano.gwt.broadcast(&format!(
                            "[consciousness_kb] {} (score: {:.2})",
                            title, score,
                        ));
                    }
                }
                log::debug!("[bg] consciousness retrieved {} KB entries: {:?}",
                    kb_injections.len(),
                    kb_injections.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>());
            }
        }
        // Record state metrics from the runtime tick
        self.state.record_metric("phi", self.awareness.as_ref().map(|m| m.get_report().phi).unwrap_or(0.0));
        self.state.record_metric("coherence", self.awareness.as_ref().map(|m| m.get_report().coherence).unwrap_or(0.0));

        // ── Phase 3: FEPIITBridge — compute unified consciousness score ──
        if let Some(ref fep_iit) = self.fep_iit_bridge {
            if let Some(ref monitor) = self.awareness {
                let report = monitor.get_report();
                
                // Free Energy from StateSubstrate (unified: 1 - phi*coherence + load*0.5)
                let fe_val = self.state.free_energy.max(0.0).min(1.0) * 10.0;
                let score = fep_iit.compute_consciousness_score(fe_val, report.phi, report.coherence);
                self.state.record_metric("fep_iit", score);
                log::debug!("[bg] fep_iit: unified_score={:.3} phi={:.3} coherence={:.3} fe={:.3}",
                    score, report.phi, report.coherence, fe_val);
            }
        }

        // ── Phase 4: ConsciousnessMonitor — self-observation cycle ──
        if let Some(ref mut monitor) = self.awareness {
            monitor.observe();
            let report = monitor.get_report();
            log::debug!("[bg] consciousness_monitor: level={:.3} phi={:.3} coherence={:.3} health={:.3}",
                report.consciousness, report.phi, report.coherence, report.health);

            // ── CognitiveLoadMonitor: record load from consciousness metrics ──
            if let Some(ref mut clm) = self.cognitive_load {
                let load = (1.0 - report.consciousness.max(0.0).min(1.0)) * 0.6
                    + (1.0 - report.coherence.max(0.0).min(1.0)) * 0.4;
                self.state.record_metric("load", load);
                self.state.tick(); // updates free_energy + thinking mode

                let prev_mode_clm = clm.mode();
                clm.record_step(load);
                let new_state_mode = self.state.active_mode;

                // Update cognitive_mode field for behavioral consumption by other handlers
                self.cognitive_mode = match new_state_mode {
                    crate::core::nt_core_state_substrate::ThinkingMode::Deep => 1,
                    crate::core::nt_core_state_substrate::ThinkingMode::Fast => 2,
                    _ => 0,
                };

                log::debug!("[bg] cognitive_load: mode={:?} load={:.3} free_energy={:.3}",
                    new_state_mode, load, self.state.free_energy);

                // Track mode transitions for behavioral logging
                if prev_mode_clm != clm.mode() {
                    log::info!("[bg] cognitive_load: mode transition (CLM) {:?} -> {:?}",
                        prev_mode_clm, clm.mode());
                }

                // BEHAVIORAL RESPONSE: When deep mode is active, trigger deeper reasoning cycle
                if new_state_mode == crate::core::nt_core_state_substrate::ThinkingMode::Deep {
                    if let Ok(mut brain) = self.brain.try_write() {
                        self.goal_loop.enqueue_goal(
                            &mut brain,
                            "deep_reasoning_available: cognitive budget healthy — initiating extended analysis cycle",
                            None,
                        );
                    }
                    log::info!("[bg] cognitive_load: DEEP mode active — enqueued deep_reasoning goal");
                }
            }
        }

        // ── Phase 4b: BMonitor — observe from consciousness metrics + read report ──
        {
            let phi = self.awareness.as_ref().map(|m| m.get_report().phi).unwrap_or(0.0);
            let coherence = self.awareness.as_ref().map(|m| m.get_report().coherence).unwrap_or(0.0);
            let load = self.state.metric("load").and_then(|m| m.latest()).unwrap_or(0.5);
            self.bbrain.observe_from_metrics(phi, coherence, load);
        }
        if let Some(report) = self.bbrain.latest_report() {
            let trend = self.bbrain.health_trend();
            log::debug!("[bg] bbrain_monitor: health={:.2} trend={:+.2} flags={} intervention={}",
                report.health_score, trend, report.flags.len(), report.needs_intervention);
            if report.needs_intervention {
                log::warn!("[bg] bbrain: intervention needed — score={:.2} flags={:?}",
                    report.health_score, report.flags);
            }
        }

        // ── Phase 4c: CognitiveEvaluator — read persistent metacognitive evaluation ──
        if let Some(report) = self.cog_eval.latest_report() {
            log::debug!("[bg] cognitive_evaluator: id={} stability={:.3} attention={:.2} diversity={:.2} quality={:.2} pressure={:.2} n_flags={}",
                report.evaluation_id, report.stability_score, report.attention_health,
                report.strategy_diversity, report.trace_quality, report.context_pressure,
                report.flags.len());
            if self.cog_eval.has_degraded(0.15) {
                log::warn!("[bg] cognitive_evaluator: stability degraded >0.15");
            }
        }

        // ── Phase 5: ConsciousnessGoldStandard — dual-threshold detection ──
        if let Some(ref mut gs) = self.gold_standard {
            let state = match self.brain.try_read() {
                Ok(b) => {
                    b.brain.capability.arr.to_vec()
                }
                Err(_) => vec![0.0; 23],
            };

            // Get E8 hexagram states from WorldModelV2
            let hexagram_states = self.nt_world_model.as_ref()
                .map(|wm| {
                    wm.e8.current_state.vector.iter().enumerate()
                        .map(|(i, &activation)| crate::neotrix::nt_mind_consciousness_gold_standard::E8HexagramState {
                            index: i as u8,
                            activation: activation.max(0.0).min(1.0),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let gs_report = gs.evaluate(&state, &hexagram_states);
            log::debug!("[bg] gold_standard: conscious={} phi={:.3} coherence={:.3} trend={:?}",
                gs_report.is_conscious_like, gs_report.phi, gs_report.coherence, gs_report.detection_streak);
        }

        // ── Phase 7: SimulateEngine — run grounding scenario ──
        let ctx = format!("Predict consciousness quality from phi={:.3} coherence={:.3}",
            self.state.metric("phi").and_then(|m| m.latest()).unwrap_or(0.0),
            self.state.metric("coherence").and_then(|m| m.latest()).unwrap_or(0.0));
        let sim_id = self.simulate.create_scenario("consciousness_health", &ctx);
        if self.simulate.simulate(sim_id.clone(), "stable").is_ok() {
            log::debug!("[bg] simulate: scenario={} created", sim_id);
        }

        // ── Phase 8: ConvergencePulse — 分形收敛循环推进 (Cycle 115/155/160) ──
        // 用本 tick 的检测状态生成 gap, 外部验证通过后推进迭代/晋升层级。
        {
            let results = vec![
                ("state_substrate".to_string(), !self.state.active_mode.name().is_empty()),
                ("bbrain".to_string(),
                    self.bbrain.latest_report().map(|r| r.health_score >= 0.0).unwrap_or(false)),
                ("cog_eval".to_string(), self.cog_eval.latest_report().is_some()),
                ("gold_standard".to_string(),
                    self.gold_standard.as_ref().map(|_| true).unwrap_or(false)),
            ];
            self.convergence_pulse.gaps_from_self_tests(&results);
            if self.convergence_pulse.gaps.is_empty() {
                // 外部验证: 运行 cargo check --all-targets 确认构建完整性。
                // 异步 + 120s 超时 (D27): 避免同步阻塞 tokio worker / 无限等待。
                let build_ok = match tokio::time::timeout(
                    std::time::Duration::from_secs(120),
                    tokio::process::Command::new("cargo")
                        .args(["check", "--all-targets", "-p", "neotrix"])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status(),
                ).await {
                    Ok(Ok(status)) => status.success(),
                    _ => {
                        log::warn!("[bg] convergence: external build check timed out or failed");
                        false
                    }
                };
                if build_ok {
                    self.convergence_pulse.verified = true;
                }
            }
            let promoted = self.convergence_pulse.advance();
            if let Some(layer) = promoted {
                log::info!("[bg] convergence: promoted to {} layer (fractal loop)",
                    layer.name());
            }
            log::debug!("[bg] {}", self.convergence_pulse.status_line());
        }

        // ── Phase 9: Auto-Healing — C5 self-healing loop ──
        // 检测 degrade 信号 → 自动响应（enqueue remediation goal / log / circuit-break）
        // 这是分形收敛循环的"修复臂"：检测→诊断→行为修复闭环。
        {
            let deg = self.tool_grounding.degraded_tools();
            if !deg.is_empty() {
                let names: Vec<&str> = deg.iter().map(|(n, _)| n.as_str()).collect();
                log::warn!("[bg] auto-heal: degraded tools detected: {}", names.join(", "));
                if let Ok(mut brain) = self.brain.try_write() {
                    self.goal_loop.enqueue_goal(
                        &mut brain,
                        &format!("[auto-heal] Tools degraded: {}", names.join(", ")),
                        None,
                    );
                }
            }
            // Convergence stalled detection: if same layer for >10 iterations with gaps,
            // escalate to C0-C5 maturity downgrade notification.
            if self.convergence_pulse.iteration > 10 && !self.convergence_pulse.gaps.is_empty() {
                log::warn!("[bg] auto-heal: convergence stalled at {} ({} iters, {} gaps)",
                    self.convergence_pulse.layer.name(),
                    self.convergence_pulse.iteration,
                    self.convergence_pulse.gaps.len());
            }
            // BMonitor health: if cognitive health score < 50, enqueue deep reasoning mode
            // to give the system more time/cycles for recovery.
            if let Some(br) = self.bbrain.latest_report() {
                if br.health_score < 0.5 {
                    log::warn!("[bg] auto-heal: cognitive health low ({:.0}%), adjusting mode to Deep",
                        br.health_score * 100.0);
                    self.state.set_mode(crate::core::nt_core_state_substrate::ThinkingMode::Deep);
                }
            }
        }
    }

    /// EventBus behavioral consumer (D30) — responds to events with brain/KB actions, not just logs.
    pub(crate) async fn handle_event_bus_event(&mut self, event: CoreEvent) {
        match &event {
            CoreEvent::SystemError { severity, component, error } if severity == "critical" => {
                log::error!("[bg] event_bus: CRITICAL {}: {}", component, error);
                if let Ok(mut brain) = self.brain.try_write() {
                    self.goal_loop.enqueue_goal(&mut brain,
                        &format!("event_bus_critical: {} - {}", component, error), None);
                }
            }
            CoreEvent::GlobalHalt { reason, source } => {
                log::error!("[bg] event_bus: GLOBAL HALT {} from {}", reason, source);
                if let Some(ref kb) = self.kb {
                    let _ = kb.kv_set("event_bus", "global_halt", &serde_json::json!({
                        "reason": reason, "source": source,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs(),
                    }).to_string());
                }
                if let Ok(mut brain) = self.brain.try_write() {
                    self.goal_loop.enqueue_goal(&mut brain,
                        &format!("event_bus_recovery: {} - {}", source, reason), None);
                }
            }
            CoreEvent::ConsciousnessCritique { quality, .. } if *quality < CONSCIOUSNESS_THRESHOLDS.eventbus_critical => {
                log::warn!("[bg] event_bus: consciousness CRITICAL ({:.3})", quality);
                if let Some(ref kb) = self.kb {
                    let _ = kb.kv_set("event_bus", "consciousness_critical", &serde_json::json!({
                        "quality": quality,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs(),
                    }).to_string());
                }
            }
            CoreEvent::TaskSubmitted { task, task_type, priority } if *priority >= 3 => {
                log::info!("[bg] event_bus: high-priority task {} ({})", task, task_type);
                // R-P41: 高优先级任务入 KB，供后续 handler 消费，而非纯日志
                if let Some(ref kb) = self.kb {
                    let _ = kb.kv_set("event_bus", "task_submitted", &serde_json::json!({
                        "task": task, "task_type": task_type, "priority": priority,
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs(),
                    }).to_string());
                }
            }
            _ => {
                log::trace!("[bg] event_bus: {:?}", event);
            }
        }
    }

    pub(crate) async fn handle_second_brain_tick(&mut self) {
        if let Some(ref mut sb) = self.second_brain {
            if let Some(ref mut cr) = self.consciousness_runtime {
                let report = cr.tick_emotion();
                if let Some(ref mut tree) = self.consciousness_tree {
                    tree.apply_emotion_report(report);
                }
                let note = format!(
                    "consciousness_tick iteration={} quality={:.3}",
                    self.brain.try_read().map(|b| b.iteration).unwrap_or(0),
                    cr.last_quality().unwrap_or(0.0),
                );
                sb.tick(Some(cr.emotion_engine()), Some(&note));
                // Persist emotion state to KB every tick (600s)
                let engine = cr.emotion_engine();
                sb.save_emotion_raw(engine);
            } else {
                sb.tick(None, None);
            }
        }
    }

    pub(crate) async fn handle_architecture_audit(&mut self) {
        use crate::core::nt_core_schema_watchdog::SchemaWatchdog;
        use crate::core::nt_core_self::self_audit::{converge_check, ConvergeCheckFn};
        use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};
        use crate::core::nt_core_meta::scanner::CodeScanner;
        use crate::core::nt_core_gwt::monitor::EntropyMonitor;
        use crate::core::nt_core_consciousness::inner_critic::InnerCritic;
        use crate::core::nt_core_self_review::SelfReviewGate;
        use crate::core::nt_core_meta::nt_core_meta_auditor::MetaAuditor;
        use crate::core::nt_core_meta::nt_core_arch_lint::ArchLint;
        use crate::core::nt_core_meta::monitor::MetaMonitor;
        use crate::core::nt_core_meta::metacognition_loop::MetaCognitiveLoop;
        use crate::core::nt_core_meta::self_model::SelfModel;

        let mut watchdog = SchemaWatchdog::new();
        let mut gaps = 0;
        for (type_name, fields) in &[
            ("KnowledgeNode", vec!["id", "title", "node_type", "content", "summary", "url", "domain", "language", "confidence", "importance", "access_count", "metadata", "created_at", "updated_at"]),
            ("NodeType", vec!["Concept", "Paper", "Repository", "Person", "Event", "Source", "Tool", "Framework", "Algorithm", "Theory", "Method", "Dataset", "Benchmark", "Organization", "Book", "Course", "Article", "CodeSnippet", "Idea", "Question", "Insight", "HarnessProfile", "Image", "EvolutionPattern", "ConversationEvolution", "Textbook", "Resource", "External", "Summary", "Guide", "Skill", "Reference", "WikiPage"]),
        ] {
            let f: Vec<String> = fields.iter().map(|s| s.to_string()).collect();
            if watchdog.detect_drift(type_name, &f).is_some() {
                gaps += 1;
            }
        }
        if gaps > 0 {
            log::warn!("[bg] arch_audit: {} schema gaps detected", gaps);
        } else {
            log::info!("[bg] arch_audit: clean — all schemas match");
        }
        let report = converge_check(".");
        if !report.findings.is_empty() {
            log::warn!("[bg] converge_check: {} ghosts, {} orphans, {} stale",
                report.ghost_count, report.stale_count, report.orphan_count);
        }

        // ── Inline self-test: types WITHOUT persistent fields use fresh instances (acceptable) ──
        let model = SelfModel::new();
        let scanner = CodeScanner::new(".");
        let entropy = EntropyMonitor::new(10, 0.5, 3);
        let meta_auditor = MetaAuditor::new();
        let arch_lint = ArchLint::new();
        let meta_monitor = MetaMonitor::new(model.clone());
        let meta_cog_loop = MetaCognitiveLoop::new(model);

        let mut self_tests = SelfTestRegistry::new();
        self_tests.register(Box::new(crate::core::nt_core_self_test::ExternalVerifier));
        self_tests.register(Box::new(watchdog));
        self_tests.register(Box::new(ConvergeCheckFn));
        self_tests.register(Box::new(scanner));
        self_tests.register(Box::new(entropy));
        self_tests.register(Box::new(InnerCritic::new()));
        self_tests.register(Box::new(SelfReviewGate::new(false)));
        self_tests.register(Box::new(meta_auditor));
        self_tests.register(Box::new(arch_lint));
        self_tests.register(Box::new(meta_monitor));
        self_tests.register(Box::new(meta_cog_loop));
        self_tests.register(Box::new(crate::neotrix::l8_autonomic_impl::nt_mind_self_diagnose::SelfDiagnose));
        self_tests.register(Box::new(crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default()));
        self_tests.register(Box::new(crate::core::l7_capability::nt_core_antidistil::DistillationDetector::new()));
        self_tests.register(Box::new(crate::neotrix::l1_body_impl::nt_act_autonomy::oracle_gate::OracleGate::new()));
        self_tests.register(Box::new(crate::neotrix::l1_body_impl::nt_act_code::semantic_entropy::SemanticEntropyGate::new()));
        self_tests.register(Box::new(crate::neotrix::l1_body_impl::nt_act_sandbox::ActionSandbox::new()));
        self_tests.register(Box::new(crate::core::nt_core_consciousness_review::ConsciousnessReview::new()));
        self_tests.register(Box::new(crate::neotrix::l8_autonomic_impl::nt_mind::consciousness_bridge::ConsciousnessBridge::new()));
        self_tests.register(Box::new(crate::neotrix::l1_body_impl::nt_shield::browser_security::BrowserSecurityScanner::new(
            crate::neotrix::l1_body_impl::nt_shield::browser_security::BrowserSecurityConfig::default(),
        )));
        self_tests.register(Box::new(crate::neotrix::l1_body_impl::nt_shield::check_registry::CheckRegistry::new()));
        self_tests.register(Box::new(crate::core::nt_core_telemetry::TelemetryStore::new(100)));
        // ── 派单控制面 SelfTest (P5, T3 inline) — 周期验证 P0-P4 共进化闭环:
        // 真实多轮派单 → learner 路由迁移 + MANTA 拓扑修复 + MAGE 四子图共进化 +
        // 跨轮持久化恢复。控制面从"仪式"变"可验证的自进化系统"。
        self_tests.register(Box::new(
            crate::neotrix::nt_mind::DispatchControlPlaneSelfTest::default(),
        ));

        // NOTE (Cycle 159b): NeoCodexSelfAudit::new() is intentionally NOT
        // registered here. Its Default snapshot reports provider-not-resolvable +
        // catalog-empty (3 permanent failures) → failure_count > 0 on every
        // architecture audit → spurious self-review goals enqueued. The audit is
        // a TUI-side live snapshot (NeoCodexSelfAudit::capture via EvolutionLoop),
        // not a BackgroundLoop detector. R-P26: SelfTest must be wired where its
        // data source exists.

        // ── Absorbed module SelfTests (Cycle 113) ──
        crate::core::nt_core_self_test_integration::register_absorbed_modules(&mut self_tests);

        // ── Substrate + Engine SelfTests (Cycle 119 architecture refactor) ──
        self_tests.register(Box::new(crate::core::nt_core_scoring_substrate::ScoringSubstrate::new().with_threshold(0.5)));
        self_tests.register(Box::new(crate::core::nt_core_state_substrate::StateSubstrate::new()));
        self_tests.register(Box::new(crate::core::nt_core_simulate_engine::SimulateEngine::new()));
        // ── ConvergencePulse SelfTest (Cycle 159c: fractal loop state machine) ──
        self_tests.register(Box::new(ConvergencePulse::default()));
        // ── ToolGroundingMonitor SelfTest — persistent instance (R-P49~R-P53) ──
        self_tests.register(Box::new(self.tool_grounding.clone()));

        if let Some(ref sb) = self.second_brain {
            match sb.self_test() {
                Ok(()) => log::info!("[SELF-TEST] SecondBrain ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] SecondBrain ❌ FAIL: {}", failures.join("; ")),
            }
        }

        // ── Inline self-test: types WITH persistent fields — clone or call self_test() directly ──
        // KnowledgeGapDetector
        if let Some(ref gap_detector) = self.gap_detector {
            match gap_detector.self_test() {
                Ok(()) => log::info!("[SELF-TEST] KnowledgeGapDetector ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] KnowledgeGapDetector ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::core::nt_core_meta::knowledge_gap_detector::KnowledgeGapDetector::new()));
        }

        // BMonitor (direct field, not Option)
        match self.bbrain.self_test() {
            Ok(()) => log::info!("[SELF-TEST] BMonitor ✅ pass"),
            Err(failures) => log::warn!("[SELF-TEST] BMonitor ❌ FAIL: {}", failures.join("; ")),
        }

        // CognitiveEvaluator (direct field)
        match self.cog_eval.self_test() {
            Ok(()) => log::info!("[SELF-TEST] CognitiveEvaluator ✅ pass"),
            Err(failures) => log::warn!("[SELF-TEST] CognitiveEvaluator ❌ FAIL: {}", failures.join("; ")),
        }

        // ConsciousnessTree
        if let Some(ref tree) = self.consciousness_tree {
            match tree.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessTree ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] ConsciousnessTree ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::core::nt_core_consciousness_tree::ConsciousnessTree::new()));
        }

        // ConsciousnessRuntime
        if let Some(ref cr) = self.consciousness_runtime {
            match cr.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessRuntime ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] ConsciousnessRuntime ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::core::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime::new()));
        }

        // ConsciousnessMonitor (awareness)
        if let Some(ref monitor) = self.awareness {
            match monitor.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessMonitor ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] ConsciousnessMonitor ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            let mut cm = crate::neotrix::nt_mind_consciousness_monitor::ConsciousnessMonitor::new();
            cm.observe();
            self_tests.register(Box::new(cm));
        }

        // FEPIITBridge
        if let Some(ref bridge) = self.fep_iit_bridge {
            match bridge.self_test() {
                Ok(()) => log::info!("[SELF-TEST] FEPIITBridge ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] FEPIITBridge ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::neotrix::l5_consciousness_impl::nt_core_fep_iit::bridge::FEPIITBridge::new()));
        }

        // ConsciousnessGoldStandard
        if let Some(ref gs) = self.gold_standard {
            match gs.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessGoldStandard ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] ConsciousnessGoldStandard ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::neotrix::l9_transcendent_impl::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard::new()));
        }

        // CognitiveLoadMonitor (existing clone pattern)
        if let Some(ref clm) = self.cognitive_load {
            match clm.self_test() {
                Ok(()) => log::info!("[SELF-TEST] CognitiveLoadMonitor ✅ pass"),
                Err(failures) => log::warn!("[SELF-TEST] CognitiveLoadMonitor ❌ FAIL: {}", failures.join("; ")),
            }
        } else {
            self_tests.register(Box::new(crate::core::nt_core_consciousness::CognitiveLoadMonitor::new()));
        }

        // ── L1 Shield + IO SelfTest registrations ──
        // (disabled: these types don't implement SelfTest yet)
        self_tests.register(Box::new(crate::neotrix::nt_memory_kb::nt_memory_commit_tracker::NarrativeConsistencyChecker::new()));

        // ── Run registry self-tests for remaining modules ──
        let results = self_tests.run_all();
        let mut failure_count = 0;
        for r in &results {
            if r.passed {
                log::info!("{}", r.summary());
            } else {
                log::warn!("{}", r.summary());
                failure_count += 1;
            }
        }
        
        // Pass SelfTest results to ConsciousnessTree for real branch health
        if let Some(ref mut tree) = self.consciousness_tree {
            tree.set_branch_health_from_self_tests(&results);
            log::debug!("[bg] consciousness_tree: branch health updated from {} SelfTest results", results.len());
        }

        // Cycle 206 R-P79 闭环: 从 KB absorbed_capability 元数据同步到能力网分支
        if let Some(kb) = self.kb.clone() {
            if let Some(ref mut tree) = self.consciousness_tree {
                match kb.absorbed_capabilities() {
                    Ok(pairs) if !pairs.is_empty() => {
                        let refs: Vec<(&str, &str)> = pairs
                            .iter()
                            .map(|(b, c)| (b.as_str(), c.as_str()))
                            .collect();
                        let synced = tree.sync_absorbed_capabilities_from_kb(&refs);
                        log::debug!(
                            "[bg] consciousness_tree: synced {} absorbed capabilities from KB ({} total)",
                            synced,
                            pairs.len()
                        );
                    }
                    Ok(_) => log::debug!("[bg] no absorbed_capability metadata in KB"),
                    Err(e) => log::warn!("[bg] absorbed_capabilities failed: {}", e),
                }
            }
        }
        
        if !report.findings.is_empty() || failure_count > 0 {
            let reason = format!(
                "arch_audit: {} converge issues + {} self-test failures — enqueueing self-review",
                report.findings.len(), failure_count,
            );
            log::warn!("[bg] {}", reason);
            if let Ok(mut brain) = self.brain.try_write() {
                self.goal_loop.enqueue_goal(&mut brain, &reason, None);
            }
        }
    }

}
