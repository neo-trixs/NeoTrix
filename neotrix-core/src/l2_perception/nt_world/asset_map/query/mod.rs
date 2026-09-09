//! NT-WORLD Asset Map: CSEQL 查询语法解析器
//!
//! 支持 FOFA 风格的结构化查询语法:
//! - `ip="192.168.1.0/24"`
//! - `port="80" && title*="admin"`
//! - `cert="example.com" && cert.is_valid=true`

use std::collections::HashMap;

/// 查询 AST 节点
#[derive(Debug, Clone, PartialEq)]
pub enum QueryExpr {
    /// 字段比较: field op value
    Compare {
        field: String,
        op: CompareOp,
        value: String,
    },
    /// 逻辑与
    And(Box<QueryExpr>, Box<QueryExpr>),
    /// 逻辑或
    Or(Box<QueryExpr>, Box<QueryExpr>),
    /// 逻辑非
    Not(Box<QueryExpr>),
    /// 范围查询: field>min && field<max
    Range {
        field: String,
        min: Option<i64>,
        max: Option<i64>,
    },
}

/// 比较操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    /// 精确匹配: =
    Eq,
    /// 不等于: !=
    Ne,
    /// 通配符匹配: *=
    Like,
    /// 大于: >
    Gt,
    /// 小于: <
    Lt,
    /// 大于等于: >=
    Ge,
    /// 小于等于: <=
    Le,
}

/// 解析错误
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("expected '{0}' at position {1}")]
    Expected(char, usize),
    #[error("invalid field name: {0}")]
    InvalidField(String),
    #[error("invalid operator: {0}")]
    InvalidOperator(String),
    #[error("unmatched parentheses")]
    UnmatchedParentheses,
}

/// 查询解析器
pub struct QueryParser {
    /// 输入字符串
    input: Vec<char>,
    /// 当前位置
    pos: usize,
}

impl QueryParser {
    /// 创建新的解析器
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// 解析查询字符串
    pub fn parse(input: &str) -> Result<QueryExpr, ParseError> {
        let mut parser = Self::new(input);
        let expr = parser.parse_or()?;
        Ok(expr)
    }

    /// 解析 OR 表达式 (最低优先级)
    fn parse_or(&mut self) -> Result<QueryExpr, ParseError> {
        let mut left = self.parse_and()?;

        while self.peek() == Some('|') && self.peek_next() == Some('|') {
            self.advance(2);
            self.skip_whitespace();
            let right = self.parse_and()?;
            left = QueryExpr::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// 解析 AND 表达式
    fn parse_and(&mut self) -> Result<QueryExpr, ParseError> {
        let mut left = self.parse_not()?;

        while self.peek() == Some('&') && self.peek_next() == Some('&') {
            self.advance(2);
            self.skip_whitespace();
            let right = self.parse_not()?;
            left = QueryExpr::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// 解析 NOT 表达式
    fn parse_not(&mut self) -> Result<QueryExpr, ParseError> {
        if self.peek() == Some('!') {
            self.advance(1);
            let expr = self.parse_primary()?;
            return Ok(QueryExpr::Not(Box::new(expr)));
        }
        self.parse_primary()
    }

    /// 解析 primary 表达式 (比较或括号)
    fn parse_primary(&mut self) -> Result<QueryExpr, ParseError> {
        self.skip_whitespace();

        // 括号
        if self.peek() == Some('(') {
            self.advance(1);
            let expr = self.parse_or()?;
            self.skip_whitespace();
            if self.peek() != Some(')') {
                return Err(ParseError::Expected(')', self.pos));
            }
            self.advance(1);
            return Ok(expr);
        }

        // 字段比较
        self.parse_compare()
    }

    /// 解析比较表达式
    fn parse_compare(&mut self) -> Result<QueryExpr, ParseError> {
        self.skip_whitespace();

        // 读取字段名
        let field = self.read_field()?;
        self.skip_whitespace();

        // 检查是否是范围查询
        if self.peek() == Some('>') || self.peek() == Some('<') {
            return self.parse_range(field);
        }

        // 读取操作符
        let op = self.read_op()?;
        self.skip_whitespace();

        // 读取值
        let value = self.read_value()?;

        Ok(QueryExpr::Compare { field, op, value })
    }

    /// 解析范围查询
    fn parse_range(&mut self, field: String) -> Result<QueryExpr, ParseError> {
        let mut min = None;
        let mut max = None;

        if self.peek() == Some('>') {
            self.advance(1);
            if self.peek() == Some('=') {
                self.advance(1);
            }
            self.skip_whitespace();
            let value = self.read_number()?;
            min = Some(value);
        }

        if self.peek() == Some('<') {
            self.advance(1);
            if self.peek() == Some('=') {
                self.advance(1);
            }
            self.skip_whitespace();
            let value = self.read_number()?;
            max = Some(value);
        }

        Ok(QueryExpr::Range { field, min, max })
    }

    /// 读取字段名
    fn read_field(&mut self) -> Result<String, ParseError> {
        let mut field = String::new();
        while let Some(&ch) = self.input.get(self.pos) {
            if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                field.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        if field.is_empty() {
            return Err(ParseError::InvalidField("empty".into()));
        }
        Ok(field)
    }

    /// 读取操作符
    fn read_op(&mut self) -> Result<CompareOp, ParseError> {
        match self.peek() {
            Some('=') => {
                self.advance(1);
                if self.peek() == Some('=') {
                    self.advance(1);
                }
                Ok(CompareOp::Eq)
            }
            Some('!') => {
                self.advance(1);
                if self.peek() != Some('=') {
                    return Err(ParseError::Expected('=', self.pos));
                }
                self.advance(1);
                Ok(CompareOp::Ne)
            }
            Some('*') => {
                self.advance(1);
                if self.peek() != Some('=') {
                    return Err(ParseError::Expected('=', self.pos));
                }
                self.advance(1);
                Ok(CompareOp::Like)
            }
            Some('>') => {
                self.advance(1);
                if self.peek() == Some('=') {
                    self.advance(1);
                    Ok(CompareOp::Ge)
                } else {
                    Ok(CompareOp::Gt)
                }
            }
            Some('<') => {
                self.advance(1);
                if self.peek() == Some('=') {
                    self.advance(1);
                    Ok(CompareOp::Le)
                } else {
                    Ok(CompareOp::Lt)
                }
            }
            _ => Err(ParseError::InvalidOperator(format!("{:?}", self.peek()))),
        }
    }

    /// 读取值 (支持引号和非引号)
    fn read_value(&mut self) -> Result<String, ParseError> {
        self.skip_whitespace();

        if self.peek() == Some('"') {
            // 引号内的值
            self.advance(1);
            let mut value = String::new();
            loop {
                match self.peek() {
                    Some('"') => {
                        self.advance(1);
                        return Ok(value);
                    }
                    Some('\\') => {
                        self.advance(1);
                        if let Some(ch) = self.peek() {
                            value.push(ch);
                            self.advance(1);
                        }
                    }
                    Some(ch) => {
                        value.push(ch);
                        self.advance(1);
                    }
                    None => return Err(ParseError::UnexpectedEof),
                }
            }
        } else {
            // 非引号值 (读到空格或运算符)
            let mut value = String::new();
            while let Some(&ch) = self.input.get(self.pos) {
                if ch == ' ' || ch == '&' || ch == '|' || ch == '(' || ch == ')' {
                    break;
                }
                value.push(ch);
                self.pos += 1;
            }
            Ok(value)
        }
    }

    /// 读取数字
    fn read_number(&mut self) -> Result<i64, ParseError> {
        let mut num_str = String::new();
        while let Some(&ch) = self.input.get(self.pos) {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        num_str.parse().map_err(|_| ParseError::InvalidOperator(num_str))
    }

    /// 查看当前字符
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    /// 查看下一个字符
    fn peek_next(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    /// 跳过空白
    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.input.get(self.pos) {
            if ch.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// 前进n个字符
    fn advance(&mut self, n: usize) {
        self.pos += n;
    }
}

/// 查询求值器 — 对资产数据执行查询
pub struct QueryEvaluator;

impl QueryEvaluator {
    /// 求值查询表达式
    pub fn evaluate(expr: &QueryExpr, asset: &HashMap<String, String>) -> bool {
        match expr {
            QueryExpr::Compare { field, op, value } => {
                let asset_value = asset.get(field.as_str()).map(|s| s.as_str()).unwrap_or("");
                match op {
                    CompareOp::Eq => asset_value == value.as_str(),
                    CompareOp::Ne => asset_value != value.as_str(),
                    CompareOp::Like => {
                        // 简单通配符匹配 (支持 * 前缀/后缀)
                        if value.starts_with('*') && value.ends_with('*') {
                            let pattern = &value[1..value.len()-1];
                            asset_value.contains(pattern)
                        } else if value.starts_with('*') {
                            let pattern = &value[1..];
                            asset_value.ends_with(pattern)
                        } else if value.ends_with('*') {
                            let pattern = &value[..value.len()-1];
                            asset_value.starts_with(pattern)
                        } else {
                            asset_value == value.as_str()
                        }
                    }
                    CompareOp::Gt => {
                        asset_value.parse::<i64>().map(|v| v > value.parse().unwrap_or(0)).unwrap_or(false)
                    }
                    CompareOp::Lt => {
                        asset_value.parse::<i64>().map(|v| v < value.parse().unwrap_or(0)).unwrap_or(false)
                    }
                    CompareOp::Ge => {
                        asset_value.parse::<i64>().map(|v| v >= value.parse().unwrap_or(0)).unwrap_or(false)
                    }
                    CompareOp::Le => {
                        asset_value.parse::<i64>().map(|v| v <= value.parse().unwrap_or(0)).unwrap_or(false)
                    }
                }
            }
            QueryExpr::And(left, right) => {
                Self::evaluate(left, asset) && Self::evaluate(right, asset)
            }
            QueryExpr::Or(left, right) => {
                Self::evaluate(left, asset) || Self::evaluate(right, asset)
            }
            QueryExpr::Not(expr) => {
                !Self::evaluate(expr, asset)
            }
            QueryExpr::Range { field, min, max } => {
                let asset_value = asset.get(field.as_str()).and_then(|s| s.parse::<i64>().ok());
                match asset_value {
                    Some(v) => {
                        let min_ok = min.map(|m| v >= m).unwrap_or(true);
                        let max_ok = max.map(|m| v <= m).unwrap_or(true);
                        min_ok && max_ok
                    }
                    None => false,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_compare() {
        let expr = QueryParser::parse(r#"ip="192.168.1.1""#).unwrap();
        let mut asset = HashMap::new();
        asset.insert("ip".into(), "192.168.1.1".into());
        assert!(QueryEvaluator::evaluate(&expr, &asset));
    }

    #[test]
    fn parse_and_query() {
        let expr = QueryParser::parse(r#"port="80" && title*="admin""#).unwrap();
        let mut asset = HashMap::new();
        asset.insert("port".into(), "80".into());
        asset.insert("title".into(), "Admin Dashboard".into());
        assert!(QueryEvaluator::evaluate(&expr, &asset));
    }

    #[test]
    fn parse_or_query() {
        let expr = QueryParser::parse(r#"port="80" || port="443""#).unwrap();
        let mut asset = HashMap::new();
        asset.insert("port".into(), "443".into());
        assert!(QueryEvaluator::evaluate(&expr, &asset));
    }

    #[test]
    fn parse_not_query() {
        let expr = QueryParser::parse(r#"!port="22""#).unwrap();
        let mut asset = HashMap::new();
        asset.insert("port".into(), "80".into());
        assert!(QueryEvaluator::evaluate(&expr, &asset));
    }

    #[test]
    fn parse_range_query() {
        let expr = QueryParser::parse("port>100 && port<1000").unwrap();
        let mut asset = HashMap::new();
        asset.insert("port".into(), "8080".into());
        assert!(QueryEvaluator::evaluate(&expr, &asset));
    }

    #[test]
    fn parse_wildcard_query() {
        let expr = QueryParser::parse(r#"title*="admin""#).unwrap();
        let mut asset = HashMap::new();
        asset.insert("title".into(), "Admin Login".into());
        assert!(QueryEvaluator::evaluate(&expr, &asset));
    }
}
