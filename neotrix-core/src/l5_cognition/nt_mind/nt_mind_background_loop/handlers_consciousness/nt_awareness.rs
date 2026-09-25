use super::super::*;
use log::info;

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
                        if let Err(e) = kb.kv_set("consciousness", "phi_report", &phi_json.to_string()) {
                            log::warn!("[consciousness] failed to persist phi_report: {}", e);
                        }
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
                        if let Err(e) = kb.kv_set("consciousness", "gold_standard", &gs_json.to_string()) {
                            log::warn!("[consciousness] failed to persist gold_standard: {}", e);
                        }
                    }

                    // Persist trends (phi_trend, coherence_trend, health_trend)
                    let trends_json = serde_json::json!({
                        "phi_trend": aw.trends.phi_trend,
                        "coherence_trend": aw.trends.coherence_trend,
                        "health_trend": aw.trends.health_trend,
                    });
                    if let Err(e) = kb.kv_set("consciousness", "trends", &trends_json.to_string()) {
                        log::warn!("[consciousness] failed to persist trends: {}", e);
                    }

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
                    if let Err(e) = kb.kv_set("consciousness", "conversation", &conv_json.to_string()) {
                        log::warn!("[consciousness] failed to persist conversation: {}", e);
                    }

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
                    if let Err(e) = kb.kv_set("consciousness", "blind_spots", &spots_json.to_string()) {
                        log::warn!("[consciousness] failed to persist blind_spots: {}", e);
                    }

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
                    //         let input = crate::l5_cognition::l1_facade::nt_core_intra_reflection::ReflectionInput {
                    //             reasoning_trace: trace,
                    //             e8_mode_history: Vec::new(),
                    //             execution_time_ms: 0,
                    //             error_count: 0,
                    //             outcome_success: Some(phi > 0.3),
                    //         };
                    //         let report =
                    //             crate::l5_cognition::l1_facade::nt_core_intra_reflection::analyze(
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
                    if let Err(e) = kb.record_consciousness_snapshot(
                        phi,
                        coherence,
                        is_conscious,
                        tier_label,
                        &details,
                    ) {
                        log::warn!("[consciousness] failed to record consciousness snapshot: {}", e);
                    }
                }
            }

        // ── L10 Transcendent wiring (T3): 超越层闭环真实接线 ──
        // 读取意识核心快照 + 能力网注册表, 运行超越层闭环, 建议真实落盘 KB,
        // 高共振建议 → goal_loop (行为影响)。此前 meta_observer/consonance/
        // transcendent_loop 仅 pub use 导出, 生产路径零调用 — 孤儿超越层。
        self.run_transcendent_tick().await;

        // Emit awareness tick event on the EventBus
        if let Some(ref bus) = self.event_bus {
            bus.emit(crate::l0_substrate::nt_core_event::CoreEvent::TaskSubmitted {
                task: "awareness_tick".into(),
                task_type: "consciousness".into(),
                priority: 1,
            });
        }
    }
}
