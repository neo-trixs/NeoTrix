pub mod ast;
pub mod lexer;
pub mod parser;

use ast::NeExpr;
use parser::Parser;

/// Parse Ne surface source code into NeIR (S-expression AST).
pub fn parse(source: &str) -> Result<NeExpr, String> {
    Parser::new(source).parse()
}

/// Parse Ne source and pretty-print as canonical S-expression.
pub fn parse_to_sexpr(source: &str) -> Result<String, String> {
    parse(source).map(|e| e.to_sexpr())
}

#[cfg(test)]
mod tests;
