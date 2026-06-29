use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GapType {
    Contradiction,
    LowConfidence,
    DriveGap,
    KnowledgeMissing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStatus {
    Unresolved,
    InProgress,
    Resolved { at: u64, how: String },
    Superseded { by_id: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicGap {
    pub id: u64,
    pub gap_type: GapType,
    pub priority: f64,
    pub domain: String,
    pub description: String,
    pub vsa_signature: Vec<u8>,
    pub created_at: u64,
    pub resolution_status: ResolutionStatus,
    pub related_gaps: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicQueue {
    gaps: Vec<EpistemicGap>,
    next_id: u64,
    max_size: usize,
}

impl EpistemicQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            gaps: Vec::with_capacity(max_size),
            next_id: 1,
            max_size,
        }
    }

    pub fn push(
        &mut self,
        gap_type: GapType,
        priority: f64,
        domain: String,
        description: String,
        vsa_signature: Vec<u8>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let gap = EpistemicGap {
            id,
            gap_type,
            priority: priority.clamp(0.0, 1.0),
            domain,
            description,
            vsa_signature,
            created_at: now,
            resolution_status: ResolutionStatus::Unresolved,
            related_gaps: Vec::new(),
        };

        self.gaps.push(gap);
        self.gaps.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if self.gaps.len() > self.max_size {
            self.gaps.pop();
        }

        id
    }

    pub fn pop_highest_priority(&mut self) -> Option<EpistemicGap> {
        let idx = self
            .gaps
            .iter()
            .position(|g| matches!(g.resolution_status, ResolutionStatus::Unresolved))?;
        Some(self.gaps.remove(idx))
    }

    pub fn peek_top(&self, n: usize) -> Vec<&EpistemicGap> {
        self.gaps
            .iter()
            .filter(|g| matches!(g.resolution_status, ResolutionStatus::Unresolved))
            .take(n)
            .collect()
    }

    pub fn resolve(&mut self, id: u64, how: String) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Some(gap) = self.gaps.iter_mut().find(|g| g.id == id) {
            gap.resolution_status = ResolutionStatus::Resolved { at: now, how };
            true
        } else {
            false
        }
    }

    pub fn supersede(&mut self, id: u64, by_id: u64) -> bool {
        if let Some(gap) = self.gaps.iter_mut().find(|g| g.id == id) {
            gap.resolution_status = ResolutionStatus::Superseded { by_id };
            true
        } else {
            false
        }
    }

    pub fn unresolved_count(&self) -> usize {
        self.gaps
            .iter()
            .filter(|g| matches!(g.resolution_status, ResolutionStatus::Unresolved))
            .count()
    }

    pub fn gaps_by_type(&self, gap_type: GapType) -> Vec<&EpistemicGap> {
        self.gaps
            .iter()
            .filter(|g| g.gap_type == gap_type)
            .collect()
    }

    pub fn summary(&self) -> String {
        let total = self.gaps.len();
        let unresolved = self.unresolved_count();
        let by_type = [
            GapType::Contradiction,
            GapType::LowConfidence,
            GapType::DriveGap,
            GapType::KnowledgeMissing,
        ]
        .iter()
        .map(|gt| {
            let cnt = self.gaps_by_type(*gt).len();
            format!("{:?}:{}", gt, cnt)
        })
        .collect::<Vec<_>>()
        .join(", ");

        let avg_priority = if self.gaps.is_empty() {
            0.0
        } else {
            self.gaps.iter().map(|g| g.priority).sum::<f64>() / self.gaps.len() as f64
        };

        format!(
            "EpistemicQueue: {}/{} unresolved, avg_priority={:.3}, [{}]",
            unresolved, total, avg_priority, by_type
        )
    }

    pub fn ingest_from_curiosity(
        &mut self,
        domain: &str,
        description: &str,
        intensity: f64,
        vsa: Vec<u8>,
    ) -> Option<u64> {
        let gap_type = if intensity > 0.7 {
            GapType::LowConfidence
        } else {
            GapType::KnowledgeMissing
        };

        let id = self.push(
            gap_type,
            intensity,
            domain.to_string(),
            description.to_string(),
            vsa,
        );
        Some(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vsa() -> Vec<u8> {
        vec![0u8; 64]
    }

    #[test]
    fn test_new_queue_empty() {
        let q = EpistemicQueue::new(100);
        assert_eq!(q.unresolved_count(), 0);
        assert!(q.peek_top(5).is_empty());
    }

    #[test]
    fn test_push_and_unresolved_count() {
        let mut q = EpistemicQueue::new(100);
        q.push(
            GapType::KnowledgeMissing,
            0.8,
            "math".into(),
            "missing info".into(),
            make_vsa(),
        );
        assert_eq!(q.unresolved_count(), 1);
    }

    #[test]
    fn test_push_sorts_by_priority() {
        let mut q = EpistemicQueue::new(100);
        q.push(
            GapType::KnowledgeMissing,
            0.3,
            "a".into(),
            "low".into(),
            make_vsa(),
        );
        q.push(
            GapType::Contradiction,
            0.9,
            "b".into(),
            "high".into(),
            make_vsa(),
        );
        let top = q.peek_top(10);
        assert_eq!(top.len(), 2);
        assert!(top[0].priority > top[1].priority);
    }

    #[test]
    fn test_pop_highest_priority() {
        let mut q = EpistemicQueue::new(100);
        q.push(GapType::DriveGap, 0.5, "x".into(), "mid".into(), make_vsa());
        q.push(
            GapType::Contradiction,
            0.9,
            "y".into(),
            "high".into(),
            make_vsa(),
        );
        let popped = q.pop_highest_priority().unwrap();
        assert_eq!(popped.priority, 0.9);
        assert_eq!(q.unresolved_count(), 1);
    }

    #[test]
    fn test_resolve() {
        let mut q = EpistemicQueue::new(100);
        let id = q.push(
            GapType::LowConfidence,
            0.6,
            "phy".into(),
            "uncertain".into(),
            make_vsa(),
        );
        assert!(q.resolve(id, "found evidence".into()));
        assert_eq!(q.unresolved_count(), 0);
        let gap = q.gaps.iter().find(|g| g.id == id).unwrap();
        assert!(matches!(
            gap.resolution_status,
            ResolutionStatus::Resolved { .. }
        ));
    }

    #[test]
    fn test_supersede() {
        let mut q = EpistemicQueue::new(100);
        let id1 = q.push(
            GapType::KnowledgeMissing,
            0.7,
            "a".into(),
            "old".into(),
            make_vsa(),
        );
        let id2 = q.push(
            GapType::KnowledgeMissing,
            0.8,
            "b".into(),
            "new".into(),
            make_vsa(),
        );
        assert!(q.supersede(id1, id2));
        let gap = q.gaps.iter().find(|g| g.id == id1).unwrap();
        assert!(matches!(
            gap.resolution_status,
            ResolutionStatus::Superseded { .. }
        ));
    }

    #[test]
    fn test_gaps_by_type() {
        let mut q = EpistemicQueue::new(100);
        q.push(
            GapType::Contradiction,
            0.5,
            "d".into(),
            "c1".into(),
            make_vsa(),
        );
        q.push(
            GapType::Contradiction,
            0.6,
            "d".into(),
            "c2".into(),
            make_vsa(),
        );
        q.push(GapType::DriveGap, 0.4, "d".into(), "dg".into(), make_vsa());
        assert_eq!(q.gaps_by_type(GapType::Contradiction).len(), 2);
        assert_eq!(q.gaps_by_type(GapType::DriveGap).len(), 1);
        assert_eq!(q.gaps_by_type(GapType::KnowledgeMissing).len(), 0);
    }

    #[test]
    fn test_ingest_from_curiosity() {
        let mut q = EpistemicQueue::new(100);
        let id = q.ingest_from_curiosity("math", "missing theorem", 0.5, make_vsa());
        assert!(id.is_some());
        assert_eq!(q.unresolved_count(), 1);
        let gap = q.gaps.first().unwrap();
        assert_eq!(gap.gap_type, GapType::KnowledgeMissing);
    }

    #[test]
    fn test_ingest_from_curiosity_high_intensity() {
        let mut q = EpistemicQueue::new(100);
        let id = q.ingest_from_curiosity("physics", "uncertain prediction", 0.8, make_vsa());
        assert!(id.is_some());
        let gap = q.gaps.first().unwrap();
        assert_eq!(gap.gap_type, GapType::LowConfidence);
    }

    #[test]
    fn test_max_size_eviction() {
        let mut q = EpistemicQueue::new(3);
        q.push(
            GapType::KnowledgeMissing,
            0.1,
            "a".into(),
            "low".into(),
            make_vsa(),
        );
        q.push(
            GapType::KnowledgeMissing,
            0.2,
            "b".into(),
            "low".into(),
            make_vsa(),
        );
        q.push(
            GapType::KnowledgeMissing,
            0.3,
            "c".into(),
            "low".into(),
            make_vsa(),
        );
        q.push(
            GapType::KnowledgeMissing,
            0.9,
            "d".into(),
            "high".into(),
            make_vsa(),
        );
        assert_eq!(q.gaps.len(), 3);
        assert!(q.gaps.iter().any(|g| g.description == "high"));
    }

    #[test]
    fn test_summary_format() {
        let mut q = EpistemicQueue::new(100);
        q.push(
            GapType::KnowledgeMissing,
            0.8,
            "math".into(),
            "missing".into(),
            make_vsa(),
        );
        let s = q.summary();
        assert!(s.contains("EpistemicQueue"));
        assert!(s.contains("KnowledgeMissing"));
    }

    #[test]
    fn test_resolve_nonexistent() {
        let mut q = EpistemicQueue::new(100);
        assert!(!q.resolve(999, "nope".into()));
    }

    #[test]
    fn test_supersede_nonexistent() {
        let mut q = EpistemicQueue::new(100);
        assert!(!q.supersede(999, 888));
    }

    #[test]
    fn test_pop_empty_returns_none() {
        let mut q = EpistemicQueue::new(100);
        assert!(q.pop_highest_priority().is_none());
    }

    #[test]
    fn test_priority_clamped() {
        let mut q = EpistemicQueue::new(100);
        q.push(
            GapType::DriveGap,
            5.0,
            "z".into(),
            "over".into(),
            make_vsa(),
        );
        assert!((q.gaps[0].priority - 1.0).abs() < 1e-9);
    }
}
