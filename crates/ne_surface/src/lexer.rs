#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Arrow,
    DoubleArrow,
    Dot,
    Equals,
    Keyword(String),
    Ident(String),
    StringLit(String),
    FloatLit(f64),
    IntLit(i64),
    Eof,
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() || c == '\n' || c == '\r' || c == '\t' => {
                    self.advance();
                }
                Some('/') if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '/' => {
                    // Line comment: skip until newline
                    while let Some(c) = self.peek() {
                        if c == '\n' || c == '\r' {
                            break;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn read_string(&mut self) -> Token {
        // opening " already consumed by next_token
        let mut s = String::new();
        loop {
            match self.advance() {
                None => {
                    s.push('"');
                    return Token::StringLit(s);
                }
                Some('"') => return Token::StringLit(s),
                Some('\\') => match self.advance() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('r') => s.push('\r'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some(c) => {
                        s.push('\\');
                        s.push(c);
                    }
                    None => s.push('\\'),
                },
                Some(c) => s.push(c),
            }
        }
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        let mut is_float = false;
        // Read remaining digits
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else if c == '.' {
                is_float = true;
                s.push(c);
                self.advance();
            } else if c == 'e' || c == 'E' {
                is_float = true;
                s.push(c);
                self.advance();
                // Optional sign after e/E
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        s.push(sign);
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }
        if is_float {
            match s.parse::<f64>() {
                Ok(n) => Token::FloatLit(n),
                Err(_) => Token::Ident(s), // fallback: treat as ident
            }
        } else {
            match s.parse::<i64>() {
                Ok(n) => Token::IntLit(n),
                Err(_) => Token::Ident(s), // fallback: too large, treat as ident
            }
        }
    }

    fn read_ident_or_keyword(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '?' || c == '!' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        // Check if it's a keyword — must match exactly (case-sensitive)
        // We consider the first character as part of the identifier for keyword detection
        if is_keyword(&s) {
            Token::Keyword(s)
        } else {
            Token::Ident(s)
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        match self.advance() {
            None => Token::Eof,
            Some('(') => Token::LParen,
            Some(')') => Token::RParen,
            Some('{') => Token::LBrace,
            Some('}') => Token::RBrace,
            Some('[') => Token::LBracket,
            Some(']') => Token::RBracket,
            Some(',') => Token::Comma,
            Some(';') => Token::Semicolon,
            Some(':') => Token::Colon,
            Some('.') => Token::Dot,
            Some('=') => {
                if self.peek() == Some('>') {
                    self.advance();
                    Token::DoubleArrow
                } else {
                    Token::Equals
                }
            }
            Some('-') => {
                if self.peek() == Some('>') {
                    self.advance();
                    Token::Arrow
                } else {
                    // Could be negative number or just an ident
                    Token::Ident("-".to_string())
                }
            }
            Some('"') => self.read_string(),
            Some(c) if c.is_ascii_digit() => self.read_number(c),
            Some(c) if c.is_alphanumeric() || c == '_' => self.read_ident_or_keyword(c),
            Some(c) => Token::Ident(c.to_string()),
        }
    }
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "fn"
            | "let"
            | "match"
            | "if"
            | "reflect"
            | "curious"
            | "dream"
            | "edit"
            | "bind"
            | "bundle"
            | "permute"
            | "similarity"
            | "vector"
            | "string"
            | "import"
            | "module"
            | "seq"
            | "default"
    )
}

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut l = Lexer::new("( ) { } [ ] , ; : . = ->");
        assert_eq!(l.next_token(), Token::LParen);
        assert_eq!(l.next_token(), Token::RParen);
        assert_eq!(l.next_token(), Token::LBrace);
        assert_eq!(l.next_token(), Token::RBrace);
        assert_eq!(l.next_token(), Token::LBracket);
        assert_eq!(l.next_token(), Token::RBracket);
        assert_eq!(l.next_token(), Token::Comma);
        assert_eq!(l.next_token(), Token::Semicolon);
        assert_eq!(l.next_token(), Token::Colon);
        assert_eq!(l.next_token(), Token::Dot);
        assert_eq!(l.next_token(), Token::Equals);
        assert_eq!(l.next_token(), Token::Arrow);
        assert_eq!(l.next_token(), Token::Eof);
    }

    #[test]
    fn test_keywords() {
        let mut l = Lexer::new("fn let match reflect curious dream edit bind bundle permute similarity vector string import module");
        assert_eq!(l.next_token(), Token::Keyword("fn".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("let".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("match".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("reflect".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("curious".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("dream".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("edit".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("bind".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("bundle".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("permute".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("similarity".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("vector".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("string".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("import".to_string()));
        assert_eq!(l.next_token(), Token::Keyword("module".to_string()));
        assert_eq!(l.next_token(), Token::Eof);
    }

    #[test]
    fn test_identifiers() {
        let mut l = Lexer::new("hello foo_bar baz-123");
        assert_eq!(l.next_token(), Token::Ident("hello".to_string()));
        assert_eq!(l.next_token(), Token::Ident("foo_bar".to_string()));
        assert_eq!(l.next_token(), Token::Ident("baz-123".to_string()));
    }

    #[test]
    fn test_numbers() {
        let mut l = Lexer::new("42 3.14 -5");
        assert_eq!(l.next_token(), Token::IntLit(42));
        assert_eq!(l.next_token(), Token::FloatLit(3.14));
        assert_eq!(l.next_token(), Token::Ident("-".to_string()));
        assert_eq!(l.next_token(), Token::IntLit(5));
    }

    #[test]
    fn test_strings() {
        let mut l = Lexer::new("\"hello\" \"hello\\nworld\"");
        assert_eq!(l.next_token(), Token::StringLit("hello".to_string()));
        assert_eq!(l.next_token(), Token::StringLit("hello\nworld".to_string()));
    }

    #[test]
    fn test_comments() {
        let mut l = Lexer::new("// this is a comment\nfn");
        assert_eq!(l.next_token(), Token::Keyword("fn".to_string()));
        assert_eq!(l.next_token(), Token::Eof);
    }

    #[test]
    fn test_complex_expr() {
        let mut l = Lexer::new("bind(a, b)");
        assert_eq!(l.next_token(), Token::Keyword("bind".to_string()));
        assert_eq!(l.next_token(), Token::LParen);
        assert_eq!(l.next_token(), Token::Ident("a".to_string()));
        assert_eq!(l.next_token(), Token::Comma);
        assert_eq!(l.next_token(), Token::Ident("b".to_string()));
        assert_eq!(l.next_token(), Token::RParen);
        assert_eq!(l.next_token(), Token::Eof);
    }
}
