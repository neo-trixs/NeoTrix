#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AiPattern {
    Notably,
    InSummary,
    ItIsWorth,
    ParallelConstruction,
    UnnecessaryFormality,
    ClichéOpening,
    RedundantEmphasis,
    OverusedTransition,
    ArtificialCertainty,
    RoboticEnumeration,
    FormulaicConclusion,
    GenericPraise,
    HedgeWords,
    Nominalization,
    PassiveVoiceOveruse,
    BuzzwordStacking,
    UnnaturalTransition,
    PaddingPhrase,
    OverpoliteRequest,
    MechanisticListing,
    SemanticRepetition,
    TemplateSalutation,
    FixedRatioRhythm,
    EmotionlessSummary,
}

impl AiPattern {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Notably => "notably",
            Self::InSummary => "in_summary",
            Self::ItIsWorth => "it_is_worth",
            Self::ParallelConstruction => "parallel_construction",
            Self::UnnecessaryFormality => "unnecessary_formality",
            Self::ClichéOpening => "cliche_opening",
            Self::RedundantEmphasis => "redundant_emphasis",
            Self::OverusedTransition => "overused_transition",
            Self::ArtificialCertainty => "artificial_certainty",
            Self::RoboticEnumeration => "robotic_enumeration",
            Self::FormulaicConclusion => "formulaic_conclusion",
            Self::GenericPraise => "generic_praise",
            Self::HedgeWords => "hedge_words",
            Self::Nominalization => "nominalization",
            Self::PassiveVoiceOveruse => "passive_voice_overuse",
            Self::BuzzwordStacking => "buzzword_stacking",
            Self::UnnaturalTransition => "unnatural_transition",
            Self::PaddingPhrase => "padding_phrase",
            Self::OverpoliteRequest => "overpolite_request",
            Self::MechanisticListing => "mechanistic_listing",
            Self::SemanticRepetition => "semantic_repetition",
            Self::TemplateSalutation => "template_salutation",
            Self::FixedRatioRhythm => "fixed_ratio_rhythm",
            Self::EmotionlessSummary => "emotionless_summary",
        }
    }

    pub fn chinese_triggers(&self) -> &'static [&'static str] {
        match self {
            Self::Notably => &["值得注意的是", "引人注目的是", "需要特别指出"],
            Self::InSummary => &["总的来说", "综上所述", "总而言之", "总体而言"],
            Self::ItIsWorth => &["值得一提的是", "值得关注的是", "值得注意的是"],
            Self::ParallelConstruction => &["不但...而且", "不仅...还", "既...又"],
            Self::UnnecessaryFormality => &["兹", "谨此", "特此", "予以", "加以"],
            Self::ClichéOpening => &["在当今社会", "在当今时代", "随着社会的不断发展", "众所周知"],
            Self::RedundantEmphasis => &["事实上", "实际上", "本质上", "从根本上说"],
            Self::OverusedTransition => &["然而", "但是", "不过", "另一方面"],
            Self::ArtificialCertainty => &["毫无疑问", "毋庸置疑", "不可否认", "显而易见"],
            Self::RoboticEnumeration => &["首先", "其次", "再次", "最后", "第一", "第二", "第三"],
            Self::FormulaicConclusion => &["因此", "所以", "由此可见", "综上"],
            Self::GenericPraise => &["意义重大", "价值深远", "影响深远", "不可或缺"],
            Self::HedgeWords => &["可能", "大概", "也许", "似乎", "某种程度"],
            Self::Nominalization => &["性", "化", "度", "率", "主义"],
            Self::PassiveVoiceOveruse => &["被", "受到", "遭到", "得以"],
            Self::BuzzwordStacking => &["赋能", "闭环", "抓手", "底层逻辑", "颗粒度", "对齐"],
            Self::UnnaturalTransition => &["基于此", "据此", "有鉴于此", "缘此"],
            Self::PaddingPhrase => &["需要注意的是", "需要说明的是", "需要强调的是"],
            Self::OverpoliteRequest => &["烦请", "敬请", "恳请", "望请"],
            Self::MechanisticListing => &["其一", "其二", "其三", "其四"],
            Self::SemanticRepetition => &["核心重点", "关键要点", "主要重点", "基本基础"],
            Self::TemplateSalutation => &["尊敬的", "亲爱的", "敬爱的"],
            Self::FixedRatioRhythm => &["四字成语", "八字对仗"],
            Self::EmotionlessSummary => &["综上所述", "经分析可知", "基于以上分析"],
        }
    }

    pub fn rewrite_advice(&self) -> &'static str {
        match self {
            Self::Notably => "replace the lead-in with a concrete fact or observation",
            Self::InSummary => "end with a specific takeaway, not a generic summary phrase",
            Self::ItIsWorth => "state the observation directly without the preamble",
            Self::ParallelConstruction => {
                "vary sentence structure; break long parallel chains into varied clauses"
            }
            Self::UnnecessaryFormality => "use everyday language matching the audience",
            Self::ClichéOpening => "start with a specific scene, data point, or question",
            Self::RedundantEmphasis => "remove — the statement should stand on its own",
            Self::OverusedTransition => "use implicit logical flow instead of explicit connectors",
            Self::ArtificialCertainty => {
                "acknowledge uncertainty when it exists; soften absolute claims"
            }
            Self::RoboticEnumeration => "embed list items in flowing prose, not numbered steps",
            Self::FormulaicConclusion => "conclude with a specific implication or call to action",
            Self::GenericPraise => "replace with specific, measurable impact statements",
            Self::HedgeWords => "commit or omit; avoid unnecessary hedging",
            Self::Nominalization => "prefer verb forms over noun forms",
            Self::PassiveVoiceOveruse => "prefer active voice; specify the actor",
            Self::BuzzwordStacking => "use plain language; one buzzword at most per paragraph",
            Self::UnnaturalTransition => {
                "use natural deduction or contrast instead of formulaic transitions"
            }
            Self::PaddingPhrase => {
                "remove — the information should be integrated into the main text"
            }
            Self::OverpoliteRequest => "be direct and specific about what you need",
            Self::MechanisticListing => "integrate items into flowing prose",
            Self::SemanticRepetition => "use one word; the other is redundant",
            Self::TemplateSalutation => {
                "match greeting to cultural context; avoid overused templates"
            }
            Self::FixedRatioRhythm => "break the cadence with varied sentence lengths",
            Self::EmotionlessSummary => {
                "add a personal judgment, implication, or emotional dimension"
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern: AiPattern,
    pub trigger: String,
    pub position: usize,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct HumanizerStats {
    pub total_scanned: u64,
    pub total_matches: u64,
    pub total_rewrites: u64,
    pub per_pattern: HashMap<&'static str, u64>,
    pub avg_rewrite_savings_chars: f64,
}

impl Default for HumanizerStats {
    fn default() -> Self {
        Self {
            total_scanned: 0,
            total_matches: 0,
            total_rewrites: 0,
            per_pattern: HashMap::new(),
            avg_rewrite_savings_chars: 0.0,
        }
    }
}

pub struct HumanizerEngine {
    pub detected: Vec<PatternMatch>,
    stats: HumanizerStats,
    enabled_patterns: Vec<AiPattern>,
}

impl HumanizerEngine {
    pub fn new() -> Self {
        Self {
            detected: Vec::new(),
            stats: HumanizerStats::default(),
            enabled_patterns: vec![
                AiPattern::Notably,
                AiPattern::InSummary,
                AiPattern::ItIsWorth,
                AiPattern::ClichéOpening,
                AiPattern::RedundantEmphasis,
                AiPattern::OverusedTransition,
                AiPattern::ArtificialCertainty,
                AiPattern::RoboticEnumeration,
                AiPattern::FormulaicConclusion,
                AiPattern::GenericPraise,
                AiPattern::BuzzwordStacking,
                AiPattern::PaddingPhrase,
                AiPattern::EmotionlessSummary,
            ],
        }
    }

    pub fn with_all_patterns(mut self) -> Self {
        self.enabled_patterns = vec![
            AiPattern::Notably,
            AiPattern::InSummary,
            AiPattern::ItIsWorth,
            AiPattern::ParallelConstruction,
            AiPattern::UnnecessaryFormality,
            AiPattern::ClichéOpening,
            AiPattern::RedundantEmphasis,
            AiPattern::OverusedTransition,
            AiPattern::ArtificialCertainty,
            AiPattern::RoboticEnumeration,
            AiPattern::FormulaicConclusion,
            AiPattern::GenericPraise,
            AiPattern::HedgeWords,
            AiPattern::Nominalization,
            AiPattern::PassiveVoiceOveruse,
            AiPattern::BuzzwordStacking,
            AiPattern::UnnaturalTransition,
            AiPattern::PaddingPhrase,
            AiPattern::OverpoliteRequest,
            AiPattern::MechanisticListing,
            AiPattern::SemanticRepetition,
            AiPattern::TemplateSalutation,
            AiPattern::FixedRatioRhythm,
            AiPattern::EmotionlessSummary,
        ];
        self
    }

    pub fn scan(&mut self, text: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();
        self.stats.total_scanned += 1;

        for pattern in &self.enabled_patterns {
            let triggers = pattern.chinese_triggers();
            for trigger in triggers {
                let mut start = 0;
                while let Some(pos) = text[start..].find(trigger) {
                    let abs_pos = start + pos;
                    let ctx_start = abs_pos.saturating_sub(20);
                    let ctx_end = (abs_pos + trigger.len() + 20).min(text.len());
                    let context = if ctx_start < abs_pos {
                        format!("...{}...", &text[ctx_start..ctx_end])
                    } else {
                        text[ctx_start..ctx_end].to_string()
                    };
                    let pm = PatternMatch {
                        pattern: *pattern,
                        trigger: trigger.to_string(),
                        position: abs_pos,
                        context,
                    };
                    matches.push(pm);
                    start = abs_pos + trigger.len();
                }
            }
        }

        self.stats.total_matches += matches.len() as u64;
        for m in &matches {
            *self.stats.per_pattern.entry(m.pattern.name()).or_insert(0) += 1;
        }
        self.detected = matches.clone();
        matches
    }

    pub fn rewrite_suggestion(&mut self, text: &str, matches: &[PatternMatch]) -> String {
        if matches.is_empty() {
            return text.to_string();
        }

        let mut result = text.to_string();
        let mut sorted = matches.to_vec();
        sorted.sort_by(|a, b| b.position.cmp(&a.position));

        for m in &sorted {
            let advice = m.pattern.rewrite_advice();
            let annotation = format!(" [AI_PATTERN: {} — {}]", m.pattern.name(), advice);
            result.insert_str(m.position + m.trigger.len(), &annotation);
        }

        self.stats.total_rewrites += 1;
        result
    }

    pub fn stats(&self) -> &HumanizerStats {
        &self.stats
    }

    pub fn tick(&mut self, text: Option<&str>) -> String {
        match text {
            Some(t) => {
                let matches = self.scan(t);
                if matches.is_empty() {
                    format!("humanizer:tick=clean")
                } else {
                    let rewrite = self.rewrite_suggestion(t, &matches);
                    format!(
                        "humanizer:tick={}_patterns={}_rewritten={}",
                        matches.len(),
                        self.stats.per_pattern.len(),
                        rewrite.len()
                    )
                }
            }
            None => {
                format!(
                    "humanizer:tick=idle_scanned={}_total_matches={}",
                    self.stats.total_scanned, self.stats.total_matches
                )
            }
        }
    }
}

impl Default for HumanizerEngine {
    fn default() -> Self {
        Self::new()
    }
}
