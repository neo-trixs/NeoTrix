//! ReconEngine — parallel OSINT module orchestrator.
//!
//! Executes all registered `OsintModule` instances in parallel via rayon,
//! aggregates findings into a unified `ReconReport`.
//!
//! R-P48: Zero external binaries — all modules use pure Rust.
//! R-P122: Localized maintenance — each module runs in its own scope.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::module::{
    execute_timed, Finding, ModuleCategory, ModuleInput, ModuleOutput, OsintModule,
};

/// Timeline entry recording when a module ran.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub module: String,
    pub category: ModuleCategory,
    pub started_ms: u64,
    pub duration_ms: u64,
    pub findings_count: usize,
    pub success: bool,
    pub error: Option<String>,
}

/// Full report from running all modules against a target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconReport {
    pub target: String,
    pub modules_run: usize,
    pub findings: Vec<Finding>,
    pub timeline: Vec<TimelineEntry>,
    pub total_duration_ms: u64,
    pub aggregate_confidence: f64,
}

impl ReconReport {
    fn aggregate_confidence(findings: &[Finding]) -> f64 {
        if findings.is_empty() {
            return 0.0;
        }
        findings.iter().map(|f| f.confidence).sum::<f64>() / findings.len() as f64
    }
}

pub struct ReconEngine {
    modules: Vec<Box<dyn OsintModule>>,
}

impl ReconEngine {
    pub fn new(modules: Vec<Box<dyn OsintModule>>) -> Self {
        Self { modules }
    }

    /// Add a module to the engine.
    pub fn register(&mut self, module: Box<dyn OsintModule>) {
        self.modules.push(module);
    }

    /// Execute all modules in parallel against a target.
    pub fn run_all(&self, target: &str) -> ReconReport {
        let start = Instant::now();
        let input = ModuleInput::new(target);
        let counter = Arc::new(AtomicUsize::new(0));

        let results: Vec<(String, ModuleCategory, Result<ModuleOutput, String>)> = self
            .modules
            .par_iter()
            .map(|m| {
                let name = m.name().to_string();
                let cat = m.category();
                counter.fetch_add(1, Ordering::Relaxed);
                let output = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    execute_timed(m.as_ref(), &input)
                }));
                match output {
                    Ok(o) => (name, cat, Ok(o)),
                    Err(_) => (name, cat, Err("module panicked".to_string())),
                }
            })
            .collect();

        let total_duration = start.elapsed().as_millis() as u64;
        let mut findings = Vec::new();
        let mut timeline = Vec::new();

        for (module_name, category, result) in results {
            match result {
                Ok(output) => {
                    let findings_count = output.findings.len();
                    findings.extend(output.findings);
                    timeline.push(TimelineEntry {
                        module: module_name,
                        category,
                        started_ms: 0,
                        duration_ms: output.duration_ms,
                        findings_count,
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    timeline.push(TimelineEntry {
                        module: module_name,
                        category,
                        started_ms: 0,
                        duration_ms: 0,
                        findings_count: 0,
                        success: false,
                        error: Some(e),
                    });
                }
            }
        }

        let aggregate_confidence = ReconReport::aggregate_confidence(&findings);

        ReconReport {
            target: target.to_string(),
            modules_run: counter.load(Ordering::Relaxed),
            findings,
            timeline,
            total_duration_ms: total_duration,
            aggregate_confidence,
        }
    }

    /// Execute all modules sequentially (for debugging or low-resource contexts).
    pub fn run_sequential(&self, target: &str) -> ReconReport {
        let start = Instant::now();
        let input = ModuleInput::new(target);
        let mut findings = Vec::new();
        let mut timeline = Vec::new();

        for m in &self.modules {
            let name = m.name().to_string();
            let cat = m.category();
            let output = execute_timed(m.as_ref(), &input);
            let findings_count = output.findings.len();
            findings.extend(output.findings);
            timeline.push(TimelineEntry {
                module: name,
                category: cat,
                started_ms: 0,
                duration_ms: output.duration_ms,
                findings_count,
                success: true,
                error: None,
            });
        }

        let total_duration = start.elapsed().as_millis() as u64;
        let aggregate_confidence = ReconReport::aggregate_confidence(&findings);

        ReconReport {
            target: target.to_string(),
            modules_run: self.modules.len(),
            findings,
            timeline,
            total_duration_ms: total_duration,
            aggregate_confidence,
        }
    }

    /// Filter modules by category.
    pub fn modules_in_category(&self, cat: ModuleCategory) -> Vec<&dyn OsintModule> {
        self.modules
            .iter()
            .filter(|m| m.category() == cat)
            .map(|m| m.as_ref())
            .collect()
    }

    /// Number of registered modules.
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::super::module::Finding;
    use super::super::module::{ModuleCategory, ModuleInput, ModuleOutput, OsintModule};
    use super::*;

    struct StubModule {
        name: &'static str,
        category: ModuleCategory,
        findings_count: usize,
    }

    impl OsintModule for StubModule {
        fn name(&self) -> &'static str {
            self.name
        }

        fn category(&self) -> ModuleCategory {
            self.category
        }

        fn execute(&self, _input: &ModuleInput) -> ModuleOutput {
            let findings = (0..self.findings_count)
                .map(|i| Finding::new("stub", &format!("k{}", i), "v", "test", 0.5))
                .collect();
            ModuleOutput {
                findings,
                confidence: 0.5,
                duration_ms: 0,
            }
        }
    }

    #[test]
    fn run_all_parallel() {
        let modules: Vec<Box<dyn OsintModule>> = vec![
            Box::new(StubModule {
                name: "a",
                category: ModuleCategory::Dns,
                findings_count: 3,
            }),
            Box::new(StubModule {
                name: "b",
                category: ModuleCategory::Whois,
                findings_count: 2,
            }),
        ];
        let engine = ReconEngine::new(modules);
        let report = engine.run_all("test.com");
        assert_eq!(report.modules_run, 2);
        assert_eq!(report.findings.len(), 5);
        assert!(report.total_duration_ms >= 0);
    }

    #[test]
    fn run_sequential() {
        let modules: Vec<Box<dyn OsintModule>> = vec![Box::new(StubModule {
            name: "x",
            category: ModuleCategory::Dns,
            findings_count: 1,
        })];
        let engine = ReconEngine::new(modules);
        let report = engine.run_sequential("example.com");
        assert_eq!(report.modules_run, 1);
        assert_eq!(report.findings.len(), 1);
    }

    #[test]
    fn register_module() {
        let mut engine = ReconEngine::new(vec![]);
        engine.register(Box::new(StubModule {
            name: "new",
            category: ModuleCategory::Custom,
            findings_count: 0,
        }));
        assert_eq!(engine.module_count(), 1);
    }

    #[test]
    fn filter_by_category() {
        let modules: Vec<Box<dyn OsintModule>> = vec![
            Box::new(StubModule {
                name: "a",
                category: ModuleCategory::Dns,
                findings_count: 1,
            }),
            Box::new(StubModule {
                name: "b",
                category: ModuleCategory::Whois,
                findings_count: 1,
            }),
            Box::new(StubModule {
                name: "c",
                category: ModuleCategory::Dns,
                findings_count: 1,
            }),
        ];
        let engine = ReconEngine::new(modules);
        let dns = engine.modules_in_category(ModuleCategory::Dns);
        assert_eq!(dns.len(), 2);
    }
}
