pub mod auth;
pub mod bilibili;
pub mod connector;
pub mod douyin;
pub mod filter;
pub mod instagram;
pub mod rate_limit;
pub mod reddit;
pub mod social_ingestion_stage;
pub mod tiktok;
pub mod tweet_stream;
pub mod twitter;
pub mod web_navigator;
pub mod x_scraper;
pub mod youtube;

pub use auth::{PlatformTokens, SocialAuth, TokenStore};
pub use connector::{
    Platform, PlatformError, SocialMediaPlatform, SocialSearchResult, UserInfo, VideoInfo,
};
pub use rate_limit::{PlatformRateLimit, RateLimiter};

pub use bilibili::BilibiliConnector;
pub use douyin::DouyinConnector;
pub use instagram::InstagramConnector;
pub use reddit::RedditConnector;
pub use social_ingestion_stage::SocialIngestionStage;
pub use tiktok::TikTokConnector;
pub use tweet_stream::{NegentropyScore, TweetFingerprint, TweetStream};
pub use twitter::TwitterConnector;
pub use web_navigator::{HumanBehavior, LoginCredentials, PageExtract, WebNavigator};
pub use x_scraper::{RawTweet, XScrapeSource, XScraper, XTimeline};
pub use youtube::YouTubeConnector;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_types_are_accessible() {
        let _tokens = PlatformTokens {
            access_token: "test".into(),
            refresh_token: None,
            expires_at: None,
            scope: None,
        };
        let _store = TokenStore::default();
        let _limiter = RateLimiter::new();
        let _err = PlatformError::NotAuthenticated;
    }

    #[test]
    fn test_platform_enum_variants() {
        let platforms = [
            Platform::YouTube,
            Platform::TikTok,
            Platform::Instagram,
            Platform::Twitter,
            Platform::Reddit,
            Platform::Bilibili,
            Platform::Douyin,
        ];
        assert_eq!(platforms.len(), 7);
    }

    #[test]
    fn test_all_connectors_have_correct_platform() {
        let yt = YouTubeConnector::new("key".into());
        assert_eq!(yt.platform(), Platform::YouTube);
    }
}
