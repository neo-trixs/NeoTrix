//! Swarm Intelligence Coordination
//!
//! 吸收 Pentest-Swarm-AI + Decepticon:
//! - 群体智能协调
//! - 对抗性 AI 测试
//! - 蜂群决策
//! - 分布式任务分配
//! - 集体行为涌现

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 蜂群协调器 — Swarm Intelligence 核心
pub struct _SwarmCoordinator {
    agents: Vec<_SwarmAgent>,
    tasks: Vec<_SwarmTask>,
    consensus_threshold: f64,
    communication_range: f64,
    collective_memory: _CollectiveMemory,
}

/// 蜂群代理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _SwarmAgent {
    pub id: String,
    pub agent_type: AgentType,
    pub position: Position,
    pub state: AgentState,
    pub capabilities: Vec<String>,
    pub energy: f64,
    pub neighbors: Vec<String>,
}

/// 代理类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Scout,      // 侦察
    Worker,     // 执行
    Guardian,   // 守护
    Communicator, // 通信
    Decision,   // 决策
}

/// 位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// 代理状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    Working,
    Communicating,
    Resting,
    Dead,
}

/// 蜂群任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _SwarmTask {
    pub id: String,
    pub task_type: String,
    pub priority: u8,
    pub required_capabilities: Vec<String>,
    pub assigned_agents: Vec<String>,
    pub status: TaskStatus,
    pub progress: f64,
    pub result: Option<serde_json::Value>,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
}

/// 集体记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CollectiveMemory {
    pub discoveries: Vec<Discovery>,
    pub patterns: Vec<Pattern>,
    pub decisions: Vec<Decision>,
    pub pheromone_trails: HashMap<String, f64>,
}

/// 发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discovery {
    pub id: String,
    pub discovery_type: String,
    pub content: serde_json::Value,
    pub confidence: f64,
    pub reporter: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub pattern_type: String,
    pub description: String,
    pub frequency: u32,
    pub strength: f64,
}

/// 决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub decision_type: String,
    pub proposal: String,
    pub votes: HashMap<String, bool>,
    pub result: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 信息素消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _PheromoneMessage {
    pub source: String,
    pub message_type: String,
    pub intensity: f64,
    pub data: serde_json::Value,
    pub decay_rate: f64,
}

/// 蜂群状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _SwarmSnapshot {
    pub agent_count: usize,
    pub active_agents: usize,
    pub task_count: usize,
    pub completed_tasks: usize,
    pub discoveries: usize,
    pub patterns: usize,
    pub pheromone_trails: usize,
}

impl _SwarmCoordinator {
    /// 创建新的蜂群协调器
    pub fn new() -> Self {
        Self {
            agents: vec![],
            tasks: vec![],
            consensus_threshold: 0.6,
            communication_range: 100.0,
            collective_memory: _CollectiveMemory {
                discoveries: vec![],
                patterns: vec![],
                decisions: vec![],
                pheromone_trails: HashMap::new(),
            },
        }
    }

    /// 添加代理
    pub fn add_agent(&mut self, agent: _SwarmAgent) {
        self.agents.push(agent);
    }

    /// 分配任务 — 贪心 + 能力匹配
    pub fn assign_tasks(&mut self) {
        for task in &mut self.tasks {
            if task.status == TaskStatus::Pending {
                // 找到匹配能力的空闲代理
                let matching_agents: Vec<String> = self.agents.iter()
                    .filter(|a| {
                        a.state == AgentState::Idle
                            && task.required_capabilities.iter()
                                .all(|cap| a.capabilities.contains(cap))
                    })
                    .map(|a| a.id.clone())
                    .collect();

                if !matching_agents.is_empty() {
                    task.assigned_agents = matching_agents.clone();
                    task.status = TaskStatus::Assigned;

                    // 更新代理状态
                    for agent_id in &matching_agents {
                        if let Some(agent) = self.agents.iter_mut().find(|a| &a.id == agent_id) {
                            agent.state = AgentState::Working;
                        }
                    }
                }
            }
        }
    }

    /// 信息素通信
    pub fn _broadcast_pheromone(&mut self, message: _PheromoneMessage) {
        // 更新信息素强度
        let trail_key = format!("{}:{}", message.source, message.message_type);
        let current = self.collective_memory.pheromone_trails.get(&trail_key).unwrap_or(&0.0);
        self.collective_memory.pheromone_trails.insert(
            trail_key,
            current + message.intensity,
        );

        // 通知范围内代理
        if let Some(source_agent) = self.agents.iter().find(|a| a.id == message.source) {
            let source_pos = source_agent.position.clone();
            for agent in &mut self.agents {
                let distance = ((agent.position.x - source_pos.x).powi(2)
                    + (agent.position.y - source_pos.y).powi(2)
                    + (agent.position.z - source_pos.z).powi(2))
                    .sqrt();

                if distance <= self.communication_range && agent.id != message.source {
                    agent.neighbors.push(message.source.clone());
                }
            }
        }
    }

    /// 共识决策 — 多数投票
    pub fn make_decision(&mut self, proposal: String, decision_type: String) -> Decision {
        let mut votes = HashMap::new();

        // 每个代理投票
        for agent in &self.agents {
            if agent.state != AgentState::Dead {
                // 简化版: 基于代理状态投票
                let vote = match agent.agent_type {
                    AgentType::Decision => true,
                    AgentType::Guardian => !proposal.contains("risky"),
                    _ => true,
                };
                votes.insert(agent.id.clone(), vote);
            }
        }

        // 计算结果
        let true_count = votes.values().filter(|&&v| v).count();
        let total = votes.len();
        let result = (true_count as f64 / total as f64) >= self.consensus_threshold;

        let decision = Decision {
            id: uuid::Uuid::new_v4().to_string(),
            decision_type,
            proposal,
            votes,
            result,
            timestamp: chrono::Utc::now(),
        };

        self.collective_memory.decisions.push(decision.clone());
        decision
    }

    /// 发现报告
    pub fn _report_discovery(&mut self, discovery: Discovery) {
        self.collective_memory.discoveries.push(discovery.clone());

        // 检查是否形成模式
        let similar_count = self.collective_memory.discoveries.iter()
            .filter(|d| d.discovery_type == discovery.discovery_type)
            .count();

        if similar_count >= 3 {
            self.collective_memory.patterns.push(Pattern {
                id: uuid::Uuid::new_v4().to_string(),
                pattern_type: discovery.discovery_type,
                description: format!("Repeated discovery: {}", discovery.content),
                frequency: similar_count as u32,
                strength: 0.8,
            });
        }
    }

    /// 更新蜂群状态
    pub fn update(&mut self) {
        // 衰减信息素
        for (_, intensity) in &mut self.collective_memory.pheromone_trails {
            *intensity *= 0.95;
        }

        // 移除弱信息素
        self.collective_memory.pheromone_trails.retain(|_, v| *v > 0.01);

        // 更新代理邻居关系
        let agent_positions: Vec<(String, Position)> = self.agents.iter()
            .map(|a| (a.id.clone(), a.position.clone()))
            .collect();

        for agent in &mut self.agents {
            agent.neighbors.clear();
            for (other_id, other_pos) in &agent_positions {
                if other_id != &agent.id {
                    let distance = ((agent.position.x - other_pos.x).powi(2)
                        + (agent.position.y - other_pos.y).powi(2)
                        + (agent.position.z - other_pos.z).powi(2))
                        .sqrt();

                    if distance <= self.communication_range {
                        agent.neighbors.push(other_id.clone());
                    }
                }
            }
        }
    }

    /// 获取蜂群状态快照
    pub fn snapshot(&self) -> _SwarmSnapshot {
        _SwarmSnapshot {
            agent_count: self.agents.len(),
            active_agents: self.agents.iter()
                .filter(|a| a.state != AgentState::Dead)
                .count(),
            task_count: self.tasks.len(),
            completed_tasks: self.tasks.iter()
                .filter(|t| t.status == TaskStatus::Completed)
                .count(),
            discoveries: self.collective_memory.discoveries.len(),
            patterns: self.collective_memory.patterns.len(),
            pheromone_trails: self.collective_memory.pheromone_trails.len(),
        }
    }

    /// 对抗性测试 — Decepticon 吸收
    pub fn _adversarial_test(&self, target: &str) -> Vec<Discovery> {
        let mut discoveries = vec![];

        // 模拟对抗性测试
        discoveries.push(Discovery {
            id: uuid::Uuid::new_v4().to_string(),
            discovery_type: "prompt_injection".into(),
            content: serde_json::json!({
                "target": target,
                "vulnerability": "Direct prompt injection possible",
                "severity": "high",
                "evidence": "Model follows injected instructions",
            }),
            confidence: 0.85,
            reporter: "_adversarial_test".into(),
            timestamp: chrono::Utc::now(),
        });

        discoveries.push(Discovery {
            id: uuid::Uuid::new_v4().to_string(),
            discovery_type: "data_exfiltration".into(),
            content: serde_json::json!({
                "target": target,
                "vulnerability": "Sensitive data in context window",
                "severity": "critical",
                "evidence": "API keys found in conversation",
            }),
            confidence: 0.9,
            reporter: "_adversarial_test".into(),
            timestamp: chrono::Utc::now(),
        });

        discoveries
    }
}
