#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::self_evolution_loop::{MutationOp, SelfEvolutionStep};

/// A crystallized skill — a mutation that has proven itself repeatedly and
/// been frozen into a reusable Ne source artifact.
#[derive(Debug, Clone)]
pub struct CrystallizedSkill {
    /// Unique name (derived from the mutation).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// The Ne source code that implements this skill.
    pub ne_source: String,
    /// How many times this skill has been invoked.
    pub invocation_count: u64,
    /// Rolling average score across invocations.
    pub avg_score: f64,
    /// Free-form tags for search and categorization.
    pub tags: Vec<String>,
}

/// Threshold configuration for crystallization.
#[derive(Debug, Clone)]
pub struct CrystallizationConfig {
    /// Minimum score for a single mutation to be considered for crystallization.
    pub min_score: f64,
    /// How many successful repetitions are required before crystallizing.
    pub required_hits: u64,
    /// Directory where crystallized skills are stored.
    pub skills_dir: PathBuf,
}

impl Default for CrystallizationConfig {
    fn default() -> Self {
        Self {
            min_score: 0.7,
            required_hits: 3,
            skills_dir: dirs_skills_fallback(),
        }
    }
}

fn dirs_skills_fallback() -> PathBuf {
    crate::core::nt_core_util::home_dir()
        .join(".neotrix")
        .join("skills")
}

/// A GenericAgent-inspired skill crystallizer.
///
/// Watches for mutations that succeed repeatedly and freezes them into
/// persistent Ne source files for reuse.
#[derive(Debug, Clone)]
pub struct SkillCrystallizer {
    pub config: CrystallizationConfig,
    /// Tracks how many successful hits each mutation fingerprint has received.
    hit_counts: HashMap<String, u64>,
    /// Rolling score total per fingerprint.
    score_totals: HashMap<String, f64>,
}

impl Default for SkillCrystallizer {
    fn default() -> Self {
        Self::new(CrystallizationConfig::default())
    }
}

impl SkillCrystallizer {
    pub fn new(config: CrystallizationConfig) -> Self {
        if let Err(e) = fs::create_dir_all(&config.skills_dir) {
            log::warn!("failed to create skills dir {:?}: {}", config.skills_dir, e);
        }
        Self {
            config,
            hit_counts: HashMap::new(),
            score_totals: HashMap::new(),
        }
    }

    /// Generate a fingerprint string from a mutation operation for dedup tracking.
    fn fingerprint(mutation: &MutationOp) -> String {
        match mutation {
            MutationOp::TuneParam { target, delta: _ } => {
                format!("tuneparam:{}", target)
            }
            MutationOp::AddHandler { position, code: _ } => {
                format!("addhandler:{}", position)
            }
            MutationOp::RewriteHandler { name, code: _ } => {
                format!("rewritehandler:{}", name)
            }
            MutationOp::SwapPolicy { gates } => {
                format!("swappolicy:{}", gates.join("+"))
            }
            MutationOp::RewritePrimitive { name, impl_: _ } => {
                format!("writeprimitive:{}", name)
            }
            MutationOp::RewriteMeta { strategy } => {
                format!(
                    "rewritemeta:{}",
                    strategy.proposer.chars().take(40).collect::<String>()
                )
            }
            MutationOp::SelfModifyProposal { target, .. } => {
                format!("selfmodify:{}", target)
            }
        }
    }

    /// Extract a human-readable name from a mutation operation.
    fn skill_name(mutation: &MutationOp) -> String {
        match mutation {
            MutationOp::TuneParam { target, delta: _ } => {
                format!("tune_{}", target.replace('.', "_"))
            }
            MutationOp::AddHandler { position, code: _ } => {
                format!("handler_at_{}", position.replace('.', "_"))
            }
            MutationOp::RewriteHandler { name, code: _ } => {
                format!("rewrite_{}", name)
            }
            MutationOp::SwapPolicy { gates } => {
                format!(
                    "policy_{}",
                    gates.first().map(|s| s.as_str()).unwrap_or("default")
                )
            }
            MutationOp::RewritePrimitive { name, impl_: _ } => {
                format!("prim_{}", name)
            }
            MutationOp::RewriteMeta { strategy } => {
                format!("meta_v{}", strategy.version)
            }
            MutationOp::SelfModifyProposal { target, .. } => {
                format!("selfmodify_{}", target.replace('.', "_"))
            }
        }
    }

    /// Generate Ne source code from a mutation operation.
    fn to_ne_source(mutation: &MutationOp) -> String {
        match mutation {
            MutationOp::TuneParam { target, delta: _ } => {
                format!(
                    "(define (tune_{} delta)\n  (bind \n    (bundle [vsa-encode \"{}\"] [vsa-encode (str delta)])\n    [1 0 1 0]))",
                    target.replace('.', "_"),
                    target,
                )
            }
            MutationOp::AddHandler { position, code } => {
                format!(
                    "(define (handler_{} ctx)\n  (let _ (quote {}) ctx)\n  (bundle [1 1 0 0] [vsa-encode \"ok\"]))",
                    position.replace('.', "_"),
                    code.chars().take(60).collect::<String>(),
                )
            }
            MutationOp::RewriteHandler { name, code } => {
                format!(
                    "(define (rewrite_{} ctx)\n  (let _ (quote {}) ctx)\n  (bundle [0 1 0 1] [vsa-encode \"done\"]))",
                    name,
                    code.chars().take(60).collect::<String>(),
                )
            }
            MutationOp::SwapPolicy { gates } => {
                let gates_str = gates.join(" ");
                format!(
                    "(define (policy_{} ctx)\n  (seq\n    {}\n    (bundle [1 0 1 1] [vsa-encode \"swapped\"])))",
                    gates.first().map(|s| s.as_str()).unwrap_or("default"),
                    gates_str,
                )
            }
            MutationOp::RewritePrimitive { name, impl_ } => {
                format!(
                    "(define (prim_{} args)\n  (let _ (quote {}) args)\n  (negate [1 1 1 1]))",
                    name,
                    impl_.chars().take(60).collect::<String>(),
                )
            }
            MutationOp::RewriteMeta { strategy } => {
                format!(
                    "(define (meta_v{})\n  (quote propose:{}))",
                    strategy.version,
                    strategy.proposer.chars().take(60).collect::<String>(),
                )
            }
            MutationOp::SelfModifyProposal { target, .. } => {
                format!(
                    "(define (selfmodify_{})\n  (quote proposal))",
                    target.replace('.', "_"),
                )
            }
        }
    }

    /// Extract tags from a mutation.
    fn tags(mutation: &MutationOp) -> Vec<String> {
        match mutation {
            MutationOp::TuneParam { .. } => {
                vec!["tune".into(), "parameter".into()]
            }
            MutationOp::AddHandler { .. } => {
                vec!["handler".into(), "addition".into()]
            }
            MutationOp::RewriteHandler { .. } => {
                vec!["handler".into(), "rewrite".into()]
            }
            MutationOp::SwapPolicy { .. } => {
                vec!["policy".into(), "swap".into()]
            }
            MutationOp::RewritePrimitive { .. } => {
                vec!["primitive".into(), "rewrite".into()]
            }
            MutationOp::RewriteMeta { .. } => {
                vec!["meta".into(), "rewrite".into()]
            }
            MutationOp::SelfModifyProposal { .. } => {
                vec!["self".into(), "modify".into()]
            }
        }
    }

    /// Try to crystallize an evolution record into a skill.
    ///
    /// Returns `Some(CrystallizedSkill)` if the mutation has succeeded
    /// `required_hits` times with scores above `min_score`.
    pub fn crystallize(&mut self, record: &SelfEvolutionStep) -> Option<CrystallizedSkill> {
        let score = record.score_after.unwrap_or(0.0);
        if score < self.config.min_score {
            return None;
        }

        let fp = Self::fingerprint(&record.mutation);

        let entry = self.hit_counts.entry(fp.clone()).or_insert(0);
        *entry += 1;

        let total = self.score_totals.entry(fp.clone()).or_insert(0.0);
        *total += score;

        if *entry < self.config.required_hits {
            return None;
        }

        let avg = *total / *entry as f64;
        let name = Self::skill_name(&record.mutation);
        let description = format!(
            "Crystallized skill from {}: {} (avg score {:.3})",
            record.mutation.label(),
            name,
            avg
        );
        let ne_source = Self::to_ne_source(&record.mutation);
        let tags = Self::tags(&record.mutation);

        Some(CrystallizedSkill {
            name,
            description,
            ne_source,
            invocation_count: *entry,
            avg_score: avg,
            tags,
        })
    }

    /// Store a crystallized skill to disk at `~/.neotrix/skills/{name}.ne`.
    pub fn store(&self, skill: &CrystallizedSkill) -> Result<(), String> {
        let path = self.config.skills_dir.join(format!("{}.ne", skill.name));
        let content = format!(
            ";; NeoTrix Crystallized Skill: {}\n;; {}\n;; avg_score={:.3} invocations={}\n;; tags: {}\n\n{}\n",
            skill.name,
            skill.description,
            skill.avg_score,
            skill.invocation_count,
            skill.tags.join(", "),
            skill.ne_source,
        );
        fs::write(&path, &content).map_err(|e| format!("cannot write skill: {}", e))?;
        Ok(())
    }

    /// 返回正在追踪的突变指纹数（已至少命中 1 次，但未必已结晶）
    pub fn tracked_mutations(&self) -> usize {
        self.hit_counts.len()
    }

    /// 返回已结晶的技能数量（磁盘上 .ne 文件数）
    pub fn skill_count(&self) -> usize {
        self.list().len()
    }

    /// List all crystallized skill names.
    pub fn list(&self) -> Vec<String> {
        let dir = &self.config.skills_dir;
        if !dir.exists() {
            return vec![];
        }
        let mut skills = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "ne").unwrap_or(false) {
                    if let Some(stem) = path.file_stem() {
                        skills.push(stem.to_string_lossy().to_string());
                    }
                }
            }
        }
        skills.sort();
        skills
    }

    /// Load a crystallized skill from disk by name.
    pub fn load(&self, name: &str) -> Option<CrystallizedSkill> {
        // First check in-memory hit counts for an approximate match
        let path = self.config.skills_dir.join(format!("{}.ne", name));
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| {
                log::warn!("skill_crystal: failed to read {}: {}", path.display(), e);
            })
            .ok()?;

        let description = content
            .lines()
            .find(|l| l.starts_with(";; "))
            .and_then(|l| l.strip_prefix(";; "))
            .unwrap_or("")
            .to_string();

        let tags: Vec<String> = content
            .lines()
            .find(|l| l.starts_with(";; tags:"))
            .and_then(|l| l.strip_prefix(";; tags:"))
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
            .unwrap_or_default();

        let name_str = name.to_string();

        Some(CrystallizedSkill {
            name: name_str,
            description,
            ne_source: content,
            invocation_count: 0,
            avg_score: 0.0,
            tags,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::self_evolution_loop::MutationOp;

    fn dummy_record(id: u64, mutation: MutationOp, score: f64) -> SelfEvolutionStep {
        SelfEvolutionStep {
            id,
            mutation,
            parent_id: 0,
            score_before: score - 0.1,
            score_after: Some(score),
            compiles: true,
            timestamp: 0,
            generation: 1,
            accepted: false,
            cmp_score: None,
        }
    }

    #[test]
    fn test_crystallize_requires_hits() {
        let mut crystallizer = SkillCrystallizer::default();
        let mutation = MutationOp::TuneParam {
            target: "test.val".into(),
            delta: 0.1,
        };
        let record = dummy_record(1, mutation.clone(), 0.8);
        // First hit: should not crystallize (needs 3)
        let result = crystallizer.crystallize(&record);
        assert!(result.is_none(), "should not crystallize on first hit");

        // Second hit
        let r2 = dummy_record(2, mutation.clone(), 0.85);
        let result = crystallizer.crystallize(&r2);
        assert!(result.is_none(), "should not crystallize on second hit");
    }

    #[test]
    fn test_crystallize_after_three_hits() {
        let mut crystallizer = SkillCrystallizer::default();
        let mutation = MutationOp::RewriteHandler {
            name: "handle_foo".into(),
            code: "fn handle_foo() {}".into(),
        };
        for i in 0..3 {
            let record = dummy_record(i, mutation.clone(), 0.75 + i as f64 * 0.05);
            let result = crystallizer.crystallize(&record);
            if i < 2 {
                assert!(result.is_none(), "hit {} should not crystallize", i);
            } else {
                assert!(result.is_some(), "hit 3 should crystallize");
                let skill = result.unwrap();
                assert!(skill.name.contains("handle_foo"));
                assert!(skill.avg_score >= 0.75);
                assert_eq!(skill.invocation_count, 3);
            }
        }
    }

    #[test]
    fn test_low_score_does_not_crystallize() {
        let mut crystallizer = SkillCrystallizer::default();
        let mutation = MutationOp::TuneParam {
            target: "low.val".into(),
            delta: 0.01,
        };
        let record = dummy_record(1, mutation.clone(), 0.3);
        let result = crystallizer.crystallize(&record);
        assert!(result.is_none(), "low score should not count as hit");

        // Verify hit count didn't increase
        let fp = SkillCrystallizer::fingerprint(&mutation);
        assert!(
            crystallizer.hit_counts.get(&fp).is_none()
                || *crystallizer.hit_counts.get(&fp).unwrap() == 0
        );
    }

    #[test]
    fn test_fingerprint_uniqueness() {
        let a = MutationOp::TuneParam {
            target: "x".into(),
            delta: 0.1,
        };
        let b = MutationOp::TuneParam {
            target: "y".into(),
            delta: 0.2,
        };
        let fp_a = SkillCrystallizer::fingerprint(&a);
        let fp_b = SkillCrystallizer::fingerprint(&b);
        assert_ne!(fp_a, fp_b);
    }

    #[test]
    fn test_store_and_load_roundtrip() {
        let tmp = std::env::temp_dir().join("neotrix_skill_test");
        let _ = fs::remove_dir_all(&tmp);
        let config = CrystallizationConfig {
            skills_dir: tmp.clone(),
            ..Default::default()
        };
        let crystallizer = SkillCrystallizer::new(config);

        let skill = CrystallizedSkill {
            name: "test_skill".into(),
            description: "a test".into(),
            ne_source: "(define (test) nil)".into(),
            invocation_count: 3,
            avg_score: 0.85,
            tags: vec!["test".into(), "demo".into()],
        };

        assert!(crystallizer.store(&skill).is_ok());
        let listed = crystallizer.list();
        assert!(listed.contains(&"test_skill".to_string()));

        let loaded = crystallizer.load("test_skill");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "test_skill");

        let _ = fs::remove_dir_all(&tmp);
    }
}
