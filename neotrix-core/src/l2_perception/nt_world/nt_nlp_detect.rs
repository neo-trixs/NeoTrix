//! NT-WORLD NLP: 语言检测
//!
//! 基于字符统计特征检测文本语言

/// 语言类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    Chinese,
    English,
    Japanese,
    Korean,
    French,
    German,
    Spanish,
    Russian,
    Arabic,
    Unknown,
}

/// 语言检测器
pub struct LanguageDetector;

impl LanguageDetector {
    /// 检测文本语言
    pub fn detect(text: &str) -> Language {
        if text.is_empty() {
            return Language::Unknown;
        }

        let total_chars = text.chars().count();
        let mut chinese_count = 0;
        let mut japanese_count = 0;
        let mut korean_count = 0;
        let mut latin_count = 0;
        let mut cyrillic_count = 0;
        let mut arabic_count = 0;

        for c in text.chars() {
            let code = c as u32;
            match code {
                // CJK统一汉字
                0x4E00..=0x9FFF => chinese_count += 1,
                // 日文平假名
                0x3040..=0x309F => japanese_count += 1,
                // 日文片假名
                0x30A0..=0x30FF => japanese_count += 1,
                // 韩文字母
                0xAC00..=0xD7AF => korean_count += 1,
                0x1100..=0x11FF => korean_count += 1,
                // 拉丁字母
                0x0041..=0x005A | 0x0061..=0x007A => latin_count += 1,
                // 西里尔字母
                0x0400..=0x04FF => cyrillic_count += 1,
                // 阿拉伯字母
                0x0600..=0x06FF => arabic_count += 1,
                _ => {}
            }
        }

        // 计算比例
        let chinese_ratio = chinese_count as f64 / total_chars as f64;
        let japanese_ratio = japanese_count as f64 / total_chars as f64;
        let korean_ratio = korean_count as f64 / total_chars as f64;
        let latin_ratio = latin_count as f64 / total_chars as f64;
        let cyrillic_ratio = cyrillic_count as f64 / total_chars as f64;
        let arabic_ratio = arabic_count as f64 / total_chars as f64;

        // 决策逻辑
        if chinese_ratio > 0.3 {
            Language::Chinese
        } else if japanese_ratio > 0.2 {
            Language::Japanese
        } else if korean_ratio > 0.2 {
            Language::Korean
        } else if latin_ratio > 0.5 {
            // 进一步区分欧洲语言
            Self::detect_latin_language(text)
        } else if cyrillic_ratio > 0.3 {
            Language::Russian
        } else if arabic_ratio > 0.3 {
            Language::Arabic
        } else {
            Language::Unknown
        }
    }

    /// 检测拉丁字母语言
    fn detect_latin_language(text: &str) -> Language {
        let text_lower = text.to_lowercase();

        // 简单的特征词检测
        let french_features = ["le ", "la ", "les ", "de ", "du ", "des ", "un ", "une ", "et ", "est "];
        let german_features = ["der ", "die ", "das ", "ein ", "eine ", "ist ", "und ", "nicht ", "mit ", "auf "];
        let spanish_features = ["el ", "la ", "los ", "las ", "un ", "una ", "es ", "y ", "de ", "en "];

        let french_score = french_features.iter().filter(|f| text_lower.contains(*f)).count();
        let german_score = german_features.iter().filter(|f| text_lower.contains(*f)).count();
        let spanish_score = spanish_features.iter().filter(|f| text_lower.contains(*f)).count();

        let max_score = french_score.max(german_score).max(spanish_score);

        if max_score == 0 {
            Language::English
        } else if french_score == max_score {
            Language::French
        } else if german_score == max_score {
            Language::German
        } else {
            Language::Spanish
        }
    }

    /// 检测是否包含中文
    pub fn contains_chinese(text: &str) -> bool {
        text.chars().any(|c| {
            let code = c as u32;
            matches!(code, 0x4E00..=0x9FFF)
        })
    }

    /// 检测是否混合语言
    pub fn is_mixed_language(text: &str) -> bool {
        let mut has_chinese = false;
        let mut has_latin = false;

        for c in text.chars() {
            let code = c as u32;
            match code {
                0x4E00..=0x9FFF => has_chinese = true,
                0x0041..=0x005A | 0x0061..=0x007A => has_latin = true,
                _ => {}
            }
        }

        has_chinese && has_latin
    }

    /// 提取中文文本
    pub fn extract_chinese(text: &str) -> String {
        text.chars()
            .filter(|c| {
                let code = *c as u32;
                matches!(code, 0x4E00..=0x9FFF | 0x3000..=0x303F | 0xFF00..=0xFFEF)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_chinese() {
        assert_eq!(LanguageDetector::detect("你好世界"), Language::Chinese);
        assert_eq!(LanguageDetector::detect("这是中文文本"), Language::Chinese);
    }

    #[test]
    fn detect_english() {
        assert_eq!(LanguageDetector::detect("Hello World"), Language::English);
        assert_eq!(LanguageDetector::detect("This is English text"), Language::English);
    }

    #[test]
    fn detect_mixed() {
        assert!(LanguageDetector::is_mixed_language("Hello 你好"));
        assert!(!LanguageDetector::is_mixed_language("Hello World"));
    }

    #[test]
    fn extract_chinese_test() {
        let text = "Hello 你好 World 世界";
        let chinese = LanguageDetector::extract_chinese(text);
        assert_eq!(chinese, "你好世界");
    }

    #[test]
    fn contains_chinese_test() {
        assert!(LanguageDetector::contains_chinese("Hello 你好"));
        assert!(!LanguageDetector::contains_chinese("Hello World"));
    }
}
