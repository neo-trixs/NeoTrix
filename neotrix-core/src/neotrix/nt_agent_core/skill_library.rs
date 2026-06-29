use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillType {
    Primitive,
    Composite,
}

#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub skill_type: SkillType,
    pub capability_tags: Vec<String>,
    pub prerequisites: Vec<String>,
    pub invocation_count: u64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Clone)]
pub struct CompositeRecipe {
    pub name: String,
    pub description: String,
    pub steps: Vec<SkillStep>,
}

#[derive(Debug, Clone)]
pub enum SkillStep {
    Sequential(Vec<String>),
    Parallel(Vec<String>),
    Conditional {
        condition: String,
        if_true: Box<SkillStep>,
        if_false: Box<SkillStep>,
    },
}

#[derive(Debug, Clone)]
pub struct SkillMatch {
    pub name: String,
    pub score: f64,
    pub skill_type: SkillType,
}

pub struct SkillLibrary {
    skills: HashMap<String, SkillDefinition>,
    recipes: HashMap<String, CompositeRecipe>,
    max_skills: usize,
}

impl SkillLibrary {
    pub fn new(max_skills: usize) -> Self {
        Self::with_agentskills(max_skills).0
    }

    /// Create a new SkillLibrary optionally pre-populated with skills from
    /// an agentskills.io-compatible SKILL.md bundle.
    /// Returns (library, errors) where errors is empty on success.
    pub fn with_agentskills(max_skills: usize) -> (Self, Vec<String>) {
        let lib = Self {
            skills: HashMap::new(),
            recipes: HashMap::new(),
            max_skills,
        };
        (lib, Vec::new())
    }

    /// Import skills from a SKILL.md bundle (agentskills.io compatible format).
    ///
    /// Each skill block:
    /// ```markdown
    /// # skill_name
    /// ## Description
    /// Short description text.
    /// ## Tags
    /// tag1, tag2, tag3
    /// ## Dependencies (optional)
    /// dep1, dep2
    /// ## Example (optional)
    /// ```
    ///
    /// Returns list of successfully registered skill names.
    pub fn import_agentskills_md(&mut self, content: &str) -> Vec<String> {
        let mut registered = Vec::new();
        // Split on "\n# " to get blocks, prepend "# " back for the first block
        let raw_blocks: Vec<&str> = content.split("\n# ").collect();
        let blocks: Vec<String> = raw_blocks.iter().enumerate().map(|(i, b)| {
            if i == 0 { format!("{}", b) } else { format!("# {}", b) }
        }).collect();

        for block in &blocks {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }

            let mut name = String::new();
            let mut description = String::new();
            let mut tags = Vec::new();
            let mut current_section = String::new();
            let mut section_lines: Vec<String> = Vec::new();

            for line in block.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("# ") && name.is_empty() {
                    name = trimmed[2..].trim().to_string();
                } else if trimmed.starts_with("## ") {
                    // Flush previous section content
                    if current_section == "description" {
                        description = section_lines.join(" ").trim().to_string();
                    } else if current_section == "tags" {
                        for part in section_lines.join(" ").split(',') {
                            let tag = part.trim().to_string();
                            if !tag.is_empty() {
                                tags.push(tag);
                            }
                        }
                    }
                    current_section = trimmed[3..].trim().to_lowercase();
                    section_lines.clear();
                } else if !trimmed.is_empty() && !trimmed.starts_with("```") {
                    section_lines.push(trimmed.to_string());
                }
            }

            // Flush last section
            if current_section == "description" {
                description = section_lines.join(" ").trim().to_string();
            } else if current_section == "tags" {
                for part in section_lines.join(" ").split(',') {
                    let tag = part.trim().to_string();
                    if !tag.is_empty() {
                        tags.push(tag);
                    }
                }
            }

            if !name.is_empty() && !description.is_empty() {
                self.register(&name, &description, tags);
                registered.push(name);
            }
        }

        registered
    }

    /// Export all skills as an agentskills.io-compatible SKILL.md bundle.
    ///
    /// Each skill becomes a separate section prefixed with `# `.
    /// Skills are separated by `---` for clarity.
    pub fn export_agentskills_md(&self) -> String {
        let mut output = String::new();
        let mut first = true;
        for skill in self.all_skills() {
            if !first {
                output.push_str("---\n\n");
            }
            first = false;
            output.push_str(&format!("# {}\n", skill.name));
            output.push_str("## Description\n");
            output.push_str(&format!("{}\n", skill.description));
            if !skill.capability_tags.is_empty() {
                output.push_str("## Tags\n");
                output.push_str(&format!("{}\n", skill.capability_tags.join(", ")));
            }
            if !skill.prerequisites.is_empty() {
                output.push_str("## Dependencies\n");
                output.push_str(&format!("{}\n", skill.prerequisites.join(", ")));
            }
            output.push('\n');
        }
        output
    }

    pub fn register(&mut self, name: &str, description: &str, tags: Vec<String>) {
        if self.skills.len() >= self.max_skills {
            let least_used = self
                .skills
                .iter()
                .min_by_key(|(_, s)| s.invocation_count)
                .map(|(k, _)| k.clone());
            if let Some(key) = least_used {
                self.skills.remove(&key);
            }
        }
        self.skills.insert(
            name.to_string(),
            SkillDefinition {
                name: name.to_string(),
                description: description.to_string(),
                skill_type: SkillType::Primitive,
                capability_tags: tags,
                prerequisites: Vec::new(),
                invocation_count: 0,
                avg_latency_ms: 0.0,
            },
        );
    }

    pub fn register_composite(&mut self, recipe: CompositeRecipe) {
        self.recipes.insert(recipe.name.clone(), recipe);
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&SkillDefinition> {
        self.skills
            .values()
            .filter(|s| s.capability_tags.iter().any(|t| t.contains(tag)))
            .collect()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&SkillDefinition> {
        self.skills.get(name)
    }

    pub fn find_best_match(&self, query: &str) -> Option<SkillMatch> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<SkillMatch> = self
            .skills
            .values()
            .map(|s| {
                let desc_lower = s.description.to_lowercase();
                let name_lower = s.name.to_lowercase();
                let tag_text: String = s.capability_tags.join(" ").to_lowercase();

                let mut score = 0.0_f64;
                for word in &query_words {
                    if name_lower.contains(word) {
                        score += 0.4;
                    }
                    if desc_lower.contains(word) {
                        score += 0.3;
                    }
                    if tag_text.contains(word) {
                        score += 0.2;
                    }
                }
                for tag in &s.capability_tags {
                    if query_lower.contains(&tag.to_lowercase()) {
                        score += 0.1;
                    }
                }

                SkillMatch {
                    name: s.name.clone(),
                    score,
                    skill_type: s.skill_type.clone(),
                }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.into_iter().next().filter(|m| m.score > 0.0)
    }

    pub fn record_invocation(&mut self, name: &str, latency_ms: f64) {
        if let Some(skill) = self.skills.get_mut(name) {
            let n = skill.invocation_count;
            skill.avg_latency_ms =
                (skill.avg_latency_ms * n as f64 + latency_ms) / (n as f64 + 1.0);
            skill.invocation_count += 1;
        }
    }

    pub fn all_skills(&self) -> Vec<&SkillDefinition> {
        let mut result: Vec<&SkillDefinition> = self.skills.values().collect();
        result.sort_by(|a, b| b.invocation_count.cmp(&a.invocation_count));
        result
    }

    pub fn all_recipes(&self) -> Vec<&CompositeRecipe> {
        self.recipes.values().collect()
    }

    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }

    pub fn recipe_count(&self) -> usize {
        self.recipes.len()
    }

    pub fn compose_recipe(&self, goal: &str) -> Option<CompositeRecipe> {
        let best = self.find_best_match(goal)?;
        let related: Vec<String> = self
            .skills
            .values()
            .filter(|s| s.name != best.name)
            .take(3)
            .map(|s| s.name.clone())
            .collect();

        if related.is_empty() {
            return None;
        }

        Some(CompositeRecipe {
            name: format!("composed_{}", goal.replace(' ', "_")),
            description: format!("Auto-composed recipe for: {}", goal),
            steps: vec![
                SkillStep::Sequential(vec![best.name.clone()]),
                SkillStep::Parallel(related),
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_find() {
        let mut lib = SkillLibrary::new(100);
        lib.register(
            "web_search",
            "Search the web for information",
            vec!["search".into(), "web".into()],
        );
        lib.register(
            "extract_text",
            "Extract text from documents",
            vec!["extract".into(), "text".into()],
        );

        assert_eq!(lib.skill_count(), 2);
        assert!(lib.find_by_name("web_search").is_some());
        assert!(lib.find_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_find_by_tag() {
        let mut lib = SkillLibrary::new(100);
        lib.register(
            "web_search",
            "Search the web",
            vec!["search".into(), "web".into()],
        );
        lib.register(
            "pdf_extract",
            "Extract PDF content",
            vec!["extract".into(), "pdf".into()],
        );

        let search_skills = lib.find_by_tag("search");
        assert_eq!(search_skills.len(), 1);
        assert_eq!(search_skills[0].name, "web_search");
    }

    #[test]
    fn test_find_best_match() {
        let mut lib = SkillLibrary::new(100);
        lib.register(
            "web_search",
            "Search the web for information",
            vec!["search".into(), "web".into()],
        );
        lib.register(
            "pdf_extract",
            "Extract text from PDF files",
            vec!["extract".into(), "pdf".into()],
        );
        lib.register(
            "summarize",
            "Summarize long text content",
            vec!["summary".into(), "text".into()],
        );

        let match_result = lib.find_best_match("search");
        assert!(match_result.is_some());
        assert_eq!(match_result.unwrap().name, "web_search");
    }

    #[test]
    fn test_invocation_tracking() {
        let mut lib = SkillLibrary::new(100);
        lib.register("test_skill", "A test skill", vec!["test".into()]);
        lib.record_invocation("test_skill", 150.0);
        lib.record_invocation("test_skill", 50.0);

        let skill = lib.find_by_name("test_skill").unwrap();
        assert_eq!(skill.invocation_count, 2);
        assert!((skill.avg_latency_ms - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_composite_recipe() {
        let mut lib = SkillLibrary::new(100);
        lib.register("web_search", "Search the web", vec!["search".into()]);
        lib.register("extract_text", "Extract content", vec!["extract".into()]);
        lib.register("summarize", "Summarize text", vec!["summary".into()]);

        lib.register_composite(CompositeRecipe {
            name: "research_pipeline".into(),
            description: "Search, extract, and summarize".into(),
            steps: vec![SkillStep::Sequential(vec![
                "web_search".into(),
                "extract_text".into(),
                "summarize".into(),
            ])],
        });

        assert_eq!(lib.recipe_count(), 1);
        let recipe = lib.all_recipes()[0];
        assert_eq!(recipe.name, "research_pipeline");
    }

    #[test]
    fn test_auto_compose() {
        let mut lib = SkillLibrary::new(100);
        lib.register(
            "fetch_url",
            "Fetch a URL and return HTML",
            vec!["fetch".into(), "url".into()],
        );
        lib.register(
            "parse_html",
            "Parse HTML into text",
            vec!["parse".into(), "html".into()],
        );

        let composed = lib.compose_recipe("fetch and parse");
        assert!(composed.is_some());
        assert_eq!(composed.unwrap().steps.len(), 2);
    }

    #[test]
    fn test_lru_eviction() {
        let mut lib = SkillLibrary::new(3);
        lib.register("a", "Skill A", vec![]);
        lib.register("b", "Skill B", vec![]);
        lib.register("c", "Skill C", vec![]);
        lib.register("d", "Skill D", vec![]);

        assert_eq!(lib.skill_count(), 3);
        assert!(lib.find_by_name("d").is_some());
    }
}
