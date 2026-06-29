#[derive(Debug, Clone, PartialEq)]
pub enum IntentPattern {
    Direct(String),
    Indirect(String),
    Clarification(String),
    Exploration(String),
    Command(String),
}

impl IntentPattern {
    pub fn label(&self) -> &'static str {
        match self {
            IntentPattern::Direct(_) => "Direct",
            IntentPattern::Indirect(_) => "Indirect",
            IntentPattern::Clarification(_) => "Clarification",
            IntentPattern::Exploration(_) => "Exploration",
            IntentPattern::Command(_) => "Command",
        }
    }

    pub fn inner(&self) -> &str {
        match self {
            IntentPattern::Direct(s)
            | IntentPattern::Indirect(s)
            | IntentPattern::Clarification(s)
            | IntentPattern::Exploration(s)
            | IntentPattern::Command(s) => s,
        }
    }
}

pub fn match_patterns(query: &str) -> Vec<(IntentPattern, f64)> {
    let q = query.trim().to_lowercase();
    let mut results: Vec<(IntentPattern, f64)> = Vec::new();

    if q.is_empty() {
        return results;
    }

    let direct_verbs = [
        "do ",
        "write ",
        "create ",
        "make ",
        "build ",
        "show ",
        "tell ",
        "give ",
        "find ",
        "implement ",
        "fix ",
        "update ",
        "delete ",
        "remove ",
        "add ",
        "change ",
        "refactor ",
        "optimize ",
    ];
    if direct_verbs.iter().any(|v| q.starts_with(v)) {
        results.push((IntentPattern::Direct(query.trim().to_string()), 0.85));
    }

    if q.starts_with("please ") {
        let rest = q.trim_start_matches("please ").trim();
        if direct_verbs.iter().any(|v| rest.starts_with(v)) {
            results.push((IntentPattern::Direct(query.trim().to_string()), 0.8));
        }
    }

    let command_triggers = [
        "run ", "execute ", "sudo ", "npx ", "cargo ", "npm ", "pip ", "bash ", "!command", "git ",
        "docker ", "make ",
    ];
    if command_triggers.iter().any(|t| q.starts_with(t)) {
        results.push((IntentPattern::Command(query.trim().to_string()), 0.9));
    }

    let clar_triggers = [
        "what do you mean",
        "can you explain",
        "i don't understand",
        "clarify",
        "what does",
        "how does",
        "explain ",
        "what is the meaning",
        "could you elaborate",
    ];
    if clar_triggers.iter().any(|t| q.contains(t)) {
        results.push((IntentPattern::Clarification(query.trim().to_string()), 0.9));
    }

    let explore_triggers = [
        "what if",
        "how about",
        "tell me about",
        "explore ",
        "investigate",
        "research ",
        "find out",
        "i wonder what",
        "tell me more",
        "what is ",
        "what are ",
    ];
    if explore_triggers.iter().any(|t| q.contains(t)) {
        results.push((IntentPattern::Exploration(query.trim().to_string()), 0.8));
    }

    let indirect_triggers = [
        "i wonder if",
        "maybe we could",
        "perhaps ",
        "what about thinking",
        "i was thinking",
        "how would we",
        "could we",
        "might be able to",
        "it would be nice if",
        "i'm thinking",
    ];
    if indirect_triggers.iter().any(|t| q.contains(t)) {
        results.push((IntentPattern::Indirect(query.trim().to_string()), 0.75));
    }

    if results.is_empty() {
        if q.ends_with('?') {
            results.push((IntentPattern::Exploration(query.trim().to_string()), 0.5));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_command_matches() {
        let results = match_patterns("write a sorting algorithm");
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|(p, _)| matches!(p, IntentPattern::Direct(_))));
    }

    #[test]
    fn test_exploration_pattern_matches() {
        let results = match_patterns("what if we used a different data structure");
        assert!(results
            .iter()
            .any(|(p, _)| matches!(p, IntentPattern::Exploration(_))));
    }

    #[test]
    fn test_clarification_pattern_matches() {
        let results = match_patterns("can you explain how this works");
        assert!(results
            .iter()
            .any(|(p, _)| matches!(p, IntentPattern::Clarification(_))));
    }

    #[test]
    fn test_indirect_pattern_matches() {
        let results = match_patterns("i wonder if we could optimize this");
        assert!(results
            .iter()
            .any(|(p, _)| matches!(p, IntentPattern::Indirect(_))));
    }

    #[test]
    fn test_command_pattern_matches() {
        let results = match_patterns("run the tests");
        assert!(results
            .iter()
            .any(|(p, _)| matches!(p, IntentPattern::Command(_))));
    }

    #[test]
    fn test_empty_query_returns_empty() {
        let results = match_patterns("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_question_marked_as_exploration_fallback() {
        let results = match_patterns("is this the right approach?");
        assert!(results
            .iter()
            .any(|(p, _)| matches!(p, IntentPattern::Exploration(_))));
    }

    #[test]
    fn test_please_direct_matches() {
        let results = match_patterns("please create a new file");
        assert!(results
            .iter()
            .any(|(p, _)| matches!(p, IntentPattern::Direct(_))));
    }

    #[test]
    fn test_pattern_label_is_correct() {
        let pat = IntentPattern::Command("run build".to_string());
        assert_eq!(pat.label(), "Command");
        assert_eq!(pat.inner(), "run build");
    }

    #[test]
    fn test_no_false_positive_for_normal_statement() {
        let results = match_patterns("the sky is blue");
        assert!(results.is_empty());
    }
}
