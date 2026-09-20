//! Web content acquisition with adaptive selectors and anti-detection.
//! Inspired by Scrapling.

pub mod selector;
pub mod fetcher;

pub use selector::*;
pub use fetcher::*;
