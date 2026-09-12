//! NT-WORLD NLP: 关键词提取
//!
//! 基于TF-IDF和TextRank的关键词提取算法

use std::collections::HashMap;

/// 关键词提取器
pub struct _KeywordExtractor;

/// 关键词及其分数
#[derive(Debug, Clone)]
pub struct Keyword {
    pub word: String,
    pub score: f64,
}

impl _KeywordExtractor {
    /// 基于TF的关键词提取 (简化版)
    pub fn _extract_by_tf(text: &str, top_k: usize) -> Vec<Keyword> {
        // 中文分词 (简单按字符分割)
        let words = Self::tokenize(text);
        let total_words = words.len() as f64;

        // 计算词频
        let mut tf: HashMap<String, usize> = HashMap::new();
        for word in &words {
            *tf.entry(word.clone()).or_insert(0) += 1;
        }

        // 转换为TF分数
        let mut keywords: Vec<Keyword> = tf.into_iter()
            .map(|(word, count)| Keyword {
                word,
                score: count as f64 / total_words,
            })
            .collect();

        // 按分数排序
        keywords.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        keywords.truncate(top_k);
        keywords
    }

    /// 基于TextRank的关键词提取
    pub fn _extract_by_textrank(text: &str, top_k: usize) -> Vec<Keyword> {
        let words = Self::tokenize(text);
        if words.is_empty() {
            return vec![];
        }

        // 构建共现矩阵
        let window_size = 5;
        let mut cooccurrence: HashMap<(String, String), usize> = HashMap::new();

        for i in 0..words.len() {
            for j in (i + 1)..std::cmp::min(i + window_size, words.len()) {
                let pair = if words[i] < words[j] {
                    (words[i].clone(), words[j].clone())
                } else {
                    (words[j].clone(), words[i].clone())
                };
                *cooccurrence.entry(pair).or_insert(0) += 1;
            }
        }

        // TextRank迭代
        let mut scores: HashMap<String, f64> = HashMap::new();
        for word in &words {
            scores.entry(word.clone()).or_insert(1.0);
        }

        let damping = 0.85;
        let iterations = 30;

        for _ in 0..iterations {
            let mut new_scores = HashMap::new();

            for (word, _score) in &scores {
                let mut rank = 0.0;

                for ((w1, w2), &count) in &cooccurrence {
                    if w1 == word || w2 == word {
                        let other = if w1 == word { w2 } else { w1 };
                        if let Some(&other_score) = scores.get(other) {
                            // 计算出度
                            let out_degree: usize = cooccurrence.iter()
                                .filter(|((a, b), _)| a == other || b == other)
                                .map(|(_, &c)| c)
                                .sum();

                            if out_degree > 0 {
                                rank += (count as f64 / out_degree as f64) * other_score;
                            }
                        }
                    }
                }

                new_scores.insert(word.clone(), (1.0 - damping) + damping * rank);
            }

            scores = new_scores;
        }

        // 转换为关键词
        let mut keywords: Vec<Keyword> = scores.into_iter()
            .map(|(word, score)| Keyword { word, score })
            .collect();

        keywords.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        keywords.truncate(top_k);
        keywords
    }

    /// 简单分词 (中文按字/词，英文按空格)
    fn tokenize(text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_english = String::new();

        for c in text.chars() {
            if c.is_ascii_alphabetic() {
                current_english.push(c);
            } else {
                if !current_english.is_empty() {
                    tokens.push(current_english.to_lowercase());
                    current_english.clear();
                }

                // 中文字符
                if c as u32 >= 0x4E00 && c as u32 <= 0x9FFF {
                    tokens.push(c.to_string());
                }
            }
        }

        if !current_english.is_empty() {
            tokens.push(current_english.to_lowercase());
        }

        // 过滤停用词
        let stop_words: std::collections::HashSet<&str> = [
            "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个",
            "上", "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好",
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "shall", "can", "need", "dare", "ought",
            "used", "to", "of", "in", "for", "on", "with", "at", "by", "from",
            "as", "into", "through", "during", "before", "after", "above", "below",
            "between", "out", "off", "over", "under", "again", "further", "then",
            "once", "here", "there", "when", "where", "why", "how", "all", "both",
            "each", "few", "more", "most", "other", "some", "such", "no", "nor",
            "not", "only", "own", "same", "so", "than", "too", "very", "s", "t",
            "and", "but", "or", "if", "while", "that", "this", "it",
        ].iter().cloned().collect();

        tokens.into_iter()
            .filter(|t| !stop_words.contains(t.as_str()) && t.len() > 1)
            .collect()
    }

    /// 提取短语 (连续的关键词)
    pub fn _extract_phrases(text: &str, min_length: usize, max_length: usize) -> Vec<String> {
        let tokens = Self::tokenize(text);
        let mut phrases = Vec::new();

        for length in min_length..=max_length {
            for window in tokens.windows(length) {
                let phrase: String = window.join("");
                if phrase.len() >= min_length {
                    phrases.push(phrase);
                }
            }
        }

        phrases
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_keywords_tf() {
        let text = "自然语言处理是人工智能的重要方向 自然语言处理包括分词 词性标注 命名实体识别";
        let keywords = _KeywordExtractor::_extract_by_tf(text, 5);
        assert!(!keywords.is_empty());
        assert!(keywords[0].score >= keywords[1].score);
    }

    #[test]
    fn extract_keywords_textrank() {
        let text = "自然语言处理是人工智能的重要方向 自然语言处理包括分词 词性标注 命名实体识别";
        let keywords = _KeywordExtractor::_extract_by_textrank(text, 5);
        assert!(!keywords.is_empty());
    }

    #[test]
    fn tokenize_test() {
        let tokens = _KeywordExtractor::tokenize("Hello 你好 World 世界");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"你".to_string()));
    }
}
