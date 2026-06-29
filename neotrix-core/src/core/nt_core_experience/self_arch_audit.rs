// Self-Architecture Audit: consciousness autonomously detects wiring gaps in its own codebase.
// Scans mod.rs vs filesystem, finds unregistered modules, generates repair proposals.
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditSeverity {
    UnregisteredModule,
    MissingReexport,
    DeadCodeFile,
    DuplicateRegistration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiringGap {
    pub file_name: String,
    pub severity: AuditSeverity,
    pub description: String,
    pub repair_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchAuditReport {
    pub gaps: Vec<WiringGap>,
    pub total_modules: usize,
    pub unregistered_count: usize,
    pub healthy_count: usize,
    pub audit_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfArchAudit {
    pub registered_modules: HashSet<String>,
    pub known_files: Vec<String>,
    pub report: Option<ArchAuditReport>,
    pub auto_repair_enabled: bool,
}

impl SelfArchAudit {
    pub fn new() -> Self {
        let mut s = Self {
            registered_modules: HashSet::new(),
            known_files: Vec::new(),
            report: None,
            auto_repair_enabled: false,
        };
        // Pre-register known modules from nt_core_experience
        let known_modules = &[
            "calibration_engine",
            "capability_synthesizer",
            "co_evolution",
            "context_manager",
            "context_memory",
            "diff_impact",
            "economic",
            "epistemic",
            "evolution_bridge",
            "evolution_task_system",
            "failure_trace",
            "fusion_deliberator",
            "goal_decomposer",
            "handler_profiler",
            "health_patrol",
            "imagination_engine",
            "loss_function",
            "meta_evolution",
            "native_evolution_explorer",
            "parl",
            "persona_adapter",
            "reliability_gate",
            "sar_diagnostic",
            "scaffold",
            "seal_closed_loop",
            "seal_proposal_bridge",
            "self_arch_audit",
            "self_evolution_engine",
            "self_evolution_loop",
            "self_evolution_meta_layer",
            "self_evolution_pipeline",
            "skill_crystal",
            "soul_identity",
            "stacked_validation",
            "workstream_exporter",
        ];
        for m in known_modules {
            s.registered_modules.insert(m.to_string());
            s.known_files.push(m.to_string());
        }
        s
    }

    /// Register the known set of declared modules in `mod.rs`.
    pub fn register_modules(&mut self, module_names: &[&str]) {
        for name in module_names {
            self.registered_modules.insert(name.to_string());
        }
    }

    /// Discover files on disk that should be modules.
    pub fn discover_files(&mut self, file_names: &[&str]) {
        for name in file_names {
            let clean = name.strip_suffix(".rs").unwrap_or(name);
            if clean != "mod" && !self.known_files.contains(&clean.to_string()) {
                self.known_files.push(clean.to_string());
            }
        }
    }

    /// Compare registered vs discovered → produce audit report.
    pub fn audit(&mut self, timestamp: u64) -> ArchAuditReport {
        let mut gaps = Vec::new();
        let mut healthy_count = 0;

        for file_mod in &self.known_files {
            if self.registered_modules.contains(file_mod) {
                healthy_count += 1;
            } else {
                gaps.push(WiringGap {
                    file_name: file_mod.clone(),
                    severity: AuditSeverity::UnregisteredModule,
                    description: format!("{} exists on disk but not declared in mod.rs", file_mod),
                    repair_action: format!("Add `pub mod {};` to mod.rs", file_mod),
                });
            }
        }

        let report = ArchAuditReport {
            unregistered_count: gaps.len(),
            healthy_count,
            total_modules: self.known_files.len(),
            gaps,
            audit_timestamp: timestamp,
        };

        self.report = Some(report.clone());
        report
    }

    /// Auto-generate diff-format repair proposals for detected gaps.
    pub fn generate_repair_proposals(&self) -> Vec<String> {
        let Some(ref report) = self.report else {
            return vec!["No audit report yet — run audit() first".into()];
        };
        report
            .gaps
            .iter()
            .map(|g| format!("// FIX: {}", g.repair_action))
            .collect()
    }

    pub fn summary(&self) -> String {
        match &self.report {
            Some(r) => format!(
                "SelfArchAudit: {}/{} healthy, {} unregistered (auto-repair={})",
                r.healthy_count, r.total_modules, r.unregistered_count, self.auto_repair_enabled
            ),
            None => "SelfArchAudit: no audit run yet".into(),
        }
    }
}
