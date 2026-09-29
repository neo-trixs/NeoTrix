//! Unified streaming pipeline — download→file/pipe→player end-to-end.
//!
//! Architecture:
//! ```text
//! ┌──────────────────────────────────────────────────────────────────┐
//! │  StreamingPipeline                                               │
//! │  URL → detect → route → download → write → player               │
//! ├──────────────────────────────────────────────────────────────────┤
//! │  Transport Backends:                                             │
//! │  • HttpStream — sequential reqwest streaming                     │
//! │  • MagnetRpc  — aria2c JSON-RPC sequential BT download           │
//! │  • FifoPipe   — zero-disk I/O via named pipe                     │
//! ├──────────────────────────────────────────────────────────────────┤
//! │  Download Engine:                                                │
//! │  • Mirror speed profiling (EMA) + HuggingFace adaptive          │
//! │  • Parallel chunk download with work-stealing                    │
//! │  • Temp-file merge (.dl_* directories) + .done markers          │
//! │  • Disk space pre-check + content-disposition detection          │
//! │  • Stall detection + n² backoff retry + SHA-256 verify          │
//! ├──────────────────────────────────────────────────────────────────┤
//! │  Player Backends:                                                │
//! │  • FileGrow — player reads growing file (mpv appending://)       │
//! │  • PipePlay — player reads from stdin/FIFO                       │
//! └──────────────────────────────────────────────────────────────────┘
//! ```



// ═══════════════════════════════════════════════════════════════════════════
// Mirror speed profiling — delegated to NT-WORLD (L2 Perception)
// ═══════════════════════════════════════════════════════════════════════════

pub use crate::l1_action::nt_action_facade::nt_world_mirror::{
    ranked_mirrors, record_mirror_speed, resolve_mirror,
};


// ═══════════════════════════════════════════════════════════════════════════
// Re-export unified ChunkState / ChunkDownloadStatus from persistence
// ═══════════════════════════════════════════════════════════════════════════

pub use super::persistence::{ChunkDownloadStatus, ChunkState};


// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

pub mod types;
pub mod dl_fs;
pub mod resilience;
pub mod parallel;
pub mod http;
pub mod alt;
pub mod pipeline;
pub mod engine;
pub use types::{AggregateProgress, DownloadProgressSnapshot, DownloadStatus, DownloadConfig, DownloadTask, PipelineConfig, PipelineProgress, PipelineStatus, PipelineHandle, TaskHandle, PipelineError};
pub use dl_fs::{check_disk_space, cleanup_stale_temps, detect_filename};
pub use resilience::{RetryPolicy, StallDetector};
pub use http::{compute_sha256, verify_sha256};
pub use alt::check_aria2c_health;
pub use engine::DownloadEngine;
pub use pipeline::StreamingPipeline;

#[cfg(test)]
mod tests {
    use super::*;
    use super::pipeline::detect_player;
use super::dl_fs::make_dl_tmp_dir;
    use super::parallel::ParallelDownloader;
    use super::pipeline::StreamingPipeline;
    use super::types::StreamingPipelineConfig;
    use std::time::Duration;
    use tokio::sync::mpsc;
use std::path::{Path, PathBuf};
use tokio::fs;

      #[tokio::test]
      async fn test_pipeline_http_streaming() {
          // 本地回环服务，**不依赖外网**。原实现打 `https://httpbin.org/bytes/4096`，
          // 而本测试每条消息只给 5s；实测该端点往返 2.2s ⇒ 余量极小，
          // 属「环境依赖型 flaky」：网络快就过、网络抖就红，且红时与被测逻辑无关。
          // 2026-09-28 改为回环端点，断言一字未动。
          use axum::routing::get;
          use axum::Router;
          let body: Vec<u8> = (0..4096u32).map(|i| (i % 251) as u8).collect();
          let app = Router::new().route(
              "/bytes/4096",
              get(move || {
                  let b = body.clone();
                  async move { b }
              }),
          );
          let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
              .await
              .expect("bind loopback");
          let addr = listener.local_addr().expect("local_addr");
          tokio::spawn(async move {
              let _ = axum::serve(listener, app).await;
          });
          let url = format!("http://{addr}/bytes/4096");

          let (tx, mut rx) = mpsc::channel(100);
          let pipeline = StreamingPipeline::new(StreamingPipelineConfig {
              url,
              output_dir: std::env::temp_dir(),
              prefer_streaming: true,
              buffer_threshold: 1024,
              ..Default::default()
          });

          let handle = pipeline.run(tx).await.unwrap();

          let mut got_complete = false;
          while let Ok(Some(p)) = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await {
              if matches!(p.status, PipelineStatus::Complete { .. }) {
                  got_complete = true;
                  break;
              }
          }

          assert!(got_complete, "回环端点应在 5s 内发 Complete");
          let output_path = handle.output_path().to_path_buf();
          handle.wait().await.ok();
          let _ = fs::remove_file(output_path).await;
      }


    #[tokio::test]
    async fn test_pipeline_file_copy() {
        let (tx, mut rx) = mpsc::channel(10);
        let test_file = std::env::temp_dir().join("nt_test_file_copy.txt");
        fs::write(&test_file, b"hello").await.unwrap();

        let pipeline = StreamingPipeline::new(StreamingPipelineConfig {
            url: format!("file://{}", test_file.to_string_lossy()),
            output_dir: std::env::temp_dir(),
            ..Default::default()
        });

        let _handle = pipeline.run(tx).await.unwrap();
        let progress = rx.recv().await.unwrap();
        assert!(matches!(progress.status, PipelineStatus::Complete { .. }));

        let _ = fs::remove_file(&test_file).await;
    }

    #[test]
    fn test_detect_player() {
        assert!(detect_player(None).is_some());
    }

    #[test]
    fn test_plan_chunks() {
        let chunks = ParallelDownloader::plan_chunks(100, 30);
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0].start, 0);
        assert_eq!(chunks[0].end, 29);
        assert_eq!(chunks[1].start, 30);
        assert_eq!(chunks[1].end, 59);
        assert_eq!(chunks[3].start, 90);
        assert_eq!(chunks[3].end, 99);
    }

    #[test]
    fn test_plan_chunks_exact() {
        let chunks = ParallelDownloader::plan_chunks(100, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start, 0);
        assert_eq!(chunks[0].end, 99);
    }

    #[test]
    fn test_plan_chunks_zero() {
        let chunks = ParallelDownloader::plan_chunks(0, 100);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_retry_policy_delay() {
        let policy = RetryPolicy::new(3);
        assert!(policy.delay(1).is_some());
        assert!(policy.delay(3).is_some());
        assert!(policy.delay(4).is_none());
    }

    #[test]
    fn test_retry_policy_is_retryable_status() {
        assert!(RetryPolicy::is_retryable_status(500));
        assert!(RetryPolicy::is_retryable_status(503));
        assert!(RetryPolicy::is_retryable_status(429));
        assert!(!RetryPolicy::is_retryable_status(404));
        assert!(!RetryPolicy::is_retryable_status(200));
    }

    #[tokio::test]
    async fn test_stall_detector() {
        let mut stall = StallDetector::new(Duration::from_millis(50));
        assert!(!stall.check(0));
        assert!(!stall.is_stalled());
        stall.record_progress();
        assert!(!stall.is_stalled());
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(stall.is_stalled());
        stall.reset();
        assert!(!stall.is_stalled());
    }

    #[tokio::test]
    async fn test_stall_detector_check() {
        let mut stall = StallDetector::new(Duration::from_millis(50));
        assert!(!stall.check(0));
        assert!(!stall.check(0));
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(stall.check(0));
        assert!(!stall.check(1024));
    }

    #[tokio::test]
    async fn test_compute_and_verify_sha256() {
        let test_file = std::env::temp_dir().join("nt_test_sha256.txt");
        fs::write(&test_file, b"hello world").await.unwrap();

        let hash = compute_sha256(&test_file).await.unwrap();
        assert_eq!(hash.len(), 64);

        assert!(verify_sha256(&test_file, &hash).await.unwrap());
        assert!(!verify_sha256(
            &test_file,
            "0000000000000000000000000000000000000000000000000000000000000000"
        )
        .await
        .unwrap());

        let _ = fs::remove_file(&test_file).await;
    }

    #[test]
    fn test_mirror_speed_recording() {
        record_mirror_speed("hf-mirror.com", 1_000_000.0);
        record_mirror_speed("huggingface.co", 500_000.0);
        let ranked = ranked_mirrors();
        assert_eq!(ranked.len(), 2);
        assert!(ranked[0].1 >= ranked[1].1);
    }

    #[test]
    fn test_download_config_defaults() {
        let cfg = DownloadConfig::default();
        assert_eq!(cfg.max_concurrent, 16);
        assert_eq!(cfg.min_disk_space, 1024 * 1024 * 1024);
        assert_eq!(cfg.retry_count, 5);
    }

    #[test]
    fn test_download_config_to_pipeline() {
        let cfg = DownloadConfig {
            max_concurrent: 8,
            timeout_secs: 120,
            retry_count: 3,
            max_chunk_bytes: 32 * 1024 * 1024,
            ..Default::default()
        };
        let pc = cfg.to_pipeline_config(
            "https://example.com/f.bin".into(),
            PathBuf::from("/tmp"),
        );
        assert_eq!(pc.concurrency, 8);
        assert_eq!(pc.max_retries, 3);
        assert_eq!(pc.chunk_size, 32 * 1024 * 1024);
    }

    #[test]
    fn test_make_dl_tmp_dir() {
        let dest = Path::new("/tmp/model.gguf");
        let tmp = make_dl_tmp_dir(dest);
        assert_eq!(tmp, PathBuf::from("/tmp/.dl_model"));
    }

    #[test]
    fn test_download_task_builder() {
        let task = DownloadTask::new("https://example.com/f.bin", "/tmp/f.bin")
            .with_priority(10);
        assert_eq!(task.priority, 10);
        assert_eq!(task.url, "https://example.com/f.bin");
    }

    // ── Douyin fusion pattern tests ──────────────────────────────────

    #[test]
    fn test_retry_policy_delay_table() {
        let table = vec![
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(5),
        ];
        let policy = RetryPolicy::with_delay_table(3, table);

        assert_eq!(policy.delay(1), Some(Duration::from_secs(1)));
        assert_eq!(policy.delay(2), Some(Duration::from_secs(2)));
        assert_eq!(policy.delay(3), Some(Duration::from_secs(5)));
        // Beyond max_retries — caps at last entry
        assert_eq!(policy.delay(4), None);
        assert_eq!(policy.delay(10), None);
    }

    #[test]
    fn test_retry_policy_delay_table_capped() {
        let table = vec![Duration::from_secs(1), Duration::from_secs(2)];
        let policy = RetryPolicy::with_delay_table(5, table);

        // Attempts beyond table length use last entry
        assert_eq!(policy.delay(3), Some(Duration::from_secs(2)));
        assert_eq!(policy.delay(5), Some(Duration::from_secs(2)));
        assert_eq!(policy.delay(6), None);
    }

    #[test]
    fn test_pipeline_config_atomic_write_default() {
        let config = StreamingPipelineConfig::default();
        assert!(!config.atomic_write);
        assert!(config.mirror_fallback_enabled);
        assert_eq!(config.min_disk_space, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_pipeline_config_atomic_write_enabled() {
        let config = StreamingPipelineConfig {
            atomic_write: true,
            min_disk_space: 512 * 1024 * 1024,
            ..Default::default()
        };
        assert!(config.atomic_write);
        assert_eq!(config.min_disk_space, 512 * 1024 * 1024);
    }

    #[test]
    fn test_download_config_to_pipeline_min_disk_space() {
        let cfg = DownloadConfig {
            min_disk_space: 256 * 1024 * 1024,
            ..Default::default()
        };
        let pc = cfg.to_pipeline_config("https://example.com/f.bin".into(), PathBuf::from("/tmp"));
        assert_eq!(pc.min_disk_space, 256 * 1024 * 1024);
    }
}
