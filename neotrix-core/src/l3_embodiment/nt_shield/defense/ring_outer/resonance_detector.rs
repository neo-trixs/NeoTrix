//! Resonance Detector - 共振检测器
//!
//! 多源信号共振确认：防止单点误报
//! 设计原则：多个独立信号源确认同一威胁

/// 共振信号源
#[derive(Debug, Clone)]
pub struct ResonanceSource {
    pub name: String,
    pub confidence: f64,
    pub weight: f64,
}

/// 共振检测结果
#[derive(Debug, Clone)]
pub struct ResonanceResult {
    pub resonating: bool,
    pub confidence: f64,
    pub sources: Vec<ResonanceSource>,
    pub confirmed_threats: Vec<String>,
}

/// 共振检测器
pub struct ResonanceDetector {
    /// 共振阈值
    resonance_threshold: f64,
    /// 最少确认源数
    min_sources: usize,
}

impl ResonanceDetector {
    pub fn new() -> Self {
        Self {
            resonance_threshold: 0.6,
            min_sources: 2,
        }
    }

    /// 检测共振
    pub fn detect(&self, sources: &[ResonanceSource]) -> ResonanceResult {
        if sources.len() < self.min_sources {
            return ResonanceResult {
                resonating: false,
                confidence: 0.0,
                sources: sources.to_vec(),
                confirmed_threats: Vec::new(),
            };
        }

        // 计算加权置信度
        let total_weight: f64 = sources.iter().map(|s| s.weight).sum();
        let weighted_confidence: f64 = sources
            .iter()
            .map(|s| s.confidence * s.weight)
            .sum::<f64>() / total_weight;

        let resonating = weighted_confidence > self.resonance_threshold
            && sources.len() >= self.min_sources;

        let confirmed_threats = if resonating {
            vec!["Multi-source confirmed threat".to_string()]
        } else {
            Vec::new()
        };

        ResonanceResult {
            resonating,
            confidence: weighted_confidence,
            sources: sources.to_vec(),
            confirmed_threats,
        }
    }
}

impl Default for ResonanceDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_resonance_few_sources() {
        let detector = ResonanceDetector::new();
        let sources = vec![ResonanceSource {
            name: "single".to_string(),
            confidence: 0.9,
            weight: 1.0,
        }];
        let result = detector.detect(&sources);
        assert!(!result.resonating);
    }

    #[test]
    fn test_resonance_multiple_sources() {
        let detector = ResonanceDetector::new();
        let sources = vec![
            ResonanceSource {
                name: "input_check".to_string(),
                confidence: 0.8,
                weight: 1.0,
            },
            ResonanceSource {
                name: "output_check".to_string(),
                confidence: 0.7,
                weight: 1.0,
            },
            ResonanceSource {
                name: "behavior_check".to_string(),
                confidence: 0.6,
                weight: 1.0,
            },
        ];
        let result = detector.detect(&sources);
        assert!(result.resonating);
        assert!(result.confidence > 0.6);
    }
}
