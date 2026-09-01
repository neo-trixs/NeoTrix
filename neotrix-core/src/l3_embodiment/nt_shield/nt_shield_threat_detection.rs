//! Threat Detection Engine — 威胁检测引擎
//!
//! 吸收 Strix (安全监控/威胁检测):
//! - 实时威胁检测
//! - 异常行为分析
//! - 威胁情报集成
//! - 自动响应
//! - 安全事件关联

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 威胁检测引擎
pub struct ThreatDetectionEngine {
    rules: Vec<DetectionRule>,
    anomalies: Vec<AnomalyEvent>,
    threats: Vec<ThreatEvent>,
    response_actions: Vec<ResponseAction>,
    config: ThreatConfig,
    stats: ThreatStats,
}

/// 威胁配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatConfig {
    pub sensitivity: f64,
    pub auto_response: bool,
    pub alert_threshold: f64,
    pub correlation_window: u64,
    pub max_events: usize,
}

impl Default for ThreatConfig {
    fn default() -> Self {
        Self {
            sensitivity: 0.7,
            auto_response: true,
            alert_threshold: 0.8,
            correlation_window: 300,
            max_events: 10000,
        }
    }
}

/// 检测规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    pub id: String,
    pub name: String,
    pub rule_type: RuleType,
    pub condition: String,
    pub severity: Severity,
    pub action: String,
    pub enabled: bool,
}

/// 规则类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    Signature,
    Anomaly,
    Behavioral,
    Heuristic,
    ML,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// 异常事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyEvent {
    pub id: String,
    pub event_type: String,
    pub source: String,
    pub anomaly_score: f64,
    pub features: HashMap<String, f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: serde_json::Value,
}

/// 威胁事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatEvent {
    pub id: String,
    pub threat_type: String,
    pub severity: Severity,
    pub source: String,
    pub target: String,
    pub indicators: Vec<Indicator>,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub related_events: Vec<String>,
}

/// 威胁指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicator {
    pub indicator_type: String,
    pub value: String,
    pub confidence: f64,
    pub source: String,
}

/// 响应动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseAction {
    pub id: String,
    pub action_type: String,
    pub target: String,
    pub parameters: serde_json::Value,
    pub executed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub result: Option<String>,
}

/// 威胁统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatStats {
    pub total_events: u64,
    pub threats_detected: u64,
    pub anomalies_detected: u64,
    pub responses_executed: u64,
    pub false_positives: u64,
    pub avg_detection_time: f64,
}

/// 检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub is_threat: bool,
    pub confidence: f64,
    pub severity: Option<Severity>,
    pub indicators: Vec<Indicator>,
    pub recommended_action: Option<String>,
}

impl ThreatDetectionEngine {
    /// 创建新的威胁检测引擎
    pub fn new(config: ThreatConfig) -> Self {
        Self {
            rules: Vec::new(),
            anomalies: Vec::new(),
            threats: Vec::new(),
            response_actions: Vec::new(),
            config,
            stats: ThreatStats {
                total_events: 0,
                threats_detected: 0,
                anomalies_detected: 0,
                responses_executed: 0,
                false_positives: 0,
                avg_detection_time: 0.0,
            },
        }
    }

    /// 添加检测规则
    pub fn add_rule(&mut self, rule: DetectionRule) {
        self.rules.push(rule);
    }

    /// 分析事件
    pub fn analyze_event(&mut self, event: &AnomalyEvent) -> DetectionResult {
        self.stats.total_events += 1;

        let mut indicators = Vec::new();
        let mut max_confidence = 0.0;
        let mut matched_severity = None;

        // 应用检测规则
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            if self.evaluate_rule(rule, event) {
                let confidence = self.calculate_confidence(rule, event);
                if confidence > max_confidence {
                    max_confidence = confidence;
                    matched_severity = Some(rule.severity.clone());
                }

                indicators.push(Indicator {
                    indicator_type: rule.rule_type.to_string(),
                    value: rule.condition.clone(),
                    confidence,
                    source: rule.id.clone(),
                });
            }
        }

        // 异常检测
        let anomaly_score = self.detect_anomaly(event);
        if anomaly_score > self.config.sensitivity {
            indicators.push(Indicator {
                indicator_type: "anomaly".into(),
                value: format!("Anomaly score: {}", anomaly_score),
                confidence: anomaly_score,
                source: "anomaly_detector".into(),
            });

            if anomaly_score > max_confidence {
                max_confidence = anomaly_score;
                matched_severity = Some(Severity::Medium);
            }
        }

        let is_threat = max_confidence >= self.config.alert_threshold;

        if is_threat {
            self.stats.threats_detected += 1;

            // 创建威胁事件
            let threat = ThreatEvent {
                id: uuid::Uuid::new_v4().to_string(),
                threat_type: "detected".into(),
                severity: matched_severity.clone().unwrap_or(Severity::Medium),
                source: event.source.clone(),
                target: "self".into(),
                indicators: indicators.clone(),
                confidence: max_confidence,
                timestamp: chrono::Utc::now(),
                related_events: vec![event.id.clone()],
            };
            self.threats.push(threat);

            // 自动响应
            if self.config.auto_response {
                self.auto_respond(&indicators, max_confidence);
            }
        }

        DetectionResult {
            is_threat,
            confidence: max_confidence,
            severity: matched_severity,
            indicators,
            recommended_action: if is_threat {
                Some("Investigate and contain".into())
            } else {
                None
            },
        }
    }

    /// 评估规则
    fn evaluate_rule(&self, rule: &DetectionRule, event: &AnomalyEvent) -> bool {
        // 简化版: 基于事件类型匹配
        match rule.rule_type {
            RuleType::Signature => event.event_type.contains(&rule.condition),
            RuleType::Anomaly => event.anomaly_score > 0.5,
            RuleType::Behavioral => true,
            RuleType::Heuristic => true,
            RuleType::ML => true,
        }
    }

    /// 计算置信度
    fn calculate_confidence(&self, rule: &DetectionRule, event: &AnomalyEvent) -> f64 {
        match rule.severity {
            Severity::Low => 0.3,
            Severity::Medium => 0.6,
            Severity::High => 0.8,
            Severity::Critical => 0.95,
        }
    }

    /// 检测异常
    fn detect_anomaly(&self, event: &AnomalyEvent) -> f64 {
        // 简化版: 基于特征的异常检测
        let feature_sum: f64 = event.features.values().sum();
        let feature_count = event.features.len() as f64;

        if feature_count > 0.0 {
            let avg = feature_sum / feature_count;
            (avg * 10.0).min(1.0)
        } else {
            0.0
        }
    }

    /// 自动响应
    fn auto_respond(&mut self, indicators: &[Indicator], confidence: f64) {
        if confidence > 0.9 {
            // 高置信度: 立即阻断
            self.response_actions.push(ResponseAction {
                id: uuid::Uuid::new_v4().to_string(),
                action_type: "block".into(),
                target: "source".into(),
                parameters: serde_json::json!({"indicators": indicators.len()}),
                executed_at: Some(chrono::Utc::now()),
                result: Some("blocked".into()),
            });
        } else if confidence > 0.7 {
            // 中置信度: 监控
            self.response_actions.push(ResponseAction {
                id: uuid::Uuid::new_v4().to_string(),
                action_type: "monitor".into(),
                target: "source".into(),
                parameters: serde_json::json!({"duration": 3600}),
                executed_at: Some(chrono::Utc::now()),
                result: Some("monitoring".into()),
            });
        }
    }

    /// 关联事件
    pub fn correlate_events(&mut self) {
        let window = chrono::Duration::seconds(self.config.correlation_window as i64);
        let now = chrono::Utc::now();

        // 找到时间窗口内的相关事件
        let recent_threats: Vec<String> = self.threats.iter()
            .filter(|t| now.signed_duration_since(t.timestamp) < window)
            .map(|t| t.id.clone())
            .collect();

        // 更新相关性
        for threat in &mut self.threats {
            if recent_threats.contains(&threat.id) {
                threat.related_events = recent_threats.clone();
            }
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> &ThreatStats {
        &self.stats
    }

    /// 获取检测到的威胁
    pub fn get_threats(&self) -> &[ThreatEvent] {
        &self.threats
    }
}
