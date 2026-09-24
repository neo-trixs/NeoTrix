//! neotrix-experience — Unified End-of-Conversation Absorption Engine (Rust native,
//! 生产路径; 历史原型为 Python 版 `absorb_session.py`, 已退役).
#![allow(clippy::unwrap_used)] // CLI bin 自持 JSON/DB 结构, 结构非法即 panic 报错优于静默降级
//! 单入口: 每次会话结束时运行 `neotrix-experience absorb <session.json>` (或 `-` 读 stdin,
//!   stdin 直写 KB 跳过本地中转文件)
//!   统一数据层: ~/.neotrix/knowledge.db 的 kv_store, namespace='experience'.
//!   统一 schema: 每条经验含 {schema_version, type, session_id, cycle, ts, domain,
//!                content, evidence, source, verify_by}.
//!   verify_by 软过期软门槛 — 经验默认 verify_by = ts + VERIFY_DEFAULT_DAYS,
//!   超期仍可检索但被 `stale` 标记,触发复核而非静默信任.
//!
//! 五阶段协议:
//!   1. Snapshot 快照     — 记录会话开始/结束上下文 (snapshot / close)
//!   2. Distill 蒸馏     — 从对话提取 patterns/rules/defects/insights (absorb)
//!   3. Classify 分类     — 映射到已有节点: domain + evidence (file:line|url)
//!   4. Persist 落盘     — 写入 kv_store experience namespace + 更新 hub 索引
//!   5. Feedback 反馈     — 同步 route_table 使后续 session 可检索 (query)
//!
//! 神经概念层: 概念=神经元 (唯一去重存储), 分支=经验节点,
//!   突触=概念↔分支引用, Hebb 共现=概念↔概念二阶联想。
//! value 透明压缩: 魔数 NTZ1 + zlib (仅当压缩有收益时), 读取透明解压。
//!
//! Usage:
//!   cargo run -p neotrix --bin neotrix-experience snapshot --cycle NNN --task "..." [--domain X]
//!   cargo run -p neotrix --bin neotrix-experience absorb <session.json>
//!   cat session.json | cargo run -p neotrix --bin neotrix-experience absorb -
//!   cargo run -p neotrix --bin neotrix-experience close --cycle NNN
//!   cargo run -p neotrix --bin neotrix-experience query --kw "关键词" [--type T] [--domain D] [--limit N] [--no-hebb] [--include-distilled]
//!   cargo run -p neotrix --bin neotrix-experience list [--type T] [--domain D]
//!   cargo run -p neotrix --bin neotrix-experience stale [--domain D]
//!   cargo run -p neotrix --bin neotrix-experience hub
//!   cargo run -p neotrix --bin neotrix-experience route --kw KEYWORD --branch BK
//!   cargo run -p neotrix --bin neotrix-experience neuron TERM [--exact]
//!   cargo run -p neotrix --bin neotrix-experience backfill
//!   cargo run -p neotrix --bin neotrix-experience prune [--stop WORD ...] [--stale-isolated]
//!   cargo run -p neotrix --bin neotrix-experience hebb
//!   cargo run -p neotrix --bin neotrix-experience distill [--domain D] [--min-group N] [--dry-run]
//!   cargo run -p neotrix --bin neotrix-experience compress [--all]
//!   cargo run -p neotrix --bin neotrix-experience gen-index [--out FILE] [--limit N]

#![forbid(unsafe_code)]
use clap::{Parser, Subcommand};

pub(crate) const NS: &str = "experience";
pub(crate) const SCHEMA_VERSION: i64 = 1;

pub(crate) const DOMAINS: [&str; 11] = [
    "NT-CORE", "NT-MIND", "NT-MEMORY", "NT-WORLD", "NT-ACT", "NT-IO", "NT-SHIELD",
    "NT-META", "NT-REPAIR", "NT-GOVERNANCE", "NT-NEXUS",
];
pub(crate) const TYPES: [&str; 6] = ["pattern", "rule", "defect", "insight", "cycle", "artifact"];

// verify_by 软过期: 经验默认审核周期, 超出后标记 stale 待复核而非静默信任
pub(crate) const VERIFY_DEFAULT_DAYS: i64 = 90;
pub(crate) const DAY: i64 = 86400;

// 中文停用字 (单字无意义) 与英文停用词
pub(crate) const CN_STOP: &str = "的了是在与和就都而这于有也一个我你他她它其之为此对从地向到";
pub(crate) const EN_STOP: [&str; 66] = [
    "the", "and", "for", "with", "from", "that", "this", "into", "were", "was",
    "will", "have", "has", "had", "not", "are", "but", "its", "than", "then",
    "when", "where", "which", "while", "should", "would", "could", "can", "may",
    "must", "only", "over", "under", "also", "been", "being", "does", "did",
    "doing", "how", "what", "why", "our", "your", "their", "his", "her",
    "area", "tree", "pass", "note", "status", "check", "action",
    "these", "those", "them", "they", "there", "here", "each", "both", "some",
    "such", "very", "just",
];

pub(crate) const CONCEPT_MIN_LEN: usize = 3;
pub(crate) const CONCEPT_MAX_LEN: usize = 8;

// ── Mastra OM 吸收 (experience-tree v2, 2026-08-24) ──
// 观察/反射 token 预算常量
pub(crate) const OBS_TOKEN_BUDGET: usize = 30_000;
pub(crate) const OBS_BUFFER_RATIO: f64 = 0.2;
pub(crate) const OBS_BUFFER_ACTIVATION: f64 = 0.8;
pub(crate) const REF_TOKEN_BUDGET: usize = 40_000;
pub(crate) const REF_BUFFER_ACTIVATION: f64 = 0.5;


/// 简易 token 估算 (单一事实源, 兼容 CJK): 无 tiktoken 时回退逐字符估算 (保守上界, 最小 1)。

#[path = "experience/exp_util.rs"]
mod exp_util;
#[path = "experience/exp_store.rs"]
mod exp_store;
#[path = "experience/exp_concept.rs"]
mod exp_concept;
#[path = "experience/exp_absorb.rs"]
mod exp_absorb;
#[path = "experience/exp_query.rs"]
mod exp_query;
#[path = "experience/exp_distill.rs"]
mod exp_distill;
#[path = "experience/exp_topo.rs"]
mod exp_topo;

#[derive(Parser)]
#[command(name = "neotrix-experience", about = "Unified end-of-conversation absorption engine")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 会话开始快照
    Snapshot {
        #[arg(long, required = true)]
        cycle: String,
        #[arg(long, default_value = "")]
        task: String,
        #[arg(long, default_value = "unknown")]
        domain: String,
    },
    /// 关闭会话快照
    Close {
        #[arg(long, required = true)]
        cycle: String,
    },
    /// 会话结束吸收
    /// 输入: session JSON (含 session_id/cycle/ts/entries), 文件路径或 "-" 读 stdin
    /// (stdin 直写 KB experience 命名空间, 跳过 pending-absorb.json 本地中转)。
    Absorb {
        /// session JSON 文件路径, 或 "-" 读 stdin
        #[arg(default_value = "-")]
        session: String,
    },
    /// P0-2 G3+G8: 记录经验复用反馈 → MapeGate burn-in 门 (晋升/回滚)
    Feedback {
        key: String,
        /// success | failure
        #[arg(long, default_value = "success")]
        outcome: String,
    },
    /// 批量节点吸收 (R-P97: Python insert_node 的 Rust port — 知识写入单一事实源)
    /// 输入: JSON 文件 (节点数组或单节点), 每条含 node_type/title/summary/content/url/domain/
    ///       language/importance/meta; 可含 capability {branch,capability,evidence} 四元组。
    /// 语义: URL 去重 + nodes/nodes_fts 双写 (FTS 显式插入, 防 PA011 desync)。
    AbsorbNode {
        /// 节点 JSON 文件 (数组或单对象), 或 "-" 读 stdin
        #[arg(default_value = "-")]
        input: String,
        #[arg(long)]
        dry_run: bool,
        /// 写 metadata.absorbed_capability 四元组 (R-P79 闭环)
        #[arg(long)]
        apply_capability: bool,
    },
    /// 批量更新已有节点的 metadata (R-P97: absorb_to_capability.py 写回路径的 Rust port)
    /// 输入: JSON 数组, 每项 {node_id, patch: {key: value}} — 读原 metadata JSON →
    ///       合并 patch → 写回。patch 值可为任意 JSON (对象如 absorbed_capability 四元组)。
    UpdateNodeMetadata {
        /// 更新清单 JSON 文件, 或 "-" 读 stdin
        #[arg(default_value = "-")]
        input: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// 检索匹配分支
    Query {
        #[arg(long, default_value = "")]
        kw: String,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        domain: Option<String>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        no_hebb: bool,
        #[arg(long)]
        json: bool,
        /// 增加 VSA 语义近邻信号: 检索后按嵌入相似度加权重排 (混合检索第三路)
        #[arg(long)]
        semantic: bool,
        /// 包含已蒸馏 (distilled) 的原始经验 — 默认过滤 (模式已升维, 原始条目仅作溯源)
        #[arg(long)]
        include_distilled: bool,
    },
    /// 列出条目
    List {
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        domain: Option<String>,
        #[arg(long)]
        cycle: Option<String>,
    },
    /// 列出过期 (verify_by) 分支 — 复核清单
    Stale {
        #[arg(long)]
        domain: Option<String>,
    },
    /// 查看 hub
    Hub,
    /// 更新 route_table
    Route {
        #[arg(long, required = true)]
        kw: String,
        #[arg(long, required = true)]
        branch: String,
    },
    /// 巡检 route_table 幽灵路由 (--clean 移除)
    RouteVerify {
        #[arg(long)]
        clean: bool,
    },
    /// 神经概念图检视 (突触链路)
    Neuron {
        term: String,
        #[arg(long)]
        exact: bool,
    },
    /// 为旧分支重建概念神经元与突触链路
    Backfill,
    /// 清理停用词污染与孤立概念神经元 (幂等自愈)
    Prune {
        #[arg(long, num_args = 0..)]
        stop: Vec<String>,
        #[arg(long)]
        stale_isolated: bool,
    },
    /// 重建 Hebb 共现突触网络
    Hebb,
    /// 清理重复分支: 内容归一化相同 → 保留一份, 删其余 (含概念图摘引用 + hub 刷新)
    Dedup {
        #[arg(long)]
        dry_run: bool,
    },
    /// 维度蒸馏: 按域+主题聚类细枝末节经验 → 升维为能力网/意识体维度模式,
    /// 原始条目标记 distilled 降权 (保留溯源, 不删除)。消退蒸馏核心。
    Distill {
        #[arg(long)]
        domain: Option<String>,
        /// 组内最少条目数才蒸馏 (默认 3)
        #[arg(long, default_value_t = 3)]
        min_group: usize,
        #[arg(long)]
        dry_run: bool,
    },
    /// 存量 value 透明压缩迁移 (zlib, 魔数标记)
    Compress {
        #[arg(long)]
        all: bool,
    },
    /// 从 KB 生成 Experience Index 派生文件
    GenIndex {
        #[arg(long, default_value = "-")]
        out: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// 测量两段文本的 VSA 词袋相似度 (阈值校准工具)
    Sim {
        #[arg(long, default_value = "")]
        a: String,
        #[arg(long, default_value = "")]
        b: String,
        #[arg(long, default_value_t = 2048)]
        dim: usize,
    },
    /// 记忆星系拓扑报告: 经验嵌入点云 → 持续同调 (Betti 数) + 记忆簇
    Topology {
        #[arg(long, default_value_t = 2048)]
        dim: usize,
        #[arg(long, default_value_t = 10)]
        steps: usize,
        /// 点云采样上限 (O(n³) 三角形计数防爆炸, 0=不限)
        #[arg(long, default_value_t = 400)]
        max_points: usize,
        #[arg(long)]
        json: bool,
    },
    /// P0-1 反射阶段 (Mastra OM 吸收): 观察日志 → 反射日志 (重写非追加)。
    /// 读取观察条目, 当总 token 超过 40k 时触发反射重写:
    /// 旧信息更激进压缩, 近期细节保留; 有界近阈值震荡 (6k↔40k)。
    /// 更新观察条目的 reflection 元数据 (version++, last_reflect_ts)。
    Reflect {
        /// 限定域 (默认全部)
        #[arg(long)]
        domain: Option<String>,
        /// 仅 dry-run, 不落盘
        #[arg(long)]
        dry_run: bool,
    },
    /// 符文配置 (Rune Socketing)：为模块设置/获取 5 色符文组合，动态调整 KB 读写策略。
    Rune {
        #[arg(long, required = true)]
        action: String, // "set" 或 "get"
        #[arg(long)]
        color: Option<String>, // crimson|indigo|obsidian|golden|alabaster
        #[arg(long)]
        module: Option<String>,
    },
    /// 成熟度星座审计：检查每个模块的 C0‑C6 等级并报告是否满足流水线要求。
    Constellation {
        #[arg(long, required = true)]
        action: String, // "audit" 或 "mature"
        #[arg(long)]
        domain: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let mut conn = exp_store::open_kb();
    match cli.cmd {
        Cmd::Snapshot { cycle, task, domain } => exp_absorb::cmd_snapshot(&conn, &cycle, &task, &domain),
        Cmd::Close { cycle } => exp_absorb::cmd_close(&conn, &cycle),
        Cmd::Absorb { session } => exp_absorb::cmd_absorb(&mut conn, &session),
        Cmd::Feedback { key, outcome } => exp_absorb::cmd_feedback(&mut conn, &key, &outcome),
        Cmd::AbsorbNode { input, dry_run, apply_capability } => {
            exp_absorb::cmd_absorb_node(&conn, &input, dry_run, apply_capability)
        }
        Cmd::UpdateNodeMetadata { input, dry_run } => {
            exp_absorb::cmd_update_node_metadata(&conn, &input, dry_run)
        }
        Cmd::Query { kw, r#type, domain, limit, no_hebb, json, semantic, include_distilled } => {
            exp_query::cmd_query(&conn, &kw, r#type.as_deref(), domain.as_deref(), limit, no_hebb, json, semantic, include_distilled);
        }
        Cmd::List { r#type, domain, cycle } => exp_query::cmd_list(&conn, r#type.as_deref(), domain.as_deref(), cycle.as_deref()),
        Cmd::Stale { domain } => exp_query::cmd_stale(&conn, domain.as_deref()),
        Cmd::Hub => exp_query::cmd_hub(&conn),
        Cmd::Route { kw, branch } => exp_query::cmd_route(&conn, &kw, &branch),
        Cmd::RouteVerify { clean } => exp_query::cmd_route_verify(&conn, clean),
        Cmd::Neuron { term, exact } => exp_query::cmd_neuron(&conn, &term, exact),
        Cmd::Backfill => exp_query::cmd_backfill(&conn),
        Cmd::Prune { stop, stale_isolated } => exp_query::cmd_prune(&conn, &stop, stale_isolated),
        Cmd::Hebb => exp_distill::cmd_hebb(&mut conn),
        Cmd::Dedup { dry_run } => exp_query::cmd_dedup(&conn, dry_run),
        Cmd::Distill { domain, min_group, dry_run } => {
            exp_distill::cmd_distill(&mut conn, domain.as_deref(), min_group, dry_run)
        }
        Cmd::Compress { all } => exp_distill::cmd_compress(&mut conn, all),
        Cmd::GenIndex { out, limit } => exp_distill::cmd_gen_index(&conn, &out, limit),
        Cmd::Sim { a, b, dim } => exp_topo::cmd_sim(&a, &b, dim),
        Cmd::Topology {
            dim,
            steps,
            max_points,
            json,
        } => exp_topo::cmd_topology(&conn, dim, steps, max_points, json),
        Cmd::Reflect { domain, dry_run } => exp_distill::cmd_reflect(&mut conn, domain.as_deref(), dry_run),
        Cmd::Rune { action, color, module } => exp_topo::cmd_rune(&mut conn, &action, color.as_deref(), module.as_deref()),
        Cmd::Constellation { action, domain } => exp_topo::cmd_constellation(&mut conn, &action, domain.as_deref()),
    }
}

#[cfg(test)]
#[path = "experience/tests.rs"]
mod tests;
