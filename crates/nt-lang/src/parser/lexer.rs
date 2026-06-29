// Phase 2b: Proper lexer producing token streams with SourceSpan

/// A source position (byte offset + line + column).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePos {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl SourcePos {
    pub const fn new(offset: usize, line: usize, column: usize) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }
}

/// A span from `start` to `end` in the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: SourcePos,
    pub end: SourcePos,
}

impl SourceSpan {
    pub const fn new(start: SourcePos, end: SourcePos) -> Self {
        Self { start, end }
    }

    /// Combine two adjacent spans into one covering both.
    pub fn covering(a: SourceSpan, b: SourceSpan) -> Self {
        Self {
            start: a.start,
            end: b.end,
        }
    }

    pub fn format(&self, source: &str) -> String {
        let line = source.lines().nth(self.start.line).unwrap_or("");
        let indicator = format!(
            "{:>width$} | {}",
            self.start.line + 1,
            line,
            width = (self.start.line + 1).to_string().len().max(3)
        );
        let caret = format!(
            "{:>width$}   {}{}",
            "",
            " ".repeat(self.start.column),
            "^".repeat(if self.start.line == self.end.line {
                self.end.column - self.start.column
            } else {
                1
            }),
            width = (self.start.line + 1).to_string().len().max(3)
        );
        format!("{}\n{}", indicator, caret)
    }
}

/// A token kind.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Identifiers & literals
    Ident(String),
    Number(String),    // integer or float, raw text to preserve precision
    StringLit(String), // "..." quoted string

    // Keywords
    Let,
    If,
    Else,
    For,
    In,
    Return,
    Fn,
    True,
    False,

    // Operators (arithmetic)
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // Operators (comparison)
    EqEq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    AndAnd,
    OrOr,
    Not,

    // Operators (assignment / arrow)
    Eq,
    Arrow,

    // VSA operators
    VsaBundle,  // ⊕
    VsaBind,    // ⊗
    VsaPermute, // ⊥
    VsaNegate,  // ¬ (VSA context)

    // Sutra fuzzy logic operators
    FuzzyAnd,   // ⊓ or &? (fuzzy AND)
    FuzzyOr,    // ⊔ or |? (fuzzy OR)
    FuzzyImply, // → or -> (fuzzy implication)

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Colon,
    Pipe, // | for pipeline stages

    // Special
    Newline,
    Indent, // block start
    Dedent, // block end

    // End of file
    Eof,

    // Error token
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

impl Token {
    pub fn new(kind: TokenKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}

/// Lexer: char-by-char scanner producing a Vec<Token>.
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 0,
            column: 0,
        }
    }

    /// Tokenize the entire source into a token stream.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            // Handle indentation before consuming regular tokens
            self.skip_whitespace_and_comments();

            if self.pos >= self.chars.len() {
                tokens.push(Token::new(
                    TokenKind::Eof,
                    SourceSpan::new(
                        SourcePos::new(self.pos, self.line, self.column),
                        SourcePos::new(self.pos, self.line, self.column),
                    ),
                ));
                return tokens;
            }

            let start = SourcePos::new(self.pos, self.line, self.column);
            let ch = self.chars[self.pos];

            let kind = match ch {
                // Single-char tokens
                '(' => {
                    self.advance();
                    TokenKind::LParen
                }
                ')' => {
                    self.advance();
                    TokenKind::RParen
                }
                '{' => {
                    self.advance();
                    TokenKind::LBrace
                }
                '}' => {
                    self.advance();
                    TokenKind::RBrace
                }
                ',' => {
                    self.advance();
                    TokenKind::Comma
                }
                ';' => {
                    self.advance();
                    TokenKind::Semicolon
                }
                ':' => {
                    self.advance();
                    TokenKind::Colon
                }

                // Operators
                '+' => {
                    self.advance();
                    TokenKind::Plus
                }
                '-' => {
                    if self.peek() == Some('>') {
                        self.advance();
                        self.advance();
                        TokenKind::Arrow
                    } else {
                        self.advance();
                        TokenKind::Minus
                    }
                }
                '*' => {
                    self.advance();
                    TokenKind::Star
                }
                '/' => {
                    self.advance();
                    if self.peek() == Some('/') {
                        // Line comment — already handled by skip_whitespace_and_comments
                        // (shouldn't reach here, but just in case)
                        while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                            self.advance();
                        }
                        continue;
                    }
                    TokenKind::Slash
                }
                '%' => {
                    self.advance();
                    TokenKind::Percent
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        TokenKind::Ne
                    } else {
                        self.advance();
                        TokenKind::Not
                    }
                }
                '=' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        TokenKind::EqEq
                    } else {
                        self.advance();
                        TokenKind::Eq
                    }
                }
                '<' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        TokenKind::Le
                    } else {
                        self.advance();
                        TokenKind::Lt
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        TokenKind::Ge
                    } else {
                        self.advance();
                        TokenKind::Gt
                    }
                }
                '&' => {
                    if self.peek() == Some('&') {
                        self.advance();
                        self.advance();
                        TokenKind::AndAnd
                    } else {
                        self.advance();
                        TokenKind::Error("Expected &&".to_string())
                    }
                }
                '|' => {
                    if self.peek() == Some('|') {
                        self.advance();
                        self.advance();
                        TokenKind::OrOr
                    } else {
                        self.advance();
                        TokenKind::Pipe
                    }
                }

                // Unicode VSA operators
                '\u{2295}' => {
                    self.advance();
                    TokenKind::VsaBundle
                } // ⊕
                '\u{2297}' => {
                    self.advance();
                    TokenKind::VsaBind
                } // ⊗
                '\u{22A5}' => {
                    self.advance();
                    TokenKind::VsaPermute
                } // ⊥
                '\u{00AC}' => {
                    self.advance();
                    TokenKind::VsaNegate
                } // ¬
                // Sutra fuzzy logic operators
                '\u{2293}' => {
                    self.advance();
                    TokenKind::FuzzyAnd
                } // ⊓
                '\u{2294}' => {
                    self.advance();
                    TokenKind::FuzzyOr
                } // ⊔
                '\u{2192}' => {
                    self.advance();
                    TokenKind::FuzzyImply
                } // →

                // String literal
                '"' => {
                    self.advance();
                    self.read_string()
                }

                // Number
                '0'..='9' => self.read_number(),

                // Identifier or keyword
                'a'..='z' | 'A'..='Z' | '_' => self.read_ident_or_keyword(),

                '\n' => {
                    self.advance();
                    // Collapse consecutive newlines into one
                    // (indentation handling is simple for now)
                    TokenKind::Newline
                }

                other => {
                    self.advance();
                    TokenKind::Error(format!("Unexpected character '{}'", other))
                }
            };

            let end = SourcePos::new(self.pos, self.line, self.column);
            tokens.push(Token::new(kind, SourceSpan::new(start, end)));
        }
    }

    // ---- Internal helpers ----

    fn advance(&mut self) {
        if self.pos < self.chars.len() {
            if self.chars[self.pos] == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '/' if self.peek() == Some('/') => {
                    // Line comment: skip until newline
                    while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                        self.advance();
                    }
                }
                '/' if self.peek() == Some('*') => {
                    // Block comment: skip until */
                    self.advance(); // skip /
                    self.advance(); // skip *
                    while self.pos + 1 < self.chars.len() {
                        if self.chars[self.pos] == '*' && self.peek() == Some('/') {
                            self.advance(); // skip *
                            self.advance(); // skip /
                            break;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn read_string(&mut self) -> TokenKind {
        let mut s = String::new();
        while self.pos < self.chars.len() && self.chars[self.pos] != '"' {
            if self.chars[self.pos] == '\\' {
                self.advance();
                match self.chars.get(self.pos) {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('r') => s.push('\r'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some(c) => s.push(*c),
                    None => s.push('\\'),
                }
                if self.pos < self.chars.len() {
                    self.advance();
                }
            } else {
                s.push(self.chars[self.pos]);
                self.advance();
            }
        }
        if self.pos < self.chars.len() {
            self.advance(); // skip closing "
        }
        TokenKind::StringLit(s)
    }

    fn read_number(&mut self) -> TokenKind {
        let start = self.pos;
        let mut is_float = false;
        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_ascii_digit() || self.chars[self.pos] == '.')
        {
            if self.chars[self.pos] == '.' {
                if is_float {
                    break; // second dot -> range operator, stop here
                }
                is_float = true;
            }
            self.advance();
        }
        let raw: String = self.chars[start..self.pos].iter().collect();
        TokenKind::Number(raw)
    }

    fn read_ident_or_keyword(&mut self) -> TokenKind {
        let start = self.pos;
        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_alphanumeric() || self.chars[self.pos] == '_')
        {
            self.advance();
        }
        let word: String = self.chars[start..self.pos].iter().collect();
        match word.as_str() {
            "let" => TokenKind::Let,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,
            "fn" => TokenKind::Fn,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Ident(word),
        }
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(source);
        lexer.tokenize().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_empty() {
        let k = kinds("");
        assert_eq!(k, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_ident() {
        let k = kinds("hello");
        assert_eq!(k[0], TokenKind::Ident("hello".into()));
    }

    #[test]
    fn test_number_int() {
        let k = kinds("42");
        assert_eq!(k[0], TokenKind::Number("42".into()));
    }

    #[test]
    fn test_number_float() {
        let k = kinds("3.14");
        assert_eq!(k[0], TokenKind::Number("3.14".into()));
    }

    #[test]
    fn test_string() {
        let k = kinds("\"hello world\"");
        assert_eq!(k[0], TokenKind::StringLit("hello world".into()));
    }

    #[test]
    fn test_keywords() {
        let k = kinds("let if else for in return fn");
        assert_eq!(k[0], TokenKind::Let);
        assert_eq!(k[1], TokenKind::If);
        assert_eq!(k[2], TokenKind::Else);
        assert_eq!(k[3], TokenKind::For);
        assert_eq!(k[4], TokenKind::In);
        assert_eq!(k[5], TokenKind::Return);
        assert_eq!(k[6], TokenKind::Fn);
    }

    #[test]
    fn test_boolean_literals() {
        let k = kinds("true false");
        assert_eq!(k[0], TokenKind::True);
        assert_eq!(k[1], TokenKind::False);
    }

    #[test]
    fn test_operators() {
        let k = kinds("+ - * / % == != < > <= >= && || ! ->");
        assert_eq!(k[0], TokenKind::Plus);
        assert_eq!(k[1], TokenKind::Minus);
        assert_eq!(k[2], TokenKind::Star);
        assert_eq!(k[3], TokenKind::Slash);
        assert_eq!(k[4], TokenKind::Percent);
        assert_eq!(k[5], TokenKind::EqEq);
        assert_eq!(k[6], TokenKind::Ne);
        assert_eq!(k[7], TokenKind::Lt);
        assert_eq!(k[8], TokenKind::Gt);
        assert_eq!(k[9], TokenKind::Le);
        assert_eq!(k[10], TokenKind::Ge);
        assert_eq!(k[11], TokenKind::AndAnd);
        assert_eq!(k[12], TokenKind::OrOr);
        assert_eq!(k[13], TokenKind::Not);
        assert_eq!(k[14], TokenKind::Arrow);
    }

    #[test]
    fn test_vsa_operators() {
        let k = kinds("⊕ ⊗ ⊥ ¬");
        assert_eq!(k[0], TokenKind::VsaBundle);
        assert_eq!(k[1], TokenKind::VsaBind);
        assert_eq!(k[2], TokenKind::VsaPermute);
        assert_eq!(k[3], TokenKind::VsaNegate);
    }

    #[test]
    fn test_fuzzy_operators() {
        let k = kinds("⊓ ⊔ →");
        assert_eq!(k[0], TokenKind::FuzzyAnd);
        assert_eq!(k[1], TokenKind::FuzzyOr);
        assert_eq!(k[2], TokenKind::FuzzyImply);
    }

    #[test]
    fn test_vsa_operator_aliases_ident() {
        let k = kinds("bundle bind permute negate");
        assert!(matches!(&k[0], TokenKind::Ident(_)));
        assert!(matches!(&k[1], TokenKind::Ident(_)));
        assert!(matches!(&k[2], TokenKind::Ident(_)));
        assert!(matches!(&k[3], TokenKind::Ident(_)));
    }

    #[test]
    fn test_delimiters() {
        let k = kinds("( ) { } , ; :");
        assert_eq!(k[0], TokenKind::LParen);
        assert_eq!(k[1], TokenKind::RParen);
        assert_eq!(k[2], TokenKind::LBrace);
        assert_eq!(k[3], TokenKind::RBrace);
        assert_eq!(k[4], TokenKind::Comma);
    }

    #[test]
    fn test_line_comment() {
        let k = kinds("a // this is a comment\nb");
        assert_eq!(k[0], TokenKind::Ident("a".into()));
        // Newline between a and b (or not, if comment consuming ate it)
        let b_idx = if k[1] == TokenKind::Newline { 2 } else { 1 };
        assert_eq!(k[b_idx], TokenKind::Ident("b".into()));
    }

    #[test]
    fn test_block_comment() {
        let k = kinds("a /* block */ b");
        assert_eq!(k[0], TokenKind::Ident("a".into()));
        assert_eq!(k[1], TokenKind::Ident("b".into()));
    }

    #[test]
    fn test_source_positions() {
        let mut lexer = Lexer::new("a\n  b");
        let tokens = lexer.tokenize();
        // a at line 0, col 0
        assert_eq!(tokens[0].span.start.line, 0);
        assert_eq!(tokens[0].span.start.column, 0);
        // b at line 1, col 2
        let b_token = tokens
            .iter()
            .find(|t| t.kind == TokenKind::Ident("b".into()))
            .unwrap();
        assert_eq!(b_token.span.start.line, 1);
        assert_eq!(b_token.span.start.column, 2);
    }

    #[test]
    fn test_source_span_format() {
        let source = "let x = 42\nreturn x";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let return_token = tokens
            .iter()
            .find(|t| t.kind == TokenKind::Return)
            .expect("Should find 'return'");
        let formatted = return_token.span.format(source);
        assert!(formatted.contains("return"));
        assert!(formatted.contains("^"));
    }

    #[test]
    fn test_escaped_string() {
        let k = kinds("\"hello\\nworld\"");
        assert_eq!(k[0], TokenKind::StringLit("hello\nworld".into()));
    }

    #[test]
    fn test_expression_tokens() {
        let k = kinds("(a + b) * c");
        assert_eq!(k[0], TokenKind::LParen);
        assert_eq!(k[1], TokenKind::Ident("a".into()));
        assert_eq!(k[2], TokenKind::Plus);
        assert_eq!(k[3], TokenKind::Ident("b".into()));
        assert_eq!(k[4], TokenKind::RParen);
        assert_eq!(k[5], TokenKind::Star);
        assert_eq!(k[6], TokenKind::Ident("c".into()));
    }
}
