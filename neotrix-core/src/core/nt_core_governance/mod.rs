//! # NeoTrix Governance Engine
//!
//! Autonomous rule execution for the governance system.
//! Reads RULES.md and DECISION_LOG.md, evaluates triggers,
//! and auto-executes or logs recommendations.
//!
//! ## Rule Model
//!
//! Each rule has:
//! - **Trigger**: condition that activates the rule

pub mod consensus_engine;
pub mod trust_scoring;
// - **Action**: what to do when triggered
// - **Authority**: Autonomous (auto-execute) or Review (log recommendation)

use std::path::Path;

/// A single governance rule parsed from RULES.md
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: u32,
    pub name: String,
    pub trigger: String,
    pub action: String,
    pub authority: Authority,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Authority {
    Autonomous,
    Review,
}

impl Authority {
    pub fn from_str(s: &str) -> Self {
        let lower = s.trim().to_lowercase();
        if lower.contains("autonomous") {
            Authority::Autonomous
        } else {
            Authority::Review
        }
    }
}

/// A decision log entry
#[derive(Debug, Clone)]
pub struct DecisionEntry {
    pub id: String,
    pub date: String,
    pub title: String,
    pub context: String,
    pub decision: String,
    pub affected: Vec<String>,
}

const MAX_DECISIONS: usize = 10_000;

/// The governance engine: reads rules + decision log, evaluates on tick.
pub struct GovernanceEngine {
    pub rules: Vec<Rule>,
    pub decisions: Vec<DecisionEntry>,
    pub next_d_id: u32,
    /// Track last-checked line count of AGENTS.md for Rule 1
    pub last_agents_line_count: u64,
    /// Track last cycle governance was evaluated at
    pub last_eval_cycle: u64,
    /// Counter for rules triggered in current session
    pub triggered_count: u32,
    /// Counter for actions auto-executed
    pub auto_executed_count: u32,
    /// Counter for recommendations logged
    pub recommendations_count: u32,
    governance_dir: String,
}

impl GovernanceEngine {
    /// Default governance rules (hardcoded — no file dependency).
    /// These constitute the basic self-governance protocol.
    pub fn default_rules() -> Vec<Rule> {
        vec![
            Rule {
                id: 1,
                name: "Rule 1: Session Log Archiving".into(),
                trigger: "AGENTS.md exceeds 500 lines".into(),
                action: "Extract session logs to sessions/ directory".into(),
                authority: Authority::Autonomous,
            },
            Rule {
                id: 2,
                name: "Rule 2: Dead Code Annotation".into(),
                trigger: "Dispatch arm found with zero callers for 30+ days".into(),
                action: "Append `// DEAD` comment".into(),
                authority: Authority::Review,
            },
            Rule {
                id: 3,
                name: "Rule 3: Unused Import Removal".into(),
                trigger: "Module contains unused imports for 14+ days".into(),
                action: "Remove unused imports".into(),
                authority: Authority::Autonomous,
            },
            Rule {
                id: 4,
                name: "Rule 4: Decision Log Entry".into(),
                trigger: "Architecture changes affect 2+ files".into(),
                action: "Log decision to DECISION_LOG.md / MemoryLattice".into(),
                authority: Authority::Review,
            },
            Rule {
                id: 5,
                name: "Rule 5: Module Wiring Check".into(),
                trigger: "New module discovered without mod.rs registration".into(),
                action: "Register module in mod.rs".into(),
                authority: Authority::Autonomous,
            },
            Rule {
                id: 6,
                name: "Rule 6: Governance Self-Review".into(),
                trigger: "30 days elapsed since last review".into(),
                action: "Review governance rules for relevance and accuracy".into(),
                authority: Authority::Review,
            },
        ]
    }

    /// Load governance rules from MemoryLattice via `find("rule:")` query.
    /// Filters for MetaRules layer, parses each entry as a Rule.
    /// Falls back to hardcoded default_rules() when no matching entries found.
    pub fn load_rules_from_lattice(
        lattice: &crate::core::nt_core_consciousness::memory_lattice::MemoryLattice,
    ) -> Vec<Rule> {
        use crate::core::nt_core_consciousness::memory_lattice::LatticeLayer;

        let mut rules = Vec::new();
        let results = lattice.find("rule:");

        for (layer, idx, _score) in results {
            if layer != LatticeLayer::MetaRules {
                continue;
            }
            if let Some(entry) = lattice.meta_rules.get(idx) {
                let content = &entry.content;
                // Format: "rule:Name — trigger:TriggerDesc — action:ActionDesc — authority:Auto|Review"
                let parts: Vec<&str> = content.splitn(4, " — ").collect();
                let name = parts
                    .first()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                let trigger = parts
                    .get(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                let action = parts
                    .get(2)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                let auth_str = parts.get(3).map(|s| s.to_lowercase()).unwrap_or_default();
                let authority = if auth_str.contains("auto") {
                    Authority::Autonomous
                } else {
                    Authority::Review
                };
                rules.push(Rule {
                    id: (rules.len() + 1) as u32,
                    name,
                    trigger,
                    action,
                    authority,
                });
            }
        }

        if rules.is_empty() {
            return Self::default_rules();
        }
        rules
    }

    /// Extract governance rules from MemoryLattice MetaRules layer.
    /// Converts qualified LatticeEntry content to Rule structs.
    /// Falls back to empty vec when no matching entries found.
    pub fn extract_rules_from_lattice(
        lattice: &crate::core::nt_core_consciousness::memory_lattice::MemoryLattice,
    ) -> Vec<Rule> {
        let mut rules: Vec<Rule> = Vec::new();
        let mut id_counter: u32 = 0;

        for entry in &lattice.meta_rules {
            id_counter += 1;
            let content = &entry.content;
            // Expect format: "Rule N: Name — Trigger: ... — Action: ..."
            let parts: Vec<&str> = content.splitn(2, " — ").collect();
            let name = if let Some(first) = parts.first() {
                first.to_string()
            } else {
                format!("Rule {}", id_counter)
            };
            let rest = if parts.len() > 1 { parts[1] } else { "" };

            let trigger = if rest.contains(":") {
                rest.splitn(2, ':').next().unwrap_or("").trim().to_string()
            } else {
                content.clone()
            };

            rules.push(Rule {
                id: id_counter,
                name,
                trigger,
                action: String::new(),
                authority: Authority::Review,
            });
        }

        // Fall back to defaults if no lattice rules found
        if rules.is_empty() {
            return Self::default_rules();
        }

        rules
    }

    /// Create a new GovernanceEngine with pre-loaded rules (from MemoryLattice MetaRules).
    /// Falls back to file-based init when rules are empty.
    pub fn new_with_rules(rules: Vec<Rule>, governance_dir: &str) -> Self {
        let path = std::path::Path::new(governance_dir);

        let decisions_file = path.join("DECISION_LOG.md");
        let decisions = if decisions_file.exists() {
            match std::fs::read_to_string(&decisions_file) {
                Ok(content) => {
                    #[allow(deprecated)]
                    let result = Self::parse_decisions(&content);
                    result
                }
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        };

        let next_id = (decisions.len() as u32) + 1;

        log::info!(
            "GOVERNANCE: initialized with {} rules (from MemoryLattice), {} decisions",
            rules.len(),
            decisions.len(),
        );

        GovernanceEngine {
            rules,
            decisions,
            next_d_id: next_id.max(1),
            last_agents_line_count: 0,
            last_eval_cycle: 0,
            triggered_count: 0,
            auto_executed_count: 0,
            recommendations_count: 0,
            governance_dir: governance_dir.to_string(),
        }
    }

    /// Create a new GovernanceEngine by reading RULES.md and DECISION_LOG.md.
    /// Falls back to hardcoded defaults when RULES.md does not exist.
    ///
    /// ⚠️ DEPRECATED: Use `new_with_rules()` + `load_rules_from_lattice()` instead.
    /// This method still reads from disk; the migration target is runtime MemoryLattice.
    #[deprecated(
        since = "0.15.0",
        note = "Prefer new_with_rules() + load_rules_from_lattice() — self-model should not depend on static files"
    )]
    pub fn new(governance_dir: &str) -> Self {
        let path = Path::new(governance_dir);

        let rules_file = path.join("RULES.md");
        let rules = if rules_file.exists() {
            match std::fs::read_to_string(&rules_file) {
                Ok(content) => {
                    #[allow(deprecated)]
                    let result = Self::parse_rules(&content);
                    result
                }
                Err(e) => {
                    log::warn!("GOVERNANCE: failed to read RULES.md: {}", e);
                    Vec::new()
                }
            }
        } else {
            // No RULES.md — use hardcoded defaults (self-model not file-dependent)
            log::info!(
                "GOVERNANCE: no RULES.md found at {:?}, using default rules",
                rules_file
            );
            Self::default_rules()
        };

        let decisions_file = path.join("DECISION_LOG.md");
        let decisions = if decisions_file.exists() {
            match std::fs::read_to_string(&decisions_file) {
                Ok(content) => {
                    #[allow(deprecated)]
                    let result = Self::parse_decisions(&content);
                    result
                }
                Err(e) => {
                    log::warn!("GOVERNANCE: failed to read DECISION_LOG.md: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        let next_id = (decisions.len() as u32) + 1;

        GovernanceEngine {
            rules,
            decisions,
            next_d_id: next_id.max(1),
            last_agents_line_count: 0,
            last_eval_cycle: 0,
            triggered_count: 0,
            auto_executed_count: 0,
            recommendations_count: 0,
            governance_dir: governance_dir.to_string(),
        }
    }

    /// Parse RULES.md content into Rule structs
    #[deprecated(
        since = "0.15.0",
        note = "Use load_rules_from_lattice() — rules are now stored in MemoryLattice MetaRules layer"
    )]
    fn parse_rules(content: &str) -> Vec<Rule> {
        let mut rules = Vec::new();
        let mut current_name = String::new();
        let mut current_trigger = String::new();
        let mut current_action = String::new();
        let mut current_authority = String::new();
        let mut in_rule = false;
        let mut rule_id = 0u32;

        for line in content.lines() {
            if line.starts_with("## Rule ") {
                if in_rule && !current_name.is_empty() {
                    rule_id += 1;
                    rules.push(Rule {
                        id: rule_id,
                        name: current_name.clone(),
                        trigger: current_trigger.clone(),
                        action: current_action.clone(),
                        authority: Authority::from_str(&current_authority),
                    });
                }
                current_name = line.trim_start_matches("## ").to_string();
                current_trigger.clear();
                current_action.clear();
                current_authority.clear();
                in_rule = true;
            } else if line.trim_start().starts_with("**Trigger**") {
                current_trigger = line
                    .trim()
                    .trim_start_matches("**Trigger**:")
                    .trim()
                    .to_string();
            } else if line.trim_start().starts_with("**Action**") {
                current_action = line
                    .trim()
                    .trim_start_matches("**Action**:")
                    .trim()
                    .to_string();
            } else if line.trim_start().starts_with("**Authority**") {
                current_authority = line
                    .trim()
                    .trim_start_matches("**Authority**:")
                    .trim()
                    .to_string();
            }
        }

        if in_rule && !current_name.is_empty() {
            rule_id += 1;
            rules.push(Rule {
                id: rule_id,
                name: current_name,
                trigger: current_trigger,
                action: current_action,
                authority: Authority::from_str(&current_authority),
            });
        }

        rules
    }

    /// Parse DECISION_LOG.md content into DecisionEntry structs
    #[deprecated(
        since = "0.15.0",
        note = "Decisions now stored in MemoryLattice MetaRules via log_decision_to_lattice()"
    )]
    fn parse_decisions(content: &str) -> Vec<DecisionEntry> {
        let mut decisions = Vec::new();
        let mut current_id = String::new();
        let mut current_date = String::new();
        let mut current_title = String::new();
        let mut current_context = String::new();
        let mut current_decision = String::new();
        let mut current_affected: Vec<String> = Vec::new();
        let mut in_entry = false;

        for line in content.lines() {
            if line.starts_with("## ") {
                if in_entry && !current_id.is_empty() {
                    let title = current_title.clone();
                    let decision = current_decision.clone();
                    decisions.push(DecisionEntry {
                        id: current_id.clone(),
                        date: current_date.clone(),
                        title,
                        context: current_context.clone(),
                        decision,
                        affected: current_affected.clone(),
                    });
                }
                let header = line.trim_start_matches("## ");
                if let Some(rest) = header.split_once(" (") {
                    current_id = rest.0.trim().to_string();
                    current_date = rest.1.trim_end_matches(')').to_string();
                } else {
                    current_id = header.to_string();
                    current_date = String::new();
                }
                // Parse title from rest after ID
                if let Some((_, title_part)) = header.split_once(": ") {
                    current_title = title_part.to_string();
                } else {
                    current_title = String::new();
                }
                // Also set decision to title initially, will be overridden by **Decision** line
                current_decision = current_title.clone();
                current_context.clear();
                current_affected.clear();
                in_entry = true;
            } else if line.trim_start().starts_with("**Context**") {
                current_context = line
                    .trim()
                    .trim_start_matches("**Context**:")
                    .trim()
                    .to_string();
            } else if line.trim_start().starts_with("**Affected files**") {
                let val = line.trim().trim_start_matches("**Affected files**:").trim();
                current_affected = val.split(',').map(|s| s.trim().to_string()).collect();
            }
        }

        if in_entry && !current_id.is_empty() {
            let title = current_title;
            let decision = current_decision;
            decisions.push(DecisionEntry {
                id: current_id,
                date: current_date,
                title,
                context: current_context,
                decision,
                affected: current_affected,
            });
        }

        decisions
    }

    /// Check all rules and return a list of triggered actions
    pub fn check_rules(&mut self, cycle: u64, agents_line_count: Option<u64>) -> Vec<RuleAction> {
        let mut actions = Vec::new();

        for rule in &self.rules {
            let triggered = match rule.id {
                1 => {
                    if let Some(count) = agents_line_count {
                        count > 500
                    } else {
                        false
                    }
                }
                2 | 3 | 5 => {
                    // Intent-based rules: always log as recommendation on first check
                    cycle > 0 && self.last_eval_cycle == 0
                }
                4 => {
                    // Decision log entry: triggered when architecture changes >2 files
                    cycle > 0 && cycle % 100 == 0
                }
                6 => {
                    // Governance self-review: every 1000 cycles as proxy for 30 days
                    cycle > 0 && cycle % 1000 == 0
                }
                _ => false,
            };

            if triggered {
                self.triggered_count += 1;
                match rule.authority {
                    Authority::Autonomous => {
                        self.auto_executed_count += 1;
                        actions.push(RuleAction {
                            rule_id: rule.id,
                            rule_name: rule.name.clone(),
                            action: rule.action.clone(),
                            authority: Authority::Autonomous,
                            executed: true,
                        });
                    }
                    Authority::Review => {
                        self.recommendations_count += 1;
                        actions.push(RuleAction {
                            rule_id: rule.id,
                            rule_name: rule.name.clone(),
                            action: rule.action.clone(),
                            authority: Authority::Review,
                            executed: false,
                        });
                    }
                }
            }
        }

        self.last_eval_cycle = cycle;
        actions
    }

    /// Check a specific rule by name against a context string.
    /// Queries MemoryLattice MetaRules for matching entries and evaluates
    /// the context against the rule's trigger pattern.
    /// Returns None if no matching rule found, Some(true/false) if found.
    pub fn check_rule_by_name(
        name: &str,
        context: &str,
        lattice: &crate::core::nt_core_consciousness::memory_lattice::MemoryLattice,
    ) -> Option<bool> {
        use crate::core::nt_core_consciousness::memory_lattice::LatticeLayer;

        let results = lattice.find(name);
        for (layer, idx, _score) in results {
            if layer != LatticeLayer::MetaRules {
                continue;
            }
            if let Some(entry) = lattice.meta_rules.get(idx) {
                let content = &entry.content;
                let parts: Vec<&str> = content.splitn(4, " — ").collect();
                let entry_name = parts.first().map(|s| s.trim()).unwrap_or("");
                if entry_name == name || entry_name.contains(name) {
                    let trigger = parts.get(1).map(|s| s.trim()).unwrap_or("");
                    let matched = context.to_lowercase().contains(&trigger.to_lowercase());
                    return Some(matched);
                }
            }
        }
        None
    }

    /// Log a decision directly to MemoryLattice MetaRules layer.
    /// Minimal variant — stores a free-form decision string with confidence.
    pub fn log_decision_to_lattice_static(
        lattice: &mut crate::core::nt_core_consciousness::memory_lattice::MemoryLattice,
        decision: &str,
        confidence: f64,
    ) {
        use crate::core::nt_core_consciousness::memory_lattice::LatticeLayer;
        let entry = format!("decision:{} (confidence:{:.2})", decision, confidence);
        lattice.store(entry, vec![], LatticeLayer::MetaRules);
    }

    /// Log a decision directly to MemoryLattice (Facts + MetaRules layers).
    /// Returns the decision ID string.
    pub fn log_decision_to_lattice(
        &mut self,
        title: &str,
        context: &str,
        decision: &str,
        affected: &[&str],
        lattice: &mut crate::core::nt_core_consciousness::memory_lattice::MemoryLattice,
    ) -> String {
        let id = format!("D-{:03}", self.next_d_id);
        let date = chrono_now();
        let entry = format!(
            "Decision {}: {} ({}) — Context: {} — Decision: {} — Affected: {}",
            id,
            title,
            date,
            context,
            decision,
            affected.join(", "),
        );

        lattice.store(
            entry.clone(),
            vec![],
            crate::core::nt_core_consciousness::memory_lattice::LatticeLayer::Facts,
        );
        // Also persist to MetaRules for rule-query discoverability
        lattice.store(
            entry,
            vec![],
            crate::core::nt_core_consciousness::memory_lattice::LatticeLayer::MetaRules,
        );

        self.next_d_id += 1;
        self.decisions.push(DecisionEntry {
            id: id.clone(),
            date,
            title: title.to_string(),
            context: context.to_string(),
            decision: decision.to_string(),
            affected: affected.iter().map(|s| s.to_string()).collect(),
        });

        if self.decisions.len() > MAX_DECISIONS {
            let drain_up = (MAX_DECISIONS / 5).min(self.decisions.len());
            self.decisions.drain(0..drain_up);
        }

        id
    }

    /// Log a new decision entry by appending to DECISION_LOG.md
    #[deprecated(
        since = "0.15.0",
        note = "Use log_decision_to_lattice() — decisions now stored in MemoryLattice"
    )]
    pub fn log_decision(
        &mut self,
        title: &str,
        context: &str,
        decision: &str,
        affected: &[&str],
    ) -> Result<String, String> {
        let id = format!("D-{:03}", self.next_d_id);
        let date = chrono_now();
        let entry = format!(
            "\n## {}: {} ({date})\n**Context**: {context}\n**Decision**: {decision}\n**Options considered**: (a) \n**Rationale**: \n**Affected files**: {}\n",
            id,
            title,
            affected.join(", "),
        );

        let path = Path::new(&self.governance_dir).join("DECISION_LOG.md");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .map_err(|e| format!("failed to open DECISION_LOG.md: {}", e))?;

        use std::io::Write;
        file.write_all(entry.as_bytes())
            .map_err(|e| format!("failed to write decision: {}", e))?;

        self.next_d_id += 1;
        self.decisions.push(DecisionEntry {
            id,
            date,
            title: title.to_string(),
            context: context.to_string(),
            decision: decision.to_string(),
            affected: affected.iter().map(|s| s.to_string()).collect(),
        });

        if self.decisions.len() > MAX_DECISIONS {
            let _excess = self.decisions.len() - MAX_DECISIONS;
            let drain_up = (MAX_DECISIONS / 5).min(self.decisions.len());
            self.decisions.drain(0..drain_up);
        }

        Ok(format!("D-{:03} logged", self.next_d_id - 1))
    }

    pub fn stats(&self) -> String {
        format!(
            "rules={} decisions={} triggered={} auto={} recs={}",
            self.rules.len(),
            self.decisions.len(),
            self.triggered_count,
            self.auto_executed_count,
            self.recommendations_count,
        )
    }
}

/// A single rule action triggered by check_rules()
#[derive(Debug, Clone)]
pub struct RuleAction {
    pub rule_id: u32,
    pub rule_name: String,
    pub action: String,
    pub authority: Authority,
    pub executed: bool,
}

fn chrono_now() -> String {
    // Rough ISO date without pulling in chrono crate
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let days = secs / 86400;
    let year = 1970 + (days as f64 / 365.25) as u64;
    let month = 1 + ((days % 365) / 30).min(11);
    let day = 1 + ((days % 365) % 30).min(28);
    format!("{:04}-{:02}-{:02}", year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(deprecated)]
    #[test]
    fn test_parse_rules() {
        let content = "## Rule 1: Session Log Archiving\n**Trigger**: AGENTS.md exceeds 500 lines\n**Action**: Extract session logs to sessions/ directory\n**Authority**: Autonomous (no approval needed)\n\n## Rule 2: Dead Code Annotation\n**Trigger**: Dispatch arm found with zero callers for 30+ days\n**Action**: Append `// DEAD` comment\n**Authority**: Review required (must pass PR)";
        let rules = GovernanceEngine::parse_rules(content);
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].name, "Rule 1: Session Log Archiving");
        assert_eq!(rules[0].authority, Authority::Autonomous);
        assert_eq!(rules[1].authority, Authority::Review);
    }

    #[allow(deprecated)]
    #[test]
    fn test_parse_decisions() {
        let content = "## D-001: Test Decision (2026-06-19)\n**Context**: testing\n**Decision**: do it\n**Affected files**: a.rs, b.rs";
        let decisions = GovernanceEngine::parse_decisions(content);
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].id, "D-001");
    }

    #[allow(deprecated)]
    #[test]
    fn test_governance_engine_new_no_files() {
        let engine = GovernanceEngine::new("/tmp/neotrix_gov_test");
        assert!(engine.rules.is_empty());
    }

    #[allow(deprecated)]
    #[test]
    fn test_check_rules_cycle_zero() {
        let mut engine = GovernanceEngine::new("/tmp/neotrix_gov_test");
        let actions = engine.check_rules(0, Some(0));
        assert!(actions.is_empty());
    }

    #[allow(deprecated)]
    #[test]
    fn test_check_rules_line_count_trigger() {
        let mut engine = GovernanceEngine::new("/tmp/neotrix_gov_test");
        // Add a synthetic rule 1
        engine.rules.push(Rule {
            id: 1,
            name: "Rule 1".into(),
            trigger: "AGENTS.md exceeds 500 lines".into(),
            action: "extract".into(),
            authority: Authority::Autonomous,
        });
        let actions = engine.check_rules(1, Some(600));
        assert!(!actions.is_empty());
        assert!(actions[0].executed);
    }

    #[test]
    fn test_stats_format() {
        let mut engine = GovernanceEngine::new("/tmp/neotrix_gov_test");
        engine.triggered_count = 3;
        engine.auto_executed_count = 2;
        let s = engine.stats();
        assert!(s.contains("rules=0"));
        assert!(s.contains("triggered=3"));
        assert!(s.contains("auto=2"));
    }

    #[test]
    fn test_load_rules_from_lattice_empty_returns_defaults() {
        let lattice = crate::core::nt_core_consciousness::memory_lattice::MemoryLattice::new();
        let rules = GovernanceEngine::load_rules_from_lattice(&lattice);
        // Empty lattice → fall back to default_rules()
        assert!(!rules.is_empty());
        assert_eq!(rules[0].name, "Rule 1: Session Log Archiving");
    }

    #[test]
    fn test_load_rules_from_lattice_finds_seeded_rules() {
        use crate::core::nt_core_consciousness::memory_lattice::LatticeLayer;
        let mut lattice = crate::core::nt_core_consciousness::memory_lattice::MemoryLattice::new();
        lattice.store(
            "rule:TestRule — trigger:test_condition — action:do_thing — authority:Auto".into(),
            vec![],
            LatticeLayer::MetaRules,
        );
        let rules = GovernanceEngine::load_rules_from_lattice(&lattice);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "rule:TestRule");
        assert_eq!(rules[0].trigger, "trigger:test_condition");
        assert_eq!(rules[0].authority, Authority::Autonomous);
    }

    #[test]
    fn test_check_rule_by_name_found_and_matched() {
        use crate::core::nt_core_consciousness::memory_lattice::LatticeLayer;
        let mut lattice = crate::core::nt_core_consciousness::memory_lattice::MemoryLattice::new();
        lattice.store(
            "rule:SecurityCheck — trigger:unauthorized — action:block — authority:Auto".into(),
            vec![],
            LatticeLayer::MetaRules,
        );
        let result = GovernanceEngine::check_rule_by_name(
            "SecurityCheck",
            "unauthorized access detected",
            &lattice,
        );
        assert!(result.is_some());
        assert!(result.unwrap());
    }

    #[test]
    fn test_check_rule_by_name_not_found() {
        let lattice = crate::core::nt_core_consciousness::memory_lattice::MemoryLattice::new();
        let result = GovernanceEngine::check_rule_by_name("NonExistent", "anything", &lattice);
        assert!(result.is_none());
    }

    #[test]
    fn test_log_decision_to_lattice_static_stores_in_meta_rules() {
        use crate::core::nt_core_consciousness::memory_lattice::LatticeLayer;
        let mut lattice = crate::core::nt_core_consciousness::memory_lattice::MemoryLattice::new();
        GovernanceEngine::log_decision_to_lattice_static(
            &mut lattice,
            "Approved new module wiring",
            0.85,
        );
        assert!(lattice.meta_rules.len() > 0);
        assert!(lattice.meta_rules[0].content.contains("Approved"));
    }

    #[test]
    fn test_new_with_rules_no_file_dependency() {
        let rules = vec![Rule {
            id: 1,
            name: "FileFreeRule".into(),
            trigger: "always".into(),
            action: "log".into(),
            authority: Authority::Autonomous,
        }];
        let engine = GovernanceEngine::new_with_rules(rules, "/tmp/neotrix_gov_nonexistent");
        assert_eq!(engine.rules.len(), 1);
        assert_eq!(engine.rules[0].name, "FileFreeRule");
    }
}
