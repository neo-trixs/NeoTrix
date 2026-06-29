/// Internal representation of nt-lang IR types.
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestSuite {
    pub name: String,
    pub description: String,
    pub source_file: PathBuf,
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    pub imports: Vec<String>,
    pub setup: Option<String>,
    pub code: String,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub description: String,
    pub source_file: PathBuf,
    pub vsa_dim: Option<usize>,
    pub imports: Vec<Import>,
    pub functions: Vec<Function>,
    pub pipeline: Option<Pipeline>,
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Expr,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub name: String,
    pub stages: Vec<PipelineStage>,
    pub input_type: Option<Type>,
    pub output_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub name: String,
    pub input: Type,
    pub output: Type,
    pub function_ref: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
    Call(String, Vec<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Block(Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    For(String, Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>),
    Return(Option<Box<Expr>>),
    PipelineRef(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    VsaBind,
    VsaBundle,
    VsaPermute,
    /// Fuzzy Kleene AND (Lagrange-interpolated, C^∞)
    FuzzyAnd,
    /// Fuzzy Kleene OR (Lagrange-interpolated, C^∞)
    FuzzyOr,
    /// Fuzzy material implication: IMPLY(a,b) = OR(NOT(a), b)
    FuzzyImply,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
    VsaNegate,
    /// Fuzzy Kleene NOT: NOT(x) = -x
    FuzzyNot,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    VsaVector(Vec<i8>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Vsa(VsaDim, Option<QuantPrecision>),
    F32,
    F64,
    U64,
    I64,
    Usize,
    Bool,
    String,
    Vec(Box<Type>),
    VecU8,
    Named(String),
    /// Fuzzy truth value in [-1, +1], Kleene 3-valued
    Fuzzy,
}

/// Symbolic VSA dimension expression for compile-time dimension tracking.
/// Enables algebraic dimension inference through computation graphs.
#[derive(Debug, Clone, PartialEq)]
pub enum VsaDim {
    /// Concrete dimension value
    Dim(usize),
    /// Named parameter (e.g., from module config or function param)
    Param(String),
    /// Sum: a + b
    Add(Box<VsaDim>, Box<VsaDim>),
    /// Product: a * b
    Mul(Box<VsaDim>, Box<VsaDim>),
    /// Modulo: a % b
    Mod(Box<VsaDim>, Box<VsaDim>),
    /// Maximum: max(a, b)
    Max(Box<VsaDim>, Box<VsaDim>),
}

impl VsaDim {
    /// Resolve to a concrete dimension value using the provided parameter bindings.
    /// Returns None if any symbolic variable is unresolved.
    pub fn resolve(&self, params: &std::collections::HashMap<String, usize>) -> Option<usize> {
        match self {
            VsaDim::Dim(v) => Some(*v),
            VsaDim::Param(name) => params.get(name).copied(),
            VsaDim::Add(a, b) => Some(a.resolve(params)? + b.resolve(params)?),
            VsaDim::Mul(a, b) => Some(a.resolve(params)? * b.resolve(params)?),
            VsaDim::Mod(a, b) => Some(a.resolve(params)? % b.resolve(params)?),
            VsaDim::Max(a, b) => Some(a.resolve(params)?.max(b.resolve(params)?)),
        }
    }

    /// Return the concrete dimension if this is a literal Dim, or None for symbolic forms.
    pub fn as_usize(&self) -> Option<usize> {
        match self {
            VsaDim::Dim(v) => Some(*v),
            _ => None,
        }
    }

    /// Return a human-readable string representation.
    pub fn display(&self) -> String {
        match self {
            VsaDim::Dim(v) => v.to_string(),
            VsaDim::Param(name) => name.clone(),
            VsaDim::Add(a, b) => format!("({}+{})", a.display(), b.display()),
            VsaDim::Mul(a, b) => format!("({}*{})", a.display(), b.display()),
            VsaDim::Mod(a, b) => format!("({}%{})", a.display(), b.display()),
            VsaDim::Max(a, b) => format!("max({},{})", a.display(), b.display()),
        }
    }
}

impl From<usize> for VsaDim {
    fn from(v: usize) -> Self {
        VsaDim::Dim(v)
    }
}

impl Type {
    pub fn name(&self) -> &str {
        match self {
            Type::Vsa(_, _) => "Vsa",
            Type::F32 => "F32",
            Type::F64 => "F64",
            Type::U64 => "U64",
            Type::I64 => "I64",
            Type::Usize => "Usize",
            Type::Bool => "Bool",
            Type::String => "String",
            Type::Vec(_) => "Vec",
            Type::VecU8 => "VecU8",
            Type::Named(name) => name.as_str(),
            Type::Fuzzy => "Fuzzy",
        }
    }

    /// Check if two types are compatible for binary operations.
    /// For VSA types, dimensions must resolve to the same concrete value.
    pub fn compatible(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Vsa(d1, _), Type::Vsa(d2, _)) => {
                // Both concrete: direct comparison
                if let (Some(a), Some(b)) = (d1.as_usize(), d2.as_usize()) {
                    return a == b;
                }
                // Either symbolic: unify by name — same Param name means same dim
                // For now, symbolics are compatible if structurally equal
                d1 == d2
            }
            (Type::Fuzzy, Type::Fuzzy) => true,
            (Type::Fuzzy, Type::Bool) | (Type::Bool, Type::Fuzzy) => true,
            (a, b) => a.name() == b.name(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuantPrecision {
    I8,
    I16,
    I32,
    F32,
    F64,
}

/// VSA algebra selection — enables multiple dispatch on bind/bundle/similarity.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Default)]
pub enum VsaAlgebra {
    /// FFT-HRR circular convolution (current default)
    #[default]
    Hrr,
    /// Rotation binding (Sutra-style, arXiv:2605.20919)
    Rotation,
    /// Binary spatter code (XOR bundle, majority sum)
    Bsc,
    /// MAP (multiply-add-permute, bipolar vectors)
    Map,
}

/// Fuzzy truth value in Kleene 3-valued logic [-1, 0, +1].
/// Used for Sutra-style polynomial fuzzy logic (arXiv:2605.20919).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FuzzyTruth {
    True = 1,
    Unknown = 0,
    False = -1,
}

impl FuzzyTruth {
    pub fn from_f64(v: f64) -> Self {
        if v > 0.5 {
            FuzzyTruth::True
        } else if v < -0.5 {
            FuzzyTruth::False
        } else {
            FuzzyTruth::Unknown
        }
    }

    /// Lagrange-interpolated Kleene AND (C^∞ everywhere, exact on {-1,0,+1}²):
    /// AND(a,b) = (a + b + ab - a² - b² + a²b²) / 2
    pub fn kleene_and(a: f64, b: f64) -> f64 {
        (a + b + a * b - a * a - b * b + a * a * b * b) / 2.0
    }

    /// Lagrange-interpolated Kleene OR:
    /// OR(a,b) = (a + b - ab + a² + b² - a²b²) / 2
    pub fn kleene_or(a: f64, b: f64) -> f64 {
        (a + b - a * b + a * a + b * b - a * a * b * b) / 2.0
    }

    /// Kleene NOT: NOT(x) = -x
    pub fn kleene_not(a: f64) -> f64 {
        -a
    }

    /// Material implication: IMPLY(a,b) = OR(NOT(a), b)
    pub fn kleene_imply(a: f64, b: f64) -> f64 {
        Self::kleene_or(Self::kleene_not(a), b)
    }

    /// Biconditional: IFF(a,b) = AND(IMPLY(a,b), IMPLY(b,a))
    pub fn kleene_iff(a: f64, b: f64) -> f64 {
        Self::kleene_and(Self::kleene_imply(a, b), Self::kleene_imply(b, a))
    }
}


impl VsaAlgebra {
    pub fn name(&self) -> &str {
        match self {
            VsaAlgebra::Hrr => "hrr",
            VsaAlgebra::Rotation => "rotation",
            VsaAlgebra::Bsc => "bsc",
            VsaAlgebra::Map => "map",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "hrr" | "fft-hrr" => Some(Self::Hrr),
            "rotation" | "sutra" => Some(Self::Rotation),
            "bsc" => Some(Self::Bsc),
            "map" => Some(Self::Map),
            _ => None,
        }
    }
}
