//! # DEPRECATED — Use `social_access` module instead
//!
//! These old social sources implement the `MediaSource` trait and are superseded by
//! the `social_access` framework which uses `PlatformExtractor` trait.
//!
//! Migration path:
//! - `twitter::TwitterSource` → `social_access::TwitterExtractor`
//! - `reddit::RedditSource` → `social_access::RedditExtractor`  
//! - `instagram::InstagramSource` → `social_access::InstagramExtractor`
//! - `tiktok::TikTokSource` → `social_access::TikTokExtractor`
//!
//! These modules will be removed in a future version.

pub mod tiktok;
pub mod instagram;
pub mod twitter;
pub mod ytdlp;
