use super::reasoning_types::ContextTier;

pub struct TierPromptBuilder;

impl TierPromptBuilder {
    fn tier_header(tier: ContextTier) -> String {
        format!(
            "[ContextTier: {} | max_tool_calls: {} | max_search_results: {}]",
            tier.name(),
            tier.max_tool_calls(),
            tier.max_search_results()
        )
    }

    pub fn build_context_prompt(tier: ContextTier, query: &str) -> String {
        let header = Self::tier_header(tier);
        let instruction = match tier {
            ContextTier::Small => {
                "Brief search. Return max 3 key findings. Be concise."
            }
            ContextTier::Medium => {
                "Balanced search. Return top findings organized by relevance."
            }
            ContextTier::Large => {
                "Thorough exploration. Include relationships and architecture context."
            }
            ContextTier::Massive => {
                "Comprehensive analysis. Full dependency chains, all connections."
            }
        };
        format!(
            "{}\n{}\n\nContext query: {}",
            header, instruction, query
        )
    }

    pub fn build_impact_prompt(tier: ContextTier, target: &str) -> String {
        let header = Self::tier_header(tier);
        let instruction = match tier {
            ContextTier::Small => {
                "Quick impact scan. Show only direct dependencies (depth 1-2)."
            }
            ContextTier::Medium => {
                "Standard impact analysis. Include depth 3."
            }
            ContextTier::Large => {
                "Deep impact. Full transitive dependencies."
            }
            ContextTier::Massive => {
                "Exhaustive. All depths, all edge types, architectural implications."
            }
        };
        format!(
            "{}\n{}\n\nImpact target: {}",
            header, instruction, target
        )
    }

    pub fn build_architecture_prompt(tier: ContextTier) -> String {
        let header = Self::tier_header(tier);
        let instruction = match tier {
            ContextTier::Small => {
                "High-level architecture overview. List top-level modules only."
            }
            ContextTier::Medium => {
                "Moderate-depth architecture. Show module hierarchy and key interfaces."
            }
            ContextTier::Large => {
                "Detailed architecture. Include internal module structure, trait bounds, and data flow."
            }
            ContextTier::Massive => {
                "Full architecture blueprint. Module dependency graphs, trait hierarchy, data ownership chains, and layer boundaries."
            }
        };
        format!("{}\n{}", header, instruction)
    }

    pub fn build_quality_prompt(tier: ContextTier) -> String {
        let header = Self::tier_header(tier);
        let instruction = match tier {
            ContextTier::Small => {
                "Quick quality scan. Check for obvious issues: unwrap, dead code, large functions."
            }
            ContextTier::Medium => {
                "Standard quality review. Check error handling, unsafe blocks, type safety, test coverage gaps."
            }
            ContextTier::Large => {
                "Deep quality audit. Evaluate API ergonomics, trait coherence, module coupling, and documentation coverage."
            }
            ContextTier::Massive => {
                "Exhaustive quality assessment. Full static analysis: soundness, Send+Sync correctness, orphan rules, unsound patterns, and architectural anti-patterns."
            }
        };
        format!("{}\n{}", header, instruction)
    }

    pub fn build_system_prompt(tier: ContextTier, tool_name: &str, query: &str) -> String {
        let header = Self::tier_header(tier);
        let (task_label, prompt_body) = match tool_name {
            "context" | "search" => {
                ("Context Gathering", Self::build_context_prompt(tier, query))
            }
            "impact" | "dependencies" => {
                ("Impact Analysis", Self::build_impact_prompt(tier, query))
            }
            "architecture" => {
                ("Architecture Analysis", Self::build_architecture_prompt(tier))
            }
            "quality" | "lint" | "review" => {
                ("Quality Assessment", Self::build_quality_prompt(tier))
            }
            other => {
                let fallback = format!(
                    "Generic analysis using tool '{}'.\nQuery: {}",
                    other, query
                );
                ("Analysis", fallback)
            }
        };
        format!(
            "=== {} ===\n{}\n\nTool: {}\n{}",
            task_label, header, tool_name, prompt_body
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_prompt_includes_tier_name() {
        let prompt = TierPromptBuilder::build_context_prompt(ContextTier::Small, "test query");
        assert!(prompt.contains("small"));
        assert!(prompt.contains("max_tool_calls: 3"));
    }

    #[test]
    fn test_context_prompt_includes_query() {
        let prompt = TierPromptBuilder::build_context_prompt(ContextTier::Medium, "find the error handler");
        assert!(prompt.contains("find the error handler"));
    }

    #[test]
    fn test_context_prompt_variations() {
        let small = TierPromptBuilder::build_context_prompt(ContextTier::Small, "x");
        let massive = TierPromptBuilder::build_context_prompt(ContextTier::Massive, "x");
        assert!(small.len() < massive.len());
        assert!(small.contains("max 3"));
        assert!(massive.contains("Full dependency chains"));
    }

    #[test]
    fn test_impact_prompt_depth() {
        let small = TierPromptBuilder::build_impact_prompt(ContextTier::Small, "foo");
        let deep = TierPromptBuilder::build_impact_prompt(ContextTier::Large, "foo");
        assert!(small.contains("depth 1-2"));
        assert!(deep.contains("Full transitive"));
        assert!(!deep.contains("depth 1-2"));
    }

    #[test]
    fn test_architecture_prompt_includes_max_calls() {
        let prompt = TierPromptBuilder::build_architecture_prompt(ContextTier::Medium);
        assert!(prompt.contains("max_tool_calls: 5"));
    }

    #[test]
    fn test_quality_prompt_scales() {
        let quick = TierPromptBuilder::build_quality_prompt(ContextTier::Small);
        let full = TierPromptBuilder::build_quality_prompt(ContextTier::Massive);
        assert!(quick.contains("Quick quality scan"));
        assert!(full.contains("Exhaustive quality assessment"));
        assert!(full.contains("Send+Sync"));
    }

    #[test]
    fn test_system_prompt_routes_correctly() {
        let ctx = TierPromptBuilder::build_system_prompt(ContextTier::Small, "search", "find x");
        assert!(ctx.contains("Context Gathering"));
        assert!(ctx.contains("max 3 key findings"));

        let impact = TierPromptBuilder::build_system_prompt(ContextTier::Large, "impact", "mod.rs");
        assert!(impact.contains("Impact Analysis"));
        assert!(impact.contains("Full transitive"));

        let arch = TierPromptBuilder::build_system_prompt(ContextTier::Medium, "architecture", "");
        assert!(arch.contains("Architecture Analysis"));
        assert!(arch.contains("Moderate-depth"));

        let quality = TierPromptBuilder::build_system_prompt(ContextTier::Massive, "review", "");
        assert!(quality.contains("Quality Assessment"));
        assert!(quality.contains("Send+Sync"));
    }

    #[test]
    fn test_system_prompt_unknown_tool_falls_back() {
        let prompt = TierPromptBuilder::build_system_prompt(ContextTier::Small, "custom_tool", "do something");
        assert!(prompt.contains("Analysis"));
        assert!(prompt.contains("Generic analysis"));
        assert!(prompt.contains("custom_tool"));
    }
}
