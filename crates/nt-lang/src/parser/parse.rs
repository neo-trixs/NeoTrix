// Phase 2b: Recursive descent parser with precedence climbing.
// Produces nt_lang::ir::Expr AST from lexer token stream.

use crate::ir::{BinOp, Expr, Literal, UnOp};
use crate::parser::lexer::{SourcePos, SourceSpan, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub span: SourceSpan,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse the top-level input as a sequence of statements → Block expression.
    pub fn parse_block(&mut self) -> Result<Expr, ParseError> {
        let mut exprs = Vec::new();
        while self.pos < self.tokens.len() && !self.is(TokenKind::Eof) {
            let stmt = self.parse_stmt()?;
            exprs.push(stmt);
            self.skip_newlines();
        }
        Ok(if exprs.len() == 1 {
            exprs.remove(0)
        } else {
            Expr::Block(exprs)
        })
    }

    /// Parse a single statement.
    pub fn parse_stmt(&mut self) -> Result<Expr, ParseError> {
        self.skip_newlines();
        if self.is_keyword("let") {
            self.parse_let()
        } else if self.is_keyword("if") {
            self.parse_if()
        } else if self.is_keyword("for") {
            self.parse_for()
        } else if self.is_keyword("return") {
            self.parse_return()
        } else {
            self.parse_expr(0)
        }
    }

    // ---- Expression parsing with precedence climbing ----

    /// Operator precedence (higher = binds tighter).
    fn prec(kind: &TokenKind) -> u8 {
        match kind {
            TokenKind::OrOr => 1,
            TokenKind::AndAnd => 2,
            TokenKind::EqEq
            | TokenKind::Ne
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::Le
            | TokenKind::Ge => 3,
            TokenKind::FuzzyImply => 1,
            TokenKind::FuzzyOr => 2,
            TokenKind::FuzzyAnd => 3,
            TokenKind::VsaBundle => 4,
            TokenKind::Plus | TokenKind::Minus => 5,
            TokenKind::VsaBind => 6,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => 7,
            TokenKind::VsaPermute => 8,
            _ => 0,
        }
    }

    /// Precedence climbing expression parser.
    fn parse_expr(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let _start = self.current_span();
        let mut lhs = self.parse_prefix()?;

        while self.pos < self.tokens.len() {
            let kind = &self.peek_kind();
            if kind == &TokenKind::Eof || kind == &TokenKind::Newline {
                break;
            }

            if let Some((op, prec)) = self.try_binary_op() {
                if prec < min_prec {
                    break;
                }
                self.advance(); // consume the operator token
                let rhs = self.parse_expr(prec + 1)?;
                lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    fn try_binary_op(&self) -> Option<(BinOp, u8)> {
        let kind = &self.peek_kind();
        let prec = Self::prec(kind);
        if prec == 0 {
            return None;
        }
        let op = match kind {
            TokenKind::Plus => BinOp::Add,
            TokenKind::Minus => BinOp::Sub,
            TokenKind::Star => BinOp::Mul,
            TokenKind::Slash => BinOp::Div,
            TokenKind::Percent => BinOp::Mod,
            TokenKind::EqEq => BinOp::Eq,
            TokenKind::Ne => BinOp::Ne,
            TokenKind::Lt => BinOp::Lt,
            TokenKind::Gt => BinOp::Gt,
            TokenKind::Le => BinOp::Le,
            TokenKind::Ge => BinOp::Ge,
            TokenKind::AndAnd => BinOp::And,
            TokenKind::OrOr => BinOp::Or,
            TokenKind::VsaBundle => BinOp::VsaBundle,
            TokenKind::VsaBind => BinOp::VsaBind,
            TokenKind::VsaPermute => BinOp::VsaPermute,
            TokenKind::FuzzyAnd => BinOp::FuzzyAnd,
            TokenKind::FuzzyOr => BinOp::FuzzyOr,
            TokenKind::FuzzyImply => BinOp::FuzzyImply,
            _ => return None,
        };
        Some((op, prec))
    }

    // ---- Prefix parsing (atoms, unary ops, grouping, calls) ----

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        self.skip_newlines();

        let kind = self.peek_kind();
        match kind {
            // Unary operators
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_expr(9)?;
                Ok(Expr::Unary(UnOp::Neg, Box::new(expr)))
            }
            TokenKind::Not => {
                self.advance();
                let expr = self.parse_expr(9)?;
                Ok(Expr::Unary(UnOp::Not, Box::new(expr)))
            }
            TokenKind::VsaNegate => {
                self.advance();
                let expr = self.parse_expr(9)?;
                Ok(Expr::Unary(UnOp::VsaNegate, Box::new(expr)))
            }

            // Grouping
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr(0)?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }

            // Block
            TokenKind::LBrace => {
                self.advance();
                let mut exprs = Vec::new();
                while self.pos < self.tokens.len()
                    && !self.is(TokenKind::RBrace)
                    && !self.is(TokenKind::Eof)
                {
                    exprs.push(self.parse_stmt()?);
                    self.skip_newlines();
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Expr::Block(exprs))
            }

            // Literals
            TokenKind::Number(_) => self.parse_number_lit(),
            TokenKind::StringLit(_) => self.parse_string_lit(),
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }

            // Identifier (possibly a function call)
            TokenKind::Ident(_) => self.parse_ident_or_call(),

            _ => {
                let token = self.current();
                Err(ParseError {
                    message: format!("Unexpected token {:?}", token.kind),
                    span: token.span,
                })
            }
        }
    }

    fn parse_number_lit(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance();
        if let TokenKind::Number(raw) = &token.kind {
            if raw.contains('.') {
                raw.parse::<f64>()
                    .map(|n| Expr::Literal(Literal::Float(n)))
                    .map_err(|_| ParseError {
                        message: format!("Invalid float literal: {}", raw),
                        span: token.span,
                    })
            } else {
                raw.parse::<i64>()
                    .map(|n| Expr::Literal(Literal::Int(n)))
                    .map_err(|_| ParseError {
                        message: format!("Invalid integer literal: {}", raw),
                        span: token.span,
                    })
            }
        } else {
            unreachable!("expected Number token in parse_number_lit")
        }
    }

    fn parse_string_lit(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance();
        if let TokenKind::StringLit(s) = &token.kind {
            Ok(Expr::Literal(Literal::String(s.clone())))
        } else {
            unreachable!("expected StringLit token in parse_string_lit")
        }
    }

    fn parse_ident_or_call(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance();
        let name = if let TokenKind::Ident(s) = &token.kind {
            s.clone()
        } else {
            unreachable!("expected Ident token in parse_ident_or_call")
        };

        if self.is(TokenKind::LParen) {
            self.advance();
            let mut args = Vec::new();
            loop {
                self.skip_newlines();
                if self.is(TokenKind::RParen) {
                    break;
                }
                if !args.is_empty() {
                    self.expect(TokenKind::Comma)?;
                    self.skip_newlines();
                }
                args.push(self.parse_expr(0)?);
            }
            self.expect(TokenKind::RParen)?;
            Ok(Expr::Call(name, args))
        } else {
            Ok(Expr::Ident(name))
        }
    }

    // ---- Statement parsing ----

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword("let")?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Eq)?;
        let val = self.parse_expr(0)?;
        Ok(Expr::Let(name, Box::new(val)))
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword("if")?;
        let cond = self.parse_expr(0)?;
        self.skip_newlines();
        let then_branch = self.parse_prefix()?; // expect block or expr
        let else_branch = if self.is_keyword("else") {
            self.advance();
            self.skip_newlines();
            Some(Box::new(self.parse_prefix()?))
        } else {
            None
        };
        Ok(Expr::If(Box::new(cond), Box::new(then_branch), else_branch))
    }

    fn parse_for(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword("for")?;
        let var = self.expect_ident()?;
        self.expect_keyword("in")?;
        let iterable = self.parse_expr(0)?;
        self.skip_newlines();
        let body = self.parse_prefix()?; // expect block or expr
        Ok(Expr::For(var, Box::new(iterable), Box::new(body)))
    }

    fn parse_return(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword("return")?;
        if self.is(TokenKind::Newline) || self.is(TokenKind::RBrace) || self.is(TokenKind::Eof) {
            Ok(Expr::Return(None))
        } else {
            let val = self.parse_expr(0)?;
            Ok(Expr::Return(Some(Box::new(val))))
        }
    }

    // ---- Token helpers ----

    fn peek_kind(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    fn current(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token {
            kind: TokenKind::Eof,
            span: SourceSpan::new(SourcePos::new(0, 0, 0), SourcePos::new(0, 0, 0)),
        })
    }

    fn current_span(&self) -> SourceSpan {
        self.current().span
    }

    fn advance(&mut self) -> Token {
        let token = self.current();
        self.pos += 1;
        token
    }

    fn is(&self, kind: TokenKind) -> bool {
        self.peek_kind() == kind
    }

    fn is_keyword(&self, keyword: &str) -> bool {
        match &self.peek_kind() {
            TokenKind::Let => keyword == "let",
            TokenKind::If => keyword == "if",
            TokenKind::Else => keyword == "else",
            TokenKind::For => keyword == "for",
            TokenKind::In => keyword == "in",
            TokenKind::Return => keyword == "return",
            TokenKind::Fn => keyword == "fn",
            TokenKind::True | TokenKind::False => false,
            _ => false,
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        if self.is(kind.clone()) {
            Ok(self.advance())
        } else {
            let token = self.current();
            Err(ParseError {
                message: format!("Expected {:?}, got {:?}", kind, token.kind),
                span: token.span,
            })
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<Token, ParseError> {
        let token = self.current();
        if self.is_keyword(keyword) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                message: format!("Expected '{}', got {:?}", keyword, token.kind),
                span: token.span,
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let token = self.current();
        if let TokenKind::Ident(s) = &token.kind {
            let s = s.clone();
            self.advance();
            Ok(s)
        } else {
            Err(ParseError {
                message: format!("Expected identifier, got {:?}", token.kind),
                span: token.span,
            })
        }
    }

    fn skip_newlines(&mut self) {
        while self.is(TokenKind::Newline) {
            self.advance();
        }
    }
}

/// Parse a source string directly into an Expr AST.
pub fn parse_expr(source: &str) -> Result<Expr, ParseError> {
    let mut lexer = crate::parser::lexer::Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse_block()
}

/// Parse a source string into a statement sequence Block.
pub fn parse_stmts(source: &str) -> Result<Expr, ParseError> {
    let mut lexer = crate::parser::lexer::Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse_block()
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Expr {
        parse_expr(s).expect("Parse failed")
    }

    fn parse_fails(s: &str) -> String {
        parse_expr(s).unwrap_err().message
    }

    #[test]
    fn test_parse_literal_int() {
        let e = parse("42");
        assert!(matches!(e, Expr::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_parse_literal_float() {
        let e = parse("3.14");
        assert!(matches!(e, Expr::Literal(Literal::Float(_))));
    }

    #[test]
    fn test_parse_literal_bool() {
        assert!(matches!(parse("true"), Expr::Literal(Literal::Bool(true))));
        assert!(matches!(
            parse("false"),
            Expr::Literal(Literal::Bool(false))
        ));
    }

    #[test]
    fn test_parse_ident() {
        let e = parse("my_var");
        assert!(matches!(e, Expr::Ident(_)));
    }

    #[test]
    fn test_parse_call_no_args() {
        let e = parse("foo()");
        assert!(matches!(e, Expr::Call(_, _)));
        if let Expr::Call(name, args) = e {
            assert_eq!(name, "foo");
            assert_eq!(args.len(), 0);
        }
    }

    #[test]
    fn test_parse_call_with_args() {
        let e = parse("foo(1, 2)");
        if let Expr::Call(name, args) = e {
            assert_eq!(name, "foo");
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected Call");
        }
    }

    #[test]
    fn test_parse_nested_call() {
        let e = parse("foo(bar(x), baz(y))");
        if let Expr::Call(name, args) = e {
            assert_eq!(name, "foo");
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected Call");
        }
    }

    #[test]
    fn test_parse_precedence_add_mul() {
        // a + b * c  →  a + (b * c)
        let e = parse("1 + 2 * 3");
        assert!(matches!(e, Expr::Binary(BinOp::Add, _, _)));
        if let Expr::Binary(BinOp::Add, _, right) = e {
            assert!(matches!(*right, Expr::Binary(BinOp::Mul, _, _)));
        }
    }

    #[test]
    fn test_parse_precedence_mul_add() {
        // a * b + c  →  (a * b) + c
        let e = parse("1 * 2 + 3");
        assert!(matches!(e, Expr::Binary(BinOp::Add, _, _)));
        if let Expr::Binary(BinOp::Add, left, _) = e {
            assert!(matches!(*left, Expr::Binary(BinOp::Mul, _, _)));
        }
    }

    #[test]
    fn test_parse_precedence_compare() {
        // a < b && c > d  →  (a < b) && (c > d)
        let expr = parse("1 < 2 && 3 > 4");
        assert!(matches!(expr, Expr::Binary(BinOp::And, _, _)));
    }

    #[test]
    fn test_parse_parenthesized() {
        // (a + b) * c  — parens override precedence
        let e = parse("(1 + 2) * 3");
        assert!(matches!(e, Expr::Binary(BinOp::Mul, _, _)));
        if let Expr::Binary(BinOp::Mul, left, _) = e {
            assert!(matches!(*left, Expr::Binary(BinOp::Add, _, _)));
        }
    }

    #[test]
    fn test_parse_unary_minus() {
        let e = parse("-42");
        assert!(matches!(e, Expr::Unary(UnOp::Neg, _)));
    }

    #[test]
    fn test_parse_unary_not() {
        let e = parse("!true");
        assert!(matches!(e, Expr::Unary(UnOp::Not, _)));
    }

    #[test]
    fn test_parse_let() {
        let e = parse("let x = 42");
        assert!(matches!(e, Expr::Let(_, _)));
        if let Expr::Let(name, val) = e {
            assert_eq!(name, "x");
            assert!(matches!(*val, Expr::Literal(Literal::Int(42))));
        }
    }

    #[test]
    fn test_parse_return() {
        assert!(matches!(parse("return 42"), Expr::Return(_)));
    }

    #[test]
    fn test_parse_return_empty() {
        assert!(matches!(parse("return"), Expr::Return(None)));
    }

    #[test]
    fn test_parse_block() {
        let e = parse("{ let x = 1\n let y = 2\n x + y }");
        assert!(matches!(e, Expr::Block(_)));
    }

    #[test]
    fn test_parse_vsa_bundle() {
        // ⊕ is VSA bundle (precedence 4)
        let e = parse("a ⊕ b");
        assert!(matches!(e, Expr::Binary(BinOp::VsaBundle, _, _)));
    }

    #[test]
    fn test_parse_vsa_bind() {
        // ⊗ is VSA bind (precedence 6)
        let e = parse("a ⊗ b");
        assert!(matches!(e, Expr::Binary(BinOp::VsaBind, _, _)));
    }

    #[test]
    fn test_parse_vsa_negate() {
        // ¬ is VSA negate (unary prefix)
        let e = parse("¬a");
        assert!(matches!(e, Expr::Unary(UnOp::VsaNegate, _)));
    }

    #[test]
    fn test_parse_vsa_function_calls() {
        assert!(matches!(parse("bundle(a, b)"), Expr::Call(name, _) if name == "bundle"));
        assert!(matches!(parse("bind(a, b)"), Expr::Call(name, _) if name == "bind"));
        assert!(matches!(parse("negate(a)"), Expr::Call(name, _) if name == "negate"));
        assert!(matches!(parse("similarity(a, b)"), Expr::Call(name, _) if name == "similarity"));
    }

    #[test]
    fn test_parse_vsa_precedence() {
        // a ⊕ b ⊗ c  →  a ⊕ (b ⊗ c)  since ⊗ (prec 6) > ⊕ (prec 4)
        let e = parse("a ⊕ b ⊗ c");
        assert!(matches!(e, Expr::Binary(BinOp::VsaBundle, _, _)));
        if let Expr::Binary(BinOp::VsaBundle, _, right) = e {
            assert!(matches!(*right, Expr::Binary(BinOp::VsaBind, _, _)));
        }
    }

    #[test]
    fn test_parse_multiple_stmts() {
        let stmts = "let x = 1\nlet y = 2\nx + y";
        let e = parse_expr(stmts).expect("Parse multi-stmt");
        assert!(matches!(e, Expr::Block(_)));
        if let Expr::Block(exprs) = e {
            assert_eq!(exprs.len(), 3);
        }
    }

    #[test]
    fn test_parse_error_unexpected() {
        let err = parse_fails("^");
        assert!(err.contains("Unexpected"));
    }

    #[test]
    fn test_parse_error_unmatched_paren() {
        let err = parse_fails("(1 + 2");
        assert!(err.contains("Expected"));
    }

    #[test]
    fn test_parse_string() {
        let e = parse("\"hello\"");
        assert!(matches!(e, Expr::Literal(Literal::String(s)) if s == "hello"));
    }

    #[test]
    fn test_parse_if() {
        let e = parse_expr("if true { 1 } else { 2 }").expect("parse if");
        assert!(matches!(e, Expr::If(_, _, _)));
    }

    #[test]
    fn test_parse_for() {
        let e = parse_expr("for x in items { x }").expect("parse for");
        assert!(matches!(e, Expr::For(_, _, _)));
    }

    #[test]
    fn test_parse_operator_andand() {
        let e = parse("a && b");
        assert!(matches!(e, Expr::Binary(BinOp::And, _, _)));
    }

    #[test]
    fn test_parse_operator_oror() {
        let e = parse("a || b");
        assert!(matches!(e, Expr::Binary(BinOp::Or, _, _)));
    }

    #[test]
    fn test_parse_fuzzy_and() {
        let e = parse("a ⊓ b");
        assert!(matches!(e, Expr::Binary(BinOp::FuzzyAnd, _, _)));
    }

    #[test]
    fn test_parse_fuzzy_or() {
        let e = parse("a ⊔ b");
        assert!(matches!(e, Expr::Binary(BinOp::FuzzyOr, _, _)));
    }

    #[test]
    fn test_parse_fuzzy_imply() {
        let e = parse("a → b");
        assert!(matches!(e, Expr::Binary(BinOp::FuzzyImply, _, _)));
    }

    #[test]
    fn test_parse_fuzzy_precedence() {
        // a ⊔ b ⊓ c → a ⊔ (b ⊓ c) since ⊓ (prec 3) > ⊔ (prec 2)
        let e = parse("a ⊔ b ⊓ c");
        assert!(matches!(e, Expr::Binary(BinOp::FuzzyOr, _, _)));
        if let Expr::Binary(BinOp::FuzzyOr, _, right) = e {
            assert!(matches!(*right, Expr::Binary(BinOp::FuzzyAnd, _, _)));
        }
    }
}
