use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchIntent {
    Song {
        title: Option<String>,
        artist: Option<String>,
    },
    Artist {
        name: String,
    },
    Album {
        name: String,
        artist: Option<String>,
    },
    Lyrics {
        keyword: String,
    },
}

pub fn classify_intent(query: &str) -> SearchIntent {
    let q = query.trim().to_lowercase();

    if q.contains("歌词") || q.contains("lyric") {
        let keyword = q.replace("歌词", "").replace("lyric", "").trim().to_string();
        return SearchIntent::Lyrics { keyword };
    }

    if q.contains("专辑") || q.contains("album") {
        let cleaned = q.replace("专辑", "").replace("album", "").trim().to_string();
        let (name, artist) = split_name_artist(&cleaned);
        return SearchIntent::Album { name, artist };
    }

    if q.contains("歌手") || q.contains("singer") || q.contains("artist") {
        let cleaned = q
            .replace("歌手", "")
            .replace("singer", "")
            .replace("artist", "")
            .replace("名", "")
            .replace("的", "")
            .trim()
            .to_string();
        return SearchIntent::Artist { name: cleaned };
    }

    let (title, artist) = split_name_artist(&q);
    SearchIntent::Song { title, artist }
}

fn split_name_artist(text: &str) -> (Option<String>, Option<String>) {
    let delimiters = [" - ", " – ", "-", "—", "「", "」"];
    for delim in delimiters {
        if let Some((left, right)) = text.split_once(delim) {
            let left = left.trim().to_string();
            let right = right.trim().to_string();
            if !left.is_empty() && !right.is_empty() {
                return (Some(left), Some(right));
            }
            if !left.is_empty() {
                return (Some(left), None);
            }
            if !right.is_empty() {
                return (None, Some(right));
            }
        }
    }
    if text.is_empty() {
        (None, None)
    } else {
        (Some(text.to_string()), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_lyrics() {
        let intent = classify_intent("周杰伦 歌词");
        assert_eq!(intent, SearchIntent::Lyrics { keyword: "周杰伦".into() });
    }

    #[test]
    fn test_classify_album() {
        let intent = classify_intent("范特西 专辑 周杰伦");
        assert!(matches!(intent, SearchIntent::Album { .. }));
    }

    #[test]
    fn test_classify_artist() {
        let intent = classify_intent("歌手 周杰伦");
        assert_eq!(intent, SearchIntent::Artist { name: "周杰伦".into() });
    }

    #[test]
    fn test_classify_song_with_delimiter() {
        let intent = classify_intent("晴天 - 周杰伦");
        match intent {
            SearchIntent::Song { title, artist } => {
                assert_eq!(title, Some("晴天".into()));
                assert_eq!(artist, Some("周杰伦".into()));
            }
            _ => panic!("expected Song"),
        }
    }

    #[test]
    fn test_classify_song_simple() {
        let intent = classify_intent("晴天");
        assert_eq!(intent, SearchIntent::Song { title: Some("晴天".into()), artist: None });
    }
}
