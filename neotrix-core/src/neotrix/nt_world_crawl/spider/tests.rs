use super::*;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

fn test_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("neotrix_spider_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn test_spider() -> CheckpointSpider {
    let dir = test_dir();
    CheckpointSpider::new(
        "test_spider",
        vec!["https://example.com".to_string()],
        dir,
    )
}

fn make_response(url: &str, status: u16, body: &str, depth: u8) -> CrawlResponse {
    CrawlResponse {
        url: url.to_string(),
        status,
        body: body.to_string(),
        headers: std::collections::HashMap::new(),
        fetched_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
        depth,
        links: Vec::new(),
        metadata: std::collections::HashMap::new(),
    }
}

#[test]
fn test_create_spider_with_start_urls() {
    let spider = test_spider();
    assert_eq!(spider.name, "test_spider");
    assert_eq!(spider.start_urls.len(), 1);
    assert!(spider.pending.len() >= 1);
    assert_eq!(spider.stats.total_discovered, 1);
}

#[test]
fn test_add_url_and_verify_queue() {
    let mut spider = test_spider();
    spider.add_url("https://test.com/page", 1, 5);
    assert_eq!(spider.pending.len(), 2);
    assert_eq!(spider.stats.total_discovered, 2);
}

#[test]
fn test_next_request_priority_order() {
    let mut spider = test_spider();
    spider.pending.clear();

    spider.add_url("https://example.com/low", 0, 10);
    spider.add_url("https://example.com/high", 0, 1);
    spider.add_url("https://example.com/medium", 0, 5);

    let first = spider.next_request().expect("expected a next request (highest priority)");
    assert_eq!(first.url, "https://example.com/high");
    assert_eq!(first.priority, 1);

    let second = spider.next_request().expect("expected a next request (medium priority)");
    assert_eq!(second.url, "https://example.com/medium");
    assert_eq!(second.priority, 5);

    let third = spider.next_request().expect("expected a next request (lowest priority)");
    assert_eq!(third.url, "https://example.com/low");
    assert_eq!(third.priority, 10);
}

#[test]
fn test_record_completed_updates_stats() {
    let mut spider = test_spider();
    let response = make_response("https://example.com/page1", 200, "<html>ok</html>", 0);
    spider.record_completed(&response);

    assert_eq!(spider.completed.len(), 1);
    assert_eq!(spider.completed[0], "https://example.com/page1");
    assert_eq!(spider.stats.total_completed, 1);
    assert_eq!(spider.stats.total_bytes, "<html>ok</html>".len());
}

#[test]
fn test_record_failed_updates_failed_list() {
    let mut spider = test_spider();
    spider.record_failed("https://example.com/bad", "Connection timeout");

    assert_eq!(spider.failed.len(), 1);
    assert_eq!(spider.failed[0].0, "https://example.com/bad");
    assert_eq!(spider.failed[0].1, "Connection timeout");
    assert_eq!(spider.stats.total_failed, 1);
}

#[test]
fn test_progress_calculation() {
    let mut spider = test_spider();
    spider.pending.clear();
    spider.stats.total_discovered = 0;
    spider.stats.total_completed = 0;

    assert!((spider.progress() - 0.0).abs() < 0.001);

    spider.stats.total_discovered = 10;
    spider.stats.total_completed = 5;
    assert!((spider.progress() - 0.5).abs() < 0.001);

    spider.stats.total_completed = 10;
    assert!((spider.progress() - 1.0).abs() < 0.001);
}

#[test]
fn test_should_checkpoint_after_interval() {
    let mut spider = test_spider();
    spider.checkpoint_interval = Duration::ZERO;
    spider.last_checkpoint = Instant::now();
    std::thread::sleep(Duration::from_millis(1));
    assert!(spider.should_checkpoint());
}

#[test]
fn test_should_checkpoint_before_interval() {
    let mut spider = test_spider();
    spider.checkpoint_interval = Duration::from_secs(3600);
    spider.last_checkpoint = Instant::now();
    assert!(!spider.should_checkpoint());
}

#[test]
fn test_save_checkpoint_creates_file() {
    let dir = std::env::temp_dir().join("neotrix_spider_save_test");
    let _ = std::fs::create_dir_all(&dir);
    let mut spider = CheckpointSpider::new("save_test", Vec::new(), dir.clone());
    spider.add_url("https://example.com/a", 0, 1);
    spider.add_url("https://example.com/b", 1, 2);

    let response = make_response("https://example.com/a", 200, "body_a", 0);
    spider.record_completed(&response);

    assert!(spider.save_checkpoint().is_ok());

    let expected_path = dir.join("save_test_checkpoint.json");
    assert!(expected_path.exists());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_load_checkpoint_restores_state() {
    let dir = std::env::temp_dir().join("neotrix_spider_load_state_test");
    let _ = std::fs::create_dir_all(&dir);
    let mut spider = CheckpointSpider::new("load_state_test", Vec::new(), dir.clone());
    spider.add_url("https://example.com/pending1", 0, 1);
    spider.add_url("https://example.com/pending2", 1, 2);

    let resp = make_response("https://example.com/done", 200, "done", 0);
    spider.record_completed(&resp);
    spider.record_failed("https://example.com/fail", "404");

    spider.save_checkpoint().expect("checkpoint should save successfully");

    let path = dir.join("load_state_test_checkpoint.json");
    let loaded = CheckpointSpider::load_checkpoint(&path).expect("should load checkpoint from file");

    assert_eq!(loaded.completed.len(), 1);
    assert_eq!(loaded.completed[0], "https://example.com/done");
    assert_eq!(loaded.failed.len(), 1);
    assert_eq!(loaded.failed[0].0, "https://example.com/fail");
    assert_eq!(loaded.stats.total_discovered, 2);
    assert_eq!(loaded.stats.total_completed, 1);
    assert_eq!(loaded.stats.total_failed, 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_load_checkpoint_restores_pending_queue() {
    let dir = std::env::temp_dir().join("neotrix_spider_pending_test");
    let _ = std::fs::create_dir_all(&dir);
    let mut spider = CheckpointSpider::new("pending_test", Vec::new(), dir.clone());
    spider.add_url("https://example.com/first", 0, 10);
    spider.add_url("https://example.com/second", 0, 5);

    spider.save_checkpoint().expect("checkpoint should save successfully");

    let path = dir.join("pending_test_checkpoint.json");
    let mut loaded = CheckpointSpider::load_checkpoint(&path).expect("should load checkpoint from file");

    assert_eq!(loaded.pending.len(), 2);

    let first = loaded.next_request().expect("expected a next request from loaded checkpoint");
    assert_eq!(first.url, "https://example.com/second");
    assert_eq!(first.priority, 5);

    let second = loaded.next_request().expect("expected second next request from loaded checkpoint");
    assert_eq!(second.url, "https://example.com/first");
    assert_eq!(second.priority, 10);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_load_checkpoint_restores_completed_urls() {
    let dir = std::env::temp_dir().join("neotrix_spider_comp_test");
    let _ = std::fs::create_dir_all(&dir);
    let mut spider = CheckpointSpider::new("comp_test", Vec::new(), dir.clone());

    for i in 0..5 {
        let url = format!("https://example.com/page{}", i);
        let resp = make_response(&url, 200, &format!("body{}", i), 0);
        spider.record_completed(&resp);
    }

    spider.save_checkpoint().expect("checkpoint should save successfully");

    let path = dir.join("comp_test_checkpoint.json");
    let loaded = CheckpointSpider::load_checkpoint(&path).expect("should load checkpoint from file");

    assert_eq!(loaded.completed.len(), 5);
    assert_eq!(loaded.stats.total_completed, 5);
    for i in 0..5 {
        let expected = format!("https://example.com/page{}", i);
        assert!(loaded.completed.contains(&expected));
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_multiple_urls_different_depths() {
    let mut spider = test_spider();
    spider.pending.clear();
    spider.stats.total_discovered = 0;

    spider.add_url("https://example.com/d0", 0, 5);
    spider.add_url("https://example.com/d1", 1, 5);
    spider.add_url("https://example.com/d2", 2, 5);

    assert_eq!(spider.pending.len(), 3);
    assert_eq!(spider.stats.total_discovered, 3);
}

#[test]
fn test_max_depth_limits() {
    let mut spider = test_spider();
    spider.pending.clear();
    spider.max_depth = 2;

    spider.add_url("https://example.com/ok", 2, 5);
    assert_eq!(spider.pending.len(), 1);

    spider.add_url("https://example.com/too_deep", 3, 5);
    assert_eq!(spider.pending.len(), 1);

    spider.add_url("https://example.com/also_ok", 1, 5);
    assert_eq!(spider.pending.len(), 2);
}

#[test]
fn test_report_format() {
    let spider = test_spider();
    let report = spider.report();
    assert!(report.starts_with("Spider["));
    assert!(report.contains("test_spider"));
    assert!(report.contains("discovered="));
    assert!(report.contains("completed="));
    assert!(report.contains("failed="));
    assert!(report.contains("pending="));
    assert!(report.contains("progress="));
}

#[test]
fn test_empty_queue_returns_none() {
    let mut spider = test_spider();
    spider.pending.clear();
    assert!(spider.next_request().is_none());
}

#[test]
fn test_progress_mixed_state() {
    let mut spider = test_spider();
    spider.pending.clear();
    spider.stats.total_discovered = 10;
    spider.stats.total_completed = 3;
    spider.stats.total_failed = 2;

    assert!((spider.progress() - 0.3).abs() < 0.001);

    for i in 0..5 {
        let url = format!("https://example.com/more{}", i);
        let resp = make_response(&url, 200, &format!("b{}", i), 0);
        spider.record_completed(&resp);
    }

    assert!((spider.progress() - 0.8).abs() < 0.001);
    assert_eq!(spider.stats.total_completed, 8);
}

#[test]
fn test_checkpoint_file_path() {
    let dir = PathBuf::from("/tmp/neotrix");
    let spider = CheckpointSpider::new("my_nt_world_crawl", Vec::new(), dir);
    let path = spider.checkpoint_path();
    assert_eq!(path, PathBuf::from("/tmp/neotrix/my_nt_world_crawl_checkpoint.json"));
}

#[test]
fn test_load_checkpoint_empty() {
    let dir = std::env::temp_dir().join("neotrix_spider_empty_test");
    let _ = std::fs::create_dir_all(&dir);
    let spider = CheckpointSpider::new("empty_test", Vec::new(), dir.clone());
    spider.save_checkpoint().expect("checkpoint should save successfully");

    let path = dir.join("empty_test_checkpoint.json");
    let loaded = CheckpointSpider::load_checkpoint(&path).expect("should load checkpoint from file");

    assert_eq!(loaded.pending.len(), 0);
    assert_eq!(loaded.completed.len(), 0);
    assert_eq!(loaded.failed.len(), 0);
    assert_eq!(loaded.stats.total_discovered, 0);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_record_completed_multiple() {
    let mut spider = test_spider();
    spider.pending.clear();

    for i in 0..5 {
        let url = format!("https://example.com/page{}", i);
        let resp = make_response(&url, 200, &format!("<html>{}</html>", i), 0);
        spider.record_completed(&resp);
    }

    assert_eq!(spider.completed.len(), 5);
    assert_eq!(spider.stats.total_completed, 5);
    assert_eq!(spider.stats.total_bytes, 5 * "<html>0</html>".len());
}

#[test]
fn test_starts_with_default_values() {
    let dir = test_dir();
    let spider = CheckpointSpider::new("defaults", Vec::new(), dir);
    assert_eq!(spider.max_depth, 3);
    assert_eq!(spider.max_concurrent, 1);
    assert_eq!(spider.checkpoint_interval, Duration::from_secs(60));
}

#[test]
fn test_crawl_with_checkpoint_completes() {
    let dir = std::env::temp_dir().join("neotrix_spider_crawl_test");
    let _ = std::fs::create_dir_all(&dir);
    let mut spider = CheckpointSpider::new("crawl_test", vec![], dir.clone());
    spider.max_depth = 0;
    spider.add_url("https://example.com/start", 0, 5);

    let mut processed = 0usize;
    let report = spider.crawl_with_checkpoint(|resp| {
        assert_eq!(resp.url, "https://example.com/start");
        processed += 1;
        Ok(())
    }).expect("crawl with checkpoint should succeed");

    assert_eq!(processed, 1);
    assert_eq!(report.total_completed, 1);
    assert_eq!(report.total_failed, 0);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_crawl_with_checkpoint_saves_on_interrupt() {
    let dir = std::env::temp_dir().join("neotrix_spider_interrupt_test");
    let _ = std::fs::create_dir_all(&dir);
    let mut spider = CheckpointSpider::new("interrupt_test", vec![], dir.clone());
    spider.checkpoint_interval = Duration::ZERO;
    spider.add_url("https://example.com/a", 0, 5);

    let report = spider.crawl_with_checkpoint(|resp| {
        if resp.url == "https://example.com/a" {
            return Ok(());
        }
        Err("simulated error".to_string())
    }).expect("crawl with checkpoint should succeed on interrupt");

    assert!(report.total_completed >= 1);

    let cp_path = dir.join("interrupt_test_checkpoint.json");
    assert!(cp_path.exists());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_resume_from_checkpoint() {
    let dir = std::env::temp_dir().join("neotrix_spider_resume_test");
    let _ = std::fs::create_dir_all(&dir);
    let mut spider = CheckpointSpider::new("resume_test", vec![], dir.clone());
    spider.add_url("https://example.com/p1", 0, 5);
    spider.add_url("https://example.com/p2", 0, 3);

    let resp1 = make_response("https://example.com/p1", 200, "ok", 0);
    spider.record_completed(&resp1);
    spider.save_checkpoint().expect("checkpoint should save successfully");

    let cp_path = dir.join("resume_test_checkpoint.json");
    let mut resumed = CheckpointSpider::resume_from_checkpoint(&cp_path).expect("should resume from checkpoint");

    assert_eq!(resumed.completed.len(), 1);
    assert_eq!(resumed.completed[0], "https://example.com/p1");
    assert_eq!(resumed.stats.total_completed, 1);

    let next = resumed.next_request().expect("expected a next request after resume");
    assert_eq!(next.url, "https://example.com/p2");
    assert_eq!(next.priority, 3);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_crawl_checkpoint_alias() {
    let dir = std::env::temp_dir().join("neotrix_spider_alias_test");
    let _ = std::fs::create_dir_all(&dir);
    let mut spider = CheckpointSpider::new("alias_test", vec![], dir.clone());
    spider.add_url("https://example.com/x", 0, 5);

    let cp: CrawlCheckpoint = SpiderCheckpoint {
        pending_requests: spider.pending.iter().cloned().collect(),
        completed_urls: spider.completed.clone(),
        failed_urls: spider.failed.clone(),
        stats: CheckpointStats {
            total_discovered: spider.stats.total_discovered,
            total_completed: spider.stats.total_completed,
            total_failed: spider.stats.total_failed,
            total_bytes: spider.stats.total_bytes,
            elapsed_secs: 0,
            created_at: 0,
        },
    };

    assert_eq!(cp.pending_requests.len(), 1);
    assert_eq!(cp.stats.total_discovered, 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_spider_report_display() {
    let dir = test_dir();
    let mut spider = CheckpointSpider::new("report_test", vec![], dir.clone());
    spider.max_depth = 0;
    spider.add_url("https://example.com/r1", 0, 5);

    let report = spider.crawl_with_checkpoint(|_| Ok(())).expect("crawl with checkpoint should succeed");
    assert_eq!(report.name, "report_test");
    assert_eq!(report.total_completed, 1);
    assert_eq!(report.total_failed, 0);

    let json = serde_json::to_string(&report).expect("report should serialize to json");
    assert!(json.contains("report_test"));
}
