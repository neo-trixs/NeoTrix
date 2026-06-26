/// NASS — NeoTrix Agent Skill System
/// Progressive disclosure Skill format with YAML frontmatter + 3-level loading
use std::collections::HashMap;

/// Three levels of progressive disclosure
#[derive(Debug, Clone, PartialEq)]
pub enum ProgressiveLevel {
    /// Level 1: Always in context (YAML metadata only)
    Metadata,
    /// Level 2: Loaded when skill is relevant (SKILL.md body)
    Body,
    /// Level 3: Loaded on demand (linked files)
    Deep,
}

/// Skill trigger condition
#[derive(Debug, Clone)]
pub enum SkillTrigger {
    /// Triggered by VSA similarity to input
    VsaPattern(Vec<u8>),
    /// Triggered by keyword match
    Keywords(Vec<String>),
    /// Triggered by task type
    TaskType(String),
    /// Triggered by domain
    Domain(String),
    /// Composition: all triggers must match
    AllOf(Vec<SkillTrigger>),
    /// Composition: any trigger can match
    AnyOf(Vec<SkillTrigger>),
}

/// A progressive-disclosure Agent Skill
#[derive(Debug, Clone)]
pub struct ProgressiveSkill {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Short description (always in context)
    pub description: String,
    /// Version
    pub version: String,
    /// Skill domain
    pub domain: String,
    /// Trigger conditions for auto-activation
    pub trigger: SkillTrigger,
    /// Level 2: Full instructions (body)
    pub body: String,
    /// Level 3: Linked resources (file name -> content)
    pub linked_resources: HashMap<String, String>,
    /// Which level is currently loaded
    pub loaded_level: ProgressiveLevel,
    /// Dependencies: skill IDs this skill composes
    pub dependencies: Vec<String>,
    /// Confidence/success rate
    pub confidence: f64,
    /// Usage count
    pub invocation_count: u64,
    /// Average execution score
    pub avg_score: f64,
}

impl ProgressiveSkill {
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        domain: &str,
        trigger: SkillTrigger,
        body: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            version: "1.0.0".to_string(),
            domain: domain.to_string(),
            trigger,
            body: body.to_string(),
            linked_resources: HashMap::new(),
            loaded_level: ProgressiveLevel::Metadata,
            dependencies: Vec::new(),
            confidence: 0.5,
            invocation_count: 0,
            avg_score: 0.0,
        }
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_linked(mut self, resources: HashMap<String, String>) -> Self {
        self.linked_resources = resources;
        self
    }

    pub fn with_version(mut self, v: &str) -> Self {
        self.version = v.to_string();
        self
    }

    pub fn matches(&self, context: &SkillMatchContext) -> f64 {
        match &self.trigger {
            SkillTrigger::VsaPattern(pattern) => {
                if let Some(vsa) = &context.vsa_vector {
                    let sim = simple_similarity(pattern, vsa);
                    sim * self.confidence
                } else {
                    0.0
                }
            }
            SkillTrigger::Keywords(kws) => {
                let text = context.input_text.as_deref().unwrap_or("");
                let matched = kws.iter().filter(|k| text.contains(k.as_str())).count();
                if kws.is_empty() {
                    0.0
                } else {
                    (matched as f64 / kws.len() as f64) * self.confidence
                }
            }
            SkillTrigger::TaskType(ttype) => match &context.task_type {
                Some(t) if t == ttype => self.confidence,
                _ => 0.0,
            },
            SkillTrigger::Domain(domain) => {
                if self.domain == *domain {
                    self.confidence
                } else {
                    0.0
                }
            }
            SkillTrigger::AllOf(triggers) => triggers
                .iter()
                .map(|t| Self::eval_trigger(t, context))
                .min_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0),
            SkillTrigger::AnyOf(triggers) => triggers
                .iter()
                .map(|t| Self::eval_trigger(t, context))
                .max_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0),
        }
    }

    fn eval_trigger(t: &SkillTrigger, ctx: &SkillMatchContext) -> f64 {
        match t {
            SkillTrigger::VsaPattern(p) => {
                if let Some(vsa) = &ctx.vsa_vector {
                    simple_similarity(p, vsa) * 0.9
                } else {
                    0.0
                }
            }
            SkillTrigger::Keywords(kws) => {
                let text = ctx.input_text.as_deref().unwrap_or("");
                let matched = kws.iter().filter(|k| text.contains(k.as_str())).count();
                if kws.is_empty() {
                    0.0
                } else {
                    matched as f64 / kws.len() as f64
                }
            }
            SkillTrigger::TaskType(t) => match &ctx.task_type {
                Some(tt) if tt == t => 0.9,
                _ => 0.0,
            },
            SkillTrigger::Domain(_) => 0.5,
            SkillTrigger::AllOf(ts) => ts
                .iter()
                .map(|st| Self::eval_trigger(st, ctx))
                .min_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0),
            SkillTrigger::AnyOf(ts) => ts
                .iter()
                .map(|st| Self::eval_trigger(st, ctx))
                .max_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0),
        }
    }

    /// Load to next progressive level
    pub fn load_level(&mut self, level: ProgressiveLevel) {
        self.loaded_level = level;
    }

    /// Execute the skill (placeholder — real execution depends on the skill type)
    pub fn execute(&mut self, _context: &SkillMatchContext) -> Result<String, String> {
        self.invocation_count += 1;
        Ok(format!("Executed skill: {} (v{})", self.name, self.version))
    }

    /// Record execution outcome
    pub fn record_outcome(&mut self, score: f64) {
        let total = self.avg_score * (self.invocation_count as f64);
        self.invocation_count += 1;
        self.avg_score = (total + score) / (self.invocation_count as f64);
        self.confidence = (self.confidence + score * 0.1).clamp(0.0, 1.0);
    }
}

/// Context for skill matching
#[derive(Debug, Clone, Default)]
pub struct SkillMatchContext {
    pub input_text: Option<String>,
    pub vsa_vector: Option<Vec<u8>>,
    pub task_type: Option<String>,
    pub domain: Option<String>,
}

fn simple_similarity(a: &[u8], b: &[u8]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.len() != b.len() {
        return 0.0;
    }
    let diff = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
    1.0 - (diff as f64 / a.len() as f64)
}

/// SkillRegistry with auto-discovery and composable skills
#[derive(Debug, Clone)]
pub struct SkillRegistry {
    skills: HashMap<String, ProgressiveSkill>,
    composable_index: HashMap<String, Vec<String>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            composable_index: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: ProgressiveSkill) {
        let deps = skill.dependencies.clone();
        let id = skill.id.clone();
        for dep in &deps {
            self.composable_index
                .entry(dep.clone())
                .or_default()
                .push(id.clone());
        }
        self.skills.insert(id, skill);
    }

    pub fn get(&self, id: &str) -> Option<&ProgressiveSkill> {
        self.skills.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut ProgressiveSkill> {
        self.skills.get_mut(id)
    }

    pub fn find(&self, context: &SkillMatchContext, top_k: usize) -> Vec<(&ProgressiveSkill, f64)> {
        let mut scored: Vec<_> = self
            .skills
            .values()
            .map(|s| {
                let score = s.matches(context);
                (s, score)
            })
            .filter(|(_, s)| *s > 0.05)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    /// Get composed skills (skills that depend on a given skill)
    pub fn dependents(&self, skill_id: &str) -> Vec<&ProgressiveSkill> {
        self.composable_index
            .get(skill_id)
            .map(|ids| ids.iter().filter_map(|id| self.skills.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn all_skills(&self) -> Vec<&ProgressiveSkill> {
        self.skills.values().collect()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Export skill as YAML+markdown (agentskills.io compatible)
    pub fn export_markdown(&self, id: &str) -> Option<String> {
        let skill = self.skills.get(id)?;
        let mut md = String::new();
        md.push_str("---\n");
        md.push_str(&format!("name: {}\n", skill.name));
        md.push_str(&format!("description: {}\n", skill.description));
        md.push_str(&format!("version: {}\n", skill.version));
        md.push_str(&format!("domain: {}\n", skill.domain));
        md.push_str(&format!("confidence: {}\n", skill.confidence));
        if !skill.dependencies.is_empty() {
            md.push_str(&format!(
                "dependencies: [{}]\n",
                skill.dependencies.join(", ")
            ));
        }
        md.push_str("---\n\n");
        md.push_str(&skill.body);
        md.push('\n');
        Some(md)
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// StarLite skill — a skill that has been "lit up" by constellation emergence
#[derive(Debug, Clone)]
pub struct StarLiteSkill {
    pub skill_id: String,
    pub constellation_id: String,
    pub emergence_score: f64,
    pub cross_timeline_validated: bool,
    pub integrated_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_skill(id: &str, name: &str, kws: Vec<&str>) -> ProgressiveSkill {
        ProgressiveSkill::new(
            id,
            name,
            &format!("A test skill for {}", name),
            "test",
            SkillTrigger::Keywords(kws.iter().map(|s| s.to_string()).collect()),
            &format!("Instructions for {}.\nStep 1: Do X\nStep 2: Do Y", name),
        )
    }

    fn make_vsa_skill(id: &str) -> ProgressiveSkill {
        let pattern = vec![1u8, 0, 1, 0, 1, 0, 1, 0];
        ProgressiveSkill::new(
            id,
            "VSA Skill",
            "VSA-pattern triggered skill",
            "reasoning",
            SkillTrigger::VsaPattern(pattern),
            "VSA skill body",
        )
    }

    #[test]
    fn test_skill_creation() {
        let skill = make_test_skill("sk_test", "TestSkill", vec!["hello", "world"]);
        assert_eq!(skill.id, "sk_test");
        assert_eq!(skill.name, "TestSkill");
        assert_eq!(skill.version, "1.0.0");
        assert_eq!(skill.loaded_level, ProgressiveLevel::Metadata);
    }

    #[test]
    fn test_skill_matches_keywords() {
        let skill = make_test_skill("sk_kw", "KW", vec!["hello", "world"]);
        let ctx = SkillMatchContext {
            input_text: Some("hello world this is a test".into()),
            ..SkillMatchContext::default()
        };
        let score = skill.matches(&ctx);
        assert!(score > 0.0);
    }

    #[test]
    fn test_skill_no_match() {
        let skill = make_test_skill("sk_nom", "NoMatch", vec!["hello"]);
        let ctx = SkillMatchContext {
            input_text: Some("goodbye".into()),
            ..SkillMatchContext::default()
        };
        let score = skill.matches(&ctx);
        assert!(score < 0.01);
    }

    #[test]
    fn test_vsa_skill_matching() {
        let mut skill = make_vsa_skill("sk_vsa1");
        skill.confidence = 0.9;
        let ctx = SkillMatchContext {
            vsa_vector: Some(vec![1u8, 0, 1, 0, 1, 0, 1, 0]),
            ..SkillMatchContext::default()
        };
        let score = skill.matches(&ctx);
        assert!(
            (score - 0.9).abs() < 0.01,
            "exact match should give confidence"
        );
    }

    #[test]
    fn test_registry_register_and_find() {
        let mut reg = SkillRegistry::new();
        reg.register(make_test_skill("a", "A", vec!["alpha"]));
        reg.register(make_test_skill("b", "B", vec!["beta"]));
        assert_eq!(reg.len(), 2);

        let ctx = SkillMatchContext {
            input_text: Some("alpha is here".into()),
            ..SkillMatchContext::default()
        };
        let results = reg.find(&ctx, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.name, "A");
    }

    #[test]
    fn test_registry_returns_top_k() {
        let mut reg = SkillRegistry::new();
        let mut sk_a = make_test_skill("a", "A", vec!["test"]);
        sk_a.confidence = 0.9;
        let mut sk_b = make_test_skill("b", "B", vec!["test"]);
        sk_b.confidence = 0.5;
        reg.register(sk_a);
        reg.register(sk_b);

        let ctx = SkillMatchContext {
            input_text: Some("test".into()),
            ..SkillMatchContext::default()
        };
        let results = reg.find(&ctx, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.name, "A");
    }

    #[test]
    fn test_composable_dependencies() {
        let mut reg = SkillRegistry::new();
        let base = ProgressiveSkill::new(
            "base",
            "Base",
            "base skill",
            "core",
            SkillTrigger::Keywords(vec!["base".into()]),
            "base body",
        );
        let composed = ProgressiveSkill::new(
            "comp",
            "Composed",
            "composed skill",
            "core",
            SkillTrigger::Keywords(vec!["composed".into()]),
            "composed body",
        )
        .with_dependencies(vec!["base".into()]);
        reg.register(base);
        reg.register(composed);

        let deps = reg.dependents("base");
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "Composed");
    }

    #[test]
    fn test_export_markdown() {
        let mut reg = SkillRegistry::new();
        reg.register(make_test_skill("export_me", "ExportTest", vec!["export"]));
        let md = reg.export_markdown("export_me");
        assert!(md.is_some());
        let content = md.unwrap();
        assert!(content.starts_with("---"));
        assert!(content.contains("name: ExportTest"));
        assert!(content.contains("domain: test"));
        assert!(content.contains("Instructions for ExportTest"));
    }

    #[test]
    fn test_progressive_levels() {
        let mut skill = make_test_skill("pl", "PL", vec!["progressive"]);
        assert_eq!(skill.loaded_level, ProgressiveLevel::Metadata);
        skill.load_level(ProgressiveLevel::Body);
        assert_eq!(skill.loaded_level, ProgressiveLevel::Body);
        skill.load_level(ProgressiveLevel::Deep);
        assert_eq!(skill.loaded_level, ProgressiveLevel::Deep);
    }

    #[test]
    fn test_execute_and_record() {
        let mut skill = make_test_skill("exec", "Exec", vec!["run"]);
        assert_eq!(skill.invocation_count, 0);
        let result = skill.execute(&SkillMatchContext::default());
        assert!(result.is_ok());
        assert_eq!(skill.invocation_count, 1);
    }

    #[test]
    fn test_confidence_evolves_with_outcomes() {
        let mut skill = make_test_skill("conf", "Conf", vec!["test"]);
        let initial = skill.confidence;
        skill.record_outcome(0.8);
        assert!(
            skill.confidence > initial,
            "confidence should increase after success"
        );
        assert_eq!(skill.avg_score, 0.8);
    }

    #[test]
    fn test_all_of_trigger() {
        let skill = ProgressiveSkill::new(
            "allof",
            "AllOf",
            "all-of test",
            "test",
            SkillTrigger::AllOf(vec![
                SkillTrigger::Keywords(vec!["hello".into()]),
                SkillTrigger::Keywords(vec!["world".into()]),
            ]),
            "body",
        );
        let ctx = SkillMatchContext {
            input_text: Some("hello world".into()),
            ..SkillMatchContext::default()
        };
        let score = skill.matches(&ctx);
        assert!(
            score > 0.0,
            "AllOf should match when all sub-triggers match"
        );
    }

    #[test]
    fn test_any_of_trigger() {
        let skill = ProgressiveSkill::new(
            "anyof",
            "AnyOf",
            "any-of test",
            "test",
            SkillTrigger::AnyOf(vec![
                SkillTrigger::Keywords(vec!["hello".into()]),
                SkillTrigger::Keywords(vec!["missing".into()]),
            ]),
            "body",
        );
        let ctx = SkillMatchContext {
            input_text: Some("hello world".into()),
            ..SkillMatchContext::default()
        };
        let score = skill.matches(&ctx);
        assert!(
            score > 0.0,
            "AnyOf should match when any sub-trigger matches"
        );
    }
}
