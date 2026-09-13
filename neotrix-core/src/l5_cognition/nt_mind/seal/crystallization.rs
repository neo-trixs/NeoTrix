//! 技能结晶引擎 — 从模板到正式技能
//!
//! 当技能模板成功 3+ 次时，自动生成可复用的子代理模板，
//! 并附带清晰的 I/O 契约（输入 schema、输出 schema、前置技能）。

use std::collections::HashMap;

/// 结晶状态

#[derive(Clone, Debug)]
pub(crate) enum _CrystallizationStatus {
    /// 模板阶段（成功 < 3 次）
    Template { success_count: u32 },
    /// 候选阶段（成功 3+ 次，待审批）
    Candidate { confidence: f64 },
    /// 已结晶（成为正式技能）
    Crystallized { skill_id: String },
    /// 已拒绝（质量不达标）
    Rejected { reason: String },
}

/// I/O 契约 — 定义技能的输入输出规范
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct _IoContract {
    /// 输入 JSON schema
    pub input_schema: String,
    /// 输出 JSON schema
    pub output_schema: String,
    /// 前置技能列表
    pub prerequisites: Vec<String>,
    /// 预估 token 消耗
    pub estimated_tokens: u32,
    /// 预估延迟（毫秒）
    pub estimated_latency_ms: u32,
}

/// 已结晶技能 — 从模板晋升为正式技能
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct _CrystallizedSkill {
    /// 技能唯一标识
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能描述
    pub description: String,
    /// I/O 契约
    pub contract: _IoContract,
    /// 成功率
    pub success_rate: f64,
    /// 总调用次数
    pub total_invocations: u32,
    /// 结晶时间戳（Unix seconds）
    pub crystallized_at: i64,
    /// 来源模板 ID 列表
    pub source_templates: Vec<String>,
}

/// 结晶统计信息
#[derive(Clone, Debug)]

pub(crate) struct _CrystallizationStats {
    /// 候选模板数量
    pub candidates: usize,
    /// 已结晶技能数量
    pub crystallized: usize,
    /// 已拒绝模板数量
    pub rejected: usize,
    /// 平均成功率
    pub avg_success_rate: f64,
}

/// 技能结晶引擎 — 监控模板成功率，自动结晶为正式技能

#[derive(Debug)]
pub struct CrystallizationEngine {
    /// 待结晶模板 (template_id → success_count)
    candidates: HashMap<String, u32>,
    /// 已结晶技能
    crystallized: Vec<_CrystallizedSkill>,
    /// 被拒绝的模板 (template_id, reason)
    rejected: Vec<(String, String)>,
    /// 结晶阈值（默认 3）
    threshold: u32,
    /// 下一个技能 ID 计数器
    next_skill_id: u32,
    /// 技能进化追踪器 (WikiSkill 模式: 跨会话持久化)
    evolution_tracker: SkillEvolutionTracker,
}

impl CrystallizationEngine {
    /// 创建新的结晶引擎，指定结晶阈值
    pub fn new(threshold: u32) -> Self {
        Self {
            candidates: HashMap::new(),
            crystallized: Vec::new(),
            rejected: Vec::new(),
            threshold,
            next_skill_id: 0,
            evolution_tracker: SkillEvolutionTracker::new(),
        }
    }

    /// 记录模板成功一次
    pub fn record_success(&mut self, template_id: &str) {
        let count = self.candidates.entry(template_id.to_string()).or_insert(0);
        *count += 1;
    }

    /// 获取达到阈值的候选列表
    pub fn candidates(&self) -> Vec<(&str, u32)> {
        self.candidates
            .iter()
            .filter(|(_, count)| **count >= self.threshold)
            .map(|(id, count)| (id.as_str(), *count))
            .collect()
    }

    /// 获取模板当前状态
    pub(crate) fn _status_of(&self, template_id: &str) -> _CrystallizationStatus {
        match self.candidates.get(template_id) {
            Some(&count) if count >= self.threshold => {
                let confidence = (count as f64 / self.threshold as f64).min(1.0);
                _CrystallizationStatus::Candidate { confidence }
            }
            Some(&count) => _CrystallizationStatus::Template {
                success_count: count,
            },
            None => {
                if self
                    .crystallized
                    .iter()
                    .any(|s| s.source_templates.contains(&template_id.to_string()))
                {
                    let skill_id = self
                        .crystallized
                        .iter()
                        .find(|s| s.source_templates.contains(&template_id.to_string()))
                        .unwrap()
                        .id
                        .clone();
                    _CrystallizationStatus::Crystallized { skill_id }
                } else if self.rejected.iter().any(|(id, _)| id == template_id) {
                    let reason = self
                        .rejected
                        .iter()
                        .find(|(id, _)| id == template_id)
                        .unwrap()
                        .1
                        .clone();
                    _CrystallizationStatus::Rejected { reason }
                } else {
                    _CrystallizationStatus::Template { success_count: 0 }
                }
            }
        }
    }

    /// 结晶一个候选为正式技能 (自动记录进化事件)
    pub fn crystallize(
        &mut self,
        template_id: &str,
        name: &str,
        description: &str,
        contract: _IoContract,
    ) -> Option<_CrystallizedSkill> {
        let success_count = match self.candidates.get(template_id) {
            Some(&c) if c >= self.threshold => c,
            _ => return None,
        };

        self.next_skill_id += 1;
        let skill_id = format!("skill_{}", self.next_skill_id);
        let skill = _CrystallizedSkill {
            id: skill_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            contract,
            success_rate: 1.0,
            total_invocations: success_count,
            crystallized_at: chrono_now(),
            source_templates: vec![template_id.to_string()],
        };

        // WikiSkill 模式: 记录进化事件到追踪器
        self.evolution_tracker.record_evolution(
            &skill_id,
            name,
            1.0,
            success_count,
            &format!("crystallized from template {}", template_id),
            "current_session",
        );

        self.crystallized.push(skill.clone());
        self.candidates.remove(template_id);
        Some(skill)
    }

    /// 拒绝一个候选
    pub fn reject(&mut self, template_id: &str, reason: &str) {
        self.rejected
            .push((template_id.to_string(), reason.to_string()));
        self.candidates.remove(template_id);
    }

    /// 获取所有已结晶技能
    pub(crate) fn _crystallized_skills(&self) -> &[_CrystallizedSkill] {
        &self.crystallized
    }

    /// 获取统计信息
    pub fn stats(&self) -> _CrystallizationStats {
        _CrystallizationStats {
            candidates: self.candidates.len(),
            crystallized: self.crystallized.len(),
            rejected: self.rejected.len(),
            avg_success_rate: if self.crystallized.is_empty() {
                0.0
            } else {
                self.crystallized
                    .iter()
                    .map(|s| s.success_rate)
                    .sum::<f64>()
                    / self.crystallized.len() as f64
            },
        }
    }

    /// 生成 SKILL.md 格式模板
    pub(crate) fn _to_skill_md(&self, skill: &_CrystallizedSkill) -> String {
        let prereqs = if skill.contract.prerequisites.is_empty() {
            "None".to_string()
        } else {
            skill.contract.prerequisites.join(", ")
        };

        format!(
            "# {}\n\n## Description\n{}\n\n## Input\n```json\n{}\n```\n\n## Output\n```json\n{}\n```\n\n## Prerequisites\n{}\n\n## Stats\n- Success Rate: {:.1}%\n- Invocations: {}\n- Crystallized: {}\n",
            skill.name,
            skill.description,
            skill.contract.input_schema,
            skill.contract.output_schema,
            prereqs,
            skill.success_rate * 100.0,
            skill.total_invocations,
            skill.crystallized_at,
        )
    }

    /// 合并另一个结晶引擎的结果（用于跨会话持久化后恢复）
    pub fn merge(&mut self, other: &CrystallizationEngine) {
        for (id, count) in &other.candidates {
            let entry = self.candidates.entry(id.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }
        for skill in &other.crystallized {
            if !self.crystallized.iter().any(|s| s.id == skill.id) {
                self.crystallized.push(skill.clone());
            }
        }
        for rejected in &other.rejected {
            if !self.rejected.iter().any(|(id, _)| id == &rejected.0) {
                self.rejected.push(rejected.clone());
            }
        }
    }

    /// WikiSkill 模式: 跨会话持久化 — 序列化为 JSON (存入 KB kv_store)
    pub fn persist_to_json(&self) -> Result<String, serde_json::Error> {
        #[derive(serde::Serialize)]
        struct PersistedState {
            candidates: HashMap<String, u32>,
            crystallized: Vec<_CrystallizedSkill>,
            rejected: Vec<(String, String)>,
            threshold: u32,
            next_skill_id: u32,
            evolution_trends: HashMap<String, SkillEvolutionTrend>,
        }
        let state = PersistedState {
            candidates: self.candidates.clone(),
            crystallized: self.crystallized.clone(),
            rejected: self.rejected.clone(),
            threshold: self.threshold,
            next_skill_id: self.next_skill_id,
            evolution_trends: self.evolution_tracker.trends.clone(),
        };
        serde_json::to_string(&state)
    }

    /// WikiSkill 模式: 从 JSON 恢复 (从 KB kv_store 加载)
    pub fn restore_from_json(json: &str) -> Result<Self, serde_json::Error> {
        #[derive(serde::Deserialize)]
        struct PersistedState {
            candidates: HashMap<String, u32>,
            crystallized: Vec<_CrystallizedSkill>,
            rejected: Vec<(String, String)>,
            threshold: u32,
            next_skill_id: u32,
            evolution_trends: HashMap<String, SkillEvolutionTrend>,
        }
        let state: PersistedState = serde_json::from_str(json)?;
        let mut tracker = SkillEvolutionTracker::new();
        tracker.trends = state.evolution_trends;
        for (id, trend) in &tracker.trends {
            tracker.records.insert(id.clone(), trend.history.clone());
        }
        Ok(Self {
            candidates: state.candidates,
            crystallized: state.crystallized,
            rejected: state.rejected,
            threshold: state.threshold,
            next_skill_id: state.next_skill_id,
            evolution_tracker: tracker,
        })
    }

    /// 获取进化追踪器引用
    pub fn evolution_tracker(&self) -> &SkillEvolutionTracker {
        &self.evolution_tracker
    }

    /// 获取进化追踪器可变引用
    pub fn evolution_tracker_mut(&mut self) -> &mut SkillEvolutionTracker {
        &mut self.evolution_tracker
    }
}

/// 技能进化追踪器 — 跨会话持久化 + 版本历史
///
/// 吸收 WikiSkill (arXiv:2608.27454) 模式:
/// - 区分原始执行经验、累积知识、可执行技能三层
/// - 技能进化路径追踪 (success_rate 趋势、版本快照)
/// - 跨会话持久化到 KB (经验→知识→技能 完整链路)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillEvolutionRecord {
    /// 技能 ID
    pub skill_id: String,
    /// 版本号 (从 1 递增)
    pub version: u32,
    /// 该版本的成功率
    pub success_rate: f64,
    /// 该版本的总调用次数
    pub total_invocations: u32,
    /// 版本变更原因
    pub change_reason: String,
    /// 时间戳 (Unix seconds)
    pub recorded_at: i64,
    /// 来源会话 ID
    pub session_id: String,
}

/// 技能进化趋势
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillEvolutionTrend {
    /// 技能 ID
    pub skill_id: String,
    /// 技能名称
    pub skill_name: String,
    /// 当前版本
    pub current_version: u32,
    /// 历史版本记录
    pub history: Vec<SkillEvolutionRecord>,
    /// 成功率趋势 (最近 N 次)
    pub success_rate_trend: Vec<f64>,
    /// 是否在退化 (最近 3 次成功率持续下降)
    pub is_degrading: bool,
    /// 跨会话复用次数
    pub cross_session_reuse: u32,
}

/// 技能进化追踪器
#[derive(Debug)]
pub struct SkillEvolutionTracker {
    /// 技能进化记录 (skill_id → 版本历史)
    records: HashMap<String, Vec<SkillEvolutionRecord>>,
    /// 技能进化趋势 (skill_id → 趋势)
    trends: HashMap<String, SkillEvolutionTrend>,
}

impl SkillEvolutionTracker {
    /// 创建新的进化追踪器
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            trends: HashMap::new(),
        }
    }

    /// 记录技能进化事件 (每次结晶或成功调用时)
    pub fn record_evolution(
        &mut self,
        skill_id: &str,
        skill_name: &str,
        success_rate: f64,
        total_invocations: u32,
        change_reason: &str,
        session_id: &str,
    ) {
        let now = chrono_now();
        let version = self.records.get(skill_id).map_or(1, |r| r.len() as u32 + 1);

        let record = SkillEvolutionRecord {
            skill_id: skill_id.to_string(),
            version,
            success_rate,
            total_invocations,
            change_reason: change_reason.to_string(),
            recorded_at: now,
            session_id: session_id.to_string(),
        };

        self.records
            .entry(skill_id.to_string())
            .or_default()
            .push(record);

        // 更新趋势
        let trend = self.trends.entry(skill_id.to_string()).or_insert_with(|| {
            SkillEvolutionTrend {
                skill_id: skill_id.to_string(),
                skill_name: skill_name.to_string(),
                current_version: version,
                history: Vec::new(),
                success_rate_trend: Vec::new(),
                is_degrading: false,
                cross_session_reuse: 0,
            }
        });

        trend.current_version = version;
        trend.history = self.records[skill_id].clone();
        trend.success_rate_trend.push(success_rate);

        // 保留最近 10 次趋势
        if trend.success_rate_trend.len() > 10 {
            trend.success_rate_trend.remove(0);
        }

        // 检测退化 (最近 3 次持续下降)
        if trend.success_rate_trend.len() >= 3 {
            let recent: Vec<f64> = trend.success_rate_trend.iter().rev().take(3).cloned().collect();
            trend.is_degrading = recent[0] < recent[1] && recent[1] < recent[2];
        }

        // 跨会话复用计数 (不同 session_id 的记录数 > 1)
        let unique_sessions: std::collections::HashSet<&str> = trend
            .history
            .iter()
            .map(|r| r.session_id.as_str())
            .collect();
        trend.cross_session_reuse = unique_sessions.len() as u32;
    }

    /// 获取技能进化趋势
    pub fn get_trend(&self, skill_id: &str) -> Option<&SkillEvolutionTrend> {
        self.trends.get(skill_id)
    }

    /// 获取所有进化中的技能 (跨会话复用 >= 2)
    pub fn get_cross_session_skills(&self) -> Vec<&SkillEvolutionTrend> {
        self.trends
            .values()
            .filter(|t| t.cross_session_reuse >= 2)
            .collect()
    }

    /// 获取退化中的技能 (需要重新蒸馏)
    pub fn get_degrading_skills(&self) -> Vec<&SkillEvolutionTrend> {
        self.trends.values().filter(|t| t.is_degrading).collect()
    }

    /// 序列化为 JSON (用于 KB 持久化)
    pub fn to_persistable_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.trends)
    }

    /// 从 JSON 恢复 (用于跨会话加载)
    pub fn from_persistable_json(json: &str) -> Result<Self, serde_json::Error> {
        let trends: HashMap<String, SkillEvolutionTrend> = serde_json::from_str(json)?;
        let mut tracker = Self::new();
        tracker.trends = trends;
        // 从趋势恢复记录
        for (id, trend) in &tracker.trends {
            tracker.records.insert(id.clone(), trend.history.clone());
        }
        Ok(tracker)
    }
}

/// 获取当前 Unix 时间戳（秒）
fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_contract() -> _IoContract {
        _IoContract {
            input_schema: r#"{"type":"object","properties":{"text":{"type":"string"}}"#.to_string(),
            output_schema: r#"{"type":"object","properties":{"result":{"type":"string"}}"#
                .to_string(),
            prerequisites: vec!["base_nlp".to_string()],
            estimated_tokens: 500,
            estimated_latency_ms: 200,
        }
    }

    #[test]
    fn test_record_success_and_candidates() {
        let mut engine = CrystallizationEngine::new(3);
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");
        assert!(engine.candidates().is_empty());

        engine.record_success("tmpl_a");
        let cands = engine.candidates();
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0], ("tmpl_a", 3));
    }

    #[test]
    fn test_crystallize() {
        let mut engine = CrystallizationEngine::new(3);
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");

        let skill = engine
            .crystallize("tmpl_a", "Test Skill", "A test skill", test_contract())
            .unwrap();
        assert_eq!(skill.id, "skill_1");
        assert_eq!(skill.name, "Test Skill");
        assert_eq!(skill.total_invocations, 3);
        assert!(engine.candidates().is_empty());
        assert_eq!(engine._crystallized_skills().len(), 1);
    }

    #[test]
    fn test_crystallize_below_threshold() {
        let mut engine = CrystallizationEngine::new(3);
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");

        let result = engine.crystallize("tmpl_a", "Fail", "Should fail", test_contract());
        assert!(result.is_none());
    }

    #[test]
    fn test_reject() {
        let mut engine = CrystallizationEngine::new(3);
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");

        engine.reject("tmpl_a", "质量不达标");
        assert!(engine.candidates().is_empty());
        assert_eq!(engine.rejected.len(), 1);

        let status = engine._status_of("tmpl_a");
        match status {
            _CrystallizationStatus::Rejected { reason } => assert_eq!(reason, "质量不达标"),
            _ => panic!("Expected Rejected"),
        }
    }

    #[test]
    fn test_stats() {
        let mut engine = CrystallizationEngine::new(3);
        engine.record_success("a");
        engine.record_success("a");
        engine.record_success("a");
        engine.crystallize("a", "A", "desc", test_contract());

        let stats = engine.stats();
        assert_eq!(stats.crystallized, 1);
        assert_eq!(stats.avg_success_rate, 1.0);
    }

    #[test]
    fn test_to_skill_md() {
        let engine = CrystallizationEngine::new(3);
        let skill = _CrystallizedSkill {
            id: "skill_1".to_string(),
            name: "My Skill".to_string(),
            description: "Does things".to_string(),
            contract: test_contract(),
            success_rate: 0.95,
            total_invocations: 10,
            crystallized_at: 1700000000,
            source_templates: vec!["tmpl_1".to_string()],
        };
        let md = engine._to_skill_md(&skill);
        assert!(md.contains("# My Skill"));
        assert!(md.contains("95.0%"));
        assert!(md.contains("base_nlp"));
    }
}
