//! LLM 成本归因 —— 把请求拆成可寻址的内容块并按缓存类别计价
//!
//! # 为什么必须"在链路上抓"而不是"读日志"
//!
//! 系统提示、注入的 tool schema、MCP schema、reminder 都是**在请求时组装、
//! 从不写进 transcript** 的。据 `tigerless/cost-xray` 实测：那段不可见前缀
//! 「可以占到上下文的一半甚至更多」。**任何基于日志的归因方案，结构性地对
//! 编码 agent 最大的成本中心是盲的。**
//!
//! 本仓当前状态（2026-09-27 实测）：
//! - `llm_types.rs:104` `Usage` 只有 `prompt_tokens` / `completion_tokens` /
//!   `total_tokens` —— **没有任何 cache 字段**
//! - 而 `nt_io_provider/anthropic/anthropic.rs:92` 正在打 `cache_control` 断点
//!
//! 即：**项目在享受 prompt 缓存，却对缓存命中/写入零计量。** 按 AIBrix 的
//! 结论（`AIBRIX_PREFIX_CACHE_INCLUDE_TOOLS` 默认 true，网关把规范化后的
//! `tools` 前置进哈希文本，"so requests that share messages but carry different
//! tools do not look like a full prefix match"），**工具集身份是缓存身份的一部分**
//! —— 一改 skill 集就击穿前缀缓存，而当前无任何可见性。
//!
//! # 2026 模型不是 per-token 也不是 per-tool-call
//!
//! 而是：把请求切成可寻址前缀 → 每块分配 token → **在 provider 的缓存边界切一刀**
//! → 每块按自己的缓存类别定价。四个环节缺一不可。

use serde::{Deserialize, Serialize};

/// 内容区：入向 / 出向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Zone {
    Input,
    Output,
}

/// 段的区分 —— **这是成本与"每轮由 harness 重新组装"的分界线**：
/// `static` = 每轮重新组装（系统提示 + tool schema）；`messages` = 对话累积。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Section {
    Static,
    Messages,
}

/// 内容桶。固定 6 值，跨 provider 归一化（Anthropic 的 `reasoning` → `Thinking`，
/// OpenAI Responses 的 `function_call` → `ToolUse`，`function_call_output` → `ToolResult`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bucket {
    System,
    Schema,
    Text,
    Thinking,
    ToolUse,
    ToolResult,
}

impl Bucket {
    pub fn as_str(self) -> &'static str {
        match self {
            Bucket::System => "system",
            Bucket::Schema => "schema",
            Bucket::Text => "text",
            Bucket::Thinking => "thinking",
            Bucket::ToolUse => "tool_use",
            Bucket::ToolResult => "tool_result",
        }
    }
}

/// 一条可寻址内容块 —— 归因的最小单元。
///
/// `skill` 与 `tool` 是**一等维度**：`tigerless/cost-xray` 是唯一把「skill 的
/// schema」与「skill 的内容加载」分开计价的实现（`("Static","Skills")` vs
/// `("Messages","Skill loads")`）。本仓有 120 个 skill，这是最需要可见性的维度。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentBlock {
    pub zone: Zone,
    pub section: Section,
    pub bucket: Bucket,
    /// 工具名；MCP 工具用 `mcp__<server>__<tool>` 形态，MCP server 名由此可还原。
    pub tool: Option<String>,
    /// skill 名（schema 与内容加载分开记账）。
    pub skill: Option<String>,
    pub role: Option<String>,
    /// 内容指纹：长会话每轮重发全量历史，按内容去重后只存一份 + 每轮增量。
    pub hash: String,
    /// 估算 token；`Some` 表示来自 provider 的精确计数（否则为 None = 估算）。
    pub tokens: Option<u32>,
}

impl ContentBlock {
    pub fn new(
        zone: Zone,
        section: Section,
        bucket: Bucket,
        hash: impl Into<String>,
    ) -> Self {
        Self {
            zone,
            section,
            bucket,
            tool: None,
            skill: None,
            role: None,
            hash: hash.into(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, t: u32) -> Self {
        self.tokens = Some(t);
        self
    }

    pub fn with_tool(mut self, t: impl Into<String>) -> Self {
        self.tool = Some(t.into());
        self
    }

    pub fn with_skill(mut self, s: impl Into<String>) -> Self {
        self.skill = Some(s.into());
        self
    }

    /// 从 `mcp__<server>__<tool>` 还原 MCP server 名。
    /// 参照 cost-xray 的字符串手术做法 —— 它不依赖 provider 元数据。
    /// 返回 `None` 表示不是 MCP 工具。
    pub fn mcp_server(&self) -> Option<&str> {
        let t = self.tool.as_deref()?;
        t.strip_prefix("mcp__")
            .and_then(|rest| rest.split_once("__"))
            .map(|(server, _)| server)
    }
}

/// 单价（每 token）。刻意不带 provider 依赖 —— 价格表是外部输入。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rates {
    pub input: f64,
    pub cache_read: f64,
    pub cache_write: f64,
    pub output: f64,
}

impl Rates {
    /// 由基础输入价按倍数派生。倍数取自 cost-xray 的 fallback 常量
    /// （`CACHE_READ_MULT = 0.1`、`CACHE_WRITE_MULT = 1.25`）。
    pub fn from_input(input: f64) -> Self {
        Self {
            input,
            cache_read: input * 0.1,
            cache_write: input * 1.25,
            output: input * 5.0,
        }
    }
}

/// 缓存类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheClass {
    Fresh,
    CacheRead,
    CacheWrite,
}

/// 一块在本次请求中落到的缓存类别与计价。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PricedBlock {
    pub block: ContentBlock,
    pub cache: CacheClass,
    pub tokens: u32,
    pub usd: f64,
}

/// provider 回传的权威 usage。`cached` / `rewrote` 是本仓 `Usage` 当前缺失的字段。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderUsage {
    pub fresh: u32,
    pub cached: u32,
    pub rewrote: u32,
    pub output: u32,
    /// 1 小时缓存写入 TTL 的写入价上浮系数（Anthropic 为 1.6）。
    pub write_1h: bool,
}

/// 归因结果 + **诚实残差**。
///
/// 契约（照抄 cost-xray 并强化）：**总额与账单精确；同一请求内各来源的拆分是
/// 近似的。** 因此残差必须公开，不能展示假精度。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribution {
    pub blocks: Vec<PricedBlock>,
    pub total_usd: f64,
    pub ours_input: u32,
    pub provider_input: u32,
    /// 拆分误差。调用方应在 UI 上显示它，而不是只显示 `total_usd`。
    pub input_err: u32,
    /// 缓存命中率。
    pub cache_hit: f64,
}

impl Attribution {
    /// 残差相对比例；超过 2% 时调用方应在 UI 明示"拆分不精确"。
    pub fn residual_ratio(&self) -> f64 {
        if self.provider_input == 0 {
            return 0.0;
        }
        self.input_err as f64 / self.provider_input as f64
    }
}

/// 在缓存边界切一刀。
///
/// 关键前提：**provider 缓存的是前缀**，所以入向块按顺序首尾相接，
/// 前 `cached` 个 token 是 cache-read，接着 `rewrote` 个是 cache-write，其余是 fresh。
/// 这是**前缀扫描**而非逐块打标 —— cost-xray 的做法。
///
/// ⚠️ 该前提对 Anthropic 的显式 `cache_control` 成立；对 OpenAI 的自动前缀缓存，
/// 边界由 provider 选定、必须推断。`ProviderUsage` 把 `cached`/`rewrote` 作为
/// **输入**而非推导，正是为了让调用方在不确定时显式传入真实值。
pub fn cut_at_cache_boundary(blocks: &[ContentBlock], usage: &ProviderUsage) -> Vec<PricedBlock> {
    let read_end = usage.cached as i64;
    let write_end = read_end + usage.rewrote as i64;
    let mut cursor: i64 = 0;
    let mut out = Vec::with_capacity(blocks.len());
    for b in blocks {
        if b.zone != Zone::Input {
            let tokens = b.tokens.unwrap_or(0);
            out.push(PricedBlock { block: b.clone(), cache: CacheClass::Fresh, tokens, usd: 0.0 });
            continue;
        }
        let tokens = b.tokens.unwrap_or(0) as i64;
        let end = cursor + tokens;
        let (lo, hi) = (cursor, end);
        // 三段重叠长度
        let n_read = overlap(lo, hi, 0, read_end);
        let n_write = overlap(lo, hi, read_end, write_end);
        let n_fresh = tokens - n_read - n_write;
        // 单一类别记账: 取该块的主导段, 避免同一块被重复计价。
        // 2026-09-27 修正: 0-token 块 (如未估 token 的 tool 名) 三段全 0,
        // 若按 `0>=0` 先命中会误判为 CacheRead —— 显式归 Fresh。
        let (cache, n) = if tokens == 0 {
            (CacheClass::Fresh, 0)
        } else if n_read >= n_write && n_read >= n_fresh {
            (CacheClass::CacheRead, n_read)
        } else if n_write >= n_fresh {
            (CacheClass::CacheWrite, n_write)
        } else {
            (CacheClass::Fresh, n_fresh)
        };
        out.push(PricedBlock { block: b.clone(), cache, tokens: n.max(0) as u32, usd: 0.0 });
        cursor = end;
    }
    out
}

fn overlap(lo: i64, hi: i64, a: i64, b: i64) -> i64 {
    (hi.min(b) - lo.max(a)).max(0)
}

/// 计价 + 算残差。
pub fn reconcile(
    mut priced: Vec<PricedBlock>,
    usage: &ProviderUsage,
    rates: &Rates,
) -> Attribution {
    let write_mult = if usage.write_1h { 1.6 } else { 1.0 };
    let mut total = 0.0f64;
    for p in priced.iter_mut() {
        p.usd = match p.cache {
            CacheClass::Fresh => p.tokens as f64 * rates.input,
            CacheClass::CacheRead => p.tokens as f64 * rates.cache_read,
            CacheClass::CacheWrite => p.tokens as f64 * rates.cache_write * write_mult,
        };
        total += p.usd;
    }
    total += usage.output as f64 * rates.output;

    let ours_input: u32 = priced
        .iter()
        .filter(|p| p.block.zone == Zone::Input)
        .map(|p| p.tokens)
        .sum();
    let provider_input = usage.fresh + usage.cached + usage.rewrote;
    let denom = usage.fresh + usage.cached + usage.rewrote;
    let cache_hit = if denom == 0 { 0.0 } else { usage.cached as f64 / denom as f64 };

    Attribution {
        blocks: priced,
        total_usd: total,
        ours_input,
        provider_input,
        input_err: ours_input.abs_diff(provider_input),
        cache_hit,
    }
}

/// 一步到位：切边界 + 计价。
pub fn attribute(
    blocks: &[ContentBlock],
    usage: &ProviderUsage,
    rates: &Rates,
) -> Attribution {
    reconcile(cut_at_cache_boundary(blocks, usage), usage, rates)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blk(section: Section, bucket: Bucket, tokens: u32, h: &str) -> ContentBlock {
        ContentBlock::new(Zone::Input, section, bucket, h).with_tokens(tokens)
    }

    #[test]
    fn no_cache_everything_is_fresh() {
        let blocks = vec![blk(Section::Messages, Bucket::Text, 100, "a")];
        let u = ProviderUsage { fresh: 100, cached: 0, rewrote: 0, output: 10, write_1h: false };
        let a = attribute(&blocks, &u, &Rates::from_input(1e-6));
        assert_eq!(a.blocks[0].cache, CacheClass::Fresh);
        assert_eq!(a.cache_hit, 0.0);
    }

    /// 核心语义: 缓存的是**前缀**, 不是随机位置。
    #[test]
    fn cache_boundary_is_a_prefix_cut() {
        let blocks = vec![
            blk(Section::Static, Bucket::System, 30, "sys"),
            blk(Section::Static, Bucket::Schema, 20, "tools"),
            blk(Section::Messages, Bucket::Text, 50, "msgs"),
        ];
        // 前 50 个 token 是 cache-read (30 sys + 20 tools)
        let u = ProviderUsage { fresh: 50, cached: 50, rewrote: 0, output: 0, write_1h: false };
        let a = attribute(&blocks, &u, &Rates::from_input(1e-6));
        assert_eq!(a.blocks[0].cache, CacheClass::CacheRead, "系统提示在缓存前缀内");
        assert_eq!(a.blocks[1].cache, CacheClass::CacheRead, "tool schema 也在前缀内");
        assert_eq!(a.blocks[2].cache, CacheClass::Fresh, "本轮新消息在缓存外");
        assert!((a.cache_hit - 0.5).abs() < 1e-9);
    }

    #[test]
    fn cache_write_is_the_band_after_read() {
        let blocks = vec![blk(Section::Static, Bucket::System, 100, "sys")];
        let u = ProviderUsage { fresh: 0, cached: 40, rewrote: 60, output: 0, write_1h: false };
        let a = attribute(&blocks, &u, &Rates::from_input(1e-6));
        // 40 read + 60 write 都落在这 100 token 的块上; 取主导段 -> write
        assert_eq!(a.blocks[0].cache, CacheClass::CacheWrite);
    }

    /// 写 1 小时 TTL 的上浮系数必须生效。
    #[test]
    fn write_1h_inflates_price() {
        let blocks = vec![blk(Section::Static, Bucket::System, 100, "sys")];
        let base = ProviderUsage { fresh: 0, cached: 0, rewrote: 100, output: 0, write_1h: false };
        let h1 = ProviderUsage { write_1h: true, ..base };
        let r = Rates::from_input(1e-6);
        let a0 = attribute(&blocks, &base, &r);
        let a1 = attribute(&blocks, &h1, &r);
        assert!((a1.total_usd / a0.total_usd - 1.6).abs() < 1e-9);
    }

    /// 缓存读远比 fresh 便宜 —— 这正是要能看到的原因。
    #[test]
    fn cache_read_is_cheaper_than_fresh() {
        let blocks = vec![blk(Section::Static, Bucket::System, 1000, "sys")];
        let fresh = ProviderUsage { fresh: 1000, cached: 0, rewrote: 0, output: 0, write_1h: false };
        let cached = ProviderUsage { fresh: 0, cached: 1000, rewrote: 0, output: 0, write_1h: false };
        let r = Rates::from_input(1e-3);
        let af = attribute(&blocks, &fresh, &r);
        let ac = attribute(&blocks, &cached, &r);
        assert!((af.total_usd / ac.total_usd - 10.0).abs() < 1e-6, "cache_read 应为 input 的 0.1 倍");
    }

    /// skill 是**一等维度** —— 本仓 120 个 skill, 这是最需要可见性的。
    #[test]
    fn skill_schema_and_skill_load_are_separate_blocks() {
        let blocks = vec![
            ContentBlock::new(Zone::Input, Section::Static, Bucket::Schema, "h1")
                .with_skill("xlsx-data").with_tokens(80),
            ContentBlock::new(Zone::Input, Section::Messages, Bucket::Text, "h2")
                .with_skill("xlsx-data").with_tokens(400),
        ];
        let u = ProviderUsage { fresh: 480, cached: 0, rewrote: 0, output: 0, write_1h: false };
        let a = attribute(&blocks, &u, &Rates::from_input(1e-6));
        assert_eq!(a.blocks.len(), 2);
        assert_eq!(a.blocks[0].block.section, Section::Static);
        assert_eq!(a.blocks[1].block.section, Section::Messages);
        assert!(a.blocks[1].tokens > a.blocks[0].tokens, "内容加载远大于 schema");
    }

    /// MCP server 名可从工具名还原 —— 用于「注入了但从未被调用」的浪费探测。
    #[test]
    fn mcp_server_is_recoverable_from_tool_name() {
        let b = ContentBlock::new(Zone::Input, Section::Static, Bucket::Schema, "h")
            .with_tool("mcp__perplexity__search");
        assert_eq!(b.mcp_server(), Some("perplexity"));

        let native = ContentBlock::new(Zone::Input, Section::Messages, Bucket::ToolUse, "h2")
            .with_tool("Read");
        assert_eq!(native.mcp_server(), None);
    }

    /// 残差必须公开 —— 不允许展示假精度。
    #[test]
    fn publishes_residual_when_estimate_is_off() {
        let blocks = vec![blk(Section::Messages, Bucket::Text, 100, "a")];
        let u = ProviderUsage { fresh: 130, cached: 0, rewrote: 0, output: 0, write_1h: false };
        let a = attribute(&blocks, &u, &Rates::from_input(1e-6));
        assert_eq!(a.input_err, 30);
        assert!(a.residual_ratio() > 0.2, "残差比例应被如实计算");
    }

    #[test]
    fn exact_estimate_has_zero_residual() {
        let blocks = vec![blk(Section::Messages, Bucket::Text, 100, "a")];
        let u = ProviderUsage { fresh: 100, cached: 0, rewrote: 0, output: 0, write_1h: false };
        let a = attribute(&blocks, &u, &Rates::from_input(1e-6));
        assert_eq!(a.input_err, 0);
        assert_eq!(a.residual_ratio(), 0.0);
    }

    #[test]
    fn output_is_priced_separately() {
        let blocks = vec![];
        let u = ProviderUsage { fresh: 0, cached: 0, rewrote: 0, output: 100, write_1h: false };
        let r = Rates::from_input(1e-6);
        let a = attribute(&blocks, &u, &r);
        assert!((a.total_usd - 100.0 * r.output).abs() < 1e-12);
    }

    /// 同一块被拆到多段时不能重复计价。
    #[test]
    fn single_block_spanning_boundary_is_counted_once() {
        let blocks = vec![blk(Section::Messages, Bucket::Text, 100, "a")];
        let u = ProviderUsage { fresh: 40, cached: 30, rewrote: 30, output: 0, write_1h: false };
        let a = attribute(&blocks, &u, &Rates::from_input(1e-6));
        assert_eq!(a.blocks.len(), 1);
        assert!(a.blocks[0].tokens <= 100);
    }

    #[test]
    fn output_zone_blocks_are_not_cut_by_input_boundary() {
        let blocks = vec![ContentBlock::new(Zone::Output, Section::Messages, Bucket::Text, "o")
            .with_tokens(40)];
        let u = ProviderUsage { fresh: 0, cached: 100, rewrote: 0, output: 0, write_1h: false };
        let a = attribute(&blocks, &u, &Rates::from_input(1e-6));
        // 出向块不应被入向缓存边界切成 CacheRead
        assert_eq!(a.blocks[0].cache, CacheClass::Fresh);
    }
}

// ============ 请求分类 (3.1 接线) ============
// `LlmRequest` 在全仓有 3 个同名定义 (`unified.rs:301` / `llm_types.rs:15`
// / `traits.rs:326`), 为避开该耦合, 本函数只收原始切片, 由各 provider 适配。

/// 简单内容哈希：长度 + 首尾各 8 字节。
/// 不是加密哈希 —— 用途是"跨轮去重", 不是"防碰撞"。8+8 字节加长度对
/// session 级去重足够；若未来要全局账本, 换 sha1(content)[:8]。
pub fn block_hash(content: &str) -> String {
    let bytes = content.as_bytes();
    let (head, tail) = if bytes.len() <= 16 {
        (bytes, &[][..])
    } else {
        (&bytes[..8], &bytes[bytes.len() - 8..])
    };
    format!("{:x}:{:02x?}{:02x?}", bytes.len(), head, tail)
}

/// 把一次组装好的请求切成可寻址块。
///
/// 块顺序 = 线上字节顺序：`static` 先（系统提示 → skill schema → tool schema），
/// `messages` 后。**这个顺序是负载**：`cut_at_cache_boundary` 做前缀切分时依赖它。
pub fn classify_input(
    system: Option<&str>,
    skill_schemas: &[(String, String)],
    tool_names: &[String],
    messages: &[(String, String)],
    estimate: &dyn Fn(&str) -> u32,
) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();
    if let Some(sys) = system {
        blocks.push(
            ContentBlock::new(Zone::Input, Section::Static, Bucket::System, block_hash(sys))
                .with_tokens(estimate(sys)),
        );
    }
    for (skill, schema) in skill_schemas {
        blocks.push(
            ContentBlock::new(Zone::Input, Section::Static, Bucket::Schema, block_hash(schema))
                .with_skill(skill.clone())
                .with_tokens(estimate(schema)),
        );
    }
    for tool in tool_names {
        blocks.push(
            ContentBlock::new(Zone::Input, Section::Static, Bucket::Schema, block_hash(tool))
                .with_tool(tool.clone()),
        );
    }
    for (role, content) in messages {
        let bucket = match role.as_str() {
            "tool" | "tool_result" | "function_call_output" => Bucket::ToolResult,
            "thinking" | "reasoning" => Bucket::Thinking,
            "function_call" | "tool_use" => Bucket::ToolUse,
            _ => Bucket::Text,
        };
        let mut b = ContentBlock::new(
            Zone::Input,
            Section::Messages,
            bucket,
            block_hash(content),
        )
        .with_tokens(estimate(content));
        b.role = Some(role.clone());
        blocks.push(b);
    }
    blocks
}

#[cfg(test)]
mod classify_tests {
    use super::*;

    fn est(s: &str) -> u32 {
        (s.len() / 4).max(1) as u32
    }

    #[test]
    fn block_order_matches_wire_order() {
        let b = classify_input(
            Some("sys"),
            &[("xlsx".into(), "schema-bytes".into())],
            &["Read".into(), "mcp__perplexity__search".into()],
            &[("user".into(), "hi".into())],
            &est,
        );
        // static 先: system -> skill schema -> tool schema ×2; messages 后
        let buckets: Vec<_> = b.iter().map(|x| (x.section, x.bucket)).collect();
        assert_eq!(
            buckets,
            vec![
                (Section::Static, Bucket::System),
                (Section::Static, Bucket::Schema),
                (Section::Static, Bucket::Schema),
                (Section::Static, Bucket::Schema),
                (Section::Messages, Bucket::Text),
            ]
        );
    }

    #[test]
    fn skill_and_tool_dimensions_are_populated() {
        let b = classify_input(
            None,
            &[("xlsx-data".into(), "s".into())],
            &["mcp__perplexity__search".into()],
            &[],
            &est,
        );
        assert_eq!(b[0].skill.as_deref(), Some("xlsx-data"));
        assert_eq!(b[1].tool.as_deref(), Some("mcp__perplexity__search"));
        assert_eq!(b[1].mcp_server(), Some("perplexity"));
    }

    #[test]
    fn role_maps_to_bucket_across_providers() {
        let b = classify_input(
            None,
            &[],
            &[],
            &[
                ("tool".into(), "r".into()),
                ("thinking".into(), "t".into()),
                ("function_call".into(), "f".into()),
                ("assistant".into(), "a".into()),
            ],
            &est,
        );
        let got: Vec<Bucket> = b.iter().map(|x| x.bucket).collect();
        assert_eq!(got, vec![Bucket::ToolResult, Bucket::Thinking, Bucket::ToolUse, Bucket::Text]);
    }

    #[test]
    fn classify_feeds_cut_end_to_end() {
        let b = classify_input(
            Some("system prompt here"),
            &[("k".into(), "skill schema".into())],
            &["Read".into()],
            &[("user".into(), "new question".into())],
            &est,
        );
        let u = ProviderUsage { fresh: 100, cached: 0, rewrote: 0, output: 5, write_1h: false };
        let a = attribute(&b, &u, &Rates::from_input(1e-6));
        assert_eq!(a.blocks.len(), b.len());
        assert!(a.blocks.iter().all(|p| p.cache == CacheClass::Fresh));
        assert_eq!(a.blocks[0].block.bucket, Bucket::System);
    }

    #[test]
    fn hash_distinguishes_content_and_is_stable() {
        assert_eq!(block_hash("abc"), block_hash("abc"));
        assert_ne!(block_hash("abc"), block_hash("abd"));
        assert_ne!(block_hash("short"), block_hash(&"short ".repeat(50)));
    }

    #[test]
    fn empty_request_is_empty_blocks_not_an_error() {
        let b = classify_input(None, &[], &[], &[], &est);
        assert!(b.is_empty());
    }
}
