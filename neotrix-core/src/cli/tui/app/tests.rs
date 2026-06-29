use super::types::{extract_thinking, extract_tool_calls};
use super::ChatMessage;

#[test]
fn test_extract_thinking_no_blocks() {
    let (blocks, clean) = extract_thinking("Hello world");
    assert!(blocks.is_empty());
    assert_eq!(clean, "Hello world");
}

#[test]
fn test_extract_thinking_with_blocks() {
    let (blocks, clean) = extract_thinking("Before\n[think]\nthinking content\n[/think]\nAfter");
    assert_eq!(blocks, vec!["thinking content"]);
    assert_eq!(clean, "Before\nAfter");
}

#[test]
fn test_extract_thinking_with_angle_tags() {
    let (blocks, clean) = extract_thinking("<think>\ndeep thought\n</think>\nresult");
    assert_eq!(blocks, vec!["deep thought"]);
    assert_eq!(clean, "result");
}

#[test]
fn test_extract_thinking_multiple_blocks() {
    let (blocks, clean) =
        extract_thinking("[think]\nfirst\n[/think]\nok\n[think]\nsecond\n[/think]\nend");
    assert_eq!(blocks, vec!["first", "second"]);
    assert_eq!(clean, "ok\nend");
}

#[test]
fn test_extract_thinking_empty_content() {
    let (blocks, clean) = extract_thinking("");
    assert!(blocks.is_empty());
    assert_eq!(clean, "");
}

#[test]
fn test_extract_thinking_only_blocks() {
    let (blocks, clean) = extract_thinking("[think]\njust thinking\n[/think]");
    assert_eq!(blocks, vec!["just thinking"]);
    assert_eq!(clean, "");
}

#[test]
fn test_extract_thinking_unclosed_tag_ignored() {
    let (blocks, clean) = extract_thinking("hi\n[think]\nno close\nthere");
    assert_eq!(blocks, vec!["no close", "there"]);
    assert_eq!(clean, "hi");
}

#[test]
fn test_extract_tool_calls_emoji_format() {
    let calls = extract_tool_calls("🛠️ search(query=rust)");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "search");
    assert_eq!(calls[0].args, "query=rust");
}

#[test]
fn test_extract_tool_calls_bracket_format() {
    let calls = extract_tool_calls("[Tool: read_file(/tmp/x)]");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "read_file");
    assert_eq!(calls[0].args, "/tmp/x");
}

#[test]
fn test_extract_tool_calls_no_args() {
    let calls = extract_tool_calls("🛠️ help");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "help");
    assert!(calls[0].args.is_empty());
}

#[test]
fn test_extract_tool_calls_multiple() {
    let calls = extract_tool_calls("🛠️ search(q=1)\nmessage\n[Tool: compute(x=2)]");
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].name, "search");
    assert_eq!(calls[1].name, "compute");
}

#[test]
fn test_extract_tool_calls_no_match() {
    let calls = extract_tool_calls("Just a normal message");
    assert!(calls.is_empty());
}

#[test]
fn test_chat_message_creation() {
    let msg = ChatMessage::new("user", "Hello world".to_string());
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "Hello world");
    assert!(msg.thinking_blocks.is_empty());
    assert!(msg.tool_calls.is_empty());
}

#[test]
fn test_chat_message_with_thinking() {
    let msg = ChatMessage::new(
        "assistant",
        "Before\n[think]\nthought\n[/think]\nAfter".to_string(),
    );
    assert_eq!(msg.content, "Before\nAfter");
    assert_eq!(msg.thinking_blocks, vec!["thought"]);
}

#[test]
fn test_chat_message_with_tool_call() {
    let msg = ChatMessage::new("assistant", "🛠️ read(/path)".to_string());
    assert_eq!(msg.tool_calls.len(), 1);
    assert_eq!(msg.tool_calls[0].name, "read");
}

#[test]
fn test_chat_message_defaults() {
    let msg = ChatMessage::new("system", "status ok".to_string());
    assert_eq!(msg.role, "system");
    assert!(msg.thinking_blocks.is_empty());
    assert!(msg.tool_calls.is_empty());
}
