pub mod auth;
pub mod connector;
pub mod rate_limit;
pub mod youtube;
pub mod tiktok;
pub mod instagram;
pub mod twitter;
pub mod reddit;
pub mod bilibili;
pub mod filter;
pub mod douyin;
pub mod web_navigator;
pub mod x_scraper;
pub mod tweet_stream;
pub mod social_ingestion_stage;


pub use auth::{PlatformTokens, SocialAuth, TokenStore};
pub use connector::{
    Platform, PlatformError, SocialSearchResult, SocialMediaPlatform, UserInfo, VideoInfo,
};
pub use rate_limit::{PlatformRateLimit, RateLimiter};

pub use youtube::YouTubeConnector;
pub use tiktok::TikTokConnector;
pub use instagram::InstagramConnector;
pub use twitter::TwitterConnector;
pub use reddit::RedditConnector;
pub use bilibili::BilibiliConnector;
pub use douyin::DouyinConnector;
pub use web_navigator::{WebNavigator, HumanBehavior, PageExtract, LoginCredentials};
pub use x_scraper::{XScraper, RawTweet, XTimeline, XScrapeSource};
pub use tweet_stream::{TweetStream, TweetFingerprint, NegentropyScore};
pub use social_ingestion_stage::SocialIngestionStage;

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
