pub mod helpers;
pub mod tracker;
pub mod types;

pub use self::tracker::AdaptiveTracker;
pub use self::types::{
    DomElement, DomSnapshot, ElementFingerprint, ElementSnapshot, FallbackSelectors, FuzzyMatch,
    SavedElement,
};

#[cfg(test)]
mod tests;
