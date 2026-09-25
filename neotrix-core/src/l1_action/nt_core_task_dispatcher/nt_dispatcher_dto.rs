//! Dispatcher DTO — L1 本地 DTO 翻译 + 任务数据结构（纯搬移，行为零变更）。
//!
//! 内容：`LocalDecomposeSuggestion` / `LocalCrtPlan` / `LocalTrace`
//! （T03c 类型本地化）+ `SubTask` / `DecompositionResult` /
//! `SubTaskResult` / `TaskExecutionContext`。

use crate::l5_cognition::nt_core::capability::nt_core_antidistil::decompose::DecomposeSuggestion;
use crate::l5_cognition::nt_core::nt_crt::{CrtPlan, CrtTimeScale};
use crate::l5_cognition::reasoning_core::TraceSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── T03c L1 本地 DTO（类型本地化，不动行为） ──
// 新写入点先构本地 DTO 再转回 L5（读点未迁，见遗留清单）；全部宽松反序列化供 LLM/旧 JSON 兼容。
// E8Policy 为纯存储字段（组合根注入、本文件无读取），保留原类型＋注记，不另建 DTO。

/// L1 本地拆解建议。from_l5：title←subtask，reason←reasoning（逐字段直对）；
/// detail 为溢出槽（L5 暂无对应字段，置空供后续扩展）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LocalDecomposeSuggestion {
    pub title: String,
    pub detail: String,
    pub reason: String,
}

impl LocalDecomposeSuggestion {
    pub fn from_l5(s: &DecomposeSuggestion) -> Self {
        Self {
            title: s.subtask.clone(),
            detail: String::new(),
            reason: s.reasoning.clone(),
        }
    }
}

/// L1 本地 CRT 计划。from_l5：scale←CrtTimeScale::label，
/// budget_hours←time_budget_seconds/3600；
/// max_ticks/sub_plans/parent_scale 无对应位（读点 DecompositionResult.crt_plan 未迁，暂舍）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LocalCrtPlan {
    pub scale: String,
    pub budget_hours: f64,
}

impl LocalCrtPlan {
    pub fn from_l5(plan: &CrtPlan) -> Self {
        Self {
            scale: plan.scale.label().to_string(),
            budget_hours: plan.time_budget_seconds / 3600.0,
        }
    }

    /// 转回 L5（写点回填用；未知字符串回落调用方原 scale，保证行为不变）。
    pub fn to_scale_label(scale: &str, fallback: CrtTimeScale) -> CrtTimeScale {
        match scale {
            "gaitian" => CrtTimeScale::Gaitian,
            "huntian" => CrtTimeScale::Huntian,
            "xuanye" => CrtTimeScale::Xuanye,
            _ => fallback,
        }
    }
}

/// L1 本地轨迹摘要。source←Debug 字符串（TraceSource / CoTOutput / ReasoningTrace 三源），
/// 对不上结构的载荷进 detail 字符串；读点（kernel_trace 构造、cot_output 字段）未迁。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LocalTrace {
    pub source: String,
    pub detail: String,
}

impl LocalTrace {
    pub fn from_trace_source(s: &TraceSource) -> Self {
        Self {
            source: format!("{:?}", s),
            detail: String::new(),
        }
    }

    pub fn from_cot(o: &crate::l5_cognition::nt_core_cot_generator::CoTOutput) -> Self {
        Self {
            source: "CoTOutput".to_string(),
            detail: o.final_answer.clone(),
        }
    }

    pub fn from_reasoning_trace(t: &crate::l5_cognition::reasoning_core::ReasoningTrace) -> Self {
        Self {
            source: format!("{:?}", t.source),
            detail: t.task.clone(),
        }
    }
}

/// 子任务定义
///
/// 降级定位（E1.4/T19）：LLM 派发中间态，只做分解输出，不做调度。正典调度见 Scheduler。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub prompt: String,                     // 发送给 LLM 的精准提示词
    pub context: HashMap<String, String>,   // 上下文信息
    pub priority: u8,                       // 优先级 1-10
    pub estimated_complexity: f64,          // 0.0-1.0
    pub required_capabilities: Vec<String>, // 所需能力标签
    pub dependencies: Vec<String>,          // 依赖的子任务 ID
    pub crt_scale: CrtTimeScale,            // 所属 CRT 时间尺度
    pub hexagram_bias: Option<u8>,          // E8 hexagram 偏好
}

/// 任务拆解结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompositionResult {
    pub original_task: String,
    pub sub_tasks: Vec<SubTask>,
    pub execution_order: Vec<String>, // 执行顺序（拓扑排序）
    pub crt_plan: CrtPlan,
    pub estimated_total_time: f64, // 预估总耗时（秒）
    pub confidence: f64,           // 拆解置信度 0.0-1.0
}

/// 子任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTaskResult {
    pub sub_task_id: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub tokens_used: u32,
    pub duration_ms: u64,
    pub cot_output: Option<crate::l5_cognition::nt_core_cot_generator::CoTOutput>,
}

/// 任务执行上下文（在拆解和执行过程中传递）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionContext {
    pub original_task: String,
    pub decomposition: DecompositionResult,
    pub completed_tasks: HashMap<String, SubTaskResult>,
    pub current_task_index: usize,
    pub global_context: HashMap<String, String>,
    pub start_time: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_core::nt_crt::CrtPlan;
    use crate::l5_cognition::nt_core::nt_crt::CrtTimeScale;
    use crate::l5_cognition::reasoning_core::TraceSource;
    use neotrix_reasoning::kernel_types::ReasoningMethod;

    #[test]
    fn test_sub_task_creation() {
        let sub_task = SubTask {
            id: "test_1".to_string(),
            title: "Test Task".to_string(),
            description: "A test task".to_string(),
            prompt: "Do something".to_string(),
            context: HashMap::new(),
            priority: 5,
            estimated_complexity: 0.5,
            required_capabilities: vec!["general".to_string()],
            dependencies: Vec::new(),
            crt_scale: CrtTimeScale::Gaitian,
            hexagram_bias: Some(3),
        };
        assert_eq!(sub_task.id, "test_1");
        assert_eq!(sub_task.crt_scale, CrtTimeScale::Gaitian);
    }

    #[test]
    fn test_t03c_local_dto_roundtrip_and_from_l5() {
        // T03c 类型本地化冒烟：from_l5 字段直对＋serde 宽松往返（最小字面量）。
        let s = DecomposeSuggestion {
            subtask: "Phase 1: design".to_string(),
            reasoning: "reduce scope".to_string(),
        };
        let local_s = LocalDecomposeSuggestion::from_l5(&s);
        assert_eq!(local_s.title, "Phase 1: design");
        assert_eq!(local_s.reason, "reduce scope");
        assert!(local_s.detail.is_empty());

        let plan = CrtPlan::new(CrtTimeScale::Gaitian, 3600.0);
        let local_p = LocalCrtPlan::from_l5(&plan);
        assert_eq!(local_p.scale, "gaitian");
        assert_eq!(local_p.budget_hours, 1.0);
        assert_eq!(
            LocalCrtPlan::to_scale_label(&local_p.scale, CrtTimeScale::Xuanye),
            CrtTimeScale::Gaitian
        );

        let local_t = LocalTrace::from_trace_source(&TraceSource::LLMDriven);
        assert_eq!(local_t.source, "LLMDriven");

        let cot = crate::l5_cognition::nt_core_cot_generator::CoTOutput {
            reasoning_steps: Vec::new(),
            final_answer: "ans".to_string(),
            overall_confidence: 0.9,
            raw_response: String::new(),
        };
        let local_c = LocalTrace::from_cot(&cot);
        assert_eq!(local_c.source, "CoTOutput");
        assert_eq!(local_c.detail, "ans");

        let trace = crate::l5_cognition::reasoning_core::ReasoningTrace {
            trace_id: "t".to_string(),
            task: "task".to_string(),
            method: ReasoningMethod::Deductive,
            hexagram: neotrix_types::e8_reasoning::ReasoningHexagram::new(0),
            stage: 0,
            steps: Vec::new(),
            intermediate_states: Vec::new(),
            convergence: 0.5,
            final_quality: 0.5,
            llm_response: None,
            source: TraceSource::LLMDriven,
            timestamp: 0,
        };
        let local_r = LocalTrace::from_reasoning_trace(&trace);
        assert_eq!(local_r.source, "LLMDriven");
        assert_eq!(local_r.detail, "task");

        // serde 往返＋宽松（缺字段不断行）。
        assert!(serde_json::to_string(&local_s).is_ok());
        assert!(serde_json::to_string(&local_p).is_ok());
        assert!(serde_json::to_string(&local_t).is_ok());
        assert!(serde_json::from_str::<LocalDecomposeSuggestion>("{}").is_ok());
        assert!(serde_json::from_str::<LocalCrtPlan>("{}").is_ok());
        assert!(serde_json::from_str::<LocalTrace>("{}").is_ok());
        if let Ok(json) = serde_json::to_string(&local_s) {
            if let Ok(back) = serde_json::from_str::<LocalDecomposeSuggestion>(&json) {
                assert_eq!(back.title, "Phase 1: design");
            } else {
                assert!(false, "local dto roundtrip must parse");
            }
        } else {
            assert!(false, "local dto roundtrip must serialize");
        }
    }
}
