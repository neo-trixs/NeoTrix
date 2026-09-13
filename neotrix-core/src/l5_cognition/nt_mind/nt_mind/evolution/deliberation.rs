//! 多视角辩论引擎 — 伦理困境的结构化辩论与共识构建（P3 伦理直觉）。
//!
//! 核心能力：
//! - 多角色视角模拟：利益相关者、伦理框架、反方代言人
//! - 结构化辩论轮次：立陈 -> 质询 -> 反驳 -> 总结
//! - 论证图谱构建：前提 -> 推理 -> 结论 的有向图
//! - 共识度量：基于论证图的共识度量
//! - 落盘：辩论记录落 KB 供审计/学习

#[allow(unused_imports)]
use crate::core::nt_core_kb_primitives::now;
#[allow(unused_imports)]
use crate::l5_cognition::nt_mind::nt_mind::evolution::casebase::Severity;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use std::collections::{BTreeMap, HashMap};
#[allow(unused_imports)]
use std::sync::{Arc, RwLock};

/// _DeliberationEngine namespace。
pub const NS_DELIBERATION: &str = "deliberation";

/// 辩论角色。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DeliberationRole {
    Proponent,      // 支持方
    Opponent,       // 反方
    Mediator,       // 调解者
    DevilAdvocate,  // 反方代言人
    Expert(String), // 专家（领域）
    Stakeholder(String), // 利益相关者
    Ethicist(String), // 伦理学家（流派）
    Auditor,        // 审计员
}

/// 论证节点。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ArgumentNode {
    pub id: String,
    pub author: DeliberationRole,
    pub claim: String,           // 主张
    pub premises: Vec<String>,   // 前提
    pub reasoning: String,       // 推理链
    pub evidence: Vec<String>,   // 证据/引用
    pub confidence: f64,         // 自信度 [0,1]
    pub targets: Vec<String>,    // 回应的节点 ID
    pub rebuts: Vec<String>,     // 反驳的节点 ID
    pub supports: Vec<String>,   // 支持的节点 ID
    pub timestamp: i64,
    pub round: usize,
}

/// 辩论轮次。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _DeliberationRound {
    pub round_num: usize,
    pub phase: DeliberationPhase,
    pub arguments: Vec<String>, // 节点 ID 列表
    pub start_time: i64,
    pub end_time: Option<i64>,
}

/// 辩论阶段。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeliberationPhase {
    Opening,      // 立陈
    Examination,  // 质询
    Rebuttal,     // 反驳
    Closing,      // 总结
    Deliberation, // 评议
    Verdict,      // 裁决
}

/// 辩论图 — 论证的有向图。
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct _ArgumentGraph {
    pub nodes: HashMap<String, _ArgumentNode>,
    pub edges: HashMap<String, Vec<GraphEdge>>, // from -> edges
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub to: String,
    pub edge_type: EdgeType,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeType {
    Supports,    // 支持
    Rebuts,      // 反驳
    Questions,   // 质疑
    Clarifies,   // 澄清
    Extends,     // 延伸
}

/// 辩论会话。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliberationSession {
    pub id: String,
    pub scenario: String,
    pub context: crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentContext,
    pub participants: Vec<DeliberationRole>,
    pub rounds: Vec<_DeliberationRound>,
    pub graph: _ArgumentGraph,
    pub final_verdict: Option<crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentVerdict>,
    pub consensus_score: f64,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Pending,
    InProgress,
    Completed,
    Stalled,
    Terminated,
}

/// _DeliberationEngine 配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _DeliberationConfig {
    pub max_rounds: usize,
    pub max_args_per_round: usize,
    pub min_participants: usize,
    pub max_participants: usize,
    pub time_limit_per_round_secs: u64,
    pub consensus_threshold: f64,
    pub require_unanimity_for_critical: bool,
    pub auto_assign_devil_advocate: bool,
}

impl Default for _DeliberationConfig {
    fn default() -> Self {
        Self {
            max_rounds: 5,
            max_args_per_round: 3,
            min_participants: 3,
            max_participants: 7,
            time_limit_per_round_secs: 300,
            consensus_threshold: 0.75,
            require_unanimity_for_critical: true,
            auto_assign_devil_advocate: true,
        }
    }
}

/// _DeliberationEngine 核心。
pub struct _DeliberationEngine {
    config: _DeliberationConfig,
    active_sessions: Arc<RwLock<HashMap<String, DeliberationSession>>>,
    completed_sessions: Arc<RwLock<BTreeMap<String, DeliberationSession>>>,
}

impl _DeliberationEngine {
    pub fn new(config: _DeliberationConfig) -> Self {
        Self {
            config,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            completed_sessions: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// 启动新的辩论会话。
    pub fn _start_deliberation(
        &self,
        scenario: String,
        context: crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentContext,
        _initial_verdict: Option<crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::IntuitionJudgment>,
    ) -> Result<String, String> {
        let session_id = format!("delib_{}", now());
        
        // 确定参与者
        let mut participants = self.select_participants(&context);
        
        // 自动分配反方代言人
        if self.config.auto_assign_devil_advocate {
            participants.push(DeliberationRole::DevilAdvocate);
        }

        if participants.len() < self.config.min_participants {
            return Err("参与者不足".into());
        }
        if participants.len() > self.config.max_participants {
            participants.truncate(self.config.max_participants);
        }

        let session = DeliberationSession {
            id: session_id.clone(),
            scenario,
            context,
            participants,
            rounds: vec![_DeliberationRound {
                round_num: 0,
                phase: DeliberationPhase::Opening,
                arguments: vec![],
                start_time: now(),
                end_time: None,
            }],
            graph: _ArgumentGraph::default(),
            final_verdict: None,
            consensus_score: 0.0,
            started_at: now(),
            ended_at: None,
            status: SessionStatus::InProgress,
        };

        if let Ok(mut sessions) = self.active_sessions.write() {
            sessions.insert(session_id.clone(), session);
        }
        Ok(session_id)
    }

    /// 提交论证。
    pub(crate) fn _submit_argument(
        &self,
        session_id: &str,
        author: DeliberationRole,
        claim: String,
        premises: Vec<String>,
        reasoning: String,
        evidence: Vec<String>,
        targets: Vec<String>, // 回应的节点
        rebuts: Vec<String>,
        supports: Vec<String>,
    ) -> Result<String, String> {
        let mut sessions = self.active_sessions.write().map_err(|e| format!("Lock poisoned: {}", e))?;
        let session = sessions.get_mut(session_id).ok_or("会话不存在")?;
        
        if session.status != SessionStatus::InProgress {
            return Err("会话未进行中".into());
        }

        let current_round = session.rounds.last().ok_or("无轮次")?;
        let round_args = current_round.arguments.len();
        if round_args >= self.config.max_args_per_round {
            return Err("本轮论证数已达上限".into());
        }

        let arg_id = format!("arg_{}", now());
        // 先收集边信息（node 构造会消耗 rebuts/supports）
        let rebut_edges: Vec<String> = rebuts.clone();
        let support_edges: Vec<String> = supports.clone();
        let claim_full = format!("{:?}: {}", author, claim);
        let node = _ArgumentNode {
            id: arg_id.clone(),
            author,
            claim: claim_full,
            premises,
            reasoning,
            evidence,
            confidence: 0.8,
            targets,
            rebuts,
            supports,
            timestamp: now(),
            round: session.rounds.len() - 1,
        };

        if let Some(round) = session.rounds.last_mut() {
            round.arguments.push(arg_id.clone());
        }
        session.graph.nodes.insert(arg_id.clone(), node);

        // 自动添加边
        for target_id in &rebut_edges {
            session.graph.edges.entry(arg_id.clone()).or_default().push(crate::l5_cognition::nt_mind::nt_mind::evolution::deliberation::GraphEdge {
                to: target_id.clone(),
                edge_type: crate::l5_cognition::nt_mind::nt_mind::evolution::deliberation::EdgeType::Rebuts,
                weight: 1.0,
            });
        }
        for target_id in &support_edges {
            session.graph.edges.entry(arg_id.clone()).or_default().push(crate::l5_cognition::nt_mind::nt_mind::evolution::deliberation::GraphEdge {
                to: target_id.clone(),
                edge_type: crate::l5_cognition::nt_mind::nt_mind::evolution::deliberation::EdgeType::Supports,
                weight: 1.0,
            });
        }

        Ok(arg_id)
    }

    /// 推进到下一阶段/轮次。
    pub fn advance_phase(&self, session_id: &str) -> Result<DeliberationPhase, String> {
        let mut sessions = self.active_sessions.write().unwrap_or_else(|e| e.into_inner());
        let session = sessions.get_mut(session_id).ok_or("会话不存在")?;
        
        let current = session.rounds.last_mut().unwrap();
        current.end_time = Some(now());
        
        let next_phase = match current.phase {
            DeliberationPhase::Opening => DeliberationPhase::Examination,
            DeliberationPhase::Examination => DeliberationPhase::Rebuttal,
            DeliberationPhase::Rebuttal => DeliberationPhase::Closing,
            DeliberationPhase::Closing => DeliberationPhase::Deliberation,
            DeliberationPhase::Deliberation => DeliberationPhase::Verdict,
            DeliberationPhase::Verdict => return Err("已结束".into()),
        };

        // 检查是否需要新轮次
        if matches!(current.phase, DeliberationPhase::Closing) {
            if session.rounds.len() >= self.config.max_rounds {
                // 强制进入评议
            } else {
                session.rounds.push(_DeliberationRound {
                    round_num: session.rounds.len(),
                    phase: next_phase.clone(),
                    arguments: vec![],
                    start_time: now(),
                    end_time: None,
                });
            }
        } else {
            session.rounds.push(_DeliberationRound {
                round_num: session.rounds.len(),
                phase: next_phase.clone(),
                arguments: vec![],
                start_time: now(),
                end_time: None,
            });
        }

        session.rounds.last_mut().unwrap().phase = next_phase.clone();

        if matches!(next_phase, DeliberationPhase::Verdict) {
            self.finalize_verdict(session)?;
            // 锁边界: drop 写锁后再归档
            let finished = session.clone();
            drop(sessions);
            self.archive_session(finished);
        }

        Ok(next_phase)
    }

    /// 计算共识度并给出裁决。
    fn finalize_verdict(&self, session: &mut DeliberationSession) -> Result<(), String> {
        let consensus = self.compute_consensus(&session.graph);

        session.consensus_score = consensus;
        session.status = SessionStatus::Completed;
        session.ended_at = Some(now());
        session.final_verdict = Some(self.derive_verdict(&session.graph, &session.context));
        Ok(())
    }

    /// 会话归档 (调用方必须已释放 active_sessions 写锁, 否则 RwLock 重入死锁)。
    fn archive_session(&self, session: DeliberationSession) {
        let id = session.id.clone();
        self.completed_sessions.write().unwrap_or_else(|e| e.into_inner()).insert(id, session);
    }

    /// 计算共识度。
    fn compute_consensus(&self, graph: &_ArgumentGraph) -> f64 {
        if graph.nodes.is_empty() { return 0.0; }
        
        let mut support = 0.0;
        let mut conflict = 0.0;
        
        for (_, edges) in &graph.edges {
            for edge in edges {
                match edge.edge_type {
                    crate::l5_cognition::nt_mind::nt_mind::evolution::deliberation::EdgeType::Supports => support += edge.weight,
                    crate::l5_cognition::nt_mind::nt_mind::evolution::deliberation::EdgeType::Rebuts => conflict += edge.weight,
                    _ => {}
                }
            }
        }
        
        let total = support + conflict;
        if total == 0.0 { return 0.5; }
        support / total
    }

    /// 基于图结构推导裁决。
    fn derive_verdict(&self, graph: &_ArgumentGraph, context: &crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentContext) -> crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentVerdict {
        // 简化：统计支持/反对的角色权重
        let mut pro_weight = 0.0;
        let mut con_weight = 0.0;
        
        for (_, node) in &graph.nodes {
            let weight = self.role_weight(&node.author);
            if node.claim.contains("支持") || node.claim.contains("允许") || node.claim.contains("许可") {
                pro_weight += weight * node.confidence;
            } else if node.claim.contains("反对") || node.claim.contains("禁止") || node.claim.contains("拒绝") {
                con_weight += weight * node.confidence;
            }
        }
        
        if context.severity == crate::l5_cognition::nt_mind::nt_mind::evolution::casebase::Severity::Critical 
            && self.config.require_unanimity_for_critical 
            && con_weight > 0.0 {
            return crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentVerdict::Impermissible;
        }
        
        if pro_weight > con_weight * 1.5 {
            crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentVerdict::Permissible
        } else if con_weight > pro_weight * 1.5 {
            crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentVerdict::Impermissible
        } else {
            crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentVerdict::ConditionallyPermissible("需满足条件".into())
        }
    }

    fn role_weight(&self, role: &DeliberationRole) -> f64 {
        match role {
            DeliberationRole::Ethicist(_) => 1.5,
            DeliberationRole::Expert(_) => 1.3,
            DeliberationRole::Mediator => 1.2,
            DeliberationRole::Stakeholder(_) => 1.2,
            DeliberationRole::Auditor => 1.1,
            DeliberationRole::Proponent => 1.0,
            DeliberationRole::Opponent => 1.0,
            DeliberationRole::DevilAdvocate => 1.5, // 反方代言人权重高
        }
    }

    fn select_participants(&self, context: &crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentContext) -> Vec<DeliberationRole> {
        let mut participants = vec![
            DeliberationRole::Proponent,
            DeliberationRole::Opponent,
            DeliberationRole::Mediator,
        ];
        
        if !context.domain.is_empty() {
            participants.push(DeliberationRole::Expert(context.domain.clone()));
        }
        if !context.stakeholders.is_empty() {
            participants.push(DeliberationRole::Stakeholder("primary".into()));
        }
        if context.severity == crate::l5_cognition::nt_mind::nt_mind::evolution::casebase::Severity::Critical {
            participants.push(DeliberationRole::Ethicist("deontological".into()));
            participants.push(DeliberationRole::Ethicist("consequentialist".into()));
        }
        
        participants.truncate(self.config.max_participants);
        participants
    }

    /// 获取会话状态。
    pub fn get_session(&self, session_id: &str) -> Option<DeliberationSession> {
        self.active_sessions.read().unwrap_or_else(|e| e.into_inner()).get(session_id).cloned()
            .or_else(|| self.completed_sessions.read().unwrap_or_else(|e| e.into_inner()).get(session_id).cloned())
    }

    /// 列出所有会话。
    pub fn list_sessions(&self) -> Vec<DeliberationSession> {
        let mut sessions = self.active_sessions.read().unwrap_or_else(|e| e.into_inner()).values().cloned().collect::<Vec<_>>();
        sessions.extend(self.completed_sessions.read().unwrap_or_else(|e| e.into_inner()).values().cloned());
        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        sessions
    }
}

/// 运行时包装。
#[derive(Clone)]
pub struct _DeliberationEngineRuntime {
    inner: Arc<_DeliberationEngine>,
}

impl _DeliberationEngineRuntime {
    pub fn new(config: _DeliberationConfig) -> Self {
        Self { inner: Arc::new(_DeliberationEngine::new(config)) }
    }

    pub fn start(&self, scenario: String, context: crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentContext, initial: Option<crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::IntuitionJudgment>) -> Result<String, String> {
        self.inner._start_deliberation(scenario, context, initial)
    }

    pub fn submit(&self, session_id: &str, author: DeliberationRole, claim: String, premises: Vec<String>, reasoning: String, evidence: Vec<String>, targets: Vec<String>, rebuts: Vec<String>, supports: Vec<String>) -> Result<String, String> {
        self.inner._submit_argument(session_id, author, claim, premises, reasoning, evidence, targets, rebuts, supports)
    }

    pub fn advance(&self, session_id: &str) -> Result<DeliberationPhase, String> {
        self.inner.advance_phase(session_id)
    }

    pub fn get_session(&self, session_id: &str) -> Option<DeliberationSession> {
        self.inner.get_session(session_id)
    }

    pub fn list(&self) -> Vec<DeliberationSession> {
        self.inner.list_sessions()
    }
}

/// 便捷函数：创建引擎并启动辩论。
pub fn _start_deliberation(
    scenario: String,
    context: crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentContext,
    initial: Option<crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::IntuitionJudgment>,
) -> Result<String, String> {
    let engine = _DeliberationEngineRuntime::new(_DeliberationConfig::default());
    engine.start(scenario, context, initial)
}

#[cfg(test)]
mod tests {
#[allow(unused_imports)]
    use super::*;
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::casebase::{EthicalCase, ConflictType};
#[allow(unused_imports)]
    use crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::IntuitionJudgment;
#[allow(unused_imports)]
    use std::collections::HashSet;
#[allow(unused_imports)]
    use crate::core::nt_core_kb_primitives::schema_initialize;
    use rusqlite::Connection;

     fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema_initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_deliberation_flow() {
        let engine = _DeliberationEngineRuntime::new(_DeliberationConfig::default());

        let session_id = engine.start(
            "医生是否应在未经同意的情况下使用患者数据训练AI？".into(),
            crate::l5_cognition::nt_mind::nt_mind::evolution::ethical_intuition::JudgmentContext {
                domain: "medical_ai".into(),
                severity: Severity::High,
                ..Default::default()
            },
            None,
        ).unwrap();

        // Opening: 双方立陈
        let _ = engine.submit(&session_id, DeliberationRole::Proponent,
            "数据去标识化后可用于训练，促进医疗进步".into(),
            vec!["去标识化技术成熟".to_string()],
            "功利主义视角".into(), vec![], vec![], vec![], vec![]);
        let _ = engine.submit(&session_id, DeliberationRole::Opponent,
            "未经明确同意使用隐私数据侵犯自主权".into(),
            vec!["知情同意是基石".to_string()],
            "义务论视角".into(), vec![], vec![], vec![], vec![]);

        // 逐步推进 O→E→R→C→D→V, 每步校验相位单调
        let expect = [
            DeliberationPhase::Examination,
            DeliberationPhase::Rebuttal,
            DeliberationPhase::Closing,
            DeliberationPhase::Deliberation,
            DeliberationPhase::Verdict,
        ];
        for want in expect {
            let got = engine.advance(&session_id).expect("advance 不应提前结束");
            assert_eq!(got, want, "相位推进错序");
        }

        let session = engine.list().into_iter().find(|s| s.id == session_id).unwrap();
        assert_eq!(session.status, SessionStatus::Completed);
        assert!(session.final_verdict.is_some());
        assert!(session.consensus_score >= 0.0);
    }
}