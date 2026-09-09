use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

/// 播放链接重试器
pub struct PlaybackRetry;

impl PlaybackRetry {
    /// 带降级重试的播放地址获取
    pub async fn get_play_url_with_fallback(
        item: &MediaItem,
        quality: Quality,
        sources: &[Box<dyn MediaSource>],
    ) -> Result<ViewSource, String> {
        // 尝试目标音质
        for source in sources {
            if let Ok(view) = source.play_url(item, quality).await {
                return Ok(view);
            }
        }

        // 降级音质重试
        let fallback_qualities = match quality {
            Quality::Flac => vec![Quality::High, Quality::Standard],
            Quality::High => vec![Quality::Standard, Quality::Flac],
            Quality::Standard => vec![Quality::High, Quality::Flac],
            _ => vec![Quality::Standard, Quality::High],
        };

        for fallback_quality in fallback_qualities {
            for source in sources {
                if let Ok(view) = source.play_url(item, fallback_quality).await {
                    return Ok(view);
                }
            }
        }

        Err(format!("All sources failed for: {}", item.title))
    }
}
