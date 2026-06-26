// ---- Stage 1 S-expression parser and transpiler ----

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    LParen,
    RParen,
    LBracket,
    RBracket,
    Atom(String),
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SExpr {
    Number(i64),
    Float(f64),
    Str(String),
    Symbol(String),
    Vector(Vec<i64>),
    List(Vec<SExpr>),
}

pub(crate) fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '(' {
            tokens.push(Token::LParen);
            chars.next();
        } else if c == ')' {
            tokens.push(Token::RParen);
            chars.next();
        } else if c == '[' {
            tokens.push(Token::LBracket);
            chars.next();
        } else if c == ']' {
            tokens.push(Token::RBracket);
            chars.next();
        } else if c == '"' {
            chars.next();
            let mut s = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '"' {
                    chars.next();
                    break;
                }
                if ch == '\\' {
                    chars.next();
                    if let Some(&esc) = chars.peek() {
                        match esc {
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            'r' => s.push('\r'),
                            '"' => s.push('"'),
                            '\\' => s.push('\\'),
                            c => s.push(c),
                        }
                        chars.next();
                    }
                } else {
                    s.push(ch);
                    chars.next();
                }
            }
            tokens.push(Token::Str(s));
        } else {
            let mut atom = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() || ch == '(' || ch == ')' || ch == '[' || ch == ']' {
                    break;
                }
                atom.push(ch);
                chars.next();
            }
            tokens.push(Token::Atom(atom));
        }
    }
    tokens
}

pub(crate) fn parse_sexpr(tokens: &[Token], pos: &mut usize) -> Result<SExpr, String> {
    if *pos >= tokens.len() {
        return Err("unexpected end of input".to_string());
    }
    match &tokens[*pos] {
        Token::LParen => {
            *pos += 1;
            let mut items = Vec::new();
            while *pos < tokens.len() && tokens[*pos] != Token::RParen {
                items.push(parse_sexpr(tokens, pos)?);
            }
            if *pos >= tokens.len() {
                return Err("unclosed parenthesis".to_string());
            }
            *pos += 1;
            Ok(SExpr::List(items))
        }
        Token::LBracket => {
            *pos += 1;
            let mut nums = Vec::new();
            while *pos < tokens.len() && tokens[*pos] != Token::RBracket {
                match &tokens[*pos] {
                    Token::Atom(s) => {
                        let n = s.parse::<i64>().map_err(|e| {
                            format!("expected integer in vector, got '{}': {}", s, e)
                        })?;
                        *pos += 1;
                        nums.push(n);
                    }
                    other => {
                        return Err(format!("expected integer in vector, got {:?}", other));
                    }
                }
            }
            if *pos >= tokens.len() {
                return Err("unclosed bracket".to_string());
            }
            *pos += 1;
            Ok(SExpr::Vector(nums))
        }
        Token::Atom(s) => {
            let val = s.clone();
            *pos += 1;
            if val == "true" {
                Ok(SExpr::Symbol("true".to_string()))
            } else if val == "false" {
                Ok(SExpr::Symbol("false".to_string()))
            } else if val == "nil" {
                Ok(SExpr::Symbol("nil".to_string()))
            } else if val.contains('.') {
                let n = val
                    .parse::<f64>()
                    .map_err(|e| format!("invalid float '{}': {}", val, e))?;
                Ok(SExpr::Float(n))
            } else if let Ok(n) = val.parse::<i64>() {
                Ok(SExpr::Number(n))
            } else {
                Ok(SExpr::Symbol(val))
            }
        }
        Token::Str(s) => {
            let val = s.clone();
            *pos += 1;
            Ok(SExpr::Str(val))
        }
        Token::RParen => Err("unexpected ')'".to_string()),
        Token::RBracket => Err("unexpected ']'".to_string()),
    }
}

pub(crate) fn transpile_sexpr(expr: &SExpr) -> String {
    match expr {
        SExpr::Number(n) => format!("{}i64", n),
        SExpr::Float(f) => format!("{}f64", f),
        SExpr::Str(s) => format!("\"{}\".to_string()", s),
        SExpr::Symbol(s) => match s.as_str() {
            "true" => "true".to_string(),
            "false" => "false".to_string(),
            "nil" => "()".to_string(),
            name => name.to_string(),
        },
        SExpr::Vector(v) => format!("vec!{:?}", v),
        SExpr::List(items) => {
            if items.is_empty() {
                return "()".to_string();
            }
            if let SExpr::Symbol(op) = &items[0] {
                let op_str = op.as_str();
                match op_str {
                    "bind" => {
                        let a = transpile_sexpr(&items[1]);
                        let b = transpile_sexpr(&items[2]);
                        format!("QuantizedVSA::bind(&{}, &{})", a, b)
                    }
                    "unbind" => {
                        let a = transpile_sexpr(&items[1]);
                        let b = transpile_sexpr(&items[2]);
                        format!("QuantizedVSA::unbind(&{}, &{})", a, b)
                    }
                    "bundle" => {
                        let args: Vec<String> = items[1..].iter().map(transpile_sexpr).collect();
                        let refs: Vec<String> = args.iter().map(|a| format!("&{}", a)).collect();
                        format!("QuantizedVSA::bundle(&[{}])", refs.join(", "))
                    }
                    "negate" => {
                        let x = transpile_sexpr(&items[1]);
                        format!("QuantizedVSA::negate(&{})", x)
                    }
                    "permute" => {
                        let x = transpile_sexpr(&items[1]);
                        let n = if items.len() > 2 {
                            transpile_sexpr(&items[2])
                        } else {
                            "1".to_string()
                        };
                        format!("QuantizedVSA::permute(&{}, {})", x, n)
                    }
                    "rotate_left" => {
                        let x = transpile_sexpr(&items[1]);
                        let n = if items.len() > 2 {
                            transpile_sexpr(&items[2])
                        } else {
                            "1".to_string()
                        };
                        format!("QuantizedVSA::permute(&{}, {})", x, n)
                    }
                    "rotate_right" => {
                        let x = transpile_sexpr(&items[1]);
                        let n = if items.len() > 2 {
                            transpile_sexpr(&items[2])
                        } else {
                            "1".to_string()
                        };
                        format!("QuantizedVSA::permute(&{}, -({}))", x, n)
                    }
                    "similarity" => {
                        let a = transpile_sexpr(&items[1]);
                        let b = transpile_sexpr(&items[2]);
                        format!("QuantizedVSA::similarity(&{}, &{})", a, b)
                    }
                    "if" => {
                        let c = transpile_sexpr(&items[1]);
                        let t = transpile_sexpr(&items[2]);
                        let e = transpile_sexpr(&items[3]);
                        format!("if {} {{ {} }} else {{ {} }}", c, t, e)
                    }
                    "let" => {
                        let name = match &items[1] {
                            SExpr::Symbol(n) => n.clone(),
                            other => transpile_sexpr(other),
                        };
                        let val_expr = transpile_sexpr(&items[2]);
                        let body = transpile_sexpr(&items[3]);
                        format!("{{ let {} = {}; {} }}", name, val_expr, body)
                    }
                    "seq" => {
                        let exprs: Vec<String> = items[1..].iter().map(transpile_sexpr).collect();
                        exprs.join("; ")
                    }
                    "lambda" => "<lambda>".to_string(),
                    "+" | "add" => {
                        let args: Vec<String> = items[1..].iter().map(transpile_sexpr).collect();
                        args.join(" + ")
                    }
                    "*" | "mul" => {
                        let args: Vec<String> = items[1..].iter().map(transpile_sexpr).collect();
                        args.join(" * ")
                    }
                    "-" | "sub" => {
                        let args: Vec<String> = items[1..].iter().map(transpile_sexpr).collect();
                        args.join(" - ")
                    }
                    "/" | "div" => {
                        let args: Vec<String> = items[1..].iter().map(transpile_sexpr).collect();
                        args.join(" / ")
                    }
                    _ => {
                        let args: Vec<String> = items[1..].iter().map(transpile_sexpr).collect();
                        format!("{}({})", op, args.join(", "))
                    }
                }
            } else {
                let items_s: Vec<String> = items.iter().map(transpile_sexpr).collect();
                format!("({})", items_s.join(" "))
            }
        }
    }
}
