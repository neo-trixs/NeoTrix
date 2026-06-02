use ratatui::text::{Line, Span};
use ratatui::style::{Style, Color, Modifier};

/// 渲染 Markdown 文本为 ratatui Lines
pub fn render_markdown(text: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_content = String::new();

    for raw_line in text.lines() {
        let line = raw_line.trim_end();

        if let Some(code_part) = line.strip_prefix("```") {
        if in_code_block {
            let lang = code_lang.to_lowercase();
            for code_line in code_content.lines() {
                let spans = if lang.contains("rust") || lang.contains("rs") {
                    highlight_rust_line(code_line)
                } else if lang.contains("python") || lang.contains("py") {
                    highlight_python_line(code_line)
                } else if lang.contains("javascript") || lang.contains("js") || lang.contains("typescript") || lang.contains("ts") {
                    highlight_javascript_line(code_line)
                } else if lang.contains("json") {
                    highlight_json_line(code_line)
                } else if lang.contains("sh") || lang.contains("bash") || lang.contains("zsh") || lang.contains("shell") {
                    highlight_shell_line(code_line)
                } else {
                    vec![Span::styled(code_line.to_string(), Style::default().fg(Color::Cyan))]
                };
                let mut line_spans = vec![Span::raw("  ")];
                line_spans.extend(spans);
                lines.push(Line::from(line_spans));
            }
                code_content.clear();
                in_code_block = false;
                code_lang.clear();
            } else {
                in_code_block = true;
                code_lang = code_part.trim().to_string();
                if !code_lang.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!(" [{}.]", code_lang),
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                        ),
                    ]));
                }
            }
            continue;
        }

        if in_code_block {
            code_content.push_str(line);
            code_content.push('\n');
            continue;
        }

        if line == "---" || line == "***" || line == "___" {
            lines.push(Line::from(vec![
                Span::styled(
                    "\u{2500}".repeat(50),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            continue;
        }

        lines.push(Line::from(render_inline(line)));
    }

    if in_code_block && !code_content.is_empty() {
        for code_line in code_content.lines() {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}", code_line),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }
    }

    lines
}

fn render_inline(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '`' => {
                let mut code = String::new();
                while let Some(&next) = chars.peek() {
                    if next == '`' {
                        chars.next();
                        break;
                    }
                    code.push(next);
                    chars.next();
                }
                spans.push(Span::styled(code, Style::default().fg(Color::Green)));
            }
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut bold = String::new();
                while let Some(&n) = chars.peek() {
                    if n == '*' {
                        chars.next();
                        if chars.peek() == Some(&'*') {
                            chars.next();
                            break;
                        }
                        bold.push('*');
                    } else {
                        bold.push(n);
                        chars.next();
                    }
                }
                spans.push(Span::styled(bold, Style::default().add_modifier(Modifier::BOLD)));
            }
            '#' if spans.is_empty() && text.trim_start().starts_with('#') => {
                let mut level = 1;
                while chars.peek() == Some(&'#') {
                    chars.next();
                    level += 1;
                }
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
                let mut heading = String::new();
                for c in chars.by_ref() {
                    heading.push(c);
                }
                let style = match level {
                    1 => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    2 => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    _ => Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                };
                spans.push(Span::styled(heading, style));
                break;
            }
            '-' | '*' | '+' if spans.is_empty() && text.trim_start().starts_with(['-', '*', '+']) => {
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
                let mut item = String::new();
                for c in chars.by_ref() {
                    item.push(c);
                }
                spans.push(Span::styled(format!(" \u{2022} {}", item), Style::default()));
                break;
            }
            _ => {
                let mut normal = String::new();
                normal.push(ch);
                while let Some(&next) = chars.peek() {
                    if next == '`' || next == '*' {
                        break;
                    }
                    normal.push(next);
                    chars.next();
                }
                spans.push(Span::raw(normal));
            }
        }
    }

    spans
}

fn highlight_rust_line(line: &str) -> Vec<Span<'static>> {
    let keywords = [
        "fn", "let", "mut", "if", "else", "for", "while", "loop", "match",
        "return", "true", "false", "Some", "None", "Ok", "Err", "pub",
        "use", "mod", "struct", "enum", "impl", "trait", "async", "await",
        "move", "ref", "where", "type", "const", "static", "unsafe",
        "self", "super", "crate", "in", "as", "break", "continue",
    ];

    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '/' if chars.peek() == Some(&'/') => {
                let mut comment = String::from("//");
                for c in chars.by_ref() {
                    comment.push(c);
                }
                spans.push(Span::styled(comment, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)));
                break;
            }
            '"' => {
                let mut s = String::from("\"");
                for c in chars.by_ref() {
                    s.push(c);
                    if c == '"' && s.chars().rev().nth(1) != Some('\\') { break; }
                }
                spans.push(Span::styled(s, Style::default().fg(Color::Green)));
            }
            c if c.is_ascii_digit() || (c == '-' && chars.peek().is_some_and(|p| p.is_ascii_digit())) => {
                let mut num = if c == '-' { "-".to_string() } else { c.to_string() };
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_digit() || n == '.' || n == '_' || n == 'x' || n.is_ascii_hexdigit() {
                        num.push(n);
                        chars.next();
                    } else { break; }
                }
                spans.push(Span::styled(num, Style::default().fg(Color::Magenta)));
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut word = c.to_string();
                while let Some(&n) = chars.peek() {
                    if n.is_alphanumeric() || n == '_' { word.push(n); chars.next(); }
                    else { break; }
                }
                if keywords.contains(&word.as_str()) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Cyan)));
                } else if word == "self" || word == "Self" {
                    spans.push(Span::styled(word, Style::default().fg(Color::Yellow)));
                } else {
                    spans.push(Span::raw(word));
                }
            }
            '#' | '!' if line.trim_start().starts_with("#!") || line.trim_start().starts_with("#[") => {
                let mut attr = ch.to_string();
                for c in chars.by_ref() {
                    attr.push(c);
                }
                spans.push(Span::styled(attr, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)));
                break;
            }
            _ => {
                spans.push(Span::raw(ch.to_string()));
            }
        }
    }

    spans
}

fn highlight_python_line(line: &str) -> Vec<Span<'static>> {
    let keywords = [
        "def", "class", "if", "elif", "else", "for", "while", "break",
        "continue", "return", "yield", "import", "from", "as", "with",
        "try", "except", "finally", "raise", "pass", "None", "True",
        "False", "and", "or", "not", "in", "is", "lambda", "async",
        "await", "self", "print",
    ];
    let builtins = ["len", "range", "int", "str", "float", "list", "dict",
        "set", "tuple", "type", "isinstance", "enumerate", "zip", "map",
        "filter", "sorted", "open", "super", "hasattr", "getattr"];

    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '#' => {
                let mut comment = String::from("#");
                for c in chars.by_ref() { comment.push(c); }
                spans.push(Span::styled(comment, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)));
                break;
            }
            '"' | '\'' => {
                let quote = ch;
                let mut s = String::from(quote.to_string());
                let triple = chars.peek() == Some(&quote);
                if triple {
                    s.push(quote); chars.next(); s.push(quote); chars.next();
                }
                while let Some(c) = chars.next() {
                    s.push(c);
                    if c == '\\' {
                        if let Some(n) = chars.next() { s.push(n); }
                    } else if triple && s.ends_with(&format!("{0}{0}{0}", quote)) && s.len() > 3 {
                        break;
                    } else if !triple && c == quote {
                        break;
                    }
                }
                spans.push(Span::styled(s, Style::default().fg(Color::Green)));
            }
            c if c.is_ascii_digit() || (c == '-' && chars.peek().is_some_and(|p| p.is_ascii_digit())) => {
                let mut num = if c == '-' { "-".to_string() } else { c.to_string() };
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_digit() || n == '.' || n == '_' { num.push(n); chars.next(); }
                    else { break; }
                }
                spans.push(Span::styled(num, Style::default().fg(Color::Magenta)));
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut word = c.to_string();
                while let Some(&n) = chars.peek() {
                    if n.is_alphanumeric() || n == '_' { word.push(n); chars.next(); }
                    else { break; }
                }
                if keywords.contains(&word.as_str()) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                } else if builtins.contains(&word.as_str()) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Cyan)));
                } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Cyan)));
                } else {
                    spans.push(Span::raw(word));
                }
            }
            _ => { spans.push(Span::raw(ch.to_string())); }
        }
    }
    spans
}

fn highlight_javascript_line(line: &str) -> Vec<Span<'static>> {
    let keywords = [
        "function", "const", "let", "var", "if", "else", "for", "while",
        "do", "switch", "case", "break", "continue", "return", "throw",
        "try", "catch", "finally", "new", "delete", "typeof", "instanceof",
        "class", "extends", "super", "import", "export", "default", "from",
        "as", "async", "await", "yield", "this", "null", "undefined",
        "true", "false", "of", "in",
    ];
    let builtins = ["console", "document", "window", "Math", "JSON",
        "Array", "Object", "String", "Number", "Boolean", "Promise",
        "Map", "Set", "Symbol", "RegExp", "Date", "Error"];

    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '/' if chars.peek() == Some(&'/') => {
                let mut comment = String::from("//");
                for c in chars.by_ref() { comment.push(c); }
                spans.push(Span::styled(comment, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)));
                break;
            }
            '"' | '\'' | '`' => {
                let quote = ch;
                let mut s = String::from(quote.to_string());
                while let Some(c) = chars.next() {
                    s.push(c);
                    if c == '\\' { if let Some(n) = chars.next() { s.push(n); } }
                    else if c == quote { break; }
                }
                spans.push(Span::styled(s, Style::default().fg(Color::Green)));
            }
            c if c.is_ascii_digit() => {
                let mut num = c.to_string();
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_digit() || n == '.' || n == 'x' { num.push(n); chars.next(); }
                    else { break; }
                }
                spans.push(Span::styled(num, Style::default().fg(Color::Magenta)));
            }
            c if c.is_alphabetic() || c == '_' || c == '$' => {
                let mut word = c.to_string();
                while let Some(&n) = chars.peek() {
                    if n.is_alphanumeric() || n == '_' || n == '$' { word.push(n); chars.next(); }
                    else { break; }
                }
                if keywords.contains(&word.as_str()) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                } else if builtins.contains(&word.as_str()) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Cyan)));
                } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Cyan)));
                } else {
                    spans.push(Span::raw(word));
                }
            }
            _ => { spans.push(Span::raw(ch.to_string())); }
        }
    }
    spans
}

fn highlight_json_line(line: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                let mut s = String::from("\"");
                while let Some(c) = chars.next() {
                    s.push(c);
                    if c == '\\' { if let Some(n) = chars.next() { s.push(n); } }
                    else if c == '"' { break; }
                }
                let trimmed = line.trim_start();
                let is_key = s.len() > 2 && trimmed.starts_with('"') && trimmed[1..].contains("\":");
                spans.push(Span::styled(s, Style::default().fg(if is_key { Color::Yellow } else { Color::Green })));
            }
            c if c.is_ascii_digit() || c == '-' => {
                let mut num = c.to_string();
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_digit() || n == '.' || n == 'e' || n == 'E' || n == '-' || n == '+' { num.push(n); chars.next(); }
                    else { break; }
                }
                spans.push(Span::styled(num, Style::default().fg(Color::Magenta)));
            }
            't' if line.trim_start().starts_with("true") => {
                spans.push(Span::styled("true".to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
                for _ in 0..3 { chars.next(); }
            }
            'f' if line.trim_start().starts_with("false") => {
                spans.push(Span::styled("false".to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
                for _ in 0..4 { chars.next(); }
            }
            'n' if line.trim_start().starts_with("null") => {
                spans.push(Span::styled("null".to_string(), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)));
                for _ in 0..3 { chars.next(); }
            }
            _ => { spans.push(Span::raw(ch.to_string())); }
        }
    }
    spans
}

fn highlight_shell_line(line: &str) -> Vec<Span<'static>> {
    let keywords = [
        "if", "then", "else", "elif", "fi", "for", "while", "do", "done",
        "case", "esac", "function", "return", "exit", "export", "local",
        "source", "echo", "printf", "read", "set", "unset", "declare",
        "typeset", "select", "until",
    ];

    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '#' => {
                let mut comment = String::from("#");
                for c in chars.by_ref() { comment.push(c); }
                spans.push(Span::styled(comment, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)));
                break;
            }
            '"' | '\'' => {
                let quote = ch;
                let mut s = String::from(quote.to_string());
                while let Some(c) = chars.next() {
                    s.push(c);
                    if c == '\\' { if let Some(n) = chars.next() { s.push(n); } }
                    else if c == quote { break; }
                }
                spans.push(Span::styled(s, Style::default().fg(Color::Green)));
            }
            '$' => {
                let mut var = String::from("$");
                if chars.peek() == Some(&'{') {
                    var.push('{'); chars.next();
                    while let Some(&n) = chars.peek() {
                        if n == '}' { var.push('}'); chars.next(); break; }
                        var.push(n); chars.next();
                    }
                } else {
                    while let Some(&n) = chars.peek() {
                        if n.is_alphanumeric() || n == '_' { var.push(n); chars.next(); }
                        else { break; }
                    }
                }
                spans.push(Span::styled(var, Style::default().fg(Color::Magenta)));
            }
            c if c.is_ascii_digit() => {
                let mut num = c.to_string();
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_digit() || n == '.' { num.push(n); chars.next(); }
                    else { break; }
                }
                spans.push(Span::styled(num, Style::default().fg(Color::Magenta)));
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut word = c.to_string();
                while let Some(&n) = chars.peek() {
                    if n.is_alphanumeric() || n == '_' || n == '-' { word.push(n); chars.next(); }
                    else { break; }
                }
                if keywords.contains(&word.as_str()) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                } else if line.trim_start().split_whitespace().next().map_or(false, |cmd| word == cmd) {
                    spans.push(Span::styled(word, Style::default().fg(Color::Cyan)));
                } else {
                    spans.push(Span::raw(word));
                }
            }
            _ => { spans.push(Span::raw(ch.to_string())); }
        }
    }
    spans
}

pub fn render_thinking_block(blocks: &[String]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        " \u{250c}\u{2500} thinking \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        Style::default().fg(Color::Blue).add_modifier(Modifier::ITALIC),
    )));
    for block in blocks {
        lines.push(Line::from(Span::styled(
            format!(" \u{2502} {}", block),
            Style::default().fg(Color::Blue).add_modifier(Modifier::ITALIC),
        )));
    }
    lines.push(Line::from(Span::styled(
        " \u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        Style::default().fg(Color::Blue).add_modifier(Modifier::ITALIC),
    )));
    lines
}

pub fn role_style(role: &str) -> Style {
    match role {
        "user" => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        "assistant" => Style::default().fg(Color::Green),
        "system" => Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
        "error" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        _ => Style::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_plain_text() {
        let lines = render_markdown("Hello world");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_render_code_block() {
        let text = "```rust\nfn main() {\n    println!(\"hi\");\n}\n```";
        let lines = render_markdown(text);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_render_bold() {
        let spans = render_inline("Hello **world** here");
        assert!(spans.len() >= 2);
    }

    #[test]
    fn test_render_inline_code() {
        let spans = render_inline("use `std::fs` module");
        assert!(spans.len() >= 2);
        let code_span = spans.iter().find(|s| s.content == "std::fs");
        assert!(code_span.is_some(), "Could not find `std::fs` in spans: {:?}", spans);
    }

    #[test]
    fn test_render_heading() {
        let lines = render_markdown("# Title\n## Subtitle\nBody");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_render_list() {
        let lines = render_markdown("- item1\n- item2\n- item3");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_render_separator() {
        let lines = render_markdown("before\n---\nafter");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_role_style() {
        let s = role_style("user");
        assert!(s.fg.is_some());
        let s2 = role_style("unknown");
        assert!(s2.fg.is_none());
    }

    #[test]
    fn test_python_highlighting() {
        let spans = highlight_python_line("def hello():");
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_json_highlighting() {
        let spans = highlight_json_line("  \"key\": \"value\"");
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_shell_highlighting() {
        let spans = highlight_shell_line("echo hello");
        assert!(!spans.is_empty());
    }
}
