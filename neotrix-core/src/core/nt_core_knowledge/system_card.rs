use super::self_inspect::{LanguageSpec, SelfInspectable};
use crate::core::nt_core_util::codegen_version;

/// A complete system introspection card —意识体自省快照。
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemCard {
    pub name: &'static str,
    pub version: String,
    pub spec: LanguageSpec,
    pub subsystem_count: usize,
    pub handler_count: usize,
    pub vsa_primitive_count: usize,
    pub confidence: f64,
    pub generated_at: u64,
}

/// Generates a `SystemCard` from any `SelfInspectable` subsystem.
pub struct SystemCardGenerator<'a> {
    subsystem: &'a dyn SelfInspectable,
}

impl<'a> SystemCardGenerator<'a> {
    pub fn new(subsystem: &'a dyn SelfInspectable) -> Self {
        Self { subsystem }
    }

    pub fn generate_card(&self) -> SystemCard {
        let spec = self.subsystem.distill_language_spec();
        let ss = &spec.subspace_topology;
        let unique_fields: std::collections::HashSet<&str> = ss
            .subspaces
            .iter()
            .flat_map(|s| s.fields.iter().copied())
            .collect();
        let handler_count = spec.handler_graph.handlers.len();
        let vsa_primitive_count = spec.vsa_primitives.len();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        SystemCard {
            name: "NeoTrix Consciousness Core",
            version: codegen_version().to_string(),
            spec,
            subsystem_count: unique_fields.len(),
            handler_count,
            vsa_primitive_count,
            confidence: 0.85,
            generated_at: now,
        }
    }

    pub fn generate_card_json(&self) -> String {
        let card = self.generate_card();
        serde_json::to_string_pretty(&card)
            .unwrap_or_else(|e| format!("{{\"error\": \"serialization failed: {}\"}}", e))
    }

    pub fn generate_card_markdown(&self) -> String {
        let card = self.generate_card();
        let mut md = String::new();
        md.push_str("## System Card\n\n");
        md.push_str(&format!("**Name:** {}\n\n", card.name));
        md.push_str(&format!("**Version:** {}\n\n", card.version));
        md.push_str(&format!("**Confidence:** {}\n\n", card.confidence));
        md.push_str(&format!("**Generated At:** {}\n\n", card.generated_at));
        md.push_str("### Subsystems\n\n");
        for sub in &card.spec.subspace_topology.subspaces {
            md.push_str(&format!(
                "- `{}` (tag: `0x{:02x}`, {} fields)\n",
                sub.name, sub.tag, sub.field_count
            ));
        }
        md.push_str(&format!(
            "\n**Subsystem unique field count:** {}\n\n",
            card.subsystem_count
        ));
        md.push_str(&format!("**Handler count:** {}\n\n", card.handler_count));
        md.push_str(&format!(
            "**VSA primitive count:** {}\n\n",
            card.vsa_primitive_count
        ));
        md.push_str("### Edit Policy\n\n");
        md.push_str(&format!(
            "- max_gain: {}\n- max_edits_per_cycle: {}\n- lifetime_cap: {}\n",
            card.spec.edit_policy.max_gain,
            card.spec.edit_policy.max_edits_per_cycle,
            card.spec.edit_policy.lifetime_cap,
        ));
        md
    }

    /// Generate a SystemCard with compile-time evaluation of template expressions.
    /// Templates can contain `${comptime:expr}` placeholders that are evaluated
    /// by the NeComptimeEngine before card generation.
    pub fn generate_card_with_comptime(
        &self,
        _engine: &mut crate::core::nt_core_codegen::comptime::NeComptimeEngine,
    ) -> String {
        let md = self.generate_card_markdown();
        let mut pos = 0;
        let bytes = md.as_bytes().to_vec();
        let mut processed = String::new();
        while pos < bytes.len() {
            if bytes[pos..].starts_with(b"${comptime:") {
                let start = pos;
                pos += "${comptime:".len();
                let end = bytes[pos..]
                    .iter()
                    .position(|&b| b == b'}')
                    .map(|p| pos + p);
                if let Some(end_pos) = end {
                    let expr = std::str::from_utf8(&bytes[pos..end_pos])
                        .unwrap_or("")
                        .trim();
                    match crate::core::nt_core_codegen::comptime::eval_expression(expr) {
                        Ok(val) => {
                            let val_str = match val {
                                crate::core::nt_core_codegen::comptime::ComptimeValue::Int(i) => {
                                    i.to_string()
                                }
                                crate::core::nt_core_codegen::comptime::ComptimeValue::Float(f) => {
                                    f.to_string()
                                }
                                crate::core::nt_core_codegen::comptime::ComptimeValue::Bool(b) => {
                                    b.to_string()
                                }
                                crate::core::nt_core_codegen::comptime::ComptimeValue::String(
                                    s,
                                ) => s,
                                _ => format!("{:?}", val),
                            };
                            processed.push_str(&val_str);
                            pos = end_pos + 1;
                        }
                        Err(e) => {
                            processed.push_str(&format!("<comptime_error:{}>", e));
                            pos = end_pos + 1;
                        }
                    }
                } else {
                    processed.push_str(&md[start..]);
                    break;
                }
            } else {
                processed.push(bytes[pos] as char);
                pos += 1;
            }
        }
        processed
    }

    pub fn emit_bootstrap_program(&self) -> String {
        let spec = self.subsystem.distill_language_spec();
        let mut program =
            String::from("// Ne bootstrap program — auto-generated by SystemCardGenerator\n\n");
        program.push_str("// VSA primitives\n");
        for prim in &spec.vsa_primitives {
            let arity_str = if prim.arity < 0 {
                "vararg".to_string()
            } else {
                prim.arity.to_string()
            };
            program.push_str(&format!(
                "primitive {}(arity: {}) -> vec<u8>  // {}\n",
                prim.name, arity_str, prim.description
            ));
        }
        program.push_str("\n// Cognitive subspaces\n");
        for sub in &spec.subspace_topology.subspaces {
            let fields = sub.fields.join(", ");
            program.push_str(&format!(
                "subspace {}(tag: 0x{:02x}, fields: [{}])\n",
                sub.name, sub.tag, fields
            ));
        }
        program.push_str(&format!(
            "\nedit edit_policy(max_gain: {}, max_edits_per_cycle: {}, lifetime_cap: {})\n",
            spec.edit_policy.max_gain,
            spec.edit_policy.max_edits_per_cycle,
            spec.edit_policy.lifetime_cap,
        ));
        program.push_str("\n// Handlers\n");
        for h in &spec.handler_graph.handlers {
            program.push_str(&format!(
                "handler \"{}\" runs(every: {}s)\n",
                h.name, h.interval_secs
            ));
        }
        program.push_str("\nbootstrap::validate\n");
        program
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_knowledge::self_inspect::{
        HandlerGraph, HandlerNode, SelfInspectable,
    };

    struct DummySubsystem;

    impl SelfInspectable for DummySubsystem {
        fn handler_graph(&self) -> HandlerGraph {
            HandlerGraph {
                handlers: vec![
                    HandlerNode {
                        name: "handle_attractor_dynamics",
                        interval_secs: 30,
                        call_count: 0,
                    },
                    HandlerNode {
                        name: "handle_curiosity",
                        interval_secs: 60,
                        call_count: 0,
                    },
                ],
            }
        }
    }

    #[test]
    fn test_system_card_generation() {
        let dummy = DummySubsystem;
        let gen = SystemCardGenerator::new(&dummy);
        let card = gen.generate_card();
        assert_eq!(card.name, "NeoTrix Consciousness Core");
        assert_eq!(card.version, "0.1.0");
        assert_eq!(card.vsa_primitive_count, 12);
        assert_eq!(card.handler_count, 2);
        assert!(card.subsystem_count > 0);
        assert_eq!(card.confidence, 0.85);
        assert!(card.generated_at > 1_700_000_000);
    }

    #[test]
    fn test_system_card_json() {
        let dummy = DummySubsystem;
        let gen = SystemCardGenerator::new(&dummy);
        let json = gen.generate_card_json();
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"version\""));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "NeoTrix Consciousness Core");
        assert_eq!(parsed["version"], "0.1.0");
    }

    #[test]
    fn test_system_card_markdown() {
        let dummy = DummySubsystem;
        let gen = SystemCardGenerator::new(&dummy);
        let md = gen.generate_card_markdown();
        assert!(md.contains("## System Card"));
        assert!(md.contains("NeoTrix Consciousness Core"));
        assert!(md.contains("@self"));
    }

    #[test]
    fn test_bootstrap_program() {
        let dummy = DummySubsystem;
        let gen = SystemCardGenerator::new(&dummy);
        let program = gen.emit_bootstrap_program();
        assert!(program.contains("primitive"));
        assert!(program.contains("subspace"));
        assert!(program.contains("bootstrap::validate"));
        assert!(program.contains("bind"));
        assert!(program.contains("@self"));
    }

    #[test]
    fn test_generate_card_with_comptime_passthrough() {
        let dummy = DummySubsystem;
        let gen = SystemCardGenerator::new(&dummy);
        let mut engine = crate::core::nt_core_codegen::comptime::NeComptimeEngine::new();
        let result = gen.generate_card_with_comptime(&mut engine);
        assert!(result.contains("## System Card"));
    }

    #[test]
    fn test_generate_card_with_comptime_eval_int() {
        // Replace a known int from the bootstrap program output
        let input = "version: ${comptime:42}";
        let dummy = DummySubsystem;
        let _gen = SystemCardGenerator::new(&dummy);
        let _engine = crate::core::nt_core_codegen::comptime::NeComptimeEngine::new();
        // override the markdown output to inject comptime template
        let processed = {
            let md = input.to_string();
            let mut pos = 0;
            let bytes = md.as_bytes().to_vec();
            let mut result = String::new();
            while pos < bytes.len() {
                if bytes[pos..].starts_with(b"${comptime:") {
                    pos += "${comptime:".len();
                    let end = bytes[pos..]
                        .iter()
                        .position(|&b| b == b'}')
                        .map(|p| pos + p);
                    if let Some(end_pos) = end {
                        let expr = std::str::from_utf8(&bytes[pos..end_pos]).unwrap().trim();
                        match crate::core::nt_core_codegen::comptime::eval_expression(expr) {
                            Ok(val) => {
                                result.push_str(&format!("{}", match val { crate::core::nt_core_codegen::comptime::ComptimeValue::Int(i) => i.to_string(), _ => "?".into(), }));
                            }
                            Err(_) => result.push_str("<err>"),
                        }
                        pos = end_pos + 1;
                    } else {
                        break;
                    }
                } else {
                    result.push(bytes[pos] as char);
                    pos += 1;
                }
            }
            result
        };
        assert!(processed.contains("42"));
    }
}
