//! Response Quality — 响应质量评估模块
//!
//! 评估 LLM 响应的质量指标，供响应缓存和选择决策消费。

/// 响应质量评分
#[derive(Debug, Clone)]
pub(crate) struct ResponseQualityScore {
    pub coherence: f64,
    pub relevance: f64,
    pub completeness: f64,
}

impl ResponseQualityScore {
    pub fn new(coherence: f64, relevance: f64, completeness: f64) -> Self {
        Self {
            coherence,
            relevance,
            completeness,
        }
    }

    pub fn composite(&self) -> f64 {
        (self.coherence + self.relevance + self.completeness) / 3.0
    }
}

/// 评估响应质量
pub(crate) fn evaluate_response_quality(content: &str) -> ResponseQualityScore {
    let coherence = if content.is_empty() {
        0.0
    } else {
        let len = content.len() as f64;
        (1.0 - (len / 10000.0).min(1.0)) * 0.8
    };
    let relevance = 0.7;
    let completeness = if content.contains('.') || content.contains('。') {
        0.9
    } else {
        0.5
    };
    ResponseQualityScore::new(coherence, relevance, completeness)
}
