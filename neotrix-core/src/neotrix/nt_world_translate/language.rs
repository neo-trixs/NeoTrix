#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum Language {
    English,
    Chinese,
    Spanish,
    French,
    German,
    Japanese,
    Korean,
    Russian,
    Arabic,
    Portuguese,
    Italian,
    Dutch,
    Polish,
    Turkish,
    Vietnamese,
    Thai,
    Hindi,
    Bengali,
    Unknown,
}

impl Language {
    pub fn detect(text: &str) -> Language {
        let mut cn = 0usize;
        let mut ja = 0usize;
        let mut ko = 0usize;
        let mut ru = 0usize;
        let mut ar = 0usize;
        let mut th = 0usize;
        let mut latin = 0usize;

        for c in text.chars().take(200) {
            match c {
                '\u{4e00}'..='\u{9fff}' | '\u{3400}'..='\u{4dbf}' => cn += 1,
                '\u{3040}'..='\u{309f}' | '\u{30a0}'..='\u{30ff}' => ja += 1,
                '\u{ac00}'..='\u{d7af}' => ko += 1,
                '\u{0400}'..='\u{04ff}' => ru += 1,
                '\u{0600}'..='\u{06ff}' => ar += 1,
                '\u{0e00}'..='\u{0e7f}' => th += 1,
                'a'..='z' | 'A'..='Z' => latin += 1,
                _ => {}
            }
        }

        if ja > 0 {
            return Language::Japanese;
        }
        if cn > 2 {
            return Language::Chinese;
        }
        if ko > 0 {
            return Language::Korean;
        }
        if ru > latin && ru > 0 {
            return Language::Russian;
        }
        if ar > 0 {
            return Language::Arabic;
        }
        if th > 0 {
            return Language::Thai;
        }
        if latin > 0 {
            return Language::English;
        }

        Language::Unknown
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Chinese => "zh",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::Russian => "ru",
            Language::Arabic => "ar",
            Language::Portuguese => "pt",
            Language::Italian => "it",
            Language::Dutch => "nl",
            Language::Polish => "pl",
            Language::Turkish => "tr",
            Language::Vietnamese => "vi",
            Language::Thai => "th",
            Language::Hindi => "hi",
            Language::Bengali => "bn",
            Language::Unknown => "??",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Chinese => "中文",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::German => "Deutsch",
            Language::Japanese => "日本語",
            Language::Korean => "한국어",
            Language::Russian => "Русский",
            Language::Arabic => "العربية",
            Language::Portuguese => "Português",
            Language::Italian => "Italiano",
            Language::Dutch => "Nederlands",
            Language::Polish => "Polski",
            Language::Turkish => "Türkçe",
            Language::Vietnamese => "Tiếng Việt",
            Language::Thai => "ไทย",
            Language::Hindi => "हिन्दी",
            Language::Bengali => "বাংলা",
            Language::Unknown => "Unknown",
        }
    }
}

impl Language {
    pub fn language_family(&self) -> u8 {
        match self {
            Language::English | Language::German | Language::Dutch => 1,
            Language::Spanish | Language::French | Language::Portuguese | Language::Italian => 2,
            Language::Russian | Language::Polish => 3,
            Language::Japanese | Language::Korean => 4,
            Language::Chinese => 5,
            Language::Arabic => 6,
            Language::Hindi | Language::Bengali => 7,
            Language::Turkish => 8,
            Language::Vietnamese => 9,
            Language::Thai => 10,
            Language::Unknown => 0,
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "en" => Language::English,
            "zh" => Language::Chinese,
            "es" => Language::Spanish,
            "fr" => Language::French,
            "de" => Language::German,
            "ja" => Language::Japanese,
            "ko" => Language::Korean,
            "ru" => Language::Russian,
            "ar" => Language::Arabic,
            "pt" => Language::Portuguese,
            "it" => Language::Italian,
            "nl" => Language::Dutch,
            "pl" => Language::Polish,
            "tr" => Language::Turkish,
            "vi" => Language::Vietnamese,
            "th" => Language::Thai,
            "hi" => Language::Hindi,
            "bn" => Language::Bengali,
            _ => Language::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LanguageDetector;

impl LanguageDetector {
    pub fn new() -> Self {
        LanguageDetector
    }

    pub fn detect(&self, text: &str) -> Language {
        Language::detect(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_english() {
        assert_eq!(
            Language::detect("Hello world this is English"),
            Language::English
        );
    }

    #[test]
    fn test_detect_chinese() {
        assert_eq!(Language::detect("你好世界，这是中文"), Language::Chinese);
    }

    #[test]
    fn test_detect_japanese() {
        assert_eq!(Language::detect("こんにちは世界"), Language::Japanese);
    }

    #[test]
    fn test_detect_korean() {
        assert_eq!(Language::detect("안녕하세요 세계"), Language::Korean);
    }

    #[test]
    fn test_detect_russian() {
        assert_eq!(Language::detect("Привет мир"), Language::Russian);
    }

    #[test]
    fn test_code_english() {
        assert_eq!(Language::English.code(), "en");
    }

    #[test]
    fn test_code_chinese() {
        assert_eq!(Language::Chinese.code(), "zh");
    }

    #[test]
    fn test_name_contains_native() {
        assert_eq!(Language::Chinese.name(), "中文");
        assert_eq!(Language::Japanese.name(), "日本語");
    }

    #[test]
    fn test_detect_empty() {
        assert_eq!(Language::detect(""), Language::Unknown);
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(Language::detect("12345!@#$%"), Language::Unknown);
    }

    #[test]
    fn test_detect_mixed_japanese_chinese() {
        assert_eq!(Language::detect("日本語と漢字"), Language::Japanese);
    }
}
