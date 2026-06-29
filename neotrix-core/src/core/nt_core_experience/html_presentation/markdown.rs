/// Minimal markdown-to-HTML converter: handles **bold**, *italic*, `code`, and paragraphs.
pub(super) fn markdown_to_html(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut html = String::new();
    html.push_str("<p>");
    let mut chars = trimmed.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    let mut content = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc == '*' {
                            chars.next();
                            if chars.peek() == Some(&'*') {
                                chars.next();
                                break;
                            } else {
                                content.push('*');
                                continue;
                            }
                        }
                        content.push(chars.next().unwrap());
                    }
                    html.push_str("<strong>");
                    html.push_str(&html_escape(&content));
                    html.push_str("</strong>");
                } else {
                    let mut content = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc == '*' {
                            chars.next();
                            break;
                        }
                        content.push(chars.next().unwrap());
                    }
                    html.push_str("<em>");
                    html.push_str(&html_escape(&content));
                    html.push_str("</em>");
                }
            }
            '`' => {
                let mut content = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc == '`' {
                        chars.next();
                        break;
                    }
                    content.push(chars.next().unwrap());
                }
                html.push_str("<code>");
                html.push_str(&html_escape(&content));
                html.push_str("</code>");
            }
            '\n' => {
                html.push_str("</p>\n<p>");
            }
            other => {
                html.push(other);
            }
        }
    }
    html.push_str("</p>");
    html
}

pub(super) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
