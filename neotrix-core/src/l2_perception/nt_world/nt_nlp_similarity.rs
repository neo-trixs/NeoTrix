//! NT-WORLD NLP: 文本相似度计算
//!
//! 实现多种文本相似度算法：
//! - 余弦相似度 (基于词频向量)
//! - 编辑距离 (Levenshtein)
//! - Jaccard相似度 (集合相似度)
//! - BM25相似度

use std::collections::HashMap;

/// 文本相似度计算器
pub struct _TextSimilarity;

impl _TextSimilarity {
    /// 余弦相似度 (基于字符级向量)
    pub fn cosine_similarity(text1: &str, text2: &str) -> f64 {
        if text1.is_empty() || text2.is_empty() {
            return 0.0;
        }

        let mut vec1: HashMap<char, f64> = HashMap::new();
        let mut vec2: HashMap<char, f64> = HashMap::new();

        // 构建字符频率向量
        for c in text1.chars() {
            *vec1.entry(c).or_insert(0.0) += 1.0;
        }
        for c in text2.chars() {
            *vec2.entry(c).or_insert(0.0) += 1.0;
        }

        // 计算点积
        let mut dot_product = 0.0;
        for (c, &count) in &vec1 {
            if let Some(&count2) = vec2.get(c) {
                dot_product += count * count2;
            }
        }

        // 计算模长
        let norm1: f64 = vec1.values().map(|x| x * x).sum::<f64>().sqrt();
        let norm2: f64 = vec2.values().map(|x| x * x).sum::<f64>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }

    /// 编辑距离 (Levenshtein距离)
    pub fn _edit_distance(text1: &str, text2: &str) -> usize {
        let len1 = text1.len();
        let len2 = text2.len();
        let chars1: Vec<char> = text1.chars().collect();
        let chars2: Vec<char> = text2.chars().collect();

        let mut dp = vec![vec![0usize; len2 + 1]; len1 + 1];

        // 初始化
        for i in 0..=len1 {
            dp[i][0] = i;
        }
        for j in 0..=len2 {
            dp[0][j] = j;
        }

        // 填充DP表
        for i in 1..=len1 {
            for j in 1..=len2 {
                if chars1[i - 1] == chars2[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1];
                } else {
                    dp[i][j] = 1 + dp[i - 1][j - 1].min(dp[i][j - 1]).min(dp[i - 1][j]);
                }
            }
        }

        dp[len1][len2]
    }

    /// 编辑距离相似度 (归一化到0-1)
    pub fn _edit_distance_similarity(text1: &str, text2: &str) -> f64 {
        let max_len = text1.chars().count().max(text2.chars().count());
        if max_len == 0 {
            return 1.0;
        }
        let distance = Self::_edit_distance(text1, text2);
        1.0 - (distance as f64 / max_len as f64)
    }

    /// Jaccard相似度 (基于字符集)
    pub fn _jaccard_similarity(text1: &str, text2: &str) -> f64 {
        let set1: std::collections::HashSet<char> = text1.chars().collect();
        let set2: std::collections::HashSet<char> = text2.chars().collect();

        if set1.is_empty() && set2.is_empty() {
            return 1.0;
        }

        let intersection: usize = set1.intersection(&set2).count();
        let union: usize = set1.union(&set2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Jaccard相似度 (基于词组)
    pub fn _jaccard_similarity_words(text1: &str, text2: &str) -> f64 {
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

        if words1.is_empty() && words2.is_empty() {
            return 1.0;
        }

        let intersection: usize = words1.intersection(&words2).count();
        let union: usize = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// SimHash (局部敏感哈希，用于近似去重)
    pub fn _simhash(text: &str, hash_bits: usize) -> u64 {
        let mut v = vec![0i64; hash_bits];
        let words: Vec<&str> = text.split_whitespace().collect();

        for word in &words {
            let hash = Self::simple_hash(word);
            for i in 0..hash_bits {
                if hash & (1 << i) != 0 {
                    v[i] += 1;
                } else {
                    v[i] -= 1;
                }
            }
        }

        let mut fingerprint = 0u64;
        for i in 0..hash_bits {
            if v[i] > 0 {
                fingerprint |= 1 << i;
            }
        }
        fingerprint
    }

    /// SimHash距离 (汉明距离)
    pub fn _simhash_distance(hash1: u64, hash2: u64) -> u32 {
        (hash1 ^ hash2).count_ones()
    }

    /// 简单哈希函数
    fn simple_hash(s: &str) -> u64 {
        let mut hash: u64 = 0;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }

    /// BM25相似度 (基于词频)
    pub fn _bm25_score(query: &str, document: &str, k1: f64, b: f64) -> f64 {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let doc_words: Vec<&str> = document.split_whitespace().collect();
        let doc_len = doc_words.len() as f64;

        // 计算文档词频
        let mut doc_tf: HashMap<&str, usize> = HashMap::new();
        for word in &doc_words {
            *doc_tf.entry(word).or_insert(0) += 1;
        }

        let avg_doc_len = doc_len; // 单文档场景
        let mut score = 0.0;

        for query_word in &query_words {
            if let Some(&tf) = doc_tf.get(query_word) {
                let tf = tf as f64;
                let numerator = tf * (k1 + 1.0);
                let denominator = tf + k1 * (1.0 - b + b * doc_len / avg_doc_len);
                score += numerator / denominator;
            }
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_similarity_test() {
        let s1 = "hello world";
        let s2 = "hello rust";
        let sim = _TextSimilarity::cosine_similarity(s1, s2);
        assert!(sim > 0.5); // 共享 "hello" 和空格
    }

    #[test]
    fn edit_distance_test() {
        assert_eq!(_TextSimilarity::_edit_distance("kitten", "sitting"), 3);
        assert_eq!(_TextSimilarity::_edit_distance("hello", "hello"), 0);
        assert_eq!(_TextSimilarity::_edit_distance("", "abc"), 3);
    }

    #[test]
    fn jaccard_similarity_test() {
        let sim = _TextSimilarity::_jaccard_similarity("abc", "abd");
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn simhash_test() {
        let h1 = _TextSimilarity::_simhash("hello world", 64);
        let h2 = _TextSimilarity::_simhash("hello world", 64);
        assert_eq!(_TextSimilarity::_simhash_distance(h1, h2), 0);

        let h3 = _TextSimilarity::_simhash("completely different", 64);
        assert!(_TextSimilarity::_simhash_distance(h1, h3) > 10);
    }

    #[test]
    fn bm25_test() {
        let score = _TextSimilarity::_bm25_score("hello", "hello world hello rust", 1.5, 0.75);
        assert!(score > 0.0);
    }
}
