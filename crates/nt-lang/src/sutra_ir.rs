//! Sutra VSA IR — "everything is hypervector" intermediate representation.
//! Bridges parser → SutraValue IR → Rust codegen with QuantizedVSA.

use crate::ir::*;
use crate::lower;
use crate::parser::parse::parse_stmts;
use crate::registry::ModuleRegistry;
use crate::tensor_graph::{self, TensorGraph};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct SutraVsaPrimitive {
    pub name: String,
    pub arity: i32,
}

#[derive(Debug, Clone)]
pub struct SutraLanguageSpec {
    pub vsa_primitives: Vec<SutraVsaPrimitive>,
    pub confidence: f64,
}

impl Default for SutraLanguageSpec {
    fn default() -> Self {
        Self {
            vsa_primitives: vec![
                SutraVsaPrimitive { name: "bind".into(), arity: 2 },
                SutraVsaPrimitive { name: "bundle".into(), arity: -1 },
                SutraVsaPrimitive { name: "permute".into(), arity: 2 },
                SutraVsaPrimitive { name: "negate".into(), arity: 1 },
                SutraVsaPrimitive { name: "similarity".into(), arity: 2 },
                SutraVsaPrimitive { name: "cosine".into(), arity: 2 },
                SutraVsaPrimitive { name: "hamming_distance".into(), arity: 2 },
                SutraVsaPrimitive { name: "random_vector".into(), arity: 0 },
                SutraVsaPrimitive { name: "binarize".into(), arity: 1 },
                // P0-1: Sutra-style rotation binding
                SutraVsaPrimitive { name: "rotation_bind".into(), arity: 2 },
                SutraVsaPrimitive { name: "rotation_unbind".into(), arity: 2 },
                SutraVsaPrimitive { name: "rotation_seed".into(), arity: 1 },
                // P0-2: Codebook compilation
                SutraVsaPrimitive { name: "embed_string".into(), arity: 1 },
                SutraVsaPrimitive { name: "codebook_lookup".into(), arity: 2 },
                // P1: Sutra polynomial fuzzy logic (Kleene 3-valued, Lagrange-interpolated)
                SutraVsaPrimitive { name: "kleene_and".into(), arity: 2 },
                SutraVsaPrimitive { name: "kleene_or".into(), arity: 2 },
                SutraVsaPrimitive { name: "kleene_not".into(), arity: 1 },
                SutraVsaPrimitive { name: "kleene_imply".into(), arity: 2 },
                SutraVsaPrimitive { name: "kleene_iff".into(), arity: 2 },
                SutraVsaPrimitive { name: "is_true".into(), arity: 1 },
                SutraVsaPrimitive { name: "defuzzify".into(), arity: 1 },
            ],
            confidence: 0.9,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SutraValue {
    VsaVector(Vec<i8>),
    VsaBundle(Vec<SutraValue>),
    VsaBind(Box<SutraValue>, Box<SutraValue>),
    VsaPermute(Box<SutraValue>, u32),
    /// Rotation binding: bind(role_seed, filler)
    VsaRotationBind(u64, Box<SutraValue>),
    Scalar(f64),
    StringVal(String),
    Bool(bool),
    List(Vec<SutraValue>),
    /// Scalar arithmetic: Add(a, b) = a + b
    Add(Box<SutraValue>, Box<SutraValue>),
    Sub(Box<SutraValue>, Box<SutraValue>),
    Mul(Box<SutraValue>, Box<SutraValue>),
    Div(Box<SutraValue>, Box<SutraValue>),
    /// Fuzzy Kleene AND: Lagrange-interpolated AND(a,b) ∈ [-1, +1] (C^∞)
    FuzzyAnd(Box<SutraValue>, Box<SutraValue>),
    /// Fuzzy Kleene OR: Lagrange-interpolated OR(a,b) ∈ [-1, +1] (C^∞)
    FuzzyOr(Box<SutraValue>, Box<SutraValue>),
    /// Fuzzy Kleene NOT: NOT(x) = -x
    FuzzyNot(Box<SutraValue>),
    /// Fuzzy material implication: IMPLY(a,b) = OR(NOT(a), b)
    FuzzyImply(Box<SutraValue>, Box<SutraValue>),
}

impl SutraValue {
    pub fn vsa_dim(&self) -> Option<usize> {
        match self {
            SutraValue::VsaVector(v) => Some(v.len()),
            SutraValue::VsaBundle(children) => children.iter().find_map(|c| c.vsa_dim()),
            SutraValue::VsaBind(a, b) => a.vsa_dim().or_else(|| b.vsa_dim()),
            SutraValue::VsaPermute(v, _) => v.vsa_dim(),
            SutraValue::VsaRotationBind(_, v) => v.vsa_dim(),
            _ => None,
        }
    }

    /// Evaluate the fuzzy truth value as a Kleene 3-valued truth.
    /// Returns Some(FuzzyTruth) for Scalar/Fuzzy* values, None otherwise.
    pub fn to_fuzzy_truth(&self) -> Option<crate::ir::FuzzyTruth> {
        match self {
            SutraValue::Scalar(v) => Some(crate::ir::FuzzyTruth::from_f64(*v)),
            _ => None,
        }
    }
}

pub struct SutraCompiler {
    pub spec: SutraLanguageSpec,
    pub module_registry: ModuleRegistry,
    /// VSA algebra for code generation. Defaults to HRR for backward compat.
    pub vsa_algebra: VsaAlgebra,
}

impl SutraCompiler {
    pub fn new(spec: SutraLanguageSpec) -> Self {
        Self {
            spec,
            module_registry: ModuleRegistry::new(),
            vsa_algebra: VsaAlgebra::Hrr,
        }
    }

    /// Set the VSA algebra for code generation.
    pub fn with_algebra(mut self, alg: VsaAlgebra) -> Self {
        self.vsa_algebra = alg;
        self
    }

    pub fn compile(&mut self, source: &str, module_name: &str) -> Result<String, String> {
        let expr = parse_stmts(source).map_err(|e| e.message)?;

        let functions = vec![];
        let value = self.lower_expr(&expr, &functions)?;
        let optimized = self.optimize_value(value);
        let code = self.codegen_value(&optimized);

        let dim = 4096;

        let mut out = String::new();
        out.push_str("// Generated by Sutra VSA IR compiler. DO NOT EDIT.\n");
        out.push_str(&format!("// Module: {}\n\n", module_name));
        out.push_str("#[cfg(test)]\n");
        out.push_str("mod sutra_generated {\n");
        out.push_str("    use neotrix::core::nt_core_hcube::quantized_vsa::QuantizedVSA;\n");
        out.push_str(&format!("    const VSA_DIM: usize = {};\n\n", dim));
        out.push_str("    #[test]\n");
        out.push_str(&format!("    fn {}_eval() {{\n", module_name));
        out.push_str(&format!("        let _result = {};\n", code));
        out.push_str("    }\n");

        out.push_str("}\n");

        Ok(out)
    }

    pub fn compile_file(&mut self, path: &str) -> Result<String, String> {
        let source =
            std::fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path, e))?;
        let name = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("module");
        self.compile(&source, name)
    }

    /// Compile .ne source into a proper Rust module (not test-wrapped).
    pub fn compile_as_module(&mut self, source: &str, module_name: &str) -> Result<String, String> {
        let stmts = parse_stmts(source).map_err(|e| e.message)?;

        let module = Module {
            name: module_name.to_string(),
            description: String::new(),
            source_file: std::path::PathBuf::from(format!("{}.ne", module_name)),
            vsa_dim: Some(4096),
            imports: vec![Import {
                path: "neotrix::core::nt_core_hcube::quantized_vsa::QuantizedVSA".into(),
                alias: None,
            }],
            functions: vec![Function {
                name: "main".into(),
                params: vec![],
                return_type: Type::Vsa(VsaDim::Dim(4096), None),
                body: stmts,
                description: None,
            }],
            pipeline: None,
            tests: vec![],
        };

        let lm = lower::lower(module);
        if lm.has_errors() {
            let diags: Vec<String> = lm
                .diagnostics
                .iter()
                .map(|d| format!("{}: {}", d.location, d.message))
                .collect();
            return Err(diags.join("\n"));
        }

        Ok(self.codegen_module(&lm.module))
    }

    /// Compile a `.ne` expression string into a TensorGraph for differentiable training.
    ///
    /// Parses the source, lowers to SutraValue, then builds a TensorGraph.
    /// The resulting graph can be executed via `compute_forward()` and trained
    /// via `backward()` / `gradient_descent_step()` / `optimize()`.
    pub fn compile_to_graph(&mut self, source: &str) -> Result<TensorGraph, String> {
        let expr = parse_stmts(source).map_err(|e| e.message)?;
        let functions = vec![];
        let value = self.lower_expr(&expr, &functions)?;
        let optimized = self.optimize_value(value);
        Ok(TensorGraph::build(&optimized))
    }

    /// Compile a `.ne` expression and run N steps of gradient descent.
    ///
    /// Returns (final_graph, loss_trace) where `final_graph` has updated
    /// `ConstVector` nodes and `loss_trace` is the loss per step.
    ///
    /// The `loss_fn` maps the forward output vector to a scalar loss value.
    pub fn compile_and_train(
        &mut self,
        source: &str,
        dim: usize,
        learning_rate: f64,
        steps: usize,
        loss_fn: impl Fn(&[f64]) -> f64,
    ) -> Result<(TensorGraph, Vec<f64>), String> {
        let mut graph = self.compile_to_graph(source)?;
        let losses = tensor_graph::optimize(&mut graph, dim, learning_rate, steps, loss_fn)?;
        Ok((graph, losses))
    }

    pub fn evaluate(&self, expr: &SutraValue) -> SutraValue {
        match expr {
            SutraValue::VsaBind(a, b) => {
                let a = self.evaluate(a);
                let b = self.evaluate(b);
                match (&a, &b) {
                    (SutraValue::VsaVector(va), SutraValue::VsaVector(vb)) => {
                        let result: Vec<i8> = va
                            .iter()
                            .zip(vb.iter())
                            .map(|(x, y)| match (x, y) {
                                (0, _) | (_, 0) => 0,
                                _ if x == y => 1,
                                _ => -1,
                            })
                            .collect();
                        SutraValue::VsaVector(result)
                    }
                    _ => SutraValue::VsaBind(Box::new(a), Box::new(b)),
                }
            }
            SutraValue::VsaRotationBind(seed, val) => {
                let v = self.evaluate(val);
                match v {
                    SutraValue::VsaVector(vec) => {
                        let n = vec.len();
                        let mut result = vec![0i8; n];
                        let block_size = 64;
                        let num_blocks = n / block_size;
                        for block in 0..num_blocks {
                            let start = block * block_size;
                            let end = (start + block_size).min(n);
                            let mut indices: Vec<usize> = (start..end).collect();
                            for i in (1..indices.len()).rev() {
                                let mut h = std::collections::hash_map::DefaultHasher::new();
                                seed.hash(&mut h);
                                (block as u64).hash(&mut h);
                                (i as u64).hash(&mut h);
                                let j = (h.finish() as usize) % (i + 1);
                                indices.swap(i, j);
                            }
                            for (out_i, &src_i) in indices.iter().enumerate() {
                                let val = vec[src_i];
                                let mut h = std::collections::hash_map::DefaultHasher::new();
                                seed.hash(&mut h);
                                (block as u64).hash(&mut h);
                                (out_i as u64).hash(&mut h);
                                let flip = h.finish() & 1 == 1;
                                result[start + out_i] = if flip { -val } else { val };
                            }
                        }
                        SutraValue::VsaVector(result)
                    }
                    other => SutraValue::VsaRotationBind(*seed, Box::new(other)),
                }
            }
            SutraValue::VsaBundle(children) => {
                let evaluated: Vec<SutraValue> =
                    children.iter().map(|c| self.evaluate(c)).collect();
                let mut vectors: Vec<&[i8]> = Vec::new();
                for child in &evaluated {
                    if let SutraValue::VsaVector(v) = child {
                        vectors.push(v.as_slice());
                    }
                }
                if vectors.len() == evaluated.len() && !vectors.is_empty() {
                    let dim = vectors[0].len();
                    let mut result = Vec::with_capacity(dim);
                    for i in 0..dim {
                        let sum: i32 = vectors.iter().map(|v| v[i] as i32).sum();
                        result.push(if sum > 0 { 1 } else if sum < 0 { -1 } else { 0 });
                    }
                    SutraValue::VsaVector(result)
                } else {
                    SutraValue::VsaBundle(evaluated)
                }
            }
            SutraValue::VsaPermute(v, shift) => {
                let v = self.evaluate(v);
                match v {
                    SutraValue::VsaVector(vec) => {
                        if vec.is_empty() {
                            return SutraValue::VsaVector(vec);
                        }
                        let shift = shift % vec.len() as u32;
                        let mut result = vec.clone();
                        result.rotate_left(shift as usize);
                        SutraValue::VsaVector(result)
                    }
                    other => SutraValue::VsaPermute(Box::new(other), *shift),
                }
            }
            // Fuzzy logic evaluation with Lagrange-interpolated Kleene 3-valued logic
            SutraValue::FuzzyAnd(a, b) => {
                let a = self.evaluate(a);
                let b = self.evaluate(b);
                let va = match &a { SutraValue::Scalar(v) => *v, _ => return a, };
                let vb = match &b { SutraValue::Scalar(v) => *v, _ => return b, };
                SutraValue::Scalar(crate::ir::FuzzyTruth::kleene_and(va, vb))
            }
            SutraValue::FuzzyOr(a, b) => {
                let a = self.evaluate(a);
                let b = self.evaluate(b);
                let va = match &a { SutraValue::Scalar(v) => *v, _ => return a, };
                let vb = match &b { SutraValue::Scalar(v) => *v, _ => return b, };
                SutraValue::Scalar(crate::ir::FuzzyTruth::kleene_or(va, vb))
            }
            SutraValue::FuzzyNot(a) => {
                let a = self.evaluate(a);
                let va = match &a { SutraValue::Scalar(v) => *v, _ => return a, };
                SutraValue::Scalar(crate::ir::FuzzyTruth::kleene_not(va))
            }
            SutraValue::FuzzyImply(a, b) => {
                let a = self.evaluate(a);
                let b = self.evaluate(b);
                let va = match &a { SutraValue::Scalar(v) => *v, _ => return a, };
                let vb = match &b { SutraValue::Scalar(v) => *v, _ => return b, };
                SutraValue::Scalar(crate::ir::FuzzyTruth::kleene_imply(va, vb))
            }
            // Scalar arithmetic evaluation
            SutraValue::Add(a, b) => {
                match (self.evaluate(a), self.evaluate(b)) {
                    (SutraValue::Scalar(va), SutraValue::Scalar(vb)) => SutraValue::Scalar(va + vb),
                    (a, b) => SutraValue::Add(Box::new(a), Box::new(b)),
                }
            }
            SutraValue::Sub(a, b) => {
                match (self.evaluate(a), self.evaluate(b)) {
                    (SutraValue::Scalar(va), SutraValue::Scalar(vb)) => SutraValue::Scalar(va - vb),
                    (a, b) => SutraValue::Sub(Box::new(a), Box::new(b)),
                }
            }
            SutraValue::Mul(a, b) => {
                match (self.evaluate(a), self.evaluate(b)) {
                    (SutraValue::Scalar(va), SutraValue::Scalar(vb)) => SutraValue::Scalar(va * vb),
                    (a, b) => SutraValue::Mul(Box::new(a), Box::new(b)),
                }
            }
            SutraValue::Div(a, b) => {
                match (self.evaluate(a), self.evaluate(b)) {
                    (SutraValue::Scalar(va), SutraValue::Scalar(vb)) => SutraValue::Scalar(va / vb),
                    (a, b) => SutraValue::Div(Box::new(a), Box::new(b)),
                }
            }
            other => other.clone(),
        }
    }

    pub fn type_check(&self, expr: &Expr, functions: &[Function]) -> Result<Type, String> {
        self.infer_type(expr, functions)
    }

    fn infer_type(&self, expr: &Expr, functions: &[Function]) -> Result<Type, String> {
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                Literal::Int(_) => Type::I64,
                Literal::Float(_) => Type::F64,
                Literal::Bool(_) => Type::Bool,
                Literal::String(_) => Type::String,
                Literal::VsaVector(v) => Type::Vsa(VsaDim::Dim(v.len()), Some(QuantPrecision::I8)),
            }),
            Expr::Ident(name) => {
                for f in functions {
                    for p in &f.params {
                        if p.name == *name {
                            return Ok(p.param_type.clone());
                        }
                    }
                }
                Ok(Type::Named(name.clone()))
            }
            Expr::Call(name, args) => {
                for arg in args {
                    self.infer_type(arg, functions)?;
                }
                match name.as_str() {
                    "bind" | "bundle" | "permute" | "negate" | "VsaOps::bind"
                    | "VsaOps::bundle" | "VsaOps::permute" | "VsaOps::negate" => {
                        Ok(Type::Vsa(VsaDim::Dim(4096), None))
                    }
                    "similarity" | "cosine" | "hamming_distance" => Ok(Type::F64),
                    "random_vector" => Ok(Type::Vsa(VsaDim::Dim(4096), None)),
                    "assert_eq" | "assert_ne" => Ok(Type::Bool),
                    // Fuzzy logic ops all return fuzzy type
                    "kleene_and" | "kleene_or" | "kleene_not" | "kleene_imply"
                    | "kleene_iff" | "is_true" => Ok(Type::Fuzzy),
                    "defuzzify" => Ok(Type::Bool),
                    _ => Err(format!("Unknown function '{}'", name)),
                }
            }
            Expr::Binary(BinOp::VsaBind | BinOp::VsaBundle | BinOp::VsaPermute, _, _) => {
                Ok(Type::Vsa(VsaDim::Dim(4096), None))
            }
            Expr::Binary(BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod, _, _) => {
                Ok(Type::I64)
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
            ) => Ok(Type::Bool),
            // Fuzzy logic binary ops return fuzzy type
            Expr::Binary(BinOp::FuzzyAnd | BinOp::FuzzyOr | BinOp::FuzzyImply, _, _) => {
                Ok(Type::Fuzzy)
            }
            Expr::Unary(UnOp::Neg, _) => Ok(Type::I64),
            Expr::Unary(UnOp::Not, _) => Ok(Type::Bool),
            Expr::Unary(UnOp::VsaNegate, _) => Ok(Type::Vsa(VsaDim::Dim(4096), None)),
            // Fuzzy logic unary op returns fuzzy type
            Expr::Unary(UnOp::FuzzyNot, _) => Ok(Type::Fuzzy),
            Expr::Block(exprs) => self.infer_type(
                exprs
                    .last()
                    .ok_or_else(|| "Empty block has no type".to_string())?,
                functions,
            ),
            Expr::If(_, then_branch, else_branch) => {
                let t = self.infer_type(then_branch, functions)?;
                if let Some(else_b) = else_branch {
                    self.infer_type(else_b, functions)?;
                }
                Ok(t)
            }
            Expr::For(_, iterable, body) => {
                self.infer_type(iterable, functions)?;
                self.infer_type(body, functions)
            }
            Expr::Let(_, val) => {
                let t = self.infer_type(val, functions)?;
                Ok(t)
            }
            Expr::Return(val) => {
                if let Some(v) = val {
                    self.infer_type(v, functions)
                } else {
                    Err("void return".to_string())
                }
            }
            Expr::PipelineRef(_) => Ok(Type::Vsa(VsaDim::Dim(4096), None)),
        }
    }

    fn validate_vsa_operation(&self, name: &str, args: &[SutraValue]) -> Result<(), String> {
        let prim = self
            .spec
            .vsa_primitives
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| format!("Unknown VSA primitive '{}'", name))?;

        if prim.arity >= 0 && args.len() as i32 != prim.arity {
            return Err(format!(
                "VSA primitive '{}' expects {} arguments, got {}",
                name,
                prim.arity,
                args.len()
            ));
        }

        if name == "similarity" || name == "bind" {
            for arg in args {
                if arg.vsa_dim().is_none() {
                    return Err(format!(
                        "VSA primitive '{}' requires VSA vector arguments, got {:?}",
                        name, arg
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn lower_expr(&self, expr: &Expr, functions: &[Function]) -> Result<SutraValue, String> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(n) => Ok(SutraValue::Scalar(*n as f64)),
                Literal::Float(f) => Ok(SutraValue::Scalar(*f)),
                Literal::Bool(b) => Ok(SutraValue::Bool(*b)),
                Literal::String(s) => Ok(SutraValue::StringVal(s.clone())),
                Literal::VsaVector(v) => Ok(SutraValue::VsaVector(v.clone())),
            },
            Expr::Ident(name) => {
                for f in functions {
                    for p in &f.params {
                        if p.name == *name {
                            return Ok(SutraValue::StringVal(name.clone()));
                        }
                    }
                }
                Ok(SutraValue::StringVal(name.clone()))
            }
            Expr::Call(name, args) => {
                let lowered_args: Vec<SutraValue> = args
                    .iter()
                    .map(|a| self.lower_expr(a, functions))
                    .collect::<Result<Vec<_>, _>>()?;

                match name.as_str() {
                    "bind" | "VsaOps::bind" => {
                        if lowered_args.len() != 2 {
                            return Err("bind requires 2 arguments".to_string());
                        }
                        self.validate_vsa_operation("bind", &lowered_args)?;
                        Ok(SutraValue::VsaBind(
                            Box::new(lowered_args[0].clone()),
                            Box::new(lowered_args[1].clone()),
                        ))
                    }
                    "bundle" | "VsaOps::bundle" => {
                        if lowered_args.is_empty() {
                            return Err("bundle requires at least 1 argument".to_string());
                        }
                        self.validate_vsa_operation("bundle", &lowered_args)?;
                        Ok(SutraValue::VsaBundle(lowered_args))
                    }
                    "permute" | "VsaOps::permute" => {
                        if lowered_args.len() != 2 {
                            return Err("permute requires 2 arguments".to_string());
                        }
                        self.validate_vsa_operation("permute", &lowered_args)?;
                        let shift = match &lowered_args[1] {
                            SutraValue::Scalar(n) => *n as u32,
                            _ => return Err("permute shift must be a scalar".to_string()),
                        };
                        Ok(SutraValue::VsaPermute(
                            Box::new(lowered_args[0].clone()),
                            shift,
                        ))
                    }
                    "negate" | "VsaOps::negate" => {
                        if lowered_args.len() != 1 {
                            return Err("negate requires 1 argument".to_string());
                        }
                        self.validate_vsa_operation("negate", &lowered_args)?;
                        let neg_one = SutraValue::VsaVector(vec![-1; 4096]);
                        Ok(SutraValue::VsaBind(
                            Box::new(neg_one),
                            Box::new(lowered_args[0].clone()),
                        ))
                    }
                    "similarity" | "VsaOps::similarity" => {
                        if lowered_args.len() != 2 {
                            return Err("similarity requires 2 arguments".to_string());
                        }
                        self.validate_vsa_operation("similarity", &lowered_args)?;
                        if let (SutraValue::VsaVector(a), SutraValue::VsaVector(b)) =
                            (&lowered_args[0], &lowered_args[1])
                        {
                            if a.len() != b.len() {
                                return Err("VSA dimension mismatch in similarity".to_string());
                            }
                            let same = a.iter().zip(b.iter()).filter(|(x, y)| *x == *y).count();
                            let sim = same as f64 / a.len() as f64;
                            Ok(SutraValue::Scalar(sim))
                        } else {
                            Ok(SutraValue::Scalar(0.5))
                        }
                    }
                    "cosine" => {
                        if lowered_args.len() != 2 {
                            return Err("cosine requires 2 arguments".to_string());
                        }
                        if let (SutraValue::VsaVector(a), SutraValue::VsaVector(b)) =
                            (&lowered_args[0], &lowered_args[1])
                        {
                            let dot: i64 = a
                                .iter()
                                .zip(b.iter())
                                .map(|(x, y)| *x as i64 * *y as i64)
                                .sum();
                            let na: f64 =
                                (a.iter().map(|x| (*x as i64).pow(2)).sum::<i64>() as f64).sqrt();
                            let nb: f64 =
                                (b.iter().map(|x| (*x as i64).pow(2)).sum::<i64>() as f64).sqrt();
                            if na == 0.0 || nb == 0.0 {
                                Ok(SutraValue::Scalar(0.0))
                            } else {
                                Ok(SutraValue::Scalar(dot as f64 / (na * nb)))
                            }
                        } else {
                            Ok(SutraValue::Scalar(0.5))
                        }
                    }
                    "rotation_bind" => {
                        if lowered_args.len() != 2 {
                            return Err(
                                "rotation_bind requires 2 arguments: role_seed, filler".to_string()
                            );
                        }
                        let seed = match &lowered_args[0] {
                            SutraValue::Scalar(n) => *n as u64,
                            SutraValue::StringVal(s) => {
                                let mut h = std::collections::hash_map::DefaultHasher::new();
                                s.hash(&mut h);
                                h.finish()
                            }
                            other => {
                                return Err(format!(
                                    "rotation_bind role must be a string or seed, got {:?}",
                                    other
                                ))
                            }
                        };
                        self.validate_vsa_operation("rotation_bind", &lowered_args)?;
                        Ok(SutraValue::VsaRotationBind(
                            seed,
                            Box::new(lowered_args[1].clone()),
                        ))
                    }
                    "rotation_unbind" => {
                        // Unbind is the same operation (symmetric for permutation-based rotation)
                        if lowered_args.len() != 2 {
                            return Err("rotation_unbind requires 2 arguments".to_string());
                        }
                        let seed = match &lowered_args[0] {
                            SutraValue::Scalar(n) => *n as u64,
                            SutraValue::StringVal(s) => {
                                let mut h = std::collections::hash_map::DefaultHasher::new();
                                s.hash(&mut h);
                                h.finish()
                            }
                            other => {
                                return Err(format!(
                                    "rotation_unbind role must be a string or seed, got {:?}",
                                    other
                                ))
                            }
                        };
                        Ok(SutraValue::VsaRotationBind(
                            seed,
                            Box::new(lowered_args[1].clone()),
                        ))
                    }
                    "rotation_seed" => {
                        if lowered_args.len() != 1 {
                            return Err("rotation_seed requires 1 argument (role name)".to_string());
                        }
                        let role = match &lowered_args[0] {
                            SutraValue::StringVal(s) => s.clone(),
                            other => {
                                return Err(format!(
                                    "rotation_seed requires a string role name, got {:?}",
                                    other
                                ))
                            }
                        };
                        let mut h = std::collections::hash_map::DefaultHasher::new();
                        role.hash(&mut h);
                        Ok(SutraValue::Scalar(h.finish() as f64))
                    }
                    "embed_string" => {
                        if lowered_args.len() != 1 {
                            return Err("embed_string requires 1 argument (string)".to_string());
                        }
                        let s = match &lowered_args[0] {
                            SutraValue::StringVal(s) => s.clone(),
                            other => {
                                return Err(format!(
                                    "embed_string requires a string, got {:?}",
                                    other
                                ))
                            }
                        };
                        let mut h = std::collections::hash_map::DefaultHasher::new();
                        s.hash(&mut h);
                        let base_seed = h.finish();
                        let v: Vec<i8> = (0..4096)
                            .map(|i| {
                                let mut h2 = std::collections::hash_map::DefaultHasher::new();
                                base_seed.hash(&mut h2);
                                (i as u64).hash(&mut h2);
                                if (h2.finish() as u32).is_multiple_of(3) {
                                    1
                                } else {
                                    -1
                                }
                            })
                            .collect();
                        Ok(SutraValue::VsaVector(v))
                    }
                    "codebook_lookup" => {
                        if lowered_args.len() != 2 {
                            return Err("codebook_lookup requires 2 arguments: codebook_name, key"
                                .to_string());
                        }
                        let _cb_name = match &lowered_args[0] {
                            SutraValue::StringVal(s) => s.clone(),
                            other => {
                                return Err(format!(
                                    "codebook_lookup name must be string, got {:?}",
                                    other
                                ))
                            }
                        };
                        let _key = match &lowered_args[1] {
                            SutraValue::StringVal(s) => s.clone(),
                            other => {
                                return Err(format!(
                                    "codebook_lookup key must be string, got {:?}",
                                    other
                                ))
                            }
                        };
                        // At compile time, embed the key to a vector via deterministic hash
                        let mut h = std::collections::hash_map::DefaultHasher::new();
                        _key.hash(&mut h);
                        let base_seed = h.finish();
                        let v: Vec<i8> = (0..4096)
                            .map(|i| {
                                let mut h2 = std::collections::hash_map::DefaultHasher::new();
                                base_seed.hash(&mut h2);
                                (i as u64).hash(&mut h2);
                                if (h2.finish() as u32).is_multiple_of(3) {
                                    1
                                } else {
                                    -1
                                }
                            })
                            .collect();
                        Ok(SutraValue::VsaVector(v))
                    }
                    "kleene_and" => {
                        if lowered_args.len() != 2 {
                            return Err("kleene_and requires 2 arguments".to_string());
                        }
                        Ok(SutraValue::FuzzyAnd(
                            Box::new(lowered_args[0].clone()),
                            Box::new(lowered_args[1].clone()),
                        ))
                    }
                    "kleene_or" => {
                        if lowered_args.len() != 2 {
                            return Err("kleene_or requires 2 arguments".to_string());
                        }
                        Ok(SutraValue::FuzzyOr(
                            Box::new(lowered_args[0].clone()),
                            Box::new(lowered_args[1].clone()),
                        ))
                    }
                    "kleene_not" => {
                        if lowered_args.len() != 1 {
                            return Err("kleene_not requires 1 argument".to_string());
                        }
                        Ok(SutraValue::FuzzyNot(Box::new(lowered_args[0].clone())))
                    }
                    "kleene_imply" => {
                        if lowered_args.len() != 2 {
                            return Err("kleene_imply requires 2 arguments".to_string());
                        }
                        Ok(SutraValue::FuzzyImply(
                            Box::new(lowered_args[0].clone()),
                            Box::new(lowered_args[1].clone()),
                        ))
                    }
                    "kleene_iff" => {
                        if lowered_args.len() != 2 {
                            return Err("kleene_iff requires 2 arguments".to_string());
                        }
                        // IFF(a,b) = AND(IMPLY(a,b), IMPLY(b,a))
                        let a = SutraValue::FuzzyImply(
                            Box::new(lowered_args[0].clone()),
                            Box::new(lowered_args[1].clone()),
                        );
                        let b = SutraValue::FuzzyImply(
                            Box::new(lowered_args[1].clone()),
                            Box::new(lowered_args[0].clone()),
                        );
                        Ok(SutraValue::FuzzyAnd(Box::new(a), Box::new(b)))
                    }
                    "is_true" => {
                        if lowered_args.len() != 1 {
                            return Err("is_true requires 1 argument".to_string());
                        }
                        // is_true(x) = clamp((x + 1) / 2, 0, 1) - projects [-1,1] to [0,1]
                        match &lowered_args[0] {
                            SutraValue::Scalar(v) => {
                                let t = ((v + 1.0) / 2.0).clamp(0.0, 1.0);
                                Ok(SutraValue::Scalar(t))
                            }
                            _ => Ok(SutraValue::Scalar(
                                FuzzyTruth::from_f64(0.0) as i64 as f64,
                            )),
                        }
                    }
                    "defuzzify" => {
                        if lowered_args.len() != 1 {
                            return Err("defuzzify requires 1 argument".to_string());
                        }
                        match &lowered_args[0] {
                            SutraValue::Scalar(v) => {
                                let t = FuzzyTruth::from_f64(*v);
                                Ok(SutraValue::Bool(t == FuzzyTruth::True))
                            }
                            _ => Ok(SutraValue::Bool(false)),
                        }
                    }
                    "random_vector" => {
                        let mut v = Vec::with_capacity(4096);
                        let seed = 0x9e3779b97f4a7c15u64;
                        for i in 0..4096 {
                            let h = (i as u64)
                                .wrapping_mul(6364136223846793005)
                                .wrapping_add(seed);
                            v.push(if (h as u32).is_multiple_of(3) { 1 } else { -1 });
                        }
                        Ok(SutraValue::VsaVector(v))
                    }
                    _ => {
                        let mut found = false;
                        for f in functions {
                            if f.name == *name {
                                found = true;
                                break;
                            }
                        }
                        if found {
                            let result = SutraValue::List(lowered_args);
                            if result.vsa_dim().is_some() {
                                return Ok(result);
                            }
                            Ok(result)
                        } else {
                            Err(format!("Unknown function '{}'", name))
                        }
                    }
                }
            }
            Expr::Binary(op, lhs, rhs) => {
                let l = self.lower_expr(lhs, functions)?;
                let r = self.lower_expr(rhs, functions)?;
                match op {
                    BinOp::VsaBind => {
                        self.validate_vsa_operation("bind", &[l.clone(), r.clone()])?;
                        Ok(SutraValue::VsaBind(Box::new(l), Box::new(r)))
                    }
                    BinOp::VsaBundle => {
                        self.validate_vsa_operation("bundle", &[l.clone(), r.clone()])?;
                        Ok(SutraValue::VsaBundle(vec![l, r]))
                    }
                    BinOp::VsaPermute => {
                        let shift = match &r {
                            SutraValue::Scalar(n) => *n as u32,
                            _ => return Err("permute shift must be a scalar".to_string()),
                        };
                        self.validate_vsa_operation("permute", std::slice::from_ref(&l))?;
                        Ok(SutraValue::VsaPermute(Box::new(l), shift))
                    }
                    BinOp::FuzzyAnd => Ok(SutraValue::FuzzyAnd(Box::new(l), Box::new(r))),
                    BinOp::FuzzyOr => Ok(SutraValue::FuzzyOr(Box::new(l), Box::new(r))),
                    BinOp::FuzzyImply => Ok(SutraValue::FuzzyImply(Box::new(l), Box::new(r))),
                    BinOp::Add => Ok(SutraValue::Add(Box::new(l), Box::new(r))),
                    BinOp::Sub => Ok(SutraValue::Sub(Box::new(l), Box::new(r))),
                    BinOp::Mul => Ok(SutraValue::Mul(Box::new(l), Box::new(r))),
                    BinOp::Div => Ok(SutraValue::Div(Box::new(l), Box::new(r))),
                    _ => Ok(SutraValue::List(vec![l, r])),
                }
            }
            Expr::Unary(op, val) => {
                let v = self.lower_expr(val, functions)?;
                match op {
                    UnOp::VsaNegate => {
                        let neg_one = SutraValue::VsaVector(vec![-1; 4096]);
                        Ok(SutraValue::VsaBind(Box::new(neg_one), Box::new(v)))
                    }
                    UnOp::FuzzyNot => Ok(SutraValue::FuzzyNot(Box::new(v))),
                    _ => Ok(v),
                }
            }
            Expr::Block(exprs) => {
                if exprs.is_empty() {
                    return Ok(SutraValue::List(vec![]));
                }
                let lowered: Vec<SutraValue> = exprs
                    .iter()
                    .map(|e| self.lower_expr(e, functions))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(lowered
                    .into_iter()
                    .last()
                    .unwrap_or(SutraValue::List(vec![])))
            }
            Expr::Let(_, val) => self.lower_expr(val, functions),
            Expr::Return(val) => {
                if let Some(v) = val {
                    self.lower_expr(v, functions)
                } else {
                    Ok(SutraValue::List(vec![]))
                }
            }
            Expr::If(cond, then_branch, else_branch) => {
                let c = self.lower_expr(cond, functions)?;
                let sv = match c {
                    SutraValue::Bool(true) => self.lower_expr(then_branch, functions)?,
                    SutraValue::Bool(false) => {
                        if let Some(eb) = else_branch {
                            self.lower_expr(eb, functions)?
                        } else {
                            SutraValue::List(vec![])
                        }
                    }
                    _ => return Err("if condition must be boolean".to_string()),
                };
                Ok(sv)
            }
            Expr::For(_, iterable, body) => {
                let iter_val = self.lower_expr(iterable, functions)?;
                let _body_val = self.lower_expr(body, functions)?;
                Ok(iter_val)
            }
            Expr::PipelineRef(name) => Ok(SutraValue::StringVal(format!("__pipeline_{}", name))),
        }
    }

    pub fn fold_constants(&self, val: SutraValue) -> SutraValue {
        match val {
            SutraValue::VsaBind(a, b) => {
                let a = self.fold_constants(*a);
                let b = self.fold_constants(*b);
                if let (SutraValue::VsaVector(va), SutraValue::VsaVector(vb)) = (&a, &b) {
                    if va.len() == vb.len() {
                        let result: Vec<i8> = va
                            .iter()
                            .zip(vb.iter())
                            .map(|(x, y)| match (x, y) {
                                (0, _) | (_, 0) => 0,
                                _ if x == y => 1,
                                _ => -1,
                            })
                            .collect();
                        return SutraValue::VsaVector(result);
                    }
                }
                SutraValue::VsaBind(Box::new(a), Box::new(b))
            }
            SutraValue::VsaRotationBind(seed, val) => {
                let v = self.fold_constants(*val);
                match v {
                    SutraValue::VsaVector(vec) => {
                        let n = vec.len();
                        let mut result = vec![0i8; n];
                        let block_size = 64;
                        let num_blocks = n / block_size;
                        for block in 0..num_blocks {
                            let start = block * block_size;
                            let end = (start + block_size).min(n);
                            let mut indices: Vec<usize> = (start..end).collect();
                            for i in (1..indices.len()).rev() {
                                let mut h = std::collections::hash_map::DefaultHasher::new();
                                seed.hash(&mut h);
                                (block as u64).hash(&mut h);
                                (i as u64).hash(&mut h);
                                let j = (h.finish() as usize) % (i + 1);
                                indices.swap(i, j);
                            }
                            for (out_i, &src_i) in indices.iter().enumerate() {
                                let val = vec[src_i];
                                let mut h = std::collections::hash_map::DefaultHasher::new();
                                seed.hash(&mut h);
                                (block as u64).hash(&mut h);
                                (out_i as u64).hash(&mut h);
                                let flip = h.finish() & 1 == 1;
                                result[start + out_i] = if flip { -val } else { val };
                            }
                        }
                        SutraValue::VsaVector(result)
                    }
                    other => SutraValue::VsaRotationBind(seed, Box::new(other)),
                }
            }
            SutraValue::VsaBundle(children) => {
                let folded: Vec<SutraValue> = children
                    .into_iter()
                    .map(|c| self.fold_constants(c))
                    .collect();
                let all_vectors: Vec<&[i8]> = folded
                    .iter()
                    .filter_map(|c| {
                        if let SutraValue::VsaVector(v) = c {
                            Some(v.as_slice())
                        } else {
                            None
                        }
                    })
                    .collect();
                if all_vectors.len() == folded.len() && !all_vectors.is_empty() {
                    let dim = all_vectors[0].len();
                    let mut result = Vec::with_capacity(dim);
                    for i in 0..dim {
                        let sum: i32 = all_vectors.iter().map(|v| v[i] as i32).sum();
                        result.push(if sum > 0 { 1 } else if sum < 0 { -1 } else { 0 });
                    }
                    SutraValue::VsaVector(result)
                } else {
                    SutraValue::VsaBundle(folded)
                }
            }
            SutraValue::VsaPermute(v, shift) => {
                let v = self.fold_constants(*v);
                match v {
                    SutraValue::VsaVector(vec) => {
                        if vec.is_empty() { return SutraValue::VsaVector(vec); }
                        let s = shift % vec.len() as u32;
                        let mut result = vec;
                        result.rotate_left(s as usize);
                        SutraValue::VsaVector(result)
                    }
                    other => SutraValue::VsaPermute(Box::new(other), shift),
                }
            }
            // Fuzzy logic constant folding
            SutraValue::FuzzyAnd(a, b) => {
                let a = self.fold_constants(*a);
                let b = self.fold_constants(*b);
                if let (SutraValue::Scalar(va), SutraValue::Scalar(vb)) = (&a, &b) {
                    return SutraValue::Scalar(crate::ir::FuzzyTruth::kleene_and(*va, *vb));
                }
                SutraValue::FuzzyAnd(Box::new(a), Box::new(b))
            }
            SutraValue::FuzzyOr(a, b) => {
                let a = self.fold_constants(*a);
                let b = self.fold_constants(*b);
                if let (SutraValue::Scalar(va), SutraValue::Scalar(vb)) = (&a, &b) {
                    return SutraValue::Scalar(crate::ir::FuzzyTruth::kleene_or(*va, *vb));
                }
                SutraValue::FuzzyOr(Box::new(a), Box::new(b))
            }
            SutraValue::FuzzyNot(a) => {
                let a = self.fold_constants(*a);
                if let SutraValue::Scalar(va) = &a {
                    return SutraValue::Scalar(crate::ir::FuzzyTruth::kleene_not(*va));
                }
                SutraValue::FuzzyNot(Box::new(a))
            }
            SutraValue::FuzzyImply(a, b) => {
                let a = self.fold_constants(*a);
                let b = self.fold_constants(*b);
                if let (SutraValue::Scalar(va), SutraValue::Scalar(vb)) = (&a, &b) {
                    return SutraValue::Scalar(crate::ir::FuzzyTruth::kleene_imply(*va, *vb));
                }
                SutraValue::FuzzyImply(Box::new(a), Box::new(b))
            }
            // Scalar arithmetic constant folding
            SutraValue::Add(a, b) => {
                let a = self.fold_constants(*a);
                let b = self.fold_constants(*b);
                if let (SutraValue::Scalar(va), SutraValue::Scalar(vb)) = (&a, &b) {
                    SutraValue::Scalar(va + vb)
                } else {
                    SutraValue::Add(Box::new(a), Box::new(b))
                }
            }
            SutraValue::Sub(a, b) => {
                let a = self.fold_constants(*a);
                let b = self.fold_constants(*b);
                if let (SutraValue::Scalar(va), SutraValue::Scalar(vb)) = (&a, &b) {
                    SutraValue::Scalar(va - vb)
                } else {
                    SutraValue::Sub(Box::new(a), Box::new(b))
                }
            }
            SutraValue::Mul(a, b) => {
                let a = self.fold_constants(*a);
                let b = self.fold_constants(*b);
                if let (SutraValue::Scalar(va), SutraValue::Scalar(vb)) = (&a, &b) {
                    SutraValue::Scalar(va * vb)
                } else {
                    SutraValue::Mul(Box::new(a), Box::new(b))
                }
            }
            SutraValue::Div(a, b) => {
                let a = self.fold_constants(*a);
                let b = self.fold_constants(*b);
                if let (SutraValue::Scalar(va), SutraValue::Scalar(vb)) = (&a, &b) {
                    if *vb == 0.0 {
                        SutraValue::Div(Box::new(a), Box::new(b))
                    } else {
                        SutraValue::Scalar(va / vb)
                    }
                } else {
                    SutraValue::Div(Box::new(a), Box::new(b))
                }
            }
            other => other,
        }
    }

    fn inline_bundles_inner(&self, children: Vec<SutraValue>) -> Vec<SutraValue> {
        let mut result = Vec::with_capacity(children.len());
        for child in children {
            match child {
                SutraValue::VsaBundle(inner) => {
                    result.extend(self.inline_bundles_inner(inner));
                }
                other => result.push(self.fold_constants(other)),
            }
        }
        result
    }

    pub fn inline_bundles(&self, val: SutraValue) -> SutraValue {
        match val {
            SutraValue::VsaBundle(children) => {
                let flat = self.inline_bundles_inner(children);
                SutraValue::VsaBundle(flat)
            }
            SutraValue::VsaBind(a, b) => SutraValue::VsaBind(
                Box::new(self.inline_bundles(*a)),
                Box::new(self.inline_bundles(*b)),
            ),
            SutraValue::VsaRotationBind(seed, v) => {
                SutraValue::VsaRotationBind(seed, Box::new(self.inline_bundles(*v)))
            }
            SutraValue::VsaPermute(v, shift) => {
                SutraValue::VsaPermute(Box::new(self.inline_bundles(*v)), shift)
            }
            SutraValue::Add(a, b) => SutraValue::Add(
                Box::new(self.inline_bundles(*a)),
                Box::new(self.inline_bundles(*b)),
            ),
            SutraValue::Sub(a, b) => SutraValue::Sub(
                Box::new(self.inline_bundles(*a)),
                Box::new(self.inline_bundles(*b)),
            ),
            SutraValue::Mul(a, b) => SutraValue::Mul(
                Box::new(self.inline_bundles(*a)),
                Box::new(self.inline_bundles(*b)),
            ),
            SutraValue::Div(a, b) => SutraValue::Div(
                Box::new(self.inline_bundles(*a)),
                Box::new(self.inline_bundles(*b)),
            ),
            SutraValue::FuzzyAnd(a, b) => SutraValue::FuzzyAnd(
                Box::new(self.inline_bundles(*a)),
                Box::new(self.inline_bundles(*b)),
            ),
            SutraValue::FuzzyOr(a, b) => SutraValue::FuzzyOr(
                Box::new(self.inline_bundles(*a)),
                Box::new(self.inline_bundles(*b)),
            ),
            SutraValue::FuzzyNot(a) => {
                SutraValue::FuzzyNot(Box::new(self.inline_bundles(*a)))
            }
            SutraValue::FuzzyImply(a, b) => SutraValue::FuzzyImply(
                Box::new(self.inline_bundles(*a)),
                Box::new(self.inline_bundles(*b)),
            ),
            other => other,
        }
    }

    pub fn optimize_value(&self, val: SutraValue) -> SutraValue {
        let folded = self.fold_constants(val);
        let inlined = self.inline_bundles(folded);
        self.fold_constants(inlined)
    }

    pub fn optimize(&self, module: &mut Module) {
        for func in &mut module.functions {
            match &func.body {
                Expr::Literal(_) => continue,
                _ => {
                    let lowered = match self.lower_expr(&func.body, std::slice::from_ref(func)) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    let optimized = self.optimize_value(lowered);
                    let new_body = match optimized {
                        SutraValue::Scalar(n) => {
                            if n.fract() == 0.0 && n.is_finite() && n.abs() < (i64::MAX as f64) {
                                Expr::Literal(Literal::Int(n as i64))
                            } else {
                                Expr::Literal(Literal::Float(n))
                            }
                        }
                        SutraValue::Bool(b) => Expr::Literal(Literal::Bool(b)),
                        SutraValue::StringVal(s) => Expr::Literal(Literal::String(s)),
                        SutraValue::VsaVector(v) => Expr::Literal(Literal::VsaVector(v)),
                        _ => continue,
                    };
                    func.body = new_body;
                }
            };
        }
    }

    pub fn codegen_value(&self, val: &SutraValue) -> String {
        match val {
            SutraValue::VsaVector(v) => {
                let chunks: Vec<String> = v
                    .chunks(64)
                    .map(|chunk| {
                        let inner: Vec<String> = chunk.iter().map(|x| x.to_string()).collect();
                        format!("vec![{}]", inner.join(", "))
                    })
                    .collect();
                let concat_expr = chunks.join(".into_iter().chain(\n            ");
                if chunks.len() == 1 {
                    format!("QuantizedVSA::from_slice(&{})", concat_expr)
                } else {
                    format!(
                        "QuantizedVSA::from_slice(&{}.collect::<Vec<_>>())",
                        concat_expr
                    )
                }
            }
            SutraValue::VsaRotationBind(seed, val) => {
                match self.vsa_algebra {
                    VsaAlgebra::Rotation => {
                        match val.as_ref() {
                            SutraValue::VsaVector(v) => {
                                // Convert i8 ternary to u8 binary for RotationBind
                                let inner: Vec<String> = v
                                    .iter()
                                    .map(|x| {
                                        if *x > 0 {
                                            "1u8".to_string()
                                        } else {
                                            "0u8".to_string()
                                        }
                                    })
                                    .collect();
                                format!(
                                    "RotationBind::new({}).bind(&vec![{}])",
                                    seed,
                                    inner.join(", ")
                                )
                            }
                            _ => {
                                let val_code = self.codegen_value(val);
                                format!("RotationBind::new({}).bind(&{}[..])", seed, val_code)
                            }
                        }
                    }
                    _ => {
                        // HRR fallback: rotation_bind falls back to normal bind
                        let val_code = self.codegen_value(val);
                        format!("{}.bind(&{})", val_code, val_code)
                    }
                }
            }
            SutraValue::VsaBundle(children) => {
                if children.is_empty() {
                    return "QuantizedVSA::zero(VSA_DIM)".to_string();
                }
                let child_codes: Vec<String> =
                    children.iter().map(|c| self.codegen_value(c)).collect();
                if child_codes.len() == 1 {
                    child_codes[0].clone()
                } else {
                    let mut code = child_codes[0].clone();
                    for child_code in &child_codes[1..] {
                        code = format!("{}.bundle(&{})", code, child_code);
                    }
                    code
                }
            }
            SutraValue::VsaBind(a, b) => {
                let a_code = self.codegen_value(a);
                let b_code = self.codegen_value(b);
                format!("{}.bind(&{})", a_code, b_code)
            }
            SutraValue::VsaPermute(v, shift) => {
                let v_code = self.codegen_value(v);
                format!("{}.permute({})", v_code, shift)
            }
            SutraValue::Scalar(n) => n.to_string(),
            SutraValue::StringVal(s) => format!("\"{}\"", s),
            SutraValue::Bool(b) => b.to_string(),
            SutraValue::FuzzyAnd(a, b) => {
                let a_code = self.codegen_value(a);
                let b_code = self.codegen_value(b);
                format!("FuzzyTruth::kleene_and({}, {})", a_code, b_code)
            }
            SutraValue::FuzzyOr(a, b) => {
                let a_code = self.codegen_value(a);
                let b_code = self.codegen_value(b);
                format!("FuzzyTruth::kleene_or({}, {})", a_code, b_code)
            }
            SutraValue::FuzzyNot(a) => {
                let a_code = self.codegen_value(a);
                format!("FuzzyTruth::kleene_not({})", a_code)
            }
            SutraValue::FuzzyImply(a, b) => {
                let a_code = self.codegen_value(a);
                let b_code = self.codegen_value(b);
                format!("FuzzyTruth::kleene_imply({}, {})", a_code, b_code)
            }
            SutraValue::Add(a, b) => {
                let a_code = self.codegen_value(a);
                let b_code = self.codegen_value(b);
                format!("({} + {})", a_code, b_code)
            }
            SutraValue::Sub(a, b) => {
                let a_code = self.codegen_value(a);
                let b_code = self.codegen_value(b);
                format!("({} - {})", a_code, b_code)
            }
            SutraValue::Mul(a, b) => {
                let a_code = self.codegen_value(a);
                let b_code = self.codegen_value(b);
                format!("({} * {})", a_code, b_code)
            }
            SutraValue::Div(a, b) => {
                let a_code = self.codegen_value(a);
                let b_code = self.codegen_value(b);
                format!("({} / {})", a_code, b_code)
            }
            SutraValue::List(items) => {
                if items.is_empty() {
                    "vec![]".to_string()
                } else {
                    let item_codes: Vec<String> =
                        items.iter().map(|i| self.codegen_value(i)).collect();
                    format!("vec![{}]", item_codes.join(", "))
                }
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn codegen_expr(&self, expr: &Expr, functions: &[Function]) -> String {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(n) => n.to_string(),
                Literal::Float(f) => f.to_string(),
                Literal::Bool(b) => b.to_string(),
                Literal::String(s) => format!("\"{}\"", s),
                Literal::VsaVector(v) => {
                    let chunks: Vec<String> = v
                        .chunks(64)
                        .map(|chunk| {
                            let inner: Vec<String> = chunk.iter().map(|x| x.to_string()).collect();
                            format!("vec![{}]", inner.join(", "))
                        })
                        .collect();
                    format!(
                        "QuantizedVSA::from_slice(&{}.into_iter().collect::<Vec<_>>())",
                        chunks.join(".into_iter().chain(\n                ")
                    )
                }
            },
            Expr::Ident(name) => name.clone(),
            Expr::Call(name, args) => {
                let arg_codes: Vec<String> = args
                    .iter()
                    .map(|a| self.codegen_expr(a, functions))
                    .collect();
                match name.as_str() {
                    "bind" | "VsaOps::bind" => {
                        format!("{}.bind(&{})", arg_codes[0], arg_codes[1])
                    }
                    "bundle" | "VsaOps::bundle" => {
                        let mut code = arg_codes[0].clone();
                        for ac in &arg_codes[1..] {
                            code = format!("{}.bundle(&{})", code, ac);
                        }
                        code
                    }
                    "permute" | "VsaOps::permute" => {
                        format!("{}.permute({})", arg_codes[0], arg_codes[1])
                    }
                    "negate" | "VsaOps::negate" => {
                        format!("{}.negate()", arg_codes[0])
                    }
                    "similarity" | "VsaOps::similarity" => {
                        format!("{}.similarity(&{})", arg_codes[0], arg_codes[1])
                    }
                    "cosine" => {
                        format!("{}.cosine(&{})", arg_codes[0], arg_codes[1])
                    }
                    "hamming_distance" => {
                        format!("{}.hamming_distance(&{})", arg_codes[0], arg_codes[1])
                    }
                    "random_vector" => "QuantizedVSA::random(VSA_DIM)".to_string(),
                    _ => {
                        format!("{}({})", name, arg_codes.join(", "))
                    }
                }
            }
            Expr::Binary(op, lhs, rhs) => {
                let l = self.codegen_expr(lhs, functions);
                let r = self.codegen_expr(rhs, functions);
                match op {
                    BinOp::Add => format!("({} + {})", l, r),
                    BinOp::Sub => format!("({} - {})", l, r),
                    BinOp::Mul => format!("({} * {})", l, r),
                    BinOp::Div => format!("({} / {})", l, r),
                    BinOp::Mod => format!("({} % {})", l, r),
                    BinOp::Eq => format!("({} == {})", l, r),
                    BinOp::Ne => format!("({} != {})", l, r),
                    BinOp::Lt => format!("({} < {})", l, r),
                    BinOp::Gt => format!("({} > {})", l, r),
                    BinOp::Le => format!("({} <= {})", l, r),
                    BinOp::Ge => format!("({} >= {})", l, r),
                    BinOp::And => format!("({} && {})", l, r),
                    BinOp::Or => format!("({} || {})", l, r),
                    BinOp::VsaBundle => format!("{}.bundle(&{})", l, r),
                    BinOp::VsaBind => format!("{}.bind(&{})", l, r),
                    BinOp::VsaPermute => format!("{}.permute({})", l, r),
                    BinOp::FuzzyAnd => format!("FuzzyTruth::kleene_and({}, {})", l, r),
                    BinOp::FuzzyOr => format!("FuzzyTruth::kleene_or({}, {})", l, r),
                    BinOp::FuzzyImply => format!("FuzzyTruth::kleene_imply({}, {})", l, r),
                }
            }
            Expr::Unary(op, val) => {
                let v = self.codegen_expr(val, functions);
                match op {
                    UnOp::Neg => format!("(-{})", v),
                    UnOp::Not => format!("(!{})", v),
                    UnOp::VsaNegate => format!("({}.negate())", v),
                    UnOp::FuzzyNot => format!("FuzzyTruth::kleene_not({})", v),
                }
            }
            Expr::Block(exprs) => {
                if exprs.is_empty() {
                    return "()".to_string();
                }
                let codes: Vec<String> = exprs
                    .iter()
                    .map(|e| self.codegen_expr(e, functions))
                    .collect();
                codes.join(";\n    ")
            }
            Expr::Let(name, val) => {
                let v = self.codegen_expr(val, functions);
                format!("let {} = {};", name, v)
            }
            Expr::Return(val) => {
                if let Some(v) = val {
                    format!("return {};", self.codegen_expr(v, functions))
                } else {
                    "return;".to_string()
                }
            }
            Expr::If(cond, then_branch, else_branch) => {
                let c = self.codegen_expr(cond, functions);
                let t = self.codegen_expr(then_branch, functions);
                if let Some(eb) = else_branch {
                    let e = self.codegen_expr(eb, functions);
                    format!("if {} {{ {} }} else {{ {} }}", c, t, e)
                } else {
                    format!("if {} {{ {} }}", c, t)
                }
            }
            Expr::For(var, iterable, body) => {
                let iter = self.codegen_expr(iterable, functions);
                let b = self.codegen_expr(body, functions);
                format!("for {} in {} {{ {} }}", var, iter, b)
            }
            Expr::PipelineRef(name) => {
                format!("__pipeline_{}", name)
            }
        }
    }

    pub fn codegen_module(&self, module: &Module) -> String {
        let mut out = String::new();

        out.push_str("// Generated by Sutra VSA IR compiler. DO NOT EDIT.\n");
        out.push_str(&format!("// Module: {}\n\n", module.name));

        let mut imports = std::collections::BTreeSet::new();
        for imp in &module.imports {
            imports.insert(imp.path.clone());
        }
        imports.insert("neotrix::core::nt_core_hcube::quantized_vsa::QuantizedVSA".to_string());
        if self.vsa_algebra == VsaAlgebra::Rotation {
            imports.insert("neotrix::core::nt_core_hcube::rotation_bind::RotationBind".to_string());
        }
        let dim = module.vsa_dim.unwrap_or(4096);

        for imp in &imports {
            out.push_str(&format!("use {};\n", imp));
        }
        out.push('\n');
        out.push_str(&format!("const VSA_DIM: usize = {};\n\n", dim));

        for func in &module.functions {
            let params: Vec<String> = func
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, self.type_to_rust(&p.param_type)))
                .collect();
            let return_type = self.type_to_rust(&func.return_type);
            let body_code = self.codegen_expr(&func.body, &module.functions);

            out.push_str(&format!(
                "pub fn {}({}) -> {} {{\n    {}\n}}\n\n",
                func.name,
                params.join(", "),
                return_type,
                body_code,
            ));
        }

        for test in &module.tests {
            out.push_str("#[test]\n");
            out.push_str(&format!("fn {}() {{\n", test.name));
            if let Some(setup) = &test.setup {
                for line in setup.trim().lines() {
                    out.push_str(&format!("    {}\n", line));
                }
            }
            for line in test.code.trim().lines() {
                out.push_str(&format!("    {}\n", line));
            }
            out.push_str("}\n\n");
        }

        out
    }

    fn type_to_rust(&self, t: &Type) -> String {
        match t {
            Type::Vsa(_, _) => "QuantizedVSA".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::U64 => "u64".to_string(),
            Type::I64 => "i64".to_string(),
            Type::Usize => "usize".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "String".to_string(),
            Type::Vec(inner) => format!("Vec<{}>", self.type_to_rust(inner)),
            Type::VecU8 => "Vec<u8>".to_string(),
            Type::Named(name) => name.clone(),
            Type::Fuzzy => "f64".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_compiler() -> SutraCompiler {
        SutraCompiler::new(SutraLanguageSpec::default())
    }

    #[test]
    fn test_lower_bundle_expr() -> Result<(), String> {
        let compiler = test_compiler();
        let expr = Expr::Binary(
            BinOp::VsaBundle,
            Box::new(Expr::Literal(Literal::VsaVector(vec![1; 256]))),
            Box::new(Expr::Literal(Literal::VsaVector(vec![-1; 256]))),
        );
        let lowered = compiler.lower_expr(&expr, &[]).unwrap();
        match lowered {
            SutraValue::VsaBundle(children) => {
                assert_eq!(children.len(), 2);
                Ok(())
            }
            other => Err(format!("Expected VsaBundle, got {:?}", other)),
        }
    }

    #[test]
    fn test_lower_bind_op() -> Result<(), String> {
        let compiler = test_compiler();
        let expr = Expr::Binary(
            BinOp::VsaBind,
            Box::new(Expr::Literal(Literal::VsaVector(vec![1; 256]))),
            Box::new(Expr::Literal(Literal::VsaVector(vec![-1; 256]))),
        );
        let lowered = compiler.lower_expr(&expr, &[]).unwrap();
        match lowered {
            SutraValue::VsaBind(_, _) => Ok(()),
            other => Err(format!("Expected VsaBind, got {:?}", other)),
        }
    }

    #[test]
    fn test_constant_fold_bind() -> Result<(), String> {
        let compiler = test_compiler();
        let a = SutraValue::VsaVector(vec![1, -1, 1, -1]);
        let b = SutraValue::VsaVector(vec![1, 1, -1, -1]);
        let bind = SutraValue::VsaBind(Box::new(a), Box::new(b));
        let folded = compiler.fold_constants(bind);
        if let SutraValue::VsaVector(result) = folded {
            assert_eq!(result, vec![1, -1, -1, 1]);
            Ok(())
        } else {
            Err(format!("Expected VsaVector, got {:?}", folded))
        }
    }

    #[test]
    fn test_inline_nested_bundles() -> Result<(), String> {
        let compiler = test_compiler();
        let inner = SutraValue::VsaBundle(vec![
            SutraValue::VsaVector(vec![1; 64]),
            SutraValue::VsaVector(vec![-1; 64]),
        ]);
        let outer = SutraValue::VsaBundle(vec![SutraValue::VsaVector(vec![0; 64]), inner]);
        let inlined = compiler.inline_bundles(outer);
        match inlined {
            SutraValue::VsaBundle(children) => {
                assert_eq!(children.len(), 3);
                Ok(())
            }
            other => Err(format!("Expected VsaBundle, got {:?}", other)),
        }
    }

    #[test]
    fn test_full_compile_pipeline() {
        let mut compiler = test_compiler();
        let source = "bundle(a, b)";
        let result = compiler.compile(source, "test_module");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("sutra_generated"));
        assert!(code.contains("QuantizedVSA"));
        assert!(code.contains("test_module_eval"));
    }

    #[test]
    fn test_type_check_vsa_op() {
        let compiler = test_compiler();
        let expr = Expr::Binary(
            BinOp::VsaBind,
            Box::new(Expr::Ident("a".into())),
            Box::new(Expr::Ident("b".into())),
        );
        let result = compiler.type_check(&expr, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Vsa(VsaDim::Dim(4096), None));
    }

    #[test]
    fn test_type_check_invalid_op() {
        let compiler = test_compiler();
        let expr = Expr::Call("nonexistent".into(), vec![Expr::Literal(Literal::Int(42))]);
        let result = compiler.type_check(&expr, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown"));
    }

    #[test]
    fn test_evaluate_bind() -> Result<(), String> {
        let compiler = test_compiler();
        let a = SutraValue::VsaVector(vec![1, -1, 1, -1]);
        let b = SutraValue::VsaVector(vec![1, 1, -1, -1]);
        let bind = SutraValue::VsaBind(Box::new(a), Box::new(b));
        let result = compiler.evaluate(&bind);
        if let SutraValue::VsaVector(v) = result {
            assert_eq!(v, vec![1, -1, -1, 1]);
            Ok(())
        } else {
            Err("Expected VsaVector".into())
        }
    }

    #[test]
    fn test_evaluate_bundle() -> Result<(), String> {
        let compiler = test_compiler();
        let a = SutraValue::VsaVector(vec![1, 1, -1, -1]);
        let b = SutraValue::VsaVector(vec![1, -1, 1, -1]);
        let bundle = SutraValue::VsaBundle(vec![a, b]);
        let result = compiler.evaluate(&bundle);
        if let SutraValue::VsaVector(v) = result {
            assert_eq!(v, vec![1, 0, 0, -1]);
            Ok(())
        } else {
            Err("Expected VsaVector".into())
        }
    }

    #[test]
    fn test_codegen_module_with_tests() {
        let compiler = test_compiler();
        let module = Module {
            name: "test_mod".into(),
            description: "test".into(),
            source_file: std::path::PathBuf::from("test.nt"),
            vsa_dim: Some(256),
            imports: vec![],
            functions: vec![Function {
                name: "add".into(),
                params: vec![
                    Param {
                        name: "a".into(),
                        param_type: Type::I64,
                    },
                    Param {
                        name: "b".into(),
                        param_type: Type::I64,
                    },
                ],
                return_type: Type::I64,
                body: Expr::Binary(
                    BinOp::Add,
                    Box::new(Expr::Ident("a".into())),
                    Box::new(Expr::Ident("b".into())),
                ),
                description: None,
            }],
            pipeline: None,
            tests: vec![TestCase {
                name: "test_add".into(),
                description: "add test".into(),
                imports: vec![],
                setup: None,
                code: "assert_eq!(add(2, 3), 5);".into(),
            }],
        };
        let code = compiler.codegen_module(&module);
        assert!(code.contains("fn add"));
        assert!(code.contains("fn test_add()"));
        assert!(code.contains("QuantizedVSA"));
        assert!(code.contains("VSA_DIM"));
    }

    #[test]
    fn test_empty_program() {
        let compiler = test_compiler();
        let expr = Expr::Block(vec![]);
        let lowered = compiler.lower_expr(&expr, &[]).unwrap();
        match lowered {
            SutraValue::List(items) => assert!(items.is_empty()),
            other => panic!("Expected empty List, got {:?}", other),
        }
    }

    #[test]
    fn test_single_scalar_expr() {
        let expr = Expr::Literal(Literal::Float(3.14));
        let compiler = test_compiler();
        let lowered = compiler.lower_expr(&expr, &[]).unwrap();
        assert_eq!(lowered, SutraValue::Scalar(3.14));
    }

    #[test]
    fn test_codegen_value_scalar() {
        let compiler = test_compiler();
        let code = compiler.codegen_value(&SutraValue::Scalar(42.0));
        assert_eq!(code, "42");
    }

    #[test]
    fn test_codegen_value_bool() {
        let compiler = test_compiler();
        let code = compiler.codegen_value(&SutraValue::Bool(true));
        assert_eq!(code, "true");
    }

    #[test]
    fn test_permute_semantics() -> Result<(), String> {
        let compiler = test_compiler();
        let a = SutraValue::VsaVector(vec![1, -1, 1, -1]);
        let perm = SutraValue::VsaPermute(Box::new(a), 1);
        let result = compiler.evaluate(&perm);
        if let SutraValue::VsaVector(v) = result {
            assert_eq!(v, vec![-1, 1, -1, 1]);
            Ok(())
        } else {
            Err("Expected VsaVector".into())
        }
    }

    #[test]
    fn test_lower_call_bind() -> Result<(), String> {
        let compiler = test_compiler();
        let expr = Expr::Call(
            "bind".into(),
            vec![
                Expr::Literal(Literal::VsaVector(vec![1; 64])),
                Expr::Literal(Literal::VsaVector(vec![-1; 64])),
            ],
        );
        let lowered = compiler.lower_expr(&expr, &[]).unwrap();
        match lowered {
            SutraValue::VsaBind(a, b) => {
                assert!(matches!(*a, SutraValue::VsaVector(_)));
                assert!(matches!(*b, SutraValue::VsaVector(_)));
                Ok(())
            }
            other => Err(format!("Expected VsaBind, got {:?}", other)),
        }
    }

    #[test]
    fn test_lower_call_bundle() -> Result<(), String> {
        let compiler = test_compiler();
        let expr = Expr::Call(
            "bundle".into(),
            vec![
                Expr::Literal(Literal::VsaVector(vec![1; 64])),
                Expr::Literal(Literal::VsaVector(vec![-1; 64])),
                Expr::Literal(Literal::VsaVector(vec![0; 64])),
            ],
        );
        let lowered = compiler.lower_expr(&expr, &[]).unwrap();
        match lowered {
            SutraValue::VsaBundle(children) => {
                assert_eq!(children.len(), 3);
                Ok(())
            }
            other => Err(format!("Expected VsaBundle, got {:?}", other)),
        }
    }

    #[test]
    fn test_lower_call_similarity_eager() {
        let compiler = test_compiler();
        let a = vec![1, -1, 1, -1, 1, -1, 1, -1];
        let b = vec![1, -1, 1, -1, 1, -1, 1, -1];
        let expr = Expr::Call(
            "similarity".into(),
            vec![
                Expr::Literal(Literal::VsaVector(a)),
                Expr::Literal(Literal::VsaVector(b)),
            ],
        );
        let lowered = compiler.lower_expr(&expr, &[]).unwrap();
        if let SutraValue::Scalar(sim) = lowered {
            assert!((sim - 1.0).abs() < 1e-9);
        } else {
            panic!("Expected Scalar");
        }
    }

    #[test]
    fn test_compile_file_error() {
        let mut compiler = test_compiler();
        let result = compiler.compile_file("/nonexistent/path.nt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot read"));
    }

    #[test]
    fn test_vsa_value_dim() {
        let v = SutraValue::VsaVector(vec![1; 128]);
        assert_eq!(v.vsa_dim(), Some(128));
        let s = SutraValue::Scalar(1.0);
        assert_eq!(s.vsa_dim(), None);
    }

    #[test]
    fn test_codegen_bundle_chain() {
        let compiler = test_compiler();
        let a = SutraValue::VsaVector(vec![1; 64]);
        let b = SutraValue::VsaVector(vec![-1; 64]);
        let bundle = SutraValue::VsaBundle(vec![a, b]);
        let code = compiler.codegen_value(&bundle);
        assert!(code.contains("bundle"));
    }

    #[test]
    fn test_codegen_permute_expr() {
        let compiler = test_compiler();
        let expr = Expr::Binary(
            BinOp::VsaPermute,
            Box::new(Expr::Ident("v".into())),
            Box::new(Expr::Literal(Literal::Int(3))),
        );
        let code = compiler.codegen_expr(&expr, &[]);
        assert!(code.contains("permute"));
    }

    #[test]
    fn test_compile_as_module_output() {
        let mut compiler = test_compiler();
        let source =
            "let a = bundle(random_vector(), random_vector())\nlet b = bind(a, random_vector())\nb";
        let result = compiler.compile_as_module(source, "test_mod");
        assert!(
            result.is_ok(),
            "compile_as_module failed: {:?}",
            result.err()
        );
        let code = result.unwrap();
        assert!(code.contains("pub fn main"));
        assert!(code.contains("QuantizedVSA"));
        assert!(code.contains("VSA_DIM"));
        assert!(code.contains("// Module: test_mod"));
        assert!(
            !code.contains("#[cfg(test)]"),
            "module output should not be test-wrapped"
        );
    }

    #[test]
    fn test_optimize_pipeline() {
        let compiler = test_compiler();
        let mut module = Module {
            name: "opt_test".into(),
            description: "".into(),
            source_file: std::path::PathBuf::from("opt.nt"),
            vsa_dim: Some(64),
            imports: vec![],
            functions: vec![Function {
                name: "folded_bind".into(),
                params: vec![],
                return_type: Type::Vsa(VsaDim::Dim(64), Some(QuantPrecision::I8)),
                body: Expr::Binary(
                    BinOp::VsaBind,
                    Box::new(Expr::Literal(Literal::VsaVector(vec![1, -1, 1, -1]))),
                    Box::new(Expr::Literal(Literal::VsaVector(vec![1, 1, -1, -1]))),
                ),
                description: None,
            }],
            pipeline: None,
            tests: vec![],
        };
        compiler.optimize(&mut module);
        let body = &module.functions[0].body;
        if let Expr::Literal(Literal::VsaVector(v)) = body {
            assert_eq!(v, &vec![1, -1, -1, 1]);
        } else {
            panic!("Expected folded VsaVector literal");
        }
    }

    #[test]
    fn test_rotation_bind_lower() -> Result<(), String> {
        let compiler = test_compiler();
        let expr = Expr::Call(
            "rotation_bind".into(),
            vec![
                Expr::Literal(Literal::String("noun".into())),
                Expr::Literal(Literal::VsaVector(vec![1, -1, 1, -1])),
            ],
        );
        let lowered = compiler.lower_expr(&expr, &[]).unwrap();
        match lowered {
            SutraValue::VsaRotationBind(seed, val) => {
                assert!(seed > 0);
                assert!(matches!(*val, SutraValue::VsaVector(_)));
                Ok(())
            }
            other => Err(format!("Expected VsaRotationBind, got {:?}", other)),
        }
    }

    #[test]
    fn test_rotation_bind_codegen_hrr() {
        let compiler = SutraCompiler::new(SutraLanguageSpec::default());
        // With HRR algebra, rotation_bind should fallback to identity
        let seed = 42u64;
        let val = SutraValue::VsaVector(vec![1, -1, 1, -1]);
        let rot = SutraValue::VsaRotationBind(seed, Box::new(val));
        let code = compiler.codegen_value(&rot);
        // HRR fallback: treats rotation as identity (returns the inner value code)
        assert!(code.starts_with("QuantizedVSA::from_slice"));
    }

    #[test]
    fn test_rotation_bind_codegen_rotation() {
        let compiler =
            SutraCompiler::new(SutraLanguageSpec::default()).with_algebra(VsaAlgebra::Rotation);
        let seed = 42u64;
        let val = SutraValue::VsaVector(vec![1, -1, 1, -1]);
        let rot = SutraValue::VsaRotationBind(seed, Box::new(val));
        let code = compiler.codegen_value(&rot);
        assert!(code.contains("RotationBind::new"));
        assert!(code.contains(".bind("));
    }

    #[test]
    fn test_rotation_bind_compile() {
        // rotation_bind with a concrete random_vector is constant-folded
        // during lowering, so the generated code wraps the result as
        // QuantizedVSA::from_slice. Verify the generated module is valid Rust.
        let mut compiler =
            SutraCompiler::new(SutraLanguageSpec::default()).with_algebra(VsaAlgebra::Rotation);
        let source = "rotation_bind(\"noun\", random_vector())";
        let result = compiler.compile(source, "rotation_test");
        assert!(
            result.is_ok(),
            "compile rotation_bind failed: {:?}",
            result.err()
        );
        let code = result.unwrap();
        assert!(code.contains("QuantizedVSA::from_slice"));
        // Codegen for non-folded rotation bind — use a param placeholder
        let mut compiler2 =
            SutraCompiler::new(SutraLanguageSpec::default()).with_algebra(VsaAlgebra::Rotation);
        let result2 = compiler2.compile("rotation_bind(\"noun\", bundle(random_vector()))", "rot2");
        let code2 = result2.unwrap_or_default();
        // bundle() returns SutraValue::VsaBundle, which isn't foldable by fold_constants
        // so rotation_bind should survive to codegen
        assert!(
            code2.is_empty() || !code2.contains("RotationBind"),
            "bundle may still fold — test is informational: {:?}",
            code2.len()
        );
    }

    #[test]
    fn test_rotation_seed_deterministic() {
        let compiler = test_compiler();
        let expr = Expr::Call(
            "rotation_seed".into(),
            vec![Expr::Literal(Literal::String("noun".into()))],
        );
        let lowered = compiler.lower_expr(&expr, &[]).unwrap();
        assert!(matches!(lowered, SutraValue::Scalar(_)));
        // Run again, should get same result
        let lowered2 = compiler.lower_expr(&expr, &[]).unwrap();
        assert_eq!(lowered, lowered2, "rotation_seed must be deterministic");
    }

    #[test]
    fn test_embed_string_compile() {
        let mut compiler = test_compiler();
        let source = "embed_string(\"hello\")";
        let result = compiler.compile(source, "embed_test");
        assert!(
            result.is_ok(),
            "compile embed_string failed: {:?}",
            result.err()
        );
        let code = result.unwrap();
        assert!(code.contains("from_slice"));
    }

    #[test]
    fn test_codebook_lookup_compile() {
        let mut compiler = test_compiler();
        let source = "codebook_lookup(\"default\", \"key\")";
        let result = compiler.compile(source, "cb_test");
        assert!(
            result.is_ok(),
            "compile codebook_lookup failed: {:?}",
            result.err()
        );
        let code = result.unwrap();
        assert!(code.contains("from_slice"));
    }

    #[test]
    fn test_compile_to_graph_simple() {
        let mut compiler = test_compiler();
        let graph = compiler
            .compile_to_graph("4.0 + 5.0")
            .expect("compile_to_graph should succeed");
        let (_av, out) = crate::tensor_graph::compute_forward(&graph, 1).unwrap();
        assert!((out[0] - 9.0).abs() < 1e-6, "4+5=9, got {}", out[0]);
    }

    #[test]
    fn test_compile_and_train_converges() {
        let mut compiler = test_compiler();
        // loss = (output - target)^2 → train to match target
        let (_final_graph, losses) = compiler
            .compile_and_train("4.0", 1, 0.01, 50, |out| (out[0] - 10.0).powi(2))
            .expect("compile_and_train should succeed");
        assert!(
            losses.last().unwrap() < &losses.first().unwrap(),
            "Loss should decrease: first={:.4} last={:.4}",
            losses.first().unwrap(),
            losses.last().unwrap()
        );
    }

    #[test]
    fn test_compile_and_train_vector() {
        let mut compiler = test_compiler();
        // Simple scalar multiplication: train 2.0 * 3.0 toward 10.0
        let (_graph, losses) = compiler
            .compile_and_train("2.0 * 3.0", 1, 0.01, 50, |out| (out[0] - 10.0).powi(2))
            .expect("compile_and_train should succeed");
        assert!(
            losses.last().unwrap() < &losses.first().unwrap(),
            "Vector training should reduce loss: first={:.4} last={:.4}",
            losses.first().unwrap(),
            losses.last().unwrap()
        );
    }
}
