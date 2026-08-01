use super::types::{Message, Role};

pub fn sanitize_history(messages: &mut Vec<Message>) {
    let mut i = 0;
    while i < messages.len() {
        if messages[i].role == Role::Tool {
            // A Tool result is only meaningful if the immediately-preceding
            // Assistant chain declared the exact tool_call_id this result
            // answers. Merely seeing *some* Assistant with tool_calls is not
            // enough: a leftover result for a call the model never declared
            // (e.g. after a mid-turn message was dropped) would be forwarded
            // to the provider and rejected.
            let call_id = messages[i].tool_call_id.as_deref();
            let has_preceding_call = messages[..i].iter().rev().any(|m| {
                m.role == Role::Assistant
                    && m.tool_calls.as_ref()
                        .is_some_and(|calls| calls.iter().any(|tc| tc.id.as_str() == call_id.unwrap_or("")))
            });
            if !has_preceding_call {
                messages.remove(i);
                continue;
            }
        }
        if messages[i].role == Role::Assistant {
            if let Some(tool_calls) = messages[i].tool_calls.as_ref() {
                let ids: Vec<&str> = tool_calls.iter()
                    .map(|tc| tc.id.as_str()).collect();
                let has_all = ids.iter().all(|id| {
                    messages[i + 1..].iter().any(|m| {
                        m.role == Role::Tool && m.tool_call_id.as_deref() == Some(id)
                    })
                });
                if !has_all {
                    messages.remove(i);
                    continue;
                }
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_io_provider::types::{ToolCallInfo, ToolCallFunction};

    fn tool_msg(content: &str, call_id: &str) -> Message {
        Message {
            role: Role::Tool,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: Some(call_id.to_string()),
        }
    }

    fn assistant_with_calls(ids: &[&str]) -> Message {
        Message {
            role: Role::Assistant,
            content: String::new(),
            tool_calls: Some(ids.iter().map(|id| ToolCallInfo {
                id: id.to_string(),
                call_type: "function".to_string(),
                function: ToolCallFunction {
                    name: "test".to_string(),
                    arguments: "{}".to_string(),
                },
            }).collect()),
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
        let mut msgs = vec![
            user_msg("hello"),
            tool_msg("result", "call_1"),
        ];
        sanitize_history(&mut msgs);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, Role::User);
    }

    #[test]
    fn test_orphaned_tool_call_removed() {
        let mut msgs = vec![
            user_msg("hello"),
            assistant_with_calls(&["call_1"]),
        ];
        sanitize_history(&mut msgs);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, Role::User);
    }

    #[test]
    fn test_complete_pair_preserved() {
        let mut msgs = vec![
            user_msg("hello"),
            assistant_with_calls(&["call_1"]),
            tool_msg("result", "call_1"),
        ];
        sanitize_history(&mut msgs);
        assert_eq!(msgs.len(), 3);
    }

    #[test]
    fn test_multiple_calls_all_missing_removed() {
        let mut msgs = vec![
            user_msg("hello"),
            assistant_with_calls(&["call_1", "call_2"]),
            tool_msg("result", "call_1"),
        ];
        sanitize_history(&mut msgs);
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_chain_preserved() {
        let mut msgs = vec![
            user_msg("q1"),
            assistant_with_calls(&["c1"]),
            tool_msg("r1", "c1"),
            assistant_msg("done"),
            user_msg("q2"),
            assistant_with_calls(&["c2"]),
            tool_msg("r2", "c2"),
            assistant_msg("final"),
        ];
        sanitize_history(&mut msgs);
        assert_eq!(msgs.len(), 8);
    }

    #[test]
    fn test_tool_result_for_undeclared_call_removed() {
        // Regression: a Tool result whose tool_call_id matches no preceding
        // Assistant tool_calls used to survive sanitization (the check only
        // required *some* Assistant with calls). Providers reject results
        // with no matching declaration. c2 is undeclared -> its result must
        // be dropped, while the declared c1 pair is preserved.
        let mut msgs = vec![
            user_msg("hello"),
            assistant_with_calls(&["c1"]),
            tool_msg("r1", "c1"),
            tool_msg("r2", "c2"),
        ];
        sanitize_history(&mut msgs);
        assert_eq!(msgs.len(), 3);
        assert!(!msgs.iter().any(|m| m.role == Role::Tool && m.tool_call_id.as_deref() == Some("c2")));
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
