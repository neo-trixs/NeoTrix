use super::types::{Message, Role};

/// Remove orphan tool results that have no preceding assistant message.
/// Simple heuristic: remove Tool-role messages not preceded by Assistant.
pub fn sanitize_history(messages: &mut Vec<Message>) {
    let mut i = 0;
    while i < messages.len() {
        if messages[i].role == Role::Tool {
            let has_preceding_assistant = messages[..i]
                .iter()
                .rev()
                .any(|m| m.role == Role::Assistant);
            if !has_preceding_assistant {
                messages.remove(i);
                continue;
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool_msg(content: &str) -> Message {
        Message {
            role: Role::Tool,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn user_msg(content: &str) -> Message {
        Message {
            role: Role::User,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn assistant_msg(content: &str) -> Message {
        Message {
            role: Role::Assistant,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    #[test]
    fn test_orphaned_tool_result_removed() {
        let mut msgs = vec![user_msg("hello"), tool_msg("result")];
        sanitize_history(&mut msgs);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, Role::User);
    }

    #[test]
    fn test_tool_with_preceding_assistant_preserved() {
        let mut msgs = vec![
            user_msg("hello"),
            assistant_msg("let me check"),
            tool_msg("result"),
        ];
        sanitize_history(&mut msgs);
        assert_eq!(msgs.len(), 3);
    }

    #[test]
    fn test_chain_preserved() {
        let mut msgs = vec![
            user_msg("q1"),
            assistant_msg("checking"),
            tool_msg("r1"),
            assistant_msg("done"),
            user_msg("q2"),
            assistant_msg("checking more"),
            tool_msg("r2"),
            assistant_msg("final"),
        ];
        sanitize_history(&mut msgs);
        assert_eq!(msgs.len(), 8);
    }

    #[test]
    fn test_no_tool_calls_unchanged() {
        let mut msgs = vec![
            user_msg("hello"),
            assistant_msg("hi"),
            user_msg("how are you"),
            assistant_msg("fine"),
        ];
        let len = msgs.len();
        sanitize_history(&mut msgs);
        assert_eq!(msgs.len(), len);
    }
}
