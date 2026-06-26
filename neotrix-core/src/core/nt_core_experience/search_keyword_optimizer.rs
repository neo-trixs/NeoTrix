/// SearchKeywordOptimizer — 搜索关键词收益率追踪与进化系统
///
/// 追踪每个搜索关键词的信号/噪音比，跨域上下游关联传播，
/// 周期性地修剪低收益关键词并探索高收益关键词的语义邻域。
///
/// 这是搜索策略的自我进化层：让意识知道"哪些词能找到好东西"。
///
/// # 收益率模型
/// yield = (signal_count + 1) / (use_count + 2)  (贝叶斯平滑)
/// confidence = min(1.0, use_count / 10.0)
///
/// # 上下游传播
/// 关键词 A 搜索发现项目 P → 从 P 的派生词 B 被记录为 A 的下游。
/// 当 evolve() 运行时，高收益 A 的下游词获得继承收益加成。

use std::collections::{HashMap, HashSet};

/// 单个关键词的使用记录
#[derive(Debug, Clone)]
pub struct KeywordRecord {
    /// 关键词原文
    pub keyword: String,
    /// 领域分类（"design", "vsa", "agent-framework", "self-evolution" 等）
    pub domain: String,
    /// 使用次数
    pub use_count: u64,
    /// 高信号（相关）结果数
    pub signal_count: u64,
    /// 低信号（噪音）结果数
    pub noise_count: u64,
    /// 最近使用 cycle
    pub last_used: u64,
    /// 首次使用 cycle
    pub first_used: u64,
    /// 已发现的项目/仓库名称（最多 10 条）
    pub signal_sources: Vec<String>,
}

impl KeywordRecord {
    pub fn yield_rate(&self) -> f64 {
        (self.signal_count + 1) as f64 / (self.use_count + 2) as f64
    }

    pub fn confidence(&self) -> f64 {
        (self.use_count as f64 / 10.0).min(1.0)
    }

    /// 复合评分 = yield × confidence，用于排序
    pub fn composite_score(&self) -> f64 {
        self.yield_rate() * self.confidence()
    }
}

/// 优化器配置
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// 关键词被视为"可修剪"的最小使用次数
    pub min_uses_before_prune: u64,
    /// 低收益阈值（低于此值且超过 min_uses 则修剪）
    pub low_yield_threshold: f64,
    /// 高收益阈值（高于此值则传播到下游）
    pub high_yield_threshold: f64,
    /// 修剪间隔（cycle 数）
    pub prune_interval: u64,
    /// 进化间隔（cycle 数）
    pub evolve_interval: u64,
    /// 最大记录的关键词数
    pub max_keywords: usize,
    /// 每个关键词最多追踪的信号来源数
    pub max_signal_sources: usize,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            min_uses_before_prune: 3,
            low_yield_threshold: 0.25,
            high_yield_threshold: 0.6,
            prune_interval: 100,
            evolve_interval: 200,
            max_keywords: 300,
            max_signal_sources: 10,
        }
    }
}

/// 搜索关键词优化器。
#[derive(Debug, Clone)]
pub struct SearchKeywordOptimizer {
    /// 关键词 → 记录
    pub keywords: HashMap<String, KeywordRecord>,
    /// 关键词 → (上游集合, 下游集合)
    /// 上游 = 哪些词发现了当前词；下游 = 当前词发现了哪些词
    pub keyword_graph: HashMap<String, (HashSet<String>, HashSet<String>)>,
    /// 累计搜索次数
    pub search_count: u64,
    /// 上次修剪 cycle
    pub last_prune_cycle: u64,
    /// 上次进化 cycle
    pub last_evolve_cycle: u64,
    /// 配置
    pub config: OptimizerConfig,
}

impl SearchKeywordOptimizer {
    pub fn new(config: OptimizerConfig) -> Self {
        Self {
            keywords: HashMap::new(),
            keyword_graph: HashMap::new(),
            search_count: 0,
            last_prune_cycle: 0,
            last_evolve_cycle: 0,
            config,
        }
    }

    /// 记录一次搜索的结果。
    ///
    /// - `keyword`: 使用的搜索词
    /// - `domain`: 领域标签
    /// - `signal_count`: 本次搜索获得的高信号结果数
    /// - `noise_count`: 本次搜索获得的低信号/噪音结果数
    /// - `signal_sources`: 发现的值得注意的项目/页面名称
    /// - `upstream_keywords`: 哪些上游关键词引导了这次搜索
    /// - `cycle`: 当前 cycle 编号
    pub fn record_search(
        &mut self,
        keyword: &str,
        domain: &str,
        signal_count: u64,
        noise_count: u64,
        signal_sources: Vec<String>,
        upstream_keywords: Vec<String>,
        cycle: u64,
    ) {
        self.search_count += 1;

        // 更新/创建关键词记录
        let entry = self.keywords.entry(keyword.to_string()).or_insert(KeywordRecord {
            keyword: keyword.to_string(),
            domain: domain.to_string(),
            use_count: 0,
            signal_count: 0,
            noise_count: 0,
            last_used: cycle,
            first_used: cycle,
            signal_sources: Vec::new(),
        });

        entry.use_count += 1;
        entry.signal_count += signal_count;
        entry.noise_count += noise_count;
        entry.last_used = cycle;

        // 合并信号来源（去重 + 上限）
        for src in signal_sources {
            if !entry.signal_sources.contains(&src) {
                if entry.signal_sources.len() < self.config.max_signal_sources {
                    entry.signal_sources.push(src.clone());
                }
            }
        }

        // 更新上下游关系
        let graph_entry = self.keyword_graph.entry(keyword.to_string())
            .or_insert_with(|| (HashSet::new(), HashSet::new()));

        for up in &upstream_keywords {
            if up != keyword {
                graph_entry.1.insert(up.clone());
            }
        }
        // 反向下游边（与上游边分离借用）
        for up in &upstream_keywords {
            if up != keyword {
                let down_entry = self.keyword_graph.entry(up.clone())
                    .or_insert_with(|| (HashSet::new(), HashSet::new()));
                down_entry.0.insert(keyword.to_string());
            }
        }
    }

    /// 查询某个领域的最佳关键词（按复合评分降序）。
    pub fn suggest_keywords(&self, domain: &str, count: usize) -> Vec<(String, f64)> {
        let mut scored: Vec<(String, f64)> = self.keywords.iter()
            .filter(|(_, r)| r.domain == domain && r.use_count > 0)
            .map(|(k, r)| (k.clone(), r.composite_score()))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(count);
        scored
    }

    /// 查询所有领域中复合评分最高的关键词（跨域推荐）。
    pub fn suggest_keywords_cross_domain(&self, count: usize) -> Vec<(String, String, f64)> {
        let mut scored: Vec<(String, String, f64)> = self.keywords.iter()
            .filter(|(_, r)| r.use_count > 0)
            .map(|(k, r)| (k.clone(), r.domain.clone(), r.composite_score()))
            .collect();
        scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(count);
        scored
    }

    /// 获取某个关键词的当前收益率信息。
    pub fn keyword_info(&self, keyword: &str) -> Option<(f64, f64, &KeywordRecord)> {
        self.keywords.get(keyword).map(|r| {
            let _upstream = self.keyword_graph.get(keyword)
                .map(|(u, _)| u.len())
                .unwrap_or(0);
            let _downstream = self.keyword_graph.get(keyword)
                .map(|(_, d)| d.len())
                .unwrap_or(0);
            (r.composite_score(), r.yield_rate(), r)
        })
    }

    /// 修剪低收益关键词。
    /// 条件：使用次数 >= min_uses_before_prune 且 yield < low_yield_threshold
    fn prune_low_yield(&mut self) {
        let to_remove: Vec<String> = self.keywords.iter()
            .filter(|(_, r)| {
                r.use_count >= self.config.min_uses_before_prune
                    && r.yield_rate() < self.config.low_yield_threshold
            })
            .map(|(k, _)| k.clone())
            .collect();

        for k in &to_remove {
            self.keywords.remove(k);
            // 同时清理图边
            if let Some((up, down)) = self.keyword_graph.remove(k) {
                for u in &up {
                    if let Some((_, d)) = self.keyword_graph.get_mut(u) {
                        d.remove(k);
                    }
                }
                for d in &down {
                    if let Some((u, _)) = self.keyword_graph.get_mut(d) {
                        u.remove(k);
                    }
                }
            }
        }
    }

    /// 修剪超限关键词（超过 max_keywords 时淘汰低评分词）。
    fn prune_overflow(&mut self) {
        if self.keywords.len() <= self.config.max_keywords {
            return;
        }
        let excess = self.keywords.len() - self.config.max_keywords;
        let mut scored: Vec<(String, f64)> = self.keywords.iter()
            .map(|(k, r)| (k.clone(), r.composite_score()))
            .collect();
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for (k, _) in scored.iter().take(excess) {
            self.keywords.remove(k);
            if let Some((up, down)) = self.keyword_graph.remove(k) {
                for u in &up {
                    if let Some((_, d)) = self.keyword_graph.get_mut(u) {
                        d.remove(k);
                    }
                }
                for d in &down {
                    if let Some((u, _)) = self.keyword_graph.get_mut(d) {
                        u.remove(k);
                    }
                }
            }
        }
    }

    /// 进化：将高收益关键词的信号传播到下游词。
    ///
    /// 对每个高收益关键词（yield > high_yield_threshold），
    /// 遍历其下游词，如果下游词尚未建立足够的置信度，
    /// 则将其复合评分提升到接近上游词的 yield × 0.8。
    /// 这样新词不需要大量使用就能继承关联词的信噪比。
    fn evolve_keywords(&mut self) {
        // 收集高收益关键词
        let high_yield: Vec<String> = self.keywords.iter()
            .filter(|(_, r)| r.yield_rate() > self.config.high_yield_threshold
                && r.use_count >= self.config.min_uses_before_prune)
            .map(|(k, _)| k.clone())
            .collect();

        for k in &high_yield {
            let parent_yield = self.keywords.get(k)
                .map(|r| r.yield_rate())
                .unwrap_or(0.5);

            // 遍历下游词
            let downstream = self.keyword_graph.get(k)
                .map(|(_, d)| d.iter().cloned().collect::<Vec<_>>())
                .unwrap_or_default();

            for d in &downstream {
                if let Some(child) = self.keywords.get_mut(d) {
                    if child.use_count < self.config.min_uses_before_prune {
                        // 下游词使用不足，注入继承信号
                        let inherited = (parent_yield * 0.8).max(child.yield_rate());
                        let synthetic_signal = (inherited * 3.0).ceil() as u64;
                        child.signal_count = child.signal_count.max(synthetic_signal);
                    }
                }
            }
        }
    }

    /// 周期性修剪 + 进化。应在 tick() 中调用。
    pub fn tick(&mut self, cycle: u64) {
        // 修剪低收益词
        if cycle - self.last_prune_cycle >= self.config.prune_interval {
            self.prune_low_yield();
            self.prune_overflow();
            self.last_prune_cycle = cycle;
        }

        // 进化：传播高收益信号到下游
        if cycle - self.last_evolve_cycle >= self.config.evolve_interval {
            self.evolve_keywords();
            self.last_evolve_cycle = cycle;
        }
    }

    /// 统计摘要。
    pub fn stats(&self) -> String {
        let total = self.keywords.len();
        let high_yield_count = self.keywords.values()
            .filter(|r| r.yield_rate() > self.config.high_yield_threshold)
            .count();
        let low_yield_count = self.keywords.values()
            .filter(|r| r.yield_rate() < self.config.low_yield_threshold && r.use_count >= self.config.min_uses_before_prune)
            .count();

        // 按域统计
        let mut domain_counts: HashMap<&str, usize> = HashMap::new();
        for r in self.keywords.values() {
            *domain_counts.entry(&r.domain).or_insert(0) += 1;
        }
        let mut domain_strs: Vec<String> = domain_counts.into_iter()
            .map(|(d, c)| format!("{}={}", d, c))
            .collect();
        domain_strs.sort();
        let avg_yield: f64 = if total > 0 {
            self.keywords.values().map(|r| r.yield_rate()).sum::<f64>() / total as f64
        } else {
            0.0
        };

        format!(
            "keywords: {} total ({} high, {} low, avg_yield={:.2}), searches={}, domains=[{}]",
            total, high_yield_count, low_yield_count, avg_yield, self.search_count,
            domain_strs.join(", "),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_opt() -> SearchKeywordOptimizer {
        SearchKeywordOptimizer::new(OptimizerConfig {
            min_uses_before_prune: 2,
            evolve_interval: 50,
            ..Default::default()
        })
    }

    #[test]
    fn test_record_search_creates_entry() {
        let mut opt = default_opt();
        opt.record_search("open-design", "design", 3, 1, vec!["nexu-io/open-design".into()], vec![], 1);
        assert_eq!(opt.keywords.len(), 1);
        let r = opt.keywords.get("open-design").unwrap();
        assert_eq!(r.use_count, 1);
        assert_eq!(r.signal_count, 3);
        assert_eq!(r.noise_count, 1);
    }

    #[test]
    fn test_record_search_updates_existing() {
        let mut opt = default_opt();
        opt.record_search("open-design", "design", 3, 1, vec![], vec![], 1);
        opt.record_search("open-design", "design", 1, 2, vec![], vec![], 2);
        let r = opt.keywords.get("open-design").unwrap();
        assert_eq!(r.use_count, 2);
        assert_eq!(r.signal_count, 4);
        assert_eq!(r.noise_count, 3);
    }

    #[test]
    fn test_yield_rate_bayesian() {
        let mut opt = default_opt();
        // 0 uses → yield = (0+1)/(0+2) = 0.5
        assert!(opt.keywords.is_empty());
        // 3 signal / 1 noise out of 4 uses → yield ≈ (3+1)/(4+2) = 0.667
        opt.record_search("test-kw", "test", 3, 1, vec![], vec![], 1);
        let r = opt.keywords.get("test-kw").unwrap();
        assert!((r.yield_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_suggest_keywords_orders_by_score() {
        let mut opt = default_opt();
        opt.record_search("high-yield", "design", 10, 0, vec![], vec![], 1);
        opt.record_search("low-yield", "design", 0, 5, vec![], vec![], 1);
        let suggestions = opt.suggest_keywords("design", 2);
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].0, "high-yield");
    }

    #[test]
    fn test_upstream_downstream_graph() {
        let mut opt = default_opt();
        // A 发现 B
        opt.record_search("design-systems", "design", 2, 0,
            vec!["awesome-design-systems".into()], vec![], 1);
        // B 是 A 的下游（从 A 发现了 B）
        opt.record_search("tokens-css", "design", 1, 0,
            vec![], vec!["design-systems".into()], 2);

        let graph = &opt.keyword_graph;
        // tokens-css 的上游包含 design-systems
        assert!(graph.get("tokens-css")
            .map(|(up, _)| up.contains("design-systems"))
            .unwrap_or(false));
        // design-systems 的下游包含 tokens-css
        assert!(graph.get("design-systems")
            .map(|(_, down)| down.contains("tokens-css"))
            .unwrap_or(false));
    }

    #[test]
    fn test_prune_low_yield() {
        let mut opt = default_opt();
        opt.record_search("good", "test", 5, 0, vec![], vec![], 1);
        opt.record_search("bad", "test", 0, 5, vec![], vec![], 1);
        opt.record_search("bad", "test", 0, 3, vec![], vec![], 2);
        assert_eq!(opt.keywords.len(), 2);
        opt.prune_low_yield();
        assert_eq!(opt.keywords.len(), 1);
        assert!(opt.keywords.contains_key("good"));
    }

    #[test]
    fn test_evolve_propagates_signal() {
        let mut opt = SearchKeywordOptimizer::new(OptimizerConfig {
            min_uses_before_prune: 2,
            high_yield_threshold: 0.5,
            evolve_interval: 5,
            ..Default::default()
        });

        // 高收益父词
        opt.record_search("parent", "test", 8, 1, vec![], vec![], 1);
        opt.record_search("parent", "test", 7, 2, vec![], vec![], 2);

        // 低使用次数的下游子词
        opt.record_search("child", "test", 0, 0, vec![], vec!["parent".into()], 3);

        let child_before = opt.keywords.get("child").unwrap().signal_count;
        assert_eq!(child_before, 0);

        opt.evolve_keywords();

        let child_after = opt.keywords.get("child").unwrap().signal_count;
        assert!(child_after > 0, "child should inherit signal from parent");
    }

    #[test]
    fn test_suggest_keywords_cross_domain() {
        let mut opt = default_opt();
        opt.record_search("vsa-rs", "vsa", 5, 0, vec![], vec![], 1);
        opt.record_search("open-design", "design", 3, 1, vec![], vec![], 1);
        let cross = opt.suggest_keywords_cross_domain(2);
        assert_eq!(cross.len(), 2);
        // vsa-rs should rank first (higher yield)
        assert_eq!(cross[0].0, "vsa-rs");
    }

    #[test]
    fn test_prune_overflow() {
        let mut opt = SearchKeywordOptimizer::new(OptimizerConfig {
            max_keywords: 3,
            min_uses_before_prune: 1,
            ..Default::default()
        });
        for i in 0..5 {
            opt.record_search(&format!("kw{}", i), "test", 1, 0, vec![], vec![], i);
        }
        assert_eq!(opt.keywords.len(), 5);
        opt.prune_overflow();
        assert!(opt.keywords.len() <= 3);
    }

    #[test]
    fn test_tick_does_not_crash() {
        let mut opt = default_opt();
        opt.record_search("test", "test", 1, 0, vec![], vec![], 1);
        opt.tick(10);
        opt.tick(110);
        assert!(opt.keywords.contains_key("test"));
    }

    #[test]
    fn test_stats_returns_string() {
        let mut opt = default_opt();
        let s = opt.stats();
        assert!(s.contains("keywords: 0"));
        opt.record_search("test", "test", 1, 0, vec![], vec![], 1);
        let s = opt.stats();
        assert!(s.contains("keywords: 1"));
        assert!(s.contains("test=1"));
    }
}
