use super::x_scraper::XScraper;

/// SocialIngestionStage — 负熵驱动的社交媒体信息吸收
/// Note: BrainStage impl lives in l8_autonomic_impl to avoid L1→L8 dependency.
pub struct SocialIngestionStage {
    scraper: std::sync::Mutex<XScraper>,
    scrape_interval_secs: u64,
}

impl SocialIngestionStage {
    pub fn new() -> Self {
        Self {
            scraper: std::sync::Mutex::new(XScraper::new()),
            scrape_interval_secs: 300,
        }
    }

    pub fn with_interval(mut self, secs: u64) -> Self {
        self.scrape_interval_secs = secs;
        self
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn scrape_interval(&self) -> u64 {
        self.scrape_interval_secs
    }

    pub fn scraper(&self) -> &std::sync::Mutex<XScraper> {
        &self.scraper
    }
}

impl Default for SocialIngestionStage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_ingestion_stage_construction() {
        let stage = SocialIngestionStage::new();
        assert_eq!(stage.scrape_interval(), 300);
    }

    #[test]
    fn test_with_interval() {
        let stage = SocialIngestionStage::new().with_interval(600);
        assert_eq!(stage.scrape_interval(), 600);
    }
}
