use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::types::{
    CheckpointStats, CrawlRequest, CrawlResponse, SessionType, SpiderCheckpoint, SpiderReport,
    SpiderStats,
};

const MAX_PENDING: usize = 10_000;
const MAX_COMPLETED: usize = 100_000;
const MAX_FAILED: usize = 10_000;

pub struct CheckpointSpider {
    pub name: String,
    pub start_urls: Vec<String>,
    pub pending: std::collections::VecDeque<CrawlRequest>,
    pub completed: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub stats: SpiderStats,
    pub max_depth: u8,
    pub max_concurrent: usize,
    pub checkpoint_dir: PathBuf,
    pub checkpoint_interval: Duration,
    pub last_checkpoint: Instant,
}

impl CheckpointSpider {
    pub fn new(name: &str, start_urls: Vec<String>, checkpoint_dir: PathBuf) -> Self {
        let mut spider = CheckpointSpider {
            name: name.to_string(),
            start_urls: start_urls.clone(),
            pending: std::collections::VecDeque::new(),
            completed: Vec::new(),
            failed: Vec::new(),
            stats: SpiderStats::new(),
            max_depth: 3,
            max_concurrent: 1,
            checkpoint_dir,
            checkpoint_interval: Duration::from_secs(60),
            last_checkpoint: Instant::now(),
        };

        for url in start_urls {
            spider.add_url(&url, 0, 5);
        }

        spider
    }

    pub fn add_url(&mut self, url: &str, depth: u8, priority: u8) {
        if depth > self.max_depth {
            return;
        }
        if self.pending.len() >= MAX_PENDING {
            return;
        }
        let req = CrawlRequest {
            url: url.to_string(),
            depth,
            priority,
            headers: HashMap::new(),
            max_retries: 3,
            retry_count: 0,
            session_type: SessionType::Fetcher,
        };
        self.pending.push_back(req);
        self.stats.total_discovered += 1;
    }

    pub fn next_request(&mut self) -> Option<CrawlRequest> {
        if self.pending.is_empty() {
            return None;
        }

        let mut best_idx = 0;
        let mut best_priority = self.pending[0].priority;
        for (i, req) in self.pending.iter().enumerate() {
            if req.priority < best_priority {
                best_priority = req.priority;
                best_idx = i;
            }
        }

        self.pending.remove(best_idx)
    }

    pub fn record_completed(&mut self, response: &CrawlResponse) {
        self.completed.push(response.url.clone());
        self.enforce_completed_cap();
        self.stats.total_completed += 1;
        self.stats.total_bytes += response.body.len();
    }

    pub fn record_failed(&mut self, url: &str, error: &str) {
        self.failed.push((url.to_string(), error.to_string()));
        self.enforce_failed_cap();
        self.stats.total_failed += 1;
    }

    fn enforce_completed_cap(&mut self) {
        if self.completed.len() > MAX_COMPLETED {
            self.completed.drain(0..(MAX_COMPLETED * 20 / 100).max(1));
        }
    }

    fn enforce_failed_cap(&mut self) {
        if self.failed.len() > MAX_FAILED {
            self.failed.drain(0..(MAX_FAILED * 20 / 100).max(1));
        }
    }

    pub fn save_checkpoint(&self) -> Result<(), String> {
        let checkpoint = SpiderCheckpoint {
            pending_requests: self.pending.iter().cloned().collect(),
            completed_urls: self.completed.clone(),
            failed_urls: self.failed.clone(),
            stats: CheckpointStats {
                total_discovered: self.stats.total_discovered,
                total_completed: self.stats.total_completed,
                total_failed: self.stats.total_failed,
                total_bytes: self.stats.total_bytes,
                elapsed_secs: self.stats.elapsed().as_secs(),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            },
        };

        let json = serde_json::to_string_pretty(&checkpoint).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&self.checkpoint_dir).map_err(|e| e.to_string())?;
        let path = self.checkpoint_path();
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_checkpoint(path: &PathBuf) -> Result<Self, String> {
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let cp: SpiderCheckpoint = serde_json::from_str(&json).map_err(|e| e.to_string())?;

        let parent_dir = path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf();

        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "spider".to_string());

        let mut spider = CheckpointSpider::new(&file_stem, Vec::new(), parent_dir);

        for req in cp.pending_requests {
            if spider.pending.len() >= MAX_PENDING {
                break;
            }
            spider.pending.push_back(req);
        }
        spider.completed = cp.completed_urls;
        spider.failed = cp.failed_urls;
        spider.enforce_completed_cap();
        spider.enforce_failed_cap();
        spider.stats.total_discovered = cp.stats.total_discovered;
        spider.stats.total_completed = cp.stats.total_completed;
        spider.stats.total_failed = cp.stats.total_failed;
        spider.stats.total_bytes = cp.stats.total_bytes;

        Ok(spider)
    }

    pub fn should_checkpoint(&self) -> bool {
        self.last_checkpoint.elapsed() >= self.checkpoint_interval
    }

    pub fn progress(&self) -> f64 {
        let total = self.stats.total_discovered;
        if total == 0 {
            return 0.0;
        }
        self.stats.total_completed as f64 / total as f64
    }

    pub fn report(&self) -> String {
        format!(
            "Spider[{}] discovered={} completed={} failed={} pending={} bytes={} elapsed={:.1}s progress={:.1}%",
            self.name,
            self.stats.total_discovered,
            self.stats.total_completed,
            self.stats.total_failed,
            self.pending.len(),
            self.stats.total_bytes,
            self.stats.elapsed().as_secs_f64(),
            self.progress() * 100.0,
        )
    }

    pub(crate) fn checkpoint_path(&self) -> PathBuf {
        self.checkpoint_dir
            .join(format!("{}_checkpoint.json", self.name))
    }

    pub fn crawl_with_checkpoint<F>(&mut self, mut process_page: F) -> Result<SpiderReport, String>
    where
        F: FnMut(&CrawlResponse) -> Result<(), String>,
    {
        while let Some(req) = self.next_request() {
            if self.should_checkpoint() {
                self.save_checkpoint()?;
                self.last_checkpoint = Instant::now();
            }

            let fake_resp = CrawlResponse {
                url: req.url.clone(),
                status: 200,
                body: String::new(),
                headers: HashMap::new(),
                fetched_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
                depth: req.depth,
                links: Vec::new(),
                metadata: HashMap::new(),
            };

            match process_page(&fake_resp) {
                Ok(()) => {
                    self.record_completed(&fake_resp);
                }
                Err(e) => {
                    self.record_failed(&req.url, &e);
                }
            }

            let mut discovered_links: Vec<String> = Vec::new();
            if req.depth < self.max_depth {
                discovered_links.push(format!("{}/page", req.url));
            }
            for link in discovered_links {
                self.add_url(&link, req.depth + 1, req.priority + 1);
            }
        }

        self.save_checkpoint()?;
        Ok(SpiderReport {
            name: self.name.clone(),
            total_discovered: self.stats.total_discovered,
            total_completed: self.stats.total_completed,
            total_failed: self.stats.total_failed,
            total_bytes: self.stats.total_bytes,
            elapsed_secs: self.stats.elapsed().as_secs(),
        })
    }

    pub fn resume_from_checkpoint(path: &PathBuf) -> Result<Self, String> {
        Self::load_checkpoint(path)
    }
}
