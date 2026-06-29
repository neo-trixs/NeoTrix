use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SM2Item {
    pub id: u64,
    pub memory_id: String,
    pub repetitions: u32,
    pub interval_days: f64,
    pub easiness_factor: f64,
    pub next_review_at: u64,
    pub last_reviewed_at: u64,
    pub quality: u8,
    pub times_recalled: u32,
    pub times_forgotten: u32,
    pub vsa_signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SM2Scheduler {
    items: Vec<SM2Item>,
    next_id: u64,
    default_interval_days: f64,
    max_interval_days: f64,
}

impl SM2Scheduler {
    pub fn new(default_interval_days: f64, max_interval_days: f64) -> Self {
        Self {
            items: Vec::new(),
            next_id: 0,
            default_interval_days,
            max_interval_days,
        }
    }

    pub fn add_item(&mut self, memory_id: &str, vsa_signature: Vec<u8>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.items.push(SM2Item {
            id,
            memory_id: memory_id.to_string(),
            repetitions: 0,
            interval_days: self.default_interval_days,
            easiness_factor: 2.5,
            next_review_at: now + (self.default_interval_days as u64 * 86400),
            last_reviewed_at: now,
            quality: 0,
            times_recalled: 0,
            times_forgotten: 0,
            vsa_signature,
        });
        id
    }

    pub fn review_item(&mut self, id: u64, quality: u8) {
        let quality = quality.min(5);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Some(item) = self.items.iter_mut().find(|it| it.id == id) {
            item.last_reviewed_at = now;
            item.quality = quality;
            if quality >= 3 {
                item.repetitions += 1;
                item.times_recalled += 1;
                if item.repetitions == 1 {
                    item.interval_days = 1.0;
                } else if item.repetitions == 2 {
                    item.interval_days = 6.0;
                } else {
                    item.interval_days =
                        (item.interval_days * item.easiness_factor).min(self.max_interval_days);
                }
            } else {
                item.repetitions = 0;
                item.interval_days = 1.0;
                item.times_forgotten += 1;
            }
            let ef = item.easiness_factor
                + (0.1 - (5.0 - quality as f64) * (0.08 + (5.0 - quality as f64) * 0.02));
            item.easiness_factor = ef.max(1.3);
            item.next_review_at = now + (item.interval_days as u64 * 86400);
        }
    }

    pub fn items_due_now(&self, timestamp: u64) -> Vec<&SM2Item> {
        self.items
            .iter()
            .filter(|it| it.next_review_at <= timestamp)
            .collect()
    }

    pub fn items_due_before(&self, timestamp: u64) -> Vec<&SM2Item> {
        self.items
            .iter()
            .filter(|it| it.next_review_at <= timestamp)
            .collect()
    }

    pub fn overdue_count(&self, timestamp: u64) -> usize {
        self.items
            .iter()
            .filter(|it| it.next_review_at <= timestamp)
            .count()
    }

    pub fn report(&self) -> String {
        self.scheduler_report()
    }

    pub fn scheduler_report(&self) -> String {
        let total = self.items.len();
        let due = self
            .items
            .iter()
            .filter(|it| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                it.next_review_at <= now
            })
            .count();
        let avg_ef: f64 = if total > 0 {
            self.items.iter().map(|it| it.easiness_factor).sum::<f64>() / total as f64
        } else {
            0.0
        };
        format!(
            "SM2Scheduler: {} items, {} due, avg EF {:.2}, max interval {:.0}d",
            total, due, avg_ef, self.max_interval_days
        )
    }

    pub fn consolidation_candidates(&self, limit: usize, timestamp: u64) -> Vec<&SM2Item> {
        let mut candidates: Vec<&SM2Item> = self
            .items
            .iter()
            .filter(|it| it.next_review_at <= timestamp)
            .collect();
        candidates.sort_by(|a, b| {
            let a_priority = a.interval_days / (a.easiness_factor.max(0.1));
            let b_priority = b.interval_days / (b.easiness_factor.max(0.1));
            a_priority
                .partial_cmp(&b_priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(limit);
        candidates
    }

    pub fn mark_reviewed(&mut self, id: u64, quality: u8, timestamp: u64) {
        if let Some(item) = self.items.iter_mut().find(|it| it.id == id) {
            let quality = quality.min(5);
            item.last_reviewed_at = timestamp;
            item.quality = quality;
            if quality >= 3 {
                item.repetitions += 1;
                item.times_recalled += 1;
                if item.repetitions == 1 {
                    item.interval_days = 1.0;
                } else if item.repetitions == 2 {
                    item.interval_days = 6.0;
                } else {
                    item.interval_days =
                        (item.interval_days * item.easiness_factor).min(self.max_interval_days);
                }
            } else {
                item.repetitions = 0;
                item.interval_days = 1.0;
                item.times_forgotten += 1;
            }
            let ef = item.easiness_factor
                + (0.1 - (5.0 - quality as f64) * (0.08 + (5.0 - quality as f64) * 0.02));
            item.easiness_factor = ef.max(1.3);
            item.next_review_at = timestamp + (item.interval_days as u64 * 86400);
        }
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn get_item(&self, id: u64) -> Option<&SM2Item> {
        self.items.iter().find(|it| it.id == id)
    }

    pub fn all_items(&self) -> &[SM2Item] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scheduler() -> SM2Scheduler {
        SM2Scheduler::new(1.0, 365.0)
    }

    #[test]
    fn test_add_item() {
        let mut s = scheduler();
        let id = s.add_item("mem_1", vec![1, 2, 3]);
        assert_eq!(s.item_count(), 1);
        let item = s.get_item(id).unwrap();
        assert_eq!(item.memory_id, "mem_1");
        assert!((item.easiness_factor - 2.5).abs() < 1e-6);
        assert_eq!(item.interval_days, 1.0);
    }

    #[test]
    fn test_review_item_quality_high() {
        let mut s = scheduler();
        let id = s.add_item("mem_1", vec![1, 2, 3]);
        s.review_item(id, 4);
        let item = s.get_item(id).unwrap();
        assert_eq!(item.repetitions, 1);
        assert_eq!(item.interval_days, 1.0);
        assert!(item.easiness_factor > 2.5);
    }

    #[test]
    fn test_review_item_quality_low() {
        let mut s = scheduler();
        let id = s.add_item("mem_1", vec![1, 2, 3]);
        s.review_item(id, 1);
        let item = s.get_item(id).unwrap();
        assert_eq!(item.repetitions, 0);
        assert_eq!(item.interval_days, 1.0);
        assert!(item.easiness_factor < 2.5);
    }

    #[test]
    fn test_review_item_spaced_repetition() {
        let mut s = scheduler();
        let id = s.add_item("mem_1", vec![1, 2, 3]);
        s.review_item(id, 4);
        assert!((s.get_item(id).unwrap().easiness_factor - 2.5).abs() < 0.2);
        assert_eq!(s.get_item(id).unwrap().interval_days, 1.0);
        s.review_item(id, 4);
        assert_eq!(s.get_item(id).unwrap().interval_days, 6.0);
        s.review_item(id, 4);
        let ef = s.get_item(id).unwrap().easiness_factor;
        let expected = 6.0 * ef;
        assert!(
            (s.get_item(id).unwrap().interval_days - expected).abs() < 0.1,
            "interval should be interval * EF = {} * {} = {}",
            6.0,
            ef,
            expected
        );
    }

    #[test]
    fn test_items_due_now() {
        let mut s = scheduler();
        s.add_item("mem_1", vec![]);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let due = s.items_due_now(now + 86400 * 365);
        assert_eq!(due.len(), 1);
    }

    #[test]
    fn test_overdue_count() {
        let mut s = scheduler();
        s.add_item("mem_1", vec![]);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        assert_eq!(s.overdue_count(now - 1), 0);
        assert_eq!(s.overdue_count(now + 86400 * 365), 1);
    }

    #[test]
    fn test_consolidation_candidates() {
        let mut s = scheduler();
        s.add_item("mem_1", vec![]);
        s.add_item("mem_2", vec![]);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let candidates = s.consolidation_candidates(10, now + 86400 * 365);
        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn test_mark_reviewed() {
        let mut s = scheduler();
        let id = s.add_item("mem_1", vec![]);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        s.mark_reviewed(id, 5, now);
        let item = s.get_item(id).unwrap();
        assert_eq!(item.repetitions, 1);
        assert_eq!(item.last_reviewed_at, now);
    }

    #[test]
    fn test_scheduler_report() {
        let mut s = scheduler();
        s.add_item("mem_1", vec![]);
        let report = s.scheduler_report();
        assert!(report.contains("SM2Scheduler"));
        assert!(report.contains("1 items"));
    }
}
