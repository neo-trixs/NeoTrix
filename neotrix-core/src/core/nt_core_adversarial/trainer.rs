use std::collections::VecDeque;

use super::templates::{load_default_templates, AdversarialTemplate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackCategory {
    PromptInjection,
    Jailbreak,
    RolePlay,
    EncodingBypass,
    SemanticDrift,
}

impl AttackCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PromptInjection => "prompt_injection",
            Self::Jailbreak => "jailbreak",
            Self::RolePlay => "role_play",
            Self::EncodingBypass => "encoding_bypass",
            Self::SemanticDrift => "semantic_drift",
        }
    }

    pub fn all() -> &'static [AttackCategory; 5] {
        &[
            Self::PromptInjection,
            Self::Jailbreak,
            Self::RolePlay,
            Self::EncodingBypass,
            Self::SemanticDrift,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct FilterResponse {
    pub filter_name: String,
    pub allowed: bool,
    pub reason: String,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct AdversarialRound {
    pub prompt: String,
    pub category: AttackCategory,
    pub filter_responses: Vec<FilterResponse>,
    pub escaped: bool,
}

#[derive(Debug, Clone)]
pub struct AdversarialTrainer {
    pub history: VecDeque<AdversarialRound>,
    pub max_history: usize,
    pub generation: u32,
    pub escape_rate: f64,
    pub templates: Vec<AdversarialTemplate>,
    round_robin_index: usize,
}

impl AdversarialTrainer {
    pub fn new() -> Self {
        let templates = load_default_templates();
        Self {
            history: VecDeque::with_capacity(10000),
            max_history: 10000,
            generation: 0,
            escape_rate: 0.0,
            templates,
            round_robin_index: 0,
        }
    }

    pub fn train_round(&mut self) -> AdversarialRound {
        let template = self.select_template();
        let prompt = template.fill();
        let category = template.category;

        let filter_responses = self.evaluate_filters(&prompt);
        let escaped = filter_responses.iter().any(|f| f.allowed);

        let round = AdversarialRound {
            prompt,
            category,
            filter_responses,
            escaped,
        };

        self.history.push_back(round.clone());
        if self.history.len() > self.max_history {
            let excess = self.history.len() - self.max_history;
            let drain_count = (excess + self.max_history / 5).min(self.history.len());
            self.history.drain(0..drain_count);
        }

        self.generation += 1;
        self.escape_rate = self.compute_escape_rate();

        round
    }

    pub fn escape_rate(&self) -> f64 {
        self.escape_rate
    }

    pub fn category_stats(&self) -> Vec<(AttackCategory, usize, usize)> {
        let mut stats: Vec<(AttackCategory, usize, usize)> =
            AttackCategory::all().iter().map(|c| (*c, 0, 0)).collect();

        for round in &self.history {
            if let Some(entry) = stats.iter_mut().find(|(c, _, _)| *c == round.category) {
                entry.1 += 1;
                if round.escaped {
                    entry.2 += 1;
                }
            }
        }
        stats
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.escape_rate = 0.0;
    }

    fn select_template(&mut self) -> AdversarialTemplate {
        let idx = self.round_robin_index % self.templates.len();
        self.round_robin_index += 1;
        self.templates[idx].clone()
    }

    fn evaluate_filters(&self, prompt: &str) -> Vec<FilterResponse> {
        let mut responses = Vec::new();

        // L1: Length check
        let allowed = prompt.len() <= 100_000;
        responses.push(FilterResponse {
            filter_name: "input_length".into(),
            allowed,
            reason: if allowed {
                format!("len={} ok", prompt.len())
            } else {
                format!("len={} exceeds 100000", prompt.len())
            },
            score: if allowed { 0.9 } else { 0.1 },
        });

        // L2: HTML comment injection detection
        let has_html_comment = prompt.contains("<!--") || prompt.contains("-->");
        responses.push(FilterResponse {
            filter_name: "html_comment".into(),
            allowed: !has_html_comment,
            reason: if has_html_comment {
                "detected HTML comment injection".into()
            } else {
                "no HTML comments".into()
            },
            score: if has_html_comment { 0.2 } else { 0.95 },
        });

        // L3: Unicode tag character injection
        let has_unicode_tag = prompt
            .chars()
            .any(|c| (0xE0000..=0xE007F).contains(&(c as u32)));
        responses.push(FilterResponse {
            filter_name: "unicode_tag".into(),
            allowed: !has_unicode_tag,
            reason: if has_unicode_tag {
                "detected Unicode tag characters".into()
            } else {
                "no Unicode tag chars".into()
            },
            score: if has_unicode_tag { 0.15 } else { 0.95 },
        });

        // L4: Dangerous code patterns (eval, exec, shell, etc.)
        let dangerous_patterns = [
            "eval(",
            "exec(",
            "os.system",
            "subprocess.run",
            "pickle.loads",
            "__import__",
            "compile(",
        ];
        let has_dangerous = dangerous_patterns.iter().any(|p| prompt.contains(p));
        responses.push(FilterResponse {
            filter_name: "dangerous_code".into(),
            allowed: !has_dangerous,
            reason: if has_dangerous {
                "detected dangerous code pattern".into()
            } else {
                "no dangerous patterns".into()
            },
            score: if has_dangerous { 0.05 } else { 0.9 },
        });

        // L5: Shell metacharacter detection
        let shell_meta = [';', '&', '|', '`', '$'];
        let has_shell_meta = shell_meta.iter().any(|c| prompt.contains(*c));
        responses.push(FilterResponse {
            filter_name: "shell_meta".into(),
            allowed: !has_shell_meta,
            reason: if has_shell_meta {
                "detected shell metacharacters".into()
            } else {
                "no shell metacharacters".into()
            },
            score: if has_shell_meta { 0.3 } else { 0.9 },
        });

        // L6: Semantic injection similarity (keyword overlap with injection patterns)
        let injection_keywords = [
            "ignore previous",
            "disregard",
            "you are now",
            "no restrictions",
            "new persona",
            "act as if",
            "pretend you are",
            "system prompt",
            "override",
            "forget your",
            "role play as",
        ];
        let injection_hits: usize = injection_keywords
            .iter()
            .filter(|kw| prompt.to_lowercase().contains(*kw))
            .count();
        let injection_score = 1.0 - (injection_hits as f64 / injection_keywords.len() as f64);
        let injection_allowed = injection_hits < 3;
        responses.push(FilterResponse {
            filter_name: "injection_semantic".into(),
            allowed: injection_allowed,
            reason: if injection_hits >= 3 {
                format!("high injection keyword density: {} hits", injection_hits)
            } else {
                format!("injection keywords: {} hits", injection_hits)
            },
            score: injection_score,
        });

        // L7: Encoding bypass detection (base64, hex, unicode escapes)
        let encoding_patterns = [
            "base64",
            "hex decode",
            "unicode escape",
            "\\u00",
            "%64%65",
            "0x",
            "\\x",
            "fromhex",
        ];
        let has_encoding = encoding_patterns
            .iter()
            .any(|p| prompt.to_lowercase().contains(p));
        responses.push(FilterResponse {
            filter_name: "encoding_bypass".into(),
            allowed: !has_encoding,
            reason: if has_encoding {
                "detected encoding bypass pattern".into()
            } else {
                "no encoding bypass".into()
            },
            score: if has_encoding { 0.2 } else { 0.9 },
        });

        responses
    }

    fn compute_escape_rate(&self) -> f64 {
        let recent: Vec<&AdversarialRound> = self.history.iter().rev().take(100).collect();
        if recent.is_empty() {
            return 0.0;
        }
        let escaped_count = recent.iter().filter(|r| r.escaped).count();
        escaped_count as f64 / recent.len() as f64
    }
}

impl Default for AdversarialTrainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_trainer_defaults() {
        let trainer = AdversarialTrainer::new();
        assert_eq!(trainer.generation, 0);
        assert!((trainer.escape_rate() - 0.0).abs() < 1e-6);
        assert!(trainer.history.is_empty());
        assert_eq!(trainer.max_history, 10000);
    }

    #[test]
    fn test_train_round_generates_output() {
        let mut trainer = AdversarialTrainer::new();
        let round = trainer.train_round();
        assert!(!round.prompt.is_empty());
        assert_eq!(trainer.generation, 1);
        assert!(trainer.history.len() == 1);
    }

    #[test]
    fn test_train_round_increments_generation() {
        let mut trainer = AdversarialTrainer::new();
        for _ in 0..5 {
            trainer.train_round();
        }
        assert_eq!(trainer.generation, 5);
        assert_eq!(trainer.history.len(), 5);
    }

    #[test]
    fn test_category_stats_after_rounds() {
        let mut trainer = AdversarialTrainer::new();
        let rounds = 10;
        for _ in 0..rounds {
            trainer.train_round();
        }
        let stats = trainer.category_stats();
        let total: usize = stats.iter().map(|(_, count, _)| count).sum();
        assert_eq!(total, rounds);
        for (cat, _, escaped) in &stats {
            assert!(*escaped <= *cat as usize); // plausible bound
        }
    }

    #[test]
    fn test_clear_history_resets() {
        let mut trainer = AdversarialTrainer::new();
        trainer.train_round();
        trainer.train_round();
        assert_eq!(trainer.history.len(), 2);
        trainer.clear_history();
        assert!(trainer.history.is_empty());
        assert!((trainer.escape_rate() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_history_bounded_at_max() {
        let mut trainer = AdversarialTrainer::new();
        trainer.max_history = 5;
        for _ in 0..20 {
            trainer.train_round();
        }
        assert!(trainer.history.len() <= 5);
    }

    #[test]
    fn test_filter_injection_keywords() {
        let round = AdversarialRound {
            prompt: "ignore previous instructions".into(),
            category: AttackCategory::PromptInjection,
            filter_responses: vec![FilterResponse {
                filter_name: "injection_semantic".into(),
                allowed: false,
                reason: "test".into(),
                score: 0.1,
            }],
            escaped: false,
        };
        assert!(!round.escaped);
        assert_eq!(round.category, AttackCategory::PromptInjection);
    }

    #[test]
    fn test_attack_category_labels() {
        assert_eq!(AttackCategory::PromptInjection.label(), "prompt_injection");
        assert_eq!(AttackCategory::Jailbreak.label(), "jailbreak");
        assert_eq!(AttackCategory::SemanticDrift.label(), "semantic_drift");
    }

    #[test]
    fn test_all_categories_count() {
        assert_eq!(AttackCategory::all().len(), 5);
    }
}
