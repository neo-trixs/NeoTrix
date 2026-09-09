pub mod core;
pub mod audio;
pub mod video;
pub mod image;
pub mod text;
pub mod infra;
pub mod evolution;

pub use core::types::*;
pub use core::engine::MediaEngine;
pub use core::api::{MediaApi, SourceInfo, media_api};
pub use core::playback::{PlaybackController, PlayMode, PlaybackState};
pub use core::now_playing::{NowPlaying, PlayerDisplay};
pub use core::resource_store::ResourceStore;
pub use core::lx_script::LxScriptSource;

pub fn build_default_engine() -> MediaEngine {
    use std::sync::Arc;

    let mut engine = MediaEngine::new();

    engine.add_source(Arc::new(audio::netease::NeteaseSource::new()), 1);
    engine.add_source(Arc::new(audio::kuwo::KuwoSource::new()), 2);
    engine.add_source(Arc::new(audio::kugou::KugouSource::new()), 3);
    engine.add_source(Arc::new(audio::migu::MiguSource::new()), 4);
    engine.add_source(Arc::new(audio::soundcloud::SoundCloudSource::new()), 5);
    engine.add_source(Arc::new(audio::spotify::SpotifySource::new()), 6);
    engine.add_source(Arc::new(audio::piped::PipedSource::new()), 7);
    engine.add_source(Arc::new(audio::jiosaavn::JioSaavnSource::new()), 8);
    engine.add_source(Arc::new(audio::deezer::DeezerSource::new()), 9);
    engine.add_source(Arc::new(audio::bandcamp::BandcampSource::new()), 10);
    engine.add_source(Arc::new(audio::qqmusic::QQMusicSource::new()), 11);

    engine.add_source(Arc::new(video::youtube::YouTubeSource::new()), 12);
    engine.add_source(Arc::new(video::bilibili::BilibiliSource::new()), 13);
    engine.add_source(Arc::new(video::vimeo::VimeoSource::new()), 14);

    engine.add_source(Arc::new(image::pexels::PexelsSource::new()), 15);
    engine.add_source(Arc::new(image::unsplash::UnsplashSource::new()), 16);
    engine.add_source(Arc::new(image::pixabay::PixabaySource::new()), 17);
    engine.add_source(Arc::new(image::wikimedia::WikimediaSource::new()), 18);

    engine.add_source(Arc::new(text::document::arxiv::ArxivSource::new()), 19);
    engine.add_source(Arc::new(text::document::semantic_scholar::SemanticScholarSource::new()), 20);
    engine.add_source(Arc::new(text::book::openlibrary::OpenLibrarySource::new()), 21);
    engine.add_source(Arc::new(text::book::annas_archive::AnnasArchiveSource::new()), 22);
    engine.add_source(Arc::new(text::lyrics::lrclib::LrclibSource::new()), 23);
    engine.add_source(Arc::new(text::lyrics::multi::MultiLyricSource::new()), 24);
    engine.add_source(Arc::new(text::lyrics::genius::GeniusSource::new()), 25);
    engine.add_source(Arc::new(text::social::tiktok::TikTokSource::new()), 26);
    engine.add_source(Arc::new(text::social::instagram::InstagramSource::new()), 27);
    engine.add_source(Arc::new(text::social::twitter::TwitterSource::new()), 28);
    engine.add_source(Arc::new(text::social::ytdlp::YtdlpSource::new()), 29);

    engine
}
