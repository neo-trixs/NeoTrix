pub mod document;
pub mod book;
pub mod lyrics;
pub mod feed;
pub mod social;

pub use document::arxiv::ArxivSource;
pub use document::semantic_scholar::SemanticScholarSource;
pub use book::openlibrary::OpenLibrarySource;
pub use book::annas_archive::AnnasArchiveSource;
pub use lyrics::lrclib::LrclibSource;
pub use lyrics::genius::GeniusSource;
pub use lyrics::multi::MultiLyricSource;
pub use feed::{FeedParser, FeedEngine, FeedRegistry};
pub use social::tiktok::TikTokSource;
pub use social::instagram::InstagramSource;
pub use social::twitter::TwitterSource;
pub use social::ytdlp::YtdlpSource;
