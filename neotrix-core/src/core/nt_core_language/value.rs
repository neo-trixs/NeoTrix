use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum NeValue {
    Nil,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Vsa(Vec<u8>),
    List(Vec<NeValue>),
    Lambda(Vec<String>, Vec<usize>),
    Primitive(String),
    Exports(HashMap<String, NeValue>),
    TestResult {
        passed: u64,
        failed: u64,
        total: u64,
        assert_count: u64,
        coverage: u64,
    },
}

impl Default for NeValue {
    fn default() -> Self {
        Self::Nil
    }
}

impl fmt::Display for NeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Float(x) => write!(f, "{x}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Str(s) => write!(f, "\"{s}\""),
            Self::Vsa(v) => write!(f, "<vsa:{}b>", v.len()),
            Self::List(xs) => {
                write!(f, "[")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{x}")?;
                }
                write!(f, "]")
            }
            Self::Lambda(params, _) => write!(f, "<lambda({})>", params.join(" ")),
            Self::Primitive(name) => write!(f, "<prim:{name}>"),
            Self::Exports(map) => write!(f, "<exports:{}>", map.len()),
            Self::TestResult {
                passed,
                failed,
                total,
                assert_count,
                coverage,
            } => {
                write!(
                    f,
                    "<test:{}p/{}f/{}t/{}a/{}c>",
                    passed, failed, total, assert_count, coverage
                )
            }
        }
    }
}

impl NeValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nil | Self::Bool(false) => false,
            Self::Int(n) => *n != 0,
            Self::Float(x) => *x != 0.0,
            _ => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Nil => "nil",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
            Self::Str(_) => "str",
            Self::Vsa(_) => "vsa",
            Self::List(_) => "list",
            Self::Lambda(_, _) => "lambda",
            Self::Primitive(_) => "primitive",
            Self::Exports(_) => "exports",
            Self::TestResult { .. } => "test-result",
        }
    }
}
