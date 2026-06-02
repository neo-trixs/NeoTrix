use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MicroEdit {
    AdjustDimension(String, f64),
    UpdateLearningRate(f64),
    NormalizeVector,
    AddExtension(Vec<(String, f64)>),
    SetProvenance(String),
    BatchAdjust(Vec<(String, f64)>),
    AddedDimension(String, f64),
    ModifiedDimension(String, f64, f64),
    RemovedDimension(String),
}

impl MicroEdit {
    pub fn delta_label(&self) -> &'static str {
        match self {
            MicroEdit::AddedDimension(_, _) => "ADDED",
            MicroEdit::ModifiedDimension(_, _, _) => "MODIFIED",
            MicroEdit::RemovedDimension(_) => "REMOVED",
            MicroEdit::AdjustDimension(_, _) => "ADJUSTED",
            MicroEdit::BatchAdjust(_) => "BATCH_ADJUSTED",
            _ => "OTHER",
        }
    }

    pub fn dimension_name(&self) -> Option<&str> {
        match self {
            MicroEdit::AddedDimension(name, _) => Some(name),
            MicroEdit::ModifiedDimension(name, _, _) => Some(name),
            MicroEdit::RemovedDimension(name) => Some(name),
            MicroEdit::AdjustDimension(name, _) => Some(name),
            _ => None,
        }
    }

    /// 人类可读的摘要（供 T3 视图生成使用）
    pub fn summary(&self) -> String {
        match self {
            MicroEdit::AdjustDimension(name, val) => format!("adjust dimension '{}' by {}", name, val),
            MicroEdit::UpdateLearningRate(lr) => format!("update learning rate to {}", lr),
            MicroEdit::NormalizeVector => "normalize capability vector".into(),
            MicroEdit::AddExtension(exts) => format!("add {} extensions", exts.len()),
            MicroEdit::SetProvenance(src) => format!("set provenance to '{}'", src),
            MicroEdit::BatchAdjust(adjustments) => format!("batch adjust {} dimensions", adjustments.len()),
            MicroEdit::AddedDimension(name, val) => format!("add dimension '{}' = {}", name, val),
            MicroEdit::ModifiedDimension(name, old, new) => format!("modify '{}': {} → {}", name, old, new),
            MicroEdit::RemovedDimension(name) => format!("remove dimension '{}'", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfEdit {
    pub task_type: crate::core::TaskType,
    pub target_dimensions: Vec<String>,
    pub adjustment_magnitude: f64,
    pub tool_calls: Vec<ToolCall>,
    pub config_overrides: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub action: String,
    pub params: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micro_edit_adjust_dimension() {
        let edit = MicroEdit::AdjustDimension("typography".into(), 0.5);
        assert_eq!(edit.delta_label(), "ADJUSTED");
        assert_eq!(edit.dimension_name(), Some("typography"));
        assert_eq!(edit.summary(), "adjust dimension 'typography' by 0.5");
    }

    #[test]
    fn test_micro_edit_update_learning_rate() {
        let edit = MicroEdit::UpdateLearningRate(0.01);
        assert_eq!(edit.delta_label(), "OTHER");
        assert_eq!(edit.dimension_name(), None);
        assert_eq!(edit.summary(), "update learning rate to 0.01");
    }

    #[test]
    fn test_micro_edit_normalize() {
        let edit = MicroEdit::NormalizeVector;
        assert_eq!(edit.summary(), "normalize capability vector");
    }

    #[test]
    fn test_micro_edit_add_extension() {
        let edit = MicroEdit::AddExtension(vec![("dim1".into(), 0.5)]);
        assert_eq!(edit.summary(), "add 1 extensions");
    }

    #[test]
    fn test_micro_edit_set_provenance() {
        let edit = MicroEdit::SetProvenance("test_source".into());
        assert_eq!(edit.summary(), "set provenance to 'test_source'");
    }

    #[test]
    fn test_micro_edit_batch_adjust() {
        let edit = MicroEdit::BatchAdjust(vec![("a".into(), 0.1), ("b".into(), 0.2)]);
        assert_eq!(edit.summary(), "batch adjust 2 dimensions");
        assert_eq!(edit.delta_label(), "BATCH_ADJUSTED");
    }

    #[test]
    fn test_micro_edit_dimension_tracking() {
        let added = MicroEdit::AddedDimension("new_dim".into(), 0.8);
        assert_eq!(added.delta_label(), "ADDED");
        assert_eq!(added.dimension_name(), Some("new_dim"));
        assert_eq!(added.summary(), "add dimension 'new_dim' = 0.8");

        let modified = MicroEdit::ModifiedDimension("dim".into(), 0.3, 0.7);
        assert_eq!(modified.delta_label(), "MODIFIED");
        assert_eq!(modified.dimension_name(), Some("dim"));
        assert_eq!(modified.summary(), "modify 'dim': 0.3 → 0.7");

        let removed = MicroEdit::RemovedDimension("old_dim".into());
        assert_eq!(removed.delta_label(), "REMOVED");
        assert_eq!(removed.dimension_name(), Some("old_dim"));
        assert_eq!(removed.summary(), "remove dimension 'old_dim'");
    }

    #[test]
    fn test_micro_edit_clone() {
        let edit = MicroEdit::AdjustDimension("test".into(), 0.5);
        let cloned = edit.clone();
        assert_eq!(edit.summary(), cloned.summary());
    }

    #[test]
    fn test_self_edit_creation() {
        let mut config = HashMap::new();
        config.insert("learning_rate".into(), 0.05);
        let se = SelfEdit {
            task_type: crate::core::TaskType::CodeGeneration,
            target_dimensions: vec!["analysis".into(), "creativity".into()],
            adjustment_magnitude: 0.1,
            tool_calls: Vec::new(),
            config_overrides: config,
        };
        assert_eq!(se.target_dimensions.len(), 2);
        assert!((se.adjustment_magnitude - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_tool_call_creation() {
        let mut params = HashMap::new();
        params.insert("url".into(), "https://example.com".into());
        let tc = ToolCall {
            tool: "web_scrape".into(),
            action: "fetch".into(),
            params,
        };
        assert_eq!(tc.tool, "web_scrape");
        assert_eq!(tc.params.get("url").expect("value should be ok in test"), "https://example.com");
    }
}
