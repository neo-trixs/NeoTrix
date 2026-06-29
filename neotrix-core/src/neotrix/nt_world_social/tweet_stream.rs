use super::x_scraper::RawTweet;
use std::collections::{HashMap, HashSet};

const MAX_SEEN_TWEETS: usize = 50_000;

/// 推文去重指纹
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TweetFingerprint {
    pub tweet_id: String,
    pub text_hash: u64,
}

impl TweetFingerprint {
    pub fn from_tweet(t: &RawTweet) -> Self {
        let text_hash = simple_hash(&t.text);
        Self {
            tweet_id: t.tweet_id.clone(),
            text_hash,
        }
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

/// 负熵评分结果
#[derive(Debug, Clone)]
pub struct NegentropyScore {
    pub raw_tweet: RawTweet,
    /// 信息增益: [0, 1] — 越高表示越值得吸收
    pub information_gain: f64,
    /// 新奇性: [0, 1] — 基于内容与既有知识的差异
    pub novelty: f64,
    /// 相关性: [0, 1] — 与当前兴趣/知识缺口的匹配度
    pub relevance: f64,
    /// 信号纯度: [0, 1] — 内容密度(去噪声), 短/垃圾推文扣分
    pub signal_purity: f64,
    /// 综合负熵值: information_gain * (0.4 + 0.3*novelty + 0.2*relevance + 0.1*signal_purity)
    pub negentropy: f64,
}

impl NegentropyScore {
    pub fn is_worth_absorbing(&self) -> bool {
        self.negentropy > 0.25
    }
}

/// 推文流处理引擎 — 负熵：去重 → 新奇性评分 → 信息增益排序 → 选择性吸收
pub struct TweetStream {
    /// 已见过的推文指纹 (去重)
    seen: HashSet<TweetFingerprint>,
    /// 已见过的 author+text 前缀 (近似去重)
    seen_prefixes: HashSet<u64>,
    /// 已吸收推文计数
    absorbed_count: usize,
    /// 总处理计数
    total_seen: usize,
}

impl Default for TweetStream {
    fn default() -> Self {
        Self::new()
    }
}

impl TweetStream {
    pub fn new() -> Self {
        Self {
            seen: HashSet::with_capacity(4096),
            seen_prefixes: HashSet::with_capacity(4096),
            absorbed_count: 0,
            total_seen: 0,
        }
    }

    /// 去重检查: 已见过的推文返回 false
    pub fn is_novel(&self, tweet: &RawTweet) -> bool {
        let fp = TweetFingerprint::from_tweet(tweet);
        if self.seen.contains(&fp) {
            return false;
        }
        let prefix = simple_hash(&tweet.text.chars().take(60).collect::<String>());
        if self.seen_prefixes.contains(&prefix) {
            return false;
        }
        true
    }

    /// 标记为已处理
    pub fn mark_seen(&mut self, tweet: &RawTweet) {
        if self.seen.len() >= MAX_SEEN_TWEETS {
            let drain_count = self.seen.len() / 2;
            let to_remove: Vec<TweetFingerprint> =
                self.seen.iter().take(drain_count).cloned().collect();
            for fp in &to_remove {
                self.seen.remove(fp);
            }
            self.seen_prefixes.clear();
        }
        let fp = TweetFingerprint::from_tweet(tweet);
        self.seen.insert(fp);
        let prefix = simple_hash(&tweet.text.chars().take(60).collect::<String>());
        self.seen_prefixes.insert(prefix);
        self.total_seen += 1;
    }

    /// 计算单条推文的信号纯度 (去噪声)
    pub fn signal_purity(tweet: &RawTweet) -> f64 {
        let text = tweet.text.trim();
        if text.len() < 10 {
            return 0.05;
        }
        let word_count = text.split_whitespace().count() as f64;
        let char_count = text.len() as f64;
        let avg_word_len = char_count / word_count.max(1.0);
        let has_url = text.contains("http");
        let mention_ratio = text.matches('@').count() as f64 / word_count.max(1.0);
        let hash_ratio = text.matches('#').count() as f64 / word_count.max(1.0);

        let mut purity = 1.0;
        purity -= mention_ratio.min(0.5) * 0.3; // 过多 @ 降级
        purity -= hash_ratio.min(0.5) * 0.2; // 过多 # 降级
        purity += (avg_word_len / 8.0).min(0.2); // 长词正信号
        if has_url {
            purity -= 0.1;
        } // 纯链接稀释
        if text.len() < 30 {
            purity -= 0.2;
        }
        if tweet.likes < 1 && tweet.retweets < 1 && !tweet.is_thread {
            purity -= 0.1; // 零互动降级
        }
        purity.clamp(0.0, 1.0)
    }

    /// 计算推文的信息增益 (负熵核心)
    /// 基于: 内容长度 * 信号纯度 * (1 - 与关键词的重复度)
    pub fn information_gain(tweet: &RawTweet, known_concepts: &[String]) -> f64 {
        let text = tweet.text.trim();
        if text.is_empty() {
            return 0.0;
        }
        let purity = Self::signal_purity(tweet);
        let length_factor = (text.len() as f64 / 500.0).min(1.0);
        let interaction_boost =
            ((tweet.likes as f64 * 0.01) + (tweet.retweets as f64 * 0.03)).min(0.3);

        let known_overlap = if known_concepts.is_empty() {
            0.0
        } else {
            let text_lower = text.to_lowercase();
            let matches = known_concepts
                .iter()
                .filter(|c| text_lower.contains(&c.to_lowercase()))
                .count();
            (matches as f64 / known_concepts.len() as f64).min(0.8)
        };

        let gain = length_factor * purity * (1.0 - known_overlap) + interaction_boost;
        gain.clamp(0.0, 1.0)
    }

    /// 计算综合负熵分数
    pub fn score_tweet(
        &self,
        tweet: &RawTweet,
        known_concepts: &[String],
        curiosity_bonus: f64,
    ) -> NegentropyScore {
        let info_gain = Self::information_gain(tweet, known_concepts);
        let purity = Self::signal_purity(tweet);
        let novelty = if self.total_seen == 0 {
            1.0
        } else {
            1.0 - (self.total_seen as f64 / 1000.0).min(0.5)
        };
        let relevance = (info_gain * 0.5 + purity * 0.3 + curiosity_bonus * 0.2).min(1.0);

        let negentropy = info_gain * (0.4 + 0.3 * novelty + 0.2 * relevance + 0.1 * purity);

        NegentropyScore {
            raw_tweet: tweet.clone(),
            information_gain: info_gain,
            novelty,
            relevance,
            signal_purity: purity,
            negentropy,
        }
    }

    /// 批量处理时间线: 去重 → 评分 → 按负熵排序 → 返回值得吸收的
    pub fn process_timeline(
        &mut self,
        timeline: &[RawTweet],
        known_concepts: &[String],
        curiosity_bonus: f64,
        max_absorb: usize,
    ) -> Vec<NegentropyScore> {
        let mut scored: Vec<NegentropyScore> = Vec::new();

        for tweet in timeline {
            if !self.is_novel(tweet) {
                continue;
            }
            self.mark_seen(tweet);
            let score = self.score_tweet(tweet, known_concepts, curiosity_bonus);
            if score.is_worth_absorbing() {
                scored.push(score);
            }
        }

        // 按负熵降序排列
        scored.sort_by(|a, b| {
            b.negentropy
                .partial_cmp(&a.negentropy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(max_absorb);

        let count = scored.len();
        self.absorbed_count += count;

        scored
    }

    /// 从推文文本提取关键词 (简单实现)
    pub fn extract_keywords(text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .filter(|w| !w.starts_with("http") && !w.starts_with('@') && !w.starts_with('#'))
            .collect();
        let mut freq: HashMap<&str, usize> = HashMap::new();
        for w in &words {
            *freq.entry(w).or_insert(0) += 1;
        }
        let mut keywords: Vec<String> = freq
            .into_iter()
            .filter(|(_, c)| *c > 1)
            .map(|(w, _)| w.to_string())
            .collect();
        keywords.sort();
        keywords.truncate(10);
        keywords
    }

    /// 将 NegentropyScore 转换为 IngestionScratchpad 的 input 文本
    pub fn score_to_ingestion_text(score: &NegentropyScore) -> String {
        format!(
            "[Social] @{}: {}\nLikes: {} | RT: {} | Replies: {}\n---\nNegentropy: {:.3} | Gain: {:.3}",
            score.raw_tweet.author_handle,
            score.raw_tweet.text,
            score.raw_tweet.likes,
            score.raw_tweet.retweets,
            score.raw_tweet.replies,
            score.negentropy,
            score.information_gain,
        )
    }

    pub fn stats(&self) -> TweetStreamStats {
        TweetStreamStats {
            total_seen: self.total_seen,
            absorbed_count: self.absorbed_count,
            dedup_set_size: self.seen.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TweetStreamStats {
    pub total_seen: usize,
    pub absorbed_count: usize,
    pub dedup_set_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tweet(id: &str, text: &str, likes: u64, rt: u64) -> RawTweet {
        RawTweet {
            tweet_id: id.into(),
            author: "Test".into(),
            author_handle: "test".into(),
            text: text.into(),
            created_at: 1000,
            likes,
            retweets: rt,
            replies: 0,
            views: None,
            url: format!("https://x.com/i/web/status/{}", id),
            is_thread: false,
            has_media: false,
            language: Some("en".into()),
        }
    }

    #[test]
    fn test_dedup_exact() {
        let mut stream = TweetStream::new();
        let t = make_tweet(
            "1",
            "Hello world this is a test tweet with enough words",
            10,
            2,
        );
        assert!(stream.is_novel(&t));
        stream.mark_seen(&t);
        assert!(!stream.is_novel(&t));
    }

    #[test]
    fn test_dedup_prefix() {
        let mut stream = TweetStream::new();
        let t1 = make_tweet(
            "1",
            "Hello world this is a test tweet with enough words for dedup",
            10,
            2,
        );
        let t2 = make_tweet(
            "2",
            "Hello world this is a test tweet with enough words but different",
            5,
            1,
        );
        stream.mark_seen(&t1);
        assert!(!stream.is_novel(&t2));
    }

    #[test]
    fn test_signal_purity_long_text() {
        let t = make_tweet("1", "This is a meaningful sentence with substantial length and real information content for analysis and signal processing purposes", 10, 5);
        let purity = TweetStream::signal_purity(&t);
        assert!(purity > 0.5);
    }

    #[test]
    fn test_signal_purity_short() {
        let t = make_tweet("1", "hi", 0, 0);
        let purity = TweetStream::signal_purity(&t);
        assert!(purity < 0.3);
    }

    #[test]
    fn test_information_gain_known() {
        let t = make_tweet(
            "1",
            "Rust programming is great for systems programming",
            50,
            10,
        );
        let gain = TweetStream::information_gain(&t, &["rust".into(), "programming".into()]);
        let gain2 = TweetStream::information_gain(&t, &[]);
        assert!(gain <= gain2);
    }

    #[test]
    fn test_score_tweet() {
        let stream = TweetStream::new();
        let t = make_tweet("1", "A new breakthrough in AI alignment research with impressive results and detailed methodology that advances the field significantly", 100, 25);
        let score = stream.score_tweet(&t, &[], 0.3);
        assert!(score.negentropy > 0.0);
        assert!(score.information_gain > 0.0);
    }

    #[test]
    fn test_process_timeline_dedup() {
        let mut stream = TweetStream::new();
        let t1 = make_tweet("1", "First tweet with substantial content for analysis purposes and signal detection in social media streams", 10, 2);
        let t2 = make_tweet("1", "Different tweet content that should be novel and score highly for ingestion priority ranking", 5, 1);
        let timeline = vec![t1.clone(), t1.clone(), t2.clone()];
        let results = stream.process_timeline(&timeline, &[], 0.0, 10);
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_extract_keywords() {
        let kw = TweetStream::extract_keywords(
            "Rust programming language is great for systems programming and network services",
        );
        assert!(kw.contains(&"programming".to_string()));
    }

    #[test]
    fn test_score_to_ingestion_text() {
        let t = make_tweet(
            "1",
            "Test tweet with enough words for ingestion conversion pipeline testing purposes",
            10,
            2,
        );
        let stream = TweetStream::new();
        let score = stream.score_tweet(&t, &[], 0.0);
        let text = TweetStream::score_to_ingestion_text(&score);
        assert!(text.contains("@test"));
        assert!(text.contains("Negentropy"));
    }

    #[test]
    fn test_stats() {
        let mut stream = TweetStream::new();
        let t = make_tweet("1", "Test tweet with enough content words for statistical tracking and metrics collection verification", 5, 1);
        stream.process_timeline(&[t], &[], 0.0, 10);
        let stats = stream.stats();
        assert!(stats.total_seen >= 1);
    }

    #[test]
    fn test_negentropy_worth_absorbing() {
        let t = make_tweet("1", "A groundbreaking paper on quantum computing demonstrates a new error correction code that reduces overhead by 40 percent while maintaining fault tolerance thresholds that were previously thought impossible to achieve with current hardware limitations", 500, 150);
        let stream = TweetStream::new();
        let score = stream.score_tweet(&t, &[], 0.5);
        assert!(score.is_worth_absorbing());
    }
}
