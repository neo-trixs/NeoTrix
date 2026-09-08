pub struct FeedDedup;

impl FeedDedup {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FeedDedup {
    fn default() -> Self {
        Self::new()
    }
}
