use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnswerMode {
    Speed,
    Balanced,
    Quality,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceType {
    Web,
    Discussions,
    Academic,
    Code,
    Local,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: SourceType,
    pub relevance: f64,
    pub cached: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WidgetKind {
    Weather,
    Calculator,
    Stock,
    Definition,
    Translation,
    Time,
    News,
    None,
}

#[derive(Debug, Clone)]
pub struct ContextSource {
    pub content: String,
    pub source_type: SourceType,
    pub relevance_score: f64,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct AnswerEngineConfig {
    pub mode: AnswerMode,
    pub max_sources: usize,
    pub max_context_tokens: usize,
    pub citation_required: bool,
    pub stream_enabled: bool,
    pub temperature: f64,
}

impl Default for AnswerEngineConfig {
    fn default() -> Self {
        Self {
            mode: AnswerMode::Balanced,
            max_sources: 10,
            max_context_tokens: 4096,
            citation_required: true,
            stream_enabled: false,
            temperature: 0.3,
        }
    }
}

impl AnswerEngineConfig {
    pub fn for_mode(mode: AnswerMode) -> Self {
        match mode {
            AnswerMode::Speed => Self {
                mode,
                max_sources: 3,
                max_context_tokens: 1024,
                citation_required: false,
                stream_enabled: true,
                temperature: 0.1,
            },
            AnswerMode::Balanced => Self::default(),
            AnswerMode::Quality => Self {
                mode,
                max_sources: 25,
                max_context_tokens: 16384,
                citation_required: true,
                stream_enabled: false,
                temperature: 0.5,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnswerSegment {
    pub text: String,
    pub citations: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct AnswerResult {
    pub segments: Vec<AnswerSegment>,
    pub sources_used: Vec<SearchResult>,
    pub mode_used: AnswerMode,
    pub processing_time: Duration,
    pub token_count: usize,
}

pub struct ContextBuilder {
    max_tokens: usize,
    sources: Vec<ContextSource>,
}

impl ContextBuilder {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            sources: Vec::new(),
        }
    }

    pub fn add_source(&mut self, source: ContextSource) {
        self.sources.push(source);
        self.sources.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if self.sources.len() > self.max_tokens / 256 {
            self.sources.truncate(self.max_tokens / 256);
        }
    }

    pub fn assemble(&self) -> String {
        let mut ctx = String::new();
        let mut remaining = self.max_tokens;
        for source in &self.sources {
            if remaining == 0 {
                break;
            }
            let snippet: String = source.content.chars().take(remaining).collect();
            ctx.push_str(&snippet);
            ctx.push('\n');
            remaining = remaining.saturating_sub(snippet.chars().count());
        }
        ctx
    }

    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
}

#[derive(Debug, Clone)]
pub struct WidgetProvider {
    widgets: Vec<WidgetKind>,
}

impl Default for WidgetProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl WidgetProvider {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }

    pub fn detect_widget(&mut self, query: &str) -> WidgetKind {
        let lower = query.to_lowercase();
        if lower.contains("weather") || lower.contains("temperature") {
            self.widgets.push(WidgetKind::Weather);
            return WidgetKind::Weather;
        }
        if (lower.contains("+") || lower.contains("-") || lower.contains("*") || lower.contains("/"))
            && lower.chars().any(|c| c.is_ascii_digit()) {
                self.widgets.push(WidgetKind::Calculator);
                return WidgetKind::Calculator;
            }
        if lower.contains("stock") || lower.contains("price") || lower.contains("$") {
            self.widgets.push(WidgetKind::Stock);
            return WidgetKind::Stock;
        }
        if lower.starts_with("define") || lower.starts_with("what is") {
            self.widgets.push(WidgetKind::Definition);
            return WidgetKind::Definition;
        }
        if lower.contains("translate") || lower.contains("in ") {
            self.widgets.push(WidgetKind::Translation);
            return WidgetKind::Translation;
        }
        WidgetKind::None
    }

    pub fn active_widgets(&self) -> &[WidgetKind] {
        &self.widgets
    }
}

pub struct AnswerEngine {
    config: AnswerEngineConfig,
    context: ContextBuilder,
    widgets: WidgetProvider,
}

impl AnswerEngine {
    pub fn new(config: AnswerEngineConfig) -> Self {
        let max_tokens = config.max_context_tokens;
        Self {
            config,
            context: ContextBuilder::new(max_tokens),
            widgets: WidgetProvider::new(),
        }
    }

    pub fn with_mode(mode: AnswerMode) -> Self {
        Self::new(AnswerEngineConfig::for_mode(mode))
    }

    pub fn config(&self) -> &AnswerEngineConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: AnswerEngineConfig) {
        let max_tokens = config.max_context_tokens;
        self.config = config;
        self.context = ContextBuilder::new(max_tokens);
    }

    pub fn feed_context(&mut self, source: ContextSource) {
        self.context.add_source(source);
    }

    pub fn detect_widget(&mut self, query: &str) -> WidgetKind {
        self.widgets.detect_widget(query)
    }

    pub fn prepare_query(&self, query: &str) -> PreparedQuery {
        let mode = self.config.mode;
        let sources = self.context.source_count();
        let widget = self.widgets.active_widgets().first().copied().unwrap_or(WidgetKind::None);
        PreparedQuery {
            query: query.to_string(),
            mode,
            context_sources: sources,
            widget,
            max_sources: self.config.max_sources,
            temperature: self.config.temperature,
        }
    }

    pub fn rank_results(&self, results: &[SearchResult]) -> Vec<SearchResult> {
        let mut ranked = results.to_vec();
        ranked.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked.truncate(self.config.max_sources);
        ranked
    }

    pub fn build_answer(&self, query: &str, results: &[SearchResult]) -> AnswerResult {
        let start = Instant::now();
        let ranked = self.rank_results(results);
        let context = self.context.assemble();
        let token_estimate = query.len() + context.len() / 4;
        let segment = AnswerSegment {
            text: format!(
                "Answer for: {}\nSources: {}\nMode: {:?}",
                query,
                ranked.len(),
                self.config.mode
            ),
            citations: ranked.iter().map(|r| r.url.clone()).collect(),
            confidence: if ranked.is_empty() { 0.0 } else {
                (ranked.iter().map(|r| r.relevance).sum::<f64>() / ranked.len() as f64)
                    .max(0.0)
                    .min(1.0)
            },
        };
        AnswerResult {
            segments: vec![segment],
            sources_used: ranked,
            mode_used: self.config.mode,
            processing_time: start.elapsed(),
            token_count: token_estimate,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreparedQuery {
    pub query: String,
    pub mode: AnswerMode,
    pub context_sources: usize,
    pub widget: WidgetKind,
    pub max_sources: usize,
    pub temperature: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum AnswerEngineError {
    #[error("No sources available for query: {0}")]
    NoSources(String),
    #[error("Context too large: {0} tokens exceeds max of {1}")]
    ContextTooLarge(usize, usize),
    #[error("Widget detection failed")]
    WidgetDetectionFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_answer_mode_config() {
        let speed = AnswerEngineConfig::for_mode(AnswerMode::Speed);
        assert_eq!(speed.max_sources, 3);
        assert!(speed.stream_enabled);
        let quality = AnswerEngineConfig::for_mode(AnswerMode::Quality);
        assert_eq!(quality.max_sources, 25);
        assert!(!quality.stream_enabled);
    }

    #[test]
    fn test_widget_detection() {
        let mut wp = WidgetProvider::new();
        assert_eq!(wp.detect_widget("weather in Tokyo"), WidgetKind::Weather);
        assert_eq!(wp.detect_widget("25 + 17"), WidgetKind::Calculator);
        assert_eq!(wp.detect_widget("define gravity"), WidgetKind::Definition);
        assert_eq!(wp.detect_widget("hello world"), WidgetKind::None);
    }

    #[test]
    fn test_context_builder_ordering() {
        let mut cb = ContextBuilder::new(1000);
        cb.add_source(ContextSource {
            content: "low relevance".into(),
            source_type: SourceType::Web,
            relevance_score: 0.3,
            timestamp: Instant::now(),
        });
        cb.add_source(ContextSource {
            content: "high relevance".into(),
            source_type: SourceType::Academic,
            relevance_score: 0.9,
            timestamp: Instant::now(),
        });
        let assembled = cb.assemble();
        assert!(assembled.contains("high relevance"));
        assert!(assembled.contains("low relevance"));
    }

    #[test]
    fn test_answer_engine_rank() {
        let engine = AnswerEngine::with_mode(AnswerMode::Balanced);
        let results = vec![
            SearchResult {
                title: "a".into(), url: "http://a.com".into(), snippet: "".into(),
                source: SourceType::Web, relevance: 0.5, cached: false,
            },
            SearchResult {
                title: "b".into(), url: "http://b.com".into(), snippet: "".into(),
                source: SourceType::Academic, relevance: 0.9, cached: false,
            },
        ];
        let ranked = engine.rank_results(&results);
        assert_eq!(ranked[0].title, "b");
    }

    #[test]
    fn test_build_answer() {
        let engine = AnswerEngine::with_mode(AnswerMode::Speed);
        let results = vec![
            SearchResult {
                title: "test".into(), url: "http://test.com".into(), snippet: "content".into(),
                source: SourceType::Web, relevance: 0.8, cached: false,
            },
        ];
        let answer = engine.build_answer("test query", &results);
        assert_eq!(answer.sources_used.len(), 1);
        assert!(answer.segments[0].citations.contains(&"http://test.com".to_string()));
    }

    #[test]
    fn test_prepared_query() {
        let mut engine = AnswerEngine::with_mode(AnswerMode::Quality);
        engine.feed_context(ContextSource {
            content: "ctx".into(),
            source_type: SourceType::Academic,
            relevance_score: 1.0,
            timestamp: Instant::now(),
        });
        let pq = engine.prepare_query("test");
        assert_eq!(pq.mode, AnswerMode::Quality);
        assert_eq!(pq.max_sources, 25);
    }

    #[test]
    fn test_answer_engine_error() {
        let err = AnswerEngineError::NoSources("test".into());
        assert_eq!(format!("{}", err), "No sources available for query: test");
    }
}
