use super::brain_impl::ReasoningBrain;
use crate::core::nt_core_edit::MicroEdit;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalStatus {
    Open,
    Committed,
    RolledBack,
    Failed(&'static str),
}

#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub id: u64,
    pub edits: Vec<MicroEdit>,
    pub snapshot_capability: Vec<f64>,
    pub snapshot_learning_rate: f64,
    pub status: JournalStatus,
    pub created_at: u64,
}

pub struct EditJournal {
    entries: Vec<JournalEntry>,
    next_id: u64,
    max_entries: usize,
}

impl EditJournal {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
            max_entries: max_entries.max(1),
        }
    }

    pub fn begin_transaction(&mut self, edits: Vec<MicroEdit>, brain: &ReasoningBrain) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let entry = JournalEntry {
            id,
            snapshot_capability: brain.capability.arr().to_vec(),
            snapshot_learning_rate: brain.learning_rate,
            edits,
            status: JournalStatus::Open,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        self.entries.push(entry);
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
        id
    }

    pub fn find_mut(&mut self, tx_id: u64) -> Option<&mut JournalEntry> {
        self.entries.iter_mut().find(|e| e.id == tx_id)
    }

    pub fn commit(&mut self, tx_id: u64, brain: &mut ReasoningBrain) -> bool {
        let entry = match self.find_mut(tx_id) {
            Some(e) if e.status == JournalStatus::Open => e,
            _ => return false,
        };

        for edit in &entry.edits {
            let edits_slice = [edit.clone()];
            let applied = brain.apply_micro_edits(&edits_slice);
            if applied.is_empty() {
                entry.status = JournalStatus::Failed("apply returned empty");
                return false;
            }
        }

        entry.status = JournalStatus::Committed;
        true
    }

    pub fn rollback(&mut self, tx_id: u64, brain: &mut ReasoningBrain) -> bool {
        let entry = match self.find_mut(tx_id) {
            Some(e) if e.status == JournalStatus::Open => e,
            _ => return false,
        };

        for (i, val) in entry.snapshot_capability.iter().enumerate() {
            if i < brain.capability.arr().len() {
                brain.capability.arr_mut()[i] = *val;
            }
        }
        brain.learning_rate = entry.snapshot_learning_rate;
        entry.status = JournalStatus::RolledBack;
        true
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        let committed = self
            .entries
            .iter()
            .filter(|e| e.status == JournalStatus::Committed)
            .count();
        let rolled = self
            .entries
            .iter()
            .filter(|e| e.status == JournalStatus::RolledBack)
            .count();
        (self.entries.len(), committed, rolled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_edit::MicroEdit;
    use crate::neotrix::nt_mind::self_iterating::ReasoningBrain;

    #[test]
    fn test_commit_and_rollback() {
        let mut brain = ReasoningBrain::default();
        let _starting_lr = brain.learning_rate;
        let mut journal = EditJournal::new(10);

        let edits = vec![MicroEdit::UpdateLearningRate(0.5)];
        let tx_id = journal.begin_transaction(edits.clone(), &brain);
        journal.commit(tx_id, &mut brain);
        assert!(brain.learning_rate - 0.5 < 0.001);

        let tx_id2 = journal.begin_transaction(edits.clone(), &brain);
        journal.rollback(tx_id2, &mut brain);
        assert!((brain.learning_rate - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_begin_then_rollback_restores_original() {
        let mut brain = ReasoningBrain::default();
        let snapshot = brain.capability.arr().to_vec();
        let mut journal = EditJournal::new(10);

        let edits = vec![MicroEdit::AdjustDimension("intelligence".into(), 0.3)];
        let tx_id = journal.begin_transaction(edits, &brain);
        journal.rollback(tx_id, &mut brain);
        for (i, v) in snapshot.iter().enumerate() {
            assert!((brain.capability.arr()[i] - v).abs() < 0.001);
        }
    }

    #[test]
    fn test_double_rollback_fails() {
        let mut brain = ReasoningBrain::default();
        let mut journal = EditJournal::new(10);

        let tx_id = journal.begin_transaction(vec![MicroEdit::NormalizeVector], &brain);
        assert!(journal.rollback(tx_id, &mut brain));
        assert!(!journal.rollback(tx_id, &mut brain));
    }

    #[test]
    fn test_stats() {
        let mut brain = ReasoningBrain::default();
        let mut journal = EditJournal::new(10);

        let t1 = journal.begin_transaction(vec![MicroEdit::NormalizeVector], &brain);
        journal.commit(t1, &mut brain);
        let t2 = journal.begin_transaction(vec![MicroEdit::UpdateLearningRate(0.1)], &brain);
        journal.rollback(t2, &mut brain);
        let (total, committed, rolled) = journal.stats();
        assert_eq!(total, 2);
        assert_eq!(committed, 1);
        assert_eq!(rolled, 1);
    }
}
