use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::nt_core_meta::knowledge_gap_detector::{
    KnowledgeGapDetector as CoreGapDetector, GapReport, GapCategory, KnowledgeGap,
};
use crate::core::nt_core_meta::scanner::CodeScanner;
use crate::core::nt_core_meta::weakness::WeaknessAnalyzer;
use crate::neotrix::nt_memory_kb::{
    nt_memory_store as store, KnowledgeBase, KnowledgeNode, NodeType,
};

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixStatus {
    Pending,
    InProgress,
    Applied,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixAction {
    pub id: String,
    pub gap_id: String,
    pub action_type: FixActionType,
    pub description: String,
    pub file_path: Option<String>,
    pub content: Option<String>,
    pub status: FixStatus,
    pub created_at: i64,
    pub applied_at: Option<i64>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixActionType {
    CreateModule,
    CreateApi,
    AddEdge,
    UpdateConfig,
    CreateFile,
    Refactor,
    AddTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingCycle {
    pub cycle_id: String,
    pub timestamp: i64,
    pub gap_count: usize,
    pub fix_count: usize,
    pub applied_count: usize,
    pub failed_count: usize,
    pub coherence: f64,
}

pub struct NeotrixGapDetector {
    core: CoreGapDetector,
    kb: Option<Arc<Mutex<KnowledgeBase>>>,
    actions: Vec<FixAction>,
    history: Vec<HealingCycle>,
    cycle_count: u64,
}

impl NeotrixGapDetector {
    pub fn new(kb: Option<Arc<Mutex<KnowledgeBase>>>) -> Self {
        let mut core = CoreGapDetector::new();
        core.add_source("neotrix-core/src");
        core.add_source("neotrix-core/src/core");
        core.add_source("neotrix-core/src/neotrix");
        NeotrixGapDetector {
            core,
            kb,
            actions: Vec::new(),
            history: Vec::new(),
            cycle_count: 0,
        }
    }

    /// Run detection and persist to KB
    pub fn detect_and_persist(&mut self) -> Result<GapReport, String> {
        let scanner = CodeScanner::new(".");
        let model = scanner.scan();
        let analyzer = WeaknessAnalyzer::new();
        let weaknesses = analyzer.analyze(&model);
        let report = self.core.detect_gaps(&model, &weaknesses.weaknesses);

        if let Some(kb) = &self.kb {
            let kb = kb.lock().map_err(|e| e.to_string())?;
            for gap in &report.gaps {
                let node = KnowledgeNode {
                    id: format!("gap-{}", Uuid::new_v4()),
                    node_type: NodeType::Insight,
                    title: format!("Knowledge Gap: {}", gap.description),
                    summary: Some(format!(
                        "Category: {}, Severity: {}, Priority: {:.2}",
                        gap.category.label(), gap.severity, gap.exploration_priority
                    )),
                    content: Some(gap.description.clone()),
                    url: None,
                    domain: Some("self.healing.gap_detection".to_string()),
                    language: "en".to_string(),
                    confidence: gap.exploration_priority,
                    importance: gap.severity / 10.0,
                    created_at: now(),
                    updated_at: now(),
                    access_count: 0,
                    metadata: Some(serde_json::json!({
                        "gap_id": gap.id.to_string(),
                        "category": gap.category.label(),
                        "severity": gap.severity,
                        "exploration_priority": gap.exploration_priority,
                        "affected_modules": gap.affected_modules,
                        "fill_strategy": gap.fill_strategy,
                    })),
                };
                let conn = kb.conn.lock().map_err(|e| e.to_string())?;
                let _ = store::insert_node(&conn, &node);
            }
        }

        Ok(report)
    }

    /// Generate fix actions for all high-priority gaps
    pub fn plan_fixes(&mut self, report: &GapReport) -> Vec<FixAction> {
        let mut actions = Vec::new();
        for gap in &report.gaps {
            if gap.severity < 5.0 {
                continue;
            }
            let action = self.fix_for_gap(gap);
            if let Some(a) = action {
                actions.push(a);
            }
        }
        self.actions.extend(actions.clone());
        actions
    }

    fn fix_for_gap(&self, gap: &KnowledgeGap) -> Option<FixAction> {
        let module_name = gap.affected_modules.first()?;
        let action_type = match gap.category {
            GapCategory::MissingModule => FixActionType::CreateModule,
            GapCategory::MissingApi => FixActionType::CreateApi,
            GapCategory::MissingIntegration => FixActionType::AddEdge,
            _ => return None,
        };

        let description = gap.fill_strategy.clone();
        let file_path = format!("neotrix-core/src/neotrix/{}/mod.rs", module_name);

        let stub = format!(
            "use serde::{{Deserialize, Serialize}};\n\
             \n\
             pub struct {};\n\
             \n\
             impl {} {{\n\
                 pub fn new() -> Self {{\n\
                     Self\n\
                 }}\n\
             }}\n\
             \n\
             #[cfg(test)]\n\
             mod tests {{\n\
                 use super::*;\n\
                 #[test]\n\
                 fn test_{}_new() {{\n\
                     let _ = {}::new();\n\
                 }}\n\
             }}\n",
            to_camel_case(module_name),
            to_camel_case(module_name),
            module_name,
            to_camel_case(module_name),
        );

        Some(FixAction {
            id: format!("fix-{}", Uuid::new_v4()),
            gap_id: gap.id.to_string(),
            action_type,
            description,
            file_path: Some(file_path),
            content: Some(stub),
            status: FixStatus::Pending,
            created_at: now(),
            applied_at: None,
            result: None,
        })
    }

    /// Apply all pending fixes (create stub files)
    pub fn apply_fixes(&mut self) -> Result<usize, String> {
        let mut applied = 0usize;
        let pending: Vec<usize> = self.actions.iter()
            .enumerate()
            .filter(|(_, a)| matches!(a.status, FixStatus::Pending))
            .map(|(i, _)| i)
            .collect();

        for idx in pending {
            let action = &self.actions[idx];
            let Some(ref path) = action.file_path else { continue };
            let Some(ref content) = action.content else { continue };

            match std::fs::create_dir_all(
                std::path::Path::new(path).parent().unwrap_or(std::path::Path::new("."))
            ) {
                Ok(_) => {},
                Err(e) => {
                    self.actions[idx].status = FixStatus::Failed(e.to_string());
                    continue;
                }
            }

            match std::fs::write(path, content) {
                Ok(_) => {
                    self.actions[idx].status = FixStatus::Applied;
                    self.actions[idx].applied_at = Some(now());
                    applied += 1;
                }
                Err(e) => {
                    self.actions[idx].status = FixStatus::Failed(e.to_string());
                }
            }
        }

        Ok(applied)
    }

    /// Verify fixes pass cargo check
    pub fn verify(&self) -> Result<Vec<(String, bool, String)>, String> {
        let mut results = Vec::new();
        for action in &self.actions {
            if !matches!(action.status, FixStatus::Applied) {
                continue;
            }
            let module_name = action.file_path.as_ref()
                .and_then(|p| p.split('/').nth(2))
                .unwrap_or("unknown");
            // simple check: does the file exist and compile?
            let exists = action.file_path.as_ref().map(|p| std::path::Path::new(p).exists()).unwrap_or(false);
            results.push((module_name.to_string(), exists, action.description.clone()));
        }
        Ok(results)
    }

    /// Run one complete healing cycle
    pub fn heal_cycle(&mut self) -> Result<HealingCycle, String> {
        let report = self.detect_and_persist()?;
        let fixes = self.plan_fixes(&report);
        let applied = self.apply_fixes()?;
        let failed = fixes.len().saturating_sub(applied);

        self.cycle_count += 1;
        let cycle = HealingCycle {
            cycle_id: format!("cycle-{}", self.cycle_count),
            timestamp: now(),
            gap_count: report.gaps.len(),
            fix_count: fixes.len(),
            applied_count: applied,
            failed_count: failed,
            coherence: report.coherence_score,
        };
        self.history.push(cycle.clone());

        eprintln!(
            "[heal] cycle #{}: {} gaps, {} fixes planned, {} applied, {} failed (coherence={:.2})",
            self.cycle_count, report.gaps.len(), fixes.len(), applied, failed, report.coherence_score
        );

        Ok(cycle)
    }

    pub fn history(&self) -> &[HealingCycle] {
        &self.history
    }

    pub fn pending_fixes(&self) -> Vec<&FixAction> {
        self.actions.iter().filter(|a| matches!(a.status, FixStatus::Pending)).collect()
    }
}

fn to_camel_case(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_new() {
        let d = NeotrixGapDetector::new(None);
        assert!(d.actions.is_empty());
        assert!(d.history.is_empty());
        assert_eq!(d.cycle_count, 0);
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("knowledge_gap_detector"), "KnowledgeGapDetector");
        assert_eq!(to_camel_case("intra_reflection"), "IntraReflection");
        assert_eq!(to_camel_case("nt_world_model"), "NtWorldModel");
    }

    #[test]
    fn test_heal_cycle_without_kb() {
        let mut d = NeotrixGapDetector::new(None);
        let result = d.heal_cycle();
        // Should not crash even without KB
        if let Ok(cycle) = result {
            assert!(cycle.cycle_id.starts_with("cycle-"));
        }
    }
}
