//! Unicode script detection for JEV model routing
//!
//! Ported from Laya's lang.py — identifies dominant script in text to route
//! between English and multilingual checkpoints.

use std::collections::HashMap;

/// Unicode script ranges that the English (ModernBERT-large) checkpoint cannot read
const SCRIPT_RANGES: &[(&str, &[(u32, u32)])] = &[
    ("greek", &[(0x0370, 0x03FF), (0x1F00, 0x1FFF)]),
    ("cyrillic", &[(0x0400, 0x052F), (0x2DE0, 0x2DFF), (0xA640, 0xA69F)]),
    ("armenian", &[(0x0530, 0x058F)]),
    ("hebrew", &[(0x0590, 0x05FF)]),
    ("arabic", &[(0x0600, 0x06FF), (0x0750, 0x077F), (0x08A0, 0x08FF), (0xFB50, 0xFDFF), (0xFE70, 0xFEFF)]),
    ("devanagari", &[(0x0900, 0x097F), (0xA8E0, 0xA8FF)]),
    ("bengali", &[(0x0980, 0x09FF)]),
    ("gurmukhi", &[(0x0A00, 0x0A7F)]),
    ("gujarati", &[(0x0A80, 0x0AFF)]),
    ("oriya", &[(0x0B00, 0x0B7F)]),
    ("tamil", &[(0x0B80, 0x0BFF)]),
    ("telugu", &[(0x0C00, 0x0C7F)]),
    ("kannada", &[(0x0C80, 0x0CFF)]),
    ("malayalam", &[(0x0D00, 0x0D7F)]),
    ("sinhala", &[(0x0D80, 0x0DFF)]),
    ("thai", &[(0x0E00, 0x0E7F)]),
    ("lao", &[(0x0E80, 0x0EFF)]),
    ("tibetan", &[(0x0F00, 0x0FFF)]),
    ("myanmar", &[(0x1000, 0x109F)]),
    ("georgian", &[(0x10A0, 0x10FF)]),
    ("ethiopic", &[(0x1200, 0x137F)]),
    ("khmer", &[(0x1780, 0x17FF)]),
    ("hangul", &[(0x1100, 0x11FF), (0x3130, 0x318F), (0xAC00, 0xD7AF)]),
    ("kana", &[(0x3040, 0x309F), (0x30A0, 0x30FF), (0x31F0, 0x31FF)]),
    ("han", &[(0x3400, 0x4DBF), (0x4E00, 0x9FFF), (0xF900, 0xFAFF)]),
];

/// Function words for Latin-script language detection
const STOP_WORDS: &[(&str, &[&str])] = &[
    ("en", &["the", "and", "is", "are", "was", "were", "to", "of", "in", "for", "with", "that",
             "this", "it", "you", "have", "has", "not", "but", "on", "at", "be", "as", "from",
             "will", "can", "would", "there", "their", "what", "which", "please", "we", "i"]),
    ("fr", &["le", "la", "les", "des", "une", "est", "pour", "dans", "que", "qui", "avec", "sur",
             "pas", "plus", "nous", "vous", "être", "cette", "mais", "sont", "ont", "aux", "ce"]),
    ("de", &["der", "die", "das", "und", "ist", "ein", "eine", "den", "dem", "nicht", "mit", "für",
             "auf", "von", "zu", "sich", "auch", "werden", "wurde", "haben", "sind", "oder", "aber"]),
    ("es", &["el", "los", "las", "que", "por", "con", "para", "una", "es", "se", "del", "como",
             "pero", "son", "está", "este", "esta", "todo", "más", "muy", "hay", "sus"]),
    ("pt", &["os", "as", "que", "em", "um", "uma", "para", "com", "não", "é", "se", "do", "da",
             "dos", "das", "mas", "são", "está", "este", "esta", "muito", "pelo", "pela"]),
    ("it", &["il", "lo", "gli", "che", "di", "per", "con", "non", "è", "si", "del", "della", "sono",
             "questo", "questa", "anche", "come", "più", "sono", "nella", "alla"]),
    ("nl", &["het", "een", "van", "is", "op", "te", "dat", "niet", "met", "voor", "zijn", "aan",
             "door", "maar", "ook", "worden", "deze", "naar", "wordt"]),
];

/// Non-English diacritics
const NON_EN_DIACRITICS: &[char] = &[
    'à','â','ä','ã','á','å','ç','é','è','ê','ë','í','ì','î','ï','ñ',
    'ó','ò','ô','ö','õ','ø','ú','ù','û','ü','ý','ÿ','ß','æ','œ','đ',
    'ł','ş','ţ','ğ','ı','å','ä','ö',
];

/// Detection result
#[derive(Debug, Clone)]
pub struct ScriptDetection {
    pub script: String,
    pub language: Option<String>,
    pub is_english: bool,
    pub non_latin_fraction: f64,
}

/// Detect the dominant Unicode script in text
pub fn detect_script(text: &str) -> String {
    let mut counts: HashMap<String, u32> = HashMap::new();
    let mut latin = 0u32;

    for ch in text.chars() {
        if !ch.is_alphabetic() {
            continue;
        }
        let cp = ch as u32;
        if cp < 0x0250 || (0x1E00..=0x1EFF).contains(&cp) {
            latin += 1;
            continue;
        }
        for (name, ranges) in SCRIPT_RANGES {
            if ranges.iter().any(|&(lo, hi)| cp >= lo && cp <= hi) {
                *counts.entry(name.to_string()).or_insert(0) += 1;
                break;
            }
        }
    }
    counts.insert("latin".to_string(), latin);

    let total: u32 = counts.values().sum();
    if total == 0 {
        return "unknown".to_string();
    }

    counts.iter()
        .max_by_key(|(_, &count)| count)
        .map(|(name, _)| name.clone())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Compute fraction of non-Latin characters
pub fn non_latin_fraction(text: &str) -> f64 {
    let mut latin = 0u32;
    let mut total = 0u32;

    for ch in text.chars() {
        if !ch.is_alphabetic() {
            continue;
        }
        total += 1;
        let cp = ch as u32;
        if cp < 0x0250 || (0x1E00..=0x1EFF).contains(&cp) {
            latin += 1;
        }
    }

    if total == 0 {
        return 0.0;
    }

    1.0 - (latin as f64 / total as f64)
}

/// Guess Latin-script language from function words
pub fn guess_latin_language(text: &str) -> Option<String> {
    let words: Vec<String> = text.split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();

    if words.len() < 4 {
        return None;
    }

    let mut scores: HashMap<String, u32> = HashMap::new();
    for (lang, sw_list) in STOP_WORDS {
        let count = words.iter().filter(|w| sw_list.contains(&w.as_str())).count() as u32;
        if count > 0 {
            scores.insert(lang.to_string(), count);
        }
    }

    let diac_count = text.to_lowercase().chars()
        .filter(|c| NON_EN_DIACRITICS.contains(c))
        .count() as f64;
    let diac_rate = diac_count / text.len().max(1) as f64;

    let en_score = scores.get("en").copied().unwrap_or(0);
    let best = scores.iter()
        .filter(|(lg, _)| lg.as_str() != "en")
        .max_by_key(|(_, &s)| s)
        .map(|(lg, &s)| (lg.clone(), s));

    if let Some((ref best_lg, best_score)) = best {
        if best_score == 0 && diac_rate < 0.02 {
            return if en_score > 0 { Some("en".into()) } else { None };
        }
        if best_score >= 2.max(en_score + 2) {
            return Some(best_lg.clone());
        }
        if diac_rate >= 0.04 && best_score >= en_score {
            return Some(best_lg.clone());
        }
    }

    if en_score > 0 { Some("en".into()) } else { None }
}

/// Full detection: script, language, is_english, non_latin_fraction
pub fn analyse(text: &str) -> ScriptDetection {
    let script = detect_script(text);
    let non_latin = non_latin_fraction(text);

    if script == "unknown" {
        return ScriptDetection {
            script: "unknown".into(),
            language: None,
            is_english: true,
            non_latin_fraction: 0.0,
        };
    }

    if script != "latin" {
        return ScriptDetection {
            script,
            language: None,
            is_english: false,
            non_latin_fraction: non_latin,
        };
    }

    let lang = guess_latin_language(text);
    let is_english = matches!(lang.as_deref(), None | Some("en"));

    ScriptDetection {
        script: "latin".into(),
        language: lang,
        is_english,
        non_latin_fraction: non_latin,
    }
}

/// Quick check: is this text English enough for the English checkpoint?
pub fn is_english(text: &str) -> bool {
    analyse(text).is_english
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_english() {
        let det = analyse("Hello, this is a test message");
        assert_eq!(det.script, "latin");
        assert!(det.is_english);
    }

    #[test]
    fn test_detect_chinese() {
        let det = analyse("这是一个测试消息");
        assert_eq!(det.script, "han");
        assert!(!det.is_english);
    }

    #[test]
    fn test_detect_hindi() {
        let det = analyse("यह एक परीक्षा संदेश है");
        assert_eq!(det.script, "devanagari");
        assert!(!det.is_english);
    }

    #[test]
    fn test_detect_french() {
        let det = analyse("Le chat est sur la table, nous allons au marché");
        assert_eq!(det.script, "latin");
        assert!(!det.is_english); // French detected
    }

    #[test]
    fn test_non_latin_fraction() {
        // Char-based over alphabetic chars only: "Hello 你好" → 2/7.
        let f = non_latin_fraction("Hello 你好");
        assert!((f - 2.0 / 7.0).abs() < 1e-9, "got {}", f);

        let g = non_latin_fraction("Hello 你好世界");
        assert!(g > 0.3, "got {}", g);
    }

    #[test]
    fn test_unknown_script() {
        let det = analyse("12345 !@#$%");
        assert_eq!(det.script, "unknown");
    }
}
