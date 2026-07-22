//! Stream data into KB — six absorb functions for real-time knowledge ingestion.
//!
//! Each function is ~25 lines, idempotent, and designed to be called at the
//! data's point of origin (no batching, no caching, no waiting).
//!
//! # Functions
//!
//! | Function | Source | KB Node |
//! |----------|--------|---------|
//! | `absorb_thinking_trace` | `SiliconSelf.add_thinking_trace()` | `thinking_trace` |
//! | `absorb_self_test_failure` | `run_all()` result | `self_test_failure` |
//! | `absorb_event` | EventBus subscriber | `event_record` |
//! | `absorb_finding` | detector evaluate() | `detection_finding` |
//! | `absorb_goal_result` | goal_loop.rs | `goal_result` |
//! | `absorb_json_record` | generic structured data | any `NodeType` |

use std::collections::HashMap;

use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType;

/// Absorb a ThinkingTrace into KB for RL training and audit.
/// Idempotent by trace id.
pub fn absorb_thinking_trace(
    kb: &KnowledgeBase,
    trace_id: &str,
    task: &str,
    steps: &[HashMap<String, String>],
    total_duration_ms: u64,
    total_tokens: u64,
) -> Result<String, String> {
    let title = format!("thinking_trace_{}", trace_id);
    let summary = format!("task={} steps={} duration={}ms tokens={}", task, steps.len(), total_duration_ms, total_tokens);
    let node_id = kb.insert_or_get_node(&title, NodeType::ThinkingTrace, Some(&summary), Some(trace_id), Some("thinking_trace"))?;

    let meta = serde_json::json!({
        "task": task,
        "step_count": steps.len(),
        "total_duration_ms": total_duration_ms,
        "total_tokens": total_tokens,
        "steps": steps,
    });
    kb.update_node_metadata(&node_id, &meta)?;

    Ok(node_id)
}

/// Absorb a SelfTest failure into KB for defect traceability.
pub fn absorb_self_test_failure(
    kb: &KnowledgeBase,
    module_name: &str,
    failure_message: &str,
) -> Result<String, String> {
    let dedup_key = format!("self_test_failure/{}", module_name);
    let node_id = kb.insert_or_get_node(
        &format!("self-test: {}", module_name),
        NodeType::SelfTestFailure,
        Some(failure_message),
        Some(&dedup_key),
        Some("self_test"),
    )?;

    let meta = serde_json::json!({
        "module": module_name,
        "failure": failure_message,
        "timestamp_secs": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
    });
    kb.update_node_metadata(&node_id, &meta)?;

    Ok(node_id)
}

/// Absorb an EventBus event into KB for event traceability.
pub fn absorb_event(
    kb: &KnowledgeBase,
    event_type: &str,
    payload: &serde_json::Value,
) -> Result<String, String> {
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let dedup_key = format!("event/{}/{}", event_type, ts);
    let title = format!("event: {} @ {}", event_type, ts);
    let summary = format!("event_type={} at {}", event_type, ts);

    let node_id = kb.insert_or_get_node(&title, NodeType::EventRecord, Some(&summary), Some(&dedup_key), Some("event_bus"))?;

    let meta = serde_json::json!({
        "event_type": event_type,
        "payload": payload,
        "timestamp_secs": ts,
    });
    kb.update_node_metadata(&node_id, &meta)?;

    Ok(node_id)
}

/// Absorb a detection module's individual finding into KB.
pub fn absorb_finding(
    kb: &KnowledgeBase,
    detector: &str,
    severity: &str,
    description: &str,
    file_line: &str,
) -> Result<String, String> {
    let dedup_key = format!("finding/{}/{}", detector, file_line);
    let title = format!("[{}] {}: {}", severity, detector, description.chars().take(80).collect::<String>());

    let node_id = kb.insert_or_get_node(&title, NodeType::DetectionFinding, Some(description), Some(&dedup_key), Some(detector))?;

    let meta = serde_json::json!({
        "detector": detector,
        "severity": severity,
        "file_line": file_line,
        "timestamp_secs": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
    });
    kb.update_node_metadata(&node_id, &meta)?;

    Ok(node_id)
}

/// Absorb a goal completion/failure result into KB.
pub fn absorb_goal_result(
    kb: &KnowledgeBase,
    goal_id: &str,
    category: &str,
    status: &str,
    reward: f64,
    iterations: u64,
) -> Result<String, String> {
    let title = format!("goal: {} [{}]", goal_id, status);
    let summary = format!("category={} status={} reward={:.2} iter={}", category, status, reward, iterations);

    let node_id = kb.insert_or_get_node(&title, NodeType::GoalResult, Some(&summary), Some(goal_id), Some("goal_loop"))?;

    let meta = serde_json::json!({
        "goal_id": goal_id,
        "category": category,
        "status": status,
        "reward": reward,
        "iterations": iterations,
    });
    kb.update_node_metadata(&node_id, &meta)?;

    Ok(node_id)
}

/// Absorb a generic JSON record into KB.
pub fn absorb_json_record(
    kb: &KnowledgeBase,
    node_type: NodeType,
    title: &str,
    dedup_key: &str,
    domain: &str,
    data: &serde_json::Value,
) -> Result<String, String> {
    let summary = data.as_object()
        .and_then(|m| m.get("summary"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let summary_opt = if summary.is_empty() { None } else { Some(&summary[..]) };

    let node_id = kb.insert_or_get_node(title, node_type, summary_opt, Some(dedup_key), Some(domain))?;
    kb.update_node_metadata(&node_id, data)?;
    Ok(node_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn test_kb(name: &str) -> KnowledgeBase {
        let dir = PathBuf::from(std::env::temp_dir()).join(format!("nt_auto_abs_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test.db");
        KnowledgeBase::open(Some(db_path)).expect("open KB")
    }

    #[test]
    fn test_absorb_thinking_trace_creates_node() {
        let kb = test_kb("trace");
        let steps = vec![
            HashMap::from([("strategy".into(), "decompose".into()), ("domain".into(), "code".into())]),
        ];
        let id = absorb_thinking_trace(&kb, "trace-001", "refactor module", &steps, 1500, 4500).expect("absorb");
        assert!(!id.is_empty(), "should return a node id");

        let id2 = absorb_thinking_trace(&kb, "trace-001", "refactor module", &steps, 1500, 4500).expect("absorb again");
        assert_eq!(id, id2, "same dedup_key → same node");
    }

    #[test]
    fn test_absorb_self_test_failure_creates_node() {
        let kb = test_kb("st_fail");
        let id = absorb_self_test_failure(&kb, "SchemaWatchdog", "drift detected: field missing").expect("absorb");
        assert!(!id.is_empty());
    }

    #[test]
    fn test_absorb_event_creates_node() {
        let kb = test_kb("event");
        let payload = serde_json::json!({"task": "test", "priority": 5});
        let id = absorb_event(&kb, "TaskSubmitted", &payload).expect("absorb");
        assert!(!id.is_empty());
    }

    #[test]
    fn test_absorb_finding_creates_node() {
        let kb = test_kb("finding");
        let id = absorb_finding(&kb, "EntropyMonitor", "warning", "GWT deadlock detected", "run.rs:1250").expect("absorb");
        assert!(!id.is_empty());
    }

    #[test]
    fn test_absorb_goal_result_creates_node() {
        let kb = test_kb("goal");
        let id = absorb_goal_result(&kb, "goal-42", "code_review", "completed", 0.85, 3).expect("absorb");
        assert!(!id.is_empty());
    }

    #[test]
    fn test_absorb_json_record_creates_node() {
        let kb = test_kb("json_rec");
        let data = serde_json::json!({"summary": "test record", "value": 42});
        let id = absorb_json_record(&kb, NodeType::Source, "test record", "json-001", "test", &data).expect("absorb");
        assert!(!id.is_empty());
    }

    #[test]
    fn test_absorb_goal_dedup() {
        let kb = test_kb("goal_dedup");
        let a = absorb_goal_result(&kb, "dedup-test", "review", "completed", 1.0, 1).expect("first");
        let b = absorb_goal_result(&kb, "dedup-test", "review", "completed", 1.0, 1).expect("second");
        assert_eq!(a, b, "same goal_id → same node");
    }

    #[test]
    fn test_absorb_event_different_timestamps_give_different_nodes() {
        let kb = test_kb("event_ts");
        let payload = serde_json::json!({"msg": "hello"});
        let a = absorb_event(&kb, "AgentFeedback", &payload).expect("first");
        // Force time separation — timestamps differ by at least 1ms
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = absorb_event(&kb, "AgentFeedback", &payload).expect("second");
        assert_ne!(a, b, "different timestamps → different nodes");
    }
}
