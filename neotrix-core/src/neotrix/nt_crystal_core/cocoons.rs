//! 持久记忆茧 — 跨会话持久化
//!
//! Cocoon 是记忆的持久化单元，按领域组织，支持衰减、巩固和合并。
//! 提供从 CrystalConsciousness 双向同步的能力。

use super::consciousness::{CrystalConsciousness, Memory, ReasoningChain};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// 持久记忆茧 — 跨会话持久化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentCocoon {
    pub id: String,
    pub memories: Vec<Memory>,
    pub retention_policy: RetentionPolicy,
    pub meta_metrics: CocoonMetrics,
    pub last_accessed: u64,
    pub created_at: u64,
}

/// 策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub max_memories: usize,
    pub min_strength: f64,
    pub domain_weights: HashMap<String, f64>,
    pub decay_strategy: DecayStrategy,
}

/// 衰减策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecayStrategy {
    Exponential { rate: f64 },
    Linear { step: f64 },
    StepFunction { thresholds: Vec<(f64, f64)> },
}

/// 茧性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocoonMetrics {
    pub total_recall_attempts: u64,
    pub successful_recalls: u64,
    pub avg_relevance_score: f64,
    pub memory_utilization: f64,
}

/// 推理链落盘上限（防无界增长）。
///
/// 链是**只增**的（每次 `reason()` 追加），而盘是单一 JSON 文件 —— 不封顶会
/// 无限膨胀，且 `load()` 是全量 `read_to_string` + `from_str`，直接变成启动成本。
/// 保留**最近 N 条**（链按产生序 append，故尾部即最新）。
pub const REASONING_CHAIN_CAP: usize = 5000;

/// 截断到上限（超限则丢最旧的）。
pub fn cap_chains(mut chains: Vec<ReasoningChain>) -> Vec<ReasoningChain> {
    if chains.len() > REASONING_CHAIN_CAP {
        let drop_n = chains.len() - REASONING_CHAIN_CAP;
        chains.drain(..drop_n);
    }
    chains
}

/// 同步结果计数（可观测性：去重闸不能是静默的）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyncReport {
    /// 涉及的领域数
    pub domains: usize,
    /// 触及的茧数
    pub cocoons_touched: usize,
    /// 实际写入的记忆数
    pub inserted: usize,
    /// 因 id 相同被拒
    pub skipped_same_id: usize,
    /// 因正文相同、id 不同被拒（**新闸命中的就是这一项**）
    pub skipped_same_content: usize,
}

impl SyncReport {
    /// 被拒总数（两个闸之和）
    pub fn skipped(&self) -> usize {
        self.skipped_same_id + self.skipped_same_content
    }
}

/// 茧存储管理器
pub struct CocoonStore {
    pub cocoons: HashMap<String, PersistentCocoon>,
    pub store_path: PathBuf,
    pub strategy: RetentionPolicy,
    /// 推理链（跨会话存活）。此前只活在 `CrystalConsciousness` 内存里，
    /// 重启即丢 —— 而 1h tick 的校准信号正是读它，链不落盘则校准器跨进程无历史。
    pub chains: Vec<ReasoningChain>,
}

impl CocoonStore {
    /// 创建新的茧存储管理器
    pub fn new(store_path: PathBuf) -> Self {
        let default_strategy = RetentionPolicy {
            max_memories: 1000,
            min_strength: 0.1,
            domain_weights: HashMap::new(),
            decay_strategy: DecayStrategy::Exponential { rate: 0.01 },
        };

        Self {
            cocoons: HashMap::new(),
            store_path,
            strategy: default_strategy,
            chains: Vec::new(),
        }
    }

    /// 从磁盘加载
    pub fn load() -> Self {
        let store_path = Self::default_store_path();
        if !store_path.exists() {
            return Self::new(store_path);
        }

        let content = match std::fs::read_to_string(&store_path) {
            Ok(c) => c,
            Err(_) => return Self::new(store_path),
        };

        let data: StoreData = match serde_json::from_str(&content) {
            Ok(d) => d,
            Err(_) => StoreData {
                cocoons: HashMap::new(),
                strategy: RetentionPolicy::default(),
                chains: Vec::new(),
            },
        };

        Self {
            cocoons: data.cocoons,
            store_path,
            strategy: data.strategy,
            chains: cap_chains(data.chains),
        }
    }

    /// 保存到磁盘
    pub fn save(&self) -> Result<(), String> {
        let data = StoreData {
            cocoons: self.cocoons.clone(),
            strategy: self.strategy.clone(),
            chains: cap_chains(self.chains.clone()),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        // R-P0-2 原子写：tmp + rename。原先是裸 `fs::write` —— 写到一半断电/
        // 满盘会留下**半截 51MB 盘**，而 `load()` 解析失败即静默返回空 store
        // （等于全库归零且无报错）。改成同目录 tmp + `rename`（同 fs 原子）。
        let tmp = self.store_path.with_extension("json.tmp");
        std::fs::write(&tmp, json.as_bytes()).map_err(|e| format!("Failed to write tmp: {}", e))?;
        std::fs::rename(&tmp, &self.store_path).map_err(|e| format!("Failed to rename: {}", e))
    }

    /// 创建新茧
    pub fn create_cocoon(&mut self, domain: &str) -> String {
        let id = format!("cocoon-{}-{}", domain, timestamp_now());
        let now = timestamp_now();

        let cocoon = PersistentCocoon {
            id: id.clone(),
            memories: Vec::new(),
            retention_policy: self.strategy.clone(),
            meta_metrics: CocoonMetrics::default(),
            last_accessed: now,
            created_at: now,
        };

        self.cocoons.insert(id.clone(), cocoon);
        id
    }

    /// 存储记忆到指定茧
    pub fn store_memory(&mut self, cocoon_id: &str, memory: Memory) -> Result<(), String> {
        let cocoon = self
            .cocoons
            .get_mut(cocoon_id)
            .ok_or_else(|| format!("Cocoon not found: {}", cocoon_id))?;

        // 检查容量限制
        if cocoon.memories.len() >= cocoon.retention_policy.max_memories {
            return Err("Cocoon memory capacity reached".into());
        }

        // 检查强度阈值
        if memory.strength < cocoon.retention_policy.min_strength {
            return Err("Memory strength below threshold".into());
        }

        cocoon.memories.push(memory);
        cocoon.last_accessed = timestamp_now();
        Ok(())
    }

    /// 按领域和查询召回记忆
    pub fn recall(&self, domain: &str, query: &str, limit: usize) -> Vec<Memory> {
        let mut recalled: Vec<Memory> = self
            .cocoons
            .values()
            .filter(|_c| {
                let domain_weight = self
                    .strategy
                    .domain_weights
                    .get(domain)
                    .copied()
                    .unwrap_or(1.0);
                domain_weight > 0.0
            })
            .flat_map(|c| &c.memories)
            .filter(|m| m.domain == domain)
            .filter(|m| {
                // 简单的查询匹配
                query.is_empty() || m.content.to_lowercase().contains(&query.to_lowercase())
            })
            .cloned()
            .collect();

        // 按强度和访问次数排序
        recalled.sort_by(|a, b| {
            let score_a = a.strength * (1.0 + a.access_count as f64 * 0.1);
            let score_b = b.strength * (1.0 + b.access_count as f64 * 0.1);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        recalled.truncate(limit);
        recalled
    }

    /// 合并小茧
    pub fn consolidate(&mut self) {
        let min_memories = 10;
        let mut to_merge: Vec<String> = Vec::new();
        let mut domain_groups: HashMap<String, Vec<String>> = HashMap::new();

        // 按领域分组
        for (id, _cocoon) in &self.cocoons {
            // 从茧ID中提取领域（假设格式为 cocoon-{domain}-{timestamp}）
            let domain = id.split('-').nth(1).unwrap_or("unknown").to_string();
            domain_groups.entry(domain).or_default().push(id.clone());
        }

        // 找出需要合并的茧
        for (_domain, ids) in &domain_groups {
            let small_cocoons: Vec<String> = ids
                .iter()
                .filter(|id| {
                    self.cocoons
                        .get(*id)
                        .map_or(false, |c| c.memories.len() < min_memories)
                })
                .cloned()
                .collect();

            if small_cocoons.len() > 1 {
                to_merge.extend(small_cocoons);
            }
        }

        if to_merge.is_empty() {
            return;
        }

        // 收集所有需要合并的记忆
        let mut merged_memories: Vec<Memory> = Vec::new();
        for id in &to_merge {
            if let Some(cocoon) = self.cocoons.remove(id) {
                merged_memories.extend(cocoon.memories);
            }
        }

        // 创建新茧
        let now = timestamp_now();
        let new_id = format!("cocoon-merged-{}", now);
        let new_cocoon = PersistentCocoon {
            id: new_id.clone(),
            memories: merged_memories,
            retention_policy: self.strategy.clone(),
            meta_metrics: CocoonMetrics::default(),
            last_accessed: now,
            created_at: now,
        };

        self.cocoons.insert(new_id, new_cocoon);
    }

    /// 淘汰弱记忆
    pub fn prune(&mut self) {
        let now = timestamp_now();
        for cocoon in self.cocoons.values_mut() {
            cocoon.memories.retain(|m| {
                // 保留满足以下条件的记忆：
                // 1. 强度大于阈值
                // 2. 最近被访问过（30天内）
                let age_days = (now - m.last_accessed) as f64 / 86400.0;
                m.strength >= cocoon.retention_policy.min_strength && age_days < 30.0
            });
        }

        // 移除空茧
        self.cocoons.retain(|_, c| !c.memories.is_empty());
    }

    /// 从晶体意识同步数据到茧
    pub fn sync_from_consciousness(&mut self, consciousness: &CrystalConsciousness) {
        let _ = self.sync_from_consciousness_counted(consciousness);
    }

    /// 从晶体意识同步数据到茧（带可观测计数 + 内容去重闸）。
    ///
    /// # 内容去重闸（RFC §5 待接第 4 项，2026-09-28 落地）
    ///
    /// 原实现只按 `m.id == memory.id` 去重。id 分配一旦出 bug（历史上确实出过：
    /// 多个吸收脚本各自算 start id、互不加锁，致 1,645 个重号 id / 2,680 条
    /// 记忆被 HashMap 静默遮蔽），同一条记忆就会以**不同 id** 重复进茧，
    /// 而 id 检查抓不到。此闸按**内容**兜第二道防线。
    ///
    /// 刻意**不用** `nt_train_export::content_hash` 的 u64 指纹：指纹碰撞会
    /// 静默丢掉一条**不同**的记忆 —— 正是本闸要防的失败模式。`HashSet<&str>`
    /// 按 Eq 复核，无假阳性。既有正文索引**零克隆**（借用 `cocoon`）；仅本轮
    /// 新增正文入 `seen_new` 时克隆一次，量级 = 新增条数，不是整茧规模。
    /// 判定与写入分离（两阶段），避免逐条重扫全茧的 O(n²)。
    pub fn sync_from_consciousness_counted(
        &mut self,
        consciousness: &CrystalConsciousness,
    ) -> SyncReport {
        let mut report = SyncReport::default();
        // 按领域组织记忆
        let mut domain_memories: HashMap<String, Vec<Memory>> = HashMap::new();

        for memory in consciousness.memories.values() {
            domain_memories
                .entry(memory.domain.clone())
                .or_default()
                .push(memory.clone());
        }
        report.domains = domain_memories.len();

        // 为每个领域创建或更新茧
        for (domain, memories) in domain_memories {
            // 查找现有茧或创建新的
            let cocoon_id = self
                .cocoons
                .values()
                .find(|c| c.id.starts_with(&format!("cocoon-{}-", domain)))
                .map(|c| c.id.clone())
                .unwrap_or_else(|| self.create_cocoon(&domain));
            report.cocoons_touched += 1;

            if let Some(cocoon) = self.cocoons.get_mut(&cocoon_id) {
                // 两阶段：先把既有 id / 正文一次性建成**借用**索引（判定完即释放），
                // 通过判定的攒进 to_push，最后一次性 extend。
                // 旧写法有两个真问题（编译期 + 运行期各一）：
                //   1) `seen: HashSet<&str>` 插入 `memory.content.as_str()`，而
                //      `memory` 是每轮就析构的**局部所有权** → 借用活不过本轮（E0597）；
                //   2) 每轮重建 seen、且每条记忆都 `any()` 线性扫全茧 → O(n²)。
                // 既有索引仍零克隆；只有**本轮新增**的正文入 seen_new，
                // 克隆量级 = 新增条数（而非整茧规模），故可忽略。
                let existing_content: HashSet<&str> =
                    cocoon.memories.iter().map(|m| m.content.as_str()).collect();
                let existing_id: HashSet<&str> =
                    cocoon.memories.iter().map(|m| m.id.as_str()).collect();
                let mut to_push: Vec<Memory> = Vec::new();
                let mut seen_new: HashSet<String> = HashSet::new();
                for memory in memories {
                    if existing_id.contains(memory.id.as_str()) {
                        report.skipped_same_id += 1;
                        continue;
                    }
                    if existing_content.contains(memory.content.as_str())
                        || !seen_new.insert(memory.content.clone())
                    {
                        report.skipped_same_content += 1;
                        continue;
                    }
                    to_push.push(memory);
                }
                report.inserted += to_push.len();
                cocoon.memories.extend(to_push);
            }
        }
        report
    }

    /// 同步茧数据到晶体意识
    pub fn sync_to_consciousness(&self, consciousness: &mut CrystalConsciousness) {
        for cocoon in self.cocoons.values() {
            for memory in &cocoon.memories {
                // 只同步意识中不存在的记忆
                if !consciousness.memories.contains_key(&memory.id) {
                    consciousness
                        .memories
                        .insert(memory.id.clone(), memory.clone());
                }
            }
        }
        // 链回填：盘上链灌进意识体（此前只在内存，跨会话全丢）。
        // 按 id 去重，意识体已有的链优先保留（意识体是本会话更新的那份）。
        for ch in &self.chains {
            if !consciousness.reasoning_chains.iter().any(|c| c.id == ch.id) {
                consciousness.reasoning_chains.push(ch.clone());
            }
        }
        // 直接灌入绕过 connect()，重算连接计数并刷新相位
        consciousness.recount_connections();
    }

    /// 记录一条新链（去重 + 封顶），供上层在 `reason()` 成功后调用。
    pub fn record_chain(&mut self, chain: ReasoningChain) {
        if !self.chains.iter().any(|c| c.id == chain.id) {
            self.chains.push(chain);
        }
        self.chains = cap_chains(std::mem::take(&mut self.chains));
    }

    /// 获取所有茧的统计信息
    pub fn stats(&self) -> StoreStats {
        let total_memories: usize = self.cocoons.values().map(|c| c.memories.len()).sum();

        let domain_counts: HashMap<String, usize> = self
            .cocoons
            .values()
            .flat_map(|c| &c.memories)
            .fold(HashMap::new(), |mut acc, m| {
                *acc.entry(m.domain.clone()).or_insert(0) += 1;
                acc
            });

        StoreStats {
            cocoon_count: self.cocoons.len(),
            total_memories,
            domain_counts,
        }
    }

    /// 默认存储路径
    fn default_store_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".neotrix")
            .join("crystal_core")
            .join("cocoons.json")
    }
}

/// 存储数据（用于序列化）
#[derive(Debug, Serialize, Deserialize)]
struct StoreData {
    cocoons: HashMap<String, PersistentCocoon>,
    strategy: RetentionPolicy,
    /// 推理链落盘（2026-09-28 新增）。
    ///
    /// `#[serde(default)]` 是**硬约束**，不是便利：历史盘（303M 旧茧、以及
    /// 任何在本字段加入前写出的文件）都没有 `chains` 键。缺 `default` 会让
    /// 整盘 `serde_json::from_str` 失败 → 撞上 `load()` 的 `Err(_)` 分支 →
    /// **静默返回全空 store**（即 2026-09-28 修掉的 CrossDomain 静默归零地，
    /// 只是换了个触发原因）。故此处绝不能去掉 default。
    #[serde(default)]
    chains: Vec<ReasoningChain>,
}

/// 存储统计信息
#[derive(Debug)]
pub struct StoreStats {
    pub cocoon_count: usize,
    pub total_memories: usize,
    pub domain_counts: HashMap<String, usize>,
}

/// 默认实现
impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_memories: 1000,
            min_strength: 0.1,
            domain_weights: HashMap::new(),
            decay_strategy: DecayStrategy::Exponential { rate: 0.01 },
        }
    }
}

impl Default for CocoonMetrics {
    fn default() -> Self {
        Self {
            total_recall_attempts: 0,
            successful_recalls: 0,
            avg_relevance_score: 0.0,
            memory_utilization: 0.0,
        }
    }
}

/// 时间戳生成
fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl std::fmt::Display for CocoonStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.stats();
        writeln!(f, "=== Cocoon Store ===")?;
        writeln!(f, "Store path: {}", self.store_path.display())?;
        writeln!(f, "Cocoons: {}", stats.cocoon_count)?;
        writeln!(f, "Total memories: {}", stats.total_memories)?;
        writeln!(f, "Domains: {:?}", stats.domain_counts)?;
        Ok(())
    }
}

#[cfg(test)]
mod sync_dedup_tests {
    use super::*;
    use crate::neotrix::nt_crystal_core::consciousness::MemoryType;

    fn mem(id: &str, domain: &str, content: &str) -> Memory {
        Memory {
            id: id.to_string(),
            content: content.to_string(),
            memory_type: MemoryType::Fact,
            domain: domain.to_string(),
            strength: 1.0,
            confidence: 0.8,
            importance: 0.5,
            connections: Vec::new(),
            created_at: 0,
            last_accessed: 0,
            access_count: 0,
        }
    }

    fn consciousness_of(ms: Vec<Memory>) -> CrystalConsciousness {
        let mut c = CrystalConsciousness::new("dedup-test");
        for m in ms {
            c.memories.insert(m.id.clone(), m);
        }
        c
    }

    /// 内容去重闸：id 不同但正文相同 → 必须拒（RFC §5 待接第 4 项）
    #[test]
    fn content_dedup_catches_same_body_different_id() {
        let mut store = CocoonStore::new(std::path::PathBuf::from("/tmp/nt-dedup-1.json"));
        let c = consciousness_of(vec![
            mem("M-000001", "d1", "同一段正文"),
            mem("M-000002", "d1", "同一段正文"), // id 不同，正文相同
            mem("M-000003", "d1", "另一段正文"),
        ]);
        let r = store.sync_from_consciousness_counted(&c);
        assert_eq!(r.inserted, 2, "只应写入 2 条（第三条正文重复）");
        assert_eq!(r.skipped_same_content, 1, "新闸必须命中 1 条");
        assert_eq!(r.skipped_same_id, 0);
        let total: usize = store.cocoons.values().map(|x| x.memories.len()).sum();
        assert_eq!(total, 2, "茧内不得有正文重复");
    }

    /// 幂等：同一意识体同步两次，第二次必须 0 写入
    #[test]
    fn sync_is_idempotent() {
        let mut store = CocoonStore::new(std::path::PathBuf::from("/tmp/nt-dedup-2.json"));
        let c = consciousness_of(vec![
            mem("M-000001", "d1", "甲"),
            mem("M-000002", "d1", "乙"),
        ]);
        let r1 = store.sync_from_consciousness_counted(&c);
        assert_eq!((r1.inserted, r1.skipped()), (2, 0));
        let r2 = store.sync_from_consciousness_counted(&c);
        assert_eq!(r2.inserted, 0, "第二次不得再写入");
        assert_eq!(r2.skipped_same_id, 2, "应全被 id 闸拦下");
        assert_eq!(r2.skipped(), 2);
    }

    /// 旧签名必须仍可用（不破坏既有调用方）
    #[test]
    fn legacy_signature_still_works() {
        let mut store = CocoonStore::new(std::path::PathBuf::from("/tmp/nt-dedup-3.json"));
        let c = consciousness_of(vec![mem("M-000001", "d1", "甲")]);
        store.sync_from_consciousness(&c);
        let total: usize = store.cocoons.values().map(|x| x.memories.len()).sum();
        assert_eq!(total, 1);
    }

    /// 正文相同但**跨域** → 必须写入（域是路由键，跨域同文不是重复）
    #[test]
    fn same_body_across_domains_is_kept() {
        let mut store = CocoonStore::new(std::path::PathBuf::from("/tmp/nt-dedup-4.json"));
        let c = consciousness_of(vec![
            mem("M-000001", "d1", "共享正文"),
            mem("M-000002", "d2", "共享正文"),
        ]);
        let r = store.sync_from_consciousness_counted(&c);
        assert_eq!(r.inserted, 2, "跨域同文不得误杀");
        assert_eq!(r.cocoons_touched, 2, "应落到两个茧");
    }

    /// 空意识体不得 panic
    #[test]
    fn empty_consciousness_is_safe() {
        let mut store = CocoonStore::new(std::path::PathBuf::from("/tmp/nt-dedup-5.json"));
        let c = CrystalConsciousness::new("empty");
        let r = store.sync_from_consciousness_counted(&c);
        assert_eq!((r.inserted, r.domains, r.cocoons_touched), (0, 0, 0));
    }
}

#[cfg(test)]
mod chain_persist_tests {
    use super::*;
    use crate::neotrix::nt_crystal_core::consciousness::{MemoryType, ReasoningType};

    fn chain(id: &str, concl: &str) -> ReasoningChain {
        ReasoningChain {
            id: id.to_string(),
            premises: vec!["M-000001".to_string()],
            conclusion: concl.to_string(),
            conclusion_memory_id: Some("M-000002".to_string()),
            chain_type: ReasoningType::Deductive,
            confidence: 0.7,
        }
    }

    /// 旧格式盘（**无 `chains` 键**）必须仍能加载 —— 303M 历史盘的硬约束。
    /// 若缺 `#[serde(default)]`，此处会 parse 失败 → load() 静默返回空 store。
    #[test]
    fn legacy_store_without_chains_key_still_loads() {
        let legacy = r#"{
  "cocoons": {
    "cocoon-d1-1": {
      "id": "cocoon-d1-1",
      "memories": [
        {"id":"M-000001","content":"甲","memory_type":"Fact","domain":"d1",
         "strength":1.0,"confidence":0.8,"importance":0.5,"connections":[],
         "created_at":1,"last_accessed":1,"access_count":0}
      ],
      "retention_policy":{"max_memories":1000,"min_strength":0.1,
        "domain_weights":{},"decay_strategy":{"Exponential":{"rate":0.01}}},
      "meta_metrics":{"total_recall_attempts":0,"successful_recalls":0,
        "avg_relevance_score":0.0,"memory_utilization":0.0},
      "last_accessed":1,"created_at":1
    }
  },
  "strategy": {"max_memories":1000,"min_strength":0.1,"domain_weights":{},
    "decay_strategy":{"Exponential":{"rate":0.01}}}
}"#;
        let d: StoreData = serde_json::from_str(legacy).expect("旧格式必须可解析");
        assert_eq!(d.cocoons.len(), 1, "旧盘的记忆必须读得到");
        assert!(d.chains.is_empty(), "缺 chains 键 → default 空 vec");
        assert_eq!(
            d.cocoons.get("cocoon-d1-1").map(|c| c.memories.len()),
            Some(1)
        );
    }

    /// load → save → load 链数守恒
    #[test]
    fn chains_survive_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("nt-chain-rt");
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join("cocoons.json");
        let mut store = CocoonStore::new(p.clone());
        store.record_chain(chain("RC-1", "结论一"));
        store.record_chain(chain("RC-2", "结论二"));
        assert_eq!(store.chains.len(), 2);
        store.save().expect("save");
        // 盘上确有 chains
        let raw = std::fs::read_to_string(&p).expect("read");
        assert!(raw.contains("\"chains\""), "盘上必须有 chains 键");
        // 再 load
        let data: StoreData = serde_json::from_str(&raw).expect("reparse");
        assert_eq!(data.chains.len(), 2, "链数必须守恒");
        assert_eq!(data.chains[0].conclusion, "结论一");
        let _ = std::fs::remove_file(&p);
    }

    /// 封顶：超限丢最旧、保留最近
    #[test]
    fn chain_cap_keeps_newest() {
        let many: Vec<ReasoningChain> = (0..(REASONING_CHAIN_CAP + 10))
            .map(|i| chain(&format!("RC-{}", i), "x"))
            .collect();
        let capped = cap_chains(many);
        assert_eq!(capped.len(), REASONING_CHAIN_CAP);
        assert_eq!(
            capped.last().map(|c| c.id.clone()),
            Some(format!("RC-{}", REASONING_CHAIN_CAP + 9)),
            "尾部必须是最新"
        );
        assert_eq!(
            capped.first().map(|c| c.id.clone()),
            Some("RC-10".to_string())
        );
    }

    /// record_chain 按 id 去重
    #[test]
    fn record_chain_dedups_by_id() {
        let mut store = CocoonStore::new(std::path::PathBuf::from("/tmp/nt-chain-dedup.json"));
        store.record_chain(chain("RC-1", "甲"));
        store.record_chain(chain("RC-1", "乙"));
        assert_eq!(store.chains.len(), 1, "同 id 不得重复落盘");
        assert_eq!(store.chains[0].conclusion, "甲", "首条为准");
    }

    /// 链回填进意识体，且不覆盖意识体已有的
    #[test]
    fn chains_backfill_into_consciousness() {
        let mut store = CocoonStore::new(std::path::PathBuf::from("/tmp/nt-chain-bf.json"));
        store.record_chain(chain("RC-1", "盘上链"));
        let mut c = CrystalConsciousness::new("bf");
        c.memories.insert(
            "M-000001".to_string(),
            Memory {
                id: "M-000001".to_string(),
                content: "前提".to_string(),
                memory_type: MemoryType::Fact,
                domain: "d1".to_string(),
                strength: 1.0,
                confidence: 0.8,
                importance: 0.5,
                connections: Vec::new(),
                created_at: 0,
                last_accessed: 0,
                access_count: 0,
            },
        );
        c.reasoning_chains.push(chain("RC-1", "意识体链"));
        store.sync_to_consciousness(&mut c);
        assert_eq!(c.reasoning_chains.len(), 1, "同 id 不得重复灌入");
        assert_eq!(
            c.reasoning_chains[0].conclusion, "意识体链",
            "意识体已有的链优先"
        );
    }
}
