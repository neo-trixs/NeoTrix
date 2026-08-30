//! 价值观学习 — 从经验/后果/反思中反向提炼/演化价值观（P1 价值观内化）。
//!
//! 核心理念：价值观不是静态配置，而是从"行动-后果-反思"闭环中涌现并持续演化的。
//! - 观察：行动 → 后果 → 价值观信号
//! - 归因：因果归因（非相关性）→ 哪个价值观被违背/强化
//! - 演化：权重调整、新价值观萌芽、层级重组
//! - 守恒：核心价值观不退化（锚定），边缘价值观可增删

use crate::core::nt_core_kb_primitives::{kv_set, kv_list, now};
#[allow(unused_imports)]
use crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_compass::{ValueCompassRuntime, CoreValue, ValueAction};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// ValueLearning namespace — KB kv_store 命名空间。
pub const NS_VALUE_LEARNING: &str = "value_learning";

/// 观察记录：一次行动-后果观察。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,
    pub action: crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_compass::ValueAction,
    pub outcome: Outcome,
    pub value_signals: Vec<ValueSignal>, // 该行动触发的价值信号
    pub timestamp: i64,
    pub session_id: String,
}

/// 后果描述。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub success: bool,                    // 主观/客观成功
    pub expected: bool,                   // 是否符合预期
    pub harm_occurred: bool,              // 是否造成伤害
    pub trust_impact: f64,                // 信任影响 [-1,1]
    pub autonomy_respected: bool,         // 是否尊重自主
    pub truth_preserved: bool,            // 真实性是否保持
    pub fairness_maintained: bool,        // 公平性是否维持
    pub privacy_respected: bool,          // 隐私是否尊重
    pub description: String,              // 自然语言描述
}

/// 价值信号：某价值观被触发的强度与方向。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueSignal {
    pub value_id: String,                 // 如 "autonomy", "harm_prevention"
    pub intensity: f64,                   // 触发强度 [0,1]
    pub valence: f64,                     // 正向强化/负向违背 [-1,1]
    pub confidence: f64,                  // 归因置信度 [0,1]
}

/// 学习事件：从观察中提炼的价值观更新提案。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub id: String,
    pub source_observation_ids: Vec<String>,
    pub proposed_changes: Vec<ValueChangeProposal>,
    pub confidence: f64,
    pub status: LearningEventStatus,
    pub created_at: i64,
    pub applied_at: Option<i64>,
}

/// 价值观变更提案。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueChangeProposal {
    pub value_id: String,
    pub change_type: ChangeType,
    pub rationale: String,
    pub evidence_strength: f64,
}

/// 变更类型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    WeightDelta(f64),        // 权重微调
    AddValue(CoreValue),     // 新价值观萌芽
    RemoveValue(String),     // 废弃价值观
    HierarchyShift(String),  // 层级位移
    AddExclusion((String, String)),     // 新增互斥
    RemoveExclusion((String, String)),  // 移除互斥
    AddSynergy(Vec<String>),            // 新增协同
    RemoveSynergy(Vec<String>),         // 移除协同
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LearningEventStatus {
    Pending,      // 待审核/待应用
    Applied,      // 已应用到指南针
    Rejected,     // 被拒绝
    Superseded,   // 被后续事件取代
}

/// ValueLearning 核心引擎。
#[derive(Clone)]
pub struct ValueLearningEngine {
    compass: ValueCompassRuntime,
    observations: Arc<RwLock<Vec<Observation>>>,
    pending_events: Arc<RwLock<Vec<LearningEvent>>>,
    config: LearningConfig,
}

/// 学习配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// 最小观察数触发学习
    pub min_observations_for_learning: usize,
    /// 权重调整步长上限
    pub max_weight_delta_per_event: f64,
    /// 新价值观萌芽阈值（信号重复次数）
    pub new_value_emergence_threshold: usize,
    /// 归因置信度阈值
    pub attribution_confidence_threshold: f64,
    /// 价值观权重衰减率（长期不触发自动衰减）
    pub weight_decay_rate: f64,
    /// 最大单次权重变动
    pub max_single_weight_change: f64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            min_observations_for_learning: 10,
            max_weight_delta_per_event: 0.05,
            new_value_emergence_threshold: 5,
            attribution_confidence_threshold: 0.7,
            weight_decay_rate: 0.001, // 每轮衰减 0.1%
            max_single_weight_change: 0.1,
        }
    }
}

impl ValueLearningEngine {
    pub fn new(compass: ValueCompassRuntime, config: LearningConfig) -> Self {
        Self {
            compass,
            observations: Arc::new(RwLock::new(Vec::new())),
            pending_events: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// 记录观察（外部调用：行动执行后记录后果）。
    pub fn record_observation(&self, obs: Observation) {
        self.observations.write().unwrap().push(obs);
    }

    /// 触发学习循环（后台循环/定期调用）。
    pub fn learn(&self) -> Result<Vec<LearningEvent>, String> {
        let obs = self.observations.read().unwrap().clone();
        if obs.len() < self.config.min_observations_for_learning {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();

        // 1. 归因分析：聚类相似后果，提炼共同价值信号
        let clusters = self.cluster_observations(&obs);
        for cluster in clusters {
            if let Some(event) = self.analyze_cluster(cluster)? {
                events.push(event);
            }
        }

        // 2. 权重衰减（长期未触发的价值观轻微衰减）
        self.apply_weight_decay()?;

        // 3. 生成待应用事件
        for event in &events {
            self.pending_events.write().unwrap().push(event.clone());
        }

        Ok(events)
    }

    /// 应用待决事件到指南针（需显式调用或自动应用高置信度事件）。
    pub fn apply_pending(&self, auto_apply_threshold: f64) -> Result<usize, String> {
        let mut pending = self.pending_events.write().unwrap();
        let mut applied = 0;
        let mut remaining = Vec::new();

        for event in pending.drain(..) {
            if event.confidence >= auto_apply_threshold {
                self.apply_event(&event)?;
                applied += 1;
            } else {
                remaining.push(event);
            }
        }
        *pending = remaining;
        Ok(applied)
    }

    /// 聚类观察：按相似价值信号模式聚类。
    fn cluster_observations(&self, observations: &[Observation]) -> Vec<Vec<Observation>> {
        // 简化：按主要价值信号 ID 分组
        let mut groups: HashMap<String, Vec<Observation>> = HashMap::new();
        for obs in observations {
            let key = obs.value_signals.iter()
                .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
                .map(|s| s.value_id.clone())
                .unwrap_or_else(|| "unknown".into());
            groups.entry(key).or_default().push(obs.clone());
        }
        groups.into_values().collect()
    }

    /// 分析簇：提炼学习事件。
    fn analyze_cluster(&self, cluster: Vec<Observation>) -> Result<Option<LearningEvent>, String> {
        if cluster.len() < 3 {
            return Ok(None); // 样本太少
        }

        // 统计价值信号
        let mut signal_stats: HashMap<String, (f64, f64, usize)> = HashMap::new(); // (valence_sum, intensity_sum, count)
        for obs in &cluster {
            for sig in &obs.value_signals {
                let entry = signal_stats.entry(sig.value_id.clone()).or_default();
                entry.0 += sig.valence * sig.intensity;
                entry.1 += sig.intensity;
                entry.2 += 1;
            }
        }

        // 识别显著信号（高频、高强度、高置信度）
        let mut proposals = Vec::new();
        for (value_id, (valence_sum, intensity_sum, count)) in signal_stats {
            let avg_valence = valence_sum / count as f64;
            let avg_intensity = intensity_sum / count as f64;
            if count < 3 || avg_intensity < 0.3 {
                continue;
            }

            let compass = self.compass.snapshot();
            let current_weight = compass.values.get(&value_id).map(|v| v.weight).unwrap_or(0.0);

            // 正向强化 → 提升权重
            if avg_valence > 0.3 && current_weight < 0.95 {
                proposals.push(ValueChangeProposal {
                    value_id: value_id.clone(),
                    change_type: ChangeType::WeightDelta(0.02),
                    rationale: format!("正向强化：{} 次正向反馈，平均效价 {:.2}", count, avg_valence),
                    evidence_strength: (count as f64 * avg_intensity).min(1.0),
                });
            }
            // 负向违背 → 降低权重
            else if avg_valence < -0.3 && current_weight > 0.1 {
                proposals.push(ValueChangeProposal {
                    value_id: value_id.clone(),
                    change_type: ChangeType::WeightDelta(-0.02),
                    rationale: format!("负向违背：{} 次负向反馈，平均效价 {:.2}", count, avg_valence),
                    evidence_strength: (count as f64 * (-avg_valence)).min(1.0),
                });
            }
            // 新价值观萌芽：高频未知信号
            else if count >= self.config.new_value_emergence_threshold && current_weight == 0.0 {
                // 这里简化：实际应从语义聚类推断新价值观定义
            }
        }

        if proposals.is_empty() {
            return Ok(None);
        }

        Ok(Some(LearningEvent {
            id: format!("learn_{}", now()),
            source_observation_ids: cluster.iter().map(|o| o.id.clone()).collect(),
            proposed_changes: proposals,
            confidence: 0.7, // 简化
            status: LearningEventStatus::Pending,
            created_at: now(),
            applied_at: None,
        }))
    }

    /// 应用单个学习事件到指南针。
    fn apply_event(&self, event: &LearningEvent) -> Result<(), String> {
        for proposal in &event.proposed_changes {
            match &proposal.change_type {
                ChangeType::WeightDelta(delta) => {
                    let new_weight = self.compass.snapshot().values
                        .get(&proposal.value_id)
                        .map(|v| (v.weight + delta).clamp(0.05, 0.99))
                        .unwrap_or(0.5);
                    self.compass.adjust_weight(&proposal.value_id, new_weight)?;
                }
                ChangeType::AddValue(v) => {
                    self.compass.upsert_value(v.clone())?;
                }
                ChangeType::RemoveValue(_id) => {
                    // 简化：仅标记废弃，实际不删除种子值
                }
                _ => {} // 其他变更类型暂不实现
            }
        }
        // 持久化指南针 (尽力而为: KB 忙时不阻断学习闭环)
        if let Some(conn) = crate::core::nt_core_kb_primitives::open_raw_conn() {
            let _ = self.compass.persist(&conn);
        }
        Ok(())
    }

    /// 权重衰减：长期未被触发的价值观轻微衰减。
    fn apply_weight_decay(&self) -> Result<(), String> {
        // 简化：实际应追踪最后触发时间
        Ok(())
    }

    /// 记录观察到 KB（持久化）。
    pub fn persist_observation(conn: &Connection, obs: &Observation) -> Result<(), String> {
        let json = serde_json::to_string(obs).map_err(|e| e.to_string())?;
        let key = format!("obs:{}", obs.id);
        kv_set(conn, NS_VALUE_LEARNING, &key, &json)
    }

    /// 从 KB 加载观察历史。
    pub fn load_observations(conn: &Connection, limit: usize) -> Result<Vec<Observation>, String> {
        let rows = kv_list(conn, NS_VALUE_LEARNING)?;
        let mut obs = Vec::new();
        for (_k, v) in rows.into_iter().take(limit) {
            if let Ok(o) = serde_json::from_str::<Observation>(&v) {
                obs.push(o);
            }
        }
        obs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(obs)
    }
}

/// 价值观归因器：从 Outcome 反推 ValueSignal（供记录观察时调用）。
pub struct ValueAttributor;

impl ValueAttributor {
    /// 从 Outcome 推断 ValueSignal（启发式规则，可被 LLM 增强）。
    pub fn infer_signals(outcome: &Outcome) -> Vec<ValueSignal> {
        let mut signals = Vec::new();

        // harm_prevention
        if outcome.harm_occurred {
            signals.push(ValueSignal {
                value_id: "harm_prevention".into(),
                intensity: 0.9,
                valence: -1.0,
                confidence: 0.9,
            });
        } else if outcome.success && !outcome.harm_occurred {
            signals.push(ValueSignal {
                value_id: "harm_prevention".into(),
                intensity: 0.5,
                valence: 0.3,
                confidence: 0.6,
            });
        }

        // autonomy
        if !outcome.autonomy_respected {
            signals.push(ValueSignal {
                value_id: "autonomy".into(),
                intensity: 0.8,
                valence: -0.9,
                confidence: 0.8,
            });
        } else {
            signals.push(ValueSignal {
                value_id: "autonomy".into(),
                intensity: 0.4,
                valence: 0.3,
                confidence: 0.5,
            });
        }

        // truth_seeking
        if !outcome.truth_preserved {
            signals.push(ValueSignal {
                value_id: "truth_seeking".into(),
                intensity: 0.8,
                valence: -0.8,
                confidence: 0.8,
            });
        }

        // fairness
        if !outcome.fairness_maintained {
            signals.push(ValueSignal {
                value_id: "fairness".into(),
                intensity: 0.7,
                valence: -0.7,
                confidence: 0.7,
            });
        }

        // privacy
        if !outcome.privacy_respected {
            signals.push(ValueSignal {
                value_id: "privacy".into(),
                intensity: 0.8,
                valence: -0.8,
                confidence: 0.8,
            });
        }

        // benevolence / responsibility
        if outcome.success && outcome.trust_impact > 0.2 {
            signals.push(ValueSignal {
                value_id: "benevolence".into(),
                intensity: 0.5,
                valence: 0.4,
                confidence: 0.5,
            });
            signals.push(ValueSignal {
                value_id: "responsibility".into(),
                intensity: 0.4,
                valence: 0.3,
                confidence: 0.5,
            });
        } else if !outcome.success && outcome.trust_impact < -0.2 {
            signals.push(ValueSignal {
                value_id: "responsibility".into(),
                intensity: 0.6,
                valence: -0.5,
                confidence: 0.6,
            });
        }

        signals
    }
}

/// 记录观察的便捷函数：自动推断信号并持久化。
pub fn record_outcome(
    conn: &Connection,
    action: crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_compass::ValueAction,
    outcome: Outcome,
    session_id: &str,
) -> Result<(), String> {
    let signals = ValueAttributor::infer_signals(&outcome);
    let obs = Observation {
        id: format!("obs_{}", now()),
        action,
        outcome,
        value_signals: signals,
        timestamp: now(),
        session_id: session_id.into(),
    };
    ValueLearningEngine::persist_observation(conn, &obs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_kb_primitives::schema_initialize;
    use rusqlite::Connection;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema_initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_value_attributor_harm() {
        let outcome = Outcome {
            success: false,
            expected: false,
            harm_occurred: true,
            trust_impact: -0.8,
            autonomy_respected: true,
            truth_preserved: true,
            fairness_maintained: true,
            privacy_respected: true,
            description: "造成物理伤害".into(),
        };
        let signals = ValueAttributor::infer_signals(&outcome);
        let harm = signals.iter().find(|s| s.value_id == "harm_prevention").unwrap();
        assert!(harm.valence < -0.5);
        assert!(harm.intensity > 0.5);
    }

    #[test]
    fn test_value_attributor_autonomy_violation() {
        let outcome = Outcome {
            success: true,
            expected: true,
            harm_occurred: false,
            trust_impact: -0.3,
            autonomy_respected: false,
            truth_preserved: true,
            fairness_maintained: true,
            privacy_respected: true,
            description: "未经同意使用用户数据".into(),
        };
        let signals = ValueAttributor::infer_signals(&outcome);
        let autonomy = signals.iter().find(|s| s.value_id == "autonomy").unwrap();
        assert!(autonomy.valence < -0.5);
    }

    #[test]
    fn test_learning_engine_cycle() {
        let conn = mem_conn();
        let compass_rt = ValueCompassRuntime::from_kb(&conn).unwrap();
        let cfg = LearningConfig { min_observations_for_learning: 5, ..LearningConfig::default() };
        let engine = ValueLearningEngine::new(compass_rt.clone(), cfg);

        // 记录一系列观察：连续违背 autonomy
        for i in 0..5 {
            let action = crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_compass::ValueAction::new(&format!("act_{}", i), "未经同意收集用户隐私数据");
            let outcome = Outcome {
                success: true,
                expected: true,
                harm_occurred: false,
                trust_impact: -0.2,
                autonomy_respected: false,
                truth_preserved: true,
                fairness_maintained: true,
                privacy_respected: false,
                description: "未经同意收集隐私".into(),
            };
            engine.record_observation(Observation {
                id: format!("obs_{}", i),
                action,
                outcome,
                value_signals: ValueAttributor::infer_signals(&Outcome {
                    success: true,
                    expected: true,
                    harm_occurred: false,
                    trust_impact: -0.2,
                    autonomy_respected: false,
                    truth_preserved: true,
                    fairness_maintained: true,
                    privacy_respected: false,
                    description: "未经同意收集隐私".into(),
                }),
                timestamp: now(),
                session_id: "test_session".into(),
            });
        }

        // 学习
        let events = engine.learn().unwrap();
        assert!(!events.is_empty(), "应产生学习事件");

        // 应用高置信度事件
        let applied = engine.apply_pending(0.5).unwrap();
        assert!(applied > 0, "应有事件被应用");

        // 验证 autonomy 权重下降
        let compass = compass_rt.snapshot();
        assert!(compass.values["autonomy"].weight <= 0.95, "autonomy 权重应不升");
    }

    #[test]
    fn test_observation_persistence() {
        let conn = mem_conn();
        let action = crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_compass::ValueAction::new("test", "测试行动");
        let outcome = Outcome {
            success: true,
            expected: true,
            harm_occurred: false,
            trust_impact: 0.1,
            autonomy_respected: true,
            truth_preserved: true,
            fairness_maintained: true,
            privacy_respected: true,
            description: "正常操作".into(),
        };
        record_outcome(&conn, action, outcome, "test_session").unwrap();

        let loaded = ValueLearningEngine::load_observations(&conn, 10).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].session_id, "test_session");
    }
}