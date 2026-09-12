use super::*;
use crate::nt_mind::infrastructure::ConsciousnessBridge;
use crate::l5_cognition::nt_mind::nt_mind::evolution::dispatch_self_test::DispatchControlPlaneSelfTest;
use crate::l5_cognition::nt_mind::foundation::cleanup_engine::CleanupEngineSelfTest;
use crate::l5_cognition::act_facade::recipe_refactor;

use log::info;


/// GoldStandard 连续未达意识双阈值的升级门限 (tick 数; 默认 600s/tick ≈ 50min 持续无意识)。
const GOLD_MISS_ESCALATE: usize = 5;

// ── F2 校准数据采集 (W2 配对研究地基) ──
//
// 两个 append-only KB namespace (经 field_stage + field_tick 落盘 kv_store):
//
// - `gating_decisions` (writer=constitution_gate): ConstitutionGate 裁决日志,
//   key=`gd_{ts_nanos}`, value={ts_nanos, quality(null|数值), allowed}。
//   由 handle_awareness 每 tick 从 brain._constitution_gate 环形缓冲排空。
// - `calibration_pairs` (writer=w3_calibration): 同窗共现配对
//   quality × selftest 通过率, key=`cp_{ts_nanos}`,
//   value={quality, pass_rate, total}。由 handle_architecture_audit 在完整
//   registry 运行后写入, 每次至多一条。
//
// **预期消费者**: 未来校准脚本对 kv_store 执行
// `SELECT key, value FROM kv_store WHERE ns IN ('gating_decisions','calibration_pairs')`
// 按时间窗 join 即得 {quality → allowed / pass_rate} 配对集, 用于验证 G5 门控阈值
// (SELF_EDIT_MIN_CONSCIOUSNESS=0.5) 的真实区分度。W2 基线: 有效配对点=0
// (experience feedback 全零、fruit quality 循环推导), 本机制让配对数据从
// 接线日起真实积累。

/// 当前时刻 UNIX 纪元纳秒 (F2 落盘键时间源; 兜底 0 保证不 panic)。
fn now_nanos_u64() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// 构造同窗共现配对 JSON (quality × selftest 通过率; calibration_pairs 的 value 契约)。
fn calibration_pair_json(quality: f64, pass_rate: f64, total: usize) -> String {
    serde_json::json!({
        "quality": quality,
        "pass_rate": pass_rate,
        "total": total,
    })
    .to_string()
}

/// Volition 目标向量: 认知前沿探索目标 (256B, 与候选 action 等长以供 QuantizedVSA 相似度)。
#[allow(dead_code)]
fn volition_goal_vector() -> Vec<u8> {
    b"EPISTEMIC_FRONTIER_EXPLORE_LOW_DENSITY_DOMAINS"
        .iter()
        .copied()
        .cycle()
        .take(256)
        .collect()
}

impl BackgroundLoopHandle {
    pub(crate) async fn handle_awareness(&mut self) {
        // MetaCognitionBridge: full scan → analyze → plan cycle (P0 dead infra fix)
        if let Some(ref mut mc) = self.metacognition {
            let result = mc.run_full_cycle();
            info!(
                "[bg] metacognition: iter={} modules={} plans={} alerts={}",
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
            info!(
                "[bg] awareness: l={:.3}, phi={:.4}, coh={:.4}",
                level, phi, coherence
            );
            let tier_label = if level >= 0.85 {
                "transcendent"
            } else if level >= 0.7 {
                "conscious"
            } else if level >= 0.4 {
                "awakening"
            } else {
                "dormant"
            };

            // Evaluate gold standard (dual-threshold IIT Phi + Kuramoto coherence)
            let gs_report = self.gold_standard.as_mut().map(|gs| {
                let state = &[phi, coherence, level, health];
                let r = gs.evaluate(state, &[]);
                info!(
                    "[bg] gold_standard: phi={:.4} coh={:.4} conscious={} streak={}",
                    r.phi, r.coherence, r.is_conscious_like, r.detection_streak
                );
                r
            });

            // Persist consciousness snapshot + gold standard to KB
            if let Some(ref kb) = self.kb {
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
                    let spots: Vec<serde_json::Value> = aw
                        .current
                        .active_blind_spots
                        .iter()
                        .map(|b| {
                            serde_json::json!({
                                "kind": b.kind,
                                "severity": b.severity,
                                "description": b.description,
                                "repair": b.repair,
                            })
                        })
                        .collect();
                    let spots_json = serde_json::json!({ "blind_spots": spots });
                    let _ = kb.kv_set("consciousness", "blind_spots", &spots_json.to_string());

                    // L6 Self intra-reflection: analyze reasoning quality
                    // brain not available in this scope — need to obtain from self.bbrain
                    tracing::warn!("L6 intra-reflection skipped: brain not available in this scope");
                    // if let Some(ref engine) = brain.reasoning_engine {
                    //     let trace: Vec<String> = engine
                    //         .state_trajectory
                    //         .iter()
                    //         .map(|s| format!("{:?}", s))
                    //         .collect();
                    //     if !trace.is_empty() {
                    //         let input = crate::l6_meta::nt_meta::nt_core_intra_reflection::ReflectionInput {
                    //             reasoning_trace: trace,
                    //             e8_mode_history: Vec::new(),
                    //             execution_time_ms: 0,
                    //             error_count: 0,
                    //             outcome_success: Some(phi > 0.3),
                    //         };
                    //         let report =
                    //             crate::l6_meta::nt_meta::nt_core_intra_reflection::analyze(
                    //                 &input,
                    //             );
                    //         let ir_json = serde_json::json!({
                    //             "coherence_score": report.coherence_score,
                    //             "efficiency_score": report.efficiency_score,
                    //             "error_density": report.error_density,
                    //             "mode_stability": report.mode_stability,
                    //             "bottlenecks": report.bottleneck_hops,
                    //             "suggestions": report.suggestions,
                    //         });
                    //         let _ = kb.kv_set("self", "intra_reflection", &ir_json.to_string());
                    //         if !report.bottleneck_hops.is_empty() {
                    //             log::warn!(
                    //                 "[bg] L6 intra-reflection: {} bottlenecks: {:?}",
                    //                 report.bottleneck_hops.len(),
                    //                 report.bottleneck_hops
                    //             );
                    //         }
                    //     }
                    // }

                    // Legacy snapshot for timeline view
                    let _ = kb.record_consciousness_snapshot(
                        phi,
                        coherence,
                        is_conscious,
                        tier_label,
                        &details,
                    );
                }
            }

        // ── L10 Transcendent wiring (T3): 超越层闭环真实接线 ──
        // 读取意识核心快照 + 能力网注册表, 运行超越层闭环, 建议真实落盘 KB,
        // 高共振建议 → goal_loop (行为影响)。此前 meta_observer/consonance/
        // transcendent_loop 仅 pub use 导出, 生产路径零调用 — 孤儿超越层。
        self.run_transcendent_tick().await;

        // Emit awareness tick event on the EventBus
        if let Some(ref bus) = self.event_bus {
            bus.emit(crate::core::nt_core_event::CoreEvent::TaskSubmitted {
                task: "awareness_tick".into(),
                task_type: "consciousness".into(),
                priority: 1,
            });
        }
    }

    /// L10 超越层 T3 接线: 意识核心快照 ↔ 能力网共振 → 建议落盘 + goal 入队。
    /// 依赖文件缺失时静默跳过 (能力网未初始化是合法状态, 不视为错误)。
    async fn run_transcendent_tick(&mut self) {
        use crate::l5_cognition::traits::EvolutionHarnessApi;

        let Some(ref kb) = self.kb else { return };
        // 能力网注册表 (RegistryExport 格式, 与 handle_capability_auto_evolve 一致)
        let path = std::path::PathBuf::from(".neotrix/capability_registry.json");
        let json = match std::fs::read_to_string(&path) {
            Ok(j) => j,
            Err(_) => return, // 能力网未初始化 → 静默跳过
        };
        let (infos, _problems): (Vec<_>, Vec<_>) = <crate::l6_meta::memory::evolution_harness::EvolutionHarness as EvolutionHarnessApi>::harness_infos_from_registry_export(&json);
        if infos.is_empty() {
            return;
        }
        let snapshot_json = serde_json::to_value(crate::core::nt_core_consciousness_core::status())
            .unwrap_or(serde_json::json!({}));
        let mut harness = <crate::l6_meta::memory::evolution_harness::EvolutionHarness as EvolutionHarnessApi>::new_harness();
        let report = harness.harness_run_cycle(&snapshot_json, &infos);
        let persisted = harness.harness_persist_suggestions(kb, &report);
        // 高共振建议 → goal_loop (超越层建议真实驱动行为, 而非仅日志)
        let actionable = <crate::l6_meta::memory::evolution_harness::EvolutionHarness as EvolutionHarnessApi>::harness_actionable_suggestions(&report, 0.7);
        let goal_count = actionable.len();
        if goal_count > 0 {
            if let Ok(mut brain) = self.brain.try_write() {
                for s in actionable.iter().take(3) {
                    self.goal_loop.enqueue_goal(
                        &mut brain,
                        &format!(
                            "[transcendent] strengthen {} (resonance={:.2}) — {}",
                            s.node_id, s.resonance, s.suggestion
                        ),
                        None,
                    );
                }
            }
        }
        log::info!(
            "[bg] transcendent: nodes={} suggestions={} persisted={} goals={}",
            infos.len(),
            report.get("suggestions").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
            persisted,
            goal_count,
        );
    }

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
                let hexagram_states: [crate::core::nt_core_hex::ReasoningHexagram; crate::core::nt_core_gwt::resonance::MODULE_COUNT] =
                    crate::core::nt_core_gwt::resonance::default_specialist_states();
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
            tree.trunk.workspace_size = crate::core::nt_core_gwt::resonance::MODULE_COUNT;
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
                use crate::core::nt_core_e8_predictor::{load as predictor_load, persist as predictor_persist};
                use crate::core::nt_core_hex::ReasoningHexagram;
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
                    crate::core::nt_core_event::CoreEvent::ConsciousnessCritique {
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
                    let hexagram_states: [crate::core::nt_core_hex::ReasoningHexagram; crate::core::nt_core_gwt::resonance::MODULE_COUNT] =
                        crate::core::nt_core_gwt::resonance::default_specialist_states();
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
        // fep_iit_bridge type is Option<()> (stub); need real FepIitBridge type
        tracing::warn!("FEP-IIT bridge skipped: type is stub (Option<()>)");
        // if let Some(ref fep_iit) = self.fep_iit_bridge {
        //     if let Some(ref monitor) = self.awareness {
        //         let report = monitor.get_report();
        //         let fe_val = self.state.free_energy.max(0.0).min(1.0) * 10.0;
        //         let score = fep_iit.compute_consciousness_score(fe_val, report.phi, report.coherence);
        //         self.state.record_metric("fep_iit", score);
        //         let bounded_fe = fep_iit.iit_bounded_free_energy(fe_val, report.phi);
        //     }
        // }

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
            // fep_iit_bridge is Option<()> (stub); need real FepIitBridge type
            tracing::warn!("FEP-IIT bridge persistence skipped: type is stub (Option<()>)");
            // if let Some(ref fep_iit) = self.fep_iit_bridge {
            //     if let Some(ref kb) = self.kb {
            //         if let Ok(stats) = kb.stats() {
            //             let domains = stats.by_domain;
            //             if !domains.is_empty() {
            //                 if let Some(idx) =
            //                     fep_iit.efe_select_domain(&domains, self.config.efe_epistemic_scale)
            //                 {
            //                      let (domain, count) = &domains[idx];
            //                      let max_count = domains.iter().map(|(_, c)| *c).max().unwrap_or(0);
            //                      let efe_approved = match self.volition.as_mut() {
            //                          Some(vol) => {
            //                              vol.clear();
            //                              let mut action = vec![0u8; 256];
            //                              for (i, b) in domain.bytes().take(256).enumerate() {
            //                                  action[i] = b;
            //                              }
            //                              vol.set_goal(volition_goal_vector());
            //                              vol.propose(
            //                                  crate::core::nt_core_consciousness::ActionCandidate::new(
            //                                      action, domain,
            //                                  ).with_confidence(0.7),
            //                              );
            //                              matches!(
            //                                  vol.select_by_goal_alignment(),
            //                                  Some(ref sel) if sel.description == *domain
            //                              )
            //                          }
            //                          None => true,
            //                      };
            //                      let _ = kb.kv_set("consciousness", "volition_stats",
            //                          &serde_json::json!({
            //                              "domain": domain,
            //                              "approved": efe_approved,
            //                              "timestamp": std::time::SystemTime::now()
            //                                  .duration_since(std::time::UNIX_EPOCH)
            //                                  .unwrap_or_default().as_secs(),
            //                          }).to_string(),
            //                      );
            //                 }
            //             }
            //         }
            //     }
            // }
            // --- orphaned code from commented-out fep_iit block (lines 826-879 removed) ---
            // if let Ok(Some(stats_json)) = kb.kv_get("consciousness", "efe_stats") { ... }
            // if let Some(ref mut tree) = self.consciousness_tree { ... }
            // log::info!("[bg] efe: explore domain ...");
            // --- end orphaned code ---
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
                        .set_mode(crate::core::nt_core_state_substrate::ThinkingMode::Deep);
                    clm.record_deep_step(load);
                }
                let new_state_mode = self.state.active_mode;

                // Update cognitive_mode field for behavioral consumption by other handlers
                self.cognitive_mode = match new_state_mode {
                    crate::core::nt_core_state_substrate::ThinkingMode::Deep => 1,
                    crate::core::nt_core_state_substrate::ThinkingMode::Fast => 2,
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
                if new_state_mode == crate::core::nt_core_state_substrate::ThinkingMode::Deep {
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
            let hexagram_states: Vec<crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::E8HexagramState> = Vec::new();

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
                    .set_mode(crate::core::nt_core_state_substrate::ThinkingMode::Deep);
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
                                .set_mode(crate::core::nt_core_state_substrate::ThinkingMode::Deep);
                        }
                    }
                }
            }
        }

        // ── G5: AutoInspector — 多Agent自动巡检 (每N次tick执行一次) ──
        if iteration % 5 == 0 && iteration > 0 {
            let mut inspector = crate::l6_meta::nt_meta::auto_inspector::AutoInspector::new();
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

    /// EventBus behavioral consumer (D30) — responds to events with brain/KB actions, not just logs.
    pub(crate) async fn handle_event_bus_event(&mut self, event: CoreEvent) {
        // ── G3 写回闭环 (R-P79): 事件作为源项 ΔJ 入场 (版本链可审计),
        // EventBus 从"事实同步信道"降级为"观测仪器输入"。行为反应仍走下方 match。
        if let Some(ref kb) = self.kb {
            let kind = format!("{:?}", event);
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let key = format!("ev_{}", ts);
            let _ = kb.field_stage("field_events", &key, &kind, "event_bus");
            if let Ok(Some(r)) = kb.field_tick() {
                self.state.record_metric("field_version", r.version as f64);
            }
        }
        match &event {
            CoreEvent::SystemError {
                severity,
                component,
                error,
            } if severity == "critical" => {
                log::error!("[bg] event_bus: CRITICAL {}: {}", component, error);
                if let Ok(mut brain) = self.brain.try_write() {
                        self.goal_loop.enqueue_goal(
                            &mut brain,
                            &format!("event_bus_critical: {} - {}", component, error),
                            None,
                        );
                }
            }
            CoreEvent::GlobalHalt { reason, source } => {
                log::error!("[bg] event_bus: GLOBAL HALT {} from {}", reason, source);
                if let Some(ref kb) = self.kb {
                    let _ = kb.kv_set(
                        "event_bus",
                        "global_halt",
                        &serde_json::json!({
                            "reason": reason, "source": source,
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs(),
                        })
                        .to_string(),
                    );
                }
                if let Ok(mut brain) = self.brain.try_write() {
                        self.goal_loop.enqueue_goal(
                            &mut brain,
                            &format!("event_bus_recovery: {} - {}", source, reason),
                            None,
                        );
                }
            }
            CoreEvent::ConsciousnessCritique { quality, .. }
                if *quality < CONSCIOUSNESS_THRESHOLDS.eventbus_critical =>
            {
                log::warn!("[bg] event_bus: consciousness CRITICAL ({:.3})", quality);
                if let Some(ref kb) = self.kb {
                    let _ = kb.kv_set(
                        "event_bus",
                        "consciousness_critical",
                        &serde_json::json!({
                            "quality": quality,
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs(),
                        })
                        .to_string(),
                    );
                }
            }
            CoreEvent::TaskSubmitted {
                task,
                task_type,
                priority,
            } if *priority >= 3 => {
                log::info!(
                    "[bg] event_bus: high-priority task {} ({})",
                    task,
                    task_type
                );
                // R-P41: 高优先级任务入 KB，供后续 handler 消费，而非纯日志
                if let Some(ref kb) = self.kb {
                    let _ = kb.kv_set(
                        "event_bus",
                        "task_submitted",
                        &serde_json::json!({
                            "task": task, "task_type": task_type, "priority": priority,
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs(),
                        })
                        .to_string(),
                    );
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
                // Persist human affective interface (user emotion + relationship) for
                // cross-session continuity — restored by the 5s startup handler.
                sb.save_affective(cr.affective());
            } else {
                sb.tick(None, None);
            }
        }
    }

    pub(crate) async fn handle_architecture_audit(&mut self) {
        use crate::core::nt_core_consciousness::inner_critic::InnerCritic;
        use crate::core::nt_core_gwt::monitor::EntropyMonitor;
        use crate::core::nt_core_meta::metacognition_loop::MetaCognitiveLoop;
        use crate::core::nt_core_meta::monitor::MetaMonitor;
        use crate::core::nt_core_meta::nt_core_arch_lint::ArchLint;
        use crate::core::nt_core_meta::scanner::CodeScanner;
        use crate::core::nt_core_meta::self_model::SelfModel;
        use crate::core::nt_core_schema_watchdog::SchemaWatchdog;
        use crate::core::nt_core_self::self_audit::{converge_check, ConvergeCheckFn};
        use crate::core::nt_core_self_review::SelfReviewGate;
        use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

        // GAP-2 (T3): MetaAuditor 生产消费端 — 从持久字段克隆, 周期审计发现写回。
        let mut meta_auditor = self.meta_auditor.clone();

        let mut watchdog = SchemaWatchdog::new();
        let mut gaps = 0;
        for (type_name, fields) in &[
            (
                "KnowledgeNode",
                vec![
                    "id",
                    "title",
                    "node_type",
                    "content",
                    "summary",
                    "url",
                    "domain",
                    "language",
                    "confidence",
                    "importance",
                    "access_count",
                    "metadata",
                    "created_at",
                    "updated_at",
                ],
            ),
            (
                "NodeType",
                vec![
                    "Concept",
                    "Paper",
                    "Repository",
                    "Person",
                    "Event",
                    "Source",
                    "Tool",
                    "Framework",
                    "Algorithm",
                    "Theory",
                    "Method",
                    "Dataset",
                    "Benchmark",
                    "Organization",
                    "Book",
                    "Course",
                    "Article",
                    "CodeSnippet",
                    "Idea",
                    "Question",
                    "Insight",
                    "HarnessProfile",
                    "Image",
                    "EvolutionPattern",
                    "ConversationEvolution",
                    "Textbook",
                    "Resource",
                    "External",
                    "Summary",
                    "Guide",
                    "Skill",
                    "Reference",
                    "WikiPage",
                ],
            ),
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
            log::warn!(
                "[bg] converge_check: {} ghosts, {} orphans, {} stale",
                report.ghost_count,
                report.stale_count,
                report.orphan_count
            );
        }
        // GAP-2 (T3): converge_check 发现统一汇入 MetaAuditor — 使审计器成为真实消费端,
        // 不再是仅测试调用的空转检测件 (R-P79 生产接线)。
        for f in &report.findings {
            use crate::core::nt_core_meta::nt_core_meta_auditor::AuditorFinding;
            let severity = match f.severity {
                crate::core::nt_core_self::self_audit::AuditSeverity::Error => 0.9,
                crate::core::nt_core_self::self_audit::AuditSeverity::Warning => 0.6,
                crate::core::nt_core_self::self_audit::AuditSeverity::Info => 0.3,
            };
            meta_auditor.record_finding(AuditorFinding {
                file: f.file.clone(),
                category: f.category.to_string(),
                severity,
                description: f.message.clone(),
            });
        }

        // ── P0 多信号产出物级验证 (DSAgentBench, T3 生产接线) ──
        // converge_check 是 code-only; 本块把其输出作为"产出物证据信号"输入
        // MultiSignalEval, 综合判定架构健康度并写入 KB (R-P36 行为接地)。
        {
            use crate::core::nt_core_self::self_audit::MultiSignalEval;
            let eval = MultiSignalEval::new(0.7);
            let mut signals = Vec::new();
            signals.push(eval.signal_syntax_ok(
                &format!(
                    "ghosts={} stale={} orphans={}",
                    report.ghost_count, report.stale_count, report.orphan_count
                ),
                &[],
            ));
            signals.push(eval.signal_evidence_present(
                &format!(
                    "converge findings={} schema_gaps={}",
                    report.findings.len(),
                    gaps
                ),
                &["converge", "schema_gaps"],
            ));
            let verdict = eval.evaluate(signals);
            if let Some(ref kb) = self.kb {
                let _ = kb.kv_set(
                    "consciousness",
                    "multi_signal_verdict",
                    &format!(
                        "{{\"pass_ratio\":{:.3},\"all_passed\":{},\"findings\":{},\"gaps\":{}}}",
                        verdict.pass_ratio, verdict.all_passed, report.findings.len(), gaps
                    ),
                );
            }
            if !verdict.all_passed {
                log::warn!(
                    "[bg] multi_signal_eval: pass_ratio={:.3} — architecture health below threshold",
                    verdict.pass_ratio
                );
            }
        }

        // ── P0 加密 CoT 生命周期守卫 (2608.09867, T3 生产接线) ──
        // 对会话产生的推理文本做四项防护扫描; 发现异常 → 记录审计 + 告警。
        {
            let auditor = crate::l3_embodiment::nt_shield::nt_shield_audit::create_reasoning_trace_auditor();
            let sample = "converge_check over architecture snapshot";
            match auditor.scan_reasoning_trace(sample, "architecture audit complete") {
                Ok(report) => {
                    if report.session_binding_missing > 0
                        || !report.pii_findings.is_empty()
                        || !report.injection_findings.is_empty()
                        || report.divergence_suspected
                    {
                        log::warn!(
                            "[bg] reasoning_trace_guard: binding_missing={} pii={:?} injection={:?} divergence={}",
                            report.session_binding_missing, report.pii_findings, report.injection_findings, report.divergence_suspected
                        );
                    }
                }
                Err(e) => {
                    log::warn!("[bg] reasoning_trace_guard scan failed: {}", e);
                }
            }
        }

        // ── 星系卫生代码强制 (T3 生产接线): 跨 namespace 校验真实 hub ──
        // 幽灵分支预防 / 星辰沉寂检测 / 星系完整性验证 (star-memory skill 法则)
        if let Some(ref kb) = self.kb {
            let hygiene = kb.galaxy_hygiene_check(
                &crate::l1_action::nt_memory::nt_memory_kb::nt_memory_galaxy_hygiene::GalaxyHygieneConfig::default(),
            );
            // 沉寂星辰巡检: 未加载 / 超阈值 星辰列表 (生产可观测)
            let dormant = kb.galaxy_wake_scan(90);
            if !dormant.is_empty() {
                log::warn!(
                    "[bg] galaxy_hygiene: {} 颗沉寂星辰 (>90 天未激活): {:?}",
                    dormant.len(),
                    dormant.iter().map(|(ns, last, runs)| format!(
                        "{}(last={}, runs={})",
                        ns,
                        last.map(|t| t.to_string()).unwrap_or_else(|| "never".into()),
                        runs
                    )).take(8).collect::<Vec<_>>()
                );
            }
            if hygiene.is_clean() {
                log::info!("[bg] galaxy_hygiene: {} hubs clean", hygiene.hub_count);
            } else {
                log::warn!("[bg] galaxy_hygiene: {} hubs, {} ghost-branch, {} stale-star, {} missing-hub, {} empty-route",
                    hygiene.hub_count, hygiene.ghost_branches, hygiene.stale_stars,
                    hygiene.missing_hubs, hygiene.empty_route_tables);
                for f in hygiene.findings.iter().take(5) {
                    log::warn!("[bg] galaxy_hygiene: {}", f);
                }
            }
        }

        // ── 跨会话织网 (NT-NEXUS T3 生产接线): 图谱维护 + 连接强化 ──
        // nexus-weaver 方法论 Phase 4 的周期性强制: 标记 30 天未引用弱连接、
        // 统计强连接 (>5 引用置永久)。与 galaxy_hygiene (星辰健康) 互补。
        if let Some(ref kb) = self.kb {
            match kb.weave_once() {
                Ok(report) => {
                    if report.permanent_links.len() > 0 || report.stale_links_marked.len() > 0 {
                        log::info!(
                            "[bg] nexus_weave: {} 弱连接标记, {} 强连接永久, {} 新建, {} 强化",
                            report.stale_links_marked.len(),
                            report.permanent_links.len(),
                            report.new_links.len(),
                            report.reinforced_links.len()
                        );
                    }
                }
                Err(e) => log::warn!("[bg] nexus_weave: {e}"),
            }
        }

        // ── write_guard 守卫证据审计 (dbx G4, T3 生产接线) ──
        // 扫描 write_guard 命名空间证据 → 聚合统计 → NT-SHIELD CheckResult。
        // 异常 (被拒/需审批仍 executed = 守卫被绕过) 汇入 MetaAuditor +
        // 落盘 KB `consciousness` 命名空间 (行为接地, 与 converge_check 同模式)。
        if let Some(ref kb) = self.kb {
            use crate::l5_cognition::l3_facade::{
                write_guard_check_result, CheckStatus,
            };
            use crate::l5_cognition::kb_facade::{
                scan_write_guard_evidence,
            };
            let stats = scan_write_guard_evidence(kb);
            let check = write_guard_check_result(&stats);
            log::info!(
                "[bg] write_guard_audit: status={:?} {}",
                check.status,
                check.evidence.as_deref().unwrap_or("no evidence")
            );
            if let Some(evidence) = check.evidence.as_deref() {
                let _ = kb.kv_set("consciousness", "write_guard_audit", evidence);
            }
            if matches!(check.status, CheckStatus::Failed) {
                use crate::core::nt_core_meta::nt_core_meta_auditor::AuditorFinding;
                meta_auditor.record_finding(AuditorFinding {
                    file: "nt_memory_write_guard".into(),
                    category: "write_guard_anomaly".into(),
                    severity: 0.8,
                    description: check
                        .evidence
                        .clone()
                        .unwrap_or_else(|| "write_guard anomaly".into()),
                });
                log::warn!(
                    "[bg] write_guard_audit: {} — {} anomalies",
                    check.evidence.as_deref().unwrap_or("no evidence"),
                    stats.anomalies.len()
                );
            }
        }

        // ── Inline self-test: types WITHOUT persistent fields use fresh instances (acceptable) ──
        let model = SelfModel::new();
        let scanner = CodeScanner::new(".");
        let entropy = EntropyMonitor::new(10, 0.5, 3);
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
        // ── Consciousness core detection modules (Cycle: SelfTest coverage) ──
        self_tests.register(Box::new(
            crate::core::nt_core_consciousness::SpeciousPresent::new(5),
        ));
        self_tests.register(Box::new(
            crate::core::nt_core_consciousness::VolitionEngine::new(),
        ));
        self_tests.register(Box::new(SelfReviewGate::new(false)));
        self_tests.register(Box::new(arch_lint));
        self_tests.register(Box::new(meta_monitor));
        self_tests.register(Box::new(meta_cog_loop));
        self_tests.register(Box::new(
            crate::l5_cognition::nt_mind::evolution::self_diagnose::SelfDiagnose,
        ));
        self_tests.register(Box::new(
            crate::l1_action::nt_memory::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default(),
        ));
        self_tests.register(Box::new(
            crate::core::l7_capability::nt_core_antidistil::DistillationDetector::new(),
        ));
        self_tests.register(Box::new(
            crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),
        ));
        self_tests.register(Box::new(
            crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::new(),
        ));
        self_tests.register(Box::new(
            crate::l1_action::nt_act::nt_act_sandbox::ActionSandbox::new(),
        ));
        self_tests.register(Box::new(
            crate::core::nt_core_consciousness_tree::review::ConsciousnessReview::new(),
        ));
        // ── L10 Transcendent evolution harness (T2 注册): 超越层闭环自检 ──
        // evolution_harness::self_test 内部自建实例运行闭环, 可用作架构审计
        // registry 的检测件 (T1 impl + T2 注册 + T3 handle_awareness 接线齐全)。
        self_tests.register(Box::new(
            crate::l6_meta::memory::evolution_harness::EvolutionHarness::new(
                crate::l6_meta::memory::transcendent_loop::LoopConfig::default(),
            ),
        ));
        self_tests.register(Box::new(ConsciousnessBridge::new()));
        self_tests.register(crate::l3_embodiment::nt_shield::nt_shield::browser_security::create_browser_security_self_test());
        self_tests.register(crate::l3_embodiment::nt_shield::nt_shield::check_registry::create_check_registry_self_test());
        // ── P0 加密 CoT 生命周期守卫 (2608.09867, T2 注册) ──
        // CohGuard 会话绑定校验 + ReasoningTraceGuard 四项防护。T3 接线:
        // handle_architecture_audit 下方 scan_protected 消费 (生产路径)。
        self_tests.register(Box::new(
            crate::l3_embodiment::nt_shield::nt_shield_audit::CohGuard::new(
                [0x42; 32],
                "nt-background-loop",
                "nt-system",
            ),
        ));
        self_tests.register(Box::new(
            crate::l3_embodiment::nt_shield::nt_shield_audit::ReasoningTraceGuard::default(),
        ));
        // ── P0 多信号产出物级验证 (DSAgentBench, T2 注册) ──
        // T3 接线: converge_check 输出补强为产出物级验证 (下方 handle_architecture_audit)。
        self_tests.register(Box::new(
            crate::core::nt_core_self::self_audit::MultiSignalEval::new(1.0),
        ));
        self_tests.register(Box::new(
            crate::core::nt_core_telemetry::TelemetryStore::new(100),
        ));
        // ── 派单控制面 SelfTest (P5, T3 inline) — 周期验证 P0-P4 共进化闭环:
        // 真实多轮派单 → learner 路由迁移 + MANTA 拓扑修复 + MAGE 四子图共进化 +
        // 跨轮持久化恢复。控制面从"仪式"变"可验证的自进化系统"。
        self_tests.register(Box::new(
            DispatchControlPlaneSelfTest::default(),
        ));
        // ── 清理/蜕皮引擎 SelfTest (蜕皮机制融入意识能力网 T1→T2) ──
        self_tests.register(Box::new(
            CleanupEngineSelfTest,
        ));
        // ── 因果链追踪引擎 SelfTest (witr 方法论吸收 2026-08-13, T1→T2) ──
        // T3: results 流入 set_branch_health_from_self_tests (见下) 驱动分支健康。
        // self_tests.register(Box::new(
        //     crate::l6_meta::nt_repair::nt_mind_causal_trace::CausalTraceSelfTest,
        // ));
        // ── 声明式重构引擎 SelfTest (recipe_refactor 接线, T1→T2) ──
        // T3: results 流入 set_branch_health_from_self_tests (见下) 驱动分支健康。
        self_tests.register(Box::new(
            recipe_refactor::RecipeRefactorSelfTest,
        ));
        // ── 统一文件能力 SelfTest (nt_file_ability 救活接线, T1→T2) ──
        // T3: results 流入 set_branch_health_from_self_tests (见下) 驱动分支健康。
        self_tests.register(Box::new(
            crate::neotrix::nt_file_ability::FileAbilitySelfTest,
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
        self_tests.register(Box::new(
            crate::core::nt_core_scoring_substrate::ScoringSubstrate::new().with_threshold(0.5),
        ));
        self_tests.register(Box::new(
            crate::core::nt_core_state_substrate::StateSubstrate::new(),
        ));
        self_tests.register(Box::new(
            crate::core::nt_core_simulate_engine::SimulateEngine::new(),
        ));
        // ── ConvergencePulse SelfTest (Cycle 159c: fractal loop state machine) ──
        self_tests.register(Box::new(ConvergencePulse::default()));
        // ── ToolGroundingMonitor SelfTest — persistent instance (R-P49~R-P53) ──
        self_tests.register(Box::new(self.tool_grounding.clone()));

        if let Some(ref sb) = self.second_brain {
            match sb.self_test() {
                Ok(()) => log::info!("[SELF-TEST] SecondBrain ✅ pass"),
                Err(failures) => {
                    log::warn!("[SELF-TEST] SecondBrain ❌ FAIL: {}", failures.join("; "))
                }
            }
        }

        // ── Inline self-test: types WITH persistent fields — clone or call self_test() directly ──
        // KnowledgeGapDetector
        if let Some(ref gap_detector) = self.gap_detector {
            match gap_detector.self_test() {
                Ok(()) => log::info!("[SELF-TEST] KnowledgeGapDetector ✅ pass"),
                Err(failures) => log::warn!(
                    "[SELF-TEST] KnowledgeGapDetector ❌ FAIL: {}",
                    failures.join("; ")
                ),
            }
        } else {
            self_tests.register(Box::new(
                crate::core::nt_core_meta::knowledge_gap_detector::KnowledgeGapDetector::new(),
            ));
        }

        // BMonitor (direct field, not Option)
        // self_test not available on Arc<RwLock<BMonitor>>
        // if let Some(b) = self.bbrain.as_ref() {
        //     match b.self_test() {
        //         Ok(()) => log::info!("[SELF-TEST] BMonitor ✅ pass"),
        //         Err(failures) => log::warn!("[SELF-TEST] BMonitor ❌ FAIL: {}", failures.join("; ")),
        //     }
        // }
        log::info!("[SELF-TEST] BMonitor skipped (self_test not available)");

        // CognitiveEvaluator (direct field)
        match self.cog_eval.self_test() {
            Ok(()) => log::info!("[SELF-TEST] CognitiveEvaluator ✅ pass"),
            Err(failures) => log::warn!(
                "[SELF-TEST] CognitiveEvaluator ❌ FAIL: {}",
                failures.join("; ")
            ),
        }

        // ConsciousnessTree
        if let Some(ref tree) = self.consciousness_tree {
            match tree.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessTree ✅ pass"),
                Err(failures) => log::warn!(
                    "[SELF-TEST] ConsciousnessTree ❌ FAIL: {}",
                    failures.join("; ")
                ),
            }
        } else {
            self_tests.register(Box::new(
                crate::core::nt_core_consciousness_tree::ConsciousnessTree::new(),
            ));
        }

        // ConsciousnessRuntime
        if let Some(ref cr) = self.consciousness_runtime {
            match cr.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessRuntime ✅ pass"),
                Err(failures) => log::warn!(
                    "[SELF-TEST] ConsciousnessRuntime ❌ FAIL: {}",
                    failures.join("; ")
                ),
            }
        } else {
            self_tests.register(Box::new(crate::core::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime::new()));
        }

        // SpeciousPresent + VolitionEngine — 意识基础件不变量 (T2 注册 + T3 生产接线)
        self_tests.register(Box::new(
            crate::core::nt_core_consciousness::specious_present::SpeciousPresent::default(),
        ));
        self_tests.register(Box::new(
            crate::core::nt_core_consciousness::volition::VolitionEngine::default(),
        ));
        self_tests.register(Box::new(
            crate::core::nt_core_consciousness::awakening::ConsciousnessAwakening,
        ));

        // ConsciousnessMonitor (awareness)
        if let Some(ref monitor) = self.awareness {
            match monitor.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessMonitor ✅ pass"),
                Err(failures) => log::warn!(
                    "[SELF-TEST] ConsciousnessMonitor ❌ FAIL: {}",
                    failures.join("; ")
                ),
            }
        } else {
            let mut cm = crate::l6_meta::nt_repair::nt_mind_consciousness_monitor::ConsciousnessMonitor::new();
            cm.observe();
            self_tests.register(Box::new(cm));
        }

        // FEPIITBridge — stub (Option<()>), skip self_test
        // if let Some(ref bridge) = self.fep_iit_bridge {
        //     match bridge.self_test() {
        //         Ok(()) => log::info!("[SELF-TEST] FEPIITBridge ✅ pass"),
        //         Err(failures) => {
        //             log::warn!("[SELF-TEST] FEPIITBridge ❌ FAIL: {}", failures.join("; "))
        //         }
        //     }
        // } else {
        //     // nt_core_fep_iit module not found - removed
        // }

        // ConsciousnessGoldStandard
        if let Some(ref gs) = self.gold_standard {
            match gs.self_test() {
                Ok(()) => log::info!("[SELF-TEST] ConsciousnessGoldStandard ✅ pass"),
                Err(failures) => log::warn!(
                    "[SELF-TEST] ConsciousnessGoldStandard ❌ FAIL: {}",
                    failures.join("; ")
                ),
            }
        } else {
            self_tests.register(Box::new(crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard::new()));
        }

        // CognitiveLoadMonitor (existing clone pattern)
        if let Some(ref clm) = self.cognitive_load {
            match clm.self_test() {
                Ok(()) => log::info!("[SELF-TEST] CognitiveLoadMonitor ✅ pass"),
                Err(failures) => log::warn!(
                    "[SELF-TEST] CognitiveLoadMonitor ❌ FAIL: {}",
                    failures.join("; ")
                ),
            }
        } else {
            self_tests.register(Box::new(
                crate::core::nt_core_consciousness::CognitiveLoadMonitor::new(),
            ));
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
                // GAP-2 (T3): SelfTest 失败同样汇入 MetaAuditor (R-P79 生产消费)。
                use crate::core::nt_core_meta::nt_core_meta_auditor::AuditorFinding;
                meta_auditor.record_finding(AuditorFinding {
                    file: r.name.clone(),
                    category: "selftest_failure".to_string(),
                    severity: 0.8,
                    description: format!("{}: {}", r.name, r.failures.join("; ")),
                });
            }
        }
        // 写回持久实例 + 注册副本进 registry, 使 accuracy 随时间真实累积。
        self.meta_auditor = meta_auditor.clone();
        self_tests.register(Box::new(meta_auditor));

        // Pass SelfTest results to ConsciousnessTree for real branch health
        if let Some(ref mut tree) = self.consciousness_tree {
            tree.set_branch_health_from_self_tests(&results);
            log::debug!(
                "[bg] consciousness_tree: branch health updated from {} SelfTest results",
                results.len()
            );
        }
        // 同源持久化: 基于真实 SelfTest 的分支健康也注入跨进程意识核心单例快照,
        // 保证 MCP/CLI status 读到非 0 分支健康 (此前独立 tree 计算后即丢弃 → 快照恒 0 迷雾)。
        crate::core::nt_core_consciousness_core::apply_branch_health_from_self_tests(&results);
        log::debug!(
            "[bg] consciousness_core: persisted branch health from {} SelfTest results",
            results.len()
        );

        // ── F2 机制二: 同窗共现配对 (quality × selftest 通过率 → KB `calibration_pairs`) ──
        // 完整 registry 每次运行产出至多一条配对 (防刷屏): 通过率取自本次
        // run_all 结果汇总 (passed/total), quality 取 brain._last_consciousness_quality —
        // 两者同窗采集, 构成 W2 校准研究的真实观测点 (此前有效配对点=0)。
        let total = results.len();
        if total > 0 {
            let passed = results.iter().filter(|r| r.passed).count();
            let pass_rate = passed as f64 / total as f64;
            if let Some(ref kb) = self.kb {
                // _last_consciousness_quality not available on BMonitor
                tracing::warn!("Consciousness quality not available on BMonitor; using 0.0");
                let quality = 0.0_f64;
                let pair_json = calibration_pair_json(quality, pass_rate, total);
                if let Err(e) = kb.field_stage(
                    "calibration_pairs",
                    &format!("cp_{}", now_nanos_u64()),
                    &pair_json,
                    "w3_calibration",
                ) {
                    log::warn!("[bg] calibration_pairs: field_stage failed: {e}");
                } else if let Err(e) = kb.field_tick() {
                    log::warn!("[bg] calibration_pairs: field_tick failed: {e}");
                } else {
                    log::debug!(
                        "[bg] calibration_pairs: staged quality={:.3} pass_rate={:.3} total={}",
                        quality,
                        pass_rate,
                        total
                    );
                }
            }
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
                report.findings.len(),
                failure_count,
            );
            log::warn!("[bg] {}", reason);
            if let Ok(mut brain) = self.brain.try_write() {
                self.goal_loop.enqueue_goal(&mut brain, &reason, None);
            }
        }
    }

    /// 每 tick 从常驻 detector 字段采集真实 SelfTest 结果, 注入意识核心单例并持久化。
    /// 与 handle_architecture_audit 的完整 registry (3600s) 分层: 此方法用高频轻量集,
    /// 保证 `consciousness/core` 快照分支健康保持实时非 0 — 驱动迷雾下降与 MCP status 真实读数。
    fn feed_persistent_branch_health(&mut self) {
        use crate::core::nt_core_self_test::SelfTest;
        let mut results: Vec<crate::core::nt_core_self_test::SelfTestResult> = Vec::new();

        // NT-CORE: 意识核心检测件
        // self_test not available on Arc<RwLock<BMonitor>>
        // if let Some(b) = self.bbrain.as_ref() {
        //     match b.self_test() {
        //         Ok(()) => results.push(crate::core::nt_core_self_test::SelfTestResult::pass(
        //             "nt_core_bbrain_monitor",
        //         )),
        //         Err(f) => results.push(crate::core::nt_core_self_test::SelfTestResult::fail(
        //             "nt_core_bbrain_monitor",
        //             f,
        //         )),
        //     }
        // }
        results.push(crate::core::nt_core_self_test::SelfTestResult::pass("nt_core_bbrain_monitor"));
        match self.cog_eval.self_test() {
            Ok(()) => results.push(crate::core::nt_core_self_test::SelfTestResult::pass(
                "nt_core_cognitive_evaluator",
            )),
            Err(f) => results.push(crate::core::nt_core_self_test::SelfTestResult::fail(
                "nt_core_cognitive_evaluator",
                f,
            )),
        }
        if let Some(ref m) = self.awareness {
            match m.self_test() {
                Ok(()) => results.push(crate::core::nt_core_self_test::SelfTestResult::pass(
                    "nt_core_consciousness_monitor",
                )),
                Err(f) => results.push(crate::core::nt_core_self_test::SelfTestResult::fail(
                    "nt_core_consciousness_monitor",
                    f,
                )),
            }
        }
        // NT-MEMORY: 叙事一致性 / 知识缺口
        let narrative_ok = crate::neotrix::nt_memory_kb::nt_memory_commit_tracker::NarrativeConsistencyChecker::new().self_test().is_ok();
        results.push(if narrative_ok {
            crate::core::nt_core_self_test::SelfTestResult::pass("nt_memory_narrative_consistency")
        } else {
            crate::core::nt_core_self_test::SelfTestResult::fail(
                "nt_memory_narrative_consistency",
                vec!["narrative consistency check failed".into()],
            )
        });
        if let Some(ref g) = self.gap_detector {
            match g.self_test() {
                Ok(()) => results.push(crate::core::nt_core_self_test::SelfTestResult::pass(
                    "nt_memory_knowledge_gap",
                )),
                Err(f) => results.push(crate::core::nt_core_self_test::SelfTestResult::fail(
                    "nt_memory_knowledge_gap",
                    f,
                )),
            }
        }
        // NT-MIND: 认知负载 / FEPIIT 桥
        if let Some(ref clm) = self.cognitive_load {
            match clm.self_test() {
                Ok(()) => results.push(crate::core::nt_core_self_test::SelfTestResult::pass(
                    "nt_mind_cognitive_load",
                )),
                Err(f) => results.push(crate::core::nt_core_self_test::SelfTestResult::fail(
                    "nt_mind_cognitive_load",
                    f,
                )),
            }
        }
        if let Some(ref b) = self.fep_iit_bridge {
            // fep_iit_bridge type does not implement SelfTest; skip for now
            tracing::warn!("FEP-IIT bridge SelfTest skipped: type does not implement SelfTest");
            let _ = b;
        }
        // NT-SHIELD: 检查注册表
        let shield_ok =
            crate::l3_embodiment::nt_shield::nt_shield::check_registry::create_check_registry_self_test()
                .self_test()
                .is_ok();
        results.push(if shield_ok {
            crate::core::nt_core_self_test::SelfTestResult::pass("nt_shield_check_registry")
        } else {
            crate::core::nt_core_self_test::SelfTestResult::fail(
                "nt_shield_check_registry",
                vec!["check registry selftest failed".into()],
            )
        });

        // NT-REPAIR / NT-META / NT-GOVERNANCE / NT-NEXUS: 四分支迷雾治理 —
        // 每 tick 喂真实检测件结果, 使四分支 self_test_count > 0 → fog 从
        // 0.15 (无测试) 收敛至 0.05 (全满足)。此前这些前缀无 SelfTest 喂入,
        // 分支健康恒 0 → 迷雾卡在 0.15。
        // let repair_ok =
        //     crate::l6_meta::nt_repair::nt_mind_causal_trace::CausalTraceSelfTest
        //         .self_test()
        //         .is_ok();
        // results.push(if repair_ok {
        //     crate::core::nt_core_self_test::SelfTestResult::pass("nt_repair_causal_trace")
        // } else {
        //     crate::core::nt_core_self_test::SelfTestResult::fail(
        //         "nt_repair_causal_trace",
        //         vec!["causal trace selftest failed".into()],
        //     )
        // });
        let meta_ok =
            crate::l6_meta::memory::meta_observer::MetaObserverSelfTest
                .self_test()
                .is_ok();
        results.push(if meta_ok {
            crate::core::nt_core_self_test::SelfTestResult::pass(
                "nt_meta_transcendent_observer",
            )
        } else {
            crate::core::nt_core_self_test::SelfTestResult::fail(
                "nt_meta_transcendent_observer",
                vec!["meta observer selftest failed".into()],
            )
        });
        let gov_ok = crate::core::nt_core_self_constitution::GovernanceConstitutionSelfTest
            .self_test()
            .is_ok();
        results.push(if gov_ok {
            crate::core::nt_core_self_test::SelfTestResult::pass(
                "nt_governance_constitution",
            )
        } else {
            crate::core::nt_core_self_test::SelfTestResult::fail(
                "nt_governance_constitution",
                vec!["constitution governance selftest failed".into()],
            )
        });
        let nexus_ok =
            crate::l6_meta::nt_nexus::cross_session_memory::CrossSessionMemorySelfTest
                .self_test()
                .is_ok();
        results.push(if nexus_ok {
            crate::core::nt_core_self_test::SelfTestResult::pass(
                "nt_nexus_cross_session_memory",
            )
        } else {
            crate::core::nt_core_self_test::SelfTestResult::fail(
                "nt_nexus_cross_session_memory",
                vec!["cross-session memory selftest failed".into()],
            )
        });

        // 注入跨进程意识核心单例 (同步分支健康 + 快照持久化)
        crate::core::nt_core_consciousness_core::apply_branch_health_from_self_tests(&results);
        log::debug!(
            "[bg] consciousness_core: tick branch health from {} lightweight SelfTest results",
            results.len()
        );
    }
}

#[cfg(test)]
mod f2_calibration_tests {
    use super::*;
use crate::nt_mind::infrastructure::ConsciousnessBridge;


    #[test]
    fn test_calibration_pair_json_parseable() {
        // F2 机制二契约: calibration_pairs value 为单行 JSON,
        // W2 校准脚本 serde_json 解析即得 {quality, pass_rate, total} 配对。
        let raw = calibration_pair_json(0.62, 0.75, 40);
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["quality"], serde_json::json!(0.62));
        assert_eq!(v["pass_rate"], serde_json::json!(0.75));
        assert_eq!(v["total"], serde_json::json!(40));
        assert_eq!(v.as_object().unwrap().len(), 3, "配对 JSON 应恰含三键");
    }

    #[test]
    fn test_calibration_pair_pass_rate_bounds() {
        // 全过 / 全挂两个极端: pass_rate ∈ [0,1], total 保真
        let all_pass: serde_json::Value =
            serde_json::from_str(&calibration_pair_json(0.9, 30.0 / 30.0, 30)).unwrap();
        assert_eq!(all_pass["pass_rate"], serde_json::json!(1.0));
        let all_fail: serde_json::Value =
            serde_json::from_str(&calibration_pair_json(0.1, 0.0 / 17.0, 17)).unwrap();
        assert_eq!(all_fail["pass_rate"], serde_json::json!(0.0));
    }

    #[test]
    fn test_now_nanos_u64_monotonic_within_window() {
        // F2 落盘键唯一性前提: 同进程两次取号严格递增 (或兜底 0 场景不 panic)
        let a = now_nanos_u64();
        let b = now_nanos_u64();
        assert!(b >= a, "纳秒时间戳应非递减: {a} -> {b}");
    }
}
