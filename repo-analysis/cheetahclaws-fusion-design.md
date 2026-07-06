# CheetahClaws ↔ NeoTrix 融合设计

> Reference: arXiv 2605.26112 "From Model Scaling to System Scaling" (Shangding Gu, UC Berkeley)
> CheetahClaws: Python-native Claude Code reimplementation, inspired by OpenClaw (NeoTrix)
> Generated: 2026-05-29

---

## 1. 全景差距图

### 架构对比总览

| 维度 | NeoTrix (Rust) | CheetahClaws (Python) |
|------|---------------|----------------------|
| 核心语言 | Rust + TypeScript | Python 3.8+ |
| 代码量 | ~25 modules, ~5700+ lines analyzed | ~85 files, ~40K lines |
| 代理循环 | 事件驱动 + ticker + SEAL loop | Generator yield events |
| 记忆系统 | ReasoningBank (BM25 + vector + Walsh) | 文件 + YAML frontmatter + keyword |
| 任务系统 | GoalLoop (budget/breaker/motivation) | Task DAG (blocks/blocked_by) |
| 上下文压缩 | 无 | 2层: snip → AI summarize |
| 大结果处理 | 无 | Auto-fanout: 并行子摘要 |
| 内存信任 | 无 confidence 字段 | 4字段: confidence/source/last_used/conflict |
| 安全检查 | 无 | CSRF/JWT/bash denylist/cred denylist |
| 规划 | SEAL loop | Plan mode (read-only analysis) |
| 世界模型 | CapabilityVector + RL | 无 |
| 意识模块 | SiliconSelf + GWT | 无 |

### 三级对比表

#### ✅ 我们有，他们没有

| 特性 | NeoTrix 模块 | 重要性 |
|------|-------------|--------|
| 世界模型 + RL 奖励 | SelfIteratingBrain / world_model | 高 |
| CapabilityVector 进化 | ReasoningBrain / core.rs | 高 |
| 22+1 维能力向量 | CapabilityVector | 高 |
| SEAL 自迭代循环 | seal_loop.rs | 高 |
| 意识模型 (SiliconSelf + GWT) | thinking_model/ + consciousness/ | 高 |
| 知识超立方体 (VSA) | hypercube/ | 高 |
| 编排器 + AgentTeam | Orchestrator + AgentTeam | 高 |
| 元认知循环 | MetaCognitionBridge | 中 |
| Agent 协议 UDP 发现 | agent_protocol/ | 中 |
| 断路器 + 速率限制器 | GoalLoop RateLimiter/CircuitBreaker | 中 |
| 多大脑管理 | MultiBrainManager | 低 |

#### ⚠️ 他们有，我们没有

| 特性 | CheetahClaws 位置 | NeoTrix 差距 | Impact |
|------|-------------------|-------------|--------|
| **Auto-fanout** | agent.py:255-278 | 大工具结果直接溢出 ctx | **高** |
| **2层上下文压缩** | compaction.py:75-196 | 无轻量预压缩，直接靠 reasoning | **高** |
| **内存信任元数据** | store.py:31-48 | ReasoningMemory 无 confidence/conflict | **高** |
| **任务依赖图** | task/types.py, store.py:99-138 | GoalLoop 无 blocks/blocked_by | **高** |
| **检查点回滚** | checkpoint/ | SEAL 循环无快速回退 | **中** |
| **只读去重** | agent.py:75-82 | 相同 Read/Glob 浪费 token | **中** |
| **History sanitization** | compaction.py:107-153 | tool_calls 可能不配对 | **中** |
| **提示注入检测** | context.py:22-32 | 无系统上下文安全检查 | **中** |
| **输出 cap 自动降低** | agent.py:341-396 | 溢出错误不自动恢复 | **中** |
| **Plan mode** | agent.py:300-316 | 无只读分析阶段 | **中** |
| **安全加固** | security.md | 无 CSRF/JWT/bash denylist | **中** |
| **Auto-nudge** | agent.py:158-166 | 模型输出纯文本不提示 | **低** |

#### ❌ 都缺

| 特性 | 论文描述 (arXiv 2605.26112) |
|------|---------------------------|
| 上下文治理策略 | §4.1: 每轮 context 应是 selection policy 的输出, 不是固定 buffer |
| 内存实时验证 | §4.2: 检索时应加 staleness penalty + 环境重验证 |
| 技能验证 post-condition | §4.3: 每个 skill 应有 explicit post-condition check |
| 过程指标报告 | §5.1: 应报告 trajectory quality / memory hygiene / context efficiency |
| 纵向评估 | §5.2: 多 episode 而非单场评价 |

---

## 2. 优先级矩阵

### P0 — 立即实现 (高 Impact, 低 Effort)

| ID | 特性 | Impact | Effort | 对接点 |
|----|------|--------|--------|--------|
| P0-1 | **记忆信任元数据** | 高 | 低 (~1h) | ReasoningMemory 加 confidence/source/last_used/conflict_group; 搜索加 recency 权重 |
| P0-2 | **只读去重** | 中 | 低 (~0.5h) | SelfIteratingBrain 或 Agent loop 加 (name, md5(args)) 缓存 |
| P0-3 | **History sanitization** | 中 | 低 (~0.5h) | background_loop 或 agent 消息处理加 tool_call ↔ response 配对检查 |

### P1 — 分批实现 (高 Impact, 中等 Effort)

| ID | 特性 | Impact | Effort | 对接点 |
|----|------|--------|--------|--------|
| P1-1 | **任务依赖图** | 高 | 中 (~3h) | GoalTracker 加 blocks/blocked_by; goal_queue 改为 DAG scheduler |
| P1-2 | **2层上下文压缩** | 高 | 中 (~4h) | background_loop 加 snip 层; ReasoningEngine 加 summarize 压缩 |
| P1-3 | **检查点回滚** | 中 | 中 (~3h) | SelfIteratingBrain 加 snapshot stack; SEAL loop 回退入口 |

### P2 — 文献驱动 (中 Impact, 高 Effort)

| ID | 特性 | Impact | Effort | 对接点 |
|----|------|--------|--------|--------|
| P2-1 | **Auto-fanout** | 高 | 高 (~6h) | 新模块 multi_agent/fanout.rs; 大工具结果自动分片 |
| P2-2 | **提示注入检测** | 中 | 中 (~3h) | context 构造前加 regex 扫描层 |
| P2-3 | **输出 cap 自动降低** | 中 | 中 (~3h) | LLM provider wrapper 加 error parser |
| P2-4 | **安全加固** | 中 | 高 (~8h) | Web UI CSRF, JWT, bash denylist, cred denylist |

### P3 — 论文驱动远期 (取决于实际需求)

| ID | 特性 | Impact | Effort | 依据 |
|----|------|--------|--------|------|
| P3-1 | 上下文治理策略 | 高 | 高 | §4.1: selection policy, 最小上下文 |
| P3-2 | 内存 staleness penalty | 高 | 中 | §4.2: 检索时 trust re-establish |
| P3-3 | 技能 post-condition 验证 | 高 | 高 | §4.3: adaptive routing + verification |
| P3-4 | 过程指标报告 | 中 | 中 | §5.1: 超越 endpoint 成功率 |
| P3-5 | Plan mode | 中 | 低 | agent.py:300 模式切换 |

---

## 3. 每个缺失特性的详细设计

### P0-1: 记忆信任元数据

**现状**: `ReasoningMemory` 无 confidence/source/last_used/conflict_group 字段。
搜索用 BM25 + vector + Walsh，无 freshness/confidence 加权。

**设计修改**:

```rust
// crates/neotrix-types/src/core/memory/mem.rs
// 新增加字段:
pub confidence: f64,              // 0.0–1.0, default 1.0
pub source: MemorySource,         // User | Model | Tool | Consolidator | External
pub last_used_at: i64,            // unix timestamp, 检索时更新
pub conflict_group: String,       // 关联冲突记忆标签
pub verification_time: i64,       // 最后验证时间戳，用于 staleness
```

```rust
// crates/neotrix-types/src/core/memory/search.rs
// 搜索 RRF 融合时增加 freshness 权重:
fn rank_with_freshness(results: Vec<SearchResult>, now: i64) -> Vec<SearchResult> {
    results.iter_mut().for_each(|r| {
        let days_since_use = (now - r.mem.last_used_at) / 86400;
        let freshness_boost = (-0.02 * days_since_use as f64).exp(); // decay per day
        r.score *= freshness_boost * r.mem.confidence;
    });
    results.sort_by(|a, b| b.score.partial_cmp(&a.score));
}
```

**集成点**: ReasoningBank 的 store/consolidate/recall 三处修改。no new file.

**测试策略**:
- `test_freshness_decay`: 旧记忆得分低于新记忆
- `test_confidence_weight`: 高 confidence 记忆优先
- `test_conflict_detection`: 同 conflict_group 标记冲突

### P0-2: 只读去重

**现状**: 相同 tool call (Read/Glob/Grep) 在同一上下文中可能重复执行。

**设计修改**:

```rust
// neotrix-core/src/neotrix/agent/executor.rs 或等效处
struct ReadOnlyDedup {
    seen: HashMap<(String, u64), String>,  // (tool_name, hash_of_args) → cached result
}

impl ReadOnlyDedup {
    fn check(&mut self, name: &str, args: &serde_json::Value) -> Option<String> {
        let hash = hash_args(args);
        if self.seen.contains_key(&(name.to_string(), hash)) {
            return Some("[dedup] same call already executed this turn".into());
        }
        None
    }
    fn insert(&mut self, name: &str, args: &serde_json::Value) {
        let hash = hash_args(args);
        self.seen.insert((name.to_string(), hash), String::new());
    }
    fn clear() { self.seen.clear(); }  // per-turn reset
}
```

**集成点**: Agent loop 中 tool dispatch 之前。per-turn 周期清除。

### P1-1: 任务依赖图

**现状**: GoalQueue 是 Vec<GoalTracker>，优先级排序，无依赖边。

**设计修改**:

```rust
// GoalTracker 新增:
pub blocks: Vec<String>,        // 此任务阻塞的 ID
pub blocked_by: Vec<String>,    // 阻塞此任务的 ID

// GoalLoop 新增 DAG scheduler:
pub goal_graph: DAG<GoalTracker>,

// DAG scheduler algorithm:
// 1. 找出所有 blocked_by 已满足 (即 predecessors 状态为 Achieved) 的节点
// 2. 在这些节点中选择优先级最高的作为 active_goal
// 3. 依赖不满足的任务保持 pending

// 反向边自动注册 (借鉴 CheetahClaws task/store.py:99-138)
```

**集成点**: `goal_loop/`, `GoalTracker struct`, 持久化至 `goals.json`。

### P1-2: 2层上下文压缩

**现状**: 无轻量预压缩。上下文膨胀时只靠模型自身处理。

**设计修改**:

```rust
// neotrix-core/src/neotrix/compaction.rs (新文件)
// Layer 1: snip
fn snip_old_tool_results(
    messages: &mut [Message],
    max_chars: usize,
    preserve_last_n_turns: usize,  // = 6
) -> bool {  // returns true if enough was snipped
    for msg in messages.iter_mut().rev().skip(preserve_last_n_turns) {
        if msg.role == Role::Tool && msg.content.len() > max_chars {
            let half = max_chars / 4;
            msg.content = format!("{}...\n[... {} chars snipped ...]\n{}",
                &msg.content[..half], msg.content.len() - max_chars,
                &msg.content[msg.content.len() - half..]);
        }
    }
}

// Layer 2: AI summarize (通过 ReasoningEngine)
fn compact_via_engine(
    messages: &mut Vec<Message>,
    engine: &ReasoningEngine,
) -> bool {
    // 找到分割点: ~30% 最旧的历史
    // 将旧消息块发送到 LLM 摘要
    // 替换为 summary_msg + ack_msg + recent_messages
    // 确保 tool_call ↔ response 配对不被切开
}
```

**集成点**: `background_loop.rs` ticker 或 `ReasoningEngine` 推理前调用。冲突避免: 与 compression 不重复。

---

## 4. KnowledgeSource 注册方案

若实现以下 P3 特性，需注册新 KnowledgeSource:

| 特性 | KnowledgeSource | 核心向量注入 |
|------|----------------|--------------|
| 上下文治理 | `ContextPolicy` | inference_depth +0.15, synthesis +0.10 |
| 记忆信任 | `TrustworthyMemory` | domain_specificity +0.12, verification +0.18 |
| 安全检查 | `SecurityAudit` | quality_gates +0.20, verification +0.15 |

---

## 5. 分阶段实施计划

### Session 1 (P0: 记忆信任 + 去重 + sanitization)
- P0-1: ReasoningMemory 加 4 字段, search 加 freshness 加权
- P0-2: Agent loop 只读去重
- P0-3: History sanitization (tool_call↔response 配对)
- `cargo check --lib` + 测试

### Session 2 (P1-1: 任务依赖图)
- GoalTracker 加 blocks/blocked_by
- DAG scheduler (依赖满足 → 自动激活)
- 持久化 + 测试

### Session 3 (P1-2: 上下文压缩)
- snip 层 (无 API 成本)
- summarize 层 (通过 ReasoningEngine)
- 触发条件: 70% context window
- 测试

### Session 4 (P1-3: 检查点回滚 + 其余)
- SelfIteratingBrain snapshot stack
- SEAL loop 回退入口
- P2 按需选择

---

## 6. 附录: 论文公式映射

论文公式 (1): ℙ_H = Φ(ℛ, ℳ, 𝒞, 𝒮, 𝒪, 𝒢)

| 组件 | NeoTrix 对应 | 当前状态 |
|------|-------------|----------|
| ℛ 推理 | ReasoningEngine | ✅ Active |
| ℳ 记忆 | ReasoningBank | ✅ Core, missing confidence |
| 𝒞 上下文 | BackgroundLoop context | ❌ No compression |
| 𝒮 技能路由 | SkillsEngine + CapabilityRouter | 🟡 Partial |
| 𝒪 编排 | Orchestrator + GoalLoop | ✅ Core |
| 𝒢 治理 | — | ❌ Missing (P2-4) |

论文指出 CheetahClaws 在 𝒞 (上下文构造) 和 ℳ (记忆信任) 有具体实现优于 NeoTrix，
这正对应 P0-1 和 P1-2。
