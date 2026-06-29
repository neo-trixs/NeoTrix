use crate::core::nt_core_truth::emotion_tag::EmotionAnalyzer;

#[derive(Debug, Clone)]
pub enum QuestionType {
    None,
    YesNo,
    WhQuestion,
    Rhetorical,
    TagQuestion,
    Imperative,
}

impl QuestionType {
    fn from_text(text: &str) -> Self {
        let lower = text.trim().to_lowercase();
        if lower.ends_with('?') {
            if lower.starts_with("isn't")
                || lower.starts_with("aren't")
                || lower.starts_with("don't")
                || lower.starts_with("doesn't")
            {
                return QuestionType::TagQuestion;
            }
            if lower.starts_with("what")
                || lower.starts_with("why")
                || lower.starts_with("how")
                || lower.starts_with("when")
                || lower.starts_with("where")
                || lower.starts_with("which")
                || lower.starts_with("who")
                || lower.starts_with("whom")
                || lower.starts_with("whose")
            {
                return QuestionType::WhQuestion;
            }
            if lower.starts_with("do")
                || lower.starts_with("does")
                || lower.starts_with("did")
                || lower.starts_with("is")
                || lower.starts_with("are")
                || lower.starts_with("was")
                || lower.starts_with("were")
                || lower.starts_with("can")
                || lower.starts_with("could")
                || lower.starts_with("will")
                || lower.starts_with("would")
                || lower.starts_with("shall")
                || lower.starts_with("should")
                || lower.starts_with("may")
                || lower.starts_with("might")
                || lower.starts_with("has")
                || lower.starts_with("have")
                || lower.starts_with("had")
            {
                return QuestionType::YesNo;
            }
            return QuestionType::Rhetorical;
        }
        if lower.starts_with("please")
            || lower.starts_with("could you")
            || lower.starts_with("would you")
            || lower.starts_with("can you")
        {
            return QuestionType::Imperative;
        }
        QuestionType::None
    }
}

#[derive(Debug, Clone)]
pub struct LinguisticFeatureSet {
    pub question_type: QuestionType,
    pub formality: f64,
    pub urgency: f64,
    pub aggression: f64,
    pub politeness: f64,
    pub exclamation_count: usize,
    pub question_count: usize,
    pub all_caps_words: usize,
    pub word_count: usize,
}

impl LinguisticFeatureSet {
    pub fn new() -> Self {
        Self {
            question_type: QuestionType::None,
            formality: 0.5,
            urgency: 0.0,
            aggression: 0.0,
            politeness: 0.5,
            exclamation_count: 0,
            question_count: 0,
            all_caps_words: 0,
            word_count: 0,
        }
    }

    pub fn analyze(text: &str) -> Self {
        let lower = text.to_lowercase();
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();
        let exclamation_count = text.chars().filter(|&c| c == '!').count();
        let question_count = text.chars().filter(|&c| c == '?').count();
        let all_caps_words = words
            .iter()
            .filter(|w| w.len() > 1 && w.chars().all(|c| c.is_uppercase()))
            .count();
        let question_type = QuestionType::from_text(text);

        let formality = Self::compute_formality(&lower, &words);
        let politeness = Self::compute_politeness(&lower);
        let urgency = Self::compute_urgency(&lower, &words, exclamation_count);
        let aggression = Self::compute_aggression(&lower, &words, all_caps_words);

        Self {
            question_type,
            formality,
            urgency,
            aggression,
            politeness,
            exclamation_count,
            question_count,
            all_caps_words,
            word_count,
        }
    }

    fn compute_formality(lower: &str, _words: &[&str]) -> f64 {
        let formal_count = FORMAL_MARKERS.iter().filter(|m| lower.contains(*m)).count();
        let informal_count = INFORMAL_MARKERS
            .iter()
            .filter(|m| lower.contains(*m))
            .count();
        if formal_count + informal_count == 0 {
            return 0.5;
        }
        (formal_count as f64 + 1.0) / (formal_count + informal_count + 1) as f64
    }

    fn compute_politeness(lower: &str) -> f64 {
        let polite_hits = POLITE_MARKERS.iter().filter(|m| lower.contains(*m)).count();
        let impolite_hits = IMPOLITE_MARKERS
            .iter()
            .filter(|m| lower.contains(*m))
            .count();
        if polite_hits + impolite_hits == 0 {
            return 0.5;
        }
        (polite_hits as f64 + 1.0) / (polite_hits + impolite_hits + 1) as f64
    }

    fn compute_urgency(lower: &str, words: &[&str], excl: usize) -> f64 {
        let time_urgent = URGENCY_MARKERS
            .iter()
            .filter(|m| lower.contains(*m))
            .count();
        let excl_score = (excl as f64).min(3.0) / 3.0;
        let short_score = if words.len() <= 3 { 0.5 } else { 0.0 };
        let score = (time_urgent as f64 * 0.15 + excl_score * 0.25 + short_score).min(1.0);
        score
    }

    fn compute_aggression(lower: &str, _words: &[&str], all_caps: usize) -> f64 {
        let aggressive = AGGRESSIVE_MARKERS
            .iter()
            .filter(|m| lower.contains(*m))
            .count();
        let caps_score = (all_caps as f64 * 0.2).min(0.6);
        let score = (aggressive as f64 * 0.15 + caps_score).min(1.0);
        score
    }
}

impl Default for LinguisticFeatureSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct HumanEmotionReading {
    pub valence: f64,
    pub arousal: f64,
    pub dominance: f64,
    pub primary_emotion: String,
    pub confidence: f64,
    pub linguistic: LinguisticFeatureSet,
    pub source_text_snippet: String,
}

impl HumanEmotionReading {
    pub fn neutral() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.0,
            dominance: 0.5,
            primary_emotion: "neutral".to_string(),
            confidence: 0.0,
            linguistic: LinguisticFeatureSet::new(),
            source_text_snippet: String::new(),
        }
    }

    pub fn is_significant(&self) -> bool {
        self.confidence > 0.3 && (self.valence.abs() > 0.3 || self.arousal > 0.3)
    }

    pub fn dominant_tone_label(&self) -> &'static str {
        if !self.is_significant() {
            return "neutral";
        }
        if self.valence > 0.3 && self.arousal > 0.5 {
            "excited"
        } else if self.valence > 0.3 {
            "happy"
        } else if self.valence < -0.3 && self.arousal > 0.5 {
            "angry"
        } else if self.valence < -0.3 {
            "sad"
        } else if self.arousal > 0.6 {
            "anxious"
        } else {
            "neutral"
        }
    }
}

#[derive(Clone)]
pub struct HumanEmotionDetector {
    analyzer: EmotionAnalyzer,
}

impl Default for HumanEmotionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl HumanEmotionDetector {
    pub fn new() -> Self {
        Self {
            analyzer: EmotionAnalyzer::new(),
        }
    }

    pub fn detect_from_text(&self, text: &str, source: &str) -> HumanEmotionReading {
        let snippet = text.chars().take(200).collect::<String>();
        let linguistic = LinguisticFeatureSet::analyze(text);

        let tag = self.analyzer.tag_text(text, source);

        let valence = tag.valence;
        let arousal = tag.arousal.clamp(0.0, 1.0);
        let dominance = tag.dominance.clamp(0.0, 1.0);

        let primary_emotion = if linguistic.question_count > 0 && tag.primary_emotion == "neutral" {
            "curious"
        } else {
            &tag.primary_emotion
        };

        let word_hit_weight = (tag.word_hits.len() as f64 * 0.1).min(0.4);
        let ling_weight = if linguistic.question_count > 0 {
            0.3
        } else if linguistic.exclamation_count > 0 {
            0.2
        } else {
            0.0
        };
        let confidence = (0.3 + word_hit_weight + ling_weight).min(1.0);

        HumanEmotionReading {
            valence,
            arousal,
            dominance,
            primary_emotion: primary_emotion.to_string(),
            confidence,
            linguistic,
            source_text_snippet: snippet,
        }
    }
}

const FORMAL_MARKERS: &[&str] = &[
    "would you",
    "could you",
    "i would appreciate",
    "please",
    "kindly",
    "regarding",
    "furthermore",
    "nevertheless",
    "consequently",
    "therefore",
    "as per",
    "in accordance",
    "i understand",
    "i acknowledge",
    "per your request",
    "at your earliest convenience",
    "thank you for your",
];

const INFORMAL_MARKERS: &[&str] = &[
    "gonna", "wanna", "yeah", "nah", "yep", "nope", "cool", "awesome", "hey", "btw", "lol", "omg",
    "idk", "imo", "tbh", "thx", "pls", "u r", "ur", "dunno", "gotta", "kinda", "sorta", "cuz",
    "cause",
];

const POLITE_MARKERS: &[&str] = &[
    "please",
    "thank you",
    "thanks",
    "appreciate",
    "kindly",
    "would you",
    "could you",
    "may i",
    "if you don't mind",
    "sorry",
    "excuse me",
    "pardon",
    "i'd be grateful",
    "i'd appreciate",
];

const IMPOLITE_MARKERS: &[&str] = &[
    "shut up",
    "shut",
    "idiot",
    "stupid",
    "useless",
    "worthless",
    "damn",
    "hell",
    "crap",
    "freaking",
    "ridiculous",
    "unbelievable",
];

const URGENCY_MARKERS: &[&str] = &[
    "urgent",
    "asap",
    "immediately",
    "right now",
    "quickly",
    "hurry",
    "deadline",
    "emergency",
    "critical",
    "important",
    "fast",
    "soon",
    "today",
    "now",
    "temporary",
    "limited time",
];

const AGGRESSIVE_MARKERS: &[&str] = &[
    "never",
    "always",
    "everyone",
    "nobody",
    "disgusting",
    "terrible",
    "horrible",
    "awful",
    "unacceptable",
    "intolerable",
    "outrageous",
    "insulting",
    "offensive",
    "absurd",
];
