use super::value::NeValue;

fn walk_vars_dyn(expr: &NeExpr, f: &mut dyn FnMut(&str)) {
    match expr {
        NeExpr::Var(v) => f(v),
        NeExpr::Call(_, args) => args.iter().for_each(|a| walk_vars_dyn(a, f)),
        NeExpr::Bind(a, b) => {
            walk_vars_dyn(a, f);
            walk_vars_dyn(b, f);
        }
        NeExpr::Bundle(xs) => xs.iter().for_each(|x| walk_vars_dyn(x, f)),
        NeExpr::Negate(x) => walk_vars_dyn(x, f),
        NeExpr::Permute(x) => walk_vars_dyn(x, f),
        NeExpr::Similarity(a, b) => {
            walk_vars_dyn(a, f);
            walk_vars_dyn(b, f);
        }
        NeExpr::If(c, t, e) => {
            walk_vars_dyn(c, f);
            walk_vars_dyn(t, f);
            walk_vars_dyn(e, f);
        }
        NeExpr::Let(_, val, body) => {
            walk_vars_dyn(val, f);
            walk_vars_dyn(body, f);
        }
        NeExpr::Seq(xs) => xs.iter().for_each(|a| walk_vars_dyn(a, f)),
        NeExpr::Lambda(params, body) => {
            let mut collected = Vec::new();
            walk_vars_dyn(body, &mut |v| collected.push(v.to_string()));
            for v in &collected {
                if !params.contains(v) {
                    f(v);
                }
            }
        }
        NeExpr::Literal(_) => {}
        NeExpr::Loop {
            body,
            max_iters: _,
            counter_name: _,
        } => {
            walk_vars_dyn(body, f);
        }
        NeExpr::LoopExpr {
            var: _,
            init,
            condition,
            body,
        } => {
            walk_vars_dyn(init, f);
            walk_vars_dyn(condition, f);
            walk_vars_dyn(body, f);
        }
        NeExpr::Match { value, arms } => {
            walk_vars_dyn(value, f);
            for (pat, body) in arms {
                walk_vars_dyn(pat, f);
                walk_vars_dyn(body, f);
            }
        }
        NeExpr::Import { .. } => {}
        NeExpr::Export { name: _, value } => {
            walk_vars_dyn(value, f);
        }
        NeExpr::Assert {
            condition,
            message: _,
            tolerance: _,
        } => {
            walk_vars_dyn(condition, f);
        }
        NeExpr::Test {
            name: _,
            body,
            expected: _,
        } => {
            walk_vars_dyn(body, f);
        }
        NeExpr::Property {
            name: _,
            generator,
            property,
            iterations: _,
        } => {
            walk_vars_dyn(generator, f);
            walk_vars_dyn(property, f);
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum NeExpr {
    Literal(NeValue),
    Var(String),
    Call(String, Vec<NeExpr>),
    Bind(Box<NeExpr>, Box<NeExpr>),
    Bundle(Vec<NeExpr>),
    Negate(Box<NeExpr>),
    Permute(Box<NeExpr>),
    Similarity(Box<NeExpr>, Box<NeExpr>),
    If(Box<NeExpr>, Box<NeExpr>, Box<NeExpr>),
    Let(String, Box<NeExpr>, Box<NeExpr>),
    Seq(Vec<NeExpr>),
    Lambda(Vec<String>, Box<NeExpr>),
    Loop {
        body: Box<NeExpr>,
        max_iters: Option<usize>,
        counter_name: String,
    },
    LoopExpr {
        var: String,
        init: Box<NeExpr>,
        condition: Box<NeExpr>,
        body: Box<NeExpr>,
    },
    Match {
        value: Box<NeExpr>,
        arms: Vec<(NeExpr, NeExpr)>,
    },
    Import {
        path: String,
    },
    Export {
        name: String,
        value: Box<NeExpr>,
    },
    Assert {
        condition: Box<NeExpr>,
        message: String,
        tolerance: f64,
    },
    Test {
        name: String,
        body: Box<NeExpr>,
        expected: bool,
    },
    Property {
        name: String,
        generator: Box<NeExpr>,
        property: Box<NeExpr>,
        iterations: u64,
    },
}

impl NeExpr {
    pub fn walk_vars<F: FnMut(&str)>(&self, f: &mut F) {
        walk_vars_dyn(self, f);
    }
}

pub fn parse_ne(source: &str) -> Result<NeExpr, String> {
    let tokens = tokenize(source)?;
    let mut pos = 0;
    if pos >= tokens.len() {
        return Err("empty source".into());
    }
    parse_expr(&tokens, &mut pos)
}

/// Parse a file containing multiple top-level S-expressions.
/// Wraps them in a `Seq` so all statements execute sequentially.
pub fn parse_file(source: &str) -> Result<NeExpr, String> {
    let tokens = tokenize(source)?;
    let mut pos = 0;
    let mut exprs = Vec::new();
    while pos < tokens.len() {
        exprs.push(parse_expr(&tokens, &mut pos)?);
    }
    if exprs.is_empty() {
        return Err("empty source".into());
    }
    if exprs.len() == 1 {
        Ok(exprs.remove(0))
    } else {
        Ok(NeExpr::Seq(exprs))
    }
}

fn tokenize(source: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else if ch == '(' || ch == ')' || ch == '[' || ch == ']' {
            tokens.push(ch.to_string());
            chars.next();
        } else if ch == ';' {
            while chars.next().map_or(false, |c| c != '\n') {}
        } else if ch == '"' {
            let mut s = String::from('"');
            chars.next();
            loop {
                match chars.next() {
                    None => return Err("unclosed string".into()),
                    Some('"') => {
                        s.push('"');
                        break;
                    }
                    Some('\\') => {
                        s.push('\\');
                        s.push(chars.next().ok_or("truncated escape")?);
                    }
                    Some(c) => s.push(c),
                }
            }
            tokens.push(s);
        } else {
            let mut word = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '(' || c == ')' || c == '[' || c == ']' || c == ';' {
                    break;
                }
                word.push(c);
                chars.next();
            }
            tokens.push(word);
        }
    }
    Ok(tokens)
}

fn parse_expr(tokens: &[String], pos: &mut usize) -> Result<NeExpr, String> {
    if *pos >= tokens.len() {
        return Err("unexpected end".into());
    }
    let tok = &tokens[*pos];
    *pos += 1;

    if tok == "(" {
        let expr = parse_list_expr(tokens, pos)?;
        if *pos >= tokens.len() || tokens[*pos] != ")" {
            return Err("expected )".into());
        }
        *pos += 1;
        Ok(expr)
    } else if tok == ")" {
        Err("unexpected )".into())
    } else if tok == "[" {
        let mut items = Vec::new();
        while *pos < tokens.len() && tokens[*pos] != "]" {
            items.push(parse_expr(tokens, pos)?);
        }
        if *pos >= tokens.len() {
            return Err("unclosed [".into());
        }
        *pos += 1;
        Ok(NeExpr::Literal(NeValue::List(
            items
                .into_iter()
                .map(|e| expr_to_value(&e))
                .collect::<Result<_, _>>()?,
        )))
    } else if !tok.is_empty() && tok.as_bytes()[0] == b'"' {
        let inner = &tok[1..tok.len() - 1];
        Ok(NeExpr::Literal(NeValue::Str(inner.to_string())))
    } else if tok == "true" {
        Ok(NeExpr::Literal(NeValue::Bool(true)))
    } else if tok == "false" {
        Ok(NeExpr::Literal(NeValue::Bool(false)))
    } else if tok == "nil" {
        Ok(NeExpr::Literal(NeValue::Nil))
    } else if let Ok(n) = tok.parse::<i64>() {
        Ok(NeExpr::Literal(NeValue::Int(n)))
    } else if let Ok(x) = tok.parse::<f64>() {
        Ok(NeExpr::Literal(NeValue::Float(x)))
    } else {
        Ok(NeExpr::Var(tok.clone()))
    }
}

fn parse_list_expr(tokens: &[String], pos: &mut usize) -> Result<NeExpr, String> {
    if *pos >= tokens.len() {
        return Err("unexpected end in list".into());
    }
    let head = &tokens[*pos];
    *pos += 1;

    if head == "if" {
        let cond = parse_expr(tokens, pos)?;
        let then = parse_expr(tokens, pos)?;
        let else_ = parse_expr(tokens, pos)?;
        Ok(NeExpr::If(Box::new(cond), Box::new(then), Box::new(else_)))
    } else if head == "let" {
        let name = if *pos < tokens.len() {
            tokens[*pos].clone()
        } else {
            return Err("let needs name".into());
        };
        *pos += 1;
        let val = parse_expr(tokens, pos)?;
        let body = parse_expr(tokens, pos)?;
        Ok(NeExpr::Let(name, Box::new(val), Box::new(body)))
    } else if head == "bind" || head == "xor" {
        let a = parse_expr(tokens, pos)?;
        let b = parse_expr(tokens, pos)?;
        Ok(NeExpr::Bind(Box::new(a), Box::new(b)))
    } else if head == "bundle" || head == "maj" {
        let mut args = Vec::new();
        while *pos < tokens.len() && tokens[*pos] != ")" {
            args.push(parse_expr(tokens, pos)?);
        }
        Ok(NeExpr::Bundle(args))
    } else if head == "neg" {
        let x = parse_expr(tokens, pos)?;
        Ok(NeExpr::Negate(Box::new(x)))
    } else if head == "perm" || head == "rotate" {
        let x = parse_expr(tokens, pos)?;
        Ok(NeExpr::Permute(Box::new(x)))
    } else if head == "sim" || head == "cosine" {
        let a = parse_expr(tokens, pos)?;
        let b = parse_expr(tokens, pos)?;
        Ok(NeExpr::Similarity(Box::new(a), Box::new(b)))
    } else if head == "fn" || head == "lambda" {
        if *pos >= tokens.len() || tokens[*pos] != "[" {
            return Err("lambda needs [params]".into());
        }
        *pos += 1;
        let mut params = Vec::new();
        while *pos < tokens.len() && tokens[*pos] != "]" {
            params.push(tokens[*pos].clone());
            *pos += 1;
        }
        if *pos >= tokens.len() {
            return Err("lambda unclosed ]".into());
        }
        *pos += 1;
        let body = parse_expr(tokens, pos)?;
        Ok(NeExpr::Lambda(params, Box::new(body)))
    } else if head == "do" || head == "begin" {
        let mut exprs = Vec::new();
        while *pos < tokens.len() && tokens[*pos] != ")" {
            exprs.push(parse_expr(tokens, pos)?);
        }
        if exprs.is_empty() {
            return Err("do needs body".into());
        }
        Ok(NeExpr::Seq(exprs))
    } else if head == "loop" {
        let first = parse_expr(tokens, pos)?;
        // Detect new syntax: (loop (<var> <init>) <condition> <body>)
        // vs old syntax: (loop <body> [max_iters])
        let is_old = if *pos >= tokens.len() || tokens[*pos] == ")" {
            true
        } else {
            tokens[*pos].parse::<usize>().is_ok()
        };
        if is_old {
            let max_iters = if *pos < tokens.len() && tokens[*pos] != ")" {
                let tok = &tokens[*pos];
                *pos += 1;
                tok.parse::<usize>().ok()
            } else {
                None
            };
            Ok(NeExpr::Loop {
                body: Box::new(first),
                max_iters,
                counter_name: "__i".into(),
            })
        } else {
            match first {
                NeExpr::Call(name, mut args) if args.len() == 1 => {
                    let init = args.remove(0);
                    let condition = parse_expr(tokens, pos)?;
                    let body = parse_expr(tokens, pos)?;
                    Ok(NeExpr::LoopExpr {
                        var: name,
                        init: Box::new(init),
                        condition: Box::new(condition),
                        body: Box::new(body),
                    })
                }
                _ => Err("invalid loop syntax: expected (loop (<var> <init>) <condition> <body>) or (loop <body> [max_iters])".into()),
            }
        }
    } else if head == "match" {
        let value = parse_expr(tokens, pos)?;
        let mut arms = Vec::new();
        while *pos < tokens.len() && tokens[*pos] != ")" {
            if tokens[*pos] != "(" {
                return Err("match arm must be (pattern body)".into());
            }
            *pos += 1;
            let pat = parse_expr(tokens, pos)?;
            let body = parse_expr(tokens, pos)?;
            if *pos >= tokens.len() || tokens[*pos] != ")" {
                return Err("match arm missing )".into());
            }
            *pos += 1;
            arms.push((pat, body));
        }
        if arms.is_empty() {
            return Err("match needs at least one arm".into());
        }
        Ok(NeExpr::Match {
            value: Box::new(value),
            arms,
        })
    } else if head == "import" {
        if *pos >= tokens.len() {
            return Err("import needs path".into());
        }
        let path_tok = &tokens[*pos];
        *pos += 1;
        if !path_tok.starts_with('"') {
            return Err("import path must be a string".into());
        }
        let path = path_tok[1..path_tok.len() - 1].to_string();
        Ok(NeExpr::Import { path })
    } else if head == "export" {
        if *pos >= tokens.len() {
            return Err("export needs name".into());
        }
        let name = tokens[*pos].clone();
        *pos += 1;
        let value = parse_expr(tokens, pos)?;
        Ok(NeExpr::Export {
            name,
            value: Box::new(value),
        })
    } else if head == "assert" {
        let condition = parse_expr(tokens, pos)?;
        let message = if *pos < tokens.len() && tokens[*pos] != ")" {
            let msg_tok = &tokens[*pos];
            *pos += 1;
            if msg_tok.starts_with('"') {
                msg_tok[1..msg_tok.len() - 1].to_string()
            } else {
                msg_tok.clone()
            }
        } else {
            "assertion failed".to_string()
        };
        let tolerance = if *pos < tokens.len() && tokens[*pos] != ")" {
            let tol_tok = &tokens[*pos];
            *pos += 1;
            tol_tok.parse::<f64>().unwrap_or(0.0)
        } else {
            0.0
        };
        Ok(NeExpr::Assert {
            condition: Box::new(condition),
            message,
            tolerance,
        })
    } else if head == "test" {
        let name = if *pos < tokens.len() {
            let tok = &tokens[*pos];
            *pos += 1;
            if tok.starts_with('"') {
                tok[1..tok.len() - 1].to_string()
            } else {
                tok.clone()
            }
        } else {
            return Err("test needs name".into());
        };
        let body = parse_expr(tokens, pos)?;
        let expected = if *pos < tokens.len() && tokens[*pos] != ")" {
            let tok = &tokens[*pos];
            *pos += 1;
            tok == "true"
        } else {
            true
        };
        Ok(NeExpr::Test {
            name,
            body: Box::new(body),
            expected,
        })
    } else if head == "property" {
        let name = if *pos < tokens.len() {
            let tok = &tokens[*pos];
            *pos += 1;
            if tok.starts_with('"') {
                tok[1..tok.len() - 1].to_string()
            } else {
                tok.clone()
            }
        } else {
            return Err("property needs name".into());
        };
        let generator = parse_expr(tokens, pos)?;
        let property = parse_expr(tokens, pos)?;
        let iterations = if *pos < tokens.len() && tokens[*pos] != ")" {
            let tok = &tokens[*pos];
            *pos += 1;
            tok.parse::<u64>().unwrap_or(100)
        } else {
            100
        };
        Ok(NeExpr::Property {
            name,
            generator: Box::new(generator),
            property: Box::new(property),
            iterations,
        })
    } else {
        let mut args = Vec::new();
        while *pos < tokens.len() && tokens[*pos] != ")" {
            args.push(parse_expr(tokens, pos)?);
        }
        Ok(NeExpr::Call(head.clone(), args))
    }
}

fn expr_to_value(e: &NeExpr) -> Result<NeValue, String> {
    match e {
        NeExpr::Literal(v) => Ok(v.clone()),
        _ => Err(format!("cannot convert expr to value at parse time: {e:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_literal_int() {
        let e = parse_ne("42").unwrap();
        assert!(matches!(e, NeExpr::Literal(NeValue::Int(42))));
    }

    #[test]
    fn test_parse_literal_string() {
        let e = parse_ne(r#""hello""#).unwrap();
        assert!(matches!(e, NeExpr::Literal(NeValue::Str(_))));
    }

    #[test]
    fn test_parse_simple_call() {
        let e = parse_ne("(bind a b)").unwrap();
        assert!(matches!(e, NeExpr::Bind(_, _)));
    }

    #[test]
    fn test_parse_nested() {
        let e = parse_ne("(bind (xor a b) c)").unwrap();
        assert!(matches!(e, NeExpr::Bind(_, _)));
    }

    #[test]
    fn test_parse_if() {
        let e = parse_ne("(if true 1 0)").unwrap();
        assert!(matches!(e, NeExpr::If(_, _, _)));
    }

    #[test]
    fn test_parse_let() {
        let e = parse_ne("(let x 5 x)").unwrap();
        assert!(matches!(e, NeExpr::Let(_, _, _)));
    }

    #[test]
    fn test_parse_comment() {
        let e = parse_ne("42 ; this is a comment\n 43").unwrap();
        assert!(matches!(e, NeExpr::Literal(NeValue::Int(43))));
    }

    #[test]
    fn test_parse_lambda() {
        let e = parse_ne("(fn [x] x)").unwrap();
        assert!(matches!(e, NeExpr::Lambda(_, _)));
    }

    #[test]
    fn test_parse_list_literal() {
        let e = parse_ne("[1 2 3]").unwrap();
        assert!(matches!(e, NeExpr::Literal(NeValue::List(_))));
    }

    #[test]
    fn test_parse_do() {
        let e = parse_ne("(do 1 2 3)").unwrap();
        assert!(matches!(e, NeExpr::Seq(_)));
    }

    #[test]
    fn test_parse_error_unclosed() {
        assert!(parse_ne("(if true 1").is_err());
    }

    #[test]
    fn test_parse_error_unclosed_string() {
        assert!(parse_ne(r#""hello"#).is_err());
    }

    #[test]
    fn test_parse_error_unexpected_paren() {
        assert!(parse_ne(")").is_err());
    }

    #[test]
    fn test_walk_vars() {
        let e = parse_ne("(bind a b)").unwrap();
        let mut vars = Vec::new();
        e.walk_vars(&mut |v| vars.push(v.to_string()));
        vars.sort();
        assert_eq!(vars, vec!["a", "b"]);
    }

    #[test]
    fn test_walk_vars_skips_lambda_params() {
        let e = parse_ne("(fn [x] (bind x y))").unwrap();
        let mut vars = Vec::new();
        e.walk_vars(&mut |v| vars.push(v.to_string()));
        assert_eq!(vars, vec!["y"]);
    }

    #[test]
    fn test_parse_loop() {
        let e = parse_ne("(loop x 5)").unwrap();
        assert!(matches!(e, NeExpr::Loop { .. }));
    }

    #[test]
    fn test_parse_loop_default_iters() {
        let e = parse_ne("(loop x)").unwrap();
        assert!(matches!(e, NeExpr::Loop { .. }));
    }

    #[test]
    fn test_parse_match() {
        let e = parse_ne("(match 1 (1 \"one\") (_ \"other\"))").unwrap();
        assert!(matches!(e, NeExpr::Match { .. }));
    }

    #[test]
    fn test_parse_import() {
        let e = parse_ne("(import \"math.ne\")").unwrap();
        assert!(matches!(e, NeExpr::Import { .. }));
    }

    #[test]
    fn test_parse_export() {
        let e = parse_ne("(export pi 3.14)").unwrap();
        assert!(matches!(e, NeExpr::Export { .. }));
    }
}
