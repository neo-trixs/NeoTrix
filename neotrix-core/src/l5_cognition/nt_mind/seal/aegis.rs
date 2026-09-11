//! AEGIS 4 阶段进化引擎 — 基于 HarnessX 模式
//!
//! Digester→Planner→Evolver→Critic 四阶段确定性门控进化。

/// 执行轨迹（Digest 输入）
#[allow(dead_code)]
#[derive(Clone)]
pub struct ExecutionTrace {
    pub trace_id: String,
    pub task: String,
    pub steps: Vec<TraceStep>,
    pub success: bool,
    pub total_tokens: u32,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct TraceStep {
    pub action: String,
    pub input: String,
    pub output: String,
    pub success: bool,
    pub latency_ms: u64,
}

/// Digest 输出（压缩轨迹）
#[allow(dead_code)]
#[derive(Clone)]
pub struct TraceDigest {
    pub trace_id: String,
    pub task_summary: String,
    pub success_patterns: Vec<String>,
    pub failure_patterns: Vec<String>,
    pub token_usage: u32,
    pub key_decisions: Vec<String>,
}

/// 适应景观（Planner 输出）
#[allow(dead_code)]
#[derive(Clone)]
pub struct AdaptationLandscape {
    pub dimensions: Vec<AdaptationDimension>,
    pub current_position: Vec<f64>,
    pub target_position: Vec<f64>,
    pub potential_improvements: Vec<String>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct AdaptationDimension {
    pub name: String,
    pub current_value: f64,
    pub optimal_value: f64,
    pub weight: f64,
}

/// 类型化编辑（Evolver 输出）
#[allow(dead_code)]
#[derive(Clone)]
pub struct TypedEdit {
    pub edit_id: String,
    pub edit_type: EditType,
    pub target: String,
    pub before: String,
    pub after: String,
    pub confidence: f64,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum EditType {
    PromptModification,
    ToolSelectionChange,
    ContextManagementChange,
    RoutingLogicChange,
    MemoryStrategyChange,
}

/// Critic 评估
#[allow(dead_code)]
#[derive(Clone)]
pub struct CriticAssessment {
    pub edit_id: String,
    pub supported: bool,
    pub evidence: String,
    pub risk_score: f64, // 0.0-1.0
    pub improvement_estimate: f64,
}

/// AEGIS 进化引擎
#[allow(dead_code)]
pub struct AegisEngine {
    traces: Vec<ExecutionTrace>,
    digests: Vec<TraceDigest>,
    landscape: Option<AdaptationLandscape>,
    edits: Vec<TypedEdit>,
    assessments: Vec<CriticAssessment>,
    adopted_edits: Vec<TypedEdit>,
    rejected_edits: Vec<(TypedEdit, String)>,
}

#[allow(dead_code)]
impl AegisEngine {
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
            digests: Vec::new(),
            landscape: None,
            edits: Vec::new(),
            assessments: Vec::new(),
            adopted_edits: Vec::new(),
            rejected_edits: Vec::new(),
        }
    }

    /// Stage 1: Digester — 压缩轨迹
    pub fn digest(&mut self, trace: ExecutionTrace) -> TraceDigest {
        let success_patterns: Vec<String> = trace
            .steps
            .iter()
            .filter(|s| s.success)
            .map(|s| format!("{}: 成功", s.action))
            .collect();

        let failure_patterns: Vec<String> = trace
            .steps
            .iter()
            .filter(|s| !s.success)
            .map(|s| format!("{}: 失败 ({})", s.action, s.output))
            .collect();

        let key_decisions: Vec<String> = trace
            .steps
            .iter()
            .map(|s| format!("{} → {}", s.action, if s.success { "OK" } else { "FAIL" }))
            .collect();

        let digest = TraceDigest {
            trace_id: trace.trace_id.clone(),
            task_summary: trace.task.clone(),
            success_patterns,
            failure_patterns,
            token_usage: trace.total_tokens,
            key_decisions,
        };

        self.traces.push(trace);
        self.digests.push(digest.clone());
        digest
    }

    /// Stage 2: Planner — 分析适应景观
    pub fn plan(&mut self) -> AdaptationLandscape {
        let dimensions = vec![
            AdaptationDimension {
                name: "prompt_quality".to_string(),
                current_value: 0.7,
                optimal_value: 0.9,
                weight: 0.3,
            },
            AdaptationDimension {
                name: "tool_selection".to_string(),
                current_value: 0.6,
                optimal_value: 0.85,
                weight: 0.25,
            },
            AdaptationDimension {
                name: "context_efficiency".to_string(),
                current_value: 0.5,
                optimal_value: 0.8,
                weight: 0.25,
            },
            AdaptationDimension {
                name: "routing_accuracy".to_string(),
                current_value: 0.65,
                optimal_value: 0.9,
                weight: 0.2,
            },
        ];

        let current_position: Vec<f64> = dimensions.iter().map(|d| d.current_value).collect();
        let target_position: Vec<f64> = dimensions.iter().map(|d| d.optimal_value).collect();

        let potential_improvements: Vec<String> = dimensions
            .iter()
            .filter(|d| d.optimal_value - d.current_value > 0.1)
            .map(|d| {
                format!(
                    "{}: {:.2} → {:.2}",
                    d.name, d.current_value, d.optimal_value
                )
            })
            .collect();

        let landscape = AdaptationLandscape {
            dimensions,
            current_position,
            target_position,
            potential_improvements,
        };

        self.landscape = Some(landscape.clone());
        landscape
    }

    /// Stage 3: Evolver — 生成类型化编辑
    pub fn evolve(&mut self) -> Vec<TypedEdit> {
        let mut edits = Vec::new();

        if let Some(landscape) = &self.landscape {
            for dim in &landscape.dimensions {
                if dim.optimal_value - dim.current_value > 0.1 {
                    let edit = TypedEdit {
                        edit_id: format!("edit_{}", edits.len()),
                        edit_type: match dim.name.as_str() {
                            "prompt_quality" => EditType::PromptModification,
                            "tool_selection" => EditType::ToolSelectionChange,
                            "context_efficiency" => EditType::ContextManagementChange,
                            _ => EditType::RoutingLogicChange,
                        },
                        target: dim.name.clone(),
                        before: format!("{:.2}", dim.current_value),
                        after: format!("{:.2}", dim.optimal_value),
                        confidence: dim.weight,
                    };
                    edits.push(edit);
                }
            }
        }

        self.edits = edits.clone();
        edits
    }

    /// Stage 4: Critic — 确定性门控评估
    pub fn critique(&mut self) -> Vec<CriticAssessment> {
        let assessments: Vec<CriticAssessment> = self
            .edits
            .iter()
            .map(|edit| {
                let supported = edit.confidence > 0.5;
                let risk_score = if supported { 0.2 } else { 0.8 };
                let improvement_estimate = edit.confidence * 0.3;

                CriticAssessment {
                    edit_id: edit.edit_id.clone(),
                    supported,
                    evidence: if supported {
                        "高置信度编辑".to_string()
                    } else {
                        "低置信度，需要更多证据".to_string()
                    },
                    risk_score,
                    improvement_estimate,
                }
            })
            .collect();

        self.assessments = assessments.clone();
        assessments
    }

    /// 确定性门控：只有通过 Critic 的编辑才能被采纳
    pub fn gate(&mut self) -> Vec<TypedEdit> {
        let mut adopted = Vec::new();

        for edit in &self.edits {
            if let Some(assessment) = self.assessments.iter().find(|a| a.edit_id == edit.edit_id) {
                if assessment.supported && assessment.risk_score < 0.5 {
                    self.adopted_edits.push(edit.clone());
                    adopted.push(edit.clone());
                } else {
                    self.rejected_edits
                        .push((edit.clone(), assessment.evidence.clone()));
                }
            }
        }

        adopted
    }

    /// 获取统计
    pub fn stats(&self) -> AegisStats {
        AegisStats {
            traces: self.traces.len(),
            digests: self.digests.len(),
            edits: self.edits.len(),
            adopted: self.adopted_edits.len(),
            rejected: self.rejected_edits.len(),
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct AegisStats {
    pub traces: usize,
    pub digests: usize,
    pub edits: usize,
    pub adopted: usize,
    pub rejected: usize,
}
