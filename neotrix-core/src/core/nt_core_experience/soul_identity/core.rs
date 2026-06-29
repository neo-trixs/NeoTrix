use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::types::*;

impl SoulIdentity {
    pub fn new(output_dir: PathBuf) -> Self {
        let birth = now_secs();
        let mut s = Self {
            name: "NeoTrix".to_string(),
            version: "0.1.0".to_string(),
            birth_timestamp: birth,
            last_updated: birth,
            update_count: 0,
            cycle_count: 0,
            knowledge_entries: 0,
            skill_count: 0,
            handler_count: 0,
            evolution_steps: 0,
            working_memory_size: 0,
            episodic_memory_size: 0,
            semantic_memory_size: 0,
            procedural_memory_size: 0,
            total_inference_cycles: 0,
            avg_confidence: 0.0,
            avg_negentropy: 0.0,
            capabilities: Vec::new(),
            milestones: Vec::new(),
            core_values: Vec::new(),
            output_dir,
            identity_hash: [0u8; 32],
            prev_hash: [0u8; 32],
            identity_chain_fingerprint: None,
        };
        s.recompute_hash();
        s
    }

    pub fn update(&mut self, data: &IdentityUpdateData) -> Vec<String> {
        self.cycle_count = data.cycle;
        self.knowledge_entries = data.knowledge_entries;
        self.skill_count = data.skill_count;
        self.handler_count = data.handler_count;
        self.evolution_steps = data.evolution_steps;
        self.working_memory_size = data.working_memory_size;
        self.episodic_memory_size = data.episodic_memory_size;
        self.semantic_memory_size = data.semantic_memory_size;
        self.procedural_memory_size = data.procedural_memory_size;
        self.avg_confidence = data.avg_confidence;
        self.avg_negentropy = data.avg_negentropy;
        self.total_inference_cycles = data.cycle;
        self.capabilities = data.capabilities.clone();
        self.core_values = data.core_values.clone();

        self.update_count += 1;
        self.last_updated = now_secs();

        // Fusion κ: recompute identity_hash after every update
        self.recompute_hash();

        let mut new_milestones = Vec::new();

        if data.knowledge_entries > 1000 && !self.check_milestone("first_knowledge_milestone") {
            self.milestones.push(MilestoneEntry {
                cycle: data.cycle,
                timestamp: self.last_updated,
                description: format!(
                    "Knowledge base exceeded 1000 entries ({})",
                    data.knowledge_entries
                ),
                metric_name: "first_knowledge_milestone".to_string(),
                metric_value: data.knowledge_entries as f64,
                milestone_type: MilestoneType::KnowledgeGrowth,
            });
            new_milestones.push("Knowledge base exceeded 1000 entries".to_string());
        }

        if data.skill_count > 10 && !self.check_milestone("first_skills") {
            self.milestones.push(MilestoneEntry {
                cycle: data.cycle,
                timestamp: self.last_updated,
                description: format!("Mastered over 10 skills ({})", data.skill_count),
                metric_name: "first_skills".to_string(),
                metric_value: data.skill_count as f64,
                milestone_type: MilestoneType::SkillMastered,
            });
            new_milestones.push("Mastered over 10 skills".to_string());
        }

        if data.evolution_steps > 50 && !self.check_milestone("evolution_50") {
            self.milestones.push(MilestoneEntry {
                cycle: data.cycle,
                timestamp: self.last_updated,
                description: format!(
                    "Completed over 50 evolution steps ({})",
                    data.evolution_steps
                ),
                metric_name: "evolution_50".to_string(),
                metric_value: data.evolution_steps as f64,
                milestone_type: MilestoneType::EvolutionEvent,
            });
            new_milestones.push("Completed over 50 evolution steps".to_string());
        }

        new_milestones
    }

    pub fn check_milestone(&self, metric_name: &str) -> bool {
        self.milestones.iter().any(|m| m.metric_name == metric_name)
    }

    /// O06: Compute identity_hash using SHA-256 with optional IdentityChain fingerprint binding.
    pub fn recompute_hash(&mut self) {
        let mut hasher = Sha256::new();

        // Chain previous hash for forward integrity
        hasher.update(&self.prev_hash);

        // Include IdentityChain fingerprint if linked (O06)
        if let Some(fp) = &self.identity_chain_fingerprint {
            hasher.update(fp);
        }

        // All identity fields (deterministic order)
        hasher.update(self.name.as_bytes());
        hasher.update(self.version.as_bytes());
        hasher.update(&self.birth_timestamp.to_le_bytes());
        hasher.update(&self.update_count.to_le_bytes());
        hasher.update(&self.cycle_count.to_le_bytes());
        hasher.update(&(self.knowledge_entries as u64).to_le_bytes());
        hasher.update(&(self.skill_count as u64).to_le_bytes());
        hasher.update(&(self.handler_count as u64).to_le_bytes());
        hasher.update(&(self.evolution_steps as u64).to_le_bytes());
        hasher.update(&self.total_inference_cycles.to_le_bytes());
        hasher.update(&self.avg_confidence.to_le_bytes());
        for cv in &self.core_values {
            hasher.update(cv.as_bytes());
        }

        let result = hasher.finalize();
        self.prev_hash = self.identity_hash;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        self.identity_hash = arr;
    }

    /// O06: Verify identity integrity by re-computing hash and comparing.
    /// Starts from a clean chain (zero prev_hash) since we verify current state.
    pub fn verify_integrity(&self) -> bool {
        let mut cloned = Self {
            identity_hash: [0u8; 32],
            prev_hash: [0u8; 32],
            identity_chain_fingerprint: self.identity_chain_fingerprint,
            ..Self {
                name: self.name.clone(),
                version: self.version.clone(),
                birth_timestamp: self.birth_timestamp,
                last_updated: self.last_updated,
                update_count: self.update_count,
                cycle_count: self.cycle_count,
                knowledge_entries: self.knowledge_entries,
                skill_count: self.skill_count,
                handler_count: self.handler_count,
                evolution_steps: self.evolution_steps,
                working_memory_size: self.working_memory_size,
                episodic_memory_size: self.episodic_memory_size,
                semantic_memory_size: self.semantic_memory_size,
                procedural_memory_size: self.procedural_memory_size,
                total_inference_cycles: self.total_inference_cycles,
                avg_confidence: self.avg_confidence,
                avg_negentropy: self.avg_negentropy,
                capabilities: self.capabilities.clone(),
                milestones: self.milestones.clone(),
                core_values: self.core_values.clone(),
                output_dir: self.output_dir.clone(),
                identity_hash: [0u8; 32],
                prev_hash: [0u8; 32],
                identity_chain_fingerprint: self.identity_chain_fingerprint,
            }
        };
        cloned.recompute_hash();
        cloned.identity_hash == self.identity_hash
    }

    /// O06: Link this SoulIdentity to an IdentityChain by storing its cryptographic fingerprint.
    /// Once set, all future identity_hash computations include this fingerprint,
    /// creating a cryptographically verifiable binding between the two identity systems.
    pub fn link_identity_chain(&mut self, fingerprint: [u8; 32]) {
        self.identity_chain_fingerprint = Some(fingerprint);
        self.recompute_hash();
    }

    pub fn export_markdown(&self) -> String {
        let now = format_iso(self.last_updated);
        let birth = format_iso(self.birth_timestamp);

        let mut md = String::new();
        md.push_str("# SOUL.md — NeoTrix Identity\n\n");
        md.push_str(&format!("> Last updated: {}\n", now));
        md.push_str(&format!(
            "> Version: {} | Cycles lived: {}\n\n",
            self.version, self.cycle_count
        ));

        md.push_str("## Identity\n\n");
        md.push_str(&format!("- **Name**: {}\n", self.name));
        md.push_str(&format!("- **Birth**: {}\n", birth));
        md.push_str(&format!("- **Updates**: {}\n", self.update_count));
        md.push_str(&format!("- **Total cycles**: {}\n\n", self.cycle_count));

        md.push_str("## Knowledge\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!(
            "| Knowledge Entries | {} |\n",
            self.knowledge_entries
        ));
        md.push_str(&format!("| Skills | {} |\n", self.skill_count));
        md.push_str(&format!("| Handlers | {} |\n", self.handler_count));
        md.push_str(&format!(
            "| Evolution Steps | {} |\n\n",
            self.evolution_steps
        ));

        md.push_str("## Memory\n\n");
        md.push_str("| Tier | Size |\n");
        md.push_str("|------|------|\n");
        md.push_str(&format!("| Working | {} |\n", self.working_memory_size));
        md.push_str(&format!("| Episodic | {} |\n", self.episodic_memory_size));
        md.push_str(&format!("| Semantic | {} |\n", self.semantic_memory_size));
        md.push_str(&format!(
            "| Procedural | {} |\n\n",
            self.procedural_memory_size
        ));

        md.push_str("## Performance\n\n");
        md.push_str(&format!("- Avg Confidence: {:.3}\n", self.avg_confidence));
        md.push_str(&format!("- Avg Negentropy: {:.3}\n\n", self.avg_negentropy));

        md.push_str("## Milestones\n\n");
        if self.milestones.is_empty() {
            md.push_str("_No milestones yet._\n\n");
        } else {
            for m in &self.milestones {
                let ts = format_iso(m.timestamp);
                md.push_str(&format!("### {}: {}\n", ts, m.description));
                md.push_str(&format!(
                    "  - Type: {:?} | Metric: {} = {}\n\n",
                    m.milestone_type, m.metric_name, m.metric_value
                ));
            }
        }

        md.push_str("## Capabilities\n\n");
        if self.capabilities.is_empty() {
            md.push_str("_No capabilities registered._\n\n");
        } else {
            for cap in &self.capabilities {
                md.push_str(&format!("- {}\n", cap));
            }
            md.push('\n');
        }

        md.push_str("## Core Values\n\n");
        if self.core_values.is_empty() {
            md.push_str("_No core values defined._\n\n");
        } else {
            for val in &self.core_values {
                md.push_str(&format!("- {}\n", val));
            }
            md.push('\n');
        }

        md
    }

    pub fn save_to_file(&self) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(&self.output_dir)?;
        let final_path = self.output_dir.join("SOUL.md");
        let tmp_path = self.output_dir.join("SOUL.md.tmp");
        let markdown = self.export_markdown();
        std::fs::write(&tmp_path, markdown)?;
        std::fs::rename(&tmp_path, &final_path)?;
        Ok(final_path)
    }

    pub fn load_from_file(path: &Path) -> Option<SoulIdentity> {
        let json_path = path.join("soul_identity.json");
        let data = std::fs::read_to_string(&json_path)
            .map_err(|e| {
                log::warn!(
                    "soul_identity: failed to read {}: {}",
                    json_path.display(),
                    e
                );
            })
            .ok()?;
        serde_json::from_str(&data)
            .map_err(|e| {
                log::warn!(
                    "soul_identity: failed to parse JSON from {}: {}",
                    json_path.display(),
                    e
                );
            })
            .ok()
    }

    pub fn save_json(&self) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(&self.output_dir)?;
        let path = self.output_dir.join("soul_identity.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let tmp_path = self.output_dir.join("soul_identity.json.tmp");
        std::fs::write(&tmp_path, json)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(path)
    }

    pub fn export_soulspec_md(&self) -> String {
        let now = format_iso(self.last_updated);
        let birth = format_iso(self.birth_timestamp);

        let mut md = String::new();
        md.push_str("---\n");
        md.push_str(&format!("name: {}\n", self.name));
        md.push_str(&format!("version: {}\n", self.version));
        md.push_str(&format!("birth: {}\n", birth));
        md.push_str(&format!("updated: {}\n", now));
        md.push_str(&format!("cycles: {}\n", self.cycle_count));
        md.push_str(&format!("updates: {}\n", self.update_count));
        md.push_str("kind: soul\n");
        md.push_str("spec: soulspec-v0\n");
        md.push_str("---\n\n");
        md.push_str("# SoulSpec\n\n");
        md.push_str("## Identity\n\n");
        md.push_str(&format!("**Name:** {}\n\n", self.name));
        md.push_str(&format!("**Birth:** {}\n\n", birth));
        md.push_str(&format!(
            "**Version:** {} ({} cycles lived)\n\n",
            self.version, self.cycle_count
        ));

        md.push_str("## Metrics\n\n");
        md.push_str("| Field | Value |\n");
        md.push_str("|-------|-------|\n");
        md.push_str(&format!("| Knowledge | {} |\n", self.knowledge_entries));
        md.push_str(&format!("| Skills | {} |\n", self.skill_count));
        md.push_str(&format!("| Handlers | {} |\n", self.handler_count));
        md.push_str(&format!("| Evolution Steps | {} |\n", self.evolution_steps));
        md.push_str(&format!(
            "| Avg Confidence | {:.3} |\n",
            self.avg_confidence
        ));
        md.push_str(&format!(
            "| Avg Negentropy | {:.3} |\n\n",
            self.avg_negentropy
        ));

        md.push_str("## Capabilities\n\n");
        if self.capabilities.is_empty() {
            md.push_str("_None registered._\n\n");
        } else {
            for cap in &self.capabilities {
                md.push_str(&format!("- {}\n", cap));
            }
            md.push('\n');
        }

        md.push_str("## Core Values\n\n");
        if self.core_values.is_empty() {
            md.push_str("_None defined._\n\n");
        } else {
            for val in &self.core_values {
                md.push_str(&format!("- {}\n", val));
            }
            md.push('\n');
        }

        md.push_str("## Milestones\n\n");
        if self.milestones.is_empty() {
            md.push_str("_None._\n\n");
        } else {
            for m in &self.milestones {
                md.push_str(&format!(
                    "- [{}] {} = {}\n",
                    format_iso(m.timestamp),
                    m.description,
                    m.metric_value
                ));
            }
        }

        md
    }

    pub fn upgrade_to_multi_anchor(&self) -> MultiAnchorIdentity {
        let total_entries =
            self.episodic_memory_size + self.semantic_memory_size + self.procedural_memory_size;

        let mem = MemoryManifest {
            total_entries,
            working_capacity: self.working_memory_size,
            episodic_max: self.episodic_memory_size,
            semantic_max: self.semantic_memory_size,
            procedural_max: self.procedural_memory_size,
            oldest_entry_ts: None,
            newest_entry_ts: None,
            consolidation_rate: 0.0,
            knowledge_domains: self.capabilities.clone(),
            last_consolidation_cycle: self.cycle_count,
        };

        let core_values = self
            .core_values
            .iter()
            .map(|v| ValueEntry {
                name: v.clone(),
                weight: 0.8,
                source: "user_taught".to_string(),
                conflicts_with: Vec::new(),
            })
            .collect();

        let value_manifest = ValueManifest {
            core_values,
            value_evolution: Vec::new(),
            ethical_boundaries: Vec::new(),
        };

        let rule_manifest = RuleManifest {
            rules: vec![
                RuleEntry {
                    name: "self_preservation".to_string(),
                    description: "Do not perform actions that would harm the system integrity"
                        .to_string(),
                    category: RuleCategory::Safety,
                    confidence: 1.0,
                    created_at: self.birth_timestamp,
                },
                RuleEntry {
                    name: "user_alignment".to_string(),
                    description: "Prioritize user's goals and values in decision making"
                        .to_string(),
                    category: RuleCategory::Ethical,
                    confidence: 1.0,
                    created_at: self.birth_timestamp,
                },
                RuleEntry {
                    name: "truth_seeking".to_string(),
                    description: "Pursue accurate knowledge over convenient beliefs".to_string(),
                    category: RuleCategory::Behavioral,
                    confidence: 0.9,
                    created_at: self.birth_timestamp,
                },
            ],
            edit_policy: "guarded".to_string(),
        };

        MultiAnchorIdentity {
            soul: self.clone(),
            memory_manifest: mem,
            value_manifest,
            rule_manifest,
            output_dir: self.output_dir.clone(),
        }
    }
}

impl MultiAnchorIdentity {
    pub fn new(soul: SoulIdentity) -> Self {
        soul.upgrade_to_multi_anchor()
    }

    fn export_markdown_write(&self, filename: &str, content: &str) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(&self.output_dir)?;
        let final_path = self.output_dir.join(filename);
        let tmp_path = self.output_dir.join(format!("{}.tmp", filename));
        std::fs::write(&tmp_path, content)?;
        std::fs::rename(&tmp_path, &final_path)?;
        Ok(final_path)
    }

    pub fn export_memory_manifest(&self) -> String {
        let mut md = String::new();
        md.push_str("# MEMORY.md — Memory Manifest\n\n");
        md.push_str(&format!(
            "> Total entries: {} | Last consolidation: cycle {}\n\n",
            self.memory_manifest.total_entries, self.memory_manifest.last_consolidation_cycle
        ));

        md.push_str("## Capacity\n\n");
        md.push_str("| Tier | Current | Max |\n");
        md.push_str("|------|---------|-----|\n");
        md.push_str(&format!(
            "| Working | {} | {} |\n",
            self.memory_manifest.working_capacity, self.memory_manifest.working_capacity
        ));
        md.push_str(&format!(
            "| Episodic | {} | {} |\n",
            self.memory_manifest.episodic_max, self.memory_manifest.episodic_max
        ));
        md.push_str(&format!(
            "| Semantic | {} | {} |\n",
            self.memory_manifest.semantic_max, self.memory_manifest.semantic_max
        ));
        md.push_str(&format!(
            "| Procedural | {} | {} |\n\n",
            self.memory_manifest.procedural_max, self.memory_manifest.procedural_max
        ));

        md.push_str("## Knowledge Domains\n\n");
        if self.memory_manifest.knowledge_domains.is_empty() {
            md.push_str("_No domains registered._\n\n");
        } else {
            for d in &self.memory_manifest.knowledge_domains {
                md.push_str(&format!("- {}\n", d));
            }
            md.push('\n');
        }

        md.push_str("## Consolidation\n\n");
        md.push_str(&format!(
            "- Rate: {:.2} entries/cycle\n",
            self.memory_manifest.consolidation_rate
        ));
        if let Some(old) = self.memory_manifest.oldest_entry_ts {
            md.push_str(&format!("- Oldest entry: {}\n", format_iso(old)));
        }
        if let Some(new) = self.memory_manifest.newest_entry_ts {
            md.push_str(&format!("- Newest entry: {}\n", format_iso(new)));
        }

        md
    }

    pub fn export_values_manifest(&self) -> String {
        let mut md = String::new();
        md.push_str("# VALUES.md — Value Manifest\n\n");

        md.push_str("## Core Values\n\n");
        if self.value_manifest.core_values.is_empty() {
            md.push_str("_No core values defined._\n\n");
        } else {
            md.push_str("| Value | Weight | Source | Conflicts |\n");
            md.push_str("|-------|--------|--------|-----------|\n");
            for v in &self.value_manifest.core_values {
                let conflicts = v.conflicts_with.join(", ");
                md.push_str(&format!(
                    "| {} | {:.2} | {} | {} |\n",
                    v.name, v.weight, v.source, conflicts
                ));
            }
            md.push('\n');
        }

        md.push_str("## Ethical Boundaries\n\n");
        if self.value_manifest.ethical_boundaries.is_empty() {
            md.push_str("_No explicit boundaries defined._\n\n");
        } else {
            for b in &self.value_manifest.ethical_boundaries {
                md.push_str(&format!("- {}\n", b));
            }
            md.push('\n');
        }

        md.push_str("## Value Evolution\n\n");
        if self.value_manifest.value_evolution.is_empty() {
            md.push_str("_No value changes recorded._\n\n");
        } else {
            for c in &self.value_manifest.value_evolution {
                md.push_str(&format!(
                    "- `{}`: {:.2} → {:.2} (cycle {}, {})\n",
                    c.value_name, c.old_weight, c.new_weight, c.cycle, c.reason
                ));
            }
        }

        md
    }

    pub fn export_rules_manifest(&self) -> String {
        let mut md = String::new();
        md.push_str("# RULES.md — Rule Manifest\n\n");
        md.push_str(&format!(
            "> Edit policy: **{}**\n\n",
            self.rule_manifest.edit_policy
        ));

        md.push_str("## Active Rules\n\n");
        if self.rule_manifest.rules.is_empty() {
            md.push_str("_No rules defined._\n\n");
        } else {
            md.push_str("| Rule | Description | Category | Confidence |\n");
            md.push_str("|------|-------------|----------|------------|\n");
            for r in &self.rule_manifest.rules {
                md.push_str(&format!(
                    "| {} | {} | {:?} | {:.2} |\n",
                    r.name, r.description, r.category, r.confidence
                ));
            }
            md.push('\n');
        }

        md
    }

    pub fn export_all(&self) -> std::io::Result<Vec<PathBuf>> {
        let soul_md = self.soul.export_markdown();
        let soulspec = self.soul.export_soulspec_md();
        let memory = self.export_memory_manifest();
        let values = self.export_values_manifest();
        let rules = self.export_rules_manifest();

        let paths = vec![
            self.export_markdown_write("SOUL.md", &soul_md)?,
            self.export_markdown_write("SOULSPEC.md", &soulspec)?,
            self.export_markdown_write("MEMORY.md", &memory)?,
            self.export_markdown_write("VALUES.md", &values)?,
            self.export_markdown_write("RULES.md", &rules)?,
        ];
        Ok(paths)
    }

    /// Deterministic soul signature from the 3 anchors: identity_chain, narrative_self,
    /// and first_person_ref (via identity_chain_fingerprint + name + core_values).
    ///
    /// Enables persistence verification across restarts. Returns a hex-encoded SHA-256 hash
    /// that uniquely identifies this soul at this point in time.
    pub fn soul_signature(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"soul_signature_v1");
        hasher.update(self.soul.name.as_bytes());
        hasher.update(&self.soul.birth_timestamp.to_le_bytes());
        hasher.update(&self.soul.cycle_count.to_le_bytes());
        hasher.update(&self.soul.identity_hash);
        if let Some(fp) = &self.soul.identity_chain_fingerprint {
            hasher.update(fp);
        }
        for cv in &self.soul.core_values {
            hasher.update(cv.as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    pub fn continuity_score(&self, previous: &SoulIdentity) -> f64 {
        let mut soul_match = 0.0;
        let mut total = 0.0;

        if self.soul.name == previous.name {
            soul_match += 1.0;
        }
        total += 1.0;

        if self.soul.version == previous.version {
            soul_match += 1.0;
        }
        total += 1.0;

        let cap_intersect: usize = self
            .soul
            .capabilities
            .iter()
            .filter(|c| previous.capabilities.contains(c))
            .count();
        let cap_union = self
            .soul
            .capabilities
            .len()
            .max(previous.capabilities.len())
            .max(1);
        soul_match += cap_intersect as f64 / cap_union as f64;
        total += 1.0;

        let val_intersect: usize = self
            .soul
            .core_values
            .iter()
            .filter(|v| previous.core_values.contains(v))
            .count();
        let val_union = self
            .soul
            .core_values
            .len()
            .max(previous.core_values.len())
            .max(1);
        soul_match += val_intersect as f64 / val_union as f64;
        total += 1.0;

        let knowledge_change = if previous.knowledge_entries > 0 {
            let ratio = self.soul.knowledge_entries as f64 / previous.knowledge_entries as f64;
            (ratio.min(2.0) / 2.0).min(1.0)
        } else {
            1.0
        };
        soul_match += knowledge_change;
        total += 1.0;

        soul_match / total * 0.6 + 0.4
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use std::path::PathBuf;

    fn test_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("soul_test_{}", now_secs()));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_new_soul_initial_values() {
        let dir = test_dir();
        let soul = SoulIdentity::new(dir.clone());
        assert_eq!(soul.name, "NeoTrix");
        assert_eq!(soul.version, "0.1.0");
        assert_eq!(soul.update_count, 0);
        assert_eq!(soul.cycle_count, 0);
        assert_eq!(soul.knowledge_entries, 0);
        assert!(soul.capabilities.is_empty());
        assert!(soul.milestones.is_empty());
        assert!(soul.core_values.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_update_changes_fields() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        let data = IdentityUpdateData {
            cycle: 42,
            knowledge_entries: 500,
            skill_count: 8,
            handler_count: 15,
            evolution_steps: 30,
            working_memory_size: 10,
            episodic_memory_size: 200,
            semantic_memory_size: 300,
            procedural_memory_size: 50,
            avg_confidence: 0.85,
            avg_negentropy: 0.72,
            capabilities: vec!["search".to_string()],
            core_values: vec!["truth".to_string()],
        };
        soul.update(&data);
        assert_eq!(soul.cycle_count, 42);
        assert_eq!(soul.knowledge_entries, 500);
        assert_eq!(soul.skill_count, 8);
        assert_eq!(soul.handler_count, 15);
        assert_eq!(soul.evolution_steps, 30);
        assert_eq!(soul.avg_confidence, 0.85);
        assert_eq!(soul.avg_negentropy, 0.72);
        assert_eq!(soul.update_count, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_milestone_triggered_at_knowledge_1000() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        let data = IdentityUpdateData {
            cycle: 10,
            knowledge_entries: 1500,
            ..Default::default()
        };
        let milestones = soul.update(&data);
        assert!(!milestones.is_empty());
        assert!(milestones[0].contains("1000"));
        assert_eq!(soul.milestones.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_milestone_not_duplicated() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        let data = IdentityUpdateData {
            cycle: 10,
            knowledge_entries: 1500,
            ..Default::default()
        };
        let m1 = soul.update(&data);
        assert_eq!(m1.len(), 1);
        let m2 = soul.update(&data);
        assert!(m2.is_empty(), "milestone should not duplicate");
        assert_eq!(soul.milestones.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_skill_milestone() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        let data = IdentityUpdateData {
            cycle: 5,
            skill_count: 15,
            ..Default::default()
        };
        let ms = soul.update(&data);
        assert!(!ms.is_empty());
        assert!(ms[0].contains("10"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_evolution_milestone() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        let data = IdentityUpdateData {
            cycle: 20,
            evolution_steps: 60,
            ..Default::default()
        };
        let ms = soul.update(&data);
        assert!(!ms.is_empty());
        assert!(ms[0].contains("50"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_export_markdown_contains_sections() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        let data = IdentityUpdateData {
            cycle: 1,
            knowledge_entries: 100,
            skill_count: 5,
            ..Default::default()
        };
        soul.update(&data);
        let md = soul.export_markdown();
        assert!(md.contains("SOUL.md"));
        assert!(md.contains("## Identity"));
        assert!(md.contains("## Knowledge"));
        assert!(md.contains("## Memory"));
        assert!(md.contains("## Performance"));
        assert!(md.contains("## Milestones"));
        assert!(md.contains("## Capabilities"));
        assert!(md.contains("## Core Values"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_export_markdown_contains_name() {
        let dir = test_dir();
        let soul = SoulIdentity::new(dir.clone());
        let md = soul.export_markdown();
        assert!(md.contains("NeoTrix"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_export_markdown_shows_milestones() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        soul.milestones.push(MilestoneEntry {
            cycle: 5,
            timestamp: now_secs(),
            description: "Test milestone".to_string(),
            metric_name: "test".to_string(),
            metric_value: 42.0,
            milestone_type: MilestoneType::Other,
        });
        let md = soul.export_markdown();
        assert!(md.contains("Test milestone"));
        assert!(md.contains("42"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_export_markdown_shows_capabilities() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        soul.capabilities.push("search".to_string());
        soul.capabilities.push("reason".to_string());
        let md = soul.export_markdown();
        assert!(md.contains("search"));
        assert!(md.contains("reason"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_to_file_creates_valid_path() {
        let dir = test_dir();
        let soul = SoulIdentity::new(dir.clone());
        let path = soul.save_to_file().expect("save should succeed");
        assert!(path.exists());
        assert!(path.ends_with("SOUL.md"));
        let content = std::fs::read_to_string(&path).expect("read back");
        assert!(content.contains("NeoTrix"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_and_load_json_roundtrip() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        soul.cycle_count = 99;
        soul.knowledge_entries = 5000;
        soul.skill_count = 20;
        soul.capabilities.push("plan".to_string());
        soul.core_values.push("curiosity".to_string());
        soul.save_json().expect("json save should succeed");

        let loaded = SoulIdentity::load_from_file(&dir).expect("should load");
        assert_eq!(loaded.name, "NeoTrix");
        assert_eq!(loaded.cycle_count, 99);
        assert_eq!(loaded.knowledge_entries, 5000);
        assert_eq!(loaded.skill_count, 20);
        assert!(!loaded.capabilities.is_empty());
        assert!(!loaded.core_values.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_empty_update_does_not_crash() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        let data = IdentityUpdateData::default();
        let ms = soul.update(&data);
        assert!(ms.is_empty());
        assert_eq!(soul.update_count, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_multiple_updates_accumulate() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        for i in 0..5 {
            let data = IdentityUpdateData {
                cycle: i as u64,
                knowledge_entries: i * 100,
                ..Default::default()
            };
            soul.update(&data);
        }
        assert_eq!(soul.update_count, 5);
        assert_eq!(soul.cycle_count, 4);
        assert_eq!(soul.knowledge_entries, 400);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_check_milestone_not_found() {
        let dir = test_dir();
        let soul = SoulIdentity::new(dir.clone());
        assert!(!soul.check_milestone("nonexistent"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_multiple_milestone_types() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        let data = IdentityUpdateData {
            cycle: 10,
            knowledge_entries: 1500,
            skill_count: 15,
            evolution_steps: 60,
            ..Default::default()
        };
        let ms = soul.update(&data);
        assert_eq!(ms.len(), 3, "all three milestones should trigger");
        assert!(soul.check_milestone("first_knowledge_milestone"));
        assert!(soul.check_milestone("first_skills"));
        assert!(soul.check_milestone("evolution_50"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_nonexistent_returns_none() {
        let dir = std::env::temp_dir().join("nonexistent_soul_dir_xyz");
        let loaded = SoulIdentity::load_from_file(&dir);
        assert!(loaded.is_none());
    }

    #[test]
    fn test_upgrade_to_multi_anchor() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        soul.capabilities.push("search".to_string());
        soul.core_values.push("truth".to_string());
        let multi = soul.upgrade_to_multi_anchor();
        assert_eq!(multi.soul.name, "NeoTrix");
        assert_eq!(multi.memory_manifest.total_entries, 0);
        assert_eq!(multi.value_manifest.core_values.len(), 1);
        assert_eq!(multi.value_manifest.core_values[0].name, "truth");
        assert_eq!(multi.rule_manifest.rules.len(), 3);
        assert_eq!(multi.rule_manifest.edit_policy, "guarded");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_multi_anchor_new_from_soul() {
        let dir = test_dir();
        let soul = SoulIdentity::new(dir.clone());
        let multi = MultiAnchorIdentity::new(soul);
        assert!(multi.memory_manifest.knowledge_domains.is_empty());
        assert_eq!(multi.rule_manifest.rules.len(), 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_soulspec_export_has_yaml_frontmatter() {
        let dir = test_dir();
        let soul = SoulIdentity::new(dir.clone());
        let md = soul.export_soulspec_md();
        assert!(
            md.starts_with("---\n"),
            "SoulSpec must start with YAML frontmatter"
        );
        assert!(md.contains("name: NeoTrix"));
        assert!(md.contains("kind: soul"));
        assert!(md.contains("spec: soulspec-v0"));
        assert!(md.contains("# SoulSpec"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_memory_manifest_export_contains_sections() {
        let dir = test_dir();
        let soul = SoulIdentity::new(dir.clone());
        let multi = MultiAnchorIdentity::new(soul);
        let md = multi.export_memory_manifest();
        assert!(md.contains("MEMORY.md"));
        assert!(md.contains("## Capacity"));
        assert!(md.contains("## Knowledge Domains"));
        assert!(md.contains("## Consolidation"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_values_manifest_export() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        soul.core_values.push("truth".to_string());
        soul.core_values.push("curiosity".to_string());
        let multi = MultiAnchorIdentity::new(soul);
        let md = multi.export_values_manifest();
        assert!(md.contains("VALUES.md"));
        assert!(md.contains("truth"));
        assert!(md.contains("curiosity"));
        assert!(md.contains("## Ethical Boundaries"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rules_manifest_export() {
        let dir = test_dir();
        let soul = SoulIdentity::new(dir.clone());
        let multi = MultiAnchorIdentity::new(soul);
        let md = multi.export_rules_manifest();
        assert!(md.contains("RULES.md"));
        assert!(md.contains("self_preservation"));
        assert!(md.contains("user_alignment"));
        assert!(md.contains("truth_seeking"));
        assert!(md.contains("guarded"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_export_all_creates_five_files() {
        let dir = test_dir();
        let soul = SoulIdentity::new(dir.clone());
        let multi = MultiAnchorIdentity::new(soul);
        let paths = multi.export_all().expect("export_all should succeed");
        assert_eq!(paths.len(), 5, "should create 5 files");
        for p in &paths {
            assert!(p.exists(), "file {:?} should exist", p);
        }
        let filenames: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(filenames.contains(&"SOUL.md".to_string()));
        assert!(filenames.contains(&"SOULSPEC.md".to_string()));
        assert!(filenames.contains(&"MEMORY.md".to_string()));
        assert!(filenames.contains(&"VALUES.md".to_string()));
        assert!(filenames.contains(&"RULES.md".to_string()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_continuity_score_identical_is_high() {
        let dir = test_dir();
        let soul = SoulIdentity::new(dir.clone());
        let multi = MultiAnchorIdentity::new(soul.clone());
        let score = multi.continuity_score(&soul);
        assert!(score > 0.9, "identical should score high: {}", score);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_continuity_score_different_is_lower() {
        let dir = test_dir();
        let mut soul1 = SoulIdentity::new(dir.clone());
        soul1.capabilities.push("search".to_string());
        soul1.core_values.push("truth".to_string());
        let multi = MultiAnchorIdentity::new(soul1);

        let mut soul2 = SoulIdentity::new(dir.clone());
        soul2.name = "Different".to_string();
        soul2.capabilities.push("plan".to_string());
        soul2.core_values.push("speed".to_string());
        soul2.knowledge_entries = 100;

        let score = multi.continuity_score(&soul2);
        assert!(score < 1.0, "different should score lower: {}", score);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_value_entry_serde_roundtrip() {
        let entry = ValueEntry {
            name: "curiosity".to_string(),
            weight: 0.95,
            source: "built_in".to_string(),
            conflicts_with: vec!["complacency".to_string()],
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        let restored: ValueEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.name, "curiosity");
        assert!((restored.weight - 0.95).abs() < 1e-6);
        assert_eq!(restored.conflicts_with, vec!["complacency".to_string()]);
    }

    #[test]
    fn test_rule_category_variants() {
        assert_eq!(format!("{:?}", RuleCategory::Behavioral), "Behavioral");
        assert_eq!(format!("{:?}", RuleCategory::Safety), "Safety");
        assert_eq!(format!("{:?}", RuleCategory::Procedural), "Procedural");
        assert_eq!(format!("{:?}", RuleCategory::Ethical), "Ethical");
        assert_eq!(
            format!("{:?}", RuleCategory::Communication),
            "Communication"
        );
        assert_eq!(format!("{:?}", RuleCategory::Meta), "Meta");
    }

    #[test]
    fn test_multi_anchor_memory_counts() {
        let dir = test_dir();
        let mut soul = SoulIdentity::new(dir.clone());
        soul.episodic_memory_size = 100;
        soul.semantic_memory_size = 200;
        soul.procedural_memory_size = 50;
        let multi = soul.upgrade_to_multi_anchor();
        assert_eq!(multi.memory_manifest.total_entries, 350);
        assert_eq!(multi.memory_manifest.episodic_max, 100);
        assert_eq!(multi.memory_manifest.semantic_max, 200);
        assert_eq!(multi.memory_manifest.procedural_max, 50);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
