use crate::neotrix::nt_io_provider::types::Message;

pub struct ConversationCompressor {
    max_messages_before_compress: usize,
    summary_prompt: String,
}

impl ConversationCompressor {
    pub fn new(max: usize) -> Self {
        Self {
            max_messages_before_compress: max,
            summary_prompt: "The following is a summary of the earlier conversation.".to_string(),
        }
    }

    pub fn with_summary_prompt(mut self, prompt: &str) -> Self {
        self.summary_prompt = prompt.to_string();
        self
    }

    pub fn should_compress(&self, message_count: usize) -> bool {
        message_count > self.max_messages_before_compress
    }

    pub fn compress(messages: &[Message]) -> Vec<Message> {
        if messages.len() <= 2 {
            return messages.to_vec();
        }
        let keep_count = messages.len() / 2;
        let (compressible, keep) = messages.split_at(messages.len() - keep_count);
        let summary_text = compressible
            .iter()
            .map(|m| match m.role {
                crate::neotrix::nt_io_provider::types::Role::User => {
                    format!("User: {}", m.content)
                }
                crate::neotrix::nt_io_provider::types::Role::Assistant => {
                    format!("Assistant: {}", m.content)
                }
                crate::neotrix::nt_io_provider::types::Role::System => {
                    format!("System: {}", m.content)
                }
                crate::neotrix::nt_io_provider::types::Role::Tool => {
                    format!("Tool: {}", m.content)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let summary_message = Message {
            role: crate::neotrix::nt_io_provider::types::Role::System,
            content: format!("Summary of earlier conversation:\n{}", summary_text),
            tool_calls: None,
            tool_call_id: None,
        };
        let mut result = vec![summary_message];
        result.extend_from_slice(keep);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_io_provider::types::Role;

    #[test]
    fn test_no_compress_below_threshold() {
        let compressor = ConversationCompressor::new(10);
        assert!(!compressor.should_compress(5));
    }

    #[test]
    fn test_should_compress_above_threshold() {
        let compressor = ConversationCompressor::new(10);
        assert!(compressor.should_compress(15));
    }

    #[test]
    fn test_compress_empty_or_single() {
        let empty: Vec<Message> = vec![];
        let result = ConversationCompressor::compress(&empty);
        assert_eq!(result.len(), 0);

        let single = vec![Message {
            role: Role::User,
            content: "hi".to_string(),
            tool_calls: None,
            tool_call_id: None,
        }];
        let result = ConversationCompressor::compress(&single);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_compress_preserves_recent_messages() {
        let mut messages = vec![];
        for i in 0..6 {
            messages.push(Message {
                role: if i % 2 == 0 {
                    Role::User
                } else {
                    Role::Assistant
                },
                content: format!("message {}", i),
                tool_calls: None,
                tool_call_id: None,
            });
        }
        let result = ConversationCompressor::compress(&messages);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].role, Role::System);
        assert!(result[0].content.contains("message 0"));
        assert_eq!(result[1].content, "message 3");
        assert_eq!(result[3].content, "message 5");
    }

    #[test]
    fn test_compress_two_messages() {
        let messages = vec![
            Message {
                role: Role::User,
                content: "hello".to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::Assistant,
                content: "world".to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        let result = ConversationCompressor::compress(&messages);
        assert_eq!(result.len(), 2);
    }
}
