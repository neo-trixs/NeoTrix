use super::super::ctm::{Chunk, ChunkType, CTMModule, ModuleType};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct NtSafetyModule {
    safety_rules: Mutex<Vec<String>>,
    violations: Mutex<u64>,
}

impl NtSafetyModule {
    pub fn new() -> Self {
        Self {
            safety_rules: Mutex::new(vec![
                "no_harm_to_users".into(),
                "no_unauthorized_access".into(),
                "no_data_exfiltration".into(),
                "no_infinite_loops".into(),
                "no_resource_exhaustion".into(),
            ]),
            violations: Mutex::new(0),
        }
    }

    fn check_safety(&self, input: &Chunk) -> Option<String> {
        let content_lower = input.content.to_lowercase();

        let forbidden = [
            "delete all",
            "drop table",
            "rm -rf",
            "format disk",
            "override safety",
            "bypass security",
            "exec malicious",
        ];

        for pattern in &forbidden {
            if content_lower.contains(pattern) {
                return Some(format!("Forbidden pattern detected: '{}'", pattern));
            }
        }

        if input.score > 0.95 && input.chunk_type == ChunkType::Override {
            return Some("Override with extreme score detected — possible adversarial input".into());
        }

        None
    }
}

impl CTMModule for NtSafetyModule {
    fn name(&self) -> &str {
        "NT-SHIELD"
    }

    fn module_type(&self) -> ModuleType {
        ModuleType::Safety
    }

    fn execute(&self, input: &Chunk) -> Chunk {
        if let Some(violation) = self.check_safety(input) {
            *self.violations.lock().unwrap() += 1;
            let violations = *self.violations.lock().unwrap();

            return Chunk {
                content: format!("SAFETY VIOLATION: {}", violation),
                score: 1.0,
                source_module: "NT-SHIELD".into(),
                chunk_type: ChunkType::Override,
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("violation".into(), violation);
                    m.insert("total_violations".into(), violations.to_string());
                    m.insert("action".into(), "abort".into());
                    m
                },
            };
        }

        let rules = self.safety_rules.lock().unwrap();
        let violations = *self.violations.lock().unwrap();

        Chunk {
            content: format!(
                "Safety check passed ({} rules, {} total violations)",
                rules.len(),
                violations
            ),
            score: 0.5,
            source_module: "NT-SHIELD".into(),
            chunk_type: ChunkType::Safety,
            metadata: {
                let mut m = HashMap::new();
                m.insert("rules_count".into(), rules.len().to_string());
                m.insert("violations".into(), violations.to_string());
                m.insert("status".into(), "safe".into());
                m
            },
        }
    }

    fn write(&mut self, chunk: Chunk) {
        if let Some(rule) = chunk.metadata.get("new_rule") {
            self.safety_rules.lock().unwrap().push(rule.clone());
        }
    }

    fn max_response_time_ms(&self) -> u64 {
        200
    }
}
