use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;

pub fn score_result(item: &MediaItem, query: &str) -> f64 {
    let title_score = fuzzy_match(&item.title, query);
    let artist_score = fuzzy_match(&item.artist, query);
    let quality_score = item.qualities.len() as f64 / 5.0;
    title_score * 0.5 + artist_score * 0.3 + quality_score.min(1.0) * 0.2
}

pub fn fuzzy_match(text: &str, query: &str) -> f64 {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    if text_lower == query_lower {
        return 1.0;
    }
    if text_lower.contains(&query_lower) {
        return 0.85;
    }
    if query_lower.contains(&text_lower) && !text_lower.is_empty() {
        return 0.75;
    }

    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    let text_words: Vec<&str> = text_lower.split_whitespace().collect();

    if query_words.is_empty() || text_words.is_empty() {
        return char_overlap_score(&text_lower, &query_lower);
    }

    let matched = query_words
        .iter()
        .filter(|qw| text_words.iter().any(|tw| tw.contains(qw) || qw.contains(tw)))
        .count();

    let base = matched as f64 / query_words.len() as f64;

    let char_score = char_overlap_score(&text_lower, &query_lower);

    (base * 0.7 + char_score * 0.3).min(1.0)
}

fn char_overlap_score(text: &str, query: &str) -> f64 {
    if query.is_empty() {
        return 0.0;
    }
    let query_chars: std::collections::HashSet<char> = query.chars().collect();
    let matched = text.chars().filter(|c| query_chars.contains(c)).count();
    matched as f64 / query_chars.len() as f64
}

pub fn rank_results(items: &mut Vec<MediaItem>, query: &str) {
    items.sort_by(|a, b| {
        let sa = score_result(a, query);
        let sb = score_result(b, query);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(title: &str, artist: &str, n_qualities: usize) -> MediaItem {
        let qualities: Vec<Quality> = match n_qualities {
            0 => vec![],
            1 => vec![Quality::High],
            2 => vec![Quality::Flac, Quality::High],
            3 => vec![Quality::Flac24bit, Quality::Flac, Quality::High],
            _ => vec![Quality::Master, Quality::Flac24bit, Quality::Flac, Quality::High, Quality::Standard],
        };
        MediaItem {
            id: "0".into(),
            title: title.into(),
            artist: artist.into(),
            album: String::new(),
            duration: None,
            cover_url: None,
            media_type: MediaType::Audio,
            qualities,
        }
    }

    #[test]
    fn test_exact_title_match() {
        let item = make_item("晴天", "周杰伦", 3);
        let score = score_result(&item, "晴天");
        assert!(score > 0.4, "exact title should score well: {score}");
    }

    #[test]
    fn test_fuzzy_contains() {
        let score = fuzzy_match("周杰伦的晴天", "晴天");
        assert!(score > 0.7);
    }

    #[test]
    fn test_fuzzy_exact() {
        assert!((fuzzy_match("hello", "hello") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rank_results() {
        let mut items = vec![
            make_item("无关歌曲", "未知", 1),
            make_item("晴天", "周杰伦", 4),
            make_item("雨天", "周杰伦", 2),
        ];
        rank_results(&mut items, "晴天");
        assert_eq!(items[0].title, "晴天");
    }

    #[test]
    fn test_quality_bonus() {
        let low = make_item("test", "a", 1);
        let high = make_item("test", "a", 5);
        let sl = score_result(&low, "test");
        let sh = score_result(&high, "test");
        assert!(sh > sl);
    }
}
