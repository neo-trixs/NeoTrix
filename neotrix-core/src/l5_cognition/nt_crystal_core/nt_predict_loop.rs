//! NT-PREDICT-LOOP — FEP 预测误差驱动的爬取优先级（AutoExplore 映射）
//!
//! GenieRedux AutoExplore：智能体按世界模型动力学不确定性探索，
//! 无需外部奖励。映射到晶体：
//! - 世界模型 = 同域记忆（对该域的当前认知）；
//! - 预测误差 = 1 − 新内容与同域记忆的最大关键词交叠（surprise）；
//! - 内在好奇心 = 按域平均 surprise 排爬取优先级，写回
//!   `crawl_queue.priority`（消费方已按 priority DESC 认领，零改动对接）。
//!
//! 只读意识做预测，写库只碰 priority 列；无 unwrap / expect / panic。

use rusqlite::{Connection, OpenFlags};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::consciousness::CrystalConsciousness;

/// 预测环（持有各域 surprise 的 EMA = 不确定性地图）
pub struct NtPredictLoop {
    surprise_ema: HashMap<String, f64>,
    alpha: f64,
}

impl NtPredictLoop {
    pub fn new(alpha: f64) -> Self {
        Self {
            surprise_ema: HashMap::new(),
            alpha: alpha.clamp(0.01, 1.0),
        }
    }

    /// 预测误差：1 − 与同域记忆的最大 Jaccard 交叠（采样上限 500 条防 O(n)）
    pub fn surprise(
        consciousness: &CrystalConsciousness,
        content: &str,
        domain: &str,
    ) -> f64 {
        let query: HashSet<String> = CrystalConsciousness::keywords(content)
            .into_iter()
            .collect();
        if query.is_empty() {
            return 0.5;
        }
        let mut best = 0.0f64;
        let mut scanned = 0usize;
        for m in consciousness.memories.values() {
            if m.domain != domain {
                continue;
            }
            if scanned >= 500 {
                break;
            }
            scanned += 1;
            let cand: HashSet<String> =
                CrystalConsciousness::keywords(&m.content).into_iter().collect();
            if cand.is_empty() {
                continue;
            }
            let inter = query.intersection(&cand).count() as f64;
            let union = query.union(&cand).count().max(1) as f64;
            let j = inter / union;
            if j > best {
                best = j;
            }
        }
        (1.0 - best).clamp(0.0, 1.0)
    }

    /// 观测：先按先验算 surprise，再记忆，再更新域 EMA；
    /// 误差超阈则注意力捕获（attention_focus + arousal 上浮）。
    /// 返回 (记忆 id, surprise)。
    pub fn observe(
        &mut self,
        consciousness: &mut CrystalConsciousness,
        content: &str,
        domain: &str,
        confidence: f64,
        capture_threshold: f64,
    ) -> (String, f64) {
        use super::consciousness::MemoryType;
        let s = Self::surprise(consciousness, content, domain);
        let id = consciousness.remember(
            content,
            MemoryType::Fact,
            domain,
            confidence.clamp(0.0, 1.0),
        );
        let ema = self.surprise_ema.get(domain).copied().unwrap_or(s);
        self.surprise_ema
            .insert(domain.to_string(), ema + self.alpha * (s - ema));
        if s >= capture_threshold {
            consciousness.state.attention_focus = Some(domain.to_string());
            consciousness.state.arousal = (consciousness.state.arousal + 0.2).clamp(0.0, 1.0);
        }
        (id, s)
    }

    /// 爬取优先级：EMA 降序（越不确定越先爬）
    pub fn crawl_priorities(&self) -> Vec<(String, f64)> {
        let mut v: Vec<(String, f64)> = self
            .surprise_ema
            .iter()
            .map(|(d, s)| (d.clone(), *s))
            .collect();
        v.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v
    }

    /// 写回队列：将域优先级落 `crawl_queue.priority`（仅 pending 行）。
    /// score 0..1 → priority 0..100。返回更新行数。
    pub fn push_priorities(
        db_path: &Path,
        priorities: &[(String, f64)],
    ) -> Result<usize, String> {
        let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .map_err(|e| format!("open queue db: {e}"))?;
        let mut total = 0usize;
        for (domain, score) in priorities {
            let p = (score.clamp(0.0, 1.0) * 100.0) as i64;
            let n = conn
                .execute(
                    "UPDATE crawl_queue SET priority = ?1 WHERE domain = ?2 AND status = 'pending'",
                    rusqlite::params![p, domain],
                )
                .map_err(|e| format!("update priority: {e}"))?;
            total += n as usize;
        }
        Ok(total)
    }

    /// 域 EMA 查询（测试与外部调度用）
    pub fn domain_uncertainty(&self, domain: &str) -> f64 {
        self.surprise_ema.get(domain).copied().unwrap_or(0.0)
    }
}

impl Default for NtPredictLoop {
    fn default() -> Self {
        Self::new(0.3)
    }
}

#[cfg(test)]
mod tests {
    use super::super::consciousness::MemoryType;
    use super::*;

    fn seeded() -> CrystalConsciousness {
        // 注：中文无空格分词，关键词以空格分隔书写
        let mut c = CrystalConsciousness::new("t");
        c.remember("火焰 燃烧 释放 大量 热量", MemoryType::Fact, "physics", 0.9);
        c.remember("火焰 温度 融化 钢铁", MemoryType::Fact, "physics", 0.9);
        c.remember("光合作用 需要 叶绿素", MemoryType::Fact, "bio", 0.9);
        c
    }

    #[test]
    fn test_surprise_familiar_low_novel_high() {
        let c = seeded();
        let low = NtPredictLoop::surprise(&c, "火焰 燃烧 产生 热量", "physics");
        let high = NtPredictLoop::surprise(&c, "量子 纠缠 贝尔 不等式", "physics");
        assert!(low < high, "familiar={low} novel={high}");
        assert!(low < 0.9);
        assert!(high > 0.5);
    }

    #[test]
    fn test_surprise_empty_query_neutral() {
        let c = seeded();
        assert!((NtPredictLoop::surprise(&c, "的", "physics") - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_observe_captures_attention_on_shock() {
        let mut c = seeded();
        let before = c.state.arousal;
        let mut lp = NtPredictLoop::default();
        let (_, s) = lp.observe(
            &mut c,
            "弦理论预言十一维时空结构",
            "physics",
            0.7,
            0.6,
        );
        assert!(s >= 0.6);
        assert_eq!(c.state.attention_focus.as_deref(), Some("physics"));
        assert!(c.state.arousal > before);
    }

    #[test]
    fn test_priorities_rank_uncertain_first() {
        let mut c = seeded();
        let mut lp = NtPredictLoop::default();
        lp.observe(&mut c, "火焰 燃烧 很热", "physics", 0.8, 2.0); // 熟悉，低惊奇
        lp.observe(&mut c, "叶绿素 捕获 光能", "bio", 0.8, 2.0); // 半熟悉
        lp.observe(&mut c, "暗物质 只 参与 引力", "astro", 0.8, 2.0); // 全新域
        let ranks = lp.crawl_priorities();
        assert_eq!(ranks.first().map(|(d, _)| d.as_str()), Some("astro"));
    }

    #[test]
    fn test_push_priorities_roundtrip_on_temp_db() {
        let dir = std::env::temp_dir().join("nt_predict_loop_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let db = dir.join("q.db");
        {
            let conn = Connection::open(&db).unwrap();
            conn.execute_batch(
                "CREATE TABLE crawl_queue (id TEXT, url TEXT, depth INT, domain TEXT, priority INT, status TEXT, discovered_at INT, last_attempt INT, retry_count INT, error_message TEXT); \
                 INSERT INTO crawl_queue VALUES ('1','u1',1,'astro',1,'pending',0,0,0,NULL); \
                 INSERT INTO crawl_queue VALUES ('2','u2',1,'bio',1,'done',0,0,0,NULL);",
            )
            .unwrap();
        }
        let n = NtPredictLoop::push_priorities(&db, &[("astro".to_string(), 0.9), ("bio".to_string(), 0.1)]).unwrap();
        assert_eq!(n, 1, "only pending astro row updates");
        let conn = Connection::open(&db).unwrap();
        let p: i64 = conn
            .query_row("SELECT priority FROM crawl_queue WHERE id='1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(p, 90);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_push_priorities_missing_db_errors() {
        let r = NtPredictLoop::push_priorities(
            std::path::Path::new("/nonexistent-xyz/q.db"),
            &[("a".to_string(), 0.5)],
        );
        assert!(r.is_err());
    }
}
