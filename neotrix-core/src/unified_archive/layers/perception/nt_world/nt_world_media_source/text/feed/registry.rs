pub struct FeedRegistry;

impl FeedRegistry {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FeedRegistry {
    fn default() -> Self {
        Self::new()
    }
}
