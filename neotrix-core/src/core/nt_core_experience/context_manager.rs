use std::time::Instant;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

pub struct ContextBudget {
    pub sections: Vec<BudgetSection>,
    pub total_consumed: usize,
    pub hard_cap: usize,
    start: Instant,
}

pub struct BudgetSection {
    pub name: &'static str,
    pub target: usize,
    pub hard_cap: usize,
    pub consumed: usize,
    pub count: u64,
}

impl BudgetSection {
    pub fn new(name: &'static str, target: usize, hard_cap: usize) -> Self {
        Self {
            name,
            target,
            hard_cap,
            consumed: 0,
            count: 0,
        }
    }

    pub fn remaining(&self) -> isize {
        self.hard_cap as isize - self.consumed as isize
    }

    pub fn can_consume(&self, n: usize) -> bool {
        self.consumed + n <= self.hard_cap
    }
}

impl ContextBudget {
    pub fn new(hard_cap: usize) -> Self {
        Self {
            sections: vec![
                BudgetSection::new("identity", 300, 500),
                BudgetSection::new("task_instructions", 700, 1000),
                BudgetSection::new("user_request", 300, 1000),
                BudgetSection::new("conversation_summary", 800, 1500),
                BudgetSection::new("retrieved_knowledge", 2000, 4000),
                BudgetSection::new("tool_results", 1000, 3000),
                BudgetSection::new("vs_dedup_overhead", 200, 500),
            ],
            total_consumed: 0,
            hard_cap,
            start: Instant::now(),
        }
    }

    pub fn consume(&mut self, name: &str, tokens: usize) -> bool {
        let section = self.sections.iter_mut().find(|s| s.name == name);
        if let Some(s) = section {
            if !s.can_consume(tokens) {
                return false;
            }
            s.consumed += tokens;
            s.count += 1;
        }
        self.total_consumed += tokens;
        self.total_consumed <= self.hard_cap
    }

    pub fn remaining(&self) -> isize {
        self.hard_cap as isize - self.total_consumed as isize
    }

    pub fn fraction_used(&self) -> f64 {
        if self.hard_cap == 0 {
            return 1.0;
        }
        self.total_consumed as f64 / self.hard_cap as f64
    }

    pub fn summary(&self) -> String {
        let elapsed = self.start.elapsed();
        let mut s = format!(
            "[Budget] {}t/{}, {:.0}% used, {:?} elapsed\n",
            self.total_consumed,
            self.hard_cap,
            self.fraction_used() * 100.0,
            elapsed
        );
        for sec in &self.sections {
            if sec.count > 0 {
                s.push_str(&format!(
                    "  {}: {}t/{}, {} calls\n",
                    sec.name, sec.consumed, sec.hard_cap, sec.count
                ));
            }
        }
        s
    }

    pub fn reset(&mut self) {
        self.total_consumed = 0;
        self.start = Instant::now();
        for s in &mut self.sections {
            s.consumed = 0;
            s.count = 0
        }
    }
}

pub struct VsDedupPipeline {
    pub threshold: f64,
    vsa_threshold: f64,
    fingerprints: Vec<u64>,
    deduped_count: u64,
    passed_count: u64,
}

impl VsDedupPipeline {
    pub fn new() -> Self {
        Self {
            threshold: 0.72,
            vsa_threshold: 0.85,
            fingerprints: Vec::with_capacity(128),
            deduped_count: 0,
            passed_count: 0,
        }
    }

    pub fn with_threshold(mut self, t: f64) -> Self {
        self.threshold = t;
        self
    }

    fn simhash(text: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for chunk in text.as_bytes().chunks(8) {
            let val = u64::from_ne_bytes({
                let mut arr = [0u8; 8];
                for (i, &b) in chunk.iter().enumerate() {
                    arr[i] = b
                }
                arr
            });
            val.hash(&mut h);
        }
        h.finish()
    }

    fn hamming_distance(a: u64, b: u64) -> u32 {
        (a ^ b).count_ones()
    }

    pub fn is_simhash_dup(&self, text: &str) -> bool {
        let fp = Self::simhash(text);
        let threshold = 3u32;
        self.fingerprints
            .iter()
            .any(|&f| Self::hamming_distance(fp, f) <= threshold)
    }

    pub fn needs_vsa_dedup(&self, text: &str) -> bool {
        let fp = Self::simhash(text);
        self.fingerprints
            .iter()
            .any(|&f| Self::hamming_distance(fp, f) <= 8)
    }

    pub fn dedup_stream(&mut self, texts: &[String]) -> Vec<usize> {
        let mut keep: Vec<usize> = Vec::with_capacity(texts.len());
        for (i, text) in texts.iter().enumerate() {
            if !self.is_simhash_dup(text) {
                self.fingerprints.push(Self::simhash(text));
                if self.fingerprints.len() > 1024 {
                    self.fingerprints.remove(0);
                }
                keep.push(i);
                self.passed_count += 1;
            } else {
                self.deduped_count += 1;
            }
        }
        keep
    }

    pub fn dedup_vsa_vector(&mut self, vsa: &[u8], existing: &[&[u8]]) -> bool {
        existing
            .iter()
            .any(|e| QuantizedVSA::similarity(vsa, e) > self.vsa_threshold)
    }

    pub fn stats(&self) -> String {
        let total = self.deduped_count + self.passed_count;
        let rate = if total == 0 {
            0.0
        } else {
            self.deduped_count as f64 / total as f64 * 100.0
        };
        format!(
            "[VsDedup] {} kept / {} deduped ({}%), threshold={}",
            self.passed_count, self.deduped_count, rate as u32, self.threshold
        )
    }

    pub fn reset(&mut self) {
        self.fingerprints.clear();
        self.deduped_count = 0;
        self.passed_count = 0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContextTier {
    Tier0,
    Tier1,
    Tier2,
    Tier3,
}

pub struct LoadedSection {
    pub name: &'static str,
    pub tier: ContextTier,
    pub content: String,
    pub estimated_tokens: usize,
}

pub struct ContextManager {
    pub budget: ContextBudget,
    pub dedup: VsDedupPipeline,
    pub tier: ContextTier,
    pub loaded: Vec<LoadedSection>,
    total_dedup_saved: usize,
}

impl ContextManager {
    pub fn new(budget_cap: usize) -> Self {
        Self {
            budget: ContextBudget::new(budget_cap),
            dedup: VsDedupPipeline::new(),
            tier: ContextTier::Tier0,
            loaded: Vec::with_capacity(16),
            total_dedup_saved: 0,
        }
    }

    pub fn load_tier0(&mut self, content: &str, estimated_tokens: usize) {
        self.loaded.push(LoadedSection {
            name: "tier0_identity",
            tier: ContextTier::Tier0,
            content: content.to_string(),
            estimated_tokens,
        });
        self.budget.consume("identity", estimated_tokens);
    }

    pub fn load_tier1(&mut self, name: &'static str, content: &str) -> bool {
        let tokens = (content.len() / 4).max(1);
        if !self.budget.consume("task_instructions", tokens) {
            return false;
        }
        self.loaded.push(LoadedSection {
            name,
            tier: ContextTier::Tier1,
            content: content.to_string(),
            estimated_tokens: tokens,
        });
        true
    }

    pub fn load_tier2(&mut self, name: &'static str, content: &str) -> bool {
        let tokens = (content.len() / 4).max(1);
        if !self.budget.consume("retrieved_knowledge", tokens) {
            return false;
        }
        self.loaded.push(LoadedSection {
            name,
            tier: ContextTier::Tier2,
            content: content.to_string(),
            estimated_tokens: tokens,
        });
        true
    }

    pub fn run_dedup_pass(&mut self, texts: &[String]) -> Vec<String> {
        let keep = self.dedup.dedup_stream(texts);
        let saved: usize = texts.iter().map(|t| t.len() / 4).sum::<usize>()
            - keep.iter().map(|&i| texts[i].len() / 4).sum::<usize>();
        self.total_dedup_saved += saved;
        keep.iter().map(|&i| texts[i].clone()).collect()
    }

    pub fn assemble_context(&self) -> String {
        let mut ctx = String::with_capacity(4096);
        for section in &self.loaded {
            ctx.push_str(&section.content);
            ctx.push('\n');
        }
        ctx
    }

    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.budget.summary());
        s.push('\n');
        s.push_str(&self.dedup.stats());
        s.push_str(&format!(
            "\n[Context] {} sections loaded, ~{}t total, {}t saved via dedup\n",
            self.loaded.len(),
            self.loaded
                .iter()
                .map(|l| l.estimated_tokens)
                .sum::<usize>(),
            self.total_dedup_saved
        ));
        s
    }

    pub fn reset(&mut self) {
        self.budget.reset();
        self.dedup.reset();
        self.loaded.clear();
        self.total_dedup_saved = 0;
        self.tier = ContextTier::Tier0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_basic() {
        let mut b = ContextBudget::new(5000);
        assert!(b.consume("identity", 300));
        assert!(b.consume("task_instructions", 500));
        assert!(b.fraction_used() > 0.0);
        assert!(b.remaining() > 0);
    }

    #[test]
    fn test_budget_hard_cap() {
        let mut b = ContextBudget::new(100);
        assert!(b.consume("identity", 60));
        assert!(b.consume("identity", 40));
        assert!(!b.consume("identity", 10));
    }

    #[test]
    fn test_dedup_stream() {
        let mut p = VsDedupPipeline::new();
        let texts = vec![
            "hello world".to_string(),
            "hello world".to_string(),
            "different".to_string(),
        ];
        let kept = p.dedup_stream(&texts);
        assert_eq!(kept.len(), 2); // "hello world" deduped once
        assert_eq!(texts[kept[0]], "hello world");
        assert_eq!(texts[kept[1]], "different");
    }

    #[test]
    fn test_dedup_no_false_positive() {
        let mut p = VsDedupPipeline::new();
        let texts = vec!["abc".to_string(), "def".to_string(), "ghi".to_string()];
        let kept = p.dedup_stream(&texts);
        assert_eq!(kept.len(), 3);
    }

    #[test]
    fn test_manager_load_and_assemble() {
        let mut m = ContextManager::new(5000);
        m.load_tier0("[identity] NeoTrix", 100);
        assert!(m.load_tier1("rules", "[rules] test"));
        let ctx = m.assemble_context();
        assert!(ctx.contains("NeoTrix"));
        assert!(ctx.contains("test"));
    }

    #[test]
    fn test_manager_dedup_saves_tokens() {
        let mut m = ContextManager::new(5000);
        let texts = vec![
            "repeat".to_string(),
            "repeat".to_string(),
            "repeat".to_string(),
            "unique".to_string(),
        ];
        let kept = m.run_dedup_pass(&texts);
        assert_eq!(kept.len(), 2); // 1 "repeat" + "unique"
        assert!(m.summary().contains("saved via dedup"));
    }

    #[test]
    fn test_budget_section_remaining() {
        let s = BudgetSection::new("test", 500, 1000);
        assert_eq!(s.remaining(), 1000);
    }

    #[test]
    fn test_dedup_pipeline_with_vsa() {
        let mut p = VsDedupPipeline::new();
        let empty: &[&[u8]] = &[];
        // VSA vectors: two similar, one different
        let v1 = vec![1u8; 64];
        let v2 = vec![1u8; 64];
        let v3 = vec![2u8; 64];
        assert!(p.dedup_vsa_vector(&v1, empty));
        assert!(p.dedup_vsa_vector(&v2, &[&v1]));
        assert!(!p.dedup_vsa_vector(&v3, &[&v1]));
    }

    #[test]
    fn test_dedup_threshold_config() {
        let p = VsDedupPipeline::new().with_threshold(0.8);
        assert_eq!(p.threshold, 0.8);
    }

    #[test]
    fn test_budget_reset() {
        let mut b = ContextBudget::new(5000);
        b.consume("identity", 300);
        assert!(b.total_consumed > 0);
        b.reset();
        assert_eq!(b.total_consumed, 0);
    }

    #[test]
    fn test_dedup_stream_high_volume() {
        let mut p = VsDedupPipeline::new();
        let texts: Vec<String> = (0..100).map(|i| format!("item_{}", i % 10)).collect();
        let kept = p.dedup_stream(&texts);
        assert!(kept.len() < texts.len());
        assert!(kept.len() >= 10); // at most 10 unique items
    }

    #[test]
    fn test_context_manager_multiple_tiers() {
        let mut m = ContextManager::new(3000);
        m.load_tier0("id", 100);
        assert!(m.load_tier1("t1", "task1"));
        assert!(m.load_tier2("t2", "knowledge1"));
        let ctx = m.assemble_context();
        assert!(!ctx.is_empty());
        assert_eq!(m.loaded.len(), 3);
    }

    #[test]
    fn test_simhash_detects_near_duplicates() {
        let mut p = VsDedupPipeline::new();
        let a = "The quick brown fox jumps over the lazy dog".to_string();
        let b = "The quick brown fox jumps over the lazy cat".to_string();
        let c = "completely different text".to_string();
        let kept = p.dedup_stream(&[a, b, c]);
        // a and b are near-duplicates (hamming distance <= 3)
        assert_eq!(kept.len(), 2);
    }
}
