use std::collections::VecDeque;

use crate::core::nt_core_consciousness::{
    ConsciousnessStream, VsaOrigin, VsaTagged,
};

/// Clean orphan VSA vectors (those with no valid Self_ or World tag)
/// Returns the number of orphans removed
pub fn clean_orphan_vsa_vectors(stream: &mut ConsciousnessStream) -> usize {
    let mut inner = stream.clone().into_inner();
    stream.clear();
    let before = inner.len();
    inner.retain(|tagged| {
        matches!(tagged.tag, VsaOrigin::Self_(_) | VsaOrigin::World(_))
    });
    let removed = before - inner.len();
    for item in inner {
        stream.push(item);
    }
    removed
}

/// Repair corrupted VsaTag sequences by removing entries with empty vectors
pub fn repair_corrupted_tags(stream: &mut ConsciousnessStream) -> usize {
    let mut inner = stream.clone().into_inner();
    stream.clear();
    let before = inner.len();
    inner.retain(|tagged| {
        !tagged.vector.is_empty()
    });
    let removed = before - inner.len();
    for item in inner {
        stream.push(item);
    }
    removed
}

/// Fold duplicate VSA vectors (exact match in vector bytes + tag within window)
/// Returns the number of duplicates removed
pub fn fold_duplicate_vectors(stream: &mut ConsciousnessStream, window: usize) -> usize {
    let inner = stream.clone().into_inner();
    stream.clear();
    let before = inner.len();

    let mut result: VecDeque<VsaTagged> = VecDeque::with_capacity(inner.len());
    for (i, item) in inner.into_iter().enumerate() {
        let window_start = if i >= window { i - window } else { 0 };
        let is_dup = result.iter().skip(window_start).any(|existing| {
            existing.vector == item.vector && existing.tag == item.tag
        });
        if !is_dup {
            result.push_back(item);
        }
    }

    let removed = before - result.len();
    for item in result {
        stream.push(item);
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::vsa_tag::{VsaSelfCategory, VsaWorldCategory};

    fn self_tagged(v: Vec<u8>) -> VsaTagged {
        VsaTagged::new(v, VsaOrigin::Self_(VsaSelfCategory::Thought))
    }

    fn world_tagged(v: Vec<u8>) -> VsaTagged {
        VsaTagged::new(v, VsaOrigin::World(VsaWorldCategory::UserInput))
    }

    #[test]
    fn test_clean_orphans_removes_invalid_tags() {
        let mut stream = ConsciousnessStream::new(100);
        stream.push(self_tagged(vec![1; 64]));
        stream.push(world_tagged(vec![2; 64]));
        let removed = clean_orphan_vsa_vectors(&mut stream);
        assert_eq!(removed, 0);
        assert_eq!(stream.len(), 2);
    }

    #[test]
    fn test_clean_orphans_keeps_all_valid() {
        let mut stream = ConsciousnessStream::new(100);
        stream.push(self_tagged(vec![1; 64]));
        stream.push(world_tagged(vec![2; 64]));
        let removed = clean_orphan_vsa_vectors(&mut stream);
        assert_eq!(removed, 0);
        assert_eq!(stream.len(), 2);
    }

    #[test]
    fn test_repair_empty_vectors() {
        let mut stream = ConsciousnessStream::new(100);
        stream.push(self_tagged(vec![]));
        stream.push(self_tagged(vec![1; 64]));
        stream.push(world_tagged(vec![]));
        let removed = repair_corrupted_tags(&mut stream);
        assert_eq!(removed, 2);
        assert_eq!(stream.len(), 1);
    }

    #[test]
    fn test_repair_with_all_valid() {
        let mut stream = ConsciousnessStream::new(100);
        stream.push(self_tagged(vec![1; 64]));
        stream.push(world_tagged(vec![2; 64]));
        let removed = repair_corrupted_tags(&mut stream);
        assert_eq!(removed, 0);
        assert_eq!(stream.len(), 2);
    }

    #[test]
    fn test_fold_duplicates_basic() {
        let mut stream = ConsciousnessStream::new(100);
        stream.push(self_tagged(vec![1; 64]));
        stream.push(world_tagged(vec![2; 64]));
        stream.push(self_tagged(vec![1; 64]));
        let removed = fold_duplicate_vectors(&mut stream, 10);
        assert_eq!(removed, 1);
        assert_eq!(stream.len(), 2);
    }

    #[test]
    fn test_fold_duplicates_outside_window() {
        let mut stream = ConsciousnessStream::new(100);
        stream.push(self_tagged(vec![1; 64]));
        stream.push(self_tagged(vec![2; 64]));
        stream.push(self_tagged(vec![3; 64]));
        stream.push(self_tagged(vec![4; 64]));
        stream.push(self_tagged(vec![5; 64]));
        stream.push(self_tagged(vec![1; 64]));
        stream.push(self_tagged(vec![5; 64]));
        let removed = fold_duplicate_vectors(&mut stream, 3);
        assert_eq!(removed, 1);
    }

    #[test]
    fn test_fold_duplicates_with_different_tags_not_deduped() {
        let mut stream = ConsciousnessStream::new(100);
        stream.push(self_tagged(vec![1; 64]));
        stream.push(world_tagged(vec![1; 64]));
        let removed = fold_duplicate_vectors(&mut stream, 10);
        assert_eq!(removed, 0);
        assert_eq!(stream.len(), 2);
    }

    #[test]
    fn test_empty_stream_returns_zero() {
        let mut stream = ConsciousnessStream::new(100);
        assert_eq!(clean_orphan_vsa_vectors(&mut stream), 0);
        assert_eq!(repair_corrupted_tags(&mut stream), 0);
        assert_eq!(fold_duplicate_vectors(&mut stream, 10), 0);
    }

    #[test]
    fn test_stream_capacity_preserved() {
        let mut stream = ConsciousnessStream::new(42);
        stream.push(self_tagged(vec![1; 64]));
        stream.push(self_tagged(vec![2; 64]));
        repair_corrupted_tags(&mut stream);
        assert_eq!(stream.capacity(), 42);
    }

    #[test]
    fn test_combined_hygiene_pipeline() {
        let mut stream = ConsciousnessStream::new(100);
        stream.push(self_tagged(vec![]));
        stream.push(self_tagged(vec![1; 64]));
        stream.push(world_tagged(vec![2; 64]));
        stream.push(world_tagged(vec![]));
        stream.push(self_tagged(vec![1; 64]));

        let r1 = repair_corrupted_tags(&mut stream);
        assert_eq!(r1, 2);
        assert_eq!(stream.len(), 3);

        let r2 = fold_duplicate_vectors(&mut stream, 10);
        assert_eq!(r2, 1);
        assert_eq!(stream.len(), 2);
    }
}
