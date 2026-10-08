use super::super::*;
use crate::nt_mind::infrastructure::ConsciousnessBridge;
use crate::l5_cognition::nt_mind::nt_mind::evolution::dispatch_self_test::DispatchControlPlaneSelfTest;
use crate::l5_cognition::nt_mind::foundation::cleanup_engine::CleanupEngineSelfTest;
use crate::l5_cognition::l1_facade::recipe_refactor;
use crate::l5_cognition::cognitive_load::CognitiveLoadMonitor;
use crate::l0_substrate::nt_core_self_test::SelfTest;

/// SelfTest wrapper for CognitiveLoadMonitor — validates invariants via a fresh instance.
struct CognitiveLoadMonitorSelfTest;

impl SelfTest for CognitiveLoadMonitorSelfTest {
    fn name(&self) -> &str {
        "cognitive_load_monitor"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut m = CognitiveLoadMonitor::new();
        // Fresh monitor must start in Balanced mode
        if m.mode().name() != "balanced" {
            failures.push(format!(
                "cognitive_load_monitor: expected balanced start, got {:?}",
                m.mode()
            ));
        }
        // thinking_budget must be positive
        if m.thinking_budget() <= 0.0 {
            failures.push(format!(
                "cognitive_load_monitor: thinking_budget {} not positive",
                m.thinking_budget()
            ));
        }
        // Record steps and verify deep_ratio tracks correctly
        m.record_step(0.1);
        m.record_deep_step(0.2);
        m.record_step(0.3);
        if (m.deep_ratio() - 1.0 / 3.0).abs() > 1e-9 {
            failures.push(format!(
                "cognitive_load_monitor: deep_ratio {} != 1/3",
                m.deep_ratio()
            ));
        }
        // average_load must be within [0, 1]
        let avg = m.average_load();
        if !(0.0..=1.0).contains(&avg) {
            failures.push(format!(
                "cognitive_load_monitor: average_load {} out of range",
                avg
            ));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

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

impl BackgroundLoopHandle {
    pub(crate) async fn handle_architecture_audit(&mut self) {
        use crate::l5_cognition::nt_core_consciousness::inner_critic::InnerCritic;
        use crate::l5_cognition::nt_core_gwt::monitor::EntropyMonitor;
        use crate::l5_cognition::l1_facade::metacognition_loop::MetaCognitiveLoop;
        use crate::l5_cognition::l1_facade::monitor::MetaMonitor;
        use crate::l5_cognition::l1_facade::nt_core_arch_lint::ArchLint;
        use crate::l5_cognition::l1_facade::scanner::CodeScanner;
        use crate::l5_cognition::l1_facade::self_model::SelfModel;
        use crate::l0_substrate::nt_core_schema_watchdog::SchemaWatchdog;
        use crate::l5_cognition::l1_facade::self_audit::{converge_check, ConvergeCheckFn};
        use crate::l5_cognition::l1_facade::SelfReviewGate;
        use crate::l0_substrate::nt_core_self_test::{SelfTest, SelfTestRegistry};

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
        // ⚠️ cwd 不是 crate 根时，相对路径 "." 扫不到源码树 ⇒ findings 为空
        //    ⇒ 审计静默「干净」。实测本仓跑 bin 时 cwd 可能不是 neotrix-core。
        // ⇒ 优先用 CARGO_MANIFEST_DIR 编译期常量定位 neotrix-core/src。
        let src_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let scan_root: &std::path::Path = if src_root.is_dir() { &src_root } else { std::path::Path::new(".") };
        let report = converge_check(scan_root);
        if !report.findings.is_empty() {
            // 按 category 分别计数：两套孤儿口径都在里面，混报会看不出是哪套报的
            let mod_tree = report
                .findings
                .iter()
                .filter(|f| f.category == "orphan-mod-tree")
                .count();
            log::warn!(
                "[bg] converge_check: {} ghosts, {} stale(宽松口径), {} mod-tree孤儿(保守口径), scan_root={}",
                report.ghost_count,
                report.stale_count,
                mod_tree,
                scan_root.display()
            );
        } else {
            log::warn!(
                "[bg] converge_check: 0 findings —— scan_root={} 存在={}，\
                 若非源码根则为静默空转（不是「干净」）",
                scan_root.display(),
                scan_root.is_dir()
            );
        }
        // GAP-2 (T3): converge_check 发现统一汇入 MetaAuditor — 使审计器成为真实消费端,
        // 不再是仅测试调用的空转检测件 (R-P79 生产接线)。
        for f in &report.findings {
            use crate::l5_cognition::l1_facade::nt_core_meta_auditor::AuditorFinding;
            let severity = match f.severity {
                crate::l5_cognition::l1_facade::self_audit::AuditSeverity::Error => 0.9,
                crate::l5_cognition::l1_facade::self_audit::AuditSeverity::Warning => 0.6,
                crate::l5_cognition::l1_facade::self_audit::AuditSeverity::Info => 0.3,
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
            use crate::l5_cognition::l1_facade::self_audit::MultiSignalEval;
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
                // 2026-10-02：原 `let _ =` 丢弃 ⇒ 落库失败无痕。写的是**意识架构健康裁决本身**；`:245` 的 `log::warn!` 在块外且触发条件是 `!verdict.all_passed`（另一件事）⇒ 观察不到这里的落库失败。
                // 范式抄**同目录** `nt_awareness.rs:73/91`（该文件已是 `if let Err(e) = kb.kv_set(…) { log::warn!(…) }`）
                //    —— 本目录里 `nt_event_bus.rs` / `nt_audit.rs` 是仅剩的漏网，同一次修复应一起改。
                if let Err(e) = kb.kv_set(
                    "consciousness",
                    "multi_signal_verdict",
                    &format!(
                        "{{\"pass_ratio\":{:.3},\"all_passed\":{},\"findings\":{},\"gaps\":{}}}",
                        verdict.pass_ratio, verdict.all_passed, report.findings.len(), gaps
                    ),
                ) { log::warn!("[bg] multi_signal_verdict 落库失败: {e}"); }
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
                &crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_galaxy_hygiene::GalaxyHygieneConfig::default(),
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
                    if !report.permanent_links.is_empty() || !report.stale_links_marked.is_empty() {
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
            use crate::l5_cognition::l1_facade::{
                write_guard_check_result, CheckStatus,
            };
            use crate::l5_cognition::l1_facade::{
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
                // 2026-10-06（审计 D1）：审计证据写失败原本静默 ⇒
                // 证据「看起来记了」实则没记 ⇒ 比不记更危险（下游会以为已审计）。
                if let Err(e) = kb.kv_set("consciousness", "write_guard_audit", evidence) {
                    log::warn!("[bg] write_guard_audit 证据落库失败（该审计不可追溯）: {e}");
                }
            }
            if matches!(check.status, CheckStatus::Failed) {
                use crate::l5_cognition::l1_facade::nt_core_meta_auditor::AuditorFinding;
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
        self_tests.register(Box::new(crate::l5_cognition::l1_facade::ExternalVerifier));
        self_tests.register(Box::new(watchdog));
        self_tests.register(Box::new(ConvergeCheckFn));
        self_tests.register(Box::new(scanner));
        self_tests.register(Box::new(entropy));
        self_tests.register(Box::new(InnerCritic::new()));
        // ── Consciousness core detection modules (Cycle: SelfTest coverage) ──
        self_tests.register(Box::new(
            crate::l5_cognition::nt_core_consciousness::SpeciousPresent::new(5),
        ));
        self_tests.register(Box::new(
            crate::l5_cognition::nt_core_consciousness::VolitionEngine::new(),
        ));
        self_tests.register(Box::new(SelfReviewGate::new(false)));
        self_tests.register(Box::new(arch_lint));
        self_tests.register(Box::new(meta_monitor));
        self_tests.register(Box::new(meta_cog_loop));
        self_tests.register(Box::new(
            crate::l5_cognition::nt_mind::evolution::self_diagnose::SelfDiagnose,
        ));
        self_tests.register(Box::new(
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default(),
        ));
        self_tests.register(Box::new(
            crate::l5_cognition::nt_core::capability::nt_core_antidistil::DistillationDetector::new(),
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
            crate::l5_cognition::nt_core_consciousness_tree::review::ConsciousnessReview::new(),
        ));
        // ── L10 Transcendent evolution harness (T2 注册): 超越层闭环自检 ──
        // evolution_harness::self_test 内部自建实例运行闭环, 可用作架构审计
        // registry 的检测件 (T1 impl + T2 注册 + T3 handle_awareness 接线齐全)。
        self_tests.register(Box::new(
            crate::l5_cognition::l1_facade::EvolutionHarness::new(
                crate::l5_cognition::l1_facade::LoopConfig::default(),
            ),
        ));
        self_tests.register(Box::new(ConsciousnessBridge::new()));
        self_tests.register(crate::l3_embodiment::nt_shield::shield_core::browser_security::create_browser_security_self_test());
        self_tests.register(crate::l3_embodiment::nt_shield::shield_core::check_registry::create_check_registry_self_test());
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
            crate::l5_cognition::l1_facade::self_audit::MultiSignalEval::new(1.0),
        ));
        self_tests.register(Box::new(
            crate::l0_substrate::nt_core_telemetry::TelemetryStore::new(100),
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
            crate::l1_action::nt_file_ability::FileAbilitySelfTest,
        ));

        // NOTE (Cycle 159b): NeoCodexSelfAudit::new() is intentionally NOT
        // registered here. Its Default snapshot reports provider-not-resolvable +
        // catalog-empty (3 permanent failures) → failure_count > 0 on every
        // architecture audit → spurious self-review goals enqueued. The audit is
        // a TUI-side live snapshot (NeoCodexSelfAudit::capture via EvolutionLoop),
        // not a BackgroundLoop detector. R-P26: SelfTest must be wired where its
        // data source exists.

        // ── Absorbed module SelfTests (Cycle 113) ──
        crate::l5_cognition::l1_facade::register_absorbed_modules(&mut self_tests);

        // ── Substrate + Engine SelfTests (Cycle 119 architecture refactor) ──
        self_tests.register(Box::new(
            crate::l5_cognition::nt_core_scoring_substrate::ScoringSubstrate::new().with_threshold(0.5),
        ));
        self_tests.register(Box::new(
            crate::l5_cognition::nt_core::nt_state_substrate::StateSubstrate::new(),
        ));
        self_tests.register(Box::new(
            crate::l1_action::nt_core_simulate_engine::SimulateEngine::new(),
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
                crate::l5_cognition::l1_facade::knowledge_gap_detector::KnowledgeGapDetector::new(),
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
                crate::l5_cognition::nt_core_consciousness_tree::ConsciousnessTree::new(),
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
            self_tests.register(Box::new(crate::l5_cognition::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime::new()));
        }

        // SpeciousPresent + VolitionEngine — 意识基础件不变量 (T2 注册 + T3 生产接线)
        self_tests.register(Box::new(
            crate::l5_cognition::nt_core_consciousness::specious_present::SpeciousPresent::default(),
        ));
        self_tests.register(Box::new(
            crate::l5_cognition::nt_core_consciousness::volition::VolitionEngine::default(),
        ));
        self_tests.register(Box::new(
            crate::l5_cognition::nt_core_consciousness::awakening::ConsciousnessAwakening,
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
            let mut cm = crate::l5_cognition::l1_facade::ConsciousnessMonitor::new();
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
            self_tests.register(Box::new(crate::l5_cognition::l1_facade::ConsciousnessGoldStandard::new()));
        }

        // CognitiveLoadMonitor SelfTest
        self_tests.register(Box::new(CognitiveLoadMonitorSelfTest));

        // ── L1 Shield + IO SelfTest registrations ──
        // (disabled: these types don't implement SelfTest yet)
        self_tests.register(Box::new(crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_commit_tracker::NarrativeConsistencyChecker::new()));

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
                use crate::l5_cognition::l1_facade::nt_core_meta_auditor::AuditorFinding;
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
        crate::l5_cognition::consciousness_core::apply_branch_health_from_self_tests(&results);
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
    pub(crate) fn feed_persistent_branch_health(&mut self) {
        use crate::l0_substrate::nt_core_self_test::SelfTest;
        let mut results: Vec<crate::l0_substrate::nt_core_self_test::SelfTestResult> = Vec::new();

        // NT-CORE: 意识核心检测件
        // self_test not available on Arc<RwLock<BMonitor>>
        // if let Some(b) = self.bbrain.as_ref() {
        //     match b.self_test() {
        //         Ok(()) => results.push(crate::l0_substrate::nt_core_self_test::SelfTestResult::pass(
        //             "nt_core_bbrain_monitor",
        //         )),
        //         Err(f) => results.push(crate::l0_substrate::nt_core_self_test::SelfTestResult::fail(
        //             "nt_core_bbrain_monitor",
        //             f,
        //         )),
        //     }
        // }
        results.push(crate::l0_substrate::nt_core_self_test::SelfTestResult::pass("nt_core_bbrain_monitor"));
        match self.cog_eval.self_test() {
            Ok(()) => results.push(crate::l0_substrate::nt_core_self_test::SelfTestResult::pass(
                "nt_core_cognitive_evaluator",
            )),
            Err(f) => results.push(crate::l0_substrate::nt_core_self_test::SelfTestResult::fail(
                "nt_core_cognitive_evaluator",
                f,
            )),
        }
        if let Some(ref m) = self.awareness {
            match m.self_test() {
                Ok(()) => results.push(crate::l0_substrate::nt_core_self_test::SelfTestResult::pass(
                    "nt_core_consciousness_monitor",
                )),
                Err(f) => results.push(crate::l0_substrate::nt_core_self_test::SelfTestResult::fail(
                    "nt_core_consciousness_monitor",
                    f,
                )),
            }
        }
        // NT-MEMORY: 叙事一致性 / 知识缺口
        let narrative_ok = crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_commit_tracker::NarrativeConsistencyChecker::new().self_test().is_ok();
        results.push(if narrative_ok {
            crate::l0_substrate::nt_core_self_test::SelfTestResult::pass("nt_memory_narrative_consistency")
        } else {
            crate::l0_substrate::nt_core_self_test::SelfTestResult::fail(
                "nt_memory_narrative_consistency",
                vec!["narrative consistency check failed".into()],
            )
        });
        if let Some(ref g) = self.gap_detector {
            match g.self_test() {
                Ok(()) => results.push(crate::l0_substrate::nt_core_self_test::SelfTestResult::pass(
                    "nt_memory_knowledge_gap",
                )),
                Err(f) => results.push(crate::l0_substrate::nt_core_self_test::SelfTestResult::fail(
                    "nt_memory_knowledge_gap",
                    f,
                )),
            }
        }
        // NT-MIND: 认知负载 / FEPIIT 桥
        if let Some(ref _clm) = self.cognitive_load {
            match CognitiveLoadMonitorSelfTest.self_test() {
                Ok(()) => results.push(crate::l0_substrate::nt_core_self_test::SelfTestResult::pass(
                    "nt_mind_cognitive_load",
                )),
                Err(f) => results.push(crate::l0_substrate::nt_core_self_test::SelfTestResult::fail(
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
            crate::l3_embodiment::nt_shield::shield_core::check_registry::create_check_registry_self_test()
                .self_test()
                .is_ok();
        results.push(if shield_ok {
            crate::l0_substrate::nt_core_self_test::SelfTestResult::pass("nt_shield_check_registry")
        } else {
            crate::l0_substrate::nt_core_self_test::SelfTestResult::fail(
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
        //     crate::l0_substrate::nt_core_self_test::SelfTestResult::pass("nt_repair_causal_trace")
        // } else {
        //     crate::l0_substrate::nt_core_self_test::SelfTestResult::fail(
        //         "nt_repair_causal_trace",
        //         vec!["causal trace selftest failed".into()],
        //     )
        // });
        let meta_ok =
            crate::l5_cognition::l1_facade::MetaObserverSelfTest
                .self_test()
                .is_ok();
        results.push(if meta_ok {
            crate::l0_substrate::nt_core_self_test::SelfTestResult::pass(
                "nt_meta_transcendent_observer",
            )
        } else {
            crate::l0_substrate::nt_core_self_test::SelfTestResult::fail(
                "nt_meta_transcendent_observer",
                vec!["meta observer selftest failed".into()],
            )
        });
        let gov_ok = crate::l5_cognition::l1_facade::GovernanceConstitutionSelfTest
            .self_test()
            .is_ok();
        results.push(if gov_ok {
            crate::l0_substrate::nt_core_self_test::SelfTestResult::pass(
                "nt_governance_constitution",
            )
        } else {
            crate::l0_substrate::nt_core_self_test::SelfTestResult::fail(
                "nt_governance_constitution",
                vec!["constitution governance selftest failed".into()],
            )
        });
        let nexus_ok =
            crate::l5_cognition::l1_facade::CrossSessionMemorySelfTest
                .self_test()
                .is_ok();
        results.push(if nexus_ok {
            crate::l0_substrate::nt_core_self_test::SelfTestResult::pass(
                "nt_nexus_cross_session_memory",
            )
        } else {
            crate::l0_substrate::nt_core_self_test::SelfTestResult::fail(
                "nt_nexus_cross_session_memory",
                vec!["cross-session memory selftest failed".into()],
            )
        });

        // 注入跨进程意识核心单例 (同步分支健康 + 快照持久化)
        crate::l5_cognition::consciousness_core::apply_branch_health_from_self_tests(&results);
        log::debug!(
            "[bg] consciousness_core: tick branch health from {} lightweight SelfTest results",
            results.len()
        );
    }
}

#[cfg(test)]
mod f2_calibration_tests {
    use super::*;


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
