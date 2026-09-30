//! # nt_shared_mind — 并行子任务共享上下文
//!
//! 多个 Reasoning 子任务并行执行时，通过 `SharedMind` 实时共享发现，
//! 避免重复调用 LLM（Jaccard 重叠检测）。
//!
//! ```text
//! thread-1 ──post(fact)──▶ SharedMind ◀──post(fact)── thread-2
//!                              │
//! thread-1 ◀──check_overlap───┤
//! thread-2 ◀──check_overlap───┘
//! ```

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

// ── 文本工具（与 nt_crystal_task_fusion 同源） ──

/// 2026-09-29：本地副本已删，改用唯一事实源。
/// ⛔ 分词必须用**窄**口径（仅汉字）—— 宽口径会把 CJK 标点/全角塞进 bigram，
/// 产出 `付，` `，网` 这类垃圾词元。实测见 `nt_cjk` 的同名测试。
fn is_cjk(c: char) -> bool {
    neotrix_types::core::nt_cjk::is_cjk_han(c)
}

pub(crate) fn keywords(text: &str) -> HashSet<String> {
    let lower = text.to_lowercase();
    let mut set = HashSet::new();
    for tok in lower.split(|c: char| !(c.is_alphanumeric() || is_cjk(c))) {
        if tok.chars().count() >= 2 {
            set.insert(tok.to_string());
        }
    }
    let cjk: Vec<char> = lower.chars().filter(|c| is_cjk(*c)).collect();
    for w in cjk.windows(2) {
        set.insert(w.iter().collect());
    }
    set
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count() as f64;
    let union = (a.len() + b.len()) as f64 - inter;
    if union <= 0.0 {
        0.0
    } else {
        (inter / union).clamp(0.0, 1.0)
    }
}

// ── 类型 ──

/// 单条发现（子任务产出的事实片段）。
#[derive(Debug, Clone)]
pub struct Discovery {
    /// 发现的文本内容。
    pub text: String,
    /// 来源子任务 ID。
    pub source_id: String,
    /// 置信度。
    pub confidence: f64,
}

/// 共享心智内部状态。
#[derive(Debug, Default)]
struct SharedMindInner {
    /// 已发布的发现（子任务完成后写入）。
    facts: Vec<Discovery>,
    /// 正在执行的子任务 ID 集合（用于 UI 展示并行度）。
    active_ids: HashSet<String>,
}

/// 并行子任务共享上下文。
///
/// 多个线程通过 `Arc<SharedMind>` 共享；`Mutex` 仅在 post/check 时短暂持有，
/// 不阻塞 LLM 调用。
#[derive(Debug, Clone)]
pub struct SharedMind {
    inner: Arc<Mutex<SharedMindInner>>,
}

impl Default for SharedMind {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedMind {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SharedMindInner::default())),
        }
    }

    /// 子任务开始时注册（UI 可展示并行 spinner）。
    pub fn register_active(&self, id: &str) {
        self.inner.lock().unwrap().active_ids.insert(id.to_string());
    }

    /// 子任务结束时注销。
    pub fn unregister_active(&self, id: &str) {
        self.inner.lock().unwrap().active_ids.remove(id);
    }

    /// 发布一条发现（子任务产出后调用）。
    ///
    /// 自动去重：若已有高置信度（≥0.7）且 Jaccard ≥ 0.5 的相似发现，则合并
    /// （保留高置信版本），不重复追加。
    pub fn post(&self, discovery: Discovery) {
        let mut inner = self.inner.lock().unwrap();
        let new_kws = keywords(&discovery.text);
        for existing in &mut inner.facts {
            let e_kws = keywords(&existing.text);
            let j = jaccard(&new_kws, &e_kws);
            if j >= 0.5 && existing.confidence >= 0.7 {
                // 保留高置信版本
                if discovery.confidence > existing.confidence {
                    existing.text = discovery.text;
                    existing.confidence = discovery.confidence;
                    existing.source_id = discovery.source_id;
                }
                return;
            }
        }
        inner.facts.push(discovery);
    }

    /// 重叠检测：检查目标问题是否已被已有发现覆盖。
    ///
    /// 返回 `true` 表示已存在高置信度覆盖，可跳过该子任务。
    pub fn has_overlap(&self, question: &str, threshold: f64) -> bool {
        let inner = self.inner.lock().unwrap();
        let q_kws = keywords(question);
        for fact in &inner.facts {
            if fact.confidence < 0.5 {
                continue;
            }
            let f_kws = keywords(&fact.text);
            if jaccard(&q_kws, &f_kws) >= threshold {
                return true;
            }
        }
        false
    }

    /// 获取与问题最相关的已有发现（作为上下文注入 prompt）。
    pub fn relevant_context(&self, question: &str, max_items: usize) -> Vec<String> {
        let inner = self.inner.lock().unwrap();
        let q_kws = keywords(question);
        let mut scored: Vec<(f64, &str)> = inner
            .facts
            .iter()
            .map(|f| {
                let f_kws = keywords(&f.text);
                let j = jaccard(&q_kws, &f_kws);
                (j * f.confidence, f.text.as_str())
            })
            .filter(|(s, _)| *s > 0.05)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(max_items)
            .map(|(_, t)| t.to_string())
            .collect()
    }

    /// 当前活跃子任务数（并行度）。
    pub fn active_count(&self) -> usize {
        self.inner.lock().unwrap().active_ids.len()
    }

    /// 计算文本关键词集（供外部重叠检测用）。
    pub fn keywords_of(text: &str) -> HashSet<String> {
        keywords(text)
    }

    /// 获取所有发现的快照（克隆，锁持有时间极短）。
    pub fn snapshot(&self) -> Vec<Discovery> {
        self.inner.lock().unwrap().facts.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_and_snapshot() {
        let m = SharedMind::new();
        m.post(Discovery {
            text: "幂等性是指多次调用结果相同".to_string(),
            source_id: "st-1".to_string(),
            confidence: 0.9,
        });
        let snap = m.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].source_id, "st-1");
    }

    #[test]
    fn test_overlap_detection() {
        let m = SharedMind::new();
        m.post(Discovery {
            text: "幂等性是指对同一操作执行一次或多次结果相同".to_string(),
            source_id: "st-1".to_string(),
            confidence: 0.9,
        });
        // 中文二元字 Jaccard 天然偏低（30+ bigram 中交集 2-4 个 ≈ 0.07-0.13）
        // 阈值 0.05 即可检测到同领域重叠
        assert!(m.has_overlap("解释幂等性的定义和应用场景", 0.05));
        // 完全不同领域 → Jaccard ≈ 0
        assert!(!m.has_overlap("量子计算的基本原理和算法", 0.05));
    }

    #[test]
    fn test_overlap_skips_low_confidence() {
        let m = SharedMind::new();
        m.post(Discovery {
            text: "幂等性分析".to_string(),
            source_id: "st-1".to_string(),
            confidence: 0.3, // 低于 0.5 阈值
        });
        assert!(!m.has_overlap("幂等性的定义和应用", 0.3));
    }

    #[test]
    fn test_relevant_context_ranked() {
        let m = SharedMind::new();
        m.post(Discovery {
            text: "量子纠缠是物理现象".to_string(),
            source_id: "st-1".to_string(),
            confidence: 0.8,
        });
        m.post(Discovery {
            text: "幂等性在分布式系统中很重要".to_string(),
            source_id: "st-2".to_string(),
            confidence: 0.9,
        });
        let ctx = m.relevant_context("分布式系统幂等性", 2);
        assert_eq!(ctx.len(), 1); // 只有 st-2 相关
        assert!(ctx[0].contains("幂等性"));
    }

    #[test]
    fn test_active_count() {
        let m = SharedMind::new();
        assert_eq!(m.active_count(), 0);
        m.register_active("st-1");
        m.register_active("st-2");
        assert_eq!(m.active_count(), 2);
        m.unregister_active("st-1");
        assert_eq!(m.active_count(), 1);
    }

    #[test]
    fn test_concurrent_post() {
        use std::thread;
        let topics = [
            "量子纠缠是物理现象",
            "幂等性在分布式系统中很重要",
            "深度学习需要大量标注数据",
            "容器化部署提高了可移植性",
            "函数式编程强调不可变性",
            "索引能显著提升数据库查询速度",
            "消息队列实现了系统间解耦",
            "负载均衡分散了请求压力",
            "缓存减少了重复计算开销",
            "版本控制保障了代码可追溯",
            "微服务架构降低了耦合度",
            "持续集成加快了交付速度",
            "代码审查提升了软件质量",
            "自动化测试减少了回归风险",
            "监控告警缩短了故障响应时间",
            "日志分析帮助定位了根因",
            "灰度发布降低了上线风险",
            "蓝绿部署实现了零停机",
            "熔断机制保护了下游服务",
            "限流策略防止了系统过载",
        ];
        let m = SharedMind::new();
        let m2 = m.clone();
        let h = thread::spawn(move || {
            for i in 0..10 {
                m2.post(Discovery {
                    text: topics[i].to_string(),
                    source_id: format!("st-{i}"),
                    confidence: 0.8,
                });
            }
        });
        for i in 10..20 {
            m.post(Discovery {
                text: topics[i].to_string(),
                source_id: format!("st-{i}"),
                confidence: 0.8,
            });
        }
        h.join().unwrap();
        assert_eq!(m.snapshot().len(), 20);
    }

    #[test]
    fn test_post_dedup_merges_similar() {
        let m = SharedMind::new();
        // 两条高度相似的发现（Jaccard > 0.5）→ 应去重合并
        m.post(Discovery {
            text: "幂等性是指操作可重复执行结果一致".to_string(),
            source_id: "st-1".to_string(),
            confidence: 0.8,
        });
        m.post(Discovery {
            text: "幂等性是操作可重复执行结果保持一致".to_string(),
            source_id: "st-2".to_string(),
            confidence: 0.9, // 更高置信 → 应替换
        });
        let snap = m.snapshot();
        assert_eq!(snap.len(), 1, "相似发现应去重合并");
        assert_eq!(snap[0].confidence, 0.9, "应保留高置信版本");
        assert_eq!(snap[0].source_id, "st-2");
    }

    #[test]
    fn test_post_no_dedup_different() {
        let m = SharedMind::new();
        // 两条不同领域的发现 → 不去重
        m.post(Discovery {
            text: "幂等性是指操作可重复执行".to_string(),
            source_id: "st-1".to_string(),
            confidence: 0.8,
        });
        m.post(Discovery {
            text: "量子纠缠是物理现象".to_string(),
            source_id: "st-2".to_string(),
            confidence: 0.9,
        });
        assert_eq!(m.snapshot().len(), 2);
    }
}
