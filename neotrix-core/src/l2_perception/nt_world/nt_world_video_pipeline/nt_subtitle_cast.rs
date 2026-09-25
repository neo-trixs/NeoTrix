//! Subtitle + cast segment: subtitle engine + device discovery / cast targets.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum _CastProtocol {
    DLNA,
    Chromecast,
    AirPlay,
}

#[derive(Debug, Clone)]
pub struct _CastTarget {
    pub name: String,
    pub protocol: _CastProtocol,
    pub address: String,
    pub port: u16,
    pub supports_transcoding: bool,
}

pub struct _SubtitleEngine;

impl _SubtitleEngine {
    pub fn generate(text: &str, _language: &str) -> Vec<SubtitleEntry> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let chunk_size = 10.max(words.len() / 5);
        let mut entries = Vec::new();
        let mut start_ms = 0u64;
        for chunk in words.chunks(chunk_size) {
            let duration_ms = (chunk.len() as u64) * 200;
            entries.push(SubtitleEntry {
                index: entries.len() + 1,
                start_ms,
                end_ms: start_ms + duration_ms,
                text: chunk.join(" "),
            });
            start_ms += duration_ms;
        }
        entries
    }
}

#[derive(Debug, Clone)]
pub struct SubtitleEntry {
    pub index: usize,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

pub struct _DeviceDiscovery;

impl _DeviceDiscovery {
    pub fn scan(protocol: _CastProtocol) -> Vec<_CastTarget> {
        match protocol {
            _CastProtocol::DLNA => vec![_CastTarget {
                name: "Living Room TV (DLNA)".into(),
                protocol: _CastProtocol::DLNA,
                address: "192.168.1.100".into(),
                port: 8200,
                supports_transcoding: false,
            }],
            _CastProtocol::Chromecast => vec![_CastTarget {
                name: "Living Room TV (Chromecast)".into(),
                protocol: _CastProtocol::Chromecast,
                address: "192.168.1.101".into(),
                port: 8009,
                supports_transcoding: true,
            }],
            _CastProtocol::AirPlay => vec![],
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtitle_generation() {
        let entries = _SubtitleEngine::generate("Hello world this is a test of subtitle generation from whisper", "eng");
        assert!(!entries.is_empty());
        assert_eq!(entries[0].index, 1);
        assert!(entries[0].end_ms > entries[0].start_ms);
    }

    #[test]
    fn test_device_discovery() {
        let dlna = _DeviceDiscovery::scan(_CastProtocol::DLNA);
        assert!(!dlna.is_empty());
        assert_eq!(dlna[0].protocol, _CastProtocol::DLNA);
        let airplay = _DeviceDiscovery::scan(_CastProtocol::AirPlay);
        assert!(airplay.is_empty());
    }
}
