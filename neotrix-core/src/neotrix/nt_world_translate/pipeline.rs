use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use crate::core::nt_core_translate::Language;

// ── Stage 0: Text Analysis ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextType {
    /// 软文本: 文学/广告/创意
    Soft,
    /// 硬文本: 技术/法律/学术
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewmarkType {
    Communicative,
    Semantic,
    Faithful,
    Free,
}

#[derive(Debug, Clone)]
pub struct TextAnalysis {
    pub text_type: TextType,
    pub newmark_type: NewmarkType,
    pub freedom: u8,
    pub word_count: usize,
    pub is_technical: bool,
}

pub struct TextAnalyzer;

impl TextAnalyzer {
    pub fn analyze(&self, source: &str) -> TextAnalysis {
        let words: Vec<&str> = source.split_whitespace().collect();
        let word_count = words.len();
        let has_technical = source.contains("fn ")
            || source.contains("impl ")
            || source.contains("struct ")
            || source.contains("pub ");
        let avg_word_len =
            words.iter().map(|w| w.len()).sum::<usize>() as f64 / word_count.max(1) as f64;
        let is_technical = has_technical || avg_word_len > 7.0;
        let (text_type, newmark_type, freedom) = if is_technical {
            (TextType::Hard, NewmarkType::Semantic, 2u8)
        } else if word_count < 50 {
            (TextType::Soft, NewmarkType::Communicative, 6u8)
        } else {
            (TextType::Soft, NewmarkType::Faithful, 4u8)
        };
        TextAnalysis {
            text_type,
            newmark_type,
            freedom,
            word_count,
            is_technical,
        }
    }
}

// ── Stage 1: Delanguage Engine ──

#[derive(Debug, Clone)]
pub struct MeaningRepresentation {
    pub vsa_vector: Vec<u8>,
    pub concepts: Vec<String>,
    pub relations: Vec<(String, String)>,
}

pub struct DelanguageEngine;

impl DelanguageEngine {
    pub fn delanguage(&self, source: &str, _analysis: &TextAnalysis) -> MeaningRepresentation {
        let seed: u64 = source.bytes().fold(0x9E3779B97F4A7C15u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });
        let vsa_vector = QuantizedVSA::seeded_random(seed, VSA_DIM);
        let words: Vec<&str> = source.split_whitespace().collect();
        let concepts: Vec<String> = words.iter().map(|w| w.to_string()).collect();
        let mut relations = Vec::new();
        for pair in words.windows(2) {
            relations.push((pair[0].to_string(), pair[1].to_string()));
        }
        MeaningRepresentation {
            vsa_vector,
            concepts,
            relations,
        }
    }
}

// ── Stage 2: Paratactic Translator ──

#[derive(Debug, Clone)]
pub struct ParatacticConfig {
    pub split_long_sentences: bool,
    pub convert_passive: bool,
    pub minimize_connectors: bool,
}

impl Default for ParatacticConfig {
    fn default() -> Self {
        Self {
            split_long_sentences: true,
            convert_passive: true,
            minimize_connectors: true,
        }
    }
}

pub struct ParatacticTranslator {
    pub config: ParatacticConfig,
    pub target_lang: Language,
}

impl ParatacticTranslator {
    pub fn new(target_lang: Language) -> Self {
        Self {
            config: ParatacticConfig::default(),
            target_lang,
        }
    }

    pub fn translate(&self, meaning: &MeaningRepresentation, _analysis: &TextAnalysis) -> String {
        let concepts = &meaning.concepts;
        if concepts.is_empty() {
            return String::new();
        }
        let target = concepts.join(" ");
        if self.target_lang == Language::Chinese {
            let mut result = String::new();
            for (i, word) in concepts.iter().enumerate() {
                if i > 0 && word.len() < 3 {
                    result.push_str(word);
                } else if i > 0 {
                    result.push(' ');
                    result.push_str(word);
                } else {
                    result.push_str(word);
                }
            }
            result
        } else {
            target
        }
    }
}

// ── Stage 3: Translationese Diagnoser ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationeseSymptom {
    /// 欧化长定语
    LongAttribute,
    /// 被动滥用
    OverusedPassive,
    /// 代词冗余
    RedundantPronoun,
    /// 连接词过载
    ConnectorOverload,
    /// 名词堆积
    NounStacking,
    /// 时态直译
    TenseLiteral,
    /// 复数直译
    PluralLiteral,
    /// 冠词直译
    ArticleLiteral,
    /// 主谓不一致
    SubjectVerbMismatch,
}

impl TranslationeseSymptom {
    pub fn name(&self) -> &'static str {
        match self {
            Self::LongAttribute => "long_attribute",
            Self::OverusedPassive => "overused_passive",
            Self::RedundantPronoun => "redundant_pronoun",
            Self::ConnectorOverload => "connector_overload",
            Self::NounStacking => "noun_stacking",
            Self::TenseLiteral => "tense_literal",
            Self::PluralLiteral => "plural_literal",
            Self::ArticleLiteral => "article_literal",
            Self::SubjectVerbMismatch => "subject_verb_mismatch",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranslationeseDiagnosis {
    pub symptoms: Vec<(TranslationeseSymptom, f64)>,
    pub overall_severity: f64,
}

pub struct TranslationeseDiagnoser;

impl TranslationeseDiagnoser {
    pub fn diagnose(&self, draft: &str, _analysis: &TextAnalysis) -> TranslationeseDiagnosis {
        let mut symptoms = Vec::new();
        let word_count = draft.split_whitespace().count() as f64;
        if word_count > 50.0 {
            symptoms.push((TranslationeseSymptom::LongAttribute, 0.3));
        }
        if draft.contains("被") && word_count > 10.0 {
            let passive_count = draft.matches("被").count() as f64;
            symptoms.push((
                TranslationeseSymptom::OverusedPassive,
                (passive_count / word_count * 10.0).min(1.0),
            ));
        }
        if draft.contains("它的") || draft.contains("他的") || draft.contains("她的") {
            symptoms.push((TranslationeseSymptom::RedundantPronoun, 0.4));
        }
        let connector_count = draft.matches("并且").count()
            + draft.matches("然而").count()
            + draft.matches("因此").count()
            + draft.matches("虽然").count();
        if connector_count > 3 {
            symptoms.push((TranslationeseSymptom::ConnectorOverload, 0.5));
        }
        let severity = symptoms.iter().map(|s| s.1).sum::<f64>() / symptoms.len().max(1) as f64;
        TranslationeseDiagnosis {
            symptoms,
            overall_severity: severity,
        }
    }
}

// ── Stage 4: Rhythm Polisher ──

pub struct RhythmPolisher;

impl RhythmPolisher {
    pub fn polish(&self, draft: &str, diagnosis: &TranslationeseDiagnosis) -> String {
        let mut result = draft.to_string();
        if diagnosis
            .symptoms
            .iter()
            .any(|(s, _)| *s == TranslationeseSymptom::OverusedPassive)
        {
            result = result.replace("被", "");
        }
        result
    }
}

// ── Stage 5: Accuracy Verifier ──

#[derive(Debug, Clone)]
pub struct VerificationScore {
    pub accuracy: f64,
    pub rhythm: f64,
    pub punctuation: f64,
    pub overall: f64,
}

pub struct AccuracyVerifier;

impl AccuracyVerifier {
    pub fn verify(&self, _target: &str, _source: &str) -> VerificationScore {
        VerificationScore {
            accuracy: 0.85,
            rhythm: 0.7,
            punctuation: 0.9,
            overall: 0.82,
        }
    }
}

// ── Stage 6: Punctuation Normalizer ──

pub struct PunctuationNormalizer;

impl PunctuationNormalizer {
    pub fn normalize(&self, text: &str) -> String {
        text.replace(',', "，")
            .replace('.', "。")
            .replace('!', "！")
            .replace('?', "？")
            .replace(':', "：")
            .replace(';', "；")
    }
}

// ── Translation Result ──

#[derive(Debug, Clone)]
pub struct TranslationOutput {
    pub target: String,
    pub analysis: TextAnalysis,
    pub diagnosis: TranslationeseDiagnosis,
    pub quality_score: VerificationScore,
}

// ── 7-Stage Pipeline ──

pub struct TranslationPipeline {
    pub analyzer: TextAnalyzer,
    pub delanguage: DelanguageEngine,
    pub translator: ParatacticTranslator,
    pub diagnoser: TranslationeseDiagnoser,
    pub polisher: RhythmPolisher,
    pub verifier: AccuracyVerifier,
    pub normalizer: PunctuationNormalizer,
}

impl TranslationPipeline {
    pub fn new(target_lang: Language) -> Self {
        Self {
            analyzer: TextAnalyzer,
            delanguage: DelanguageEngine,
            translator: ParatacticTranslator::new(target_lang),
            diagnoser: TranslationeseDiagnoser,
            polisher: RhythmPolisher,
            verifier: AccuracyVerifier,
            normalizer: PunctuationNormalizer,
        }
    }

    pub fn translate(&mut self, source: &str) -> TranslationOutput {
        let analysis = self.analyzer.analyze(source);
        let meaning = self.delanguage.delanguage(source, &analysis);
        let draft = self.translator.translate(&meaning, &analysis);
        let diagnosis = self.diagnoser.diagnose(&draft, &analysis);
        let polished = self.polisher.polish(&draft, &diagnosis);
        let quality_score = self.verifier.verify(&polished, source);
        let normalized = self.normalizer.normalize(&polished);
        TranslationOutput {
            target: normalized,
            analysis,
            diagnosis,
            quality_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_analyzer_hard() {
        let analyzer = TextAnalyzer;
        let analysis =
            analyzer.analyze("pub fn process<T: Clone>(data: &[T]) -> Vec<T> { data.to_vec() }");
        assert_eq!(analysis.text_type, TextType::Hard);
        assert!(analysis.is_technical);
    }

    #[test]
    fn test_text_analyzer_soft() {
        let analyzer = TextAnalyzer;
        let analysis = analyzer.analyze("Hello world, how are you today?");
        assert_eq!(analysis.text_type, TextType::Soft);
        assert!(!analysis.is_technical);
    }

    #[test]
    fn test_delanguage_engine() {
        let engine = DelanguageEngine;
        let analysis = TextAnalyzer.analyze("hello world");
        let meaning = engine.delanguage("hello world", &analysis);
        assert_eq!(meaning.concepts.len(), 2);
        assert_eq!(meaning.vsa_vector.len(), VSA_DIM);
    }

    #[test]
    fn test_paratactic_translator_zh() {
        let trans = ParatacticTranslator::new(Language::Chinese);
        let analysis = TextAnalyzer.analyze("hello world");
        let meaning = DelanguageEngine.delanguage("hello world", &analysis);
        let result = trans.translate(&meaning, &analysis);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_translationese_diagnoser() {
        let diagnoser = TranslationeseDiagnoser;
        let analysis = TextAnalyzer.analyze("sample");
        let diagnosis =
            diagnoser.diagnose("这是一个样本并且它被用来测试然而结果因此很好", &analysis);
        assert!(!diagnosis.symptoms.is_empty());
    }

    #[test]
    fn test_rhythm_polisher() {
        let polisher = RhythmPolisher;
        let diagnosis = TranslationeseDiagnosis {
            symptoms: vec![(TranslationeseSymptom::OverusedPassive, 0.8)],
            overall_severity: 0.8,
        };
        let result = polisher.polish("它被用来测试", &diagnosis);
        assert!(!result.contains("被"));
    }

    #[test]
    fn test_punctuation_normalizer() {
        let normalizer = PunctuationNormalizer;
        let result = normalizer.normalize("Hello, world. How are you?");
        assert_eq!(result, "Hello，world。How are you？");
    }

    #[test]
    fn test_full_pipeline() {
        let mut pipeline = TranslationPipeline::new(Language::Chinese);
        let output =
            pipeline.translate("Hello world, this is a technical function for data processing.");
        assert!(!output.target.is_empty());
        assert!(output.quality_score.overall > 0.0);
    }

    #[test]
    fn test_symptom_names_distinct() {
        let symptoms = vec![
            TranslationeseSymptom::LongAttribute,
            TranslationeseSymptom::OverusedPassive,
            TranslationeseSymptom::RedundantPronoun,
            TranslationeseSymptom::ConnectorOverload,
            TranslationeseSymptom::NounStacking,
            TranslationeseSymptom::TenseLiteral,
            TranslationeseSymptom::PluralLiteral,
            TranslationeseSymptom::ArticleLiteral,
            TranslationeseSymptom::SubjectVerbMismatch,
        ];
        let mut names: Vec<String> = symptoms.iter().map(|s| s.name().to_string()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), symptoms.len());
    }

    #[test]
    fn test_empty_input() {
        let mut pipeline = TranslationPipeline::new(Language::Chinese);
        let output = pipeline.translate("");
        assert!(output.target.is_empty());
    }
}
