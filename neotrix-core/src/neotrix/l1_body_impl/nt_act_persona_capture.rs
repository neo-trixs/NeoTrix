use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonaLayer {
    HardRules,
    Identity,
    Expression,
    DecisionPatterns,
    InterpersonalStyle,
}

impl PersonaLayer {
    pub fn label(&self) -> &'static str {
        match self {
            PersonaLayer::HardRules => "Hard Rules (Constraints & Non-negotiables)",
            PersonaLayer::Identity => "Identity (Role, Background, Values)",
            PersonaLayer::Expression => "Expression (Tone, Vocabulary, Catchphrases)",
            PersonaLayer::DecisionPatterns => "Decision Patterns (Heuristics, Trade-offs)",
            PersonaLayer::InterpersonalStyle => "Interpersonal Style (Conflict, Feedback, Collaboration)",
        }
    }

    pub fn all() -> &'static [PersonaLayer] {
        &[
            PersonaLayer::HardRules,
            PersonaLayer::Identity,
            PersonaLayer::Expression,
            PersonaLayer::DecisionPatterns,
            PersonaLayer::InterpersonalStyle,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSkill {
    pub systems_owned: Vec<String>,
    pub tech_standards: Vec<String>,
    pub review_focus: Vec<String>,
    pub workflows: Vec<String>,
    pub tribal_knowledge: Vec<String>,
}

impl Default for WorkSkill {
    fn default() -> Self {
        Self {
            systems_owned: Vec::new(),
            tech_standards: Vec::new(),
            review_focus: Vec::new(),
            workflows: Vec::new(),
            tribal_knowledge: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeammatePersona {
    pub slug: String,
    pub name: String,
    pub role: String,
    pub level: String,
    pub layers: HashMap<PersonaLayer, String>,
    pub tags: Vec<String>,
}

impl TeammatePersona {
    pub fn new(slug: String, name: String, role: String, level: String) -> Self {
        let mut layers = HashMap::new();
        for layer in PersonaLayer::all() {
            layers.insert(*layer, String::new());
        }
        Self {
            slug,
            name,
            role,
            level,
            layers,
            tags: Vec::new(),
        }
    }

    pub fn set_layer(&mut self, layer: PersonaLayer, content: String) {
        self.layers.insert(layer, content);
    }

    pub fn get_layer(&self, layer: PersonaLayer) -> Option<&String> {
        self.layers.get(&layer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeammateSkill {
    pub persona: TeammatePersona,
    pub work: WorkSkill,
    pub meta: SkillMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub created_at: u64,
    pub updated_at: u64,
    pub version: u32,
    pub source_materials: Vec<String>,
}

impl SkillMeta {
    pub fn new() -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Self {
            created_at: now,
            updated_at: now,
            version: 1,
            source_materials: Vec::new(),
        }
    }
}

impl Default for SkillMeta {
    fn default() -> Self {
        Self::new()
    }
}

impl TeammateSkill {
    pub fn new(persona: TeammatePersona, work: WorkSkill) -> Self {
        Self {
            persona,
            work,
            meta: SkillMeta::new(),
        }
    }

    pub fn update(&mut self, new_persona: Option<TeammatePersona>, new_work: Option<WorkSkill>, new_sources: Vec<String>) {
        if let Some(p) = new_persona {
            self.persona = p;
        }
        if let Some(w) = new_work {
            self.work = w;
        }
        self.meta.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.meta.version += 1;
        self.meta.source_materials.extend(new_sources);
    }

    pub fn rollback_version(&mut self, target_version: u32) -> bool {
        if target_version < self.meta.version {
            self.meta.version = target_version;
            self.meta.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillDistillerConfig {
    pub auto_version: bool,
    pub max_versions: usize,
    pub quality_gate_enabled: bool,
    pub min_confidence: f64,
}

impl Default for SkillDistillerConfig {
    fn default() -> Self {
        Self {
            auto_version: true,
            max_versions: 10,
            quality_gate_enabled: true,
            min_confidence: 0.7,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillDistiller {
    config: SkillDistillerConfig,
    skills: HashMap<String, TeammateSkill>,
    history: HashMap<String, Vec<TeammateSkill>>,
}

impl SkillDistiller {
    pub fn new(config: SkillDistillerConfig) -> Self {
        Self {
            config,
            skills: HashMap::new(),
            history: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(SkillDistillerConfig::default())
    }

    pub fn create_from_description(&mut self, description: &str) -> TeammateSkill {
        let (slug, name, role, level) = Self::parse_description(description);
        let mut persona = TeammatePersona::new(slug.clone(), name, role, level);

        let inferred_layers = Self::infer_layers_from_description(description);
        for (layer, content) in inferred_layers {
            persona.set_layer(layer, content);
        }

        let work = Self::infer_work_skill(description);

        let skill = TeammateSkill::new(persona, work);
        self.skills.insert(slug.clone(), skill.clone());
        skill
    }

    fn parse_description(description: &str) -> (String, String, String, String) {
        let parts: Vec<&str> = description.split(',').map(|s| s.trim()).collect();
        let name = parts.first().copied().unwrap_or("Unknown").to_string();
        let role = parts.get(1).copied().unwrap_or("Engineer").to_string();
        let level = parts.get(2).copied().unwrap_or("Mid").to_string();
        let slug = name.to_lowercase().replace(' ', "-");
        (slug, name, role, level)
    }

    fn infer_layers_from_description(description: &str) -> Vec<(PersonaLayer, String)> {
        let desc_lower = description.to_lowercase();
        let mut layers = Vec::new();

        if desc_lower.contains("perfectionist") || desc_lower.contains("quality") {
            layers.push((PersonaLayer::HardRules, "Quality > Speed. No shortcuts on correctness.".to_string()));
        }
        if desc_lower.contains("ship fast") || desc_lower.contains("pragmatic") {
            layers.push((PersonaLayer::HardRules, "Ship working code. Iterate in production.".to_string()));
        }
        if desc_lower.contains("intj") || desc_lower.contains("analytical") {
            layers.push((PersonaLayer::Identity, "Analytical, systems-thinker, autonomous.".to_string()));
        }
        if desc_lower.contains("direct") || desc_lower.contains("blunt") {
            layers.push((PersonaLayer::Expression, "Direct, conclusion-first communication. No fluff.".to_string()));
        }
        if desc_lower.contains("mentor") || desc_lower.contains("teaching") {
            layers.push((PersonaLayer::InterpersonalStyle, "Mentor-type. Explains reasoning, not just answers.".to_string()));
        }
        if desc_lower.contains("gatekeeper") || desc_lower.contains("block") {
            layers.push((PersonaLayer::DecisionPatterns, "Blocks on naming, idempotency, error handling.".to_string()));
        }

        if layers.is_empty() {
            layers.push((PersonaLayer::Identity, "Professional engineer with strong opinions.".to_string()));
            layers.push((PersonaLayer::HardRules, "Correctness over speed. Tests required.".to_string()));
        }

        layers
    }

    fn infer_work_skill(description: &str) -> WorkSkill {
        let desc_lower = description.to_lowercase();
        let mut work = WorkSkill::default();

        if desc_lower.contains("backend") || desc_lower.contains("api") {
            work.systems_owned.push("Payments Core".to_string());
            work.systems_owned.push("Auth Service".to_string());
        }
        if desc_lower.contains("ruby") || desc_lower.contains("rails") {
            work.tech_standards.push("Ruby on Rails".to_string());
        }
        if desc_lower.contains("go") || desc_lower.contains("golang") {
            work.tech_standards.push("Go".to_string());
        }
        if desc_lower.contains("postgres") || desc_lower.contains("sql") {
            work.tech_standards.push("PostgreSQL".to_string());
        }
        if desc_lower.contains("review") || desc_lower.contains("cr") {
            work.review_focus.push("Idempotency".to_string());
            work.review_focus.push("Naming conventions".to_string());
            work.review_focus.push("Error handling".to_string());
        }
        if desc_lower.contains("payment") || desc_lower.contains("billing") {
            work.tribal_knowledge.push("Never use floats for money".to_string());
            work.tribal_knowledge.push("Always use Decimal type for currency".to_string());
        }

        work
    }

    pub fn update_skill(&mut self, slug: &str, new_persona: Option<TeammatePersona>, new_work: Option<WorkSkill>, new_sources: Vec<String>) -> Option<TeammateSkill> {
        if let Some(skill) = self.skills.get_mut(slug) {
            if self.config.auto_version {
                let current = skill.clone();
                self.history.entry(slug.to_string()).or_default().push(current);
                if self.history[slug].len() > self.config.max_versions {
                    self.history.get_mut(slug)?.remove(0);
                }
            }
            skill.update(new_persona, new_work, new_sources);
            Some(skill.clone())
        } else {
            None
        }
    }

    pub fn rollback_skill(&mut self, slug: &str, target_version: u32) -> Option<TeammateSkill> {
        if let Some(skill) = self.skills.get_mut(slug) {
            if skill.rollback_version(target_version) {
                Some(skill.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn list_skills(&self) -> Vec<String> {
        self.skills.keys().cloned().collect()
    }

    pub fn get_skill(&self, slug: &str) -> Option<&TeammateSkill> {
        self.skills.get(slug)
    }

    pub fn compare_skills(&self, slug_a: &str, slug_b: &str) -> Option<String> {
        let a = self.skills.get(slug_a)?;
        let b = self.skills.get(slug_b)?;

        let mut output = String::new();
        output.push_str(&format!("=== {} vs {} ===\n", a.persona.name, b.persona.name));
        output.push_str(&format!("Role: {} | {}\n", a.persona.role, b.persona.role));
        output.push_str(&format!("Level: {} | {}\n\n", a.persona.level, b.persona.level));

        output.push_str("--- Work Skill ---\n");
        output.push_str(&format!("Systems: {} | {}\n",
            a.work.systems_owned.join(", "),
            b.work.systems_owned.join(", ")));
        output.push_str(&format!("Standards: {} | {}\n",
            a.work.tech_standards.join(", "),
            b.work.tech_standards.join(", ")));
        output.push_str(&format!("Review Focus: {} | {}\n\n",
            a.work.review_focus.join(", "),
            b.work.review_focus.join(", ")));

        output.push_str("--- Persona (5 Layers) ---\n");
        for layer in PersonaLayer::all() {
            let layer_a = a.persona.get_layer(*layer).map(|s| s.as_str()).unwrap_or("");
            let layer_b = b.persona.get_layer(*layer).map(|s| s.as_str()).unwrap_or("");
            output.push_str(&format!("{}: {} | {}\n", layer.label(), layer_a, layer_b));
        }

        output.push_str("\n--- Recommendation ---\n");
        if a.work.review_focus.len() > b.work.review_focus.len() {
            output.push_str(&format!("{} for correctness rigor (more review focus areas)\n", a.persona.name));
        } else {
            output.push_str(&format!("{} for shipping velocity\n", b.persona.name));
        }

        Some(output)
    }

    pub fn scenario_simulation(&self, slug: &str, scenario: &str) -> Option<String> {
        let skill = self.skills.get(slug)?;
        let mut response = String::new();
        response.push_str(&format!("{} ({}):\n", skill.persona.name, skill.persona.role));

        let scenario_lower = scenario.to_lowercase();
        let default_expression = "Professional".to_string();
        let default_decision = "Pragmatic".to_string();
        let default_rules = "Quality first".to_string();
        let expression = skill.persona.get_layer(PersonaLayer::Expression).unwrap_or(&default_expression);
        let decision = skill.persona.get_layer(PersonaLayer::DecisionPatterns).unwrap_or(&default_decision);

        if scenario_lower.contains("review") || scenario_lower.contains("pr") {
            response.push_str(&format!(
                "Reviewing: {}. Focus on {}. \"{}\"",
                scenario,
                skill.work.review_focus.join(", "),
                decision
            ));
        } else if scenario_lower.contains("choice") || scenario_lower.contains("decide") {
            response.push_str(&format!(
                "Decision: {}. Heuristic: {}",
                scenario,
                decision
            ));
        } else if scenario_lower.contains("pressure") || scenario_lower.contains("deadline") {
            let hard_rules = skill.persona.get_layer(PersonaLayer::HardRules).unwrap_or(&default_rules);
            response.push_str(&format!(
                "Under pressure: {}. \"{}\"",
                scenario,
                hard_rules
            ));
        } else {
            response.push_str(&format!(
                "Response style ({}): \"{}\"",
                expression,
                scenario
            ));
        }

        Some(response)
    }
}

impl Default for SkillDistiller {
    fn default() -> Self {
        Self::with_defaults()
    }
}

pub static DISTILLER: LazyLock<Mutex<SkillDistiller>> = LazyLock::new(|| Mutex::new(SkillDistiller::with_defaults()));

pub fn create_teammate(description: &str) -> TeammateSkill {
    DISTILLER.lock().unwrap().create_from_description(description)
}

pub fn list_teammates() -> Vec<String> {
    DISTILLER.lock().unwrap().list_skills()
}

pub fn get_teammate(slug: &str) -> Option<TeammateSkill> {
    DISTILLER.lock().unwrap().get_skill(slug).cloned()
}

pub fn compare_teammates(a: &str, b: &str) -> Option<String> {
    DISTILLER.lock().unwrap().compare_skills(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_layer_labels() {
        assert_eq!(PersonaLayer::HardRules.label(), "Hard Rules (Constraints & Non-negotiables)");
        assert_eq!(PersonaLayer::Identity.label(), "Identity (Role, Background, Values)");
        assert_eq!(PersonaLayer::Expression.label(), "Expression (Tone, Vocabulary, Catchphrases)");
    }

    #[test]
    fn test_all_layers_listed() {
        let layers = PersonaLayer::all();
        assert_eq!(layers.len(), 5);
    }

    #[test]
    fn test_teammate_persona_creation() {
        let mut persona = TeammatePersona::new(
            "alex-chen".to_string(),
            "Alex Chen".to_string(),
            "Backend Engineer".to_string(),
            "Staff".to_string(),
        );
        persona.set_layer(PersonaLayer::HardRules, "Quality > Speed".to_string());
        persona.set_layer(PersonaLayer::Expression, "Direct, no fluff".to_string());

        assert_eq!(persona.slug, "alex-chen");
        assert_eq!(persona.get_layer(PersonaLayer::HardRules), Some(&"Quality > Speed".to_string()));
    }

    #[test]
    fn test_work_skill_default() {
        let work = WorkSkill::default();
        assert!(work.systems_owned.is_empty());
        assert!(work.tech_standards.is_empty());
    }

    #[test]
    fn test_teammate_skill_creation() {
        let persona = TeammatePersona::new("test".to_string(), "Test".to_string(), "Eng".to_string(), "Mid".to_string());
        let work = WorkSkill::default();
        let skill = TeammateSkill::new(persona, work);

        assert_eq!(skill.meta.version, 1);
        assert!(skill.meta.created_at > 0);
    }

    #[test]
    fn test_skill_update_increments_version() {
        let mut skill = TeammateSkill::new(
            TeammatePersona::new("test".to_string(), "Test".to_string(), "Eng".to_string(), "Mid".to_string()),
            WorkSkill::default(),
        );
        let initial_version = skill.meta.version;

        skill.update(None, None, vec!["new source".to_string()]);

        assert_eq!(skill.meta.version, initial_version + 1);
        assert_eq!(skill.meta.source_materials.len(), 1);
    }

    #[test]
    fn test_skill_rollback() {
        let mut skill = TeammateSkill::new(
            TeammatePersona::new("test".to_string(), "Test".to_string(), "Eng".to_string(), "Mid".to_string()),
            WorkSkill::default(),
        );
        skill.update(None, None, vec!["source1".to_string()]);
        skill.update(None, None, vec!["source2".to_string()]);
        assert_eq!(skill.meta.version, 3);

        let result = skill.rollback_version(2);
        assert!(result);
        assert_eq!(skill.meta.version, 2);
    }

    #[test]
    fn test_rollback_invalid_version() {
        let mut skill = TeammateSkill::new(
            TeammatePersona::new("test".to_string(), "Test".to_string(), "Eng".to_string(), "Mid".to_string()),
            WorkSkill::default(),
        );

        let result = skill.rollback_version(5);
        assert!(!result);
        assert_eq!(skill.meta.version, 1);
    }

    #[test]
    fn test_distiller_create_from_description() {
        let mut distiller = SkillDistiller::with_defaults();
        let skill = distiller.create_from_description("Alex Chen, Stripe L3 backend, INTJ perfectionist, brutal code reviewer but usually right");

        assert_eq!(skill.persona.slug, "alex-chen");
        assert_eq!(skill.persona.name, "Alex Chen");
        assert_eq!(skill.persona.role, "Stripe L3 backend");
        assert!(!skill.persona.layers.is_empty());
        assert!(!skill.work.systems_owned.is_empty());
    }

    #[test]
    fn test_distiller_infers_payment_knowledge() {
        let mut distiller = SkillDistiller::with_defaults();
        let skill = distiller.create_from_description("Alex Chen, Payments Core backend, Stripe L3, focuses on idempotency and naming");

        assert!(skill.work.tribal_knowledge.iter().any(|k| k.contains("money") || k.contains("Decimal")));
    }

    #[test]
    fn test_list_skills() {
        let mut distiller = SkillDistiller::with_defaults();
        distiller.create_from_description("Alice, Frontend, React expert");
        distiller.create_from_description("Bob, Backend, Go microservices");

        let list = distiller.list_skills();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"alice".to_string()));
        assert!(list.contains(&"bob".to_string()));
    }

    #[test]
    fn test_compare_skills() {
        let mut distiller = SkillDistiller::with_defaults();
        let a = distiller.create_from_description("Alice, Frontend, React expert, ships fast");
        let b = distiller.create_from_description("Bob, Backend, Go microservices, perfectionist");

        let comparison = distiller.compare_skills(&a.persona.slug, &b.persona.slug);
        assert!(comparison.is_some());
        let comp = comparison.unwrap();
        assert!(comp.contains("Alice"));
        assert!(comp.contains("Bob"));
        assert!(comp.contains("Work Skill"));
        assert!(comp.contains("Persona"));
    }

    #[test]
    fn test_scenario_simulation() {
        let mut distiller = SkillDistiller::with_defaults();
        let skill = distiller.create_from_description("Alice, Backend, perfectionist, blocks on naming and idempotency");

        let review_response = distiller.scenario_simulation(&skill.persona.slug, "Review this PR for payments API");
        assert!(review_response.is_some());
        let resp = review_response.unwrap();
        assert!(resp.contains("Alice"));
        assert!(resp.contains("idempotency") || resp.contains("naming"));

        let pressure_response = distiller.scenario_simulation(&skill.persona.slug, "Ship by Friday, skip tests?");
        assert!(pressure_response.is_some());
        let resp = pressure_response.unwrap();
        assert!(resp.contains("Quality") || resp.contains("tests"));
    }

    #[test]
    fn test_skill_meta_serialization() {
        let meta = SkillMeta::new();
        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: SkillMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(meta.version, deserialized.version);
        assert_eq!(meta.created_at, deserialized.created_at);
    }

    #[test]
    fn test_teammate_skill_serialization() {
        let skill = TeammateSkill::new(
            TeammatePersona::new("test".to_string(), "Test User".to_string(), "Engineer".to_string(), "Senior".to_string()),
            WorkSkill::default(),
        );

        let json = serde_json::to_string(&skill).unwrap();
        let deserialized: TeammateSkill = serde_json::from_str(&json).unwrap();
        assert_eq!(skill.persona.slug, deserialized.persona.slug);
        assert_eq!(skill.meta.version, deserialized.meta.version);
    }
}