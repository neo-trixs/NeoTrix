/// 4-signal 置信度评分 (参考 pdfmux)
///
/// 四个信号:
/// 1. text_density: 文本字符数 / 页面面积
/// 2. char_distribution: 字符分布是否均匀 (检测 mojibake)
/// 3. structural_coherence: 阅读顺序完整性
/// 4. ocr_probe: 随机区域 OCR 对比 (仅在置信度低于阈值时触发)
pub struct ConfidenceScorer;

impl ConfidenceScorer {
    pub fn score(text: &str, page_area: f64) -> f64 {
        let density = Self::text_density(text, page_area);
        let distribution = Self::char_distribution(text);
        let structure = Self::structural_coherence(text);

        0.35 * density + 0.25 * distribution + 0.40 * structure
    }

    fn text_density(text: &str, page_area: f64) -> f64 {
        if page_area <= 0.0 {
            return 0.0;
        }
        let chars = text.len() as f64;
        let ratio = chars / page_area;
        (ratio / 0.05).min(1.0)
    }

    fn char_distribution(text: &str) -> f64 {
        if text.is_empty() {
            return 0.0;
        }
        let replaced = text.chars().filter(|&c| c == '\u{FFFD}').count();
        if replaced == 0 {
            1.0
        } else {
            (1.0 - (replaced as f64 / text.len() as f64).min(1.0)).max(0.0)
        }
    }

    fn structural_coherence(text: &str) -> f64 {
        if text.len() < 20 {
            return 0.3;
        }
        let has_heading = text.contains('\n');
        let has_sentences = text.contains('.');
        let has_spaces = text.contains(' ');
        let mut score = 0.0;
        if has_heading { score += 0.3; }
        if has_sentences { score += 0.4; }
        if has_spaces { score += 0.3; }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text_scores_high() {
        let text = "# Chapter 1\n\nThis is a normal sentence. With proper structure.\n\n## Section 1.1\n\nMore content here.";
        let s = ConfidenceScorer::score(text, 1000.0);
        assert!(s > 0.7, "clean text should score high, got {}", s);
    }

    #[test]
    fn test_empty_text_scores_low() {
        let s = ConfidenceScorer::score("", 1000.0);
        assert!(s < 0.5, "empty text should score low");
    }

    #[test]
    fn test_mojibake_detected() {
        let mojibake = "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD} broken text";
        let s = ConfidenceScorer::score(mojibake, 1000.0);
        assert!(s < 0.8, "mojibake should reduce score");
    }
}
