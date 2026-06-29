use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::io::IoError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub id: String,
    pub payload: String,
    pub item_type: String,
    pub created_at: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub vsa_fingerprint: [u64; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueStrategy {
    FIFO,
    LIFO,
    PriorityFirst,
    RetryFirst,
}

#[derive(Debug, Clone)]
pub struct QueueStats {
    pub total: usize,
    pub pending: usize,
    pub failed: usize,
    pub avg_retries: f64,
}

pub struct PersistQueue {
    file_path: PathBuf,
    items: Vec<QueueItem>,
    pending_acks: HashMap<String, QueueItem>,
    failed_ids: HashSet<String>,
}

impl PersistQueue {
    pub fn new(path: &str) -> Result<Self, IoError> {
        let file_path = PathBuf::from(path);
        let items = if file_path.exists() {
            Self::load_file(&file_path)?
        } else {
            Vec::new()
        };
        Ok(Self {
            file_path,
            items,
            pending_acks: HashMap::new(),
            failed_ids: HashSet::new(),
        })
    }

    fn load_file(path: &PathBuf) -> Result<Vec<QueueItem>, IoError> {
        let file = File::open(path).map_err(|e| IoError::Storage(format!("cannot open queue file: {e}")))?;
        let reader = BufReader::new(file);
        let mut items = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|e| IoError::Storage(format!("cannot read queue line: {e}")))?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let item: QueueItem = serde_json::from_str(trimmed)
                .map_err(|e| IoError::Storage(format!("cannot parse queue item: {e}")))?;
            items.push(item);
        }
        Ok(items)
    }

    fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }

    fn compute_vsa_fingerprint(payload: &str, item_type: &str) -> [u64; 4] {
        use std::hash::{Hash, Hasher};
        let combined = format!("{}:{}", payload, item_type);
        let bytes = combined.as_bytes();
        let mut fp = [0u64; 4];
        for (i, slot) in fp.iter_mut().enumerate() {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            bytes.hash(&mut hasher);
            i.hash(&mut hasher);
            *slot = hasher.finish();
        }
        fp
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn enqueue(&mut self, payload: &str, item_type: &str) -> Result<String, IoError> {
        let id = Self::generate_id();
        let item = QueueItem {
            id: id.clone(),
            payload: payload.to_string(),
            item_type: item_type.to_string(),
            created_at: Self::now(),
            retry_count: 0,
            max_retries: 3,
            vsa_fingerprint: Self::compute_vsa_fingerprint(payload, item_type),
        };
        self.items.push(item);
        self.persist()?;
        Ok(id)
    }

    pub fn dequeue(&mut self, strategy: QueueStrategy) -> Option<QueueItem> {
        if self.items.is_empty() {
            return None;
        }
        let idx = match strategy {
            QueueStrategy::FIFO => 0,
            QueueStrategy::LIFO => self.items.len() - 1,
            QueueStrategy::RetryFirst => self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.retry_count > 0)
                .max_by_key(|(_, item)| (item.retry_count, std::cmp::Reverse(item.created_at)))
                .map(|(i, _)| i)
                .unwrap_or(0),
            QueueStrategy::PriorityFirst => 0,
        };
        let item = self.items.remove(idx);
        self.pending_acks.insert(item.id.clone(), item.clone());
        Some(item)
    }

    pub fn ack(&mut self, id: &str) -> Result<(), IoError> {
        match self.pending_acks.remove(id) {
            Some(_) => {
                self.failed_ids.remove(id);
                self.persist()
            }
            None => Err(IoError::NotFound(format!(
                "item {id} not in pending acknowledgments"
            ))),
        }
    }

    pub fn nack(&mut self, id: &str) -> Result<(), IoError> {
        let mut item = match self.pending_acks.remove(id) {
            Some(item) => item,
            None => {
                return Err(IoError::NotFound(format!(
                    "item {id} not in pending acknowledgments"
                )))
            }
        };
        item.retry_count += 1;
        if item.retry_count >= item.max_retries {
            self.failed_ids.insert(item.id.clone());
            log::warn!(
                "queue item {} exceeded max retries ({})",
                item.id,
                item.max_retries
            );
        } else {
            log::info!("queue item {} retry {}/{}", item.id, item.retry_count, item.max_retries);
            self.items.push(item);
        }
        self.persist()
    }

    pub fn persist(&self) -> Result<(), IoError> {
        let mut file = File::create(&self.file_path)
            .map_err(|e| IoError::Storage(format!("cannot create queue file: {e}")))?;
        for item in &self.items {
            let line = serde_json::to_string(item)
                .map_err(|e| IoError::Storage(format!("cannot serialize queue item: {e}")))?;
            writeln!(file, "{line}")
                .map_err(|e| IoError::Storage(format!("cannot write queue item: {e}")))?;
        }
        for item in self.pending_acks.values() {
            let line = serde_json::to_string(item)
                .map_err(|e| IoError::Storage(format!("cannot serialize queue item: {e}")))?;
            writeln!(file, "{line}")
                .map_err(|e| IoError::Storage(format!("cannot write queue item: {e}")))?;
        }
        file.flush()
            .map_err(|e| IoError::Storage(format!("cannot flush queue file: {e}")))?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn stats(&self) -> QueueStats {
        let total_pending = self.items.len() + self.pending_acks.len();
        let total = total_pending + self.failed_ids.len();
        let retry_sum: u64 = self
            .items
            .iter()
            .map(|i| i.retry_count as u64)
            .chain(self.pending_acks.values().map(|i| i.retry_count as u64))
            .sum();
        let retry_count = (self.items.len() + self.pending_acks.len()) as f64;
        let avg_retries = if retry_count > 0.0 {
            retry_sum as f64 / retry_count
        } else {
            0.0
        };
        QueueStats {
            total,
            pending: total_pending,
            failed: self.failed_ids.len(),
            avg_retries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Arc, Mutex};

    fn temp_path() -> String {
        format!("/tmp/neotrix_test_queue_{}.jsonl", Uuid::new_v4())
    }

    #[test]
    fn test_enqueue_dequeue_ack() {
        let path = temp_path();
        let mut q = PersistQueue::new(&path).unwrap();
        let id = q.enqueue("hello", "greeting").unwrap();
        assert!(!id.is_empty());
        assert_eq!(q.len(), 1);

        let item = q.dequeue(QueueStrategy::FIFO).unwrap();
        assert_eq!(item.payload, "hello");
        assert_eq!(item.item_type, "greeting");

        q.ack(&id).unwrap();
        assert_eq!(q.len(), 0);
        let stats = q.stats();
        assert_eq!(stats.pending, 0);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_enqueue_dequeue_nack_retry() {
        let path = temp_path();
        let mut q = PersistQueue::new(&path).unwrap();
        let id = q.enqueue("retry-me", "test").unwrap();

        let item = q.dequeue(QueueStrategy::FIFO).unwrap();
        assert_eq!(item.retry_count, 0);

        q.nack(&id).unwrap();
        assert_eq!(q.len(), 1); // back in queue

        let item2 = q.dequeue(QueueStrategy::RetryFirst).unwrap();
        assert_eq!(item2.retry_count, 1);

        q.ack(&item2.id).unwrap();

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_enqueue_dequeue_nack_exhaust() {
        let path = temp_path();
        let mut q = PersistQueue::new(&path).unwrap();
        q.enqueue("exhaust", "test").unwrap();

        for _ in 0..3 {
            let item = q.dequeue(QueueStrategy::FIFO).unwrap();
            let _ = q.nack(&item.id);
        }
        // After max_retries (3) nacks, item is permanently failed
        assert!(q.dequeue(QueueStrategy::FIFO).is_none());

        let stats = q.stats();
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.pending, 0);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_queue_stats() {
        let path = temp_path();
        let mut q = PersistQueue::new(&path).unwrap();
        q.enqueue("a", "type-a").unwrap();
        q.enqueue("b", "type-b").unwrap();
        q.enqueue("c", "type-c").unwrap();

        let stats = q.stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.pending, 3);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_crash_recovery() {
        let path = temp_path();
        {
            let mut q = PersistQueue::new(&path).unwrap();
            q.enqueue("persist-a", "test").unwrap();
            q.enqueue("persist-b", "test").unwrap();
        }
        // Simulate crash recovery: load from file
        let mut q = PersistQueue::new(&path).unwrap();
        assert_eq!(q.len(), 2);

        let item = q.dequeue(QueueStrategy::FIFO).unwrap();
        assert_eq!(item.payload, "persist-a");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_lifo_strategy() {
        let path = temp_path();
        let mut q = PersistQueue::new(&path).unwrap();
        q.enqueue("first", "t").unwrap();
        q.enqueue("second", "t").unwrap();
        q.enqueue("third", "t").unwrap();

        let item = q.dequeue(QueueStrategy::LIFO).unwrap();
        assert_eq!(item.payload, "third");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_thread_safe_wrapper() {
        let path = temp_path();
        let queue = Arc::new(Mutex::new(PersistQueue::new(&path).unwrap()));
        {
            let mut q = queue.lock().unwrap();
            q.enqueue("thread-safe", "test").unwrap();
        }
        {
            let mut q = queue.lock().unwrap();
            let item = q.dequeue(QueueStrategy::FIFO).unwrap();
            assert_eq!(item.payload, "thread-safe");
            q.ack(&item.id).unwrap();
        }
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_dequeue_empty() {
        let path = temp_path();
        let mut q = PersistQueue::new(&path).unwrap();
        assert!(q.dequeue(QueueStrategy::FIFO).is_none());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_ack_nonexistent() {
        let path = temp_path();
        let mut q = PersistQueue::new(&path).unwrap();
        let result = q.ack("nonexistent");
        assert!(result.is_err());
        let _ = fs::remove_file(&path);
    }
}
