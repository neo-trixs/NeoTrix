# 能力构建实施计划 — 15 Source Absorption → NeoTrix Enhancement

> 基于 `.neotrix/absorption-15-sources.md` 吸收报告，对现有 6 个核心模块的精确增强方案。
> 遵循 R-P42 (强化现有节点, 禁平行适配器) + R-P79 (同 session 接线到生产路径)。

---

## 总览: 7 个增强点 × 3 个优先级

```
P0 (Week 1)                          P1 (Week 2)                          P2 (Week 3)
┌─────────────────────────┐  ┌──────────────────────────────┐  ┌──────────────────────┐
│ ① Runtime Anti-Hallucination │  │ ④ Bitemporal KG              │  │ ⑥ Hybrid HNSW        │
│    auto_crystallizer扩展      │  │    temporal_facts 增加        │  │    RetrievalMatrix    │
│                              │  │    believed_at 字段           │  │    增加 ANN index     │
│ ② Security Fix Pipeline     │  │                              │  │                      │
│    mcp_security 增加         │  │ ⑤ Scope-Based Claims         │  │ ⑦ Shadow Agents      │
│    synthesize→verify→apply   │  │    coordinator 增加           │  │    coordinator 增加   │
│                              │  │    ScopeClaim + ChangeSet    │  │    persistent shadow  │
│ ③ Context Shunt             │  │                              │  │                      │
│    context_window 增加       │  │                              │  │                      │
│    PreToolUse hooks          │  │                              │  │                      │
└─────────────────────────┘  └──────────────────────────────┘  └──────────────────────┘
```

---

## P0-①: Runtime Anti-Hallucination Pipeline

**来源**: reverify (LLM→deterministic verify→fact) + gpt-6-astra (misalignment monitoring)
**目标模块**: `auto_crystallizer.rs` (强化现有反幻觉门)
**现状**: 反幻觉门仅在 crystallization 阶段生效 (结晶时检查 VerificationContract)
**缺口**: 推理阶段 (reasoning_engine) 无实时幻觉检测

### 文件变更

**文件**: `neotrix-core/src/unified/layers/cognition/nt_mind/auto_crystallizer.rs`

```rust
// ── 新增: 推理阶段反幻觉管道 ──

/// 推理声明 — LLM 产出的单条声明, 待确定性验证。
#[derive(Debug, Clone)]
pub struct ReasoningClaim {
    pub claim_id: String,
    pub text: String,
    pub source: String,           // 来源 (reasoning step / tool output)
    pub confidence: f64,          // LLM 自评置信度
    pub created_at: u64,
}

/// 验证结果 — 确定性工具链对声明的判定。
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// 声明被确认 (grounded in evidence)
    Confirmed { evidence: String },
    /// 声明被反驳 (contradicted by evidence)
    Refuted { evidence: String },
    /// 无法判定 (insufficient evidence)
    Inconclusive { reason: String },
}

/// 推理反幻觉管道 — 吸收 reverify 模式:
/// LLM 提案 → 确定性工具验证 → 只有 verified facts 进入 KB。
pub struct ReasoningVerificationPipeline {
    /// 待验证声明队列
    pub pending_claims: Vec<ReasoningClaim>,
    /// 已验证声明 (进入 KB 的 facts)
    pub verified_facts: Vec<VerifiedFact>,
    /// 被拒绝的声明 (进入 hallucination bin)
    pub rejected_claims: Vec<RejectedClaim>,
    /// 验证器注册表 (确定性工具链)
    pub verifiers: Vec<Box<dyn ClaimVerifier>>,
    /// 统计
    pub stats: VerificationStats,
}

/// 验证器 trait — 所有确定性验证工具实现此 trait。
pub trait ClaimVerifier: Send + Sync {
    /// 验证器名称
    fn name(&self) -> &str;
    /// 验证一条声明, 返回确定性结果
    fn verify(&self, claim: &ReasoningClaim) -> VerificationResult;
}

/// 已验证事实 — 通过验证, 可安全进入 KB。
#[derive(Debug, Clone)]
pub struct VerifiedFact {
    pub claim: ReasoningClaim,
    pub result: VerificationResult,
    pub verified_at: u64,
}

/// 被拒绝声明 — 未通过验证, 进入幻觉桶。
#[derive(Debug, Clone)]
pub struct RejectedClaim {
    pub claim: ReasoningClaim,
    pub result: VerificationResult,
    pub rejected_at: u64,
}

#[derive(Debug, Clone, Default)]
pub struct VerificationStats {
    pub total_claims: u64,
    pub confirmed: u64,
    pub refuted: u64,
    pub inconclusive: u64,
}
```

### 新增方法

```rust
impl ReasoningVerificationPipeline {
    pub fn new() -> Self { /* ... */ }

    /// 注册一个确定性验证器
    pub fn register_verifier(&mut self, v: Box<dyn ClaimVerifier>) { /* ... */ }

    /// 核心: 提交声明 → 所有验证器并行验证 → 返回综合结果
    pub fn submit_claim(&mut self, claim: ReasoningClaim) -> VerificationResult {
        // 1. 收集所有验证器结果
        // 2. 多数投票 (>= 2/3 confirmed → Confirmed)
        // 3. 任一 Refuted → Refuted (安全优先)
        // 4. 其余 → Inconclusive
        // 5. 根据结果分类: confirmed → verified_facts, refuted → rejected_claims
        // 6. 更新 stats
    }

    /// 批量验证: 一组声明 → 分为 verified / rejected / inconclusive
    pub fn verify_batch(&mut self, claims: Vec<ReasoningClaim>) -> BatchVerificationResult {
        // 逐条 submit_claim, 汇总结果
    }
}
```

### 3 个内置验证器

```rust
/// 验证器 1: 代码编译验证 — 声明"代码可编译"→ 实际 cargo check
pub struct CompileVerifier;
impl ClaimVerifier for CompileVerifier {
    fn verify(&self, claim: &ReasoningClaim) -> VerificationResult {
        // 如果声明包含代码 → 提取代码块 → 写入临时文件 → cargo check
        // 成功 → Confirmed, 失败 → Refuted
    }
}

/// 验证器 2: 测试通过验证 — 声明"测试通过"→ 实际 cargo test
pub struct TestVerifier;
impl ClaimVerifier for TestVerifier {
    fn verify(&self, claim: &ReasoningClaim) -> VerificationResult {
        // 提取测试名 → cargo test <name> → 通过/失败
    }
}

/// 验证器 3: KB 一致性验证 — 声明与 KB 现有事实一致
pub struct KBConsistencyVerifier;
impl ClaimVerifier for KBConsistencyVerifier {
    fn verify(&self, claim: &ReasoningClaim) -> VerificationResult {
        // 提取 subject+predicate → 查询 temporal_facts → 是否一致
    }
}
```

### 生产接线 (R-P79)

**接线点**: `engine_core.rs` 的 `reason()` 方法

```rust
// 在 ReasoningEngine::reason() 中, LLM 输出后插入验证管道:
pub async fn reason(&mut self, task: &str) -> Result<String, ...> {
    // ... 现有推理逻辑 ...
    let llm_output = self.call_llm(prompt).await?;

    // P0-① 接线: 推理反幻觉管道
    if let Some(ref mut vpipeline) = self.verification_pipeline {
        let claims = extract_claims(&llm_output);  // 从 LLM 输出提取声明
        let result = vpipeline.verify_batch(claims);
        if result.any_refuted() {
            // 幻觉检测到 → 触发重试或降级
            log::warn!("hallucination detected: {} claims refuted", result.refuted);
            // 进入 hallucination_bin 供审计
        }
    }

    Ok(llm_output)
}
```

### 测试计划

| 测试 | 验证点 |
|------|--------|
| `test_compile_verifier_confirms` | 编译通过 → Confirmed |
| `test_compile_verifier_refutes` | 编译失败 → Refuted |
| `test_kb_consistency_confirms` | KB 有一致事实 → Confirmed |
| `test_kb_consistency_refutes` | KB 有矛盾事实 → Refuted |
| `test_majority_vote` | 2/3 confirmed → overall Confirmed |
| `test_single_refute_blocks` | 任一 Refuted → overall Refuted |
| `test_rejected_claim_goes_to_bin` | 被拒声明进入 rejected_claims |

---

## P0-②: Security Fix Pipeline

**来源**: agentic-security (synthesize→verify→apply + dollar-cost estimates)
**目标模块**: `nt_shield_mcp_security.rs` (强化现有 MCP 安全工具)
**现状**: 6 个扫描工具, 只检测不修复
**缺口**: 无 synthesize→verify→apply 流水线, 无 dollar-cost 估算

### 文件变更

**文件**: `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield/nt_shield_mcp_security.rs`

```rust
// ── 新增: 安全修复流水线 ──

/// 修复提案 — 扫描发现 → LLM 生成修复方案
#[derive(Debug, Clone)]
pub struct FixProposal {
    pub finding: SecurityFinding,
    pub proposed_fix: String,           // 修复代码/配置
    pub confidence: f64,                // 修复置信度
    pub estimated_cost_usd: f64,        // 预估修复成本 (dollar-cost)
    pub risk_if_unfixed: DollarCost,    // 不修复的风险成本
}

/// Dollar-cost 估算 (吸收 agentic-security 模式)
#[derive(Debug, Clone)]
pub struct DollarCost {
    pub best_case: f64,      // 最佳情况损失
    pub likely_case: f64,    // 最可能损失
    pub worst_case: f64,     // 最坏情况损失
    pub source: String,      // 估算依据 (IBM/NIST 数据)
}

/// 修复验证器 — 确定性验证修复是否正确且不引入新问题
pub struct FixVerifier {
    /// 修复后重新扫描: 确认原问题已解决
    pub rescan: bool,
    /// 检查是否引入新 >=Medium 发现
    pub no_new_medium: bool,
    /// lint 验证
    pub lint_check: bool,
}

/// 安全修复流水线 — 吸收 agentic-security 模式:
/// scan → synthesize_fix → verify_fix → apply_fix
pub struct SecurityFixPipeline {
    pub proposals: Vec<FixProposal>,
    pub applied_fixes: Vec<AppliedFix>,
    pub rejected_fixes: Vec<RejectedFix>,
    pub verifier: FixVerifier,
    pub stats: FixStats,
}

#[derive(Debug, Clone)]
pub struct AppliedFix {
    pub proposal: FixProposal,
    pub verified: bool,
    pub applied_at: u64,
}

#[derive(Debug, Clone)]
pub struct RejectedFix {
    pub proposal: FixProposal,
    pub reason: String,
    pub rejected_at: u64,
}

#[derive(Debug, Clone, Default)]
pub struct FixStats {
    pub total_proposals: u64,
    pub applied: u64,
    pub rejected: u64,
    pub total_cost_saved_usd: f64,
}
```

### 新增 MCP 工具

```rust
// 在 register_defaults() 中增加:

self.register_tool(SecurityMcpTool::new(
    "fix_and_verify",
    "Scan for vulnerabilities, synthesize fixes, verify each fix (rescan + no new medium + lint), then apply. Returns fixed code with dollar-cost estimates.",
    SecurityToolCategory::VulnerabilityScan,
    fix_and_verify_handler,
)).ok();

self.register_tool(SecurityMcpTool::new(
    "dollar_cost_estimate",
    "Estimate the financial exposure of security findings using IBM/NIST cost data. Returns best/likely/worst case estimates per finding.",
    SecurityToolCategory::ComplianceCheck,
    dollar_cost_estimate_handler,
)).ok();
```

### 修复验证流程

```rust
impl FixVerifier {
    /// 验证修复: 三步门控
    pub fn verify_fix(
        &self,
        original_target: &str,
        fixed_code: &str,
        registry: &mut SecurityMcpToolRegistry,
    ) -> FixVerificationResult {
        // Step 1: rescan — 对修复后代码重新扫描
        //   原问题 category 在 fixed_code 中是否还存在?
        // Step 2: no_new_medium — 检查是否引入新 >=Medium 发现
        // Step 3: lint — 代码质量检查
        // 三步全过 → Verified, 任一失败 → Rejected (含原因)
    }
}
```

### 生产接线 (R-P79)

**接线点**: `nt_shield_mcp_security.rs` 的 `execute_tool()` 后处理

```rust
// 在 execute_tool() 中, scan 结果后自动触发修复提议:
pub fn execute_tool(&mut self, name: &str, ctx: &SecurityMcpContext) -> Result<SecurityMcpResponse, String> {
    let response = /* 现有扫描逻辑 */;

    // P0-② 接线: 扫描发现 ≥ High → 自动生成修复提议
    if name == "security_health_check" && response.risk_score > 0.5 {
        for finding in &response.findings {
            if finding.severity.numeric() >= FindingSeverity::High.numeric() {
                let proposal = synthesize_fix(finding, &ctx.target);
                self.fix_pipeline.proposals.push(proposal);
            }
        }
    }

    Ok(response)
}
```

### 测试计划

| 测试 | 验证点 |
|------|--------|
| `test_synthesize_fix_from_finding` | 从 High finding 生成修复提案 |
| `test_verify_fix_passes` | 修复通过 rescan+no_new+lint → Verified |
| `test_verify_fix_fails_rescan` | 修复后原问题仍在 → Rejected |
| `test_dollar_cost_estimate` | Critical finding → worst_case > $10K |
| `test_fix_pipeline_stats` | 统计正确累加 |

---

## P0-③: Context Shunt (Token Optimization)

**来源**: spotify-portal (PreToolUse hooks → worker model delegation, 90% savings)
**目标模块**: `context_window.rs` (强化现有上下文窗口)
**现状**: ContextWindow 是被动追踪 (observe/attend/recent), 无主动分流
**缺口**: 大文件读取自动分流到 worker model

### 文件变更

**文件**: `neotrix-core/src/unified/core/nt_core_self/context_window.rs`

```rust
// ── 新增: Context Shunt (吸收 spotify-portal 模式) ──

/// 分流决策 — PreToolUse hook 的判定结果
#[derive(Debug, Clone)]
pub enum ShuntDecision {
    /// 直接处理 (内容在阈值内)
    Inline,
    /// 分流到 worker model (内容超阈值)
    ShuntToWorker {
        worker_model: String,
        reason: String,
    },
    /// 截断处理 (只取前 N 行)
    Truncate { max_lines: usize },
}

/// Context Shunt — 吸收 spotify-portal 模式:
/// PreToolUse hook 拦截大文件读取, 分流到 worker model 处理。
pub struct ContextShunt {
    /// 触发分流的行数阈值 (spotify default: 350 lines)
    pub shunt_threshold_lines: usize,
    /// 触发分流的 token 阈值
    pub shunt_threshold_tokens: usize,
    /// Worker model 名称 (spotify: gemini-2.5-flash)
    pub worker_model: String,
    /// 分流统计
    pub stats: ShuntStats,
}

#[derive(Debug, Clone, Default)]
pub struct ShuntStats {
    pub total_checks: u64,
    pub inlined: u64,
    pub shunted: u64,
    pub truncated: u64,
    pub tokens_saved: u64,
}

impl ContextShunt {
    pub fn new() -> Self {
        Self {
            shunt_threshold_lines: 350,
            shunt_threshold_tokens: 10_000,
            worker_model: "gemini-2.5-flash".to_string(),
            stats: ShuntStats::default(),
        }
    }

    /// PreToolUse hook: 拦截工具调用前的上下文准备
    pub fn pre_tool_use(&mut self, tool_name: &str, content: &str) -> ShuntDecision {
        self.stats.total_checks += 1;

        let line_count = content.lines().count();
        let token_est = content.len() / 4;

        // cat/head/tail 大文件 → 分流
        if (tool_name == "cat" || tool_name == "head" || tool_name == "tail")
            && (line_count > self.shunt_threshold_lines
                || token_est > self.shunt_threshold_tokens)
        {
            self.stats.shunted += 1;
            self.stats.tokens_saved += token_est as u64;
            return ShuntDecision::ShuntToWorker {
                worker_model: self.worker_model.clone(),
                reason: format!(
                    "File too large for context: {} lines, ~{} tokens (threshold: {} lines)",
                    line_count, token_est, self.shunt_threshold_lines
                ),
            };
        }

        // grep/rg 结果超阈值 → 截断
        if (tool_name == "grep" || tool_name == "rg")
            && line_count > self.shunt_threshold_lines
        {
            self.stats.truncated += 1;
            return ShuntDecision::Truncate {
                max_lines: self.shunt_threshold_lines,
            };
        }

        self.stats.inlined += 1;
        ShuntDecision::Inline
    }

    /// Worker model 处理: 把大文件发给便宜模型, 返回摘要
    pub async fn delegate_to_worker(
        &self,
        content: &str,
        instruction: &str,
    ) -> Result<String, String> {
        // 调用 worker_model (gemini-2.5-flash) 处理大文件
        // 返回摘要 (token 远小于原文)
        todo!("worker model delegation via LlmProvider")
    }
}
```

### 生产接线 (R-P79)

**接线点**: `engine_core.rs` 的 `build_context()` 方法

```rust
// 在 ReasoningEngine::build_context() 中, 工具结果处理前插入 shunt:
fn build_context(&mut self, tool_results: &[(String, String)]) -> String {
    let mut context = String::new();
    for (tool_name, content) in tool_results {
        // P0-③ 接线: Context Shunt
        match self.context_shunt.pre_tool_use(tool_name, content) {
            ShuntDecision::Inline => {
                context.push_str(content);
            }
            ShuntDecision::ShuntToWorker { worker_model, reason } => {
                log::info!("shunting to {}: {}", worker_model, reason);
                if let Ok(summary) = self.context_shunt.delegate_to_worker(content, "summarize key points").await {
                    context.push_str(&summary);
                }
            }
            ShuntDecision::Truncate { max_lines } => {
                let truncated: String = content.lines().take(max_lines).collect::<Vec<_>>().join("\n");
                context.push_str(&truncated);
                context.push_str("\n... [truncated]");
            }
        }
    }
    context
}
```

### 测试计划

| 测试 | 验证点 |
|------|--------|
| `test_shunt_inline_small_file` | <350 行 → Inline |
| `test_shunt_delegates_large_file` | >350 行 cat → ShuntToWorker |
| `test_shunt_truncates_grep` | >350 行 grep → Truncate |
| `test_shunt_stats` | 统计正确累加 |
| `test_tokens_saved` | 分流后 tokens_saved > 0 |

---

## P1-④: Bitemporal Knowledge Graph

**来源**: utopia (valid_time + belief_time + forward-chaining + conflict resolution)
**目标模块**: `nt_temporal_facts.rs` (强化现有时序事实)
**现状**: valid_from/until + supersession chains + contradiction edges
**缺口**: 无 believed_at (transaction time), 无 forward-chaining rules, 无 3-mode conflict resolution

### 文件变更

**文件**: `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_historian/nt_temporal_facts.rs`

```rust
// ── 新增字段: believed_at (transaction time) ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemporalFact {
    // ... 现有字段 ...
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub valid_from: i64,
    pub valid_until: Option<i64>,
    pub superseded_by: Option<String>,
    pub supersedes: Option<String>,
    pub contradicted_by: Vec<String>,
    pub created_at: i64,
    pub source: String,
    // ── P1-④ 新增 ──
    /// 事务时间: 系统何时相信此事实 (belief_time)
    pub believed_at: i64,
}

// ── 新增: 冲突解决模式 ──

/// 冲突解决策略 (吸收 utopia 3-mode conflict resolution)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictResolution {
    /// 关闭旧版本, 保留两个版本并存
    CloseOldKeepBoth,
    /// 保留旧版本, 拒绝新版本
    RejectNew,
    /// 接受新版本, 关闭旧版本 (默认 supersede)
    AcceptNewCloseOld,
}

// ── 新增: Forward-chaining rules ──

/// 前向链规则 — 事实 A 成立时自动推导事实 B
#[derive(Debug, Clone)]
pub struct ForwardChainRule {
    pub rule_id: String,
    pub name: String,
    /// 前件: (subject, predicate, object) 模式匹配
    pub antecedent: FactPattern,
    /// 后件: 推导出的新事实模板
    pub consequent: FactTemplate,
    /// 规则是否激活
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct FactPattern {
    pub subject_pattern: Option<String>,  // None = 匹配任意
    pub predicate_pattern: Option<String>,
    pub object_pattern: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FactTemplate {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}
```

### 新增方法

```rust
impl TemporalFactLedger {
    /// P1-④: 带 believed_at 的 add_fact
    pub fn add_fact_with_belief(
        &self,
        subject: &str,
        predicate: &str,
        object: &str,
        valid_from: Option<i64>,
        valid_until: Option<i64>,
        believed_at: Option<i64>,
        source: &str,
    ) -> Result<TemporalFact, String> {
        // believed_at 默认 = now_ts()
        // INSERT 增加 believed_at 字段
    }

    /// P1-④: 冲突解决 (3-mode)
    pub fn resolve_conflict(
        &self,
        existing_id: &str,
        new_object: &str,
        resolution: ConflictResolution,
        source: &str,
    ) -> Result<ConflictAction, String> {
        match resolution {
            ConflictResolution::AcceptNewCloseOld => {
                self.supersede(existing_id, new_object, None, source)
                    .map(|f| ConflictAction::Superseded(f))
            }
            ConflictResolution::RejectNew => {
                Ok(ConflictAction::Rejected)
            }
            ConflictResolution::CloseOldKeepBoth => {
                // 旧版本 valid_until 截断, 新版本正常插入
                self.add_fact(
                    &self.get_fact(existing_id)?.unwrap().subject,
                    &self.get_fact(existing_id)?.unwrap().predicate,
                    new_object, None, None, source,
                ).map(|f| ConflictAction::BothKept(f))
            }
        }
    }

    /// P1-④: 前向链推导
    pub fn apply_forward_chain(
        &self,
        rule: &ForwardChainRule,
    ) -> Result<Vec<TemporalFact>, String> {
        // 1. 查询所有匹配 antecedent 的事实
        // 2. 为每个匹配事实, 用 consequent 模板生成新事实
        // 3. 调用 add_fact_with_belief 写入
        // 4. 返回所有新推导出的事实
    }

    /// P1-④: 点时刻查询 (增强: 支持 believed_at 过滤)
    pub fn query_valid_at_with_belief(
        &self,
        valid_ts: i64,
        belief_ts: i64,
    ) -> Result<Vec<TemporalFact>, String> {
        // WHERE valid_from <= valid_ts
        //   AND (valid_until IS NULL OR valid_until > valid_ts)
        //   AND believed_at <= belief_ts
    }
}
```

### Schema 变更

```sql
-- 在 CREATE_SQL 中增加:
ALTER TABLE temporal_facts ADD COLUMN believed_at INTEGER NOT NULL DEFAULT 0;

-- 新增规则表
CREATE TABLE IF NOT EXISTS forward_chain_rules (
    rule_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    antecedent_json TEXT NOT NULL,
    consequent_json TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1
);
```

### 生产接线 (R-P79)

**接线点**: `nt_memory_kb` 的 `ingest()` 方法

```rust
// KB ingest 时自动写入 believed_at:
pub fn ingest(&mut self, node_id: &str, content: &str, source: &str) {
    self.temporal_ledger.add_fact_with_belief(
        node_id, "content", content,
        None, None, None, source,  // believed_at = now
    ).ok();
}
```

### 测试计划

| 测试 | 验证点 |
|------|--------|
| `test_believed_at_defaults_to_now` | 默认 believed_at = current time |
| `test_query_valid_at_with_belief` | 双时间过滤正确 |
| `test_conflict_resolution_3_mode` | 3 种冲突解决策略正确 |
| `test_forward_chain_triggers` | 前件匹配 → 自动推导后件 |
| `test_forward_chain_disabled_rule` | 禁用规则不触发 |

---

## P1-⑤: Scope-Based Agent Claims

**来源**: foremerge (semantic scope claims + ChangeSet + persistent registry)
**目标模块**: `coordinator.rs` (强化现有 MultiAgentCoordinator)
**现状**: 简单的 register_agent + execute_tasks, 无 scope 语义
**缺口**: 无 scope-based claims, 无 ChangeSet pattern, 无 persistent agent registry

### 文件变更

**文件**: `neotrix-core/src/unified/layers/cognition/nt_core/nt_core_parallel/coordinator.rs`

```rust
// ── 新增: Scope-Based Claims (吸收 foremerge 模式) ──

/// 语义范围声明 — 12 种 scope 类型 (吸收 foremerge)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    ApiSurface,      // API 表面
    Schema,          // 数据库 schema
    Config,          // 配置文件
    BusinessLogic,   // 业务逻辑
    Test,            // 测试代码
    Documentation,   // 文档
    Security,        // 安全相关
    Performance,     // 性能相关
    Ui,              // UI 界面
    Data,            // 数据层
    Integration,     // 集成点
    Migration,       // 迁移脚本,
}

/// 语义范围声明 — agent 声明其工作范围
#[derive(Debug, Clone)]
pub struct ScopeClaim {
    pub claim_id: String,
    pub agent_id: String,
    pub kind: ScopeKind,
    pub description: String,
    pub created_at: u64,
    /// 声明是否仍然活跃 (agent 可释放声明)
    pub active: bool,
}

/// ChangeSet — agent 的原子变更集 (foremerge pattern)
#[derive(Debug, Clone)]
pub struct ChangeSet {
    pub changeset_id: String,
    pub agent_id: String,
    pub model: String,              // 使用的模型 (claude/codex/gpt)
    pub claims: Vec<String>,        // 关联的 claim_ids
    pub files_changed: Vec<String>, // 变更的文件列表
    pub git_fingerprint: String,    // Git commit SHA
    pub created_at: u64,
    pub status: ChangeSetStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeSetStatus {
    Proposed,
    Verified,
    Accepted,
    Rejected,
}

/// Agent 注册表 — 跨 session 持久化
pub struct AgentRegistry {
    pub agents: HashMap<String, AgentRegistration>,
    pub claims: Vec<ScopeClaim>,
    pub changesets: Vec<ChangeSet>,
}

#[derive(Debug, Clone)]
pub struct AgentRegistration {
    pub id: String,
    pub model: String,
    pub capabilities: Vec<String>,
    pub registered_at: u64,
    pub last_active: u64,
    pub active_claims: Vec<String>,
}
```

### 新增方法

```rust
impl AgentRegistry {
    pub fn new() -> Self { /* ... */ }

    /// 注册 agent (持久化)
    pub fn register(&mut self, id: &str, model: &str, caps: Vec<String>) { /* ... */ }

    /// 发布 scope claim
    pub fn claim_scope(
        &mut self,
        agent_id: &str,
        kind: ScopeKind,
        description: &str,
    ) -> Result<ScopeClaim, String> {
        // 检查是否有冲突声明 (同 kind + 重叠描述)
        // 冲突 → 生成 advisory warning (不阻塞)
        // 返回新 claim
    }

    /// 检查声明冲突
    pub fn check_conflicts(&self, new_claim: &ScopeClaim) -> Vec<ScopeConflict> {
        // 同 kind 的现有活跃声明 → advisory
        // 重叠描述 → warning
        // 永不 hard lock (foremerge 原则)
    }

    /// 发布 ChangeSet
    pub fn publish_changeset(
        &mut self,
        agent_id: &str,
        model: &str,
        claims: Vec<String>,
        files: Vec<String>,
        fingerprint: &str,
    ) -> Result<ChangeSet, String> { /* ... */ }

    /// 验证 ChangeSet
    pub fn verify_changeset(&mut self, cs_id: &str) -> Result<(), String> { /* ... */ }

    /// 接受 ChangeSet
    pub fn accept_changeset(&mut self, cs_id: &str) -> Result<(), String> { /* ... */ }

    /// 释放 scope claim
    pub fn release_claim(&mut self, claim_id: &str) { /* ... */ }
}
```

### 生产接线 (R-P79)

**接线点**: `MultiAgentCoordinator::execute_tasks()` 中自动注册 claims

```rust
// 在 execute_tasks() 中, agent 开始工作前自动声明 scope:
pub async fn execute_tasks(&self, tasks: &[Task]) -> Vec<AgentResult> {
    for task in tasks {
        // 根据 task 内容推断 scope kind
        let scope = infer_scope_kind(&task);
        self.registry.claim_scope(&task.assigned_agent, scope, &task.description).ok();
    }
    // ... 现有执行逻辑 ...
}
```

### 测试计划

| 测试 | 验证点 |
|------|--------|
| `test_claim_scope_no_lock` | 冲突声明 → advisory warning, 不阻塞 |
| `test_changeset_lifecycle` | Proposed → Verified → Accepted |
| `test_agent_registry_persistent` | 注册信息可序列化/反序列化 |
| `test_scope_conflict_detection` | 同 kind 冲突被检测 |

---

## P1-⑥ + P2-⑦: 略 (HNSW + Shadow Agents)

### P1-⑥ Hybrid HNSW

**目标**: `nt_memory_sweep_20260815.rs` 的 `RetrievalMatrix`
**变更**: 增加 `hnsw_index: Option<HnswIndex>` 字段, `hybrid_search()` 支持 `use_ann: bool` 参数
**实现**: 用 `hnsw-rs` crate (Rust 原生 HNSW), 不引入 C++ 依赖
**接线**: `hybrid_search()` 内部: if use_ann { hnsw.search() } else { cosine scan }

### P2-⑦ Shadow Agents

**目标**: `coordinator.rs` 的 `MultiAgentCoordinator`
**变更**: 增加 `shadows: Vec<ShadowAgent>` + `heartbeat_interval` + `activation_probability`
**实现**: 吸收 pi-shadow-mind 模式 — probabilistic activation, report_to_main
**接线**: `execute_tasks()` 后触发 shadow heartbeat check

---

## 实施顺序与依赖

```
Week 1 (P0):
  Day 1-2: P0-① Runtime Anti-Hallucination (auto_crystallizer.rs)
  Day 3-4: P0-② Security Fix Pipeline (nt_shield_mcp_security.rs)
  Day 5:   P0-③ Context Shunt (context_window.rs)
  Day 6-7: 全量回归 + cargo check + cargo test

Week 2 (P1):
  Day 1-2: P1-④ Bitemporal KG (nt_temporal_facts.rs)
  Day 3-4: P1-⑤ Scope-Based Claims (coordinator.rs)
  Day 5-6: 全量回归 + cargo check + cargo test

Week 3 (P2):
  Day 1-2: P1-⑥ Hybrid HNSW (nt_memory_sweep_20260815.rs)
  Day 3-4: P2-⑦ Shadow Agents (coordinator.rs)
  Day 5-7: 全量回归 + C4 验证 + 能力树注册 (R-P100)
```

---

## 验收标准

| 增强 | C0 (编译) | C1 (单测) | C2 (集成) | C3 (benchmark) | C4 (生产接线) |
|------|-----------|-----------|-----------|----------------|---------------|
| P0-① Anti-Hallucination | cargo check pass | 7 tests pass | engine::reason() 集成 | — | auto_crystallizer ✓ |
| P0-② Security Fix | cargo check pass | 5 tests pass | scan→fix 流水线 | — | mcp_security ✓ |
| P0-③ Context Shunt | cargo check pass | 5 tests pass | engine::build_context 集成 | token savings % | context_window ✓ |
| P1-④ Bitemporal KG | cargo check pass | 5 tests pass | KB ingest 集成 | query latency | temporal_facts ✓ |
| P1-⑤ Scope Claims | cargo check pass | 4 tests pass | parallel executor 集成 | — | coordinator ✓ |
| P1-⑥ HNSW | cargo check pass | 3 tests pass | RetrievalMatrix 集成 | recall@10 vs cosine | sweep_20260815 ✓ |
| P2-⑦ Shadow Agents | cargo check pass | 3 tests pass | parallel executor 集成 | overhead % | coordinator ✓ |

**总计**: 32 个新测试, 7 个增强模块, 0 个新模块 (全部强化现有节点)
