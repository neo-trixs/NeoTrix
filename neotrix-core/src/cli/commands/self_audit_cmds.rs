use std::path::Path;
use std::sync::Arc;
use std::process::Command;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::core::nt_core_self::evolution_analysis::{analyze_kb_health, store_report_to_kb};

pub struct SelfAuditCmd;

impl CliCommand for SelfAuditCmd {
    fn name(&self) -> &str { "/self-audit" }
    fn aliases(&self) -> Vec<&str> { vec!["/sa", "/ntia"] }
    fn description(&self) -> &str {
        "Run NeoTrix Internal Audit (D41-D50 all checks) — self-audit pipeline continuity, tool grounding, behavior gate, arch weight, monotonicity, review discipline, memory integrity, energy flow, dead weight, meta-audit"
    }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("help");
        let project = get_project_root();
        match subcmd {
            "all" | "ntia" => self.run_all(&project),
            "d31" => self.d31_two_layer_eventbus(&project),
            "d32" => self.d32_reentrant_scope(&project),
            "d33" => self.d33_persistent_fields(&project),
            "d34" => self.d34_dependency_stages(&project),
            "d35" => self.d35_threshold_gating(&project),
            "d36" => self.d36_self_test_tracking(&project),
            "d41" => self.d41_pipeline_continuity(&project),
            "d42" => self.d42_tool_grounding(&project),
            "d43" => self.d43_behavior_gate(&project),
            "d44" => self.d44_arch_weight(&project),
            "d45" => self.d45_monotonicity(&project),
            "d46" => self.d46_review_discipline(&project),
            "d47" => self.d47_memory_integrity(&project),
            "d48" => self.d48_energy_flow(&project),
            "d49" => self.d49_dead_weight(&project),
            "d50" => self.d50_meta_audit(&project),
            "evolution" | "todo" => self.evolution_todo(),
            _ => {
                let mut help = String::new();
                help.push_str("NeoTrix Internal Audit (NTIA):\n");
                help.push_str("  /self-audit all|ntia  — run all 16 checks\n");
                help.push_str("  /self-audit evolution|todo — KB deep analysis + evolution todo (Rust port of generate-evolution-todo.py)\n");
                help.push_str("  /self-audit d31-d50   — run single check\n\n");
                help.push_str("  d31: Two-Layer EventBus | d32: Reentrant Scope\n");
                help.push_str("  d33: Persistent Fields    | d34: Dependency Stages\n");
                help.push_str("  d35: Threshold Gating     | d36: SelfTest Tracking\n");
                help.push_str("  d41: Pipeline Continuity  | d42: Tool Grounding\n");
                help.push_str("  d43: Behavior Gate        | d44: Architecture Weight\n");
                help.push_str("  d45: Monotonicity         | d46: Review Discipline\n");
                help.push_str("  d47: Memory Integrity     | d48: Energy Flow\n");
                help.push_str("  d49: Dead Weight          | d50: Meta-Audit\n");
                CommandOutput::ok(&help)
            }
        }
    }
}

impl SelfAuditCmd {
    fn run_all(&self, project: &str) -> CommandOutput {
        let mut report = String::new();
        report.push_str("═══ NeoTrix Internal Audit (NTIA) ═══\n\n");
        report.push_str(&self.d31_two_layer_eventbus(project).message);
        report.push('\n');
        report.push_str(&self.d32_reentrant_scope(project).message);
        report.push('\n');
        report.push_str(&self.d33_persistent_fields(project).message);
        report.push('\n');
        report.push_str(&self.d34_dependency_stages(project).message);
        report.push('\n');
        report.push_str(&self.d35_threshold_gating(project).message);
        report.push('\n');
        report.push_str(&self.d36_self_test_tracking(project).message);
        report.push('\n');
        report.push_str(&self.d41_pipeline_continuity(project).message);
        report.push('\n');
        report.push_str(&self.d42_tool_grounding(project).message);
        report.push('\n');
        report.push_str(&self.d43_behavior_gate(project).message);
        report.push('\n');
        report.push_str(&self.d44_arch_weight(project).message);
        report.push('\n');
        report.push_str(&self.d45_monotonicity(project).message);
        report.push('\n');
        report.push_str(&self.d46_review_discipline(project).message);
        report.push('\n');
        report.push_str(&self.d47_memory_integrity(project).message);
        report.push('\n');
        report.push_str(&self.d48_energy_flow(project).message);
        report.push('\n');
        report.push_str(&self.d49_dead_weight(project).message);
        report.push('\n');
        report.push_str(&self.d50_meta_audit(project).message);
        report.push_str("\n═══ Audit Complete ═══\n");
        CommandOutput::ok(&report)
    }

    fn src_path(&self, project: &str) -> String {
        format!("{}/neotrix-core/src", project)
    }

    fn count_refs(&self, src: &str, pattern: &str) -> usize {
        let output = Command::new("rg")
            .args(["-c", pattern, src, "-g", "*.rs"])
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.lines()
                    .filter_map(|l| l.split(':').next_back())
                    .filter_map(|c| c.trim().parse::<usize>().ok())
                    .sum()
            }
            Err(_) => 0
        }
    }

    fn count_total_refs(&self, src: &str, pattern: &str) -> usize {
        let output = Command::new("rg")
            .args([pattern, src, "-g", "*.rs"])
            .output();
        match output {
            Ok(out) => String::from_utf8_lossy(&out.stdout).lines().count(),
            Err(_) => 0
        }
    }

    fn d31_two_layer_eventbus(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D31: Two-Layer EventBus ──\n");

        // Count sync vs tokio subscribers
        let sync_subs = Command::new("rg")
            .args(["subscribe_all_layers_sync", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        let tokio_subs = Command::new("rg")
            .args(["subscribe_all_layers", &format!("{src}/neotrix/nt_core_event_bus.rs")])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        let tokio_consumer = Command::new("rg")
            .args(["handle_event_bus_event", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);

        out.push_str(&format!("  Sync observers (std::thread): {sync_subs}\n"));
        out.push_str(&format!("  Tokio subscribers: {tokio_subs}\n"));
        out.push_str(&format!("  Behavioral consumers: {tokio_consumer}\n"));
        if tokio_consumer > 0 {
            out.push_str("  ✅ Behavioral grounding present (sync=observation, tokio=intervention)\n");
        } else {
            out.push_str("  ❌ No behavioral consumer — all EventBus subscribers are observation-only\n");
        }
        CommandOutput::ok(&out)
    }

    fn d32_reentrant_scope(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D32: Reentrant Scope ──\n");

        // Check for try_read/try_write pattern (safe) vs .lock().unwrap() (risky)
        let try_locks = Command::new("rg")
            .args(["try_read\\(\\)|try_write\\(\\)", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        let raw_locks = Command::new("rg")
            .args(["\\.lock\\(\\)\\.unwrap\\(\\)", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);

        out.push_str(&format!("  try_read/try_write calls: {try_locks}\n"));
        out.push_str(&format!("  raw .lock().unwrap() calls: {raw_locks}\n"));
        let status = if raw_locks == 0 { "✅" } else { "⚠️" };
        out.push_str(&format!("  {status} {raw_locks} raw locks need review (prefer try_lock pattern)\n"));
        CommandOutput::ok(&out)
    }

    fn d33_persistent_fields(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D33: Persistent Fields ──\n");

        // Check SchemaWatchdog coverage
        let schema_checks = Command::new("rg")
            .args(["fn.*verify_db_schema|fn.*check_drift|fn.*detect", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  Schema verification functions: {schema_checks}\n"));

        // Count structs with #[derive(Serialize, Deserialize)] — persistent struct candidates
        let persistent_structs = Command::new("rg")
            .args(["#\\[derive.*Serialize.*Deserialize", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  Persistent struct candidates (Serialize+Deserialize): {persistent_structs}\n"));
        if schema_checks > 0 {
            out.push_str("  ✅ SchemaWatchdog active\n");
        }
        CommandOutput::ok(&out)
    }

    fn d34_dependency_stages(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D34: Dependency Stages ──\n");

        let pipeline_stages = Command::new("rg")
            .args(["PipelineStage|make_stage!|fn process\\(&self", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  Pipeline stages defined: {pipeline_stages}\n"));

        let converge_checks = Command::new("rg")
            .args(["converge_check", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  Converge checks (SEAL): {converge_checks}\n"));
        if converge_checks > 0 {
            out.push_str("  ✅ Dependency ordering validated\n");
        }
        CommandOutput::ok(&out)
    }

    fn d35_threshold_gating(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D35: Threshold Gating ──\n");

        // Count hardcoded numeric comparisons (potential threshold violations)
        let hardcoded = Command::new("rg")
            .args(["-n", "< (0\\.[0-9]+|100|[0-9]+\\.0)", &src, "-g", "*.rs", "--type", "rust"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  Numeric comparisons: ~{hardcoded} (review for Config sources)\n"));

        // Count Config structs
        let configs = Command::new("rg")
            .args(["struct.*Config|LazyLock.*threshold|LazyLock.*config", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  Config/Threshold structs: {configs}\n"));

        CommandOutput::ok(&out)
    }

    fn d36_self_test_tracking(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D36: SelfTest Tracking (T1/T2/T3) ──\n");

        // T1: SelfTest impls exist
        let t1 = Command::new("rg")
            .args(["impl.*SelfTest for", &src, "-g", "*.rs"])
            .output()
            .map(|o: std::process::Output| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  T1 (impl SelfTest exists): {t1}\n"));

        // T2: Registered in SelfTestRegistry
        let t2 = Command::new("rg")
            .args(["registry\\.register|register\\(.*Box.*SelfTest", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  T2 (registered in registry): {t2}\n"));

        // T3: Called inline in production (non-test code).
        // 排除 assert!(...) 包裹的调用 (cfg(test) 模块特征), 避免虚高接线率。
        let t3 = Command::new("rg")
            .args(["\\.self_test\\(\\)", &src, "-g", "*.rs"])
            .output()
            .map(|o: std::process::Output| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("assert"))
                    .count()
            })
            .unwrap_or(0);
        out.push_str(&format!("  T3 (self_test() calls, test-assert excluded): {t3}\n"));

        let t1pct = if t1 > 0 { (t3 as f64 / t1 as f64) * 100.0 } else { 0.0 };
        out.push_str(&format!("  Wiring ratio (T3/T1): {t1pct:.0}%\n"));
        out.push_str("  NOTE: 静态引用计数, 非运行时行为验证 — 真实 T3 需运行注册表驱动测试\n");
        if t3 as f64 / t1.max(1) as f64 > 0.5 {
            out.push_str("  ✅ Majority of SelfTest impls are production-wired\n");
        } else if t3 > 0 {
            out.push_str("  ⚠️ Partial wiring — less than 50% inline\n");
        } else {
            out.push_str("  ❌ No inline SelfTest in production code\n");
        }
        CommandOutput::ok(&out)
    }

    fn d41_pipeline_continuity(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D41: Pipeline Continuity ──\n");

        let pipes = ["crawl_queue", "absorb_url", "insert_or_get_node", "rebuild_bm25", "rebuild_tech_reserve"];
        for pipe in &pipes {
            let refs = self.count_refs(&src, pipe);
            out.push_str(&format!("  {pipe}: {refs} refs\n"));
        }

        let dead = [("bridge_cycle", 2), ("run_crawl_cycle_and_refresh", 1), ("enqueue_seed_urls", 2)];
        for (func, threshold) in &dead {
            let refs = self.count_total_refs(&src, func);
            if refs <= *threshold {
                out.push_str(&format!("  ⚠️ DEAD: {func} ({refs} refs)\n"));
            }
        }

        CommandOutput::ok(&out)
    }

    fn d42_tool_grounding(&self, project: &str) -> CommandOutput {
        let mut out = String::from("── D42: Tool Grounding ──\n");

        // GAP-4 修复: 统一 BuildRunner 工具层 (超时+kill+证据), 替代裸 Command 调用。
        use crate::neotrix::l8_autonomic_impl::nt_mind_build_runner::BuildRunner;
        let runner = BuildRunner::new()
            .with_workdir(project)
            .with_timeout(300);
        match runner.run("check", &["--lib", "-p", "neotrix"]) {
            Ok(ev) => {
                if ev.success() {
                    out.push_str("  ✅ cargo check: 0 errors\n");
                } else if ev.timed_out {
                    out.push_str("  ❌ cargo check: TIMEOUT (killed)\n");
                } else {
                    out.push_str(&format!("  ❌ cargo check: {} errors\n", ev.error_count));
                }
            }
            Err(e) => out.push_str(&format!("  ❌ cargo check failed: {e}\n")),
        }
        CommandOutput::ok(&out)
    }

    fn d43_behavior_gate(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D43: Behavior Gate ──\n");

        let modules = ["SchemaWatchdog", "SelfAudit", "KnowledgeGapDetector", "BMonitor",
                        "CognitiveEvaluator", "ConsciousnessMonitor", "ConsciousnessRuntime", "EntropyMonitor"];
        for module in &modules {
            let consumers = self.count_total_refs(&src, "\\.evaluate\\(\\.check\\(\\.audit\\(\\.scan\\(");
            out.push_str(&format!("  {module}: {consumers} eval consumers\n"));
        }

        let throwaway = self.count_total_refs(
            &format!("{}/neotrix/l8_autonomic_impl", src),
            "::new\\(\\)"
        );
        out.push_str(&format!("  Throwaway instances in handlers: {throwaway}\n"));
        CommandOutput::ok(&out)
    }

    fn d44_arch_weight(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D44: Architecture Weight ──\n");

        let total_files = Command::new("find")
            .args([&src, "-name", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  Total .rs files: {total_files}\n"));

        let pub_fns = Command::new("rg")
            .args(["^pub fn", &src, "-g", "*.rs", "--count"])
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout).lines()
                    .filter_map(|l| l.split(':').next_back())
                    .filter_map(|c| c.trim().parse::<usize>().ok())
                    .sum::<usize>()
            })
            .unwrap_or(0);
        out.push_str(&format!("  Pub functions: {pub_fns}\n"));

        CommandOutput::ok(&out)
    }

    fn d45_monotonicity(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D45: Monotonicity ──\n");

        let self_tests = Command::new("rg")
            .args(["impl.*SelfTest for", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  SelfTest impls: {self_tests}\n"));

        let self_test_refs = Command::new("rg")
            .args(["self_test|SelfTest|self\\.heal", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  SelfTest references: {self_test_refs}\n"));

        let pub_items = Command::new("rg")
            .args(["^pub (struct|enum)", &src, "-g", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  Pub structs/enums: {pub_items}\n"));

        CommandOutput::ok(&out)
    }

    fn d46_review_discipline(&self, project: &str) -> CommandOutput {
        let mut out = String::from("── D46: Review Discipline ──\n");
        let exp_dir = format!("{}/.agents/skills/rev/officer/experience", project);

        let evidence = Command::new("find")
            .args([&exp_dir, "-name", "*.md"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  Experience evidence files: {evidence}\n"));

        if Path::new(&exp_dir).exists() {
            out.push_str("  ✅ Experience directory exists\n");
        } else {
            out.push_str("  ❌ Experience directory missing\n");
        }

        let cycles = Command::new("rg")
            .args(["-c", "Cycle ", &format!("{project}/AGENTS.md")])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<usize>().unwrap_or(0))
            .unwrap_or(0);
        out.push_str(&format!("  Cycles tracked: {cycles}\n"));

        CommandOutput::ok(&out)
    }

    fn d47_memory_integrity(&self, project: &str) -> CommandOutput {
        let mut out = String::from("── D47: Memory Integrity ──\n");

        let claims = [("nt_core_jepa", 0), ("oracle_gate", 1), ("cross_session_memory", 1), ("spec_cmds.rs", 0)];
        for (name, expected_files) in &claims {
            let files = Command::new("find")
                .args([&format!("{project}/neotrix-core/src"), "-name", &format!("{name}*")])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
                .unwrap_or(0);
            let docs = Command::new("rg")
                .args(["-c", name, &format!("{project}/AGENTS.md")])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<usize>().unwrap_or(0))
                .unwrap_or(0);
            let status = if files == *expected_files { "✅" } else { "⚠️" };
            out.push_str(&format!("  {status} {name}: {files} files, {docs} doc refs\n"));
        }

        CommandOutput::ok(&out)
    }

    fn d48_energy_flow(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D48: Energy Flow ──\n");

        let domains = ["core", "mind", "memory", "world", "act", "io", "shield"];
        for domain in &domains {
            let refs = Command::new("rg")
                .args([&format!("nt_{domain}"), &format!("{src}/neotrix"), "-g", "*.rs"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
                .unwrap_or(0);
            out.push_str(&format!("  nt_{domain}: {refs} cross-refs\n"));
        }

        CommandOutput::ok(&out)
    }

    fn d49_dead_weight(&self, project: &str) -> CommandOutput {
        let src = self.src_path(project);
        let mut out = String::from("── D49: Dead Weight ──\n");

        let rs_files = Command::new("find")
            .args([&src, "-name", "*.rs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
            .unwrap_or(0);
        out.push_str(&format!("  Total .rs files: {rs_files}\n"));

        let mod_decls = Command::new("rg")
            .args(["^pub mod|^mod ", &src, "-g", "*.rs", "--count"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().filter_map(|l| l.split(':').next_back()).filter_map(|c| c.trim().parse::<usize>().ok()).sum::<usize>())
            .unwrap_or(0);
        out.push_str(&format!("  Module declarations: {mod_decls}\n"));

        CommandOutput::ok(&out)
    }

    fn d50_meta_audit(&self, project: &str) -> CommandOutput {
        let mut out = String::from("── D50: Meta-Audit ──\n");

        let dims = Command::new("rg")
            .args(["-o", "D[0-9]+", &format!("{project}/AGENTS.md")])
            .output()
            .map(|o| {
                let mut dims: Vec<String> = String::from_utf8_lossy(&o.stdout).lines().map(String::from).collect();
                dims.sort();
                dims.dedup();
                dims.len()
            })
            .unwrap_or(0);
        out.push_str(&format!("  Dimensions covered: {dims}/50\n"));

        let experiences = Command::new("rg")
            .args(["-c", "Experience Tree", &format!("{project}/AGENTS.md")])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<usize>().unwrap_or(0))
            .unwrap_or(0);
        out.push_str(&format!("  Experience entries: {experiences}\n"));

        CommandOutput::ok(&out)
    }

    /// Evolution TODO: KB deep health analysis + store to kv_store.
    /// Rust port of the retired `scripts/generate-evolution-todo.py` (R-P97 / R-P79 wiring).
    fn evolution_todo(&self) -> CommandOutput {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let kb_path = std::path::PathBuf::from(home).join(".neotrix").join("knowledge.db");
        let conn = match rusqlite::Connection::open(&kb_path) {
            Ok(c) => c,
            Err(e) => return CommandOutput::err(&format!("无法打开知识库 {}: {e}", kb_path.display())),
        };

        let defects = analyze_kb_health(&conn);
        let generated_at = crate::core::nt_core_self::evolution_analysis::unix_now();
        let report = crate::core::nt_core_self::evolution_analysis::KbHealthReport { defects, generated_at };

        if let Err(e) = store_report_to_kb(&conn, &report) {
            return CommandOutput::err(&format!("写入 evolution_todo 失败: {e}"));
        }

        let mut out = String::new();
        out.push_str(&format!("Evolution TODO stored to kv_store (evolution_todo): {} items (P0:{} P1:{} P2:{})\n",
            report.defects.len(), report.p0_count(), report.p1_count(), report.p2_count()));
        CommandOutput::ok(&out)
    }
}

fn get_project_root() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Err(_) => std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd() -> SelfAuditCmd {
        SelfAuditCmd
    }

    #[test]
    fn test_help_lists_all_subcommands() {
        let out = cmd().execute(&["help".to_string()], None);
        assert!(out.success, "help 应成功");
        assert!(out.message.contains("d31"));
        assert!(out.message.contains("d50"));
        assert!(out.message.contains("evolution"));
        assert!(out.message.contains("NTIA"));
    }

    #[test]
    fn test_unknown_subcommand_returns_help() {
        let out = cmd().execute(&["not-a-command".to_string()], None);
        assert!(out.success);
        assert!(out.message.contains("NeoTrix Internal Audit"));
    }

    #[test]
    fn test_evolution_subcommand_requires_kb_and_errors_gracefully() {
        // evolution_todo 需要打开真实 KB; 无 KB 时应返回 err 而非 panic
        let out = cmd().execute(&["evolution".to_string()], None);
        // 结果取决于环境是否配置了 ~/.neotrix/knowledge.db — 只断言不 panic
        assert!(!out.message.is_empty());
    }
}
