//! Backward-compatible stub: nt_core_embed types

pub const EMBEDDING_DIM: usize = 4096;

pub struct TextEmbedder;

impl TextEmbedder {
    pub fn new() -> Self {
        Self
    }

    /// 词级哈希嵌入 (hashing trick)。
    ///
    /// 2026-09-28 替换原「字节级求和」实现。原实现按 `bytes[i % DIM] += b/255`
    /// 累加, 那是**字节袋**而非词袋: 任意两段英文的相似度都被共有字节分布
    /// (空格、e、i、t 等高频字母) 抬到 0.8 以上, 完全无法表达语义 ——
    /// "implement user authentication" 与 "color scheme for dark mode UI design"
    /// 得分 0.8+, 而 "database connection issue debug" 与真正相关的
    /// "fix database connection pool leak" 反而不是最高分。
    /// 词级 + 次线性 TF 使「共有词」成为唯一的相似度来源, 这正是相似度检索
    /// 需要的东西。保持确定性(同一文本恒等)与 EMBEDDING_DIM 不变,
    /// 消费者只有 seal_core/embedding.rs 与两处未使用的测试实例。
    pub fn embed(&self, text: &str) -> Vec<f64> {
        let mut vec = vec![0.0f64; EMBEDDING_DIM];
        let tokens = tokenize(text);
        if tokens.is_empty() {
            // 无可分词内容(纯符号/空白)时退回字节袋, 保持向量非零。
            for (i, &b) in text.as_bytes().iter().enumerate() {
                vec[i % EMBEDDING_DIM] += (b as f64) / 255.0;
            }
            return vec;
        }
        for tok in tokens {
            let h = fnv1a(&tok);
            let idx = (h as usize) % EMBEDDING_DIM;
            // 次线性 TF: 同一个词重复 10 次不该有 10 倍权重。
            vec[idx] += 1.0 + (counts(&tok, text) as f64).ln();
        }
        vec
    }

    pub fn similarity(&self, a: &str, b: &str) -> f64 {
        let va = self.embed(a);
        let vb = self.embed(b);
        let dot: f64 = va.iter().zip(vb.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = va.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
        let nb: f64 = vb.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
        dot / (na * nb)
    }

    pub fn find_most_similar<'a>(&self, query: &str, candidates: &[&'a str]) -> Option<(usize, f64, &'a str)> {
        candidates
            .iter()
            .enumerate()
            .map(|(i, &c)| (i, self.similarity(query, c), c))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn vocab_size(&self) -> usize {
        EMBEDDING_DIM
    }
}

impl Default for TextEmbedder {
    fn default() -> Self {
        Self
    }
}

/// 分词: ASCII 字母/数字组成的词 + 单个 CJK 字符。
/// CJK 不用空格分词, 故逐字成词, 否则整段中文会退化成一个巨 token。
fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            cur.push(ch.to_ascii_lowercase());
        } else if is_cjk(ch) {
            if !cur.is_empty() { out.push(std::mem::take(&mut cur)); }
            out.push(ch.to_string());
        } else {
            if !cur.is_empty() { out.push(std::mem::take(&mut cur)); }
        }
    }
    if !cur.is_empty() { out.push(cur); }
    out
}

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x3040..=0x30FF | 0xAC00..=0xD7AF)
}

fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// 词在原文中的出现次数(用于次线性 TF)。
fn counts(tok: &str, text: &str) -> usize {
    tokenize(text).iter().filter(|t| *t == tok).count()
}
