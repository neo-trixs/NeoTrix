use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiPatternCategory {
    Content,
    LanguageGrammar,
    Style,
    Communication,
    FillerHedging,
}

impl AiPatternCategory {
    pub fn all() -> &'static [AiPatternCategory] {
        &[
            AiPatternCategory::Content,
            AiPatternCategory::LanguageGrammar,
            AiPatternCategory::Style,
            AiPatternCategory::Communication,
            AiPatternCategory::FillerHedging,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            AiPatternCategory::Content => "Content / Substance",
            AiPatternCategory::LanguageGrammar => "Language & Grammar",
            AiPatternCategory::Style => "Style & Tone",
            AiPatternCategory::Communication => "Communication",
            AiPatternCategory::FillerHedging => "Filler & Hedging",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiPattern {
    pub id: usize,
    pub name: &'static str,
    pub category: AiPatternCategory,
    pub indicators: &'static [&'static str],
    pub severity: f64,
}

pub static AI_PATTERNS: &[AiPattern] = &[
    AiPattern { id: 1, name: "significance_inflation", category: AiPatternCategory::Content, indicators: &["pivotal moment", "stands as a testament", "marks a", "revolution in", "era of"], severity: 0.8 },
    AiPattern { id: 2, name: "notability_name_dropping", category: AiPatternCategory::Content, indicators: &["cited in", "featured in", "published in"], severity: 0.6 },
    AiPattern { id: 3, name: "superficial_ing_analyses", category: AiPatternCategory::Content, indicators: &["symbolizing", "reflecting", "showcasing", "highlighting", "underscoring"], severity: 0.7 },
    AiPattern { id: 4, name: "promotional_language", category: AiPatternCategory::Content, indicators: &["nestled in", "breathtaking", "vibrant", "stunning", "picturesque"], severity: 0.8 },
    AiPattern { id: 5, name: "vague_attributions", category: AiPatternCategory::Content, indicators: &["experts believe", "it is said that", "some argue", "many consider"], severity: 0.6 },
    AiPattern { id: 6, name: "formulaic_challenges", category: AiPatternCategory::Content, indicators: &["despite challenges", "continues to thrive", "remains resilient", "faces obstacles"], severity: 0.5 },
    AiPattern { id: 7, name: "ai_vocabulary", category: AiPatternCategory::LanguageGrammar, indicators: &["testament", "landscape", "showcasing", "tapestry", "realm", "nuanced", "intricate"], severity: 0.9 },
    AiPattern { id: 8, name: "copula_avoidance", category: AiPatternCategory::LanguageGrammar, indicators: &["serves as", "features", "boasts", "offers", "provides"], severity: 0.7 },
    AiPattern { id: 9, name: "negative_parallelisms", category: AiPatternCategory::LanguageGrammar, indicators: &["it's not just", "it's not only", "more than just", "not merely"], severity: 0.6 },
    AiPattern { id: 10, name: "rule_of_three", category: AiPatternCategory::LanguageGrammar, indicators: &["innovation, inspiration", "past, present", "mind, body", "work, life"], severity: 0.5 },
    AiPattern { id: 11, name: "synonym_cycling", category: AiPatternCategory::LanguageGrammar, indicators: &["protagonist", "main character", "central figure"], severity: 0.7 },
    AiPattern { id: 12, name: "false_ranges", category: AiPatternCategory::LanguageGrammar, indicators: &["from the big bang", "from a to z", "from beginner to"], severity: 0.5 },
    AiPattern { id: 13, name: "passive_subjectless", category: AiPatternCategory::LanguageGrammar, indicators: &["no configuration file needed", "it is recommended", "it is advised"], severity: 0.4 },
    AiPattern { id: 14, name: "em_dash_overuse", category: AiPatternCategory::Style, indicators: &["\u{2014}", "\u{2013}"], severity: 0.5 },
    AiPattern { id: 15, name: "boldface_overuse", category: AiPatternCategory::Style, indicators: &["**okr", "**kpi", "**roi"], severity: 0.4 },
    AiPattern { id: 16, name: "inline_header_lists", category: AiPatternCategory::Style, indicators: &["**performance:**", "**overview:**", "**summary:**"], severity: 0.5 },
    AiPattern { id: 17, name: "title_case_headings", category: AiPatternCategory::Style, indicators: &["strategic negotiations and partnerships", "artificial intelligence and"], severity: 0.3 },
    AiPattern { id: 18, name: "emojis", category: AiPatternCategory::Style, indicators: &["\u{1f680}", "\u{2728}", "\u{1f389}", "\u{1f4a1}"], severity: 0.6 },
    AiPattern { id: 19, name: "curly_quotes", category: AiPatternCategory::Style, indicators: &["\u{201c}", "\u{201d}", "\u{2018}", "\u{2019}"], severity: 0.2 },
    AiPattern { id: 20, name: "hyphenated_word_pairs", category: AiPatternCategory::Style, indicators: &["data-driven", "cross-functional", "client-facing", "enterprise-grade"], severity: 0.4 },
    AiPattern { id: 21, name: "persuasive_authority_tropes", category: AiPatternCategory::Style, indicators: &["at its core", "what matters is", "the truth is", "the fact remains"], severity: 0.5 },
    AiPattern { id: 22, name: "signposting_announcements", category: AiPatternCategory::Style, indicators: &["let's dive in", "here's what you need", "without further ado", "let's explore"], severity: 0.7 },
    AiPattern { id: 23, name: "fragmented_headers", category: AiPatternCategory::Style, indicators: &["## performance. speed matters.", "## overview. the system"], severity: 0.4 },
    AiPattern { id: 24, name: "diff_anchored_writing", category: AiPatternCategory::Style, indicators: &["was added to replace", "was introduced to", "has been updated to"], severity: 0.5 },
    AiPattern { id: 25, name: "manufactured_punchlines", category: AiPatternCategory::Style, indicators: &["it had no", "no preference. no prior. no nostalgia"], severity: 0.6 },
    AiPattern { id: 26, name: "aphorism_formulas", category: AiPatternCategory::Style, indicators: &["is the language of", "is the key to", "is the foundation of"], severity: 0.5 },
    AiPattern { id: 27, name: "conversational_rhetorical_openers", category: AiPatternCategory::Style, indicators: &["honestly? it depends", "the short answer", "simply put"], severity: 0.4 },
    AiPattern { id: 28, name: "chatbot_artifacts", category: AiPatternCategory::Communication, indicators: &["i hope this helps", "let me know if you", "feel free to ask", "is there anything else"], severity: 0.9 },
    AiPattern { id: 29, name: "cutoff_disclaimers", category: AiPatternCategory::Communication, indicators: &["while details are limited", "based on available", "to the best of my"], severity: 0.5 },
    AiPattern { id: 30, name: "sycophantic_tone", category: AiPatternCategory::Communication, indicators: &["great question", "you're absolutely right", "excellent point", "that's a great"], severity: 0.8 },
    AiPattern { id: 31, name: "filler_phrases", category: AiPatternCategory::FillerHedging, indicators: &["in order to", "due to the fact that", "at the end of the day", "in the process of"], severity: 0.6 },
    AiPattern { id: 32, name: "excessive_hedging", category: AiPatternCategory::FillerHedging, indicators: &["could potentially", "may possibly", "might perhaps", "it is possible that"], severity: 0.7 },
    AiPattern { id: 33, name: "generic_conclusions", category: AiPatternCategory::FillerHedging, indicators: &["the future looks bright", "only time will tell", "the possibilities are endless", "the journey continues"], severity: 0.6 },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub pattern_matches: Vec<PatternMatch>,
    pub total_severity: f64,
    pub pattern_count: usize,
    pub category_breakdown: HashMap<String, usize>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_id: usize,
    pub pattern_name: String,
    pub category: String,
    pub count: usize,
    pub severity: f64,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceProfile {
    pub avg_sentence_length: f64,
    pub sentence_length_variance: f64,
    pub word_complexity: f64,
    pub dash_frequency: f64,
    pub parenthetical_frequency: f64,
    pub transition_style: f64,
    pub signature_phrases: Vec<String>,
    pub paragraph_opening_style: f64,
}

impl VoiceProfile {
    pub fn from_text(text: &str) -> Self {
        let sentences: Vec<&str> = text
            .split(|c: char| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let total = sentences.len().max(1);
        let avg_sentence_length = sentences.iter().map(|s| s.len()).sum::<usize>() as f64 / total as f64;
        let sentence_length_variance = if sentences.len() > 1 {
            sentences.iter().map(|s| {
                let diff = s.len() as f64 - avg_sentence_length;
                diff * diff
            }).sum::<f64>() / total as f64
        } else { 0.0 };

        let words: Vec<&str> = text.split_whitespace().collect();
        let long_words = words.iter().filter(|w| w.len() > 8).count();
        let word_complexity = long_words as f64 / words.len().max(1) as f64;

        let dash_count = text.matches('\u{2014}').count() + text.matches("--").count();
        let dash_frequency = dash_count as f64 / total.max(1) as f64;

        let paren_count = text.matches('(').count() + text.matches('[').count();
        let parenthetical_frequency = paren_count as f64 / total.max(1) as f64;

        let parastarts_immediate = text.lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#')
                && l.len() == l.trim_start().len())
            .count();
        let parastarts_total = text.lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            .count();
        let paragraph_opening_style = if parastarts_total > 0 {
            parastarts_immediate as f64 / parastarts_total as f64
        } else { 0.5 };

        let signature_phrases = vec![];

        Self {
            avg_sentence_length,
            sentence_length_variance,
            word_complexity,
            dash_frequency,
            parenthetical_frequency,
            transition_style: 0.5,
            signature_phrases,
            paragraph_opening_style,
        }
    }

    pub fn similarity(&self, other: &VoiceProfile) -> f64 {
        let sent_diff = (self.avg_sentence_length - other.avg_sentence_length).abs() / 100.0;
        let var_diff = (self.sentence_length_variance - other.sentence_length_variance).abs() / 200.0;
        let word_diff = (self.word_complexity - other.word_complexity).abs();
        let dash_diff = (self.dash_frequency - other.dash_frequency).abs();
        let paren_diff = (self.parenthetical_frequency - other.parenthetical_frequency).abs();
        let para_diff = (self.paragraph_opening_style - other.paragraph_opening_style).abs();
        let raw = sent_diff + var_diff + word_diff + dash_diff + paren_diff + para_diff;
        (1.0 - raw / 6.0).max(0.0).min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerplexityMetrics {
    pub avg_word_length: f64,
    pub vocab_richness: f64,
    pub stopword_ratio: f64,
    pub repetitiveness: f64,
    pub perplexity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstinessMetrics {
    pub sentence_lengths: Vec<usize>,
    pub min_sentence: usize,
    pub max_sentence: usize,
    pub std_dev: f64,
    pub burstiness_score: f64,
}

pub struct HumanizeDetector;

impl HumanizeDetector {
    pub fn detect_all(text: &str) -> DetectionResult {
        let lower = text.to_lowercase();
        let mut matches = Vec::new();
        let mut category_counts: HashMap<String, usize> = HashMap::new();
        let mut total_severity = 0.0;

        for pattern in AI_PATTERNS {
            let mut count = 0;
            let mut examples = Vec::new();
            for indicator in pattern.indicators {
                let positions: Vec<usize> = lower.match_indices(indicator).map(|(i, _)| i).collect();
                count += positions.len();
                if let Some(&pos) = positions.first() {
                    let start = pos.saturating_sub(10);
                    let end = (pos + indicator.len() + 20).min(text.len());
                    examples.push(text[start..end].to_string());
                }
            }
            if count > 0 {
                total_severity += pattern.severity * count.min(5) as f64;
                let cat_label = pattern.category.label().to_string();
                *category_counts.entry(cat_label.clone()).or_insert(0) += count;
                matches.push(PatternMatch {
                    pattern_id: pattern.id,
                    pattern_name: pattern.name.to_string(),
                    category: cat_label,
                    count,
                    severity: pattern.severity,
                    examples,
                });
            }
        }

        matches.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap_or(std::cmp::Ordering::Equal));

        let text_len = text.len().max(1) as f64;
        let raw_confidence = (total_severity / text_len * 1000.0).min(5.0) / 5.0;
        let confidence = (raw_confidence * 0.7 + (matches.len() as f64 / 10.0).min(1.0) * 0.3).max(0.0).min(1.0);

        DetectionResult {
            pattern_matches: matches,
            total_severity,
            pattern_count: text.len(),
            category_breakdown: category_counts,
            confidence,
        }
    }

    pub fn compute_perplexity(text: &str) -> PerplexityMetrics {
        let words: Vec<&str> = text.split_whitespace().collect();
        let total_words = words.len().max(1);

        let avg_word_length = words.iter().map(|w| w.len()).sum::<usize>() as f64 / total_words as f64;

        let unique_words: std::collections::HashSet<&str> = words.iter().map(|w| w.trim_matches(|c: char| !c.is_alphanumeric())).filter(|w| !w.is_empty()).collect();
        let vocab_richness = unique_words.len() as f64 / total_words as f64;

        let stopwords = ["the", "a", "an", "is", "are", "was", "were", "it", "this", "that", "to", "of", "in", "for", "on", "and", "or", "but", "with", "as", "at", "by", "from", "be", "has", "have", "do", "does", "not"];
        let stopword_count = words.iter().filter(|w| stopwords.contains(&w.to_lowercase().as_str())).count();
        let stopword_ratio = stopword_count as f64 / total_words as f64;

        let word_freq: HashMap<&str, usize> = {
            let mut m = HashMap::new();
            for w in &words {
                let w = w.trim_matches(|c: char| !c.is_alphanumeric());
                if !w.is_empty() {
                    *m.entry(w).or_insert(0) += 1;
                }
            }
            m
        };
        let max_freq = word_freq.values().copied().max().unwrap_or(1) as f64;
        let repetitiveness = max_freq / total_words as f64;

        let prob_score = (vocab_richness * 2.0 + (1.0 - stopword_ratio) + avg_word_length * 0.1) / 3.0;
        let perplexity_score = (1.0 / (prob_score + 0.01)).min(10.0) / 10.0;

        PerplexityMetrics { avg_word_length, vocab_richness, stopword_ratio, repetitiveness, perplexity_score }
    }

    pub fn compute_burstiness(text: &str) -> BurstinessMetrics {
        let sentence_lengths: Vec<usize> = text
            .split(|c: char| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim().split_whitespace().count())
            .filter(|&c| c > 0)
            .collect();

        if sentence_lengths.is_empty() {
            return BurstinessMetrics {
                sentence_lengths: vec![],
                min_sentence: 0,
                max_sentence: 0,
                std_dev: 0.0,
                burstiness_score: 0.0,
            };
        }

        let min_sentence = *sentence_lengths.iter().min().unwrap();
        let max_sentence = *sentence_lengths.iter().max().unwrap();
        let mean = sentence_lengths.iter().sum::<usize>() as f64 / sentence_lengths.len() as f64;
        let variance = sentence_lengths.iter().map(|&l| {
            let diff = l as f64 - mean;
            diff * diff
        }).sum::<f64>() / sentence_lengths.len() as f64;
        let std_dev = variance.sqrt();

        let _range_ratio = if max_sentence > 0 {
            (max_sentence - min_sentence) as f64 / max_sentence as f64
        } else { 0.0 };

        let burstiness_score = (std_dev / (mean + 1.0) * 2.0).min(1.0).max(0.0);

        BurstinessMetrics { sentence_lengths, min_sentence, max_sentence, std_dev, burstiness_score }
    }

    pub fn humanize_with_voice(text: &str, voice: Option<&VoiceProfile>) -> String {
        let _ = voice;
        let patterns = AI_PATTERNS;
        let mut result = text.to_string();

        for pattern in patterns {
            let mut all_indicators_flat = Vec::new();
            for &indicator in pattern.indicators {
                all_indicators_flat.push(indicator);
            }
            for indicator in &all_indicators_flat {
                let replacement: &str = match pattern.id {
                    7 | 11 => Self::synonym_replacement(indicator),
                    8 => Self::copula_replace(indicator),
                    13 => "you need ",
                    14 => " - ",
                    15 | 16 => "",
                    18 => "",
                    19 => "\"",
                    28 => "",
                    30 => "",
                    _ => continue,
                };
                let replacement = replacement.to_string();
                if !replacement.is_empty() || pattern.id >= 14 {
                    let lower = result.to_lowercase();
                    if let Some(pos) = lower.find(indicator) {
                        if pattern.id >= 14 && replacement.is_empty() {
                            let before = &result[..pos];
                            let after = &result[pos + indicator.len()..];
                            result = format!("{}{}", before, after);
                        } else {
                            let before = &result[..pos];
                            let after = &result[pos + indicator.len()..];
                            result = format!("{}{}{}", before, replacement, after);
                        }
                    }
                }
            }
        }
        result
    }

    fn synonym_replacement(word: &str) -> &'static str {
        let _ = word;
        match word.to_lowercase().as_str() {
            "delve" => "explore",
            "leverage" => "use",
            "robust" => "solid",
            "streamline" => "simplify",
            "comprehensive" => "thorough",
            "pivotal" => "key",
            "foster" => "build",
            "facilitate" => "help",
            "utilize" => "use",
            "testament" => "example",
            "landscape" => "field",
            "tapestry" => "variety",
            "realm" => "area",
            "nuanced" => "subtle",
            "intricate" => "detailed",
            "showcasing" => "showing",
            _ => "it",
        }
    }

    fn copula_replace(phrase: &str) -> &'static str {
        let _ = phrase;
        if phrase.contains("serves as") { "is" }
        else if phrase.contains("features") { "has" }
        else if phrase.contains("boasts") { "has" }
        else if phrase.contains("offers") { "provides" }
        else if phrase.contains("provides") { "gives" }
        else { "it" }
    }

    pub fn summary_report(text: &str, voice_sample: Option<&str>) -> String {
        let detection = Self::detect_all(text);
        let perplexity = Self::compute_perplexity(text);
        let burstiness = Self::compute_burstiness(text);

        let mut report = String::new();
        report.push_str(&format!("=== AI Writing Detection Report ===\n"));
        report.push_str(&format!("Confidence: {:.1}%\n", detection.confidence * 100.0));
        report.push_str(&format!("Patterns detected: {}\n", detection.pattern_matches.len()));
        report.push_str(&format!("Perplexity score: {:.3} (higher = more natural)\n", 1.0 - perplexity.perplexity_score));
        report.push_str(&format!("Burstiness score: {:.3} (higher = more natural)\n", burstiness.burstiness_score));

        if !detection.category_breakdown.is_empty() {
            report.push_str("\nCategory breakdown:\n");
            let mut cats: Vec<_> = detection.category_breakdown.iter().collect();
            cats.sort_by(|a, b| b.1.cmp(a.1));
            for (cat, count) in &cats {
                report.push_str(&format!("  {}: {}\n", cat, count));
            }
        }

        if let Some(top) = detection.pattern_matches.first() {
            report.push_str(&format!("\nTop pattern: {} (severity: {:.1})\n", top.pattern_name, top.severity));
            if let Some(ex) = top.examples.first() {
                report.push_str(&format!("  Example: \"{}...\"\n", ex.chars().take(60).collect::<String>()));
            }
        }

        if let Some(sample) = voice_sample {
            let profile = VoiceProfile::from_text(sample);
            let current_profile = VoiceProfile::from_text(text);
            let similarity = profile.similarity(&current_profile);
            report.push_str(&format!("\nVoice similarity to sample: {:.1}%\n", similarity * 100.0));
        }

        if burstiness.burstiness_score < 0.3 {
            report.push_str("\n⚠️ Low burstiness — sentence lengths too uniform. Vary between short (3-6 words) and long (20-35 words).\n");
        }
        if perplexity.perplexity_score > 0.7 {
            report.push_str("⚠️ High perplexity — word choices too predictable. Use less common but natural alternatives.\n");
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_empty() {
        let r = HumanizeDetector::detect_all("");
        assert_eq!(r.pattern_matches.len(), 0);
        assert_eq!(r.confidence, 0.0);
    }

    #[test]
    fn test_detect_ai_text() {
        let text = "This stands as a testament to the vibrant landscape of innovation. Furthermore, experts believe it leverages robust frameworks. I hope this helps!";
        let r = HumanizeDetector::detect_all(text);
        assert!(r.pattern_matches.len() >= 3, "expected >=3 patterns, got {}", r.pattern_matches.len());
        assert!(r.confidence > 0.3);
    }

    #[test]
    fn test_detect_natural_text() {
        let text = "The cat sat on the mat. It was a sunny day. Birds sang in the trees.";
        let r = HumanizeDetector::detect_all(text);
        assert_eq!(r.pattern_matches.len(), 0);
        assert_eq!(r.confidence, 0.0);
    }

    #[test]
    fn test_pattern_categories_all_represented() {
        let mut cats = std::collections::HashSet::new();
        for p in AI_PATTERNS {
            cats.insert(p.category);
        }
        assert_eq!(cats.len(), 5);
    }

    #[test]
    fn test_voice_profile_similarity() {
        let a = VoiceProfile::from_text("This is a test. It has multiple sentences. Each one is different.");
        let b = VoiceProfile::from_text("This is another test. It also has sentences. They vary somewhat.");
        let sim = a.similarity(&b);
        assert!(sim > 0.3);
    }

    #[test]
    fn test_voice_profile_identical() {
        let text = "This is a test. It has multiple sentences.";
        let a = VoiceProfile::from_text(text);
        let b = VoiceProfile::from_text(text);
        assert!((a.similarity(&b) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_perplexity_metrics() {
        let p = HumanizeDetector::compute_perplexity("The quick brown fox jumps over the lazy dog");
        assert!(p.avg_word_length > 3.0);
        assert!(p.vocab_richness > 0.5);
    }

    #[test]
    fn test_perplexity_repetitive() {
        let p = HumanizeDetector::compute_perplexity("test test test test test");
        assert!(p.repetitiveness > 0.5);
    }

    #[test]
    fn test_burstiness_metrics() {
        let b = HumanizeDetector::compute_burstiness("Short. Medium sentence here. This is a much longer sentence with many more words to test variance.");
        assert!(b.max_sentence > b.min_sentence);
        assert!(b.std_dev > 0.0);
    }

    #[test]
    fn test_burstiness_uniform() {
        let b = HumanizeDetector::compute_burstiness("Short sentence. Short sentence. Short sentence.");
        assert!(b.burstiness_score < 0.3);
    }

    #[test]
    fn test_burstiness_empty() {
        let b = HumanizeDetector::compute_burstiness("");
        assert_eq!(b.sentence_lengths.len(), 0);
        assert_eq!(b.burstiness_score, 0.0);
    }

    #[test]
    fn test_summary_report() {
        let text = "This stands as a testament. Experts believe it leverages robust frameworks.";
        let report = HumanizeDetector::summary_report(text, None);
        assert!(report.contains("AI Writing Detection"));
        assert!(report.contains("Confidence:"));
    }

    #[test]
    fn test_summary_report_with_voice() {
        let text = "Short. Normal length sentence here.";
        let sample = "I tend to write in a casual style. My sentences vary in length. Sometimes they're quite long and detailed.";
        let report = HumanizeDetector::summary_report(text, Some(sample));
        assert!(report.contains("Voice similarity"));
    }

    #[test]
    fn test_humanize_with_voice_strips_patterns() {
        let text = "This stands as a testament. I hope this helps! Furthermore, it leverages robust frameworks.";
        let result = HumanizeDetector::humanize_with_voice(text, None);
        assert!(result.len() < text.len() || !result.contains("testament"), "should have modified text but got: {}", result);
    }

    #[test]
    fn test_detect_33_patterns_count() {
        assert_eq!(AI_PATTERNS.len(), 33);
    }

    #[test]
    fn test_all_patterns_have_indicators() {
        for p in AI_PATTERNS {
            assert!(!p.indicators.is_empty(), "pattern {} has no indicators", p.id);
            assert!(p.severity > 0.0 && p.severity <= 1.0, "pattern {} severity out of range", p.id);
        }
    }
}
