use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Distillation attack detector.
///
/// Analyzes request patterns for signs of model extraction:
/// - High request frequency from single source
/// - Repeated similar prompts (template-based extraction)
/// - Low temperature + high token usage (systematic sampling)
/// - Unusual domain/keyword patterns in API routing
#[derive(Debug, Clone)]
pub struct DistillationDetector {
    pub enabled: bool,
    pub request_log: VecDeque<RequestRecord>,
    pub source_stats: HashMap<String, SourceStats>,
    pub alert_history: Vec<DistillationAlert>,
    max_log_size: usize,
    threshold_requests_per_min: u64,
    threshold_similar_prompts: f64,
    window_seconds: u64,
}

/// A single request record for analysis.
#[derive(Debug, Clone)]
pub struct RequestRecord {
    pub timestamp: u64,
    pub source: String,
    pub prompt_hash: u64,
    pub prompt_length: usize,
    pub temperature: f32,
    pub max_tokens: u32,
    pub response_length: usize,
}

/// Aggregated statistics per request source.
#[derive(Debug, Clone, Default)]
pub struct SourceStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub first_seen: u64,
    pub last_seen: u64,
    pub unique_prompts: u64,
    pub max_rpm: f64,
    pub avg_temperature: f64,
    pub similarity_scores: VecDeque<f64>,
}

/// Alert generated when distillation is suspected.
#[derive(Debug, Clone)]
pub struct DistillationAlert {
    pub timestamp: u64,
    pub source: String,
    pub alert_type: AlertType,
    pub confidence: f64,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    HighRequestRate,
    PromptTemplateExtraction,
    SystematicSampling,
    KnownDistillationSource,
    ProxyChaining,
}

impl Default for DistillationDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DistillationDetector {
    pub fn new() -> Self {
        Self {
            enabled: true,
            request_log: VecDeque::new(),
            source_stats: HashMap::new(),
            alert_history: Vec::new(),
            max_log_size: 10000,
            threshold_requests_per_min: 60,
            threshold_similar_prompts: 0.85,
            window_seconds: 300,
        }
    }

    pub fn with_threshold(mut self, rpm: u64, similarity: f64) -> Self {
        self.threshold_requests_per_min = rpm;
        self.threshold_similar_prompts = similarity;
        self
    }

    /// Record a request and analyze for distillation patterns.
    pub fn record_request(
        &mut self,
        source: &str,
        prompt: &str,
        temperature: f32,
        max_tokens: u32,
        response_length: usize,
    ) -> Option<DistillationAlert> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let prompt_hash = simple_hash(prompt);
        let record = RequestRecord {
            timestamp: now,
            source: source.to_string(),
            prompt_hash,
            prompt_length: prompt.len(),
            temperature,
            max_tokens,
            response_length,
        };

        self.request_log.push_back(record);
        if self.request_log.len() > self.max_log_size {
            self.request_log.pop_front();
        }

        self.update_source_stats(source, now, prompt_hash, temperature);

        self.detect_anomalies(source, now)
    }

    /// Analyze recent log for anomalies.
    fn detect_anomalies(&mut self, source: &str, now: u64) -> Option<DistillationAlert> {
        // 1. High request rate
        if let Some(rate_alert) = self.check_request_rate(source, now) {
            return Some(rate_alert);
        }

        // 2. Prompt template extraction (similar prompts)
        if let Some(temp_alert) = self.check_template_extraction(source, now) {
            return Some(temp_alert);
        }

        // 3. Systematic sampling (low temp + high tokens + high volume)
        if let Some(sys_alert) = self.check_systematic_sampling(source, now) {
            return Some(sys_alert);
        }

        // 4. Known distillation source patterns
        if let Some(src_alert) = self.check_known_source(source) {
            return Some(src_alert);
        }

        None
    }

    fn check_request_rate(&mut self, source: &str, now: u64) -> Option<DistillationAlert> {
        let recent: Vec<&RequestRecord> = self.request_log
            .iter()
            .filter(|r| r.source == source && now - r.timestamp < 60)
            .collect();

        let rpm = recent.len() as f64;
        if rpm > self.threshold_requests_per_min as f64 {
            if let Some(stats) = self.source_stats.get_mut(source) {
                stats.max_rpm = stats.max_rpm.max(rpm);
            }
            let confidence = (rpm / 120.0).min(0.99);
            let alert = DistillationAlert {
                timestamp: now,
                source: source.to_string(),
                alert_type: AlertType::HighRequestRate,
                confidence,
                detail: format!("RPM={:.0}, threshold={}", rpm, self.threshold_requests_per_min),
            };
            self.alert_history.push(alert.clone());
            return Some(alert);
        }
        None
    }

    fn check_template_extraction(&mut self, source: &str, now: u64) -> Option<DistillationAlert> {
        let recent: Vec<&RequestRecord> = self.request_log
            .iter()
            .filter(|r| r.source == source && now - r.timestamp < self.window_seconds)
            .collect();

        if recent.len() < 5 {
            return None;
        }

        let mut similar_count = 0;
        let total = recent.len();
        for i in 0..total.min(20) {
            for j in (i + 1)..total.min(20) {
                if recent[i].prompt_hash == recent[j].prompt_hash
                    || (recent[i].prompt_length as i64 - recent[j].prompt_length as i64).abs() < 10
                {
                    similar_count += 1;
                }
            }
        }

        let pair_count = (total.min(20) * (total.min(20).saturating_sub(1)) / 2).max(1);
        let similarity_ratio = similar_count as f64 / pair_count as f64;

        if let Some(stats) = self.source_stats.get_mut(source) {
            stats.similarity_scores.push_back(similarity_ratio);
            if stats.similarity_scores.len() > 100 {
                stats.similarity_scores.pop_front();
            }
        }

        if similarity_ratio > self.threshold_similar_prompts {
            let alert = DistillationAlert {
                timestamp: now,
                source: source.to_string(),
                alert_type: AlertType::PromptTemplateExtraction,
                confidence: similarity_ratio,
                detail: format!("similarity_ratio={:.2}, threshold={:.2}", similarity_ratio, self.threshold_similar_prompts),
            };
            self.alert_history.push(alert.clone());
            return Some(alert);
        }
        None
    }

    fn check_systematic_sampling(&mut self, source: &str, now: u64) -> Option<DistillationAlert> {
        let recent: Vec<&RequestRecord> = self.request_log
            .iter()
            .filter(|r| r.source == source && now - r.timestamp < self.window_seconds)
            .collect();

        if recent.len() < 10 {
            return None;
        }

        let avg_temp: f32 = recent.iter().map(|r| r.temperature).sum::<f32>() / recent.len() as f32;
        let total_tokens: u32 = recent.iter().map(|r| r.max_tokens).sum();
        let avg_tokens_per_req = total_tokens as f64 / recent.len() as f64;

        // Systematic distillation: low temp + high tokens per request + high frequency
        if avg_temp < 0.3 && avg_tokens_per_req > 2000.0 && recent.len() > 20 {
            let confidence = ((1.0 - avg_temp as f64) * 0.4 + (avg_tokens_per_req / 4096.0).min(1.0) * 0.3
                + (recent.len() as f64 / 100.0).min(1.0) * 0.3)
                .min(0.99);
            let alert = DistillationAlert {
                timestamp: now,
                source: source.to_string(),
                alert_type: AlertType::SystematicSampling,
                confidence,
                detail: format!(
                    "avg_temp={:.2}, avg_tokens={:.0}, request_count={}",
                    avg_temp, avg_tokens_per_req, recent.len()
                ),
            };
            self.alert_history.push(alert.clone());
            return Some(alert);
        }
        None
    }

    fn check_known_source(&self, source: &str) -> Option<DistillationAlert> {
        let suspicious_keywords = [
            "deepseek", "moonshot", "minimax", "zhipu", "baichuan",
            "stepfun", "dashscope", "volces", "01ai",
        ];
        let source_lower = source.to_lowercase();
        for kw in &suspicious_keywords {
            if source_lower.contains(kw) {
                return Some(DistillationAlert {
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    source: source.to_string(),
                    alert_type: AlertType::KnownDistillationSource,
                    confidence: 0.7,
                    detail: format!("source contains '{}'", kw),
                });
            }
        }
        None
    }

    fn update_source_stats(&mut self, source: &str, now: u64, prompt_hash: u64, temperature: f32) {
        let stats = self.source_stats.entry(source.to_string()).or_insert_with(|| {
            SourceStats { first_seen: now, ..Default::default() }
        });

        stats.total_requests += 1;
        stats.last_seen = now;
        stats.avg_temperature = (stats.avg_temperature * (stats.total_requests as f64 - 1.0) + temperature as f64)
            / stats.total_requests as f64;

        // Track unique prompts
        if !stats.similarity_scores.iter().any(|&s| {
            ((prompt_hash as i64) - (s as i64)).unsigned_abs() < 100
        }) {
            stats.unique_prompts += 1;
        }
    }

    /// Export alerts for auditing / GWT broadcast.
    pub fn recent_alerts(&self, count: usize) -> Vec<&DistillationAlert> {
        self.alert_history.iter().rev().take(count).collect()
    }

    /// Reset detector state.
    pub fn reset(&mut self) {
        self.request_log.clear();
        self.source_stats.clear();
        self.alert_history.clear();
    }

    pub fn stats(&self) -> DetectorStats {
        let rpm: f64 = self.source_stats.values().map(|s| s.max_rpm).sum();
        DetectorStats {
            total_sources: self.source_stats.len() as u64,
            total_requests: self.source_stats.values().map(|s| s.total_requests).sum(),
            total_alerts: self.alert_history.len() as u64,
            max_rpm: rpm as u64,
            enabled: self.enabled,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectorStats {
    pub total_sources: u64,
    pub total_requests: u64,
    pub total_alerts: u64,
    pub max_rpm: u64,
    pub enabled: bool,
}

fn simple_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Evaluate a response text for signs of distillation-related artifacts.
pub fn analyze_response_pattern(response: &str) -> ResponseAnalysis {
    let length = response.len();
    let avg_line_len = if response.lines().count() > 0 {
        length as f64 / response.lines().count() as f64
    } else {
        0.0
    };
    let repetition_ratio = detect_repetition(response);

    ResponseAnalysis {
        suspicious_length: length > 10000,
        suspicious_repetition: repetition_ratio > 0.3,
        avg_line_length: avg_line_len,
        repetition_ratio,
        score: ((if length > 10000 { 0.3f64 } else { 0.0f64 })
            + (if repetition_ratio > 0.3 { 0.4f64 } else { 0.0f64 }))
            .min(1.0f64),
    }
}

#[derive(Debug, Clone)]
pub struct ResponseAnalysis {
    pub suspicious_length: bool,
    pub suspicious_repetition: bool,
    pub avg_line_length: f64,
    pub repetition_ratio: f64,
    pub score: f64,
}

fn detect_repetition(text: &str) -> f64 {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 10 {
        return 0.0;
    }
    let ngrams: Vec<Vec<&str>> = words.windows(5).map(|w| w.to_vec()).collect();
    if ngrams.is_empty() {
        return 0.0;
    }
    let total = ngrams.len() as f64;
    let unique: std::collections::HashSet<String> = ngrams
        .iter()
        .map(|ng| ng.join(" "))
        .collect();
    1.0 - (unique.len() as f64 / total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_default_state() {
        let d = DistillationDetector::new();
        assert!(d.enabled);
        assert!(d.request_log.is_empty());
        assert!(d.alert_history.is_empty());
    }

    #[test]
    fn test_record_request() {
        let mut d = DistillationDetector::new();
        let alert = d.record_request("127.0.0.1", "Hello", 0.7, 4096, 100);
        assert!(alert.is_none());
        assert_eq!(d.request_log.len(), 1);
    }

    #[test]
    fn test_high_request_rate() {
        let mut d = DistillationDetector::new();
        d.threshold_requests_per_min = 3;
        for _ in 0..5 {
            let alert = d.record_request("attacker.com", "test prompt", 0.7, 4096, 50);
            if alert.is_some() {
                let a = alert.unwrap();
                assert_eq!(a.alert_type, AlertType::HighRequestRate);
                return;
            }
        }
        panic!("Should have triggered rate alert");
    }

    #[test]
    fn test_template_extraction() {
        let mut d = DistillationDetector::new();
        d.threshold_similar_prompts = 0.3;
        for _ in 0..10 {
            d.record_request("extractor.com", "Translate the following to Chinese:", 0.7, 4096, 200);
        }
        let alerts = d.recent_alerts(10);
        let template_alerts: Vec<_> = alerts.iter().filter(|a| a.alert_type == AlertType::PromptTemplateExtraction).collect();
        assert!(!template_alerts.is_empty(), "should detect template extraction");
    }

    #[test]
    fn test_systematic_sampling() {
        let mut d = DistillationDetector::new();
        d.threshold_similar_prompts = 0.95; // high threshold to avoid template trigger
        // Need >20 requests with low temp + high tokens to trigger
        for i in 0..35 {
            let prompt = format!("Q{}: {}", i, "a".repeat((i % 10 + 5) * 10));
            d.record_request("distiller.com", &prompt, 0.1, 4096, 500);
        }
        let alerts = d.recent_alerts(35);
        let sys_alerts: Vec<_> = alerts.iter().filter(|a| a.alert_type == AlertType::SystematicSampling).collect();
        assert!(!sys_alerts.is_empty(), "should detect systematic sampling among {} total alerts {:?}",
            alerts.len(), alerts.iter().map(|a| format!("{:?}", a.alert_type)).collect::<Vec<_>>());
    }

    #[test]
    fn test_known_source_detection() {
        let d = DistillationDetector::new();
        let alert = d.check_known_source("api.deepseek.com");
        assert!(alert.is_some());
        assert_eq!(alert.unwrap().alert_type, AlertType::KnownDistillationSource);
    }

    #[test]
    fn test_clean_source_no_alert() {
        let d = DistillationDetector::new();
        let alert = d.check_known_source("api.anthropic.com");
        assert!(alert.is_none());
    }

    #[test]
    fn test_response_analysis_normal() {
        let text = "The quick brown fox jumps over the lazy dog.";
        let analysis = analyze_response_pattern(text);
        assert!(!analysis.suspicious_length);
        assert!(!analysis.suspicious_repetition);
        assert!(analysis.score < 0.5);
    }

    #[test]
    fn test_response_analysis_repetitive() {
        let text = "repeat repeat repeat repeat repeat repeat repeat repeat ";
        let text = text.repeat(20);
        let analysis = analyze_response_pattern(&text);
        assert!(analysis.suspicious_repetition || analysis.suspicious_length);
    }

    #[test]
    fn test_stats() {
        let d = DistillationDetector::new();
        let stats = d.stats();
        assert!(stats.enabled);
        assert_eq!(stats.total_sources, 0);
    }

    #[test]
    fn test_reset() {
        let mut d = DistillationDetector::new();
        d.record_request("test", "prompt", 0.7, 4096, 100);
        assert!(!d.request_log.is_empty());
        d.reset();
        assert!(d.request_log.is_empty());
        assert!(d.alert_history.is_empty());
    }

    #[test]
    fn test_multiple_sources() {
        let mut d = DistillationDetector::new();
        for i in 0..10 {
            d.record_request(&format!("source_{}", i % 3), "test prompt", 0.7, 4096, 50);
        }
        assert_eq!(d.source_stats.len(), 3);
    }
}
