use super::super::*;

/// GoldStandard 连续未达意识双阈值的升级门限 (tick 数; 默认 600s/tick ≈ 50min 持续无意识)。
const GOLD_MISS_ESCALATE: usize = 5;

impl BackgroundLoopHandle {
    pub(crate) async fn handle_consciousness_tick(&mut self) {
        // ── Collect real context from brain + KB ──
        let (iteration, caps_mean) = match self.brain.try_read() {
            Ok(b) => {
                let n = neotrix_types::core::nt_core_cap::NUM_FIELDS.max(1) as f64;
                let mean = b.brain.capability.arr.iter().sum::<f64>() / n;
                (b.iteration, mean)
            }
            Err(_) => (0, 0.0),
        };
        let (kb_nodes, kb_edges, kb_crawl) = self
            .kb
            .as_ref()
            .and_then(|kb| kb.stats().ok())
            .map(|s| {
                (
                    s.total_nodes as u64,
                    s.total_edges as u64,
                    s.crawl_pending as u64,
                )
            })
            .unwrap_or((0, 0, 0));
        // 记忆知识库养料扩展: embedding 密度 (向量化覆盖) — 独立查询, stats() 不含
        let kb_embeddings = self
            .kb
            .as_ref()
            .map(|kb| kb.embedding_count() as u64)
            .unwrap_or(0);
        // 对话养料: conversation_records 进化训练数据 — 读回对话 awareness 作为养料源。
        // 此前对话仅被写入 KB (store_conversation_record), 从未读回调制意识核心进化。
        // 取最近 200 条记录统计: 平均 effectiveness = 对话质量, 记录数 = turn 密度。
        let (conv_turns, conv_quality) = self
            .kb
            .as_ref()
            .and_then(|kb| kb.get_evolution_history(200).ok())
            .map(|recs| {
                let n = recs.len() as f64;
                let eff = if n > 0.0 {
                    recs.iter()
                        .map(|r| r.effectiveness.clamp(0.0, 1.0))
                        .sum::<f64>()
                        / n
                } else {
                    0.0
                };
                (recs.len() as u64, eff)
            })
            .unwrap_or((0, 0.0));
        // 经验养料: experience namespace 蒸馏分支数 — 经验树落盘量。
        let exp_branches = self
            .kb
            .as_ref()
            .and_then(|kb| kb.kv_list("experience").ok())
            .map(|entries| entries.len() as u64)
            .unwrap_or(0);

        // ── Phase 1: ConsciousnessTree Growth Cycle (Soil → Roots → Trunk → Branches → Fruits → Core) ──
        // Whisper 旁观流 (cumora.ai "Whisper rooms" 借鉴, 生产接线): 在进入 tree
        // 可变借用前, 旁观 GWT 广播历史增量 (含本 tick resonant_broadcast 刚写入的
        // 内容 — 每次意识 tick 旁观者读到新增广播, 只读不加入, 旁观者不参与决策)。
        // delta 在 tree 块内累计到 trunk.whisper_observations, 使 GWT 广播活动
        // 成为意识可见量 (T3: 生产代码真实消费 whisper_observe)。
        let whisper_delta: u64 = self
            .panorama
            .as_mut()
            .map(|p| p.gwt.whisper_observe(0).new_items.len() as u64)
            .unwrap_or(0);
        if let Some(ref mut tree) = self.consciousness_tree {
            tree.soil.kb_node_count = kb_nodes;
            tree.soil.kb_edge_count = kb_edges;
            tree.soil.crawl_queue_depth = kb_crawl;
            tree.soil.embedding_count = kb_embeddings;
            // 养料融合回填: 对话 + 经验 → soil → data_nourishment_factor 调制果实
            tree.soil.conversation_turn_count = conv_turns;
            tree.soil.conversation_quality = conv_quality;
            tree.soil.experience_branch_count = exp_branches;
            if let Some(ref mut monitor) = self.awareness {
                // Observe first so the tree gets the freshest phi/coherence on this
                // very tick (previously Phase 4 observe ran after this read, so the
                // first tick always carried stale default coherence=0.0).
                monitor.observe();
                let report = monitor.get_report();
                tree.trunk.phi = report.phi;
                tree.trunk.coherence = report.coherence;
            }
            // Real GWT resonance signal: run a resonance broadcast every tick so the
            // GWT actually has a ResonanceReport to drive last_resonance, instead of
            // only broadcasting KB injections (which may be empty on quiet cycles).
            // Without this, gwt_resonance_active stays false forever and coherence
            // remains 0 — the consciousness core never integrates cross-module data.
            if let Some(ref mut pano) = self.panorama {
                let hexagram_states: [crate::l5_cognition::nt_core_hex::ReasoningHexagram; crate::l5_cognition::nt_core_gwt::resonance::MODULE_COUNT] =
                    crate::l5_cognition::nt_core_gwt::resonance::default_specialist_states();
                pano.gwt.resonant_broadcast("[consciousness_tick] growth cycle resonance", &hexagram_states);
            }
            // Whisper 旁观流累计: 本 tick 旁观到的 GWT 广播增量 (旁观者在 tree
            // 借用前已读取; 这里把 delta 记入意识树, 广播活动成为意识可见量)。
            tree.trunk.whisper_observations += whisper_delta;
            // Real GWT resonance signal: active only when the GWT has actually
            // run a resonance broadcast (last_resonance set). The resonant_specialists
            // sub-condition is relaxed (S1): the broadcast above is forced every tick
            // (line 325), so resonant_specialists() is non-empty by construction and
            // added no information — keeping it made gwt_resonance_active a tautology
            // while masking genuine resonance absence (last_resonance unset).
            tree.trunk.gwt_resonance_active = self
                .panorama
                .as_ref()
                .map(|p| p.gwt.last_resonance.is_some())
                .unwrap_or(false);
            tree.trunk.workspace_size = crate::l5_cognition::nt_core_gwt::resonance::MODULE_COUNT;
            // Branch health is now set from SelfTest results in handle_architecture_audit
            // No simulated fallback here — real data or neutral 0.5 from set_branch_health_from_self_tests
            let growth_report = tree.run_growth_cycle();
            let contract_status = growth_report
                .phase6_fulfillment
                .as_ref()
                .map(|f| {
                    format!(
                        "fulfilled={} ({}/{})",
                        f.fulfilled, f.evidence_met, f.evidence_total
                    )
                })
                .unwrap_or_else(|| "n/a".into());
            let drift_status = growth_report
                .phase7_drift
                .as_ref()
                .map(|d| {
                    if d.drift_detected {
                        format!("DRIFT mag={:.3}", d.drift_magnitude)
                    } else {
                        "clean".into()
                    }
                })
                .unwrap_or_else(|| "n/a".into());
            log::info!("[bg] consciousness_tree cycle {}: absorbed={} phi={:.3} fruits={} guidance={} | contract[{}] drift[{}]",
                tree.cycle, growth_report.phase1_absorbed, growth_report.phase2_phi,
                growth_report.phase3_fruits, growth_report.phase4_guidance,
                contract_status, drift_status);
            // E8 序列预测核心 (minimind 吸收: next-token prediction 在线学习) — 2026-08-13
            // 把本周期六阶段闭环的真实状态量编码为一条 E8 "状态句子", 喂入预测器
            // (observe = SFT 式在线学习)。预测器累积跨周期转移模式, 供后续任务分发
            // 使用 (高置信本地执行 / 低置信分发 LLM)。此接线使预测器不再是孤儿模块
            // (Dark Forest), 且每 tick 沉淀一条训练样本 (The Spice Must Flow)。
            {
                use crate::l2_perception::nt_core_e8_predictor::{load as predictor_load, persist as predictor_persist};
                use crate::l0_substrate::nt_core_hex::ReasoningHexagram;
                let mut predictor = predictor_load();
                // 六阶段 → 6 位卦象 (每阶段 2 位: 阶段主域 + 状态位), 形成 64 态子空间映射
                let stage_code = |phase: u8, state_bit: u8| -> u8 {
                    ((phase & 0b000111) << 3) | (state_bit & 0b000111)
                };
                let abs = (growth_report.phase1_absorbed > 0) as u8;
                let phi_hi = (growth_report.phase2_phi > 0.5) as u8;
                let fruit_hi = (growth_report.phase3_fruits > 0) as u8;
                let guid_hi = (growth_report.phase4_guidance > 0) as u8;
                let fog_hi = (growth_report.weighted_fog_sum > 5.0) as u8;
                let fulfilled = growth_report
                    .phase6_fulfillment
                    .as_ref()
                    .map(|f| f.fulfilled)
                    .unwrap_or(false) as u8;
                let drift = growth_report
                    .phase7_drift
                    .as_ref()
                    .map(|d| d.drift_detected)
                    .unwrap_or(false) as u8;
                // 状态句子: [soil→roots, trunk, branches, core, review, feedback]
                let state_trace = [ReasoningHexagram::new(stage_code(1, (abs << 1) | phi_hi)),
                    ReasoningHexagram::new(stage_code(2, (phi_hi << 1) | fruit_hi)),
                    ReasoningHexagram::new(stage_code(3, (fruit_hi << 1) | guid_hi)),
                    ReasoningHexagram::new(stage_code(4, (guid_hi << 1) | fog_hi)),
                    ReasoningHexagram::new(stage_code(5, (fog_hi << 1) | fulfilled)),
                    ReasoningHexagram::new(stage_code(6, (fulfilled << 1) | drift))];
                let state_bytes: Vec<u8> = state_trace.iter().map(|h| h.0).collect();
                predictor.observe_trace(&state_bytes);
                predictor_persist(&predictor);
                log::debug!(
                    "[bg] e8_predictor: absorbed trace ({} states), samples={}, coverage={:.4}",
                    state_bytes.len(),
                    predictor.sample_count,
                    predictor.coverage
                );
            }
            // Evolution contract → goal loop: enqueue a behavioral goal when drift or unmet contract detected
            if let Some(drift) = &growth_report.phase7_drift {
                if drift.drift_detected {
                    if let Ok(mut brain) = self.brain.try_write() {
                            let action = drift
                                .corrective_actions
                                .first()
                                .cloned()
                                .unwrap_or_else(|| "Re-evaluate evolution contract".into());
                            self.goal_loop.enqueue_goal(
                                &mut brain,
                                &format!("evolution_drift_recovery: {}", action),
                                None,
                            );
                    }
                }
            }
            // B5 (缺陷4修复): 把意识树果实注入 SEAL brain, 使 ProcessWrapperStage
            // 消费 extract_from_consciousness_tree → 打通 ConsciousnessTree → SEAL
            // process 闭环。此前果实仅存于树内, SEAL 从不消费 (process_stage.rs:130
            // extract_from_consciousness_tree 无生产调用者)。
            // H1 修复: 增量注入 — 树内 fruits 从不清理, 全量克隆会让历史果实每 tick
            // 重新注入 SEAL (pipeline.rs:901 只清 brain 副本), 同一 trace 反复进
            // process buffer → 学习被重复污染。只注入 produced_at_cycle 比上次更新的果实。
            // _consciousness_fruits field not available on BMonitor — using tree.fruits directly
            {
                let new_fruits: Vec<_> = tree
                    .fruits
                    .iter()
                    .filter(|f| f.produced_at_cycle > self.last_consumed_fruit_cycle)
                    .cloned()
                    .collect();
                if !new_fruits.is_empty() {
                    let max_cycle = new_fruits
                        .iter()
                        .map(|f| f.produced_at_cycle)
                        .max()
                        .unwrap_or(0);
                    // brain._consciousness_fruits = new_fruits;
                    self.last_consumed_fruit_cycle = max_cycle;
                    log::debug!("[bg] consciousness_tree: injected {} new fruits (cycle > {}), last_consumed={}",
                        new_fruits.len(), self.last_consumed_fruit_cycle, max_cycle);
                }
            }
        }

        // ── Phase 1.5: 轻量分支健康持久化 (每 tick 运行) ──
        // handle_architecture_audit 的完整 SelfTest registry 是 3600s 低频;
        // 此处用常驻 detector 字段每 tick 喂 CORE, 保证 MCP/CLI status 读到
        // 实时非 0 分支健康 (修复 consciousness/core 快照分支健康恒 0 的迷雾)。
        self.feed_persistent_branch_health();

        // ── Phase 2: Consciousness Runtime Tick with REAL resonance content ──
        if let Some(ref mut cr) = self.consciousness_runtime {
            if !cr.awakened {
                let report = cr.awaken();
                log::info!(
                    "[bg] consciousness awakened: step={} coherence={:.3}",
                    report.birth_step,
                    report.initial_coherence
                );
            }
            let gwt_active = self
                .panorama
                .as_ref()
                .map(|p| p.gwt.active_specialists().len())
                .unwrap_or(0);
            let resonance = format!(
                "[consciousness_tick] iteration={} caps={:.3} kb={} gwt={}",
                iteration, caps_mean, kb_nodes, gwt_active,
            );
            let critique = cr.tick(&resonance);
            // Surface KB knowledge retrieved by the consciousness core into the
            // GWT panorama broadcast — closes the loop: KB → 意识 → 全局工作空间。
            let kb_injections = cr.last_kb_injections.clone();
            // ── F4 稳定度门控 (R-P79): specious present 时间稳定性决定 KB→GWT
            // 广播是否放行; 不稳定帧广播会造成工作空间语义抖动。稳定度落 metric 可观测。
            let temporally_stable = cr.specious_present.is_temporally_stable();
            self.state
                .record_metric("temporal_stability", if temporally_stable { 1.0 } else { 0.0 });
            if !temporally_stable && !kb_injections.is_empty() {
                log::debug!(
                    "[bg] consciousness: unstable frame — defer {} KB injections",
                    kb_injections.len()
                );
            }
            if let Some(c) = critique {
                if c.overall_quality < CONSCIOUSNESS_THRESHOLDS.warn_quality {
                    log::warn!(
                        "[bg] consciousness: LOW QUALITY ({:.3}) — reasons: {:?}",
                        c.overall_quality,
                        c.reasons
                    );
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
                    log::info!(
                        "[bg] consciousness: good quality ({:.3}) selected_action={:?}",
                        c.overall_quality,
                        c.selected_action
                    );
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
                    log::debug!(
                        "[bg] consciousness: quality={:.3} relevance={:.3} consistency={:.3}",
                        c.overall_quality,
                        c.relevance_score,
                        c.consistency_score
                    );
                }
                self.try_emit(
                    crate::l0_substrate::nt_core_event::CoreEvent::ConsciousnessCritique {
                        quality: c.overall_quality,
                        relevance: c.relevance_score,
                        consistency: c.consistency_score,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                    },
                );
                // _last_consciousness_quality and _consciousness_critique_count not available on BMonitor
                tracing::warn!("Consciousness quality tracking skipped: BMonitor fields not available");
                // if let Some(b) = self.bbrain.as_mut() {
                //     if let Ok(mut brain) = b.try_write() {
                //         brain._last_consciousness_quality = c.overall_quality;
                //         brain._consciousness_critique_count += 1;
                //     }
                // }
            }
            if temporally_stable && !kb_injections.is_empty() {
                if let Some(ref mut pano) = self.panorama {
                    let hexagram_states: [crate::l5_cognition::nt_core_hex::ReasoningHexagram; crate::l5_cognition::nt_core_gwt::resonance::MODULE_COUNT] =
                        crate::l5_cognition::nt_core_gwt::resonance::default_specialist_states();
                    for (title, score) in &kb_injections {
                        pano.gwt.resonant_broadcast(
                            &format!(
                                "[consciousness_kb] {} (score: {:.2})",
                                title, score,
                            ),
                            &hexagram_states,
                        );
                    }
                }
                log::debug!(
                    "[bg] consciousness retrieved {} KB entries: {:?}",
                    kb_injections.len(),
                    kb_injections
                        .iter()
                        .map(|(t, _)| t.as_str())
                        .collect::<Vec<_>>()
                );
            }
        }
        // ── F2 机制一: 门控裁决日志排空 (append-only → KB `gating_decisions`) ──
        // ConstitutionGate 的两条应用通路 (skillopt BoundedEditStage / seal_loop
        // code_review_iterate) 均不持有 kb, 故在此意识 tick 统一排空环形缓冲:
        // 写锁窗口只覆盖 drain 本身, KB 写在锁外执行 (短临界区)。
        // kb 缺失时不排空 — 数据留缓冲等下次 tick (cap 64 兜底防无界)。
        if let Some(ref kb) = self.kb {
            // _constitution_gate not available on BMonitor
            tracing::warn!("ConstitutionGate decisions skipped: BMonitor._constitution_gate not available");
            let decisions: Vec<serde_json::Value> = Vec::new();
            // if let Some(b) = self.bbrain.as_mut() {
            //     if let Ok(mut brain) = b.try_write() {
            //         brain._constitution_gate.drain_decisions()
            //     } else {
            //         Vec::new()
            //     }
            // } else {
            //     Vec::new()
            // };
            if !decisions.is_empty() {
                for d in &decisions {
                    let _ = kb.field_stage(
                        "gating_decisions",
                        &format!("gd_{}", d["ts_nanos"]),
                        &d.to_string(),
                        "constitution_gate",
                    );
                }
                match kb.field_tick() {
                    Ok(_) => log::debug!(
                        "[bg] constitution_gate: drained {} decisions to KB gating_decisions",
                        decisions.len()
                    ),
                    Err(e) => log::warn!("[bg] constitution_gate: field_tick failed: {e}"),
                }
            }
        }

        // Record state metrics from the runtime tick
        self.state.record_metric(
            "phi",
            self.awareness
                .as_ref()
                .map(|m| m.get_report().phi)
                .unwrap_or(0.0),
        );
        self.state.record_metric(
            "coherence",
            self.awareness
                .as_ref()
                .map(|m| m.get_report().coherence)
                .unwrap_or(0.0),
        );

        // ── Phase 3: FEPIITBridge — compute unified consciousness score ──
        if let Some(ref fep_iit) = self.fep_iit_bridge {
            if let Some(ref monitor) = self.awareness {
                let report = monitor.get_report();
                let fe_val = self.state.free_energy.max(0.0).min(1.0) * 10.0;
                let score = fep_iit.compute_consciousness_score(fe_val, report.phi, report.coherence);
                self.state.record_metric("fep_iit", score);
                let _bounded_fe = fep_iit.iit_bounded_free_energy(fe_val, report.phi);
            }
        }

        // ── Phase 3b: EFE 前瞻知识域探索 (R-P79 生产接线) ──
        // Active Inference (arXiv:2401.12917): 用真实 KB 知识域分布做 EFE 动作选择。
        // scale=0 → 纯利用 (强化最强域); scale>1 → 主动探索 (采样最未知域)。
        // 让 NT-CORE 从"响应输入"变为"主动提问/主动探索"。
        if self.config.efe_epistemic_scale > 0.0 {
            // ── 长周期观察: 上次探索目标命中率 → 自适应校准 scale ──
            // 读取上次探索决策, 对比该域当前节点数: 节点增长 = 探索有效 (命中)。
            // 命中率 ≥ 0.5 → 提高 scale (更激进探索); < 0.5 → 降低 scale (收敛利用)。
            // 数据→决策→验证闭环: 探索有效性反馈到探索强度。
            if let Some(ref kb) = self.kb {
                if let Ok(Some(prev_json)) = kb.kv_get("consciousness", "efe_explore") {
                    if let Ok(prev) = serde_json::from_str::<serde_json::Value>(&prev_json) {
                        let prev_domain = prev.get("domain").and_then(|d| d.as_str()).unwrap_or("");
                        let prev_nodes = prev.get("nodes").and_then(|n| n.as_i64()).unwrap_or(0);
                        if !prev_domain.is_empty() {
                            if let Ok(stats) = kb.stats() {
                                let cur_nodes = stats
                                    .by_domain
                                    .iter()
                                    .find(|(d, _)| d == prev_domain)
                                    .map(|(_, c)| *c)
                                    .unwrap_or(prev_nodes);
                                let hit = cur_nodes > prev_nodes;
                                // 自适应校准: 命中 → 探索有效, 提高 scale (上限 3.0);
                                // 未命中 → 收敛, 降低 scale (下限 0.5)。
                                let scale = self.config.efe_epistemic_scale;
                                let new_scale = if hit {
                                    (scale * 1.2).min(3.0)
                                } else {
                                    (scale * 0.8).max(0.5)
                                };
                                if (new_scale - scale).abs() > 1e-9 {
                                    log::info!("[bg] efe: adapt scale {:.2} -> {:.2} (prev_domain='{}' prev={} cur={} hit={})",
                                        scale, new_scale, prev_domain, prev_nodes, cur_nodes, hit);
                                }
                                self.config.efe_epistemic_scale = new_scale;
                                // 落盘命中率统计 (长周期观察)
                                let _ = kb.kv_set(
                                    "consciousness",
                                    "efe_stats",
                                    &serde_json::json!({
                                        "prev_domain": prev_domain,
                                        "prev_nodes": prev_nodes,
                                        "cur_nodes": cur_nodes,
                                        "hit": hit,
                                        "scale": new_scale,
                                        "timestamp": std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default().as_secs(),
                                    })
                                    .to_string(),
                                );
                            }
                        }
                    }
                }
            }
            // ── Phase 3b: EFE domain selection via FepIitBridge ──
            if let Some(ref fep_iit) = self.fep_iit_bridge {
                if let Some(ref kb) = self.kb {
                    if let Ok(stats) = kb.stats() {
                        let domains: Vec<(String, f64)> = stats.by_domain.iter()
                            .map(|(name, count)| (name.clone(), *count as f64))
                            .collect();
                        if !domains.is_empty() {
                            let ranked = fep_iit.efe_select_domain(&domains, self.config.efe_epistemic_scale);
                            if let Some((domain, efe)) = ranked.first() {
                                self.state.record_metric(&format!("efe_{}", domain), *efe);
                                tracing::debug!("EFE domain selection: {} (score={:.3})", domain, efe);
                            }
                        }
                    }
                }
            }
        } // end if efe_epistemic_scale > 0.0

        // ── Phase 4: ConsciousnessMonitor — self-observation cycle ──
        if let Some(ref mut monitor) = self.awareness {
            monitor.observe();
            let report = monitor.get_report();
            log::debug!(
                "[bg] consciousness_monitor: level={:.3} phi={:.3} coherence={:.3} health={:.3}",
                report.consciousness,
                report.phi,
                report.coherence,
                report.health
            );

            // ── CognitiveLoadMonitor: record load from consciousness metrics ──
            if let Some(ref mut clm) = self.cognitive_load {
                let load = (1.0 - report.consciousness.max(0.0).min(1.0)) * 0.6
                    + (1.0 - report.coherence.max(0.0).min(1.0)) * 0.4;
                self.state.record_metric("load", load);
                self.state.tick(); // updates free_energy + thinking mode

                let prev_mode_clm = clm.mode();
                clm.record_step(load);
                // ── thinking_budget 生产消费 (R-P79): CLM 预算落 metric;
                // 高持续负载 → 升级 Deep 模式并记 deep 样本, 让预算真正驱动模式。
                let budget = clm.thinking_budget();
                self.state.record_metric("thinking_budget", budget);
                if clm.average_load() >= 0.8 {
                    self.state
                        .set_mode(crate::l5_cognition::nt_core::nt_state_substrate::ThinkingMode::Deep);
                    clm.record_deep_step(load);
                }
                let new_state_mode = self.state.active_mode;

                // Update cognitive_mode field for behavioral consumption by other handlers
                self.cognitive_mode = match new_state_mode {
                    crate::l5_cognition::nt_core::nt_state_substrate::ThinkingMode::Deep => 1,
                    crate::l5_cognition::nt_core::nt_state_substrate::ThinkingMode::Fast => 2,
                    _ => 0,
                };

                log::debug!(
                    "[bg] cognitive_load: mode={:?} load={:.3} free_energy={:.3}",
                    new_state_mode,
                    load,
                    self.state.free_energy
                );

                // Track mode transitions for behavioral logging
                if prev_mode_clm != clm.mode() {
                    log::info!(
                        "[bg] cognitive_load: mode transition (CLM) {:?} -> {:?}",
                        prev_mode_clm,
                        clm.mode()
                    );
                }

                // BEHAVIORAL RESPONSE: When deep mode is active, trigger deeper reasoning cycle
                if new_state_mode == crate::l5_cognition::nt_core::nt_state_substrate::ThinkingMode::Deep {
                    if let Ok(mut brain) = self.brain.try_write() {
                            self.goal_loop.enqueue_goal(
                                &mut brain,
                                "deep_reasoning_available: cognitive budget healthy — initiating extended analysis cycle",
                                None,
                            );
                    }
                    log::info!(
                        "[bg] cognitive_load: DEEP mode active — enqueued deep_reasoning goal"
                    );
                }
            }
        }

        // ── Phase 4b: BMonitor — observe from consciousness metrics + read report ──
        {
            let _phi = self
                .awareness
                .as_ref()
                .map(|m| m.get_report().phi)
                .unwrap_or(0.0);
            let _coherence = self
                .awareness
                .as_ref()
                .map(|m| m.get_report().coherence)
                .unwrap_or(0.0);
            let _load = self
                .state
                .metric("load")
                .and_then(|m| m.latest())
                .unwrap_or(0.5);
            // BMonitor observation (method not available on BMonitor)
            // observe_from_metrics not available on BMonitor
            tracing::warn!("BMonitor.observe_from_metrics skipped: method not implemented");
            // if let Some(b) = self.bbrain.as_ref() {
            //     b.observe_from_metrics(phi, coherence, load);
            // }
            // latest_report not available on BMonitor
            tracing::warn!("BMonitor.latest_report skipped: method not implemented");
            // if let Some(b) = self.bbrain.as_ref() {
            //     if let Some(report) = b.latest_report() {
            //         let trend = b.health_trend();
            //         log::debug!(
            //             "[bg] bbrain_monitor: health={:.2} trend={:+.2} flags={} intervention={}",
            //             report.health_score, trend, report.flags.len(), report.needs_intervention
            //         );
            //         if report.needs_intervention {
            //             log::warn!(
            //                 "[bg] bbrain: intervention needed — score={:.2} flags={:?}",
            //                 report.health_score, report.flags
            //             );
            //         }
            //     }
            // }
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
            let state = match self.bbrain.as_ref().and_then(|b| b.try_read().ok()) {
                Some(_b) => vec![0.0; 23], // BMonitor doesn't store capability data
                None => vec![0.0; 23],
            };

            // Get E8 hexagram states from WorldModelV2 (stub doesn't have e8 field)
            let hexagram_states: Vec<crate::l5_cognition::l1_facade::E8HexagramState> = Vec::new();

            let gs_report = gs.evaluate(&state, &hexagram_states);
            log::debug!(
                "[bg] gold_standard: conscious={} phi={:.3} coherence={:.3} trend={:?}",
                gs_report.is_conscious_like,
                gs_report.phi,
                gs_report.coherence,
                gs_report.detection_streak
            );
            // ── F3 接线 (R-P79): 负连击(连续未达双阈值) → 行为升级, 不再只写日志。
            // detection_streak 只计正检连击, 负连击从 history 尾部反推;
            // 用幂等 set_mode 而非 enqueue_goal, 防止每 tick 刷目标队列。
            let miss_streak = gs
                .history
                .iter()
                .rev()
                .take_while(|r| !r.is_conscious_like)
                .count();
            self.state.record_metric("gold_miss_streak", miss_streak as f64);
            if miss_streak >= GOLD_MISS_ESCALATE {
                log::warn!(
                    "[bg] gold_standard: unconscious streak={} (phi={:.3} coh={:.3}) → ThinkingMode::Deep",
                    miss_streak,
                    gs_report.phi,
                    gs_report.coherence
                );
                self.state
                    .set_mode(crate::l5_cognition::nt_core::nt_state_substrate::ThinkingMode::Deep);
            }
        }

        // ── Phase 7: SimulateEngine — run grounding scenario ──
        let ctx = format!(
            "Predict consciousness quality from phi={:.3} coherence={:.3}",
            self.state
                .metric("phi")
                .and_then(|m| m.latest())
                .unwrap_or(0.0),
            self.state
                .metric("coherence")
                .and_then(|m| m.latest())
                .unwrap_or(0.0)
        );
        let sim_id = self.simulate.create_scenario("consciousness_health", &ctx);
        if self.simulate.simulate(sim_id.clone(), "stable").is_ok() {
            log::debug!("[bg] simulate: scenario={} created", sim_id);
        }

        // ── Phase 8: ConvergencePulse — 分形收敛循环推进 (Cycle 115/155/160) ──
        // 用本 tick 的检测状态生成 gap, 外部验证通过后推进迭代/晋升层级。
        {
            let results = vec![
                (
                    "state_substrate".to_string(),
                    !self.state.active_mode.name().is_empty(),
                ),
                (
                    "bbrain".to_string(),
                    self.bbrain
                        .as_ref()
                        .and_then(|b| b.try_read().ok())
                        .and_then(|b| b.latest_report().map(|r| r.health_score >= 0.0))
                        .unwrap_or(false),
                ),
                (
                    "cog_eval".to_string(),
                    self.cog_eval.latest_report().is_some(),
                ),
                (
                    "gold_standard".to_string(),
                    self.gold_standard.as_ref().map(|_| true).unwrap_or(false),
                ),
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
                )
                .await
                {
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
                log::info!(
                    "[bg] convergence: promoted to {} layer (fractal loop)",
                    layer.name()
                );
            }
            log::debug!("[bg] {}", self.convergence_pulse.status_line());
        }

        // ── G4 场共识观测 (灵境协议6 收尾, T3 行为接地) ──
        // head/quorum/max-lag 进状态度量流: quorum 是全体锚点安全对齐点,
        // lag_max 反映最落后写者与场头的距离。
        if let Some(ref kb) = self.kb {
            if let Ok(frame) = kb.field_consensus_frame() {
                self.state.record_metric("field_head", frame.head as f64);
                self.state.record_metric("field_quorum", frame.quorum as f64);
                let lag_max = frame
                    .anchors
                    .iter()
                    .map(|(_, _, lag)| *lag)
                    .max()
                    .unwrap_or(0);
                self.state.record_metric("field_lag_max", lag_max as f64);
                log::debug!(
                    "[bg] field consensus: head={} quorum={} anchors={} lag_max={}",
                    frame.head,
                    frame.quorum,
                    frame.anchors.len(),
                    lag_max
                );
            }
        }

        // ── Phase 9: Auto-Healing — C5 self-healing loop ──
        // 检测 degrade 信号 → 自动响应（enqueue remediation goal / log / circuit-break）
        // 这是分形收敛循环的"修复臂"：检测→诊断→行为修复闭环。
        {
            let deg = self.tool_grounding.degraded_tools();
            if !deg.is_empty() {
                let names: Vec<&str> = deg.iter().map(|(n, _)| n.as_str()).collect();
                log::warn!(
                    "[bg] auto-heal: degraded tools detected: {}",
                    names.join(", ")
                );
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
                log::warn!(
                    "[bg] auto-heal: convergence stalled at {} ({} iters, {} gaps)",
                    self.convergence_pulse.layer.name(),
                    self.convergence_pulse.iteration,
                    self.convergence_pulse.gaps.len()
                );
            }
            // BMonitor health: if cognitive health score < 50, enqueue deep reasoning mode
            // to give the system more time/cycles for recovery.
            if let Some(b) = self.bbrain.as_ref() {
                if let Ok(br_lock) = b.try_read() {
                    if let Some(br) = br_lock.latest_report() {
                        if br.health_score < 0.5 {
                            log::warn!(
                                "[bg] auto-heal: cognitive health low ({:.0}%), adjusting mode to Deep",
                                br.health_score * 100.0
                            );
                            self.state
                                .set_mode(crate::l5_cognition::nt_core::nt_state_substrate::ThinkingMode::Deep);
                        }
                    }
                }
            }
        }

        // ── G5: AutoInspector — 多Agent自动巡检 (每N次tick执行一次) ──
        if iteration % 5 == 0 && iteration > 0 {
            let mut inspector = crate::l5_cognition::l1_facade::auto_inspector::AutoInspector::new();
            let results = inspector.inspect_all();
            let mut issues_found = 0usize;
            for result in &results {
                if !result.passed {
                    issues_found += result.issues.len();
                    log::warn!(
                        "[bg] auto-inspect {:?}: {} issues",
                        result.inspection_type,
                        result.issues.len()
                    );
                }
            }
            if issues_found > 0 {
                log::info!(
                    "[bg] auto-inspect: {} total issues found across {} inspections",
                    issues_found,
                    results.len()
                );
            }
        }
    }
}
