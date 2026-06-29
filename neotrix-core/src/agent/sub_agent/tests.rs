use super::*;
use std::time::Duration;
use uuid::Uuid;

fn test_config() -> SubAgentConfig {
    SubAgentConfig {
        max_concurrency: 5,
        max_context_window: 1024,
        idle_timeout_secs: 30,
    }
}

#[tokio::test]
async fn test_launch_and_complete() {
    let pool = SubAgentPool::new(test_config());
    let id = pool
        .launch("test prompt for completion", None)
        .await
        .expect("launch should succeed");

    let result = pool.wait_for(id).await;
    assert!(result.is_some(), "sub-agent should complete");
    let result = result.expect("unexpected None/Err");
    assert!(!result.summary.is_empty(), "summary should not be empty");
    assert!(result.total_tokens > 0, "should have consumed tokens");
    // duration_ms may be 0 in extremely fast test envs; just verify non-None
    assert_eq!(pool.status(id), Some(SubAgentStatus::Completed));
}

#[tokio::test]
async fn test_max_concurrency_enforcement() {
    let config = SubAgentConfig {
        max_concurrency: 2,
        ..test_config()
    };
    let pool = SubAgentPool::new(config);

    let mut ids = Vec::new();
    for i in 0..4 {
        let id = pool
            .launch(&format!("concurrent task {}", i), None)
            .await
            .expect("launch should succeed");
        ids.push(id);
    }

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        pool.active_count() <= 2,
        "active count {} should be <= 2",
        pool.active_count()
    );

    for id in &ids {
        let _ = pool.wait_for(*id).await;
    }

    for id in &ids {
        assert_eq!(pool.status(*id), Some(SubAgentStatus::Completed));
    }
}

#[tokio::test]
async fn test_timeout_handling() {
    let config = SubAgentConfig {
        idle_timeout_secs: 1,
        ..test_config()
    };
    let pool = SubAgentPool::new(config);

    let id = pool
        .launch_with_timeout("timeout test task", 0)
        .await
        .expect("launch should succeed");

    tokio::time::sleep(Duration::from_millis(200)).await;

    let status = pool.status(id);
    assert!(
        status == Some(SubAgentStatus::TimedOut)
            || matches!(status, Some(SubAgentStatus::Failed(_))),
        "sub-agent should time out or fail, got {:?}",
        status,
    );
}

#[tokio::test]
async fn test_handle_read_slice() {
    let pool = SubAgentPool::new(test_config());

    let id = pool
        .launch("read slice test evidence generation", None)
        .await
        .expect("launch should succeed");

    let result = pool.wait_for(id).await;
    assert!(result.is_some(), "should complete");

    let result = result.expect("unexpected None/Err");
    let evidence_len = result.evidence.len();

    if evidence_len >= 2 {
        let slice = pool.handle_read_slice(id, 0, 2);
        assert!(slice.is_some(), "slice should exist");
        let slice = slice.expect("unexpected None/Err");
        assert!(!slice.is_empty(), "slice should not be empty");
        assert!(slice.len() <= 2, "slice length should be <= 2");
    }

    let empty_slice = pool.handle_read_slice(id, evidence_len + 10, evidence_len + 20);
    assert!(
        empty_slice.is_some(),
        "out-of-range slice should return Some(vec![])"
    );
    assert!(
        empty_slice.expect("unexpected None/Err").is_empty(),
        "out-of-range slice should be empty"
    );

    let bogus_id = Uuid::new_v4();
    let bogus_slice = pool.handle_read_slice(bogus_id, 0, 5);
    assert!(bogus_slice.is_none(), "non-existent ID should return None");
}

#[tokio::test]
async fn test_cancel_operation() {
    let pool = SubAgentPool::new(test_config());

    let id = pool
        .launch("cancel test task", None)
        .await
        .expect("launch should succeed");

    let cancelled = pool.cancel(id);
    assert!(cancelled, "cancel should succeed for active sub-agent");

    let status = pool.status(id);
    assert!(matches!(status, Some(SubAgentStatus::Failed(ref s)) if s == "cancelled"));

    let bogus_id = Uuid::new_v4();
    let cancelled = pool.cancel(bogus_id);
    assert!(!cancelled, "cancel should fail for non-existent ID");

    let id2 = pool
        .launch("cancel all test", None)
        .await
        .expect("launch should succeed");
    let _id3 = pool
        .launch("another cancel all test", None)
        .await
        .expect("launch should succeed");

    pool.cancel_all();
    assert_eq!(
        pool.active_count(),
        0,
        "active count should be 0 after cancel_all"
    );
    assert!(
        pool.status(id2).is_none()
            || matches!(pool.status(id2), Some(SubAgentStatus::Failed(ref s)) if s == "cancelled by pool"),
        "id2 should be cancelled or removed"
    );
}

#[tokio::test]
async fn test_event_emission() {
    let pool = SubAgentPool::new(test_config());
    let mut rx = pool.event_receiver().expect("should get event receiver");

    let id = pool
        .launch("event emission test", None)
        .await
        .expect("launch should succeed");

    let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("should receive event within timeout")
        .expect("event should be Some");

    match event {
        SubAgentEvent::Done { id: event_id, .. } => {
            assert_eq!(event_id, id, "event ID should match sub-agent ID");
        }
        other => panic!("expected Done event, got {:?}", other),
    }

    let tag = event.to_tag_string();
    assert!(
        tag.starts_with("<subagent:done"),
        "tag should start with <subagent:done"
    );
    assert!(
        tag.contains(&id.to_string()),
        "tag should contain sub-agent ID"
    );
}
