// Phase 2a: HIR type checking + validation pass.
// Separates concerns: parser builds Module IR, lower validates + enriches it.

use crate::ir::{
    BinOp, Expr, Function, Import, Literal, Module, Pipeline, QuantPrecision, Type, UnOp, VsaDim,
};

/// Validation diagnostics.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: LowerSeverity,
    pub message: String,
    pub location: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LowerSeverity {
    Error,
    Warning,
}

/// Result of lowering + validation.
#[derive(Debug)]
pub struct LoweredModule {
    pub module: Module,
    pub diagnostics: Vec<Diagnostic>,
}

impl LoweredModule {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == LowerSeverity::Error)
    }
}

/// Run lowering and validation on a parsed Module.
pub fn lower(m: Module) -> LoweredModule {
    let mut diags = Vec::new();

    // Check name
    if m.name.is_empty() {
        diags.push(warn("name", "Module name is empty"));
    }

    // Validate imports
    for imp in &m.imports {
        check_import(imp, &mut diags);
    }

    // Validate functions
    let mut func_names = std::collections::HashSet::new();
    for func in &m.functions {
        if !func_names.insert(func.name.clone()) {
            diags.push(error(
                &func.name,
                &format!("Duplicate function name '{}'", func.name),
            ));
        }
        check_function(func, &mut diags);
    }

    // Validate pipeline if present
    if let Some(ref pipeline) = m.pipeline {
        check_pipeline(pipeline, &func_names, &mut diags);
    }

    // Check expression types for all function bodies
    let dim = m.vsa_dim.unwrap_or(4096);
    for func in &m.functions {
        let _ = infer_expr_type(&func.body, &func.params, &func_names, dim, &mut diags);
    }

    LoweredModule {
        module: m,
        diagnostics: diags,
    }
}

// ---- Import validation ----

fn check_import(imp: &Import, diags: &mut Vec<Diagnostic>) {
    if imp.path.is_empty() {
        diags.push(warn("import", "Empty import path"));
    }
    if imp.path.starts_with('/') {
        diags.push(warn(
            &imp.path,
            "Absolute import path; relative paths preferred",
        ));
    }
}

// ---- Function validation ----

fn check_function(func: &Function, diags: &mut Vec<Diagnostic>) {
    if func.name.starts_with("__") {
        diags.push(warn(
            &func.name,
            "Name starts with '__'; reserved for internal use",
        ));
    }

    for param in &func.params {
        if param.name.is_empty() {
            diags.push(error(&func.name, "Empty parameter name"));
        }
        if !is_valid_type(&param.param_type) {
            diags.push(error(
                &func.name,
                &format!("Unknown parameter type '{}'", param.param_type.name()),
            ));
        }
    }

    if !is_valid_type(&func.return_type) {
        diags.push(error(
            &func.name,
            &format!("Unknown return type '{}'", func.return_type.name()),
        ));
    }
}

fn is_valid_type(t: &Type) -> bool {
    match t {
        Type::Vsa(_, _) => true,
        Type::F32 | Type::F64 | Type::U64 | Type::I64 | Type::Usize => true,
        Type::Bool | Type::String => true,
        Type::Fuzzy => true,
        Type::Vec(inner) => is_valid_type(inner),
        Type::VecU8 => true,
        Type::Named(name) => !name.is_empty(),
    }
}

// ---- Pipeline validation ----

fn check_pipeline(
    pipeline: &Pipeline,
    func_names: &std::collections::HashSet<String>,
    diags: &mut Vec<Diagnostic>,
) {
    if pipeline.stages.is_empty() {
        diags.push(warn(&pipeline.name, "Pipeline has no stages"));
    }

    for (i, stage) in pipeline.stages.iter().enumerate() {
        if stage.name.is_empty() {
            diags.push(error(
                &pipeline.name,
                &format!("Stage {} has empty name", i),
            ));
        }

        if let Some(ref func_ref) = stage.function_ref {
            if !func_names.contains(func_ref) {
                diags.push(error(
                    &stage.name,
                    &format!("Stage references unknown function '{}'", func_ref),
                ));
            }
        }

        // Check type compatibility between consecutive stages
        if i > 0 {
            let prev_output = &pipeline.stages[i - 1].output;
            if !types_compatible(prev_output, &stage.input) {
                diags.push(warn(
                    &stage.name,
                    &format!(
                        "Type mismatch: stage '{}' output '{}' vs stage '{}' input '{}'",
                        pipeline.stages[i - 1].name,
                        prev_output.name(),
                        stage.name,
                        stage.input.name(),
                    ),
                ));
            }
        }
    }

    // Check pipeline input/output if specified
    if let Some(ref input_type) = pipeline.input_type {
        if pipeline.stages.is_empty() {
            diags.push(warn(
                &pipeline.name,
                "Input type specified but pipeline has no stages",
            ));
        } else if !types_compatible(input_type, &pipeline.stages[0].input) {
            diags.push(warn(
                &pipeline.name,
                &format!(
                    "Pipeline input type '{}' does not match first stage input '{}'",
                    input_type.name(),
                    pipeline.stages[0].input.name(),
                ),
            ));
        }
    }

    if let Some(ref output_type) = pipeline.output_type {
        if let Some(last) = pipeline.stages.last() {
            if !types_compatible(output_type, &last.output) {
                diags.push(warn(
                    &pipeline.name,
                    &format!(
                        "Pipeline output type '{}' does not match last stage output '{}'",
                        output_type.name(),
                        last.output.name(),
                    ),
                ));
            }
        }
    }
}

fn types_compatible(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::Vsa(_, _), Type::Vsa(_, _)) => true,
        _ => a.name() == b.name(),
    }
}

// ---- Expression type inference (placeholder for full Hindley-Milner) ----

fn infer_expr_type(
    expr: &Expr,
    params: &[crate::ir::Param],
    func_names: &std::collections::HashSet<String>,
    default_dim: usize,
    diags: &mut Vec<Diagnostic>,
) -> Option<Type> {
    match expr {
        Expr::Literal(lit) => Some(literal_type(lit)),
        Expr::Ident(name) => {
            // Check if it's a parameter
            for param in params {
                if param.name == *name {
                    return Some(param.param_type.clone());
                }
            }
            // Built-in
            Some(Type::Named(name.clone()))
        }
        Expr::Call(name, args) => {
            // Recursively infer arg types
            for arg in args {
                infer_expr_type(arg, params, func_names, default_dim, diags);
            }
            if func_names.contains(name) {
                // Known function call — for now return generic Vsa
                Some(Type::Vsa(VsaDim::Dim(default_dim), None))
            } else {
                match name.as_str() {
                    "bundle" | "bind" | "permute" | "negate" | "similarity" | "VsaOps::bundle"
                    | "VsaOps::bind" | "VsaOps::permute" | "VsaOps::negate"
                    | "VsaOps::similarity" => Some(Type::Vsa(VsaDim::Dim(default_dim), None)),
                    "assert_eq" | "assert_ne" => None,
                    "println" | "format" => Some(Type::String),
                    _ => {
                        diags.push(warn(name, &format!("Unknown function '{}'", name)));
                        None
                    }
                }
            }
        }
        Expr::Binary(BinOp::VsaBundle | BinOp::VsaBind | BinOp::VsaPermute, _, _) => {
            Some(Type::Vsa(VsaDim::Dim(default_dim), None))
        }
        Expr::Binary(BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod, _, _) => {
            Some(Type::I64)
        }
        Expr::Binary(
            BinOp::Eq
            | BinOp::Ne
            | BinOp::Lt
            | BinOp::Gt
            | BinOp::Le
            | BinOp::Ge
            | BinOp::And
            | BinOp::Or,
            _,
            _,
        ) => Some(Type::Bool),
        Expr::Binary(BinOp::FuzzyAnd | BinOp::FuzzyOr | BinOp::FuzzyImply, _, _) => {
            Some(Type::Fuzzy)
        }
        Expr::Unary(UnOp::Neg, _) => Some(Type::I64),
        Expr::Unary(UnOp::Not, _) => Some(Type::Bool),
        Expr::Unary(UnOp::VsaNegate, _) => Some(Type::Vsa(VsaDim::Dim(default_dim), None)),
        Expr::Unary(UnOp::FuzzyNot, _) => Some(Type::Fuzzy),
        Expr::Block(exprs) => {
            if exprs.is_empty() {
                None
            } else {
                exprs
                    .iter()
                    .last()
                    .and_then(|e| infer_expr_type(e, params, func_names, default_dim, diags))
            }
        }
        Expr::If(cond, then_branch, else_branch) => {
            infer_expr_type(cond, params, func_names, default_dim, diags);
            let t = infer_expr_type(then_branch, params, func_names, default_dim, diags);
            let e = else_branch
                .as_ref()
                .and_then(|e| infer_expr_type(e, params, func_names, default_dim, diags));
            t.or(e)
        }
        Expr::For(_, iterable, _) => {
            infer_expr_type(iterable, params, func_names, default_dim, diags);
            None
        }
        Expr::Let(_, val) => {
            infer_expr_type(val, params, func_names, default_dim, diags);
            None
        }
        Expr::Return(val) => {
            val.as_ref()
                .and_then(|v| infer_expr_type(v, params, func_names, default_dim, diags));
            None
        }
        Expr::PipelineRef(_) => Some(Type::Vsa(VsaDim::Dim(default_dim), None)),
    }
}

fn literal_type(lit: &Literal) -> Type {
    match lit {
        Literal::Int(_) => Type::I64,
        Literal::Float(_) => Type::F64,
        Literal::Bool(_) => Type::Bool,
        Literal::String(_) => Type::String,
        Literal::VsaVector(_) => Type::Vsa(VsaDim::Dim(4096), Some(QuantPrecision::I8)),
    }
}

// ---- Helpers ----

fn error(location: &str, message: &str) -> Diagnostic {
    Diagnostic {
        severity: LowerSeverity::Error,
        message: message.to_string(),
        location: location.to_string(),
    }
}

fn warn(location: &str, message: &str) -> Diagnostic {
    Diagnostic {
        severity: LowerSeverity::Warning,
        message: message.to_string(),
        location: location.to_string(),
    }
}

// Helper to get a type's display name without requiring name() on Type
// (We already have Type::name() in ir.rs)

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;
    use std::path::PathBuf;

    #[test]
    fn test_lower_empty_module() {
        let m = Module {
            name: "empty".into(),
            description: "".into(),
            source_file: PathBuf::from("empty.nt"),
            vsa_dim: None,
            imports: vec![],
            functions: vec![],
            pipeline: None,
            tests: vec![],
        };
        let lm = lower(m);
        assert!(!lm.has_errors());
    }

    #[test]
    fn test_lower_detects_duplicate_function() {
        let m = Module {
            name: "dup".into(),
            description: "".into(),
            source_file: PathBuf::from("dup.nt"),
            vsa_dim: None,
            imports: vec![],
            functions: vec![
                Function {
                    name: "foo".into(),
                    params: vec![],
                    return_type: Type::I64,
                    body: Expr::Literal(Literal::Int(1)),
                    description: None,
                },
                Function {
                    name: "foo".into(),
                    params: vec![],
                    return_type: Type::I64,
                    body: Expr::Literal(Literal::Int(2)),
                    description: None,
                },
            ],
            pipeline: None,
            tests: vec![],
        };
        let lm = lower(m);
        assert!(lm.has_errors());
        assert!(lm
            .diagnostics
            .iter()
            .any(|d| d.message.contains("Duplicate")));
    }

    #[test]
    fn test_lower_pipeline_bad_ref() {
        let m = Module {
            name: "pipe".into(),
            description: "".into(),
            source_file: PathBuf::from("pipe.nt"),
            vsa_dim: None,
            imports: vec![],
            functions: vec![],
            pipeline: Some(Pipeline {
                name: "Pipe".into(),
                stages: vec![PipelineStage {
                    name: "s1".into(),
                    input: Type::I64,
                    output: Type::I64,
                    function_ref: Some("nonexistent".into()),
                }],
                input_type: None,
                output_type: None,
            }),
            tests: vec![],
        };
        let lm = lower(m);
        assert!(lm.has_errors());
        assert!(lm
            .diagnostics
            .iter()
            .any(|d| d.message.contains("unknown function")));
    }

    #[test]
    fn test_lower_empty_pipeline_warning() {
        let m = Module {
            name: "emptypipe".into(),
            description: "".into(),
            source_file: PathBuf::from("empty.nt"),
            vsa_dim: None,
            imports: vec![],
            functions: vec![],
            pipeline: Some(Pipeline {
                name: "Empty".into(),
                stages: vec![],
                input_type: None,
                output_type: None,
            }),
            tests: vec![],
        };
        let lm = lower(m);
        assert!(!lm.has_errors());
        assert!(lm
            .diagnostics
            .iter()
            .any(|d| d.message.contains("no stages")));
    }

    #[test]
    fn test_type_inference_literal_int() {
        assert_eq!(literal_type(&Literal::Int(42)), Type::I64);
    }

    #[test]
    fn test_type_inference_bool() {
        assert_eq!(literal_type(&Literal::Bool(true)), Type::Bool);
    }

    #[test]
    fn test_type_inference_string() {
        assert_eq!(literal_type(&Literal::String("hi".into())), Type::String);
    }

    #[test]
    fn test_types_compatible_same() {
        assert!(types_compatible(&Type::I64, &Type::I64));
    }

    #[test]
    fn test_types_compatible_vsa() {
        assert!(types_compatible(
            &Type::Vsa(VsaDim::Dim(4096), None),
            &Type::Vsa(VsaDim::Dim(1024), None)
        ));
    }

    #[test]
    fn test_types_compatible_different() {
        assert!(!types_compatible(&Type::I64, &Type::String));
    }
}
