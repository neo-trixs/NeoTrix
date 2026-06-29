use crate::ast::{EditDirective, NeExpr};
use crate::lexer::{Lexer, Token};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            let t = lexer.next_token();
            let is_eof = t == Token::Eof;
            tokens.push(t);
            if is_eof {
                break;
            }
        }
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn peek_is(&self, tok: Token) -> bool {
        self.tokens.get(self.pos) == Some(&tok)
    }

    fn peek_is_keyword(&self, kw: &str) -> bool {
        matches!(self.tokens.get(self.pos), Some(Token::Keyword(k)) if k == kw)
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        match self.advance() {
            Some(t) if *t == expected => Ok(()),
            Some(t) => Err(format!("Expected {:?}, got {:?}", expected, t)),
            None => Err(format!("Expected {:?}, got EOF", expected)),
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(s.clone()),
            Some(Token::Keyword(s)) => Ok(s.clone()),
            Some(t) => Err(format!("Expected identifier, got {:?}", t)),
            None => Err("Expected identifier, got EOF".to_string()),
        }
    }

    fn skip_semicolons(&mut self) {
        while self.peek_is(Token::Semicolon) {
            self.advance();
        }
    }

    /// Top-level parse entry point.
    pub fn parse(&mut self) -> Result<NeExpr, String> {
        self.skip_semicolons();
        match self.peek() {
            Some(Token::Keyword(ref k)) if k == "module" => {
                self.advance();
                self.parse_module_inner()
            }
            Some(Token::Keyword(ref k)) if k == "fn" => {
                self.advance();
                self.parse_fn_inner()
            }
            Some(Token::LParen) => self.parse_sexpr(),
            Some(_) => self.parse_expr(),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    // ---- Module ----
    fn parse_module_inner(&mut self) -> Result<NeExpr, String> {
        let name = self.expect_ident()?;
        let mut imports = Vec::new();
        let mut body = Vec::new();
        // Check for { for block-style module
        if self.peek_is(Token::LBrace) {
            self.advance(); // consume {
            loop {
                if self.peek_is(Token::RBrace) {
                    break;
                }
                // Check for import statements
                if self.peek_is_keyword("import") {
                    self.advance(); // consume import
                    let import_name = self.expect_ident()?;
                    imports.push(import_name);
                } else {
                    body.push(self.parse_expr()?);
                }
                self.skip_semicolons();
            }
            self.expect(Token::RBrace)?;
        }
        Ok(NeExpr::Module {
            name,
            imports,
            body,
        })
    }

    // ---- Function ----
    fn parse_fn_inner(&mut self) -> Result<NeExpr, String> {
        let name = self.expect_ident()?;
        let params = self.parse_fn_params()?;
        let return_type = if self.peek_is(Token::Arrow) {
            self.advance();
            Some(self.parse_type_name()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(NeExpr::Function {
            name,
            params,
            body: Box::new(body),
            return_type,
        })
    }

    fn parse_fn_params(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        if !self.peek_is(Token::LParen) {
            return Ok(params);
        }
        self.advance(); // consume (
        loop {
            if self.peek_is(Token::RParen) {
                break;
            }
            let name = self.expect_ident()?;
            params.push(name);
            // Optional type annotation
            if self.peek_is(Token::Colon) {
                self.advance(); // consume :
                self.parse_type_name()?; // skip the type
            }
            if self.peek_is(Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RParen)?;
        Ok(params)
    }

    fn parse_type_name(&mut self) -> Result<String, String> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(s.clone()),
            Some(Token::Keyword(s)) => Ok(s.clone()),
            Some(t) => Err(format!("Expected type name, got {:?}", t)),
            None => Err("Expected type name, got EOF".to_string()),
        }
    }

    fn parse_block(&mut self) -> Result<NeExpr, String> {
        self.expect(Token::LBrace)?;
        let mut exprs = Vec::new();
        loop {
            if self.peek_is(Token::RBrace) {
                break;
            }
            exprs.push(self.parse_expr()?);
            self.skip_semicolons();
        }
        self.expect(Token::RBrace)?;
        Ok(match exprs.len() {
            0 => NeExpr::Bundle(vec![]),
            1 => exprs.into_iter().next().unwrap(),
            _ => NeExpr::Seq(exprs),
        })
    }

    fn parse_bracket_list(&mut self) -> Result<NeExpr, String> {
        self.expect(Token::LBracket)?;
        let mut items = Vec::new();
        loop {
            if self.peek_is(Token::RBracket) {
                break;
            }
            items.push(self.parse_expr()?);
            if self.peek_is(Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBracket)?;
        Ok(NeExpr::Bundle(items))
    }

    // ---- S-expression parsing ----
    fn parse_sexpr(&mut self) -> Result<NeExpr, String> {
        self.expect(Token::LParen)?;

        let op_name = match self.peek() {
            Some(Token::Keyword(ref k)) => {
                let n = k.clone();
                self.advance();
                n
            }
            Some(Token::Ident(ref i)) => {
                let n = i.clone();
                self.advance();
                n
            }
            Some(Token::RParen) => {
                // Empty sexpr: ()
                self.advance();
                return Ok(NeExpr::Bundle(vec![]));
            }
            Some(t) => {
                return Err(format!("Expected operation name in s-expression, got {:?}", t));
            }
            None => {
                return Err("Expected operation name in s-expression, got EOF".to_string());
            }
        };

        match op_name.as_str() {
            "fn" => self.parse_sexpr_fn(),
            "module" => self.parse_sexpr_module(),
            "let" => self.parse_sexpr_let(),
            "match" => self.parse_sexpr_match(),
            "seq" => {
                let mut items = Vec::new();
                while !self.peek_is(Token::RParen) {
                    items.push(self.parse_sexpr_arg()?);
                }
                self.expect(Token::RParen)?;
                Ok(NeExpr::Seq(items))
            }
            "default" => {
                // This should only appear inside match, parse as generic call
                let mut args = Vec::new();
                while !self.peek_is(Token::RParen) {
                    args.push(self.parse_sexpr_arg()?);
                }
                self.expect(Token::RParen)?;
                Ok(NeExpr::Call("default".to_string(), args))
            }
            "bundle" => {
                let mut items = Vec::new();
                while !self.peek_is(Token::RParen) {
                    items.push(self.parse_sexpr_arg()?);
                }
                self.expect(Token::RParen)?;
                Ok(NeExpr::Bundle(items))
            }
            _ => {
                // Keyword or function call in sexpr form
                let mut args = Vec::new();
                while !self.peek_is(Token::RParen) {
                    args.push(self.parse_sexpr_arg()?);
                }
                self.expect(Token::RParen)?;
                make_keyword_node(&op_name, args)
            }
        }
    }

    fn parse_sexpr_fn(&mut self) -> Result<NeExpr, String> {
        let name = self.expect_ident()?;
        let mut params = Vec::new();
        // Check if next is a sexpr that contains params
        if self.peek_is(Token::LParen) {
            // Could be (param1 param2 ...) as params list
            self.advance();
            loop {
                if self.peek_is(Token::RParen) {
                    break;
                }
                match self.peek() {
                    Some(Token::Ident(_)) | Some(Token::Keyword(_)) => {
                        params.push(self.expect_ident()?);
                    }
                    _ => break,
                }
            }
            self.expect(Token::RParen)?;
        }
        let body = self.parse_sexpr_arg()?;
        self.expect(Token::RParen)?;
        Ok(NeExpr::Function {
            name,
            params,
            body: Box::new(body),
            return_type: None,
        })
    }

    fn parse_sexpr_module(&mut self) -> Result<NeExpr, String> {
        let name = self.expect_ident()?;
        let mut imports = Vec::new();
        let mut body = Vec::new();
        while !self.peek_is(Token::RParen) {
            if self.peek_is(Token::LParen) {
                // Could be (import ...) or an expression
                let saved = self.pos;
                self.advance(); // consume (
                if self.peek_is_keyword("import") {
                    self.advance();
                    let import_name = self.expect_ident()?;
                    imports.push(import_name);
                    self.expect(Token::RParen)?;
                } else {
                    // Not an import, backtrack
                    self.pos = saved;
                    body.push(self.parse_sexpr_arg()?);
                }
            } else {
                body.push(self.parse_sexpr_arg()?);
            }
        }
        self.expect(Token::RParen)?;
        Ok(NeExpr::Module {
            name,
            imports,
            body,
        })
    }

    fn parse_sexpr_let(&mut self) -> Result<NeExpr, String> {
        let name = self.expect_ident()?;
        let value = self.parse_sexpr_arg()?;
        let body = if !self.peek_is(Token::RParen) {
            self.parse_sexpr_arg()?
        } else {
            NeExpr::Bundle(vec![])
        };
        self.expect(Token::RParen)?;
        Ok(NeExpr::Let(
            name,
            Box::new(value),
            Box::new(body),
        ))
    }

    fn parse_sexpr_match(&mut self) -> Result<NeExpr, String> {
        let scrutinee = self.parse_sexpr_arg()?;
        let mut arms = Vec::new();
        let mut default = None;
        while !self.peek_is(Token::RParen) {
            if self.peek_is(Token::LParen) {
                self.advance();
                if self.peek_is_keyword("default") {
                    self.advance();
                    let default_body = self.parse_sexpr_arg()?;
                    self.expect(Token::RParen)?;
                    default = Some(Box::new(default_body));
                } else {
                    let pattern = self.parse_sexpr_arg()?;
                    let body = self.parse_sexpr_arg()?;
                    self.expect(Token::RParen)?;
                    arms.push((pattern, body));
                }
            } else {
                let pattern = self.parse_sexpr_arg()?;
                let body = self.parse_sexpr_arg()?;
                arms.push((pattern, body));
            }
        }
        self.expect(Token::RParen)?;
        Ok(NeExpr::Match(
            Box::new(scrutinee),
            arms,
            default,
        ))
    }

    /// Parse an expression in sexpr context — no call-form detection.
    /// `ident(` is NOT interpreted as a call; the `(` is a separate nested sexpr.
    fn parse_sexpr_arg(&mut self) -> Result<NeExpr, String> {
        match self.peek().cloned() {
            Some(Token::LParen) => self.parse_sexpr(),
            Some(Token::LBrace) => self.parse_block(),
            Some(Token::LBracket) => self.parse_bracket_list(),
            Some(Token::StringLit(s)) => {
                self.advance();
                Ok(NeExpr::LitString(s))
            }
            Some(Token::FloatLit(f)) => {
                self.advance();
                Ok(NeExpr::LitFloat(f))
            }
            Some(Token::IntLit(i)) => {
                self.advance();
                Ok(NeExpr::LitInt(i))
            }
            Some(Token::Keyword(ref n)) | Some(Token::Ident(ref n)) => {
                let mut name = n.clone();
                self.advance();
                // Handle dotted paths
                while self.peek_is(Token::Dot) {
                    self.advance();
                    let part = self.expect_ident()?;
                    name.push('.');
                    name.push_str(&part);
                }
                Ok(NeExpr::Var(name))
            }
            Some(t) => Err(format!("Unexpected token in sexpr argument: {:?}", t)),
            None => Err("Unexpected end of input in sexpr argument".to_string()),
        }
    }

    // ---- Expression parsing ----
    fn parse_expr(&mut self) -> Result<NeExpr, String> {
        match self.peek().cloned() {
            Some(Token::Keyword(ref k)) if k == "let" => {
                self.advance();
                self.parse_let_inner()
            }
            Some(Token::Keyword(ref k)) if k == "match" => {
                self.advance();
                self.parse_match_inner()
            }
            Some(Token::Keyword(ref k)) if k == "fn" => {
                self.advance();
                self.parse_fn_inner()
            }
            Some(Token::Keyword(ref k)) if k == "module" => {
                self.advance();
                self.parse_module_inner()
            }
            Some(Token::LParen) => self.parse_sexpr(),
            Some(Token::LBrace) => self.parse_block(),
            Some(Token::LBracket) => self.parse_bracket_list(),
            Some(Token::Keyword(ref n)) | Some(Token::Ident(ref n)) => {
                let mut name = n.clone();
                self.advance();
                // Handle dotted paths: a.b.c → Var("a.b.c")
                while self.peek_is(Token::Dot) {
                    self.advance(); // consume .
                    let part = self.expect_ident()?;
                    name.push('.');
                    name.push_str(&part);
                }
                // Check if it's a call form: name(args)
                if self.peek_is(Token::LParen) {
                    self.advance(); // consume (
                    let mut args = Vec::new();
                    loop {
                        if self.peek_is(Token::RParen) {
                            break;
                        }
                        args.push(self.parse_expr()?);
                        if self.peek_is(Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(Token::RParen)?;
                    make_keyword_node(&name, args)
                } else {
                    Ok(NeExpr::Var(name))
                }
            }
            Some(Token::StringLit(s)) => {
                self.advance();
                Ok(NeExpr::LitString(s))
            }
            Some(Token::FloatLit(f)) => {
                self.advance();
                Ok(NeExpr::LitFloat(f))
            }
            Some(Token::IntLit(i)) => {
                self.advance();
                Ok(NeExpr::LitInt(i))
            }
            Some(t) => Err(format!("Unexpected token: {:?}", t)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_let_inner(&mut self) -> Result<NeExpr, String> {
        let name = self.expect_ident()?;
        self.expect(Token::Equals)?;
        let value = self.parse_expr()?;
        // Optional semicolon
        self.skip_semicolons();
        // Body is the rest (if any tokens remain before block end)
        let body = if self.peek_is(Token::RBrace) || self.peek_is(Token::Eof) {
            NeExpr::Bundle(vec![])
        } else {
            self.parse_expr()?
        };
        Ok(NeExpr::Let(
            name,
            Box::new(value),
            Box::new(body),
        ))
    }

    fn parse_match_inner(&mut self) -> Result<NeExpr, String> {
        let scrutinee = self.parse_expr()?;
        self.expect(Token::LBrace)?;
        let mut arms = Vec::new();
        let mut default = None;
        loop {
            if self.peek_is(Token::RBrace) {
                break;
            }
            let pattern = self.parse_expr()?;
            self.expect(Token::DoubleArrow)?;
            let body = self.parse_expr()?;
            if matches!(pattern, NeExpr::Var(ref v) if v == "_") {
                default = Some(Box::new(body));
            } else {
                arms.push((pattern, body));
            }
            if self.peek_is(Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(NeExpr::Match(
            Box::new(scrutinee),
            arms,
            default,
        ))
    }

    /// Parse an entire block expression (used by call forms that expect a block body).
    /// This is a convenience wrapper — the actual block parsing is in parse_block.
    pub fn parse_call_form_block(&mut self) -> Result<NeExpr, String> {
        self.parse_block()
    }
}

fn make_keyword_node(name: &str, args: Vec<NeExpr>) -> Result<NeExpr, String> {
    match name {
        "reflect" => {
            if args.is_empty() {
                Ok(NeExpr::Reflect(None))
            } else if args.len() == 1 {
                let inner = args.into_iter().next().unwrap();
                // If the argument is a string literal, use it as the reflect message
                match inner {
                    NeExpr::LitString(s) => Ok(NeExpr::Reflect(Some(s))),
                    other => Ok(NeExpr::Reflect(Some(other.to_sexpr()))),
                }
            } else {
                Err("reflect takes 0 or 1 argument".to_string())
            }
        }
        "curious" => {
            if args.len() == 1 {
                Ok(NeExpr::Curious(Box::new(args.into_iter().next().unwrap())))
            } else {
                Err(format!("curious takes 1 argument, got {}", args.len()))
            }
        }
        "dream" => {
            if args.len() == 1 {
                Ok(NeExpr::Dream(Box::new(args.into_iter().next().unwrap())))
            } else {
                Err(format!("dream takes 1 argument, got {}", args.len()))
            }
        }
        "edit" => {
            if args.len() >= 2 {
                let target_str = match &args[0] {
                    NeExpr::Var(v) => v.clone(),
                    NeExpr::LitString(s) => s.clone(),
                    _ => args[0].to_sexpr(),
                };
                Ok(NeExpr::Edit(Box::new(EditDirective {
                    target: target_str.split('.').map(|s| s.to_string()).collect(),
                    value: Box::new(args[1].clone()),
                    guard: if args.len() > 2 {
                        Some(Box::new(args[2].clone()))
                    } else {
                        None
                    },
                })))
            } else {
                Err(format!("edit takes at least 2 arguments, got {}", args.len()))
            }
        }
        "bind" => {
            if args.len() == 2 {
                let mut iter = args.into_iter();
                let a = iter.next().unwrap();
                let b = iter.next().unwrap();
                Ok(NeExpr::Bind(Box::new(a), Box::new(b)))
            } else {
                Err(format!("bind takes 2 arguments, got {}", args.len()))
            }
        }
        "bundle" => Ok(NeExpr::Bundle(args)),
        "permute" => {
            if args.len() == 2 {
                let mut iter = args.into_iter();
                let a = iter.next().unwrap();
                let b = iter.next().unwrap();
                Ok(NeExpr::Permute(Box::new(a), Box::new(b)))
            } else {
                Err(format!("permute takes 2 arguments, got {}", args.len()))
            }
        }
        "similarity" => {
            if args.len() == 2 {
                let mut iter = args.into_iter();
                let a = iter.next().unwrap();
                let b = iter.next().unwrap();
                Ok(NeExpr::Similarity(Box::new(a), Box::new(b)))
            } else {
                Err(format!(
                    "similarity takes 2 arguments, got {}",
                    args.len()
                ))
            }
        }
        _ => Ok(NeExpr::Call(name.to_string(), args)),
    }
}
