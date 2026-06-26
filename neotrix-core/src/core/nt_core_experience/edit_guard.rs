/// TransactionScope-backed edit guard for self-modifying edits.
use super::safety_ball::{BallVerifier, TransactionScope};
use crate::core::nt_core_edit::MicroEdit;

/// Trait for types that can be guarded by EditGuard.
pub trait EditGuardState {
    type State: Clone;
    fn edit_state(&self) -> Self::State;
    fn restore_edit_state(&mut self, state: &Self::State);
    fn apply_micro_edits(&mut self, edits: &[MicroEdit]) -> Vec<usize>;
}

pub struct EditGuard;

impl EditGuard {
    /// Apply edits with TransactionScope + optional BallVerifier.
    /// Returns applied indices (empty if rejected or rolled back).
    pub fn apply_with_guard<T: EditGuardState>(
        brain: &mut T,
        edits: &[MicroEdit],
        verifier: Option<&mut BallVerifier>,
        label: &'static str,
    ) -> Vec<usize>
    where
        <T as EditGuardState>::State: AsRef<[f64]>,
    {
        let snapshot = brain.edit_state();
        let mut scope = TransactionScope::new(&snapshot, label);

        let applied = brain.apply_micro_edits(edits);

        if let Some(v) = verifier {
            let after = brain.edit_state();
            if !verify_capability(v, &snapshot, &after) {
                log::warn!(
                    "[EditGuard] '{}' rejected — rolling back {} edits",
                    label,
                    applied.len()
                );
                brain.restore_edit_state(&snapshot);
                scope.commit();
                return Vec::new();
            }
        }

        scope.commit();
        applied
    }

    /// Quick guard without verifier: just panic-proof with TransactionScope.
    pub fn apply_safe<T: EditGuardState>(
        brain: &mut T,
        edits: &[MicroEdit],
        label: &'static str,
    ) -> Vec<usize> {
        let snapshot = brain.edit_state();
        let mut scope = TransactionScope::new(&snapshot, label);
        let applied = brain.apply_micro_edits(edits);
        scope.commit();
        applied
    }
}

fn verify_capability(
    v: &mut BallVerifier,
    before: &(impl Clone + AsRef<[f64]>),
    after: &(impl Clone + AsRef<[f64]>),
) -> bool {
    let before_slice = before.as_ref();
    let after_slice = after.as_ref();
    if before_slice.len() != after_slice.len() {
        return false;
    }
    for i in 0..before_slice.len() {
        let delta = after_slice[i] - before_slice[i];
        if delta.abs() < 1e-12 {
            continue;
        }
        let prop = super::safety_ball::ModificationProposal {
            target: format!("index_{}", i),
            delta,
            reason: "[edit_guard] auto-verify".into(),
            gate: "[edit_guard]".into(),
        };
        let verdict = v.check_proposal(&prop, before_slice[i]);
        if !verdict.passed {
            return false;
        }
    }
    true
}
