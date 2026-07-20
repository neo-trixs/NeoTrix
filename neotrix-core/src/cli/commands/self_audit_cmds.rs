use std::path::Path;
use std::sync::Arc;
use std::process::Command;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

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
            "help" | _ => {
                let mut help = String::new();
                help.push_str("NeoTrix Internal Audit (NTIA):\n");
                help.push_str("  /self-audit all|ntia  — run all 10 checks\n");
                help.push_str("  /self-audit d41-d50   — run single check\n\n");
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
                    .filter_map(|l| l.split(':').last())
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

        let check = Command::new("cargo")
            .args(["check", "--lib", "-p", "neotrix"])
            .current_dir(project)
            .output();
        match check {
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                let errors = stderr.lines().filter(|l| l.contains("error[")).count();
                if errors == 0 {
                    out.push_str("  ✅ cargo check: 0 errors\n");
                } else {
                    out.push_str(&format!("  ❌ cargo check: {errors} errors\n"));
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
            let consumers = self.count_total_refs(&src, &format!("\\.evaluate\\(\\.check\\(\\.audit\\(\\.scan\\("));
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
                    .filter_map(|l| l.split(':').last())
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
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().filter_map(|l| l.split(':').last()).filter_map(|c| c.trim().parse::<usize>().ok()).sum::<usize>())
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
