/// NeIR — canonical S-expression intermediate representation.
/// All surface syntaxes parse into this AST.
#[derive(Debug, Clone, PartialEq)]
pub enum NeExpr {
    // ── 8 Core Primitives (The Octad) ──
    Reflect(Option<String>),
    Curious(Box<NeExpr>),
    Dream(Box<NeExpr>),
    Edit(Box<EditDirective>),
    Bind(Box<NeExpr>, Box<NeExpr>),
    Bundle(Vec<NeExpr>),
    Permute(Box<NeExpr>, Box<NeExpr>),
    Similarity(Box<NeExpr>, Box<NeExpr>),

    // ── Function / Module ──
    Module {
        name: String,
        imports: Vec<String>,
        body: Vec<NeExpr>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Box<NeExpr>,
        return_type: Option<String>,
    },
    Call(String, Vec<NeExpr>),

    // ── Values ──
    LitVector(Vec<u64>),
    LitString(String),
    LitFloat(f64),
    LitInt(i64),
    Var(String),
    Let(String, Box<NeExpr>, Box<NeExpr>),

    // ── Control flow (compiled away at compile time) ──
    Match(Box<NeExpr>, Vec<(NeExpr, NeExpr)>, Option<Box<NeExpr>>),
    Seq(Vec<NeExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditDirective {
    pub target: Vec<String>,
    pub value: Box<NeExpr>,
    pub guard: Option<Box<NeExpr>>,
}

impl NeExpr {
    /// Pretty-print as canonical S-expression
    pub fn to_sexpr(&self) -> String {
        match self {
            NeExpr::Reflect(None) => "(reflect)".to_string(),
            NeExpr::Reflect(Some(r)) => format!("(reflect \"{}\")", r),
            NeExpr::Curious(x) => format!("(curious {})", x.to_sexpr()),
            NeExpr::Dream(x) => format!("(dream {})", x.to_sexpr()),
            NeExpr::Edit(e) => {
                let target = e.target.join(".");
                let guard_part = match &e.guard {
                    Some(g) => format!(" {}", g.to_sexpr()),
                    None => String::new(),
                };
                format!("(edit {} {}{})", target, e.value.to_sexpr(), guard_part)
            }
            NeExpr::Bind(a, b) => format!("(bind {} {})", a.to_sexpr(), b.to_sexpr()),
            NeExpr::Bundle(items) => {
                let inner: Vec<String> = items.iter().map(|i| i.to_sexpr()).collect();
                format!("(bundle {})", inner.join(" "))
            }
            NeExpr::Permute(a, b) => format!("(permute {} {})", a.to_sexpr(), b.to_sexpr()),
            NeExpr::Similarity(a, b) => format!("(similarity {} {})", a.to_sexpr(), b.to_sexpr()),
            NeExpr::Module {
                name,
                imports,
                body,
            } => {
                let imports_str = imports.join(" ");
                let body_str: Vec<String> = body.iter().map(|e| e.to_sexpr()).collect();
                format!(
                    "(module {} (import {}) (seq {}))",
                    name,
                    imports_str,
                    body_str.join(" ")
                )
            }
            NeExpr::Function {
                name,
                params,
                body,
                return_type: _,
            } => {
                let params_str = params.join(" ");
                format!("(fn {} ({}) {})", name, params_str, body.to_sexpr())
            }
            NeExpr::Call(name, args) => {
                let args_str: Vec<String> = args.iter().map(|a| a.to_sexpr()).collect();
                format!("({} {})", name, args_str.join(" "))
            }
            NeExpr::LitVector(items) => {
                let inner: Vec<String> = items.iter().map(|i| format!("{}", i)).collect();
                format!("[{}]", inner.join(" "))
            }
            NeExpr::LitString(s) => format!("\"{}\"", s),
            NeExpr::LitFloat(f) => format!("{}", f),
            NeExpr::LitInt(i) => format!("{}", i),
            NeExpr::Var(v) => v.clone(),
            NeExpr::Let(name, value, body) => {
                format!("(let {} {} {})", name, value.to_sexpr(), body.to_sexpr())
            }
            NeExpr::Match(scrutinee, arms, default) => {
                let arms_str: Vec<String> = arms
                    .iter()
                    .map(|(p, e)| format!("({} {})", p.to_sexpr(), e.to_sexpr()))
                    .collect();
                let arms_joined = arms_str.join(" ");
                match default {
                    Some(d) => format!(
                        "(match {} {} (default {}))",
                        scrutinee.to_sexpr(),
                        arms_joined,
                        d.to_sexpr()
                    ),
                    None => format!("(match {} {})", scrutinee.to_sexpr(), arms_joined),
                }
            }
            NeExpr::Seq(items) => {
                let inner: Vec<String> = items.iter().map(|i| i.to_sexpr()).collect();
                format!("(seq {})", inner.join(" "))
            }
        }
    }
}
