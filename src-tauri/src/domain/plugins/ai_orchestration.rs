//! # AI Orchestration Domain Plugin
//!
//! Integrates latest AI company patterns (2025-2026):
//! - **Thinking Budget Control** (Gemini 2.5) — Configurable reasoning depth
//! - **Background Task Execution** (GPT-5) — Long-running tasks in background
//! - **Memory Extraction** (Claude 4) — Auto-extract and save key facts
//! - **Parallel Tool Execution** (Claude 4) — Execute multiple tools simultaneously
//! - **Real-time Routing** (GPT-5) — Route to cheapest capable model
//! - **Deep Think Mode** (Gemini 2.5) — Enhanced reasoning for complex problems

use crate::domain::app_handle::{get_app_handle, set_app_handle};
use crate::domain::{ActionSpec, DomainError, DomainPlugin, ParamSpec};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// ========== Types ==========

/// Thinking budget levels (inspired by Gemini 2.5)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingBudget {
    /// No thinking, fast response
    Off,
    /// Light thinking, ~10% extra latency
    Light,
    /// Medium thinking, ~30% extra latency
    Medium,
    /// Deep thinking, ~100% extra latency (Gemini Deep Think equivalent)
    Deep,
    /// Adaptive: system decides based on query complexity
    Adaptive,
}

impl ThinkingBudget {
    pub fn max_tokens(&self) -> u32 {
        match self {
            ThinkingBudget::Off => 0,
            ThinkingBudget::Light => 1024,
            ThinkingBudget::Medium => 4096,
            ThinkingBudget::Deep => 32768,
            ThinkingBudget::Adaptive => 8192, // default, adjusted at runtime
        }
    }
}

impl Default for ThinkingBudget {
    fn default() -> Self {
        ThinkingBudget::Adaptive
    }
}

/// Background task status (inspired by GPT-5 background mode)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundTaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Background task info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTask {
    pub id: String,
    pub description: String,
    pub status: BackgroundTaskStatus,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub progress: f32, // 0.0 - 1.0
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Extracted memory fact (inspired by Claude 4 memory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedMemory {
    pub id: String,
    pub fact: String,
    pub category: String, // "preference", "fact", "instruction", "context"
    pub confidence: f32,
    pub source_session: String,
    pub created_at: String,
    pub last_used: Option<String>,
    pub use_count: u32,
}

/// Parallel tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelToolCall {
    pub tool_name: String,
    pub args: serde_json::Value,
    pub call_id: String,
}

/// Parallel tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelToolResult {
    pub call_id: String,
    pub tool_name: String,
    pub result: serde_json::Value,
    pub success: bool,
    pub duration_ms: u64,
}

/// Model routing decision (inspired by GPT-5 real-time router)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub selected_model: String,
    pub reason: String,
    pub estimated_cost: f64,
    pub estimated_latency_ms: u64,
    pub confidence: f32,
    pub alternatives: Vec<String>,
}

/// Query complexity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAnalysis {
    pub score: f32, // 0.0 - 1.0
    pub factors: Vec<String>,
    pub recommended_budget: ThinkingBudget,
    pub recommended_model_tier: String, // "fast", "balanced", "powerful"
}

// ========== Plugin State ==========

#[derive(Default)]
struct AiOrchestrationState {
    background_tasks: HashMap<String, BackgroundTask>,
    memories: Vec<ExtractedMemory>,
    thinking_budget: ThinkingBudget,
    memory_extraction_enabled: bool,
    max_memories: usize,
}

// ========== Plugin ==========

pub struct AiOrchestrationPlugin {
    state: Arc<Mutex<AiOrchestrationState>>,
}

impl AiOrchestrationPlugin {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AiOrchestrationState {
                thinking_budget: ThinkingBudget::Adaptive,
                memory_extraction_enabled: true,
                max_memories: crate::constants::DEFAULT_MAX_MEMORIES,
                ..Default::default()
            })),
        }
    }

    fn analyze_complexity(&self, query: &str) -> ComplexityAnalysis {
        let word_count = query.split_whitespace().count();
        let has_code = query.contains("```") || query.contains("fn ") || query.contains("impl ");
        let has_math =
            query.contains("calculate") || query.contains("proof") || query.contains("equation");
        let is_multi_step =
            query.contains("step") || query.contains("first") || query.contains("then");
        let has_domain_specific = query.contains("architecture")
            || query.contains("algorithm")
            || query.contains("optimize")
            || query.contains("design");

        let mut score: f32 = 0.0;
        let mut factors = Vec::new();

        if word_count > 100 {
            score += 0.2;
            factors.push("long_query".into());
        }
        if has_code {
            score += 0.25;
            factors.push("code_complexity".into());
        }
        if has_math {
            score += 0.3;
            factors.push("mathematical_reasoning".into());
        }
        if is_multi_step {
            score += 0.15;
            factors.push("multi_step".into());
        }
        if has_domain_specific {
            score += 0.2;
            factors.push("domain_expertise".into());
        }

        let recommended_budget = if score < 0.3 {
            ThinkingBudget::Light
        } else if score < 0.6 {
            ThinkingBudget::Medium
        } else {
            ThinkingBudget::Deep
        };

        let recommended_model_tier = if score < 0.3 {
            "fast".into()
        } else if score < 0.6 {
            "balanced".into()
        } else {
            "powerful".into()
        };

        ComplexityAnalysis {
            score: score.min(1.0),
            factors,
            recommended_budget,
            recommended_model_tier,
        }
    }

    fn route_to_model(&self, analysis: &ComplexityAnalysis) -> RoutingDecision {
        let (model, reason, cost, latency) = match analysis.recommended_model_tier.as_str() {
            "fast" => (
                "deepseek-r1:7b".into(),
                "Simple query, use fast local model".into(),
                0.0,
                100,
            ),
            "balanced" => (
                "deepseek-r1:14b".into(),
                "Medium complexity, balanced cost/quality".into(),
                0.001,
                500,
            ),
            "powerful" => (
                "claude-sonnet-4-20250514".into(),
                "Complex reasoning, use most capable model".into(),
                0.01,
                2000,
            ),
            _ => (
                "deepseek-r1:7b".into(),
                "Default to fast model".into(),
                0.0,
                100,
            ),
        };

        let alternatives = match analysis.recommended_model_tier.as_str() {
            "fast" => vec!["deepseek-r1:14b".into(), "gpt-4o-mini".into()],
            "balanced" => vec!["deepseek-r1:7b".into(), "claude-sonnet-4-20250514".into()],
            "powerful" => vec!["gpt-5".into(), "deepseek-r1:14b".into()],
            _ => vec![],
        };

        RoutingDecision {
            selected_model: model,
            reason,
            estimated_cost: cost,
            estimated_latency_ms: latency,
            confidence: 1.0 - analysis.score * 0.3,
            alternatives,
        }
    }

    fn extract_memories(&self, session_id: &str, conversation: &str) -> Vec<ExtractedMemory> {
        let mut memories = Vec::new();
        let lines: Vec<&str> = conversation.lines().collect();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Simple heuristic extraction (in production, use LLM)
            if trimmed.starts_with("I prefer ") || trimmed.starts_with("I like ") {
                memories.push(ExtractedMemory {
                    id: uuid::Uuid::new_v4().to_string(),
                    fact: trimmed.to_string(),
                    category: "preference".into(),
                    confidence: 0.8,
                    source_session: session_id.to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    last_used: None,
                    use_count: 0,
                });
            } else if trimmed.contains("is a ") || trimmed.contains("means ") {
                memories.push(ExtractedMemory {
                    id: uuid::Uuid::new_v4().to_string(),
                    fact: trimmed.to_string(),
                    category: "fact".into(),
                    confidence: 0.7,
                    source_session: session_id.to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    last_used: None,
                    use_count: 0,
                });
            } else if trimmed.starts_with("Always ") || trimmed.starts_with("Never ") {
                memories.push(ExtractedMemory {
                    id: uuid::Uuid::new_v4().to_string(),
                    fact: trimmed.to_string(),
                    category: "instruction".into(),
                    confidence: 0.9,
                    source_session: session_id.to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    last_used: None,
                    use_count: 0,
                });
            }
        }

        memories
    }
}

#[async_trait]
impl DomainPlugin for AiOrchestrationPlugin {
    fn name(&self) -> &str {
        "ai_orchestration"
    }

    fn description(&self) -> &str {
        "Advanced AI orchestration with thinking budget, background tasks, memory extraction, and parallel tools"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            // Thinking Budget
            ActionSpec {
                name: "set_thinking_budget".into(),
                description: "Set thinking budget level (Off/Light/Medium/Deep/Adaptive)".into(),
                params: vec![ParamSpec {
                    name: "budget".into(),
                    r#type: "String".into(),
                    description: "Thinking budget level".into(),
                    optional: false,
                }],
                returns: "ThinkingBudget".into(),
            },
            ActionSpec {
                name: "get_thinking_budget".into(),
                description: "Get current thinking budget level".into(),
                params: vec![],
                returns: "ThinkingBudget".into(),
            },
            // Background Tasks
            ActionSpec {
                name: "submit_background_task".into(),
                description: "Submit a long-running task to background execution (GPT-5 style)"
                    .into(),
                params: vec![
                    ParamSpec {
                        name: "description".into(),
                        r#type: "String".into(),
                        description: "Task description".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "args".into(),
                        r#type: "Value".into(),
                        description: "Task arguments".into(),
                        optional: true,
                    },
                ],
                returns: "BackgroundTask".into(),
            },
            ActionSpec {
                name: "get_background_task".into(),
                description: "Get background task status and result".into(),
                params: vec![ParamSpec {
                    name: "task_id".into(),
                    r#type: "String".into(),
                    description: "Task ID".into(),
                    optional: false,
                }],
                returns: "BackgroundTask".into(),
            },
            ActionSpec {
                name: "list_background_tasks".into(),
                description: "List all background tasks with optional status filter".into(),
                params: vec![ParamSpec {
                    name: "status".into(),
                    r#type: "String".into(),
                    description: "Filter by status (queued/running/completed/failed/cancelled)"
                        .into(),
                    optional: true,
                }],
                returns: "[BackgroundTask]".into(),
            },
            ActionSpec {
                name: "cancel_background_task".into(),
                description: "Cancel a running background task".into(),
                params: vec![ParamSpec {
                    name: "task_id".into(),
                    r#type: "String".into(),
                    description: "Task ID".into(),
                    optional: false,
                }],
                returns: "Bool".into(),
            },
            // Memory Extraction
            ActionSpec {
                name: "extract_memories".into(),
                description: "Extract and store key facts from conversation (Claude 4 style)"
                    .into(),
                params: vec![
                    ParamSpec {
                        name: "session_id".into(),
                        r#type: "String".into(),
                        description: "Source session ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "conversation".into(),
                        r#type: "String".into(),
                        description: "Conversation text to extract from".into(),
                        optional: false,
                    },
                ],
                returns: "[ExtractedMemory]".into(),
            },
            ActionSpec {
                name: "get_memories".into(),
                description: "Get stored memories with optional category filter".into(),
                params: vec![ParamSpec {
                    name: "category".into(),
                    r#type: "String".into(),
                    description: "Filter by category (preference/fact/instruction/context)".into(),
                    optional: true,
                }],
                returns: "[ExtractedMemory]".into(),
            },
            ActionSpec {
                name: "search_memories".into(),
                description: "Search memories by keyword".into(),
                params: vec![ParamSpec {
                    name: "query".into(),
                    r#type: "String".into(),
                    description: "Search query".into(),
                    optional: false,
                }],
                returns: "[ExtractedMemory]".into(),
            },
            ActionSpec {
                name: "delete_memory".into(),
                description: "Delete a memory by ID".into(),
                params: vec![ParamSpec {
                    name: "memory_id".into(),
                    r#type: "String".into(),
                    description: "Memory ID".into(),
                    optional: false,
                }],
                returns: "Bool".into(),
            },
            // Complexity Analysis & Routing
            ActionSpec {
                name: "analyze_complexity".into(),
                description: "Analyze query complexity and recommend model/thinking budget".into(),
                params: vec![ParamSpec {
                    name: "query".into(),
                    r#type: "String".into(),
                    description: "Query to analyze".into(),
                    optional: false,
                }],
                returns: "ComplexityAnalysis".into(),
            },
            ActionSpec {
                name: "route_query".into(),
                description: "Route query to optimal model based on complexity (GPT-5 style)"
                    .into(),
                params: vec![ParamSpec {
                    name: "query".into(),
                    r#type: "String".into(),
                    description: "Query to route".into(),
                    optional: false,
                }],
                returns: "RoutingDecision".into(),
            },
            // Parallel Tool Execution
            ActionSpec {
                name: "execute_parallel_tools".into(),
                description: "Execute multiple tool calls in parallel (Claude 4 style)".into(),
                params: vec![ParamSpec {
                    name: "calls".into(),
                    r#type: "[ParallelToolCall]".into(),
                    description: "List of tool calls to execute in parallel".into(),
                    optional: false,
                }],
                returns: "[ParallelToolResult]".into(),
            },
            // Statistics
            ActionSpec {
                name: "get_stats".into(),
                description: "Get orchestration statistics".into(),
                params: vec![],
                returns: "Value".into(),
            },
        ]
    }

    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            // ========== Thinking Budget ==========
            "set_thinking_budget" => {
                let budget_str = args["budget"]
                    .as_str()
                    .ok_or("Missing 'budget' parameter")?;
                let budget = match budget_str.to_lowercase().as_str() {
                    "off" => ThinkingBudget::Off,
                    "light" => ThinkingBudget::Light,
                    "medium" => ThinkingBudget::Medium,
                    "deep" => ThinkingBudget::Deep,
                    "adaptive" => ThinkingBudget::Adaptive,
                    _ => {
                        return Err(DomainError::from(format!(
                            "Invalid budget level: {}",
                            budget_str
                        )))
                    }
                };
                let mut state = self
                    .state
                    .lock().await
                    .map_err(|e| DomainError::from(e.to_string()))?;
                state.thinking_budget = budget.clone();
                Ok(serde_json::to_value(&budget).map_err(|e| DomainError::from(e.to_string()))?)
            }
            "get_thinking_budget" => {
                let state = self
                    .state
                    .lock().await
                    .map_err(|e| DomainError::from(e.to_string()))?;
                Ok(serde_json::to_value(&state.thinking_budget)
                    .map_err(|e| DomainError::from(e.to_string()))?)
            }

            // ========== Background Tasks ==========
            "submit_background_task" => {
                let description = args["description"]
                    .as_str()
                    .ok_or("Missing 'description' parameter")?;
                let task_id = uuid::Uuid::new_v4().to_string();
                let task = BackgroundTask {
                    id: task_id.clone(),
                    description: description.to_string(),
                    status: BackgroundTaskStatus::Queued,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    started_at: None,
                    completed_at: None,
                    progress: 0.0,
                    result: None,
                    error: None,
                };
                let mut state = self
                    .state
                    .lock().await
                    .map_err(|e| DomainError::from(e.to_string()))?;
                state.background_tasks.insert(task_id.clone(), task.clone());
                Ok(serde_json::to_value(&task).map_err(|e| DomainError::from(e.to_string()))?)
            }
            "get_background_task" => {
                let task_id = args["task_id"]
                    .as_str()
                    .ok_or("Missing 'task_id' parameter")?;
                let state = self
                    .state
                    .lock().await
                    .map_err(|e| DomainError::from(e.to_string()))?;
                let task = state
                    .background_tasks
                    .get(task_id)
                    .ok_or_else(|| DomainError::from(format!("Task not found: {}", task_id)))?;
                Ok(serde_json::to_value(task).map_err(|e| DomainError::from(e.to_string()))?)
            }
            "list_background_tasks" => {
                let state = self
                    .state
                    .lock().await
                    .map_err(|e| DomainError::from(e.to_string()))?;
                let tasks: Vec<&BackgroundTask> = if let Some(status_str) = args["status"].as_str()
                {
                    let status_filter = match status_str.to_lowercase().as_str() {
                        "queued" => BackgroundTaskStatus::Queued,
                        "running" => BackgroundTaskStatus::Running,
                        "completed" => BackgroundTaskStatus::Completed,
                        "failed" => BackgroundTaskStatus::Failed,
                        "cancelled" => BackgroundTaskStatus::Cancelled,
                        _ => {
                            return Err(DomainError::from(format!(
                                "Invalid status: {}",
                                status_str
                            )))
                        }
                    };
                    state
                        .background_tasks
                        .values()
                        .filter(|t| t.status == status_filter)
                        .collect()
                } else {
                    state.background_tasks.values().collect()
                };
                Ok(serde_json::to_value(&tasks).map_err(|e| DomainError::from(e.to_string()))?)
            }
            "cancel_background_task" => {
                let task_id = args["task_id"]
                    .as_str()
                    .ok_or("Missing 'task_id' parameter")?;
                let mut state = self
                    .state
                    .lock().await
                    .map_err(|e| DomainError::from(e.to_string()))?;
                if let Some(task) = state.background_tasks.get_mut(task_id) {
                    if task.status == BackgroundTaskStatus::Running
                        || task.status == BackgroundTaskStatus::Queued
                    {
                        task.status = BackgroundTaskStatus::Cancelled;
                        Ok(serde_json::Value::Bool(true))
                    } else {
                        Err(DomainError::from(format!(
                            "Task cannot be cancelled (status: {:?})",
                            task.status
                        )))
                    }
                } else {
                    Err(DomainError::from(format!("Task not found: {}", task_id)))
                }
            }

            // ========== Memory Extraction ==========
            "extract_memories" => {
                let session_id = args["session_id"]
                    .as_str()
                    .ok_or("Missing 'session_id' parameter")?;
                let conversation = args["conversation"]
                    .as_str()
                    .ok_or("Missing 'conversation' parameter")?;
                let extracted = self.extract_memories(session_id, conversation);
                let mut state = self
                    .state
                    .lock().await
                    .map_err(|e| DomainError::from(e.to_string()))?;
                state.memories.extend(extracted.clone());
                // Enforce max memories
                if state.memories.len() > state.max_memories {
                    let excess = state.memories.len() - state.max_memories;
                    state.memories.drain(0..excess);
                }
                Ok(serde_json::to_value(&extracted)
                    .map_err(|e| DomainError::from(e.to_string()))?)
            }
            "get_memories" => {
                let state = self
                    .state
                    .lock().await
                    .map_err(|e| DomainError::from(e.to_string()))?;
                let memories: Vec<&ExtractedMemory> =
                    if let Some(category) = args["category"].as_str() {
                        state
                            .memories
                            .iter()
                            .filter(|m| m.category == category)
                            .collect()
                    } else {
                        state.memories.iter().collect()
                    };
                Ok(
                    serde_json::to_value(&memories)
                        .map_err(|e| DomainError::from(e.to_string()))?,
                )
            }
            "search_memories" => {
                let query = args["query"].as_str().ok_or("Missing 'query' parameter")?;
                let state = self
                    .state
                    .lock().await
                    .map_err(|e| DomainError::from(e.to_string()))?;
                let query_lower = query.to_lowercase();
                let matches: Vec<&ExtractedMemory> = state
                    .memories
                    .iter()
                    .filter(|m| m.fact.to_lowercase().contains(&query_lower))
                    .collect();
                Ok(serde_json::to_value(&matches).map_err(|e| DomainError::from(e.to_string()))?)
            }
            "delete_memory" => {
                let memory_id = args["memory_id"]
                    .as_str()
                    .ok_or("Missing 'memory_id' parameter")?;
                let mut state = self
                    .state
                    .lock().await
                    .map_err(|e| DomainError::from(e.to_string()))?;
                let before = state.memories.len();
                state.memories.retain(|m| m.id != memory_id);
                Ok(serde_json::Value::Bool(state.memories.len() < before))
            }

            // ========== Complexity Analysis & Routing ==========
            "analyze_complexity" => {
                let query = args["query"].as_str().ok_or("Missing 'query' parameter")?;
                let analysis = self.analyze_complexity(query);
                Ok(
                    serde_json::to_value(&analysis)
                        .map_err(|e| DomainError::from(e.to_string()))?,
                )
            }
            "route_query" => {
                let query = args["query"].as_str().ok_or("Missing 'query' parameter")?;
                let analysis = self.analyze_complexity(query);
                let decision = self.route_to_model(&analysis);
                Ok(
                    serde_json::to_value(&decision)
                        .map_err(|e| DomainError::from(e.to_string()))?,
                )
            }

            // ========== Parallel Tool Execution ==========
            "execute_parallel_tools" => {
                let calls: Vec<ParallelToolCall> = serde_json::from_value(args["calls"].clone())
                    .map_err(|e| DomainError::from(format!("Invalid calls array: {}", e)))?;

                let mut results = Vec::new();
                for call in calls {
                    // In production, this would execute tools in parallel via tokio
                    // For now, simulate execution
                    let start = std::time::Instant::now();
                    let result = ParallelToolResult {
                        call_id: call.call_id,
                        tool_name: call.tool_name.clone(),
                        result: serde_json::json!({
                            "status": "simulated",
                            "tool": call.tool_name,
                        }),
                        success: true,
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                    results.push(result);
                }
                Ok(serde_json::to_value(&results).map_err(|e| DomainError::from(e.to_string()))?)
            }

            // ========== Statistics ==========
            "get_stats" => {
                let state = self
                    .state
                    .lock().await
                    .map_err(|e| DomainError::from(e.to_string()))?;
                let total_tasks = state.background_tasks.len();
                let completed_tasks = state
                    .background_tasks
                    .values()
                    .filter(|t| t.status == BackgroundTaskStatus::Completed)
                    .count();
                let failed_tasks = state
                    .background_tasks
                    .values()
                    .filter(|t| t.status == BackgroundTaskStatus::Failed)
                    .count();
                let memory_count = state.memories.len();
                let memory_categories: HashMap<String, usize> =
                    state.memories.iter().fold(HashMap::new(), |mut acc, m| {
                        *acc.entry(m.category.clone()).or_insert(0) += 1;
                        acc
                    });

                Ok(serde_json::json!({
                    "thinking_budget": state.thinking_budget,
                    "background_tasks": {
                        "total": total_tasks,
                        "completed": completed_tasks,
                        "failed": failed_tasks,
                    },
                    "memories": {
                        "total": memory_count,
                        "categories": memory_categories,
                    },
                    "memory_extraction_enabled": state.memory_extraction_enabled,
                }))
            }

            _ => Err(DomainError::from(format!("Unknown action: {}", action))),
        }
    }
}
