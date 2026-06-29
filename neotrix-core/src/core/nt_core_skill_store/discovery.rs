use std::collections::HashMap as Map;

/// Search query for discovering skills
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub keywords: Vec<String>,
    pub category: Option<SkillCategory>,
    pub min_stars: u32,
    pub max_results: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            keywords: vec!["ai".into(), "agent".into(), "skill".into()],
            category: None,
            min_stars: 100,
            max_results: 20,
        }
    }
}

/// Where a skill was found
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillSource {
    GitHub,
    AgensiMarketplace,
    ClaudeDirectory,
    CommunityRepo,
    Web,
}

/// Skill category taxonomy (mapped from external to internal)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SkillCategory {
    Development,
    Frontend,
    Backend,
    Testing,
    Debugging,
    Security,
    Devops,
    Design,
    Content,
    Research,
    Productivity,
    Communication,
    DataScience,
    Specialized(String),
}

impl SkillCategory {
    pub fn from_external(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "development" | "coding" | "programming" => SkillCategory::Development,
            "frontend" | "ui" | "ux" | "design" => SkillCategory::Frontend,
            "backend" | "api" | "server" => SkillCategory::Backend,
            "testing" | "qa" | "test" => SkillCategory::Testing,
            "debugging" | "debug" => SkillCategory::Debugging,
            "security" | "audit" | "safe" => SkillCategory::Security,
            "devops" | "deploy" | "ci" | "cd" => SkillCategory::Devops,
            "content" | "writing" | "copy" => SkillCategory::Content,
            "research" | "paper" | "literature" => SkillCategory::Research,
            "productivity" | "workflow" => SkillCategory::Productivity,
            "data" | "science" | "ml" | "ai" => SkillCategory::DataScience,
            other => SkillCategory::Specialized(other.to_string()),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            SkillCategory::Development => "Development",
            SkillCategory::Frontend => "Frontend",
            SkillCategory::Backend => "Backend",
            SkillCategory::Testing => "Testing",
            SkillCategory::Debugging => "Debugging",
            SkillCategory::Security => "Security",
            SkillCategory::Devops => "Devops",
            SkillCategory::Design => "Design",
            SkillCategory::Content => "Content",
            SkillCategory::Research => "Research",
            SkillCategory::Productivity => "Productivity",
            SkillCategory::Communication => "Communication",
            SkillCategory::DataScience => "DataScience",
            SkillCategory::Specialized(s) => s.as_str(),
        }
    }
}

/// A skill discovered from external sources
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    pub name: String,
    pub description: String,
    pub source: SkillSource,
    pub source_url: String,
    pub category: SkillCategory,
    pub star_count: u32,
    pub author: Option<String>,
    pub methodology: Vec<String>,
    pub instructions: Option<String>,
    pub tags: Vec<String>,
    pub install_command: Option<String>,
}

/// Skill discovery engine — searches web sources for valuable skills
pub struct SkillDiscovery {
    discovered: Vec<DiscoveredSkill>,
    sources: Vec<String>,
}

impl SkillDiscovery {
    pub fn new() -> Self {
        Self {
            discovered: Vec::new(),
            sources: vec![
                "https://github.com/topics/ai-agent-skills".into(),
                "https://github.com/topics/claude-skills".into(),
                "https://github.com/topics/agent-skills".into(),
            ],
        }
    }

    pub fn register_source(&mut self, url: &str) {
        if !self.sources.contains(&url.to_string()) {
            self.sources.push(url.to_string());
        }
    }

    pub fn search(&self, query: &SearchQuery) -> Vec<DiscoveredSkill> {
        self.discovered
            .iter()
            .filter(|s| {
                let cat_ok = query
                    .category
                    .as_ref()
                    .map(|c| &s.category == c)
                    .unwrap_or(true);
                let stars_ok = s.star_count >= query.min_stars;
                let kw_ok = query.keywords.is_empty()
                    || query.keywords.iter().any(|kw| {
                        s.name.to_lowercase().contains(&kw.to_lowercase())
                            || s.description.to_lowercase().contains(&kw.to_lowercase())
                            || s.tags
                                .iter()
                                .any(|t| t.to_lowercase().contains(&kw.to_lowercase()))
                    });
                cat_ok && stars_ok && kw_ok
            })
            .take(query.max_results)
            .cloned()
            .collect()
    }

    pub fn all_discovered(&self) -> &[DiscoveredSkill] {
        &self.discovered
    }

    pub fn seed(&mut self, skills: Vec<DiscoveredSkill>) {
        self.discovered = skills;
    }

    pub fn with_top_skills() -> Self {
        let skills = vec![
            DiscoveredSkill {
                name: "superpowers".into(),
                description: "Planning-first development methodology: gather context, design plan, implement, verify. Covers the full agent development lifecycle with structured outputs.".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/anthropics/superpowers".into(),
                category: SkillCategory::Development,
                star_count: 1000,
                author: Some("Anthropic".into()),
                methodology: vec!["planning-first".into(), "context-gathering".into(), "design-review".into(), "verification".into()],
                instructions: Some("Use superpowers for tasks requiring structured planning. Start with context gathering, then design, then implement, then verify each step.".into()),
                tags: vec!["planning".into(), "development".into(), "workflow".into()],
                install_command: Some("git clone https://github.com/anthropics/superpowers".into()),
            },
            DiscoveredSkill {
                name: "frontend-design".into(),
                description: "Design system generation for frontend. Creates polished, production-grade UI components with consistent design tokens.".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/anthropics/frontend-design".into(),
                category: SkillCategory::Frontend,
                star_count: 850,
                author: Some("Anthropic".into()),
                methodology: vec!["design-tokens".into(), "component-library".into(), "responsive-layout".into()],
                instructions: Some("Generates complete UI components with consistent spacing, typography, and color palettes.".into()),
                tags: vec!["frontend".into(), "ui".into(), "design".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "systematic-debugging".into(),
                description: "Structured debugging methodology: isolate root cause, test hypotheses, verify fix. Reduces debugging time by 60%.".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/anthropics/systematic-debugging".into(),
                category: SkillCategory::Debugging,
                star_count: 800,
                author: Some("Anthropic".into()),
                methodology: vec!["root-cause-analysis".into(), "hypothesis-testing".into(), "binary-search".into(), "verification".into()],
                instructions: Some("When debugging: 1) Reproduce the issue 2) Isolate variables 3) Form hypothesis 4) Test 5) Verify fix.".into()),
                tags: vec!["debugging".into(), "testing".into(), "quality".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "test-driven-development".into(),
                description: "Red-Green-Refactor TDD cycle with automated test generation and coverage tracking.".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/anthropics/tdd-skill".into(),
                category: SkillCategory::Testing,
                star_count: 700,
                author: Some("Anthropic".into()),
                methodology: vec!["red-green-refactor".into(), "test-first".into(), "coverage-tracking".into()],
                instructions: Some("Always write tests before implementation. Red: failing test. Green: make it pass. Refactor: clean up.".into()),
                tags: vec!["testing".into(), "tdd".into(), "quality".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "code-reviewer".into(),
                description: "Security and quality code review. Identifies vulnerabilities, anti-patterns, and performance issues.".into(),
                source: SkillSource::CommunityRepo,
                source_url: "https://github.com/community/code-reviewer".into(),
                category: SkillCategory::Security,
                star_count: 600,
                author: Some("Community".into()),
                methodology: vec!["static-analysis".into(), "security-scan".into(), "performance-review".into()],
                instructions: Some("Review code for: security vulnerabilities, performance bottlenecks, code smells, and test coverage gaps.".into()),
                tags: vec!["code-review".into(), "security".into(), "quality".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "security-audit".into(),
                description: "Deep security scanning for AI-generated code. Checks for injection, data leaks, and OWASP Top 10 vulnerabilities.".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/security/ai-audit".into(),
                category: SkillCategory::Security,
                star_count: 500,
                author: Some("SecurityLabs".into()),
                methodology: vec!["injection-detection".into(), "data-leak-scan".into(), "owasp-checks".into()],
                instructions: Some("Scans code for: prompt injection, SQL injection, path traversal, hardcoded secrets, and XSS patterns.".into()),
                tags: vec!["security".into(), "audit".into(), "owasp".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "git-workflow".into(),
                description: "Structured git commit, PR, and review workflow. Ensures clean history and proper collaboration patterns.".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/git-workflows/agent-git".into(),
                category: SkillCategory::Development,
                star_count: 500,
                author: Some("GitWorkflows".into()),
                methodology: vec!["commit-conventions".into(), "pr-templates".into(), "review-workflow".into()],
                instructions: Some("Follow conventional commits format. Create focused PRs. Keep commit history clean and reviewable.".into()),
                tags: vec!["git".into(), "workflow".into(), "collaboration".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "ui-ux-pro-max".into(),
                description: "50 design styles, 21 color palettes, and responsive layout components for production UI.".into(),
                source: SkillSource::CommunityRepo,
                source_url: "https://github.com/design/ui-ux-pro".into(),
                category: SkillCategory::Design,
                star_count: 500,
                author: Some("DesignStudio".into()),
                methodology: vec!["design-system".into(), "color-theory".into(), "responsive-grid".into()],
                instructions: Some("Offers pre-built design styles: minimal, bold, playful, elegant. Automatically selects palette matching the brand context.".into()),
                tags: vec!["design".into(), "ui".into(), "ux".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "context-engineering".into(),
                description: "Custom agent system design. Builds optimized context windows and system prompts for specific tasks.".into(),
                source: SkillSource::ClaudeDirectory,
                source_url: "https://claude.ai/skills/context-engineering".into(),
                category: SkillCategory::Development,
                star_count: 400,
                author: Some("AgentArchitects".into()),
                methodology: vec!["context-optimization".into(), "prompt-engineering".into(), "system-design".into()],
                instructions: Some("Design agent context: define the task scope, select relevant tools, structure the system prompt for reliable execution.".into()),
                tags: vec!["agents".into(), "context".into(), "prompting".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "humanizer".into(),
                description: "Remove AI-generated patterns from text. Rewrites to sound natural, varied, and human-like.".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/content/humanizer".into(),
                category: SkillCategory::Content,
                star_count: 400,
                author: Some("ContentLab".into()),
                methodology: vec!["pattern-detection".into(), "style-variation".into(), "natural-flow".into()],
                instructions: Some("Detect and rewrite: repetitive sentence structures, formal transitions, AI clichés. Vary sentence length and rhythm.".into()),
                tags: vec!["content".into(), "writing".into(), "editing".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "planning-with-files".into(),
                description: "Persistent task tracking using file-based plans. Creates, updates, and closes tasks with file-based state management.".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/planners/planning-files".into(),
                category: SkillCategory::Productivity,
                star_count: 300,
                author: Some("PlanTeam".into()),
                methodology: vec!["file-based-tracking".into(), "task-decomposition".into(), "progress-logging".into()],
                instructions: Some("Create a PLAN.md for each project. Track status with tasks, deadlines, and blockers. Update progress after each action.".into()),
                tags: vec!["planning".into(), "productivity".into(), "organization".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "dev-browser".into(),
                description: "Visual browser testing and automation. Clicks, screenshots, form fills, and navigation with stealth mode.".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/browser/dev-browser".into(),
                category: SkillCategory::Testing,
                star_count: 300,
                author: Some("BrowserTeam".into()),
                methodology: vec!["browser-automation".into(), "screenshot-testing".into(), "stealth-mode".into()],
                instructions: Some("launch browser, navigate to URL, interact with elements, capture screenshots for verification.".into()),
                tags: vec!["browser".into(), "testing".into(), "automation".into()],
                install_command: Some("npm install @playwright/test".into()),
            },
            DiscoveredSkill {
                name: "obsidian-skills".into(),
                description: "Knowledge management vault. Bidirectional links, graph visualization, and semantic search for personal knowledge bases.".into(),
                source: SkillSource::CommunityRepo,
                source_url: "https://github.com/knowledge/obsidian-skills".into(),
                category: SkillCategory::Productivity,
                star_count: 200,
                author: Some("KnowledgeWorkers".into()),
                methodology: vec!["bidirectional-links".into(), "graph-navigation".into(), "semantic-search".into()],
                instructions: Some("Create interconnected notes with [[wikilinks]]. Use tags for categorization. Leverage graph view for discovery.".into()),
                tags: vec!["knowledge".into(), "notes".into(), "organization".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "scientific-skills".into(),
                description: "Scientific computing workflows: data analysis, statistical testing, visualization, and paper writing.".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/science/scientific-skills".into(),
                category: SkillCategory::DataScience,
                star_count: 200,
                author: Some("ScienceAI".into()),
                methodology: vec!["data-analysis".into(), "statistical-testing".into(), "visualization".into(), "paper-writing".into()],
                instructions: Some("For scientific tasks: clean data, apply statistical tests, generate publication-ready figures, draft findings.".into()),
                tags: vec!["science".into(), "data".into(), "research".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "devops-pipeline".into(),
                description: "CI/CD pipeline design and implementation. Covers Docker, GitHub Actions, and deployment strategies.".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/devops/pipeline-skills".into(),
                category: SkillCategory::Devops,
                star_count: 200,
                author: Some("DevOpsInc".into()),
                methodology: vec!["ci-cd".into(), "containerization".into(), "deployment".into(), "monitoring".into()],
                instructions: Some("Design pipelines: lint, test, build, deploy stages. Use caching and parallelization for speed.".into()),
                tags: vec!["devops".into(), "ci-cd".into(), "deployment".into()],
                install_command: None,
            },
        ];
        let mut s = Self::new();
        s.seed(skills);
        s
    }
}

impl Default for SkillDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_skills() -> Vec<DiscoveredSkill> {
        vec![
            DiscoveredSkill {
                name: "test-debug".into(),
                description: "Debugging skill for testing".into(),
                source: SkillSource::GitHub,
                source_url: "https://github.com/test/debug".into(),
                category: SkillCategory::Debugging,
                star_count: 150,
                author: Some("Tester".into()),
                methodology: vec!["tracing".into()],
                instructions: None,
                tags: vec!["debug".into()],
                install_command: None,
            },
            DiscoveredSkill {
                name: "web-dev".into(),
                description: "Web development framework".into(),
                source: SkillSource::CommunityRepo,
                source_url: "https://github.com/web/dev".into(),
                category: SkillCategory::Development,
                star_count: 500,
                author: None,
                methodology: vec!["html".into(), "css".into()],
                instructions: None,
                tags: vec!["web".into()],
                install_command: None,
            },
        ]
    }

    #[test]
    fn test_search_returns_results_for_known_category() {
        let mut discovery = SkillDiscovery::with_top_skills();
        discovery.seed(sample_skills());
        let query = SearchQuery {
            keywords: vec![],
            category: Some(SkillCategory::Debugging),
            min_stars: 0,
            max_results: 10,
        };
        let results = discovery.search(&query);
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .all(|s| s.category == SkillCategory::Debugging));
    }

    #[test]
    fn test_with_top_skills_seed_count() {
        let discovery = SkillDiscovery::with_top_skills();
        assert_eq!(discovery.all_discovered().len(), 15);
    }

    #[test]
    fn test_skill_category_from_external_mapping() {
        assert_eq!(
            SkillCategory::from_external("coding"),
            SkillCategory::Development
        );
        assert_eq!(
            SkillCategory::from_external("security"),
            SkillCategory::Security
        );
        assert_eq!(
            SkillCategory::from_external("ml"),
            SkillCategory::DataScience
        );
        assert_eq!(
            SkillCategory::from_external("unknown-foo"),
            SkillCategory::Specialized("unknown-foo".into())
        );
    }

    #[test]
    fn test_default_search_query() {
        let q = SearchQuery::default();
        assert_eq!(q.keywords, vec!["ai", "agent", "skill"]);
        assert_eq!(q.max_results, 20);
        assert!(q.min_stars > 0);
        assert!(q.category.is_none());
    }

    #[test]
    fn test_register_source() {
        let mut discovery = SkillDiscovery::new();
        assert_eq!(discovery.sources.len(), 3);
        discovery.register_source("https://example.com/new-source");
        assert_eq!(discovery.sources.len(), 4);
        discovery.register_source("https://example.com/new-source");
        assert_eq!(discovery.sources.len(), 4);
    }

    #[test]
    fn test_search_keyword_filtering() {
        let mut discovery = SkillDiscovery::new();
        discovery.seed(sample_skills());
        let query = SearchQuery {
            keywords: vec!["debug".into()],
            category: None,
            min_stars: 0,
            max_results: 10,
        };
        let results = discovery.search(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test-debug");
    }

    #[test]
    fn test_min_stars_filtering() {
        let mut discovery = SkillDiscovery::new();
        discovery.seed(sample_skills());
        let query = SearchQuery {
            keywords: vec![],
            category: None,
            min_stars: 300,
            max_results: 10,
        };
        let results = discovery.search(&query);
        assert!(!results.is_empty());
        assert!(results.iter().all(|s| s.star_count >= 300));
    }
}
