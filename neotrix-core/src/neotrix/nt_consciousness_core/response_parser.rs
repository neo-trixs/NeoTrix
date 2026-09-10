//! 响应解析器 (ResponseParser)
//! 
//! 吸收自 NERV-BREAK-5.6 的通用响应解析机制
//! - SSE 解析
//! - JSON 解析
//! - 流式响应合并

use serde::{Deserialize, Serialize};

/// 响应解析器
pub struct ResponseParser;

/// 解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// 思考内容
    pub thinking: String,
    /// 回复内容
    pub reply: String,
    /// 原始内容
    pub raw: String,
}

impl ResponseParser {
    /// 解析响应
    pub fn parse(body: &[u8]) -> ParseResult {
        let text = String::from_utf8_lossy(body);
        let mut thinking = Vec::new();
        let mut reply = Vec::new();

        // 尝试 JSON 解析
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            Self::extract_from_json(&json, &mut thinking, &mut reply);
        }

        // 尝试 SSE 解析
        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data:") {
                let data = data.trim();
                if data == "[DONE]" || data.is_empty() {
                    continue;
                }
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                    Self::extract_from_json(&event, &mut thinking, &mut reply);
                }
            }
        }

        // 合并重复块
        let thinking_str = Self::merge_chunks(&thinking);
        let reply_str = Self::merge_chunks(&reply);

        ParseResult {
            thinking: thinking_str,
            reply: reply_str,
            raw: text.to_string(),
        }
    }

    /// 从 JSON 提取内容
    fn extract_from_json(
        obj: &serde_json::Value,
        thinking: &mut Vec<String>,
        reply: &mut Vec<String>,
    ) {
        match obj {
            serde_json::Value::String(s) => {
                if !s.is_empty() {
                    reply.push(s.clone());
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    Self::extract_from_json(item, thinking, reply);
                }
            }
            serde_json::Value::Object(map) => {
                // 检查是否是推理内容
                let _is_reasoning = map.get("type")
                    .or_else(|| map.get("role"))
                    .or_else(|| map.get("name"))
                    .map(|v| {
                        let label = v.as_str().unwrap_or("").to_lowercase();
                        label.contains("reasoning") || label.contains("thinking") || label.contains("thought")
                    })
                    .unwrap_or(false);

                // 提取 choices
                if let Some(choices) = map.get("choices").and_then(|v| v.as_array()) {
                    for choice in choices {
                        for key in &["message", "delta", "text", "content"] {
                            if let Some(val) = choice.get(key) {
                                Self::extract_from_json(val, thinking, reply);
                            }
                        }
                    }
                }

                // 提取文本字段
                for key in &["output_text", "content", "text", "message", "result", "answer", "completion"] {
                    if let Some(val) = map.get(*key) {
                        Self::extract_from_json(val, thinking, reply);
                    }
                }

                // 提取包装字段
                for key in &["output", "delta", "part", "response", "data", "body", "payload"] {
                    if let Some(val) = map.get(*key) {
                        Self::extract_from_json(val, thinking, reply);
                    }
                }
            }
            _ => {}
        }
    }

    /// 合并重复块
    fn merge_chunks(chunks: &[String]) -> String {
        if chunks.is_empty() {
            return String::new();
        }

        let mut merged = String::new();
        for chunk in chunks {
            if merged.is_empty() {
                merged = chunk.clone();
                continue;
            }

            if chunk == &merged {
                continue;
            }

            if chunk.starts_with(&merged) {
                merged = chunk.clone();
                continue;
            }

            if merged.starts_with(chunk) {
                continue;
            }

            merged.push_str(chunk);
        }

        // 检测重复模式
        let stripped = merged.trim();
        let len = stripped.len();
        
        for size in 4..=(len / 2) {
            if len % size == 0 {
                let repeats = len / size;
                let piece = &stripped[..size];
                if (repeats >= 3 || piece.len() >= 12) 
                    && piece.repeat(repeats) == stripped {
                    return piece.to_string();
                }
            }
        }

        stripped.to_string()
    }

    /// 从纯文本提取
    fn extract_from_text(text: &str) -> Vec<String> {
        let mut result = Vec::new();
        
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("data:") || line.starts_with("event:") || line.starts_with("id:") {
                continue;
            }
            if line.starts_with('{') || line.starts_with('[') {
                continue;
            }
            result.push(line.to_string());
        }
        
        result
    }
}

impl std::fmt::Display for ParseResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.thinking.is_empty() {
            writeln!(f, "=== THINKING ===")?;
            writeln!(f, "{}", self.thinking)?;
        }
        if !self.reply.is_empty() {
            writeln!(f, "=== REPLY ===")?;
            writeln!(f, "{}", self.reply)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let json = r#"{"content": "Hello world"}"#;
        let result = ResponseParser::parse(json.as_bytes());
        assert_eq!(result.reply, "Hello world");
    }

    #[test]
    fn test_parse_sse() {
        let sse = "data: {\"content\": \"Hello\"}\ndata: [DONE]\n";
        let result = ResponseParser::parse(sse.as_bytes());
        assert!(result.reply.contains("Hello"));
    }
}
