//! NT-WORLD NLP: 情感分析
//!
//! 基于词典的情感分析器

use std::collections::HashMap;

/// 情感极性
#[derive(Debug, Clone, PartialEq)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

/// 情感分数
#[derive(Debug, Clone)]
pub struct _SentimentScore {
    pub polarity: Sentiment,
    pub positive_score: f64,
    pub negative_score: f64,
    pub objective_score: f64,
}

/// 情感分析器
pub struct _SentimentAnalyzer {
    /// 正面词典
    positive_words: std::collections::HashSet<String>,
    /// 负面词典
    negative_words: std::collections::HashSet<String>,
    /// 否定词
    negation_words: std::collections::HashSet<String>,
    /// 程度词
    degree_words: HashMap<String, f64>,
}

impl _SentimentAnalyzer {
    /// 创建新的情感分析器
    pub fn new() -> Self {
        let mut positive_words = std::collections::HashSet::new();
        let mut negative_words = std::collections::HashSet::new();
        let mut negation_words = std::collections::HashSet::new();
        let mut degree_words = HashMap::new();

        // 正面词
        for word in ["好", "优秀", "喜欢", "快乐", "成功", "美丽", "优秀", "出色", "卓越", "精彩",
                      "happy", "good", "great", "excellent", "wonderful", "beautiful", "amazing",
                      "love", "best", "perfect", "fantastic", "brilliant", "outstanding"] {
            positive_words.insert(word.to_string());
        }

        // 负面词
        for word in ["坏", "差", "讨厌", "悲伤", "失败", "丑陋", "糟糕", "痛苦", "困难", "问题",
                      "bad", "poor", "hate", "sad", "fail", "ugly", "terrible", "pain",
                      "problem", "error", "wrong", "worst", "horrible", "awful"] {
            negative_words.insert(word.to_string());
        }

        // 否定词
        for word in ["不", "没", "无", "非", "未", "别", "莫", "勿", "没有", "不是",
                      "no", "not", "never", "neither", "nor", "none", "nothing"] {
            negation_words.insert(word.to_string());
        }

        // 程度词
        for (word, factor) in [
            ("很", 1.5), ("非常", 2.0), ("特别", 2.0), ("极其", 2.5), ("稍微", 0.5),
            ("比较", 0.8), ("相当", 1.8), ("十分", 2.0), ("极", 2.5), ("略", 0.5),
            ("very", 1.5), ("extremely", 2.0), ("slightly", 0.5), ("quite", 1.2),
            ("rather", 0.8), ("pretty", 1.0), ("so", 1.5), ("too", 1.8),
        ] {
            degree_words.insert(word.to_string(), factor);
        }

        Self {
            positive_words,
            negative_words,
            negation_words,
            degree_words,
        }
    }

    /// 分析情感
    pub fn analyze(&self, text: &str) -> _SentimentScore {
        let words = self.tokenize(text);
        let mut positive_score = 0.0;
        let mut negative_score = 0.0;
        let mut negation_active = false;
        let mut degree_factor = 1.0;

        for word in &words {
            if self.negation_words.contains(word.as_str()) {
                negation_active = true;
                continue;
            }

            if let Some(&factor) = self.degree_words.get(word.as_str()) {
                degree_factor = factor;
                continue;
            }

            let word_lower = word.to_lowercase();

            if self.positive_words.contains(&word_lower) {
                let score = 1.0 * degree_factor;
                if negation_active {
                    negative_score += score;
                } else {
                    positive_score += score;
                }
            } else if self.negative_words.contains(&word_lower) {
                let score = 1.0 * degree_factor;
                if negation_active {
                    positive_score += score * 0.5; // 否定后减弱
                } else {
                    negative_score += score;
                }
            }

            // 重置状态
            if !self.degree_words.contains_key(word.as_str()) {
                degree_factor = 1.0;
            }
            negation_active = false;
        }

        // 计算总分
        let total = positive_score + negative_score;
        let objective_score = if total == 0.0 { 1.0 } else { 0.0 };

        let polarity = if positive_score > negative_score * 1.2 {
            Sentiment::Positive
        } else if negative_score > positive_score * 1.2 {
            Sentiment::Negative
        } else if total == 0.0 {
            Sentiment::Neutral
        } else {
            Sentiment::Mixed
        };

        _SentimentScore {
            polarity,
            positive_score,
            negative_score,
            objective_score,
        }
    }

    /// 简单分词
    fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();

        for c in text.chars() {
            if c.is_ascii_alphabetic() || c.is_ascii_digit() {
                current.push(c);
            } else {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                if !c.is_whitespace() {
                    tokens.push(c.to_string());
                }
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }
}

impl Default for _SentimentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_sentiment() {
        let analyzer = _SentimentAnalyzer::new();
        let result = analyzer.analyze("这个产品非常好用");
        assert!(result.positive_score > result.negative_score);
    }

    #[test]
    fn negative_sentiment() {
        let analyzer = _SentimentAnalyzer::new();
        let result = analyzer.analyze("这个服务太差了");
        assert!(result.negative_score > result.positive_score);
    }

    #[test]
    fn neutral_sentiment() {
        let analyzer = _SentimentAnalyzer::new();
        let result = analyzer.analyze("今天天气不错");
        assert!(result.positive_score > 0.0);
    }

    #[test]
    fn negation_test() {
        let analyzer = _SentimentAnalyzer::new();
        let result = analyzer.analyze("不好");
        assert!(result.negative_score > result.positive_score);
    }
}
