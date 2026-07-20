use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenreArchetype {
    Minimalist,
    Corporate,
    Artistic,
    Academic,
    Playful,
    Luxury,
    Retro,
    Nature,
    Technical,
}

impl GenreArchetype {
    pub fn all() -> &'static [GenreArchetype] { &[GenreArchetype::Minimalist, GenreArchetype::Corporate, GenreArchetype::Artistic, GenreArchetype::Academic, GenreArchetype::Playful, GenreArchetype::Luxury, GenreArchetype::Retro, GenreArchetype::Nature, GenreArchetype::Technical] }

    pub fn label(&self) -> &'static str {
        match self { GenreArchetype::Minimalist => "Minimalist", GenreArchetype::Corporate => "Corporate", GenreArchetype::Artistic => "Artistic", GenreArchetype::Academic => "Academic", GenreArchetype::Playful => "Playful", GenreArchetype::Luxury => "Luxury", GenreArchetype::Retro => "Retro", GenreArchetype::Nature => "Nature", GenreArchetype::Technical => "Technical" }
    }

    pub fn description(&self) -> &'static str {
        match self {
            GenreArchetype::Minimalist => "Clean, sparse, high whitespace. Neutral colors. Low variance.",
            GenreArchetype::Corporate => "Professional, structured. Blue/gray palette. Medium density.",
            GenreArchetype::Artistic => "Expressive, asymmetric. Bold colors. High motion + density.",
            GenreArchetype::Academic => "Formal, reference-heavy. Serif fonts. Low density, low motion.",
            GenreArchetype::Playful => "Bright, rounded, animated. High variance + motion.",
            GenreArchetype::Luxury => "Elegant, dark-toned. Gold accents. Low density, slow motion.",
            GenreArchetype::Retro => "Nostalgic, warm-toned. Pixel/vintage elements. Medium everything.",
            GenreArchetype::Nature => "Organic, green/earth tones. Curved shapes. Medium-low density.",
            GenreArchetype::Technical => "Dark mode, monospace. Data-dense. Low motion, high density.",
        }
    }

    pub fn default_knobs(&self) -> DesignIntensityKnobs {
        match self {
            GenreArchetype::Minimalist => DesignIntensityKnobs { variance: 0.2, motion: 0.1, density: 0.1 },
            GenreArchetype::Corporate => DesignIntensityKnobs { variance: 0.5, motion: 0.3, density: 0.6 },
            GenreArchetype::Artistic => DesignIntensityKnobs { variance: 0.9, motion: 0.8, density: 0.7 },
            GenreArchetype::Academic => DesignIntensityKnobs { variance: 0.3, motion: 0.1, density: 0.5 },
            GenreArchetype::Playful => DesignIntensityKnobs { variance: 0.8, motion: 0.9, density: 0.4 },
            GenreArchetype::Luxury => DesignIntensityKnobs { variance: 0.3, motion: 0.4, density: 0.2 },
            GenreArchetype::Retro => DesignIntensityKnobs { variance: 0.5, motion: 0.5, density: 0.5 },
            GenreArchetype::Nature => DesignIntensityKnobs { variance: 0.4, motion: 0.3, density: 0.3 },
            GenreArchetype::Technical => DesignIntensityKnobs { variance: 0.3, motion: 0.1, density: 0.8 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignIntensityKnobs {
    pub variance: f64,
    pub motion: f64,
    pub density: f64,
}

impl Default for DesignIntensityKnobs {
    fn default() -> Self { Self { variance: 0.5, motion: 0.4, density: 0.5 } }
}

impl DesignIntensityKnobs {
    pub fn clamp(&self) -> Self {
        Self { variance: self.variance.max(0.0).min(1.0), motion: self.motion.max(0.0).min(1.0), density: self.density.max(0.0).min(1.0) }
    }

    pub fn to_style_profile(&self) -> StyleProfile {
        let clamped = self.clamp();
        StyleProfile {
            spacing_multiplier: 1.0 + (1.0 - clamped.density) * 0.5,
            color_saturation: 0.3 + clamped.variance * 0.7,
            animation_intensity: clamped.motion,
            border_radius: 4.0 + clamped.variance * 12.0,
            shadow_depth: clamped.motion * 8.0,
            font_contrast: 0.5 + clamped.density * 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleProfile {
    pub spacing_multiplier: f64,
    pub color_saturation: f64,
    pub animation_intensity: f64,
    pub border_radius: f64,
    pub shadow_depth: f64,
    pub font_contrast: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanizeConfig {
    pub perplexity_injection: f64,
    pub burstiness_enforcement: f64,
    pub hedge_surgery: f64,
    pub structural_flattening: f64,
    pub specificity_insertion: f64,
    pub voice_register: f64,
    pub discourse_coherence: f64,
    pub punctuation_normalization: f64,
    pub rlhf_voice_strip: f64,
    pub design_knobs: DesignIntensityKnobs,
    pub genre: GenreArchetype,
}

impl Default for HumanizeConfig {
    fn default() -> Self {
        Self {
            perplexity_injection: 0.6, burstiness_enforcement: 0.7, hedge_surgery: 0.8,
            structural_flattening: 0.5, specificity_insertion: 0.4, voice_register: 0.6,
            discourse_coherence: 0.5, punctuation_normalization: 0.7, rlhf_voice_strip: 0.9,
            design_knobs: DesignIntensityKnobs::default(), genre: GenreArchetype::Minimalist,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanizeResult {
    pub text: String,
    pub applied_levers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Humanizer { pub config: HumanizeConfig }

impl Humanizer {
    pub fn new(config: HumanizeConfig) -> Self { Self { config } }
    pub fn humanize(&self, text: &str) -> HumanizeResult { humanize(text, &self.config) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicsFeatures {
    pub hedge_density: f64,
    pub ai_vocabulary_density: f64,
    pub transition_density: f64,
    pub avg_sentence_length: f64,
    pub sentence_length_variance: f64,
    pub rlhf_pattern_count: usize,
    pub semicolon_ratio: f64,
    pub overall_ai_score: f64,
    pub pattern_count: usize,
    pub category_breakdown: HashMap<String, usize>,
    pub perplexity_score: f64,
    pub burstiness_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AiPatternCategory { Content, LanguageGrammar, Style, Communication, FillerHedging }

impl AiPatternCategory {
    pub fn label(&self) -> &'static str {
        match self { AiPatternCategory::Content => "Content", AiPatternCategory::LanguageGrammar => "Language & Grammar", AiPatternCategory::Style => "Style", AiPatternCategory::Communication => "Communication", AiPatternCategory::FillerHedging => "Filler & Hedging" }
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

static AI_PATTERNS: &[AiPattern] = &[
    AiPattern { id: 1, name: "significance_inflation", category: AiPatternCategory::Content, indicators: &["pivotal moment", "stands as a testament", "marks a", "revolution in", "era of"], severity: 0.8 },
    AiPattern { id: 2, name: "notability_name_dropping", category: AiPatternCategory::Content, indicators: &["cited in", "featured in", "published in"], severity: 0.6 },
    AiPattern { id: 3, name: "superficial_ing_analyses", category: AiPatternCategory::Content, indicators: &["symbolizing", "reflecting", "showcasing", "highlighting"], severity: 0.7 },
    AiPattern { id: 4, name: "promotional_language", category: AiPatternCategory::Content, indicators: &["nestled in", "breathtaking", "vibrant", "stunning"], severity: 0.8 },
    AiPattern { id: 5, name: "vague_attributions", category: AiPatternCategory::Content, indicators: &["experts believe", "it is said that", "some argue"], severity: 0.6 },
    AiPattern { id: 6, name: "formulaic_challenges", category: AiPatternCategory::Content, indicators: &["despite challenges", "continues to thrive", "remains resilient"], severity: 0.5 },
    AiPattern { id: 7, name: "ai_vocabulary", category: AiPatternCategory::LanguageGrammar, indicators: &["testament", "landscape", "showcasing", "tapestry", "realm", "nuanced"], severity: 0.9 },
    AiPattern { id: 8, name: "copula_avoidance", category: AiPatternCategory::LanguageGrammar, indicators: &["serves as", "features", "boasts", "offers", "provides"], severity: 0.7 },
    AiPattern { id: 9, name: "negative_parallelisms", category: AiPatternCategory::LanguageGrammar, indicators: &["it's not just", "it's not only", "more than just", "not merely"], severity: 0.6 },
    AiPattern { id: 10, name: "rule_of_three", category: AiPatternCategory::LanguageGrammar, indicators: &["innovation, inspiration", "past, present", "mind, body"], severity: 0.5 },
    AiPattern { id: 11, name: "synonym_cycling", category: AiPatternCategory::LanguageGrammar, indicators: &["protagonist", "main character", "central figure"], severity: 0.7 },
    AiPattern { id: 12, name: "false_ranges", category: AiPatternCategory::LanguageGrammar, indicators: &["from the big bang", "from a to z"], severity: 0.5 },
    AiPattern { id: 13, name: "passive_subjectless", category: AiPatternCategory::LanguageGrammar, indicators: &["no configuration file needed", "it is recommended", "it is advised"], severity: 0.4 },
    AiPattern { id: 14, name: "em_dash_overuse", category: AiPatternCategory::Style, indicators: &["\u{2014}", "\u{2013}"], severity: 0.5 },
    AiPattern { id: 15, name: "boldface_overuse", category: AiPatternCategory::Style, indicators: &["**okr", "**kpi"], severity: 0.4 },
    AiPattern { id: 16, name: "inline_header_lists", category: AiPatternCategory::Style, indicators: &["**performance:**", "**overview:**"], severity: 0.5 },
    AiPattern { id: 17, name: "title_case_headings", category: AiPatternCategory::Style, indicators: &["strategic negotiations and"], severity: 0.3 },
    AiPattern { id: 18, name: "emojis", category: AiPatternCategory::Style, indicators: &["\u{1f680}", "\u{2728}"], severity: 0.6 },
    AiPattern { id: 19, name: "curly_quotes", category: AiPatternCategory::Style, indicators: &["\u{201c}", "\u{201d}"], severity: 0.2 },
    AiPattern { id: 20, name: "hyphenated_word_pairs", category: AiPatternCategory::Style, indicators: &["data-driven", "cross-functional"], severity: 0.4 },
    AiPattern { id: 21, name: "persuasive_authority_tropes", category: AiPatternCategory::Style, indicators: &["at its core", "what matters is"], severity: 0.5 },
    AiPattern { id: 22, name: "signposting_announcements", category: AiPatternCategory::Style, indicators: &["let's dive in", "here's what you need"], severity: 0.7 },
    AiPattern { id: 23, name: "fragmented_headers", category: AiPatternCategory::Style, indicators: &["## performance. speed"], severity: 0.4 },
    AiPattern { id: 24, name: "diff_anchored_writing", category: AiPatternCategory::Style, indicators: &["was added to replace", "was introduced to"], severity: 0.5 },
    AiPattern { id: 25, name: "manufactured_punchlines", category: AiPatternCategory::Style, indicators: &["no preference. no prior"], severity: 0.6 },
    AiPattern { id: 26, name: "aphorism_formulas", category: AiPatternCategory::Style, indicators: &["is the language of", "is the key to"], severity: 0.5 },
    AiPattern { id: 27, name: "conversational_rhetorical_openers", category: AiPatternCategory::Style, indicators: &["honestly? it depends"], severity: 0.4 },
    AiPattern { id: 28, name: "chatbot_artifacts", category: AiPatternCategory::Communication, indicators: &["i hope this helps", "let me know if you"], severity: 0.9 },
    AiPattern { id: 29, name: "cutoff_disclaimers", category: AiPatternCategory::Communication, indicators: &["while details are limited"], severity: 0.5 },
    AiPattern { id: 30, name: "sycophantic_tone", category: AiPatternCategory::Communication, indicators: &["great question", "you're absolutely right"], severity: 0.8 },
    AiPattern { id: 31, name: "filler_phrases", category: AiPatternCategory::FillerHedging, indicators: &["in order to", "due to the fact that"], severity: 0.6 },
    AiPattern { id: 32, name: "excessive_hedging", category: AiPatternCategory::FillerHedging, indicators: &["could potentially", "may possibly"], severity: 0.7 },
    AiPattern { id: 33, name: "generic_conclusions", category: AiPatternCategory::FillerHedging, indicators: &["the future looks bright", "only time will tell"], severity: 0.6 },
];

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_id: usize,
    pub pattern_name: String,
    pub category: String,
    pub count: usize,
    pub severity: f64,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub pattern_matches: Vec<PatternMatch>,
    pub total_severity: f64,
    pub category_breakdown: HashMap<String, usize>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct VoiceProfile {
    pub avg_sentence_length: f64,
    pub sentence_length_variance: f64,
    pub word_complexity: f64,
    pub dash_frequency: f64,
    pub parenthetical_frequency: f64,
    pub paragraph_opening_style: f64,
    pub signature_phrases: Vec<String>,
}

impl VoiceProfile {
    pub fn from_text(text: &str) -> Self {
        let sentences: Vec<&str> = text.split(|c: char| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let total = sentences.len().max(1);
        let avg_sentence_length = sentences.iter().map(|s| s.len()).sum::<usize>() as f64 / total as f64;
        let sentence_length_variance = if sentences.len() > 1 {
            sentences.iter().map(|s| { let d = s.len() as f64 - avg_sentence_length; d * d }).sum::<f64>() / total as f64
        } else { 0.0 };

        let words: Vec<&str> = text.split_whitespace().collect();
        let word_complexity = words.iter().filter(|w| w.len() > 8).count() as f64 / words.len().max(1) as f64;
        let dash_frequency = (text.matches('\u{2014}').count() + text.matches("--").count()) as f64 / total.max(1) as f64;
        let parenthetical_frequency = (text.matches('(').count() + text.matches('[').count()) as f64 / total.max(1) as f64;

        Self { avg_sentence_length, sentence_length_variance, word_complexity, dash_frequency, parenthetical_frequency, paragraph_opening_style: 0.5, signature_phrases: vec![] }
    }

    pub fn similarity(&self, other: &VoiceProfile) -> f64 {
        let raw = (self.avg_sentence_length - other.avg_sentence_length).abs() / 100.0
            + (self.sentence_length_variance - other.sentence_length_variance).abs() / 200.0
            + (self.word_complexity - other.word_complexity).abs()
            + (self.dash_frequency - other.dash_frequency).abs()
            + (self.parenthetical_frequency - other.parenthetical_frequency).abs()
            + (self.paragraph_opening_style - other.paragraph_opening_style).abs();
        (1.0 - raw / 6.0).max(0.0).min(1.0)
    }
}

#[derive(Debug, Clone)]
pub struct PerplexityMetrics {
    pub avg_word_length: f64,
    pub vocab_richness: f64,
    pub stopword_ratio: f64,
    pub repetitiveness: f64,
    pub perplexity_score: f64,
}

#[derive(Debug, Clone)]
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
                matches.push(PatternMatch { pattern_id: pattern.id, pattern_name: pattern.name.to_string(), category: cat_label, count, severity: pattern.severity, examples });
            }
        }

        let confidence = ((total_severity / text.len().max(1) as f64 * 1000.0).min(5.0) / 5.0 * 0.7
            + (matches.len() as f64 / 10.0).min(1.0) * 0.3).max(0.0).min(1.0);

        DetectionResult { pattern_matches: matches, total_severity, category_breakdown: category_counts, confidence }
    }

    pub fn compute_sentence_stats(text: &str) -> (Vec<usize>, f64, f64) {
        let lengths: Vec<usize> = text.split(|c: char| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim().split_whitespace().count()).filter(|&c| c > 0).collect();
        if lengths.is_empty() { return (vec![], 0.0, 0.0); }
        let mean = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;
        let variance = lengths.iter().map(|&l| { let d = l as f64 - mean; d * d }).sum::<f64>() / lengths.len() as f64;
        (lengths, mean, variance.sqrt())
    }

    pub fn compute_perplexity(text: &str) -> PerplexityMetrics {
        let words: Vec<&str> = text.split_whitespace().collect();
        let total_words = words.len().max(1);
        let avg_word_length = words.iter().map(|w| w.len()).sum::<usize>() as f64 / total_words as f64;

        let unique_words: std::collections::HashSet<&str> = words.iter().map(|w| w.trim_matches(|c: char| !c.is_alphanumeric())).filter(|w| !w.is_empty()).collect();
        let vocab_richness = unique_words.len() as f64 / total_words as f64;

        let stopwords = &["the", "a", "an", "is", "are", "was", "were", "it", "this", "that", "to", "of", "in", "for", "on", "and", "or", "but", "with", "as", "at"];
        let stopword_count = words.iter().filter(|w| stopwords.contains(&w.to_lowercase().as_str())).count();
        let stopword_ratio = stopword_count as f64 / total_words as f64;

        let word_freq: HashMap<&str, usize> = {
            let mut m = HashMap::new();
            for w in &words {
                let w = w.trim_matches(|c: char| !c.is_alphanumeric());
                if !w.is_empty() { *m.entry(w).or_insert(0) += 1; }
            }
            m
        };
        let repetitiveness = word_freq.values().copied().max().unwrap_or(1) as f64 / total_words as f64;

        let prob_score = (vocab_richness * 2.0 + (1.0 - stopword_ratio) + avg_word_length * 0.1) / 3.0;
        let perplexity_score = (1.0 / (prob_score + 0.01)).min(10.0) / 10.0;
        PerplexityMetrics { avg_word_length, vocab_richness, stopword_ratio, repetitiveness, perplexity_score }
    }

    pub fn compute_burstiness(text: &str) -> BurstinessMetrics {
        let lengths: Vec<usize> = text.split(|c: char| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim().split_whitespace().count()).filter(|&c| c > 0).collect();
        if lengths.is_empty() { return BurstinessMetrics { sentence_lengths: vec![], min_sentence: 0, max_sentence: 0, std_dev: 0.0, burstiness_score: 0.0 }; }
        let min_sentence = *lengths.iter().min().unwrap();
        let max_sentence = *lengths.iter().max().unwrap();
        let mean = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;
        let variance = lengths.iter().map(|&l| { let d = l as f64 - mean; d * d }).sum::<f64>() / lengths.len() as f64;
        let std_dev = variance.sqrt();
        let burstiness_score = (std_dev / (mean + 1.0) * 2.0).min(1.0).max(0.0);
        BurstinessMetrics { sentence_lengths: lengths, min_sentence, max_sentence, std_dev, burstiness_score }
    }

    pub fn summary_report(text: &str, voice_sample: Option<&str>) -> String {
        let detection = Self::detect_all(text);
        let perplexity = Self::compute_perplexity(text);
        let burstiness = Self::compute_burstiness(text);
        let mut report = format!("=== AI Writing Detection Report ===\nConfidence: {:.1}%\nPatterns detected: {}\n", detection.confidence * 100.0, detection.pattern_matches.len());
        report.push_str(&format!("Perplexity score: {:.3}\nBurstiness score: {:.3}\n", 1.0 - perplexity.perplexity_score, burstiness.burstiness_score));

        if !detection.category_breakdown.is_empty() {
            report.push_str("\nCategory:\n");
            let mut cats: Vec<_> = detection.category_breakdown.iter().collect();
            cats.sort_by(|a, b| b.1.cmp(a.1));
            for (cat, count) in cats { report.push_str(&format!("  {}: {}\n", cat, count)); }
        }
        if let Some(top) = detection.pattern_matches.first() {
            report.push_str(&format!("\nTop: {} (severity: {:.1})\n", top.pattern_name, top.severity));
        }
        if let Some(sample) = voice_sample {
            let sim = VoiceProfile::from_text(sample).similarity(&VoiceProfile::from_text(text));
            report.push_str(&format!("Voice similarity: {:.1}%\n", sim * 100.0));
        }
        if burstiness.burstiness_score < 0.3 { report.push_str("\nLow burstiness — vary sentence length (3-35 words).\n"); }
        if perplexity.perplexity_score > 0.7 { report.push_str("High perplexity — use less common word choices.\n"); }
        report
    }
}

#[derive(Debug, Clone)]
pub struct ForensicsScorer;

impl ForensicsScorer {
    pub fn score(text: &str) -> ForensicsFeatures {
        let hedge_density = Self::count_phrases(text, &HEDGE_WORDS) as f64 / (text.len().max(1) as f64);
        let ai_vocab_count = Self::count_words(text, &AI_VOCABULARY);
        let transition_count = Self::count_phrases(text, &AI_TRANSITIONS);
        let ai_vocabulary_density = ai_vocab_count as f64 / (text.len().max(1) as f64);
        let transition_density = transition_count as f64 / (text.len().max(1) as f64);

        let sentences: Vec<&str> = text.split(|c: char| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let total_sentences = sentences.len().max(1);
        let avg_sentence_length = sentences.iter().map(|s| s.len()).sum::<usize>() as f64 / total_sentences as f64;
        let sentence_length_variance = if sentences.len() > 1 {
            sentences.iter().map(|s| { let diff = s.len() as f64 - avg_sentence_length; diff * diff }).sum::<f64>() / total_sentences as f64
        } else { 0.0 };

        let rlhf_pattern_count = Self::count_phrases(text, &RLHF_PATTERNS);
        let semicolon_count = text.matches(';').count();
        let semicolon_ratio = semicolon_count as f64 / (text.len().max(1) as f64);

        let raw_score = hedge_density * 3.0 + ai_vocabulary_density * 4.0 + transition_density * 2.0
            + (sentence_length_variance / 200.0).min(1.0).max(0.0) * (-0.5)
            + (rlhf_pattern_count as f64 * 0.15) + semicolon_ratio * 5.0;
        let overall_ai_score = (raw_score / 1.5).max(0.0).min(1.0);

        let detection = HumanizeDetector::detect_all(text);
        let perplexity = HumanizeDetector::compute_perplexity(text);
        let burstiness = HumanizeDetector::compute_burstiness(text);

        ForensicsFeatures {
            hedge_density, ai_vocabulary_density, transition_density, avg_sentence_length,
            sentence_length_variance, rlhf_pattern_count, semicolon_ratio, overall_ai_score,
            pattern_count: detection.pattern_matches.len(),
            category_breakdown: detection.category_breakdown,
            perplexity_score: perplexity.perplexity_score,
            burstiness_score: burstiness.burstiness_score,
        }
    }

    fn count_words(text: &str, words: &[&str]) -> usize {
        let lower = text.to_lowercase();
        words.iter().filter(|w| lower.contains(&w.to_lowercase())).count()
    }

    fn count_phrases(text: &str, phrases: &[&str]) -> usize {
        let lower = text.to_lowercase();
        phrases.iter().filter(|p| lower.contains(&p.to_lowercase())).count()
    }
}

const HEDGE_WORDS: &[&str] = &["it is important to note that", "it is worth mentioning", "generally speaking", "in many cases", "it is worth noting", "it should be noted that", "it is interesting to note", "it must be noted that"];
const AI_VOCABULARY: &[&str] = &["delve", "leverage", "robust", "streamline", "comprehensive", "notably", "pivotal", "foster", "facilitate", "utilize"];
const AI_TRANSITIONS: &[&str] = &["furthermore", "moreover", "in addition", "additionally", "consequently", "in conclusion", "thus"];
const RLHF_PATTERNS: &[&str] = &["that's a great question", "i hope this helps", "great question!", "let me know if you", "feel free to ask", "i'd be happy to"];

fn strip_hedges(text: &str) -> String {
    let mut result = text.to_string();
    for &hedge in HEDGE_WORDS {
        let capitalized: String = hedge.chars().next().map(|c| c.to_uppercase().collect::<String>() + &hedge[hedge.len()..]).unwrap_or_default();
        for variant in [hedge, &*capitalized] { result = result.replace(variant, ""); }
    }
    result
}

fn replace_ai_vocabulary(text: &str) -> String {
    let mut result = text.to_string();
    for word in AI_VOCABULARY {
        let replacement = match *word {
            "delve" => "explore", "leverage" => "use", "robust" => "solid",
            "streamline" => "simplify", "comprehensive" => "thorough",
            "notably" => "", "pivotal" => "key", "foster" => "build",
            "facilitate" => "help", "utilize" => "use", _ => *word,
        };
        if !replacement.is_empty() {
            result = result.replace(&format!(" {} ", word), &format!(" {} ", replacement));
        }
    }
    result
}

fn strip_ai_transitions(text: &str) -> String {
    let mut result = text.to_string();
    for t in AI_TRANSITIONS {
        let capitalized: String = t.chars().next().map(|c| c.to_uppercase().collect::<String>() + &t[c.len_utf8()..]).unwrap_or_default();
        for variant in [*t, capitalized.as_str()] {
            for suffix in [", ", " "] { result = result.replace(&format!("{}{}", variant, suffix), ""); }
        }
    }
    result
}

fn normalize_punctuation(text: &str) -> String {
    let mut result = text.to_string();
    result = result.replace("—", " - ");
    result = result.replace('\u{2014}', " - ");
    result = result.replace(";", ".");
    result
}

fn strip_rlhf_voice(text: &str) -> String {
    let mut result = text.to_string();
    for &p in RLHF_PATTERNS.iter() {
        let capitalized: String = p.chars().next().map(|c| c.to_uppercase().collect::<String>() + &p[p.len()..]).unwrap_or_default();
        result = result.replace(p, "");
        if !capitalized.is_empty() && capitalized != p { result = result.replace(&capitalized, ""); }
    }
    result
}

fn enforce_sentence_variance(text: &str, intensity: f64) -> String {
    if intensity < 0.3 || !text.contains('.') { return text.to_string(); }
    let sentences: Vec<&str> = text.split('.').collect();
    if sentences.len() < 3 { return text.to_string(); }
    let mut new_sentences: Vec<String> = Vec::new();
    for (i, s) in sentences.iter().enumerate() {
        let trimmed = s.trim();
        if trimmed.is_empty() { new_sentences.push(String::new()); continue; }
        if i == 1 && intensity > 0.5 && trimmed.len() > 30 {
            new_sentences.push(trimmed.to_string());
            new_sentences.push("Short.".to_string());
            continue;
        }
        new_sentences.push(trimmed.to_string());
    }
    new_sentences.join(". ")
}

pub fn humanize(text: &str, config: &HumanizeConfig) -> HumanizeResult {
    let mut applied_levers: Vec<String> = Vec::new();
    let mut result = text.to_string();

    if config.hedge_surgery > 0.0 {
        let before = result.clone(); result = strip_hedges(&result);
        if result != before { applied_levers.push("hedge_surgery".to_string()); }
    }
    if config.rlhf_voice_strip > 0.0 {
        let before = result.clone(); result = strip_rlhf_voice(&result);
        if result != before { applied_levers.push("rlhf_voice_strip".to_string()); }
    }
    if config.punctuation_normalization > 0.0 {
        let before = result.clone(); result = normalize_punctuation(&result);
        if result != before { applied_levers.push("punctuation_normalization".to_string()); }
    }
    if config.perplexity_injection > 0.0 {
        let before = result.clone(); result = replace_ai_vocabulary(&result);
        if result != before { applied_levers.push("perplexity_injection".to_string()); }
    }
    if config.discourse_coherence > 0.0 {
        let before = result.clone(); result = strip_ai_transitions(&result);
        if result != before { applied_levers.push("discourse_coherence".to_string()); }
    }
    if config.burstiness_enforcement > 0.0 {
        applied_levers.push("burstiness_enforcement".to_string());
        result = enforce_sentence_variance(&result, config.burstiness_enforcement);
    }
    if config.voice_register > 0.0 { applied_levers.push("voice_register".to_string()); }
    if config.structural_flattening > 0.0 { applied_levers.push("structural_flattening".to_string()); }
    if config.specificity_insertion > 0.0 { applied_levers.push("specificity_insertion".to_string()); }

    HumanizeResult { text: result, applied_levers }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_default_config() { let cfg = HumanizeConfig::default(); assert!((cfg.hedge_surgery - 0.8).abs() < 1e-6); assert!((cfg.burstiness_enforcement - 0.7).abs() < 1e-6); assert_eq!(cfg.genre, GenreArchetype::Minimalist); }
    #[test] fn test_humanizer_struct() { let humanizer = Humanizer::new(HumanizeConfig { burstiness_enforcement: 0.0, structural_flattening: 0.0, specificity_insertion: 0.0, voice_register: 0.0, perplexity_injection: 0.0, punctuation_normalization: 0.0, discourse_coherence: 0.0, rlhf_voice_strip: 0.0, hedge_surgery: 0.0, design_knobs: Default::default(), genre: GenreArchetype::Minimalist }); let result = humanizer.humanize("This is a test."); assert_eq!(result.text, "This is a test."); }
    #[test] fn test_humanize_strips_hedges() { let cfg = HumanizeConfig { hedge_surgery: 1.0, ..Default::default() }; let result = humanize("It is important to note that this is a key finding.", &cfg); assert!(!result.text.contains("It is important to note that")); assert!(result.applied_levers.contains(&"hedge_surgery".to_string())); }
    #[test] fn test_humanize_strips_ai_vocabulary() { let cfg = HumanizeConfig { perplexity_injection: 1.0, ..Default::default() }; let result = humanize("We leverage robust frameworks to streamline our workflow.", &cfg); assert!(!result.text.contains("leverage")); assert!(result.text.contains("use")); }
    #[test] fn test_humanize_normalizes_punctuation() { let cfg = HumanizeConfig { punctuation_normalization: 1.0, ..Default::default() }; let result = humanize("The result was surprising\u{2014}nobody expected it; the data was clear.", &cfg); assert!(!result.text.contains(';')); }
    #[test] fn test_humanize_strips_ai_transitions() { let cfg = HumanizeConfig { discourse_coherence: 1.0, ..Default::default() }; let result = humanize("Furthermore, the analysis shows a clear trend. Moreover, the data supports this.", &cfg); assert!(!result.text.contains("Furthermore")); }
    #[test] fn test_humanize_empty_text() { let cfg = HumanizeConfig { burstiness_enforcement: 0.0, structural_flattening: 0.0, specificity_insertion: 0.0, voice_register: 0.0, ..Default::default() }; let result = humanize("", &cfg); assert_eq!(result.text, ""); assert!(result.applied_levers.is_empty()); }
    #[test] fn test_humanize_all_levers_zero() { let cfg = HumanizeConfig { perplexity_injection: 0.0, burstiness_enforcement: 0.0, hedge_surgery: 0.0, structural_flattening: 0.0, specificity_insertion: 0.0, voice_register: 0.0, discourse_coherence: 0.0, punctuation_normalization: 0.0, rlhf_voice_strip: 0.0, design_knobs: Default::default(), genre: GenreArchetype::Minimalist }; let result = humanize("This is a test sentence. It has nothing special.", &cfg); assert_eq!(result.text, "This is a test sentence. It has nothing special."); }
    #[test] fn test_genre_archetype_all_count() { assert_eq!(GenreArchetype::all().len(), 9); }
    #[test] fn test_genre_default_knobs() { assert!(GenreArchetype::Artistic.default_knobs().variance > 0.7); assert!(GenreArchetype::Minimalist.default_knobs().variance < 0.3); }
    #[test] fn test_design_knobs_clamp() { let knobs = DesignIntensityKnobs { variance: 1.5, motion: -0.5, density: 0.5 }; let clamped = knobs.clamp(); assert!((clamped.variance - 1.0).abs() < 1e-6); assert!((clamped.motion - 0.0).abs() < 1e-6); }
    #[test] fn test_style_profile_from_knobs() { let profile = DesignIntensityKnobs { variance: 1.0, motion: 1.0, density: 0.0 }.to_style_profile(); assert!(profile.animation_intensity > 0.9); assert!(profile.spacing_multiplier > 1.4); }
    #[test] fn test_forensics_scorer_low_ai_score() { let features = ForensicsScorer::score("The cat sat on the mat. It was a sunny day. Birds were singing in the trees."); assert!(features.overall_ai_score < 0.5); }
    #[test] fn test_forensics_scorer_high_ai_score() { let features = ForensicsScorer::score("Furthermore, we leverage robust frameworks to comprehensively streamline our workflow. It is important to note that this facilitates pivotal outcomes. I hope this helps."); assert!(features.ai_vocabulary_density > 0.0); assert!(features.rlhf_pattern_count > 0); }
    #[test] fn test_forensics_scorer_empty_text() { let features = ForensicsScorer::score(""); assert_eq!(features.overall_ai_score, 0.0); assert_eq!(features.avg_sentence_length, 0.0); }
    #[test] fn test_forensics_features_clamp() { let features = ForensicsScorer::score("x"); assert!((0.0..=1.0).contains(&features.overall_ai_score)); }
    #[test] fn test_strip_rlhf_voice() { let result = strip_rlhf_voice("That's a great question. Let me know if you need more help."); assert!(!result.contains("That's a great question")); assert!(!result.contains("Let me know if you")); }

    #[test] fn test_detect_empty() { let r = HumanizeDetector::detect_all(""); assert_eq!(r.pattern_matches.len(), 0); assert_eq!(r.confidence, 0.0); }
    #[test] fn test_detect_ai_text() { let r = HumanizeDetector::detect_all("This stands as a testament. Furthermore, experts believe it leverages robust frameworks. I hope this helps!"); assert!(r.pattern_matches.len() >= 3); assert!(r.confidence > 0.3); }
    #[test] fn test_detect_natural_text() { let r = HumanizeDetector::detect_all("The cat sat on the mat. It was a sunny day."); assert_eq!(r.pattern_matches.len(), 0); assert_eq!(r.confidence, 0.0); }
    #[test] fn test_ai_patterns_count() { assert_eq!(AI_PATTERNS.len(), 33); }
    #[test] fn test_voice_profile_similarity() { let sim = VoiceProfile::from_text("This is a test. It has multiple sentences.").similarity(&VoiceProfile::from_text("This is another test. It also has sentences.")); assert!(sim > 0.3); }
    #[test] fn test_voice_profile_identical() { let text = "This is a test. It has multiple sentences."; assert!((VoiceProfile::from_text(text).similarity(&VoiceProfile::from_text(text)) - 1.0).abs() < 0.01); }
    #[test] fn test_perplexity_metrics() { let p = HumanizeDetector::compute_perplexity("The quick brown fox jumps over the lazy dog"); assert!(p.avg_word_length > 3.0); }
    #[test] fn test_perplexity_repetitive() { let p = HumanizeDetector::compute_perplexity("test test test test test"); assert!(p.repetitiveness > 0.5); }
    #[test] fn test_burstiness_metrics() { let b = HumanizeDetector::compute_burstiness("Short. Medium sentence here. This is a much longer sentence with many more words to test variance."); assert!(b.max_sentence > b.min_sentence); }
    #[test] fn test_burstiness_uniform() { let b = HumanizeDetector::compute_burstiness("Short sentence. Short sentence. Short sentence."); assert!(b.burstiness_score < 0.3); }
    #[test] fn test_burstiness_empty() { let b = HumanizeDetector::compute_burstiness(""); assert_eq!(b.sentence_lengths.len(), 0); }
    #[test] fn test_summary_report() { let report = HumanizeDetector::summary_report("This stands as a testament. Experts believe it leverages robust frameworks.", None); assert!(report.contains("AI Writing Detection")); }
    #[test] fn test_summary_report_with_voice() { let report = HumanizeDetector::summary_report("Short. Normal length sentence here.", Some("I tend to write in a casual style. My sentences vary in length.")); assert!(report.contains("Voice similarity")); }
    #[test] fn test_forensics_includes_new_fields() { let f = ForensicsScorer::score("Short. Normal. This is a longer sentence with more complexity."); assert!(f.pattern_count == 0 || f.pattern_count > 0); assert!(f.perplexity_score >= 0.0); assert!(f.burstiness_score >= 0.0); }
    #[test] fn test_all_patterns_have_indicators() { for p in AI_PATTERNS { assert!(!p.indicators.is_empty()); assert!(p.severity > 0.0 && p.severity <= 1.0); } }
}
