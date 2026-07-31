use std::collections::HashMap;

/// Reverse bridge: types re-exported from `crate::core::edit`.
pub use crate::core::nt_core_edit::{SelfEdit, MicroEdit, ToolCall};

use crate::core::nt_core_knowledge::TaskType;

pub fn infer_task_type(task: &str) -> TaskType {
    let t = task.to_lowercase();
    if t.contains("设计") || t.contains("design") || t.contains("ui") || t.contains("界面") {
        TaskType::Design
    } else if t.contains("代码") || t.contains("code") || t.contains("函数") || t.contains("rust") {
        TaskType::CodeAnalysis
    } else if t.contains("测试") || t.contains("test") || t.contains("验证") || t.contains("review") {
        TaskType::CodeReview
    } else {
        TaskType::General
    }
}

pub fn select_relevant_dimensions(task_type: &TaskType) -> Vec<String> {
    match task_type {
        TaskType::Design | TaskType::UIDesign => {
            vec![
                "typography".to_string(), "grid".to_string(), "color".to_string(),
                "whitespace".to_string(), "accessibility".to_string(),
                "compound_composition".to_string(), "tailwind_proficiency".to_string(),
            ]
        }
        TaskType::CodeAnalysis | TaskType::CodeGeneration => {
            vec![
                "analysis".to_string(), "synthesis".to_string(),
                "inference_depth".to_string(), "domain_specificity".to_string(),
            ]
        }
        _ => vec!["analysis".to_string(), "creativity".to_string()],
    }
}

pub fn calculate_adjustment_magnitude(task_type: &TaskType) -> f64 {
    let base_magnitude = 0.1f64;
    let task_factor = match task_type {
        TaskType::Design | TaskType::UIDesign => 0.15f64,
        TaskType::CodeAnalysis | TaskType::CodeGeneration => 0.12f64,
        _ => 0.1f64,
    };
    f64::min(base_magnitude + task_factor, 0.5f64)
}

pub fn generate_tool_calls(task_type: &TaskType, _task: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();

    calls.push(ToolCall {
        tool: "playwright".to_string(),
        action: "screenshot".to_string(),
        params: {
            let mut m = HashMap::new();
            m.insert("target".to_string(), "output".to_string());
            m
        },
    });

    if *task_type == TaskType::Design || *task_type == TaskType::UIDesign {
        calls.push(ToolCall {
            tool: "cua".to_string(),
            action: "verify".to_string(),
            params: {
                let mut m = HashMap::new();
                m.insert("check".to_string(), "accessibility".to_string());
                m
            },
        });
    }

    calls
}
