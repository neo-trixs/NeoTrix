//! L6 / NT-ACT — cashclaw (github.com/moltlaunch/cashclaw) 吸收节点 (C1)。
//!
//! 源: cashclaw — 自治工作智能体。完整闭环: 接单 → 评估/报价 → 执行 → 交付 →
//! 收费 → 评分反馈 → 自学习。记忆层用 BM25 倒排索引 + study sessions (带时间衰减),
//! 对齐 NeoTrix experience-tree 经验吸收闭环 (SelfTest 3D 覆盖 + 持久化)。
//!
//! 本节点只落地 C1 (存在级 + 单测) 基础逻辑, 真实 LLM/支付/网络侧走 stub, 待后续
//! C2 接线到生产路径 (R-P79: 不延期死代码)。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// 任务单: 客户发布的工作请求。
#[derive(Debug, Clone, PartialEq)]
pub struct WorkOrder {
    pub id: String,
    pub title: String,
    pub description: String,
    /// 客户预算上限 (货币单位)。
    pub budget: u64,
}

/// 报价结果: 智能体对任务的评估与出价。
#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    pub accepted: bool,
    /// 出价金额; 超过预算则 accepted=false。
    pub price: u64,
    pub rationale: String,
}

/// 交付结果: 执行完成后的产出摘要。
#[derive(Debug, Clone, PartialEq)]
pub struct Delivery {
    pub order_id: String,
    pub artifact: String,
    pub success: bool,
}

/// 评分反馈: 客户对交付的打分 (1-5)。
#[derive(Debug, Clone, PartialEq)]
pub struct Review {
    pub order_id: String,
    pub rating: u8,
    pub comment: String,
}

/// 自学习记忆接口 (stub): BM25 倒排索引 + study sessions + 时间衰减。
pub trait SelfLearningMemory {
    /// 索引一条经验文本 (带时间戳)。
    fn index(&mut self, text: &str, ts: u64);
    /// BM25 检索 top-k 相关经验 (stub: 走词频 + 时间衰减)。
    fn recall(&self, query: &str, k: usize) -> Vec<String>;
    /// 运行一次 study session: 重放索引经验, 返回被巩固的条目数。
    fn study_session(&mut self) -> usize;
}

/// 自治工作智能体: 接单→报价→执行→交付→收费→评分 闭环。
pub trait AutonomousWorkAgent {
    fn accept(&mut self, order: WorkOrder) -> Quote;
    fn deliver(&mut self, order_id: &str, artifact: &str) -> Delivery;
    fn charge(&self, order_id: &str) -> Option<u64>;
    fn receive_review(&mut self, review: Review);
    fn avg_rating(&self) -> f64;
}

/// 默认 BM25 记忆: 词级倒排 + 指数时间衰减。
pub struct Bm25Memory {
    /// term -> (doc_id 集合, 每个 doc 词频)
    inverted: HashMap<String, HashMap<usize, u64>>,
    docs: Vec<(String, u64)>,
    /// 学习率: 衰减半生命周期 (时间单位)。
    decay_halflife: u64,
}

impl Bm25Memory {
    pub fn new(decay_halflife: u64) -> Self {
        Self {
            inverted: HashMap::new(),
            docs: Vec::new(),
            decay_halflife,
        }
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string())
            .collect()
    }

    /// 时间衰减权重: 越近的文档权重越高。
    fn decay(&self, ts: u64, now: u64) -> f64 {
        if self.decay_halflife == 0 {
            return 1.0;
        }
        let age = now.saturating_sub(ts);
        (0.5f64).powf(age as f64 / self.decay_halflife as f64)
    }
}

impl SelfLearningMemory for Bm25Memory {
    fn index(&mut self, text: &str, ts: u64) {
        let doc_id = self.docs.len();
        self.docs.push((text.to_string(), ts));
        for t in Self::tokenize(text) {
            let entry = self.inverted.entry(t).or_default();
            *entry.entry(doc_id).or_default() += 1;
        }
    }

    fn recall(&self, query: &str, k: usize) -> Vec<String> {
        let now = self
            .docs
            .last()
            .map(|d| d.1)
            .unwrap_or(0);
        let q_tokens = Self::tokenize(query);
        let n = self.docs.len().max(1) as f64;
        let mut scores: Vec<(usize, f64)> = Vec::new();
        for (doc_id, (_text, ts)) in self.docs.iter().enumerate() {
            let mut score = 0.0;
            for qt in &q_tokens {
                if let Some(postings) = self.inverted.get(qt) {
                    let tf = *postings.get(&doc_id).unwrap_or(&0) as f64;
                    let df = postings.len() as f64;
                    if tf > 0.0 {
                        let idf = (n - df + 0.5) / (df + 0.5);
                        score += (tf * (1.0 + 1.0) / (tf + 1.0)) * idf;
                    }
                }
            }
            if score > 0.0 {
                score *= self.decay(*ts, now);
                scores.push((doc_id, score));
            }
        }
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);
        scores
            .into_iter()
            .map(|(id, _)| self.docs[id].0.clone())
            .collect()
    }

    fn study_session(&mut self) -> usize {
        // stub: 重新索引巩固 (此处无额外动作, 仅返回已掌握条目数)
        self.docs.len()
    }
}

/// 现金爪智能体: 维护接单状态 + 评分 + 自学习记忆。
pub struct CashClaw {
    /// order_id -> 已接受报价价
    accepted: HashMap<String, u64>,
    reviews: Vec<Review>,
    memory: Bm25Memory,
}

impl CashClaw {
    pub fn new(memory_halflife: u64) -> Self {
        Self {
            accepted: HashMap::new(),
            reviews: Vec::new(),
            memory: Bm25Memory::new(memory_halflife),
        }
    }

    /// 取出自学习记忆 (供 study sessions 调用)。
    pub fn memory(&mut self) -> &mut Bm25Memory {
        &mut self.memory
    }

    fn is_within_budget(order: &WorkOrder, price: u64) -> bool {
        price <= order.budget
    }
}

impl AutonomousWorkAgent for CashClaw {
    /// 接单评估: 基础价 = 描述长度阶梯; 超预算则拒绝。
    fn accept(&mut self, order: WorkOrder) -> Quote {
        let base = 50 + order.description.len().min(450) as u64;
        let within = Self::is_within_budget(&order, base);
        if within {
            self.accepted.insert(order.id.clone(), base);
            Quote {
                accepted: true,
                price: base,
                rationale: format!("base price for {}ch description", order.description.len()),
            }
        } else {
            Quote {
                accepted: false,
                price: base,
                rationale: "over budget".into(),
            }
        }
    }

    /// 交付: 仅对先前接下的单生成成果; 同时把经验写入自学习记忆。
    fn deliver(&mut self, order_id: &str, artifact: &str) -> Delivery {
        let success = self.accepted.contains_key(order_id);
        if success {
            self.memory
                .index(&format!("delivered {artifact} for {order_id}"), now_ts());
        }
        Delivery {
            order_id: order_id.to_string(),
            artifact: artifact.to_string(),
            success,
        }
    }

    /// 收费: 交付成功的单按约定价收款。
    fn charge(&self, order_id: &str) -> Option<u64> {
        self.accepted.get(order_id).copied()
    }

    fn receive_review(&mut self, review: Review) {
        self.memory.index(
            &format!("review {}: {}", review.rating, review.comment),
            now_ts(),
        );
        self.reviews.push(review);
    }

    fn avg_rating(&self) -> f64 {
        if self.reviews.is_empty() {
            return 0.0;
        }
        let sum: u32 = self.reviews.iter().map(|r| r.rating as u32).sum();
        sum as f64 / self.reviews.len() as f64
    }
}

/// 单调时钟戳 (stub: 用系统时间秒; 真实环境可换 NT-MEMORY 时钟)。
fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Default)]
pub struct CashClawSelfTest;

impl SelfTest for CashClawSelfTest {
    fn name(&self) -> &str {
        "nt_act_cashclaw"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut agent = CashClaw::new(3600);
        let mut errs = Vec::new();

        // 接单 → 报价 → 交付 → 收费 → 评分 闭环
        let order = WorkOrder {
            id: "o1".into(),
            title: "build a scraper".into(),
            description: "scrape product prices daily".into(),
            budget: 100,
        };
        let quote = agent.accept(order);
        if !quote.accepted {
            errs.push("cashclaw: order rejected unexpectedly".into());
        }
        let delivery = agent.deliver("o1", "scraper.py");
        if !delivery.success {
            errs.push("cashclaw: delivery failed for accepted order".into());
        }
        if agent.charge("o1") != Some(quote.price) {
            errs.push("cashclaw: charge mismatch".into());
        }
        agent.receive_review(Review {
            order_id: "o1".into(),
            rating: 5,
            comment: "great work".into(),
        });
        if (agent.avg_rating() - 5.0).abs() > 1e-6 {
            errs.push("cashclaw: rating not recorded".into());
        }

        // 自学习记忆 BM25 recall
        let mut mem = Bm25Memory::new(3600);
        mem.index("scrape product prices", 100);
        mem.index("write unit tests", 200);
        let hits = mem.recall("scrape prices", 1);
        if !hits.iter().any(|h| h.contains("scrape")) {
            errs.push("cashclaw: bm25 recall missed relevant doc".into());
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_loop_accept_deliver_charge_review() {
        let mut agent = CashClaw::new(3600);
        let q = agent.accept(WorkOrder {
            id: "x".into(),
            title: "t".into(),
            description: "do the thing now".into(),
            budget: 200,
        });
        assert!(q.accepted);
        let d = agent.deliver("x", "result.txt");
        assert!(d.success);
        assert_eq!(agent.charge("x"), Some(q.price));
        agent.receive_review(Review {
            order_id: "x".into(),
            rating: 4,
            comment: "ok".into(),
        });
        assert!((agent.avg_rating() - 4.0).abs() < 1e-6);
    }

    #[test]
    fn over_budget_rejected() {
        let mut agent = CashClaw::new(3600);
        let q = agent.accept(WorkOrder {
            id: "y".into(),
            title: "t".into(),
            description: "x".into(),
            budget: 1,
        });
        assert!(!q.accepted);
        // 未接单的交付应失败
        let d = agent.deliver("y", "z");
        assert!(!d.success);
        assert_eq!(agent.charge("y"), None);
    }

    #[test]
    fn bm25_recall_ranks_relevant_first() {
        let mut mem = Bm25Memory::new(1000);
        mem.index("scrape product prices daily", 100);
        mem.index("compile rust binary", 200);
        mem.index("scrape news headlines", 150);
        let hits = mem.recall("scrape prices", 3);
        assert!(!hits.is_empty());
        assert!(hits[0].contains("scrape"));
    }
}
