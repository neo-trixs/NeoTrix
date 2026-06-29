use crate::core::skills::SkillTier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolRisk {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ToolClassification {
    pub tool_name: String,
    pub tier: SkillTier,
    pub risk_level: ToolRisk,
    pub requires_confirmation: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_classification_defaults() {
        let tc = ToolClassification {
            tool_name: "read_file".into(),
            tier: SkillTier::Builtin,
            risk_level: ToolRisk::Safe,
            requires_confirmation: false,
        };
        assert_eq!(tc.tool_name, "read_file");
        assert_eq!(tc.risk_level, ToolRisk::Safe);
        assert!(!tc.requires_confirmation);
    }

    #[test]
    fn test_tool_risk_ordering() {
        assert_ne!(ToolRisk::Safe as u8, ToolRisk::Critical as u8);
    }
}
