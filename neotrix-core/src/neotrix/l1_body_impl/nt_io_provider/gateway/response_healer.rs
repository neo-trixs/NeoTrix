/// G: Response Healing — 修复 LLM 输出的畸形 JSON 响应。
/// 修复链: 提取 (```json 围栏 / 散文包裹) → 去尾部逗号 → 闭合未闭合括号。
/// 无法修复时返回原始文本; heal / unrepairable 计数器暴露给遥测。
#[derive(Debug, Default)]
pub struct ResponseHealer {
    heal_count: u64,
    unrepairable_count: u64,
}

impl ResponseHealer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn heal(&mut self, raw: &str) -> String {
        // 已是合法 JSON → 原样返回 (不计数为修复)
        if serde_json::from_str::<serde_json::Value>(raw).is_ok() {
            return raw.to_string();
        }
        let extracted = self.extract_json(raw);
        let trimmed = self.trim_trailing_commas(&extracted);
        let closed = self.close_unclosed(&trimmed);
        if serde_json::from_str::<serde_json::Value>(&closed).is_ok() {
            self.heal_count += 1;
            return closed;
        }
        self.unrepairable_count += 1;
        raw.to_string()
    }

    /// 提取 JSON: 优先 ```json 围栏, 否则取首个 `{`/`[` 到深度归零的闭合区间
    fn extract_json(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        // 1. ```json ... ``` 围栏提取
        if let Some(fence) = trimmed.find("```json") {
            let after = &trimmed[fence + "```json".len()..];
            let content = match after.find("```") {
                Some(end) => &after[..end],
                None => after,
            };
            let c = content.trim();
            return if c.is_empty() {
                raw.to_string()
            } else {
                c.to_string()
            };
        }
        // 2. 首个 `{` / `[` → 深度归零的 JSON 区间
        let chars: Vec<char> = trimmed.chars().collect();
        let mut start = None;
        for (i, c) in chars.iter().enumerate() {
            if *c == '{' || *c == '[' {
                start = Some(i);
                break;
            }
        }
        let start = match start {
            Some(s) => s,
            None => return raw.to_string(),
        };
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escaped = false;
        let mut end = chars.len();
        for (i, c) in chars.iter().enumerate().skip(start) {
            if in_string {
                if escaped {
                    escaped = false;
                } else if *c == '\\' {
                    escaped = true;
                } else if *c == '"' {
                    in_string = false;
                }
                continue;
            }
            match *c {
                '"' => in_string = true,
                '{' | '[' => depth += 1,
                '}' | ']' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        chars[start..end].iter().collect::<String>().trim().to_string()
    }

    /// 去除对象/数组内的尾部逗号 (字符串感知, 保持 UTF-8 内容不变)
    fn trim_trailing_commas(&self, s: &str) -> String {
        let chars: Vec<char> = s.chars().collect();
        let n = chars.len();
        let mut out = String::with_capacity(s.len());
        let mut i = 0;
        let mut in_string = false;
        let mut escaped = false;
        while i < n {
            let c = chars[i];
            if in_string {
                out.push(c);
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                i += 1;
                continue;
            }
            if c == '"' {
                in_string = true;
                out.push('"');
                i += 1;
                continue;
            }
            if c == ',' {
                let mut j = i + 1;
                while j < n && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < n && (chars[j] == '}' || chars[j] == ']') {
                    i += 1;
                    continue;
                }
            }
            out.push(c);
            i += 1;
        }
        out
    }

    /// 按栈式配对闭合未闭合的 `{` / `[` (字符串感知)
    fn close_unclosed(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 4);
        let mut stack: Vec<char> = Vec::new();
        let mut in_string = false;
        let mut escaped = false;
        for c in s.chars() {
            if in_string {
                out.push(c);
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                continue;
            }
            match c {
                '"' => {
                    in_string = true;
                    out.push('"');
                }
                '{' => {
                    stack.push('{');
                    out.push('{');
                }
                '[' => {
                    stack.push('[');
                    out.push('[');
                }
                '}' => {
                    if stack.last() == Some(&'{') {
                        stack.pop();
                        out.push('}');
                    } else if stack.last() == Some(&'[') {
                        // 栈顶为 `[`: 先补其正确闭合符, 再处理当前 `}`
                        stack.pop();
                        out.push(']');
                        stack.pop();
                        out.push('}');
                    }
                    // 栈空时多余的 `}` 跳过, 避免输出畸形
                }
                ']' => {
                    if stack.last() == Some(&'[') {
                        stack.pop();
                        out.push(']');
                    } else if stack.last() == Some(&'{') {
                        stack.pop();
                        out.push('}');
                        stack.pop();
                        out.push(']');
                    }
                }
                _ => out.push(c),
            }
        }
        while let Some(open) = stack.pop() {
            out.push(match open {
                '{' => '}',
                _ => ']',
            });
        }
        out
    }

    pub fn heal_count(&self) -> u64 {
        self.heal_count
    }

    pub fn unrepairable_count(&self) -> u64 {
        self.unrepairable_count
    }
}