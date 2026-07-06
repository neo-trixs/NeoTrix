use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlRequest {
    pub url: String,
    pub depth: u8,
    pub priority: u8,
    pub headers: HashMap<String, String>,
    pub max_retries: u8,
    pub retry_count: u8,
    pub session_type: SessionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionType {
    Fetcher,
    Stealthy,
    Dynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResponse {
    pub url: String,
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub fetched_at: i64,
    pub depth: u8,
    pub links: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStats {
    pub total_discovered: usize,
    pub total_completed: usize,
    pub total_failed: usize,
    pub total_bytes: usize,
    pub elapsed_secs: u64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderCheckpoint {
    pub pending_requests: Vec<CrawlRequest>,
    pub completed_urls: Vec<String>,
    pub failed_urls: Vec<(String, String)>,
    pub stats: CheckpointStats,
}

#[derive(Debug, Clone)]
pub struct SpiderStats {
    pub total_discovered: usize,
    pub total_completed: usize,
    pub total_failed: usize,
    pub total_bytes: usize,
    pub start_time: Instant,
}

impl SpiderStats {
    pub fn new() -> Self {
        SpiderStats {
            total_discovered: 0,
            total_completed: 0,
            total_failed: 0,
            total_bytes: 0,
            start_time: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for SpiderStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderReport {
    pub name: String,
    pub total_discovered: usize,
    pub total_completed: usize,
    pub total_failed: usize,
    pub total_bytes: usize,
    pub elapsed_secs: u64,
}

pub type CrawlCheckpoint = SpiderCheckpoint;
