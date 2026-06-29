use std::collections::HashMap;

/// A comptime-evaluated expression or block
#[derive(Debug, Clone)]
pub struct ComptimeBlock {
    pub source: String,
    pub block_type: ComptimeBlockType,
    pub result: Option<ComptimeValue>,
    pub errors: Vec<String>,
    pub evaluated: bool,
    pub eval_duration_ns: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComptimeBlockType {
    Expression,
    FunctionCall,
    TypeGeneration,
    DataGeneration,
    Assertion,
    LoopUnroll,
}

#[derive(Debug, Clone)]
pub enum ComptimeValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Type(String),
    Array(Vec<ComptimeValue>),
    Struct(HashMap<String, ComptimeValue>),
    Function(String),
    Empty,
}

#[derive(Debug, Clone)]
pub struct BuiltinFn {
    pub name: String,
    pub arg_count: usize,
    pub description: String,
}

pub struct NeComptimeEngine {
    pub blocks: Vec<ComptimeBlock>,
    pub generated_code: String,
    pub const_foldings: u64,
    eval_depth: usize,
    max_depth: usize,
    builtins: HashMap<String, BuiltinFn>,
}

impl NeComptimeEngine {
    pub fn new() -> Self {
        let mut builtins = HashMap::new();
        builtins.insert(
            "len".into(),
            BuiltinFn {
                name: "len".into(),
                arg_count: 1,
                description: "return array length".into(),
            },
        );
        builtins.insert(
            "type_of".into(),
            BuiltinFn {
                name: "type_of".into(),
                arg_count: 1,
                description: "return type name as string".into(),
            },
        );
        builtins.insert(
            "assert".into(),
            BuiltinFn {
                name: "assert".into(),
                arg_count: 2,
                description: "compile-time assertion".into(),
            },
        );
        builtins.insert(
            "panic".into(),
            BuiltinFn {
                name: "panic".into(),
                arg_count: 1,
                description: "emit compile error".into(),
            },
        );
        builtins.insert(
            "int_to_float".into(),
            BuiltinFn {
                name: "int_to_float".into(),
                arg_count: 1,
                description: "convert int to float".into(),
            },
        );
        builtins.insert(
            "generate_name".into(),
            BuiltinFn {
                name: "generate_name".into(),
                arg_count: 2,
                description: "generate unique name".into(),
            },
        );
        NeComptimeEngine {
            blocks: Vec::new(),
            generated_code: String::new(),
            const_foldings: 0,
            eval_depth: 0,
            max_depth: 64,
            builtins,
        }
    }

    pub fn evaluate_block(
        &mut self,
        source: &str,
        block_type: ComptimeBlockType,
    ) -> Result<ComptimeValue, String> {
        if self.eval_depth >= self.max_depth {
            return Err("comptime recursion limit (64) exceeded".into());
        }
        let start = std::time::Instant::now();
        self.eval_depth += 1;

        let result = match &block_type {
            ComptimeBlockType::Expression => eval_expression(source),
            ComptimeBlockType::FunctionCall => self.eval_builtin_call(source),
            ComptimeBlockType::Assertion => {
                let cond = eval_expression(source)?;
                match cond {
                    ComptimeValue::Bool(true) => Ok(ComptimeValue::Empty),
                    ComptimeValue::Bool(false) => Err("comptime assertion failed".into()),
                    _ => Err("comptime assert requires bool expression".into()),
                }
            }
            _ => {
                self.generated_code
                    .push_str(&format!("// comptime {} block\n", {
                        match block_type {
                            ComptimeBlockType::TypeGeneration => "type_gen",
                            ComptimeBlockType::DataGeneration => "data_gen",
                            ComptimeBlockType::LoopUnroll => "loop_unroll",
                            _ => "unknown",
                        }
                    }));
                Ok(ComptimeValue::Empty)
            }
        };

        self.eval_depth -= 1;
        let elapsed = start.elapsed().as_nanos() as u64;
        let (val, errs) = match &result {
            Ok(v) => (Some(v.clone()), vec![]),
            Err(e) => (None, vec![e.clone()]),
        };
        self.blocks.push(ComptimeBlock {
            source: source.to_string(),
            block_type,
            result: val,
            errors: errs,
            evaluated: result.is_ok(),
            eval_duration_ns: elapsed,
        });
        result
    }

    fn eval_builtin_call(&mut self, source: &str) -> Result<ComptimeValue, String> {
        let src = source.trim();
        let paren_open = src
            .find('(')
            .ok_or_else(|| "expected '(' in call".to_string())?;
        let name = src[..paren_open].trim();
        let inner = &src[paren_open + 1..];
        let paren_close = inner
            .rfind(')')
            .ok_or_else(|| "expected ')' in call".to_string())?;
        let args_str = inner[..paren_close].trim();

        let builtin = self
            .builtins
            .get(name)
            .ok_or_else(|| format!("unknown builtin: {}", name))?;
        let args: Vec<&str> = if args_str.is_empty() {
            vec![]
        } else {
            args_str.split(',').map(|s| s.trim()).collect()
        };
        if args.len() != builtin.arg_count && builtin.name == "assert" && args.len() < 1 {
            return Err(format!("{} needs 1-2 arguments", name));
        }
        if args.len() > builtin.arg_count {
            return Err(format!(
                "{} expects {} arguments, got {}",
                name,
                builtin.arg_count,
                args.len()
            ));
        }

        match name {
            "len" => {
                let val = eval_expression(args[0])?;
                match val {
                    ComptimeValue::String(s) => Ok(ComptimeValue::Int(s.len() as i64)),
                    ComptimeValue::Array(a) => Ok(ComptimeValue::Int(a.len() as i64)),
                    _ => Err("len requires string or array".into()),
                }
            }
            "type_of" => {
                let val = eval_expression(args[0])?;
                let tname = match val {
                    ComptimeValue::Int(_) => "int",
                    ComptimeValue::Float(_) => "float",
                    ComptimeValue::Bool(_) => "bool",
                    ComptimeValue::String(_) => "string",
                    ComptimeValue::Type(_) => "type",
                    ComptimeValue::Array(_) => "array",
                    ComptimeValue::Struct(_) => "struct",
                    ComptimeValue::Function(_) => "function",
                    ComptimeValue::Empty => "empty",
                };
                Ok(ComptimeValue::String(tname.into()))
            }
            "assert" => {
                let cond = eval_expression(args[0])?;
                match cond {
                    ComptimeValue::Bool(true) => Ok(ComptimeValue::Empty),
                    ComptimeValue::Bool(false) => {
                        let msg = if args.len() > 1 {
                            match eval_expression(args[1])? {
                                ComptimeValue::String(s) => s,
                                other => format!("{:?}", other),
                            }
                        } else {
                            "assertion failed".into()
                        };
                        Err(msg)
                    }
                    _ => Err("assert requires bool".into()),
                }
            }
            "panic" => {
                let msg = eval_expression(args[0])?;
                let text = match msg {
                    ComptimeValue::String(s) => s,
                    other => format!("{:?}", other),
                };
                Err(text)
            }
            "int_to_float" => {
                let val = eval_expression(args[0])?;
                match val {
                    ComptimeValue::Int(i) => Ok(ComptimeValue::Float(i as f64)),
                    _ => Err("int_to_float requires int".into()),
                }
            }
            "generate_name" => {
                let base = match eval_expression(args[0])? {
                    ComptimeValue::String(s) => s,
                    _ => "generated".into(),
                };
                let seed = match eval_expression(args[1])? {
                    ComptimeValue::Int(i) => i,
                    ComptimeValue::Float(f) => f as i64,
                    _ => 0,
                };
                Ok(ComptimeValue::String(format!("{}_{}", base, seed)))
            }
            _ => Err(format!("unknown builtin: {}", name)),
        }
    }

    pub fn fold_constants(&mut self, ir_code: &str) -> String {
        let mut result = ir_code.to_string();
        for block in &self.blocks {
            if let Some(ref val) = block.result {
                let placeholder = format!("$comptime({})", block.source);
                let replacement = match val {
                    ComptimeValue::Int(i) => i.to_string(),
                    ComptimeValue::Float(f) => f.to_string(),
                    ComptimeValue::Bool(b) => b.to_string(),
                    ComptimeValue::String(s) => format!("\"{}\"", s),
                    _ => continue,
                };
                if result.contains(&placeholder) {
                    result = result.replace(&placeholder, &replacement);
                    self.const_foldings += 1;
                }
            }
        }
        result
    }

    pub fn generate_comptime_ir(&self) -> String {
        let mut out = String::new();
        out.push_str("; comptime blocks\n");
        for (i, block) in self.blocks.iter().enumerate() {
            let r = match &block.result {
                Some(v) => format!("{:?}", v),
                None => "[error]".into(),
            };
            out.push_str(&format!(
                "  comptime[{}] {:?}: {} -> {} ({}ns)\n",
                i, block.block_type, block.source, r, block.eval_duration_ns,
            ));
        }
        out
    }

    pub fn block_report(&self) -> Vec<(u64, String, bool, u64)> {
        self.blocks
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let preview = if b.source.len() > 40 {
                    format!("{}...", &b.source[..37])
                } else {
                    b.source.clone()
                };
                (i as u64, preview, b.evaluated, b.eval_duration_ns)
            })
            .collect()
    }
}

/// Walk source looking for `$comptime(expr)` patterns, evaluate each, replace with result.
/// On evaluation error, emits `/* comptime_error: msg */` comment in place.
/// Handles nested parentheses via depth counting.
pub fn preprocess_comptime(source: &str) -> String {
    let mut result = String::new();
    let mut rest = source;
    while let Some(start) = rest.find("$comptime(") {
        result.push_str(&rest[..start]);
        let inner_start = start + 10;
        let after = &rest[inner_start..];
        let mut depth: u32 = 1;
        let mut end = 0;
        for (i, c) in after.chars().enumerate() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth == 0 {
            let expr = &after[..end];
            match eval_expression(expr) {
                Ok(val) => {
                    let replacement = match val {
                        ComptimeValue::Int(i) => i.to_string(),
                        ComptimeValue::Float(f) => f.to_string(),
                        ComptimeValue::Bool(b) => b.to_string(),
                        ComptimeValue::String(s) => s,
                        ComptimeValue::Type(t) => t,
                        ComptimeValue::Empty => String::new(),
                        other => format!("{:?}", other),
                    };
                    result.push_str(&replacement);
                }
                Err(e) => {
                    result.push_str(&format!("/* comptime_error: {} */", e));
                }
            }
            rest = &after[end + 1..];
        } else {
            result.push_str("$comptime(");
            rest = after;
        }
    }
    result.push_str(rest);
    result
}

/// Minimal comptime expression evaluator
pub fn eval_expression(expr: &str) -> Result<ComptimeValue, String> {
    let e = expr.trim();
    if e == "true" {
        return Ok(ComptimeValue::Bool(true));
    }
    if e == "false" {
        return Ok(ComptimeValue::Bool(false));
    }
    if e == "needs_llvm" {
        return Ok(ComptimeValue::String("self-hosted backend".into()));
    }
    if e.starts_with('"') {
        if !e.ends_with('"') {
            return Err("unterminated string literal".into());
        }
        return Ok(ComptimeValue::String(e[1..e.len() - 1].to_string()));
    }
    if let Ok(i) = e.parse::<i64>() {
        return Ok(ComptimeValue::Int(i));
    }
    if let Ok(f) = e.parse::<f64>() {
        return Ok(ComptimeValue::Float(f));
    }

    let op_pos = e
        .find("+")
        .or_else(|| e.find('-'))
        .or_else(|| e.find('*'))
        .or_else(|| e.find('/'))
        .or_else(|| e.find("=="));
    if let Some(pos) = op_pos {
        let (left, rest) = e.split_at(pos);
        let op = rest
            .chars()
            .next()
            .ok_or_else(|| format!("comptime: empty expression after operator in '{}'", e))?;
        let right = rest[1..].trim();
        let left_val = eval_expression(left.trim())?;
        let right_val = eval_expression(right)?;
        return match op {
            '+' => add_values(&left_val, &right_val),
            '-' => sub_values(&left_val, &right_val),
            '*' => mul_values(&left_val, &right_val),
            '/' => div_values(&left_val, &right_val),
            _ => Err(format!(
                "comptime eval not available at compile time for: {}",
                expr
            )),
        };
    }

    Err(format!(
        "comptime eval not available at compile time for: {}",
        expr
    ))
}

fn add_values(a: &ComptimeValue, b: &ComptimeValue) -> Result<ComptimeValue, String> {
    match (a, b) {
        (ComptimeValue::Int(x), ComptimeValue::Int(y)) => Ok(ComptimeValue::Int(x + y)),
        (ComptimeValue::Float(x), ComptimeValue::Float(y)) => Ok(ComptimeValue::Float(x + y)),
        (ComptimeValue::String(x), ComptimeValue::String(y)) => {
            Ok(ComptimeValue::String(format!("{}{}", x, y)))
        }
        (ComptimeValue::Int(x), ComptimeValue::Float(y)) => Ok(ComptimeValue::Float(*x as f64 + y)),
        (ComptimeValue::Float(x), ComptimeValue::Int(y)) => Ok(ComptimeValue::Float(x + *y as f64)),
        _ => Err("type mismatch in add".into()),
    }
}

fn sub_values(a: &ComptimeValue, b: &ComptimeValue) -> Result<ComptimeValue, String> {
    match (a, b) {
        (ComptimeValue::Int(x), ComptimeValue::Int(y)) => Ok(ComptimeValue::Int(x - y)),
        (ComptimeValue::Float(x), ComptimeValue::Float(y)) => Ok(ComptimeValue::Float(x - y)),
        (ComptimeValue::Int(x), ComptimeValue::Float(y)) => Ok(ComptimeValue::Float(*x as f64 - y)),
        (ComptimeValue::Float(x), ComptimeValue::Int(y)) => Ok(ComptimeValue::Float(x - *y as f64)),
        _ => Err("type mismatch in sub".into()),
    }
}

fn mul_values(a: &ComptimeValue, b: &ComptimeValue) -> Result<ComptimeValue, String> {
    match (a, b) {
        (ComptimeValue::Int(x), ComptimeValue::Int(y)) => Ok(ComptimeValue::Int(x * y)),
        (ComptimeValue::Float(x), ComptimeValue::Float(y)) => Ok(ComptimeValue::Float(x * y)),
        (ComptimeValue::Int(x), ComptimeValue::Float(y)) => Ok(ComptimeValue::Float(*x as f64 * y)),
        (ComptimeValue::Float(x), ComptimeValue::Int(y)) => Ok(ComptimeValue::Float(x * *y as f64)),
        _ => Err("type mismatch in mul".into()),
    }
}

fn div_values(a: &ComptimeValue, b: &ComptimeValue) -> Result<ComptimeValue, String> {
    match (a, b) {
        (ComptimeValue::Int(x), ComptimeValue::Int(y)) => {
            if *y == 0 {
                return Err("division by zero".into());
            }
            Ok(ComptimeValue::Int(x / y))
        }
        (ComptimeValue::Float(x), ComptimeValue::Float(y)) => {
            if *y == 0.0 {
                return Err("division by zero".into());
            }
            Ok(ComptimeValue::Float(x / y))
        }
        (ComptimeValue::Int(x), ComptimeValue::Float(y)) => {
            if *y == 0.0 {
                return Err("division by zero".into());
            }
            Ok(ComptimeValue::Float(*x as f64 / y))
        }
        (ComptimeValue::Float(x), ComptimeValue::Int(y)) => {
            if *y == 0 {
                return Err("division by zero".into());
            }
            Ok(ComptimeValue::Float(x / *y as f64))
        }
        _ => Err("type mismatch in div".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_integer_expression() {
        let val = eval_expression("2 + 2").unwrap();
        match val {
            ComptimeValue::Int(n) => assert_eq!(n, 4),
            _ => panic!("expected Int"),
        }
    }

    #[test]
    fn test_evaluate_string_concat() {
        let val = eval_expression(r#""hello" + " world""#).unwrap();
        match val {
            ComptimeValue::String(s) => assert_eq!(s, "hello world"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn test_builtin_assert_passes() {
        let mut engine = NeComptimeEngine::new();
        let result = engine.evaluate_block("true", ComptimeBlockType::Assertion);
        assert!(result.is_ok());
    }

    #[test]
    fn test_builtin_assert_fails() {
        let mut engine = NeComptimeEngine::new();
        let result = engine.evaluate_block("false", ComptimeBlockType::Assertion);
        assert!(result.is_err());
    }

    #[test]
    fn test_const_folding() {
        let mut engine = NeComptimeEngine::new();
        let _ = engine.evaluate_block("2 + 2", ComptimeBlockType::Expression);
        let ir = "let x = $comptime(2 + 2);";
        let folded = engine.fold_constants(ir);
        assert_eq!(folded, "let x = 4;");
        assert_eq!(engine.const_foldings, 1);
    }

    #[test]
    fn test_block_report() {
        let mut engine = NeComptimeEngine::new();
        let _ = engine.evaluate_block("1 + 1", ComptimeBlockType::Expression);
        let _ = engine.evaluate_block("42", ComptimeBlockType::Expression);
        let report = engine.block_report();
        assert_eq!(report.len(), 2);
        assert!(report[0].2); // evaluated
    }

    #[test]
    fn test_max_depth_prevents_infinite_recursion() {
        let mut engine = NeComptimeEngine::new();
        engine.max_depth = 2;
        // Simulate recursion by manually nesting eval_depth
        engine.eval_depth = 2;
        let result = engine.evaluate_block("42", ComptimeBlockType::Expression);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("recursion limit"));
    }

    #[test]
    fn test_builtin_len_string() {
        let mut engine = NeComptimeEngine::new();
        let val = engine
            .evaluate_block(r#"len("hello")"#, ComptimeBlockType::FunctionCall)
            .unwrap();
        match val {
            ComptimeValue::Int(n) => assert_eq!(n, 5),
            _ => panic!("expected Int"),
        }
    }

    #[test]
    fn test_builtin_panic() {
        let mut engine = NeComptimeEngine::new();
        let result = engine.evaluate_block(r#"panic("oops")"#, ComptimeBlockType::FunctionCall);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "oops");
    }

    #[test]
    fn test_mul_and_div() {
        let val = eval_expression("10 * 5").unwrap();
        assert!(matches!(val, ComptimeValue::Int(50)));
        let val = eval_expression("100 / 3").unwrap();
        assert!(matches!(val, ComptimeValue::Int(33)));
    }

    #[test]
    fn test_needs_llvm_easter_egg() {
        let val = eval_expression("needs_llvm").unwrap();
        match val {
            ComptimeValue::String(s) => assert_eq!(s, "self-hosted backend"),
            _ => panic!("expected String"),
        }
    }

    // ── preprocess_comptime tests ──

    #[test]
    fn test_comptime_int_expr() {
        let out = preprocess_comptime("let x = $comptime(42)");
        assert_eq!(out, "let x = 42");
    }

    #[test]
    fn test_comptime_string_expr() {
        let out = preprocess_comptime("$comptime(\"hello\")");
        assert_eq!(out, "hello");
    }

    #[test]
    fn test_comptime_bool_expr() {
        let out = preprocess_comptime("if $comptime(true)");
        assert_eq!(out, "if true");
    }

    #[test]
    fn test_comptime_error_handling() {
        let out = preprocess_comptime("$comptime(bad_expr)");
        assert!(out.starts_with("/* comptime_error:"), "got: {}", out);
        assert!(out.ends_with("*/"), "got: {}", out);
    }

    #[test]
    fn test_comptime_arithmetic() {
        let out = preprocess_comptime("let y = $comptime(2 + 2)");
        assert_eq!(out, "let y = 4");
    }

    #[test]
    fn test_comptime_no_match() {
        let out = preprocess_comptime("let x = 42");
        assert_eq!(out, "let x = 42");
    }

    #[test]
    fn test_comptime_multiple_blocks() {
        let out = preprocess_comptime("a = $comptime(1), b = $comptime(true)");
        assert_eq!(out, "a = 1, b = true");
    }
}
