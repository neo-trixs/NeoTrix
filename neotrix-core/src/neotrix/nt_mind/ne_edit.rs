/// Ne Edit — first production use of Ne language self-edits.
///
/// Parses minimal `.ne` edit source and applies it to the consciousness runtime.
/// Phase 25, G21: First production Ne edit (InnerCritic.relevance_threshold).
///
/// Format:
/// ```ne
/// edit <target> to <f64_value>
///   reason "<reason_string>"
///   gate "<gate_name>"
/// ```
use crate::neotrix::nt_mind_background_loop::consciousness::ConsciousnessIntegration;

/// A single Ne edit directive parsed from source text.
#[derive(Debug, Clone, PartialEq)]
pub struct NeEdit {
    /// Dot-separated field path, e.g. "inner_critic.relevance_threshold"
    pub target: String,
    /// New floating-point value
    pub value_f64: f64,
    /// Human-readable justification
    pub reason: String,
    /// PACE gate that must approve this edit
    pub gate: String,
}

/// Parse a minimal Ne edit string.
///
/// Expected format:
/// ```text
/// edit <target> to <value>
///   reason "<reason>"
///   gate "<gate>"
/// ```
pub fn parse_ne_edit(source: &str) -> Result<NeEdit, String> {
    let mut target: Option<String> = None;
    let mut value_f64: Option<f64> = None;
    let mut reason: Option<String> = None;
    let mut gate: Option<String> = None;

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("edit ") {
            // "edit <target> to <value>"
            let rest = &line["edit ".len()..];
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            if parts.len() < 3 || parts[1] != "to" {
                return Err(format!(
                    "parse error: expected 'edit <target> to <value>', got '{}'",
                    line
                ));
            }
            target = Some(parts[0].to_string());
            value_f64 =
                Some(parts[2].parse::<f64>().map_err(|e| {
                    format!("parse error: invalid f64 value '{}': {}", parts[2], e)
                })?);
        } else if line.starts_with("reason ") {
            // "reason "<reason>""
            let rest = line["reason ".len()..].trim();
            reason = Some(rest.trim_matches('"').trim_matches('\'').to_string());
        } else if line.starts_with("gate ") {
            // "gate "<gate>""
            let rest = line["gate ".len()..].trim();
            gate = Some(rest.trim_matches('"').trim_matches('\'').to_string());
        } else {
            return Err(format!("parse error: unrecognized line '{}'", line));
        }
    }

    let target =
        target.ok_or_else(|| "parse error: missing 'edit <target> to <value>' line".to_string())?;
    let value = value_f64.ok_or_else(|| "parse error: missing value in edit line".to_string())?;
    let reason = reason.unwrap_or_else(|| "auto-optimize".to_string());
    let gate = gate.unwrap_or_else(|| "meta_monitor".to_string());

    Ok(NeEdit {
        target,
        value_f64: value,
        reason,
        gate,
    })
}

/// Parse and apply a Ne edit string to a ConsciousnessIntegration.
/// Returns a human-readable log of what was changed.
pub fn apply_ne_edit(source: &str, ci: &mut ConsciousnessIntegration) -> Result<String, String> {
    let edit = parse_ne_edit(source)?;
    Ok(ci.apply_ne_edit(&edit.target, edit.value_f64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind_background_loop::consciousness::ConsciousnessIntegration;

    #[test]
    fn test_parse_ne_edit_basic() {
        let source = r#"edit inner_critic.relevance_threshold to 0.5
  reason "relax relevance threshold after 1000+ critiques"
  gate "meta_monitor""#;
        let edit = parse_ne_edit(source).expect("should parse");
        assert_eq!(edit.target, "inner_critic.relevance_threshold");
        assert!((edit.value_f64 - 0.5).abs() < 1e-9);
        assert_eq!(
            edit.reason,
            "relax relevance threshold after 1000+ critiques"
        );
        assert_eq!(edit.gate, "meta_monitor");
    }

    #[test]
    fn test_parse_ne_edit_minimal() {
        let source = "edit inner_critic.consistency_threshold to 0.6";
        let edit = parse_ne_edit(source).expect("should parse");
        assert_eq!(edit.target, "inner_critic.consistency_threshold");
        assert!((edit.value_f64 - 0.6).abs() < 1e-9);
        assert_eq!(edit.reason, "auto-optimize");
        assert_eq!(edit.gate, "meta_monitor");
    }

    #[test]
    fn test_parse_ne_edit_missing_edit_line() {
        let result = parse_ne_edit("reason \"test\"\n  gate \"meta\"");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("missing 'edit"));
    }

    #[test]
    fn test_parse_ne_edit_invalid_value() {
        let result = parse_ne_edit("edit foo to bar");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid f64"));
    }

    #[test]
    fn test_parse_ne_edit_bad_syntax() {
        let result = parse_ne_edit("edit foo");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected 'edit"));
    }

    #[test]
    fn test_apply_ne_edit_relevance() {
        let mut ci = ConsciousnessIntegration::new();
        let old = ci.inner_critic.relevance_threshold();
        let source = r#"edit inner_critic.relevance_threshold to 0.8
  reason "tighten for higher quality"
  gate "meta_monitor""#;
        let log = apply_ne_edit(source, &mut ci).expect("should apply");
        assert!((ci.inner_critic.relevance_threshold() - 0.8).abs() < 1e-9);
        assert!(log.contains("relevance_threshold"));
        assert!(log.contains(&format!("{}", old)));
        assert!(log.contains("0.8"));
    }

    #[test]
    fn test_apply_ne_edit_consistency() {
        let mut ci = ConsciousnessIntegration::new();
        let _old = ci.inner_critic.consistency_threshold();
        let source = "edit inner_critic.consistency_threshold to 0.4";
        let log = apply_ne_edit(source, &mut ci).expect("should apply");
        assert!((ci.inner_critic.consistency_threshold() - 0.4).abs() < 1e-9);
        assert!(log.contains("consistency_threshold"));
    }

    #[test]
    fn test_apply_ne_edit_unknown_target() {
        let mut ci = ConsciousnessIntegration::new();
        let source = "edit imaginary.field to 0.5";
        let log = apply_ne_edit(source, &mut ci).expect("should apply");
        assert!(log.contains("unknown target"));
    }

    #[test]
    fn test_apply_ne_edit_gate_propagation() {
        let source = r#"edit inner_critic.relevance_threshold to 0.5
  reason "test propagation"
  gate "meta_monitor""#;
        let edit = parse_ne_edit(source).unwrap();
        assert_eq!(edit.reason, "test propagation");
        assert_eq!(edit.gate, "meta_monitor");
    }
}
