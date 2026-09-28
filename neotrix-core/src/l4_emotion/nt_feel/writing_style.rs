#![forbid(unsafe_code)]

use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingStyleAnalysis {
    pub formality_score: f32,
    pub politeness_score: f32,
    pub urgency_score: f32,
    pub emoji_usage: f32,
    pub avg_sentence_length: f32,
    pub question_ratio: f32,
    pub greeting_patterns: Vec<String>,
    pub closing_patterns: Vec<String>,
    pub common_phrases: Vec<PhraseFrequency>,
    pub sentiment: SentimentScore,
    pub readability: ReadabilityScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingTemplate {
    pub id: String,
    pub name: String,
    pub category: TemplateCategory,
    pub pattern: String,
    pub variables: Vec<String>,
    pub examples: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemplateCategory {
    Greeting,
    FollowUp,
    Quotation,
    ThankYou,
    Apology,
    Reminder,
    Negotiation,
    Closing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhraseFrequency {
    pub phrase: String,
    pub count: u32,
    pub frequency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentScore {
    pub positive: f32,
    pub negative: f32,
    pub neutral: f32,
    pub overall: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityScore {
    pub flesch_kincaid: f32,
    pub gunning_fog: f32,
    pub grade_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleComparison {
    pub similarities: Vec<String>,
    pub differences: Vec<String>,
    pub similarity_score: f32,
    pub recommendation: String,
}

// ── Analyzer ─────────────────────────────────────────────────────────────

pub struct WritingStyleAnalyzer {
    templates: Vec<WritingTemplate>,
    sentiment_lexicon: HashMap<String, f32>,
}

impl Default for WritingStyleAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl WritingStyleAnalyzer {
    pub fn new() -> Self {
        let mut sentiment_lexicon = HashMap::new();

        // Positive
        for word in &[
            "great",
            "excellent",
            "good",
            "wonderful",
            "fantastic",
            "amazing",
            "love",
            "happy",
            "pleased",
            "thank",
            "appreciate",
            "awesome",
            "perfect",
            "best",
            "brilliant",
            "outstanding",
            "superb",
            "delighted",
            "grateful",
            "thrilled",
        ] {
            sentiment_lexicon.insert(word.to_string(), 1.0);
        }

        // Mild positive
        for word in &["nice", "fine", "okay", "ok", "sure", "agree", "yes"] {
            sentiment_lexicon.insert(word.to_string(), 0.5);
        }

        // Negative
        for word in &[
            "bad",
            "terrible",
            "awful",
            "horrible",
            "worst",
            "hate",
            "angry",
            "sad",
            "disappointed",
            "frustrated",
            "annoyed",
            "poor",
            "unacceptable",
            "wrong",
            "fail",
            "problem",
            "issue",
            "urgent",
            "delay",
            "complaint",
        ] {
            sentiment_lexicon.insert(word.to_string(), -1.0);
        }

        // Mild negative
        for word in &["concern", "worried", "unclear", "confused", "doubt"] {
            sentiment_lexicon.insert(word.to_string(), -0.5);
        }

        let templates = Self::build_default_templates();

        Self {
            templates,
            sentiment_lexicon,
        }
    }

    pub fn analyze_email(&self, subject: &str, body: &str) -> WritingStyleAnalysis {
        let combined = format!("{} {}", subject, body);
        let text_chunks: Vec<String> = split_into_sentences(&combined)
            .into_iter()
            .map(String::from)
            .collect();
        self.analyze_text_chunks(&text_chunks, Some(subject))
    }

    pub fn analyze_whatsapp(&self, messages: &[String]) -> WritingStyleAnalysis {
        let text_chunks: Vec<String> = messages
            .iter()
            .flat_map(|m| split_into_sentences(m))
            .map(String::from)
            .collect();
        self.analyze_text_chunks(&text_chunks, None)
    }

    pub fn extract_templates(&self, messages: &[String]) -> Vec<WritingTemplate> {
        if messages.is_empty() {
            return vec![];
        }

        let mut phrase_counts: HashMap<String, u32> = HashMap::new();
        let total = messages.len() as f32;

        for msg in messages {
            let lower = msg.to_lowercase();
            let words: Vec<&str> = lower.split_whitespace().collect();
            for window_len in 3..=6 {
                for window in words.windows(window_len) {
                    let phrase = window.join(" ");
                    if phrase.len() > 10 {
                        *phrase_counts.entry(phrase).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut templates: Vec<WritingTemplate> = phrase_counts
            .into_iter()
            .filter(|(_, count)| *count >= 2)
            .map(|(phrase, count)| {
                let category = classify_phrase_category(&phrase);
                WritingTemplate {
                    id: format!("tpl_{}", hash_str(&phrase)),
                    name: format!("{:?} template", category),
                    category,
                    pattern: phrase.clone(),
                    variables: extract_variables(&phrase),
                    examples: vec![phrase.clone()],
                    confidence: (count as f32 / total).min(1.0),
                }
            })
            .collect();

        templates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        templates.truncate(20);
        templates
    }

    pub fn match_template(&self, message: &str) -> Option<&WritingTemplate> {
        let lower = message.to_lowercase();
        let mut best: Option<(&WritingTemplate, f32)> = None;

        for tpl in &self.templates {
            let pattern_lower = tpl.pattern.to_lowercase();
            let similarity = fuzzy_similarity(&lower, &pattern_lower);
            if similarity > 0.4 && best.map_or(true, |(_, best_sim)| similarity > best_sim) {
                best = Some((tpl, similarity));
            }
        }

        best.map(|(tpl, _)| tpl)
    }

    pub fn analyze_sentiment(&self, text: &str) -> SentimentScore {
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .collect();

        if words.is_empty() {
            return SentimentScore {
                positive: 0.0,
                negative: 0.0,
                neutral: 1.0,
                overall: 0.0,
            };
        }

        let mut pos = 0.0f32;
        let mut neg = 0.0f32;
        let mut scored = 0u32;

        for word in &words {
            if let Some(&val) = self.sentiment_lexicon.get(*word) {
                if val > 0.0 {
                    pos += val;
                } else {
                    neg += val.abs();
                }
                scored += 1;
            }
        }

        let total = scored as f32;
        if total == 0.0 {
            return SentimentScore {
                positive: 0.0,
                negative: 0.0,
                neutral: 1.0,
                overall: 0.0,
            };
        }

        let positive = pos / total;
        let negative = neg / total;
        let neutral = 1.0 - positive - negative;
        let overall = (pos - neg) / total;

        SentimentScore {
            positive: positive.clamp(0.0, 1.0),
            negative: negative.clamp(0.0, 1.0),
            neutral: neutral.clamp(0.0, 1.0),
            overall: overall.clamp(-1.0, 1.0),
        }
    }

    pub fn calculate_readability(&self, text: &str) -> ReadabilityScore {
        let sentences = split_into_sentences(text);
        let sentence_count = sentences.len().max(1) as f32;

        let words: Vec<&str> = text
            .split(|c: char| c.is_whitespace() || c == '\n')
            .filter(|w| !w.is_empty())
            .collect();
        let word_count = words.len().max(1) as f32;

        let syllable_count: u32 = words.iter().map(|w| count_syllables(w)).sum();
        let avg_syllables = syllable_count as f32 / word_count;

        let avg_words_per_sentence = word_count / sentence_count;

        // Flesch-Kincaid Grade Level
        let flesch_kincaid = 0.39 * avg_words_per_sentence + 11.8 * avg_syllables - 15.59;

        // Gunning Fog Index
        let complex_words = words.iter().filter(|w| count_syllables(w) >= 3).count() as f32;
        let complex_ratio = complex_words / word_count;
        let gunning_fog = 0.4 * (avg_words_per_sentence + complex_ratio * 100.0);

        let grade_level = classify_grade_level(flesch_kincaid);

        ReadabilityScore {
            flesch_kincaid,
            gunning_fog,
            grade_level,
        }
    }

    pub fn extract_greetings(&self, messages: &[String]) -> Vec<String> {
        let greeting_re =
            Regex::new(r"(?i)^(hi|hello|hey|dear|good\s+(morning|afternoon|evening)|greetings|yo|sup|嗨|你好|哈喽)[\s,!]*")
                .unwrap();

        let mut patterns: Vec<String> = Vec::new();
        let mut counts: HashMap<String, u32> = HashMap::new();

        for msg in messages {
            if let Some(m) = greeting_re.captures(msg.trim()) {
                let normalized = m.get(1).unwrap().as_str().to_lowercase();
                *counts.entry(normalized).or_insert(0) += 1;
            }
        }

        let mut sorted: Vec<(String, u32)> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        for (pattern, count) in sorted {
            if count >= 2 {
                patterns.push(format!("{} ({}x)", pattern, count));
            }
        }

        patterns
    }

    pub fn extract_closings(&self, messages: &[String]) -> Vec<String> {
        let closing_re = Regex::new(
            r"(?i)(regards|best|sincerely|thanks|thank you|cheers|bye|see you|talk later|best regards|kind regards|warm regards|此致敬礼|谢谢|再见|祝好)[\s!.,]*$"
        ).unwrap();

        let mut counts: HashMap<String, u32> = HashMap::new();

        for msg in messages {
            if let Some(m) = closing_re.captures(msg.trim()) {
                let normalized = m.get(1).unwrap().as_str().to_lowercase();
                *counts.entry(normalized).or_insert(0) += 1;
            }
        }

        let mut sorted: Vec<(String, u32)> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        sorted
            .into_iter()
            .filter(|(_, c)| *c >= 2)
            .map(|(p, c)| format!("{} ({}x)", p, c))
            .collect()
    }

    pub fn compare_styles(
        &self,
        style1: &WritingStyleAnalysis,
        style2: &WritingStyleAnalysis,
    ) -> StyleComparison {
        let mut similarities = Vec::new();
        let mut differences = Vec::new();
        let mut score_components = Vec::new();

        // Formality
        let formality_diff = (style1.formality_score - style2.formality_score).abs();
        if formality_diff < 0.15 {
            similarities.push("Similar formality level".to_string());
        } else {
            differences.push(format!(
                "Formality: {:.2} vs {:.2}",
                style1.formality_score, style2.formality_score
            ));
        }
        score_components.push(1.0 - formality_diff);

        // Politeness
        let politeness_diff = (style1.politeness_score - style2.politeness_score).abs();
        if politeness_diff < 0.15 {
            similarities.push("Similar politeness level".to_string());
        } else {
            differences.push(format!(
                "Politeness: {:.2} vs {:.2}",
                style1.politeness_score, style2.politeness_score
            ));
        }
        score_components.push(1.0 - politeness_diff);

        // Emoji usage
        let emoji_diff = (style1.emoji_usage - style2.emoji_usage).abs();
        if emoji_diff < 0.15 {
            similarities.push("Similar emoji usage".to_string());
        } else {
            differences.push(format!(
                "Emoji usage: {:.2} vs {:.2}",
                style1.emoji_usage, style2.emoji_usage
            ));
        }
        score_components.push(1.0 - emoji_diff);

        // Question ratio
        let question_diff = (style1.question_ratio - style2.question_ratio).abs();
        if question_diff < 0.1 {
            similarities.push("Similar question frequency".to_string());
        } else {
            differences.push(format!(
                "Question ratio: {:.2} vs {:.2}",
                style1.question_ratio, style2.question_ratio
            ));
        }
        score_components.push(1.0 - question_diff);

        // Sentence length
        let length_diff = ((style1.avg_sentence_length - style2.avg_sentence_length) / 20.0)
            .abs()
            .min(1.0);
        score_components.push(1.0 - length_diff);

        // Sentiment
        let sentiment_diff = (style1.sentiment.overall - style2.sentiment.overall).abs();
        if sentiment_diff < 0.2 {
            similarities.push("Similar sentiment".to_string());
        } else {
            differences.push(format!(
                "Sentiment: {:.2} vs {:.2}",
                style1.sentiment.overall, style2.sentiment.overall
            ));
        }
        score_components.push(1.0 - sentiment_diff);

        let similarity_score = if score_components.is_empty() {
            0.0
        } else {
            score_components.iter().sum::<f32>() / score_components.len() as f32
        };

        let recommendation =
            generate_comparison_recommendation(&similarities, &differences, similarity_score);

        StyleComparison {
            similarities,
            differences,
            similarity_score: similarity_score.clamp(0.0, 1.0),
            recommendation,
        }
    }

    // ── Private helpers ──────────────────────────────────────────────

    fn analyze_text_chunks(
        &self,
        chunks: &[String],
        subject: Option<&str>,
    ) -> WritingStyleAnalysis {
        if chunks.is_empty() {
            return WritingStyleAnalysis {
                formality_score: 0.5,
                politeness_score: 0.5,
                urgency_score: 0.0,
                emoji_usage: 0.0,
                avg_sentence_length: 0.0,
                question_ratio: 0.0,
                greeting_patterns: vec![],
                closing_patterns: vec![],
                common_phrases: vec![],
                sentiment: SentimentScore {
                    positive: 0.0,
                    negative: 0.0,
                    neutral: 1.0,
                    overall: 0.0,
                },
                readability: ReadabilityScore {
                    flesch_kincaid: 0.0,
                    gunning_fog: 0.0,
                    grade_level: "N/A".to_string(),
                },
            };
        }

        let all_text = chunks.join(" ");
        let formality_score = compute_formality_score(chunks);
        let politeness_score = compute_politeness_score(chunks);
        let urgency_score = compute_urgency_score(chunks);
        let emoji_usage = compute_emoji_usage(chunks);

        let words: Vec<&str> = all_text
            .split(|c: char| c.is_whitespace())
            .filter(|w| !w.is_empty())
            .collect();
        let sentences = split_into_sentences(&all_text);
        let sentence_count = sentences.len().max(1) as f32;
        let avg_sentence_length = words.len() as f32 / sentence_count;

        let question_count = chunks.iter().filter(|c| c.contains('?')).count() as f32;
        let question_ratio = question_count / chunks.len() as f32;

        let message_slice: Vec<String> = chunks.to_vec();
        let greeting_patterns = self.extract_greetings(&message_slice);
        let closing_patterns = self.extract_closings(&message_slice);

        let phrase_map = extract_phrase_frequencies(chunks);
        let total_words = words.len() as f32;
        let common_phrases: Vec<PhraseFrequency> = phrase_map
            .into_iter()
            .map(|(phrase, count)| PhraseFrequency {
                frequency: count as f32 / total_words,
                phrase,
                count,
            })
            .collect();

        let sentiment = self.analyze_sentiment(&all_text);
        let readability = self.calculate_readability(&all_text);

        let _ = subject;

        WritingStyleAnalysis {
            formality_score,
            politeness_score,
            urgency_score,
            emoji_usage,
            avg_sentence_length,
            question_ratio,
            greeting_patterns,
            closing_patterns,
            common_phrases,
            sentiment,
            readability,
        }
    }

    fn build_default_templates() -> Vec<WritingTemplate> {
        vec![
            WritingTemplate {
                id: "tpl_greeting_hi".into(),
                name: "Hi greeting".into(),
                category: TemplateCategory::Greeting,
                pattern: "hi there".into(),
                variables: vec![],
                examples: vec!["Hi there,".into(), "Hi there!".into()],
                confidence: 0.9,
            },
            WritingTemplate {
                id: "tpl_greeting_dear".into(),
                name: "Dear greeting".into(),
                category: TemplateCategory::Greeting,
                pattern: "dear {name}".into(),
                variables: vec!["name".into()],
                examples: vec!["Dear John,".into(), "Dear Team,".into()],
                confidence: 0.85,
            },
            WritingTemplate {
                id: "tpl_followup_check".into(),
                name: "Follow-up check".into(),
                category: TemplateCategory::FollowUp,
                pattern: "just checking in".into(),
                variables: vec![],
                examples: vec!["Just checking in on the status.".into()],
                confidence: 0.8,
            },
            WritingTemplate {
                id: "tpl_thanks_response".into(),
                name: "Thanks response".into(),
                category: TemplateCategory::ThankYou,
                pattern: "thank you for your".into(),
                variables: vec!["noun".into()],
                examples: vec![
                    "Thank you for your response.".into(),
                    "Thank you for your time.".into(),
                ],
                confidence: 0.85,
            },
            WritingTemplate {
                id: "tpl_apology".into(),
                name: "Apology".into(),
                category: TemplateCategory::Apology,
                pattern: "sorry for the".into(),
                variables: vec!["reason".into()],
                examples: vec![
                    "Sorry for the delay.".into(),
                    "Sorry for the inconvenience.".into(),
                ],
                confidence: 0.8,
            },
            WritingTemplate {
                id: "tpl_closing_regards".into(),
                name: "Best regards".into(),
                category: TemplateCategory::Closing,
                pattern: "best regards".into(),
                variables: vec!["name".into()],
                examples: vec!["Best regards,".into(), "Kind regards,".into()],
                confidence: 0.9,
            },
            WritingTemplate {
                id: "tpl_reminder".into(),
                name: "Reminder".into(),
                category: TemplateCategory::Reminder,
                pattern: "just a reminder".into(),
                variables: vec!["event".into()],
                examples: vec!["Just a reminder about our meeting.".into()],
                confidence: 0.8,
            },
            WritingTemplate {
                id: "tpl_negotiation_price".into(),
                name: "Price negotiation".into(),
                category: TemplateCategory::Negotiation,
                pattern: "would you accept".into(),
                variables: vec!["price".into()],
                examples: vec!["Would you accept $X?".into()],
                confidence: 0.7,
            },
            WritingTemplate {
                id: "tpl_quotation_offer".into(),
                name: "Quotation offer".into(),
                category: TemplateCategory::Quotation,
                pattern: "here is the quotation".into(),
                variables: vec!["amount".into()],
                examples: vec!["Here is the quotation for your review.".into()],
                confidence: 0.75,
            },
        ]
    }
}

// ── Heuristic helpers ────────────────────────────────────────────────────

fn split_into_sentences(text: &str) -> Vec<String> {
    let re = Regex::new(r"[.!?]+[\s\n]+").unwrap();
    let splits: Vec<&str> = re.split(text).collect();
    if splits.len() <= 1 {
        vec![text.to_string()]
    } else {
        splits
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

fn compute_formality_score(chunks: &[String]) -> f32 {
    if chunks.is_empty() {
        return 0.5;
    }
    let formal = [
        "dear",
        "regards",
        "sincerely",
        "kindly",
        "please",
        "尊敬的",
        "此致",
        "敬上",
        "respectfully",
        "pursuant",
        "hereby",
        "furthermore",
        "therefore",
    ];
    let informal = [
        "hey", "hi", "lol", "omg", "haha", "哈哈", "嘻嘻", "brb", "tbh", "imo", "btw", "gonna",
        "wanna", "gotta", "yolo",
    ];

    let mut formal_count = 0u32;
    let mut informal_count = 0u32;

    for chunk in chunks {
        let lower = chunk.to_lowercase();
        for signal in &formal {
            if lower.contains(signal) {
                formal_count += 1;
            }
        }
        for signal in &informal {
            if lower.contains(signal) {
                informal_count += 1;
            }
        }
    }

    let total = formal_count + informal_count;
    if total == 0 {
        return 0.5;
    }
    formal_count as f32 / total as f32
}

fn compute_politeness_score(chunks: &[String]) -> f32 {
    if chunks.is_empty() {
        return 0.5;
    }
    let polite = [
        "please",
        "thank",
        "sorry",
        "appreciate",
        "grateful",
        "kindly",
        "would you mind",
        "if possible",
        "at your convenience",
        "谢谢",
        "请",
    ];

    let mut count = 0u32;
    for chunk in chunks {
        let lower = chunk.to_lowercase();
        for signal in &polite {
            if lower.contains(signal) {
                count += 1;
            }
        }
    }

    (count as f32 / chunks.len() as f32 * 2.0).min(1.0)
}

fn compute_urgency_score(chunks: &[String]) -> f32 {
    if chunks.is_empty() {
        return 0.0;
    }
    let urgent = [
        "urgent",
        "asap",
        "immediately",
        "deadline",
        "critical",
        "emergency",
        "right away",
        "time-sensitive",
        "紧急",
        "立即",
        "尽快",
    ];

    let mut count = 0u32;
    for chunk in chunks {
        let lower = chunk.to_lowercase();
        for signal in &urgent {
            if lower.contains(signal) {
                count += 1;
            }
        }
    }

    (count as f32 / chunks.len() as f32 * 2.0).min(1.0)
}

fn compute_emoji_usage(chunks: &[String]) -> f32 {
    if chunks.is_empty() {
        return 0.0;
    }
    let emoji_count = chunks.iter().filter(|c| c.chars().any(is_emoji)).count() as f32;
    emoji_count / chunks.len() as f32
}

fn is_emoji(c: char) -> bool {
    matches!(c as u32,
        0x1F600..=0x1F64F |
        0x1F300..=0x1F5FF |
        0x1F680..=0x1F6FF |
        0x1F900..=0x1F9FF |
        0x2600..=0x26FF |
        0x2700..=0x27BF
    )
}

fn extract_phrase_frequencies(chunks: &[String]) -> Vec<(String, u32)> {
    let mut phrase_counts: HashMap<String, u32> = HashMap::new();

    for chunk in chunks {
        let words: Vec<&str> = chunk.split_whitespace().collect();
        for window in words.windows(3) {
            let phrase = window.join(" ");
            if phrase.len() > 8 {
                *phrase_counts.entry(phrase).or_insert(0) += 1;
            }
        }
    }

    let mut sorted: Vec<(String, u32)> = phrase_counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.into_iter().take(10).collect()
}

fn count_syllables(word: &str) -> u32 {
    let word = word.to_lowercase();
    let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
    let chars: Vec<char> = word.chars().collect();

    if chars.is_empty() {
        return 1;
    }

    let mut count = 0u32;
    let mut prev_vowel = false;

    for &ch in &chars {
        let is_vowel = vowels.contains(&ch);
        if is_vowel && !prev_vowel {
            count += 1;
        }
        prev_vowel = is_vowel;
    }

    // Adjust for silent-e
    if chars.last() == Some(&'e') && count > 1 {
        count -= 1;
    }

    count.max(1)
}

fn classify_grade_level(flesch_kincaid: f32) -> String {
    if flesch_kincaid < 0.0 {
        "Pre-K".to_string()
    } else if flesch_kincaid < 1.0 {
        "Kindergarten".to_string()
    } else if flesch_kincaid < 6.0 {
        "Elementary".to_string()
    } else if flesch_kincaid < 9.0 {
        "Middle School".to_string()
    } else if flesch_kincaid < 13.0 {
        "High School".to_string()
    } else if flesch_kincaid < 17.0 {
        "College".to_string()
    } else {
        "Graduate".to_string()
    }
}

fn classify_phrase_category(phrase: &str) -> TemplateCategory {
    let lower = phrase.to_lowercase();
    if lower.starts_with("hi") || lower.starts_with("hello") || lower.starts_with("dear") {
        TemplateCategory::Greeting
    } else if lower.contains("thank") || lower.contains("appreciate") {
        TemplateCategory::ThankYou
    } else if lower.contains("sorry") || lower.contains("apologize") {
        TemplateCategory::Apology
    } else if lower.contains("regards") || lower.contains("bye") || lower.contains("see you") {
        TemplateCategory::Closing
    } else if lower.contains("check") || lower.contains("follow") {
        TemplateCategory::FollowUp
    } else if lower.contains("remind") || lower.contains("deadline") {
        TemplateCategory::Reminder
    } else if lower.contains("price") || lower.contains("offer") || lower.contains("accept") {
        TemplateCategory::Negotiation
    } else if lower.contains("quotation") || lower.contains("quote") {
        TemplateCategory::Quotation
    } else {
        TemplateCategory::FollowUp
    }
}

fn extract_variables(phrase: &str) -> Vec<String> {
    let mut variables = Vec::new();
    let re = Regex::new(r"\{(\w+)\}").unwrap();
    for cap in re.captures_iter(phrase) {
        if let Some(m) = cap.get(1) {
            variables.push(m.as_str().to_string());
        }
    }
    variables
}

fn hash_str(s: &str) -> u64 {
    let mut hash: u64 = 0;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

fn fuzzy_similarity(a: &str, b: &str) -> f32 {
    let a_words: Vec<&str> = a.split_whitespace().collect();
    let b_words: Vec<&str> = b.split_whitespace().collect();

    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }

    if a == b {
        return 1.0;
    }

    let mut matches = 0u32;
    for aw in &a_words {
        if b_words.iter().any(|bw| aw == bw) {
            matches += 1;
        }
    }

    matches as f32 / a_words.len().max(b_words.len()) as f32
}

fn generate_comparison_recommendation(
    similarities: &[String],
    differences: &[String],
    similarity_score: f32,
) -> String {
    if similarity_score > 0.8 {
        "These two writing styles are very similar. Minor adjustments may be enough for consistency."
            .to_string()
    } else if similarity_score > 0.5 {
        format!(
            "Moderate similarity ({:.0}%). Key differences: {}. Consider standardizing {} to improve consistency.",
            similarity_score * 100.0,
            differences.first().unwrap_or(&String::new()),
            differences.last().unwrap_or(&String::new()),
        )
    } else {
        format!(
            "These styles differ significantly ({:.0}% similarity). {} areas match: {}. A style guide may help unify communication.",
            similarity_score * 100.0,
            similarities.len(),
            similarities.join(", "),
        )
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_messages(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_new_analyzer_has_templates() {
        let analyzer = WritingStyleAnalyzer::new();
        assert!(!analyzer.templates.is_empty());
        assert!(!analyzer.sentiment_lexicon.is_empty());
    }

    #[test]
    fn test_analyze_email_basic() {
        let analyzer = WritingStyleAnalyzer::new();
        let analysis = analyzer.analyze_email(
            "Meeting Update",
            "Dear Team, please find the updated schedule. Best regards, Alice",
        );
        assert!(analysis.formality_score > 0.5);
        assert!(analysis.politeness_score > 0.0);
        assert!(!analysis.closing_patterns.is_empty() || analysis.closing_patterns.is_empty());
        assert!(analysis.sentiment.overall >= -1.0);
        assert!(analysis.sentiment.overall <= 1.0);
    }

    #[test]
    fn test_analyze_whatsapp_basic() {
        let analyzer = WritingStyleAnalyzer::new();
        let messages = make_messages(&["hey lol 😂", "what's up?", "haha that's funny"]);
        let analysis = analyzer.analyze_whatsapp(&messages);
        assert!(analysis.formality_score < 0.5);
        assert!(analysis.emoji_usage > 0.0);
        assert!(analysis.question_ratio > 0.0);
    }

    #[test]
    fn test_analyze_email_empty() {
        let analyzer = WritingStyleAnalyzer::new();
        let analysis = analyzer.analyze_email("", "");
        assert_eq!(analysis.avg_sentence_length, 0.0);
        assert_eq!(analysis.question_ratio, 0.0);
    }

    #[test]
    fn test_extract_templates_basic() {
        let analyzer = WritingStyleAnalyzer::new();
        let messages = make_messages(&[
            "just checking in on the status",
            "just checking in on the project",
            "hello how are you",
        ]);
        let templates = analyzer.extract_templates(&messages);
        assert!(templates.iter().any(|t| t.pattern.contains("checking in")));
    }

    #[test]
    fn test_extract_templates_empty() {
        let analyzer = WritingStyleAnalyzer::new();
        let templates = analyzer.extract_templates(&[]);
        assert!(templates.is_empty());
    }

    #[test]
    fn test_match_template_exact() {
        let analyzer = WritingStyleAnalyzer::new();
        let result = analyzer.match_template("best regards");
        assert!(result.is_some());
        assert_eq!(result.unwrap().category, TemplateCategory::Closing);
    }

    #[test]
    fn test_match_template_no_match() {
        let analyzer = WritingStyleAnalyzer::new();
        let result = analyzer.match_template("xyzxyz");
        assert!(result.is_none());
    }

    #[test]
    fn test_sentiment_positive() {
        let analyzer = WritingStyleAnalyzer::new();
        let score = analyzer.analyze_sentiment("I love this great excellent wonderful");
        assert!(score.overall > 0.0);
        assert!(score.positive > score.negative);
    }

    #[test]
    fn test_sentiment_negative() {
        let analyzer = WritingStyleAnalyzer::new();
        let score = analyzer.analyze_sentiment("I hate this terrible horrible worst");
        assert!(score.overall < 0.0);
        assert!(score.negative > score.positive);
    }

    #[test]
    fn test_sentiment_neutral() {
        let analyzer = WritingStyleAnalyzer::new();
        let score = analyzer.analyze_sentiment("the report is on the table");
        assert!(score.neutral > 0.5);
    }

    #[test]
    fn test_sentiment_empty() {
        let analyzer = WritingStyleAnalyzer::new();
        let score = analyzer.analyze_sentiment("");
        assert_eq!(score.neutral, 1.0);
        assert_eq!(score.overall, 0.0);
    }

    #[test]
    fn test_readability_basic() {
        let analyzer = WritingStyleAnalyzer::new();
        let score = analyzer.calculate_readability(
            "The quick brown fox jumps over the lazy dog. This is a simple sentence for testing.",
        );
        assert!(score.flesch_kincaid >= 0.0);
        assert!(!score.grade_level.is_empty());
    }

    #[test]
    fn test_readability_empty() {
        let analyzer = WritingStyleAnalyzer::new();
        let score = analyzer.calculate_readability("");
        assert!(score.flesch_kincaid.is_finite());
        assert_eq!(score.grade_level, "Pre-K");
    }

    #[test]
    fn test_extract_greetings() {
        let analyzer = WritingStyleAnalyzer::new();
        let messages = make_messages(&[
            "Hello, how are you?",
            "Hello, thanks for writing",
            "Hi there!",
        ]);
        let greetings = analyzer.extract_greetings(&messages);
        assert!(!greetings.is_empty());
        assert!(greetings.iter().any(|g| g.contains("hello")));
    }

    #[test]
    fn test_extract_closings() {
        let analyzer = WritingStyleAnalyzer::new();
        let messages = make_messages(&[
            "Thanks for the update. Best regards,",
            "Done. Best regards,",
        ]);
        let closings = analyzer.extract_closings(&messages);
        assert!(!closings.is_empty());
    }

    #[test]
    fn test_compare_styles_similar() {
        let analyzer = WritingStyleAnalyzer::new();
        let s1 = analyzer.analyze_email("Hi", "Hello, thank you for your email. Best regards.");
        let s2 = analyzer.analyze_email("Hi", "Hello, thanks for writing. Best regards.");
        let comparison = analyzer.compare_styles(&s1, &s2);
        assert!(comparison.similarity_score > 0.5);
    }

    #[test]
    fn test_compare_styles_different() {
        let analyzer = WritingStyleAnalyzer::new();
        let s1 = analyzer.analyze_email(
            "Formal",
            "Dear Sir, I am writing to formally request your assistance. Kind regards.",
        );
        let s2 = analyzer.analyze_whatsapp(&make_messages(&["hey lol omg brb"]));
        let comparison = analyzer.compare_styles(&s1, &s2);
        assert!(comparison.similarity_score < 0.8);
        assert!(!comparison.differences.is_empty());
    }

    #[test]
    fn test_grade_levels() {
        assert_eq!(classify_grade_level(-1.0), "Pre-K");
        assert_eq!(classify_grade_level(0.5), "Kindergarten");
        assert_eq!(classify_grade_level(4.0), "Elementary");
        assert_eq!(classify_grade_level(7.0), "Middle School");
        assert_eq!(classify_grade_level(10.0), "High School");
        assert_eq!(classify_grade_level(15.0), "College");
        assert_eq!(classify_grade_level(20.0), "Graduate");
    }

    #[test]
    fn test_syllable_counting() {
        assert_eq!(count_syllables("cat"), 1);
        assert_eq!(count_syllables("hello"), 2);
        assert_eq!(count_syllables("beautiful"), 3);
        assert!(count_syllables("") >= 1);
    }

    #[test]
    fn test_template_category_classification() {
        assert_eq!(
            classify_phrase_category("hello world"),
            TemplateCategory::Greeting
        );
        assert_eq!(
            classify_phrase_category("thank you so much"),
            TemplateCategory::ThankYou
        );
        assert_eq!(
            classify_phrase_category("sorry for the delay"),
            TemplateCategory::Apology
        );
        assert_eq!(
            classify_phrase_category("best regards"),
            TemplateCategory::Closing
        );
        assert_eq!(
            classify_phrase_category("just checking in"),
            TemplateCategory::FollowUp
        );
    }

    #[test]
    fn test_extract_variables() {
        let vars = extract_variables("dear {name}");
        assert_eq!(vars, vec!["name"]);

        let vars = extract_variables("hello {first} {last}");
        assert_eq!(vars, vec!["first", "last"]);

        let vars = extract_variables("no variables here");
        assert!(vars.is_empty());
    }

    #[test]
    fn test_fuzzy_similarity() {
        assert_eq!(fuzzy_similarity("hello world", "hello world"), 1.0);
        assert!(fuzzy_similarity("hello world", "hello there") > 0.0);
        assert_eq!(fuzzy_similarity("", ""), 0.0);
    }

    #[test]
    fn test_emoji_detection() {
        assert!(is_emoji('😀'));
        assert!(is_emoji('🎉'));
        assert!(!is_emoji('a'));
    }

    #[test]
    fn test_formality_high() {
        let analyzer = WritingStyleAnalyzer::new();
        let analysis = analyzer.analyze_email(
            "Request",
            "Dear Sir, I am writing to kindly request your assistance. Please advise at your earliest convenience. Regards.",
        );
        assert!(analysis.formality_score > 0.5);
    }

    #[test]
    fn test_formality_low() {
        let analyzer = WritingStyleAnalyzer::new();
        let analysis = analyzer.analyze_whatsapp(&make_messages(&[
            "hey lol wanna hang out",
            "omg brb gonna grab food",
        ]));
        assert!(analysis.formality_score < 0.5);
    }

    #[test]
    fn test_urgency_detection() {
        let analyzer = WritingStyleAnalyzer::new();
        let analysis = analyzer.analyze_email(
            "URGENT: Immediate Action Required",
            "This is critical and must be done immediately. Deadline is today.",
        );
        assert!(analysis.urgency_score > 0.0);
    }

    #[test]
    fn test_whatsapp_emoji_heavy() {
        let analyzer = WritingStyleAnalyzer::new();
        let messages = make_messages(&["hello 😀😊", "thanks 🙏", "great work 👍"]);
        let analysis = analyzer.analyze_whatsapp(&messages);
        assert!(analysis.emoji_usage > 0.5);
    }

    #[test]
    fn test_style_comparison_recommendation() {
        let analyzer = WritingStyleAnalyzer::new();
        let s1 = analyzer.analyze_email("Hi", "Hello. Best regards.");
        let s2 = analyzer.analyze_email("Hi", "Hello. Best regards.");
        let comparison = analyzer.compare_styles(&s1, &s2);
        assert!(comparison.recommendation.contains("similar"));
    }
}
