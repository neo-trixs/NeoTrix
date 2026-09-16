//! Symbolic Temporal Reasoning — 吸收自 hirn
//! 日期解析和区间计算在 Rust 中完成，作为证据交给 reader

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalInterval {
    pub start: u64,
    pub end: Option<u64>, // None = ongoing
    pub label: String,
}

impl TemporalInterval {
    pub fn new(start: u64, end: Option<u64>, label: String) -> Self {
        Self { start, end, label }
    }

    pub fn contains(&self, ts: u64) -> bool {
        ts >= self.start && self.end.map_or(true, |e| ts <= e)
    }

    pub fn overlaps(&self, other: &TemporalInterval) -> bool {
        self.start <= other.end.unwrap_or(u64::MAX) &&
        other.start <= self.end.unwrap_or(u64::MAX)
    }

    pub fn duration_secs(&self) -> Option<u64> {
        self.end.map(|e| e - self.start)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalFact {
    pub event_id: String,
    pub interval: TemporalInterval,
    pub confidence: f64,
}

pub struct TemporalReasoner {
    facts: Vec<TemporalFact>,
}

impl TemporalReasoner {
    pub fn new() -> Self {
        Self { facts: Vec::new() }
    }

    pub fn add_fact(&mut self, fact: TemporalFact) {
        self.facts.push(fact);
    }

    /// AS OF query: state at a specific point in time
    pub fn as_of(&self, ts: u64) -> Vec<&TemporalFact> {
        self.facts.iter().filter(|f| f.interval.contains(ts)).collect()
    }

    /// 事件 A 是否在事件 B 之前？
    pub fn is_before(&self, a_id: &str, b_id: &str) -> Option<bool> {
        let a = self.facts.iter().find(|f| f.event_id == a_id)?;
        let b = self.facts.iter().find(|f| f.event_id == b_id)?;
        Some(a.interval.start < b.interval.start)
    }

    /// 重叠查询
    pub fn overlapping(&self) -> Vec<(&TemporalFact, &TemporalFact)> {
        let mut pairs = Vec::new();
        for i in 0..self.facts.len() {
            for j in (i+1)..self.facts.len() {
                if self.facts[i].interval.overlaps(&self.facts[j].interval) {
                    pairs.push((&self.facts[i], &self.facts[j]));
                }
            }
        }
        pairs
    }

    /// 时间区间内的事件
    pub fn in_range(&self, start: u64, end: u64) -> Vec<&TemporalFact> {
        let range = TemporalInterval::new(start, Some(end), "".into());
        self.facts.iter().filter(|f| f.interval.overlaps(&range)).collect()
    }
}

impl Default for TemporalReasoner {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_basic() {
        let mut reasoner = TemporalReasoner::new();
        reasoner.add_fact(TemporalFact {
            event_id: "e1".into(),
            interval: TemporalInterval::new(100, Some(200), "event1".into()),
            confidence: 0.9,
        });
        reasoner.add_fact(TemporalFact {
            event_id: "e2".into(),
            interval: TemporalInterval::new(150, Some(250), "event2".into()),
            confidence: 0.8,
        });

        assert!(reasoner.is_before("e1", "e2").unwrap());

        let at_150 = reasoner.as_of(150);
        assert_eq!(at_150.len(), 2); // Both overlap at 150
    }
}
