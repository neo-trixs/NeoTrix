use std::collections::HashMap;

pub const EMBEDDING_DIM: usize = 64;

static STOP_WORDS: &[&str] = &[
    "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
    "have", "has", "had", "do", "does", "did", "will", "would", "could",
    "should", "may", "might", "shall", "can", "need", "dare", "ought",
    "used", "to", "of", "in", "for", "on", "with", "at", "by", "from",
    "as", "into", "through", "during", "before", "after", "above", "below",
    "between", "out", "off", "over", "under", "again", "further", "then",
    "once", "here", "there", "when", "where", "why", "how", "all", "each",
    "every", "both", "few", "more", "most", "other", "some", "such", "no",
    "nor", "not", "only", "own", "same", "so", "than", "too", "very",
    "just", "because", "but", "and", "or", "if", "while", "that", "this",
    "these", "those", "it", "its", "what", "which", "who", "whom",
];

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        1.0
    } else {
        dot / (norm_a * norm_b)
    }
}

pub struct TextEmbedder {
    vocab: HashMap<String, usize>,
    next_idx: usize,
}

impl Default for TextEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEmbedder {
    pub fn new() -> Self {
        Self {
            vocab: HashMap::new(),
            next_idx: 0,
        }
    }

    pub fn embed(&mut self, text: &str) -> Vec<f64> {
        let tokens = self.tokenize(text);
        let mut vec = vec![0.0f64; EMBEDDING_DIM];

        for token in &tokens {
            let idx = self.get_or_create_index(token);
            if idx < EMBEDDING_DIM {
                vec[idx] += 1.0;
            }
        }

        let norm: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }

        vec
    }

    pub fn embed_texts(&mut self, texts: &[&str]) -> Vec<Vec<f64>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }

    pub fn similarity(&mut self, a: &str, b: &str) -> f64 {
        let va = self.embed(a);
        let vb = self.embed(b);
        cosine_similarity(&va, &vb)
    }

    pub fn find_most_similar<'a>(&mut self, query: &str, candidates: &[&'a str]) -> Option<(usize, f64, &'a str)> {
        if candidates.is_empty() {
            return None;
        }
        let qv = self.embed(query);
        let mut best_idx = 0;
        let mut best_sim = -1.0f64;

        for (i, c) in candidates.iter().enumerate() {
            let cv = self.embed(c);
            let sim = cosine_similarity(&qv, &cv);
            if sim > best_sim {
                best_sim = sim;
                best_idx = i;
            }
        }

        Some((best_idx, best_sim, candidates[best_idx]))
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        let lower = text.to_lowercase();
        let mut tokens = Vec::new();
        let mut word_buf = String::new();
        let mut cjk_buf = String::new();

        for c in lower.chars() {
            if c.is_alphanumeric() {
                word_buf.push(c);
                if c as u32 >= 0x4E00 && c as u32 <= 0x9FFF {
                    cjk_buf.push(c);
                    if cjk_buf.len() >= 2 {
                        tokens.push(cjk_buf.clone());
                        cjk_buf.remove(0);
                    }
                }
            } else {
                if !word_buf.is_empty() {
                    if word_buf.len() > 2 && !STOP_WORDS.contains(&word_buf.as_str()) {
                        tokens.push(word_buf.clone());
                    }
                    word_buf.clear();
                }
                cjk_buf.clear();
            }
        }
        if !word_buf.is_empty() && word_buf.len() > 2 && !STOP_WORDS.contains(&word_buf.as_str()) {
            tokens.push(word_buf);
        }

        let mut seen = std::collections::HashSet::new();
        tokens.retain(|s| seen.insert(s.clone()));
        tokens
    }

    fn get_or_create_index(&mut self, token: &str) -> usize {
        if let Some(&idx) = self.vocab.get(token) {
            return idx;
        }
        let idx = self.next_idx;
        self.vocab.insert(token.to_string(), idx);
        self.next_idx += 1;
        idx
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }

    pub fn known_tokens(&self) -> Vec<String> {
        let mut tokens: Vec<String> = self.vocab.keys().cloned().collect();
        tokens.sort();
        tokens
    }
}
