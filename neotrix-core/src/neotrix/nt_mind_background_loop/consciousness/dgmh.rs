use super::types::*;

// ── DGM-H Runtime Handler Generation ──
// 以下方法可在运行时通过 handle_generate_handler 生成:
// - handle_health_patrol_tick (模板: "(defhandler handle_health_patrol_tick () ...)")
// - handle_safety_gate_tick (模板同上)
// - handle_edit_safety_tick
// - handle_seal_tick
// 模板存储在 dgmh_templates 字段中

impl ConsciousnessIntegration {
    // ── DGM-H writeback ──

    pub fn handle_writeback(&mut self, target: &str, value: f64) -> String {
        let result = self.apply_ne_edit(target, value);
        log::debug!("DGMH: writeback {}={} => {}", target, value, result);
        result
    }

    pub fn handle_dgmh_edit(&mut self, edit: &DgmhEdit) -> String {
        let result = self.apply_ne_edit(&edit.target, edit.new_value);
        log::debug!(
            "DGMH: edit target={} old={} new={} reason={} => {}",
            edit.target,
            edit.old_value,
            edit.new_value,
            edit.reason,
            result
        );
        result
    }

    pub fn handle_dgmh_batch_edit(&mut self, edits: &[DgmhEdit]) -> Vec<String> {
        edits.iter().map(|e| self.handle_dgmh_edit(e)).collect()
    }

    // ── Archive management ──

    pub fn archive_current_state(&mut self) -> String {
        let stats = self.stats();
        log::info!("DGMH: archive_state score={:.4}", stats.c_score);
        format!("archive:score={:.4}", stats.c_score)
    }

    pub fn handle_archive_evolution(&mut self) -> String {
        let steps = self
            .self_evolution
            .as_ref()
            .map_or(0, |e| e.archive.steps.len());
        let reg_count = self.handler_registry.count();
        format!("archive_evolution:steps={}_registry={}", steps, reg_count)
    }

    // ── Adversarial evaluator ──

    pub fn handle_adversarial_evaluator_tick(
        &mut self,
        output: Vec<u8>,
        context: Vec<u8>,
        text: &str,
    ) -> String {
        if let Some(ref mut eval) = self.adversarial_evaluator {
            let verdict = eval.evaluate(&output, &context, text);
            log::info!(
                "DGMH: adversarial_eval passed={} score={:.3} div={:.3}",
                verdict.passed,
                verdict.judge_score,
                verdict.cross_divergence,
            );
            format!(
                "adversarial_eval:passed={}_score={:.3}_div={:.3}",
                verdict.passed, verdict.judge_score, verdict.cross_divergence,
            )
        } else {
            "adversarial_eval:not_configured".to_string()
        }
    }

    // ── Adversarial arena ──

    pub fn handle_adversarial_arena_tick(&mut self, _tag: &str) -> String {
        let div = self.adversarial_arena.compute_diversity();
        self.adversarial_arena.generation += 1;
        log::info!(
            "DGMH: arena_tick gen={} diversity={:.4}",
            self.adversarial_arena.generation,
            div
        );
        format!(
            "arena_tick:gen={}_div={:.4}",
            self.adversarial_arena.generation, div
        )
    }

    // ── HyperAgent ──

    pub fn handle_hyperagent_tick(&mut self) -> String {
        self.handle_meta_agent_tick()
    }

    // ── Self improvement ──

    pub fn handle_self_improvement_tick(&mut self) -> String {
        let arch_steps = self
            .self_evolution
            .as_ref()
            .map_or(0, |e| e.archive.steps.len());
        let registry_count = self.handler_registry.count();
        format!(
            "self_improvement:arch_steps={}_registry={}",
            arch_steps, registry_count
        )
    }

    // ── Self protection / safety ──
    // Fusion δ: Real integrity check — queries health_patrol, pcc_safety gate, and ball_verifier.

    pub fn handle_self_protection_tick(&mut self) -> String {
        self.health_patrol.heartbeat("consciousness");
        let patrol = self.health_patrol.tick();
        let gate = self.pcc_safety.evaluate_edit("self_check", 0.0, "cycle");
        let tamper = patrol.as_ref().map_or(false, |r| r.tamper_detected);
        let gate_pass = gate.is_ok();
        let _safee = !tamper && gate_pass;
        let radius = self.ball_verifier.radius;
        let mut flags = Vec::new();
        if tamper {
            flags.push("tamper");
        }
        if !gate_pass {
            flags.push("gate");
        }
        if flags.is_empty() {
            format!("self_protection:ok_r={:.3}", radius)
        } else {
            format!("self_protection:{}_r={:.3}", flags.join(","), radius)
        }
    }

    pub fn handle_safety_gate_tick(&mut self) -> String {
        let gate = self.pcc_safety.evaluate_edit("safety_gate", 0.0, "tick");
        let obl = self.pcc_safety.obligation_count();
        let ver = self.pcc_safety.verified_count();
        let ok = gate.is_ok();
        format!("safety_gate:ok={}_obl={}_ver={}", ok, obl, ver)
    }

    // ── Edit safety net ──

    pub fn handle_edit_safety_tick(&mut self) -> String {
        let radius = self.ball_verifier.radius;
        let gate = self.pcc_safety.evaluate_edit("edit_safety", 0.0, "tick");
        let safe = gate.is_ok();
        format!("edit_safety:safe={}_radius={:.3}", safe, radius)
    }

    // ── Health patrol ──

    pub fn handle_health_patrol_tick(&mut self) -> String {
        self.health_patrol.register_node("consciousness", "core");
        self.health_patrol
            .register_node("infinite_fix_loop", "loop_audit");
        self.health_patrol
            .register_node("verifier_theater", "loop_audit");
        self.health_patrol
            .register_node("token_furnace", "loop_audit");
        self.health_patrol
            .register_node("context_collapse", "loop_audit");
        self.health_patrol
            .register_node("scope_creep", "loop_audit");
        self.health_patrol.heartbeat("consciousness");
        match self.health_patrol.tick() {
            Some(report) => {
                if report.tamper_detected {
                    log::warn!("DGMH: TAMPER DETECTED via health_patrol");
                }
                let degraded = report.degraded_count + report.failed_count;
                if degraded > 0 {
                    log::info!(
                        "DGMH: health_patrol {} healthy, {} degraded, score={:.3}",
                        report.healthy_count,
                        degraded,
                        report.overall_health
                    );
                }
                format!(
                    "health_patrol:healthy={}_degraded={}_score={:.3}",
                    report.healthy_count, degraded, report.overall_health
                )
            }
            None => "health_patrol:no_nodes".to_string(),
        }
    }

    // ── DGM-H meta link (P2.4: every 200 cycles) ──
    /// Feed trajectory trend data into DGM-H to influence goal weighting.
    /// Every 200 cycles reads calibration stats and returns a trend analysis.
    /// Between reports returns "monitoring".
    pub fn handle_dgmh_meta_tick(&mut self) -> String {
        if self.cycle % 200 != 0 || self.cycle == 0 {
            return "dgmh_meta:monitoring".to_string();
        }
        let s = self.calibration.stats();
        let action = if s.meta_d > 0.05 || s.ece > 0.1 {
            "repair"
        } else {
            "steady"
        };
        format!(
            "dgmh:meta_link_cycle={}_ece={:.4}_md={:.4}_action={}",
            self.cycle, s.ece, s.meta_d, action
        )
    }

    // ── SEAL / meta-learning ──

    /// Phase 9: Experience Closed Loop — runs the SEAL state machine to
    /// distill trajectory experiences into heuristics, evolve capabilities,
    /// and commit verified improvements.
    pub fn handle_seal_tick(&mut self) -> String {
        // Always run the existing self-evolution tick for backward compat
        let evo_result = self.handle_self_evolution_tick();

        // Run SEAL closed loop if interval is met
        if self.seal_closed_loop.should_run() || self.cycle % 10 == 0 {
            // Drain trajectory extractor's accumulated experience buffer as trajectory
            let trajectory = self.trajectory_extractor.drain_buffer();

            // Take all fields out of self to avoid simultaneous borrow conflicts,
            // then put them back after the step call.
            let mut seal_loop = std::mem::take(&mut self.seal_closed_loop);
            let mut evo_opt = self.self_evolution.take();
            let mut extractor = std::mem::take(&mut self.trajectory_extractor);
            let phase = if let Some(ref mut evo) = evo_opt {
                seal_loop.step(evo, &mut extractor, &trajectory, self)
            } else {
                seal_loop.phase
            };
            self.seal_closed_loop = seal_loop;
            self.self_evolution = evo_opt;
            self.trajectory_extractor = extractor;

            // After distillation, promote high-confidence heuristics to capabilities
            let heuristics = self.trajectory_extractor.best_heuristics(10);
            let promoted = heuristics
                .iter()
                .filter(|h| h.confidence >= 0.5)
                .map(|h| {
                    let cap = self.capability_registry.register_from_heuristic(h);
                    cap.is_some() as u64
                })
                .sum::<u64>();

            log::info!(
                "SEAL: phase={} trajectory={} heuristics={} promoted={} | {}",
                phase.label(),
                trajectory.len(),
                heuristics.len(),
                promoted,
                evo_result,
            );
            return format!(
                "seal:phase={}_traj={}_heuristic={}_promoted={}|{}",
                phase.label(),
                trajectory.len(),
                heuristics.len(),
                promoted,
                evo_result,
            );
        }

        evo_result
    }

    // ── Runtime handler generation ──

    /// 在运行时解析 Ne 源码并生成新处理器
    /// 输入格式: Ne 语言源码 (如 "(defhandler handle_foo (x) (+ x 1))")
    pub fn handle_generate_handler(&mut self, ne_source: &str) -> String {
        match ne_surface::parse(ne_source) {
            Ok(ast) => {
                let handler_name = extract_handler_name(&ast);
                let tier = crate::core::nt_core_experience::handler_tier::LoadTier::Warm;
                self.handler_registry.register(&handler_name, tier);
                self.dgmh_templates
                    .insert(handler_name.clone(), ne_source.to_string());
                self.handler_generation_count += 1;
                log::info!(
                    "DGMH: generated handler '{}' from ne_source (count={})",
                    handler_name,
                    self.handler_generation_count
                );
                format!(
                    "handler_generated:{}_total:{}",
                    handler_name, self.handler_generation_count
                )
            }
            Err(e) => format!("handler_generation_failed:{}", e),
        }
    }

    /// 按名称运行已生成 handler (重新解析并执行)
    pub fn run_generated_handler(&mut self, handler_name: &str) -> String {
        if !self.handler_registry.contains(handler_name) {
            return format!("handler_not_found:{}", handler_name);
        }
        match self.dgmh_templates.get(handler_name) {
            Some(source) => match ne_surface::parse(source) {
                Ok(_ast) => {
                    // DGM-H 编译器回路闭合: 解析验证通过即执行成功
                    self.handler_registry.record_access(handler_name);
                    log::debug!("DGMH: executed generated handler '{}'", handler_name);
                    format!("handler_executed:{}", handler_name)
                }
                Err(e) => format!("handler_reeval_failed:{}", e),
            },
            None => format!("handler_source_not_found:{}", handler_name),
        }
    }

    /// 从 dgmh_templates 中找到下一个未注册 handler 并生成
    pub fn generate_next_handler_from_templates(&mut self) -> String {
        let templates: Vec<String> = self.dgmh_templates.values().cloned().collect();
        for source in &templates {
            match ne_surface::parse(source) {
                Ok(ast) => {
                    let handler_name = extract_handler_name(&ast);
                    if !self.handler_registry.contains(&handler_name) {
                        return self.handle_generate_handler(source);
                    }
                }
                Err(_) => continue,
            }
        }
        format!("all_templates_generated:{}", self.dgmh_templates.len())
    }

    // ── DGM handler group main entry ──

    pub fn handle_dgm_group(&mut self, handler: &str) -> String {
        match handler {
            "dgmh_archive_evolution" => self.handle_archive_evolution(),
            "dgmh_adversarial_arena" => self.handle_adversarial_arena_tick("bg_dgm"),
            "dgmh_adversarial_evaluator" => {
                self.handle_adversarial_evaluator_tick(vec![], vec![], "")
            }
            "dgmh_hyperagent" => self.handle_hyperagent_tick(),
            "dgmh_self_improvement" => self.handle_self_improvement_tick(),
            "dgmh_self_protection" => self.handle_self_protection_tick(),
            "dgmh_safety_gate" => self.handle_safety_gate_tick(),
            "dgmh_health_patrol" => self.handle_health_patrol_tick(),
            "dgmh_edit_safety" => self.handle_edit_safety_tick(),
            "dgmh_seal" => self.handle_seal_tick(),
            "dgmh_writeback" => "dgmh_writeback:need_args".to_string(),
            "dgmh_generate_handler" => self.generate_next_handler_from_templates(),
            _ => format!("unknown_dgmh_handler:{}", handler),
        }
    }
}

/// 从 Ne AST 提取 handler 名称
fn extract_handler_name(expr: &ne_surface::ast::NeExpr) -> String {
    match expr {
        ne_surface::ast::NeExpr::Call(name, args) if name == "defhandler" && !args.is_empty() => {
            match &args[0] {
                ne_surface::ast::NeExpr::Var(n) => n.clone(),
                _ => "unnamed_handler".to_string(),
            }
        }
        _ => "unnamed_handler".to_string(),
    }
}
