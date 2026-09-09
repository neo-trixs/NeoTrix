pub mod engine;
pub mod registry;

pub use engine::FeedEngine;
pub use registry::FeedRegistry;

pub struct FeedParser;

impl FeedParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FeedParser {
    fn default() -> Self {
        Self::new()
    }
}
