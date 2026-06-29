use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum GateDecision {
    Allow,
    Reject(String),
    Escalate(String),
}

#[derive(Debug, Clone)]
pub struct GateResult {
    pub layer: usize,
    pub layer_name: String,
    pub decision: GateDecision,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct EvolutionProposal {
    pub id: u64,
    pub subsystem: String,
    pub diff_preview: String,
    pub diff_size_bytes: usize,
    pub risk_score: f64,
    pub rationale: String,
    pub affected_files: Vec<String>,
    pub rollback_plan: String,
}

#[derive(Clone)]
pub struct EvolutionGatekeeper {
    pub proposals: Vec<EvolutionProposal>,
    pub counter: u64,
    pub allowlisted_paths: HashSet<String>,
    pub denylisted_patterns: Vec<String>,
    pub max_diff_size: usize,
    pub gate_results: Vec<Vec<GateResult>>,
}

impl EvolutionGatekeeper {
    pub fn new() -> Self {
        let mut allowlisted_paths = HashSet::new();
        allowlisted_paths.insert("neotrix-core/src".to_string());
        allowlisted_paths.insert("crates".to_string());

        EvolutionGatekeeper {
            proposals: Vec::new(),
            counter: 0,
            allowlisted_paths,
            denylisted_patterns: vec![
                "SELF.md".to_string(),
                "RULES.md".to_string(),
                "AGENTS.md".to_string(),
                "Cargo.lock".to_string(),
                ".env".to_string(),
                "credentials".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "password".to_string(),
            ],
            max_diff_size: 50000,
            gate_results: Vec::new(),
        }
    }

    pub fn submit_proposal(
        &mut self,
        subsystem: &str,
        diff_preview: &str,
        rationale: &str,
        affected_files: Vec<String>,
    ) -> u64 {
        self.counter += 1;
        let proposal = EvolutionProposal {
            id: self.counter,
            subsystem: subsystem.to_string(),
            diff_preview: diff_preview.to_string(),
            diff_size_bytes: diff_preview.len(),
            risk_score: 0.0,
            rationale: rationale.to_string(),
            affected_files,
            rollback_plan: "git revert HEAD".to_string(),
        };
        self.proposals.push(proposal);
        self.counter
    }

    pub fn run_gates(&mut self, proposal_id: u64) -> Vec<GateResult> {
        let proposal = match self.proposals.iter().find(|p| p.id == proposal_id) {
            Some(p) => p.clone(),
            None => return vec![],
        };

        let mut results: Vec<GateResult> = Vec::with_capacity(5);

        // Layer 1: Path allowlist/denylist
        let t0 = std::time::Instant::now();
        let blocked_paths: Vec<&String> = proposal
            .affected_files
            .iter()
            .filter(|f| {
                self.denylisted_patterns
                    .iter()
                    .any(|d| f.contains(d.as_str()))
            })
            .collect();
        let allowed = proposal.affected_files.iter().any(|f| {
            self.allowlisted_paths
                .iter()
                .any(|a| f.starts_with(a.as_str()))
        });
        let decision1 = if !blocked_paths.is_empty() {
            GateDecision::Reject(format!("files in denylist: {:?}", blocked_paths))
        } else if !allowed && !proposal.affected_files.is_empty() {
            GateDecision::Escalate("no files in allowlist".to_string())
        } else {
            GateDecision::Allow
        };
        results.push(GateResult {
            layer: 1,
            layer_name: "Path Allowlist".to_string(),
            decision: decision1,
            duration_ms: t0.elapsed().as_millis() as u64,
        });

        // Layer 2: Diff size limit
        let t1 = std::time::Instant::now();
        let decision2 = if proposal.diff_size_bytes > self.max_diff_size {
            GateDecision::Reject(format!(
                "diff size {} exceeds limit {}",
                proposal.diff_size_bytes, self.max_diff_size
            ))
        } else {
            GateDecision::Allow
        };
        results.push(GateResult {
            layer: 2,
            layer_name: "Diff Size Limit".to_string(),
            decision: decision2,
            duration_ms: t1.elapsed().as_millis() as u64,
        });

        // Layer 3: Secret scan
        let t2 = std::time::Instant::now();
        let mut found_secrets: Vec<String> = Vec::new();
        let lower = proposal.diff_preview.to_lowercase();
        if lower.contains("api_key") || lower.contains("api-key") || lower.contains("apikey") {
            found_secrets.push("API key pattern".to_string());
        }
        if lower.contains("sk-")
            && proposal
                .diff_preview
                .bytes()
                .filter(|&b| b.is_ascii_alphanumeric() || b == b'-')
                .count()
                > 25
        {
            found_secrets.push("OpenAI key pattern".to_string());
        }
        if lower.contains("ghp_") && proposal.diff_preview.contains("ghp_") {
            found_secrets.push("GitHub PAT pattern".to_string());
        }
        if lower.contains("begin private key") || lower.contains("begin rsa private key") {
            found_secrets.push("private key pattern".to_string());
        }
        if lower.contains("akia") {
            found_secrets.push("AWS key pattern".to_string());
        }
        if proposal.diff_preview.contains(".") && proposal.diff_preview.matches('.').count() >= 3 {
            let parts: Vec<&str> = proposal.diff_preview.split('.').collect();
            if parts.len() >= 3 && parts[0].len() > 10 && parts[1].len() > 10 {
                found_secrets.push("JWT pattern".to_string());
            }
        }
        let decision3 = if !found_secrets.is_empty() {
            GateDecision::Reject(format!("secrets detected: {:?}", found_secrets))
        } else {
            GateDecision::Allow
        };
        results.push(GateResult {
            layer: 3,
            layer_name: "Secret Scan".to_string(),
            decision: decision3,
            duration_ms: t2.elapsed().as_millis() as u64,
        });

        // Layer 4: Code pattern detection
        let t3 = std::time::Instant::now();
        let dangerous_patterns: Vec<(&str, &str)> = vec![
            ("std::process::Command", "shell execution"),
            ("std::fs::remove_dir_all", "recursive delete"),
            ("open(", "file open"),
            ("exec(", "code execution"),
            ("panic!(\"", "bare panic"),
            ("unwrap()", "unwrap in diff"),
        ];
        let mut found_dangerous: Vec<String> = Vec::new();
        for (pattern, desc) in &dangerous_patterns {
            if proposal.diff_preview.contains(pattern) {
                found_dangerous.push(desc.to_string());
            }
        }
        let decision4 = if !found_dangerous.is_empty() {
            GateDecision::Escalate(format!("dangerous patterns: {:?}", found_dangerous))
        } else {
            GateDecision::Allow
        };
        results.push(GateResult {
            layer: 4,
            layer_name: "Code Pattern Detection".to_string(),
            decision: decision4,
            duration_ms: t3.elapsed().as_millis() as u64,
        });

        // Layer 5: Risk score threshold
        let t4 = std::time::Instant::now();
        let has_allow = results
            .iter()
            .any(|r| matches!(r.decision, GateDecision::Allow));
        let has_reject = results
            .iter()
            .any(|r| matches!(r.decision, GateDecision::Reject(_)));
        let has_escalate = results
            .iter()
            .any(|r| matches!(r.decision, GateDecision::Escalate(_)));
        let decision5 = if has_reject {
            GateDecision::Reject("previous layer rejected".to_string())
        } else if has_escalate {
            GateDecision::Escalate("previous layer escalated".to_string())
        } else if has_allow {
            GateDecision::Allow
        } else {
            GateDecision::Escalate("no clear decision".to_string())
        };
        results.push(GateResult {
            layer: 5,
            layer_name: "Aggregate Decision".to_string(),
            decision: decision5,
            duration_ms: t4.elapsed().as_millis() as u64,
        });

        self.gate_results.push(results.clone());
        results
    }

    pub fn is_allowed(&self, proposal_id: u64) -> bool {
        let has_proposal = self.proposals.iter().any(|p| p.id == proposal_id);
        if !has_proposal {
            return false;
        }
        self.gate_results.iter().any(|results| {
            results
                .last()
                .map(|g| matches!(g.decision, GateDecision::Allow))
                .unwrap_or(false)
        })
    }

    pub fn summary(&self) -> String {
        let total = self.gate_results.len();
        let allowed = self
            .gate_results
            .iter()
            .filter(|r| {
                r.last()
                    .map(|g| matches!(g.decision, GateDecision::Allow))
                    .unwrap_or(false)
            })
            .count();
        format!(
            "EvolutionGatekeeper: {}/{} proposals allowed, {}/{} rejected/escalated",
            allowed,
            total,
            total - allowed,
            total
        )
    }
}
