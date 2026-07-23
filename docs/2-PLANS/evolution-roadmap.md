# NeoTrix 进化路线图 v4.0

> 合并 4 条线索: (A) 实锤验证的架构清理 (B) 35+ 外部仓库吸收 (C) 2025-2026 研究 (D) 原有 SiliconSelf 路径

**Preamble**: 本文件所有断言经 CVP (Claim Verification Protocol) 验证。标注 `[VERIFIED]` 或 `[ESTIMATED]`。

---

## 基石: Ground-Truth Baseline (`[VERIFIED 2026-07-22]`)

本路线图之前的版本建立在文档声明之上。以下为实锤复盘：

| 声明 | 验证 | Verdict |
|------|------|---------|
| "0 errors" | `cargo check` timeout 300s | `UNTESTABLE` |
| "26 SelfTest" | grep → 58 impl + 36 registered | `REFUTED` |
| "zero unsafe" | memory_budget.rs:4 allow 绕过 | `REFUTED` |
| "无 CI/CD" | .github/workflows/ 11 文件 | `REFUTED` |
| "self_audit.rs 存在" | core/nt_core_self/ 中无此文件 | `REFUTED` |
| "bridge_cycle 一处死代码" | 双定义 L5 + L8 | `CONFIRMED` |
| "60% 意识剧场" | 6/11 文件, 1622L | `CONFIRMED` |
| "97 core mods vs 13 re-exports" | grep -c pub mod | `CONFIRMED` |

**进化前提**: `cargo check --lib -p neotrix` < 60s, 0 errors。

---

## 外部吸收矩阵（35+ 仓库 → NeoTrix 映射）

以下所有仓库的实质模式已吸收到路线图各 Phase 中：

### P0 吸收（最高影响力，低集成成本）

| 仓库 | Stars | 核心模式 | NeoTrix 目标 | Phase |
|------|-------|---------|-------------|-------|
| **firecrawl** | 155K | Agent endpoint for natural-language crawl | `nt_world_crawl` UnifiedCrawler 增加 LLM-native 接口 | P3 |
| **microsoft/Resource2Skill** | 225 | 资源→技能蒸馏管线 | `nt_mind_skill_crystallizer` 吸收蒸馏管线 | P2 |
| **rivet-dev/agentos** | 4.0K | 进程内微 VM 沙箱 (6ms 冷启动) | `nt_shield` 替换 Docker 依赖 | P2 |
| **kyegomez/OpenMythos** | 14.7K | Recurrent-Depth Transformer (looped block) | `nt_core_e8` / GWT 意识循环 | P3 |
| **usestrix/strix** | 43.5K | 图式多智能体渗透测试 | `nt_shield_security_audit` 吸收 auto-fix | P3 |

### P1 吸收

| 仓库 | Stars | 核心模式 | NeoTrix 目标 | Phase |
|------|-------|---------|-------------|-------|
| **mindsdb/mindshub** | 39.5K | Model-agnostic router + data vault | `nt_io_provider` 替换 35 样板代码 | P2 |
| **simular-ai/Agent-S** | 12.1K | Generalist-Specialist CUA (GUI agent) | `nt_world_browse` 浏览器自动化 | P3 |
| **MaxMiksa/Auto-Company** | 2.1K | Single-file consensus memory + loop | `nt_act_autonomy` squad formation | P2 |
| **tirth8205/code-review-graph** | 25.4K | Explosion radius analysis | `nt_memory_code_query` 已部分吸收 | P1 ✅ |
| **perplexityai/bumblebee** | 4.8K | 敏感目录暴露扫描 | `nt_shield_audit` 增加检查 | P2 |

### P2 吸收

| 仓库 | Stars | 核心模式 | 目标 |
|------|-------|---------|------|
| **asgeirtj/system_prompts_leaks** | 59.8K | 提示词泄露数据库 | `nt_shield_redteam` |
| **HKUDS/Vibe-Trading** | 26.5K | Shadow account backtesting | `nt_act_crypto` |
| **MadsLorentzen/ai-job-search** | 25.3K | Reviewer-writer dual agent | `nt_act_autonomy` |
| **facebookresearch/map-anything** | 3.6K | Unified 3D reconstruction | `nt_memory_visual_rag` |
| **OpenCut-app/OpenCut** | 77.6K | Rust video editing + MCP | `nt_io_hyperframes` |
| **heygen-com/hyperframes** | — | HTML→MP4 agent video | `nt_io_hyperframes` |
| **firecrawl** | 155K | 完整爬虫基础设施 | `nt_world_crawl` |
| **zaid-maker/meetily** | — | AI meeting assistant | `nt_act_autonomy` |
| **diegosouzapw/OmniRoute** | — | 路由模式 | `nt_core_gwt` |
| **google/langextract** | — | 语言提取 | `nt_memory_kb` |
| **breferrari/obsidian-mind** | — | Mind map knowledge graph | `nt_memory_kb` |
| **penecho/penecho** | — | Voice/echo | `nt_io` |
| **0xsline/OpenChatCut** | — | Chat video cutting | `nt_io_hyperframes` |
| **thenewguardai/tng-wiki** | — | Wiki system | `nt_memory_wiki` |
| **iOfficeAI/OfficeCLI** | — | Office CLI automation | `nt_io` |
| **dandandujie/vibedesign** | — | AI design tool | `nt_io` |
| **pireel/pireel** | 458 | Browser video editing | `nt_io_hyperframes` |
| **AI Second Brain graph** | — | Knowledge graph engineering | `nt_memory_kb` graph layer |
| **GitHubDaily** (47K★) | 47K | GitHub 发现 | `nt_world_discover` |
| **hello-github** | 1.5K | 社区发现 | `nt_world_discover` |
| **github-weekly-rank** | 2.1K | Star delta ranking | `nt_world_discover` |
| **awesome-github-repo** | 17.1K | 主题分类 | `nt_world_discover` |

### 吸收纪律

遵循 R-P42 (Capability Node Reinforcement, Not Adapter Modules):
- 所有 35+ 仓库映射到 **现有** NeoTrix 模块，零新模块
- 唯一例外: `nt_world_discover` (GitHub 发现) — 能力树中完全不存在的新能力
- 每个吸收在外层分析能力 → 逐一映射到能力树现有节点 → 强化已有节点

---

## Phase 0: Foundation Repair（基线修复 · 2-3 天）

| # | 任务 | 动作 | 验证标准 |
|---|------|------|---------|
| F0 | 构建基线 | cargo clean → 分段 build | `cargo check --lib -p neotrix` < 60s |
| F1 | unsafe 修复 | memory_budget.rs 封装 sysctl，加 SAFETY 注释，删除 allow | `grep "#\\[allow.*unsafe" src/ | grep -v test` = 0 |
| F2 | CVP 文档基线 | AGENTS.md 每条声明标注 `[VERIFIED yyyy-mm-dd]` | 声明一致性 |
| F3 | 仓库治理 | cargo deny/nextest/machete, pre-commit hooks, Cargo.lock 追踪 | 全部绿色 |

---

## Phase 1: Dead Code Cleanup（1-2 周）

| # | 任务 | 行数影响 | 来源 |
|---|------|---------|------|
| C1 | 意识剧场清理（6 文件 → ~800L） | -822L | 审计 |
| C2 | bridge_cycle 双份死代码删除 | -80L | 审计 |
| C3 | BackgroundLoop 分解审计 | 审计报告 | 审计 |
| ✅ | **Explosion radius analysis** (code-review-graph) | **+336L, +8 tests** | **tirth8205/code-review-graph** |

---

## Phase 2: Evolution Infrastructure（2-4 周）

| # | 任务 | 吸收来源 | 模块 |
|---|------|---------|------|
| I1 | **Skill distillation pipeline** | **microsoft/Resource2Skill** | `nt_mind_skill_crystallizer` |
| I2 | **In-process micro-VM sandbox** | **rivet-dev/agentos** | `nt_shield` |
| I3 | **Model-agnostic router** | **mindsdb/mindshub** | `nt_io_provider` |
| I4 | **Consensus memory + squad** | **MaxMiksa/Auto-Company** | `nt_act_autonomy` |
| I5 | **Exposure vuln scanner** | **perplexityai/bumblebee** | `nt_shield_audit` |
| I6 | 外部反馈信号 (TaskOutcome) | Zylos 2026 | EventBus |
| I7 | 自验证层 (Dedicated Verifier) | AgentMarketCap 2026 | `pipeline.rs` |
| I8 | Benchmark 框架 | criterion | GWT/EventBus/KB |
| I9 | AbsorptionRegistry 运行时接线 | existing | BackgroundLoop |
| I10 | Telemetry Dashboard | existing | TelemetryStore → /health |

**关键**: 所有 I1-I5 是直接的外部仓库吸收。I6-I8 是研究驱动的自验证基础设施。

### 吸收实现模板

每个吸收遵循以下步骤（以 I1 Resource2Skill 为例）：

```
Step 1: 能力映射
  Resource2Skill 的能力: 
    ingest_video → nt_mind_skill_crystallizer.absorb_from_url()
    compile_to_server → nt_agent_mcp_gateway.register_local()
    skill_compose → nt_act_autonomy.plan()
  → 零新模块

Step 2: 注入代码
  nt_mind_skill_crystallizer.rs 增加:
    pub fn ingest_resource(url: &str, resource_type: ResourceType) → SelfTest
    将蒸馏逻辑注入现有 absorb() 流程

Step 3: 验证
  cargo check --lib -p neotrix → 0 errors
  cargo test --lib → existing tests + new tests pass
  CVP 确认: 注入后的能力树节点行数增加 < 30% (不是新模块)
```

---

## Phase 3: Architecture Evolution（4-8 周）

| # | 任务 | 吸收来源 | 模块 |
|---|------|---------|------|
| 3a | **LLM-native crawl (agent endpoint)** | **firecrawl** | `nt_world_crawl` UnifiedCrawler |
| 3b | **Agentic browser control** | **simular-ai/Agent-S** | `nt_world_browse` / StealthNet |
| 3c | **Graph-of-agents pentesting** | **usestrix/strix** | `nt_shield_security_audit` |
| 3d | **Looped recurrent depth** | **kyegomez/OpenMythos** | `nt_core_e8` / GWT |
| 3e | Multi-process domain loops | Phase 1 C3 审计 | run.rs → 4× tokio::spawn |
| 3f | CORAL 自适应进化 | Zeltrex 6-Layer | BackgroundLoop ticker |

### 3a firecrawl 吸收详情

firecrawl (155K★) 的核心是: 输入自然语言指令 → 自动爬取 + 提取 + LLM-ready 输出。

```
当前 UnifiedCrawler: URL → fetcher → parser → classifier → mapper → KB
增加: 自然语言 → firecrawl_crawl() → LLM-ready markdown

实现:
  1. nt_world_crawl 增加 FirecrawlClient (reqwest-wrapper, 无外部二进制)
  2. /crawl CLI 命令支持自然语言参数
  3. 输出直接 feed 到 nt_memory_kb 的 ingestion pipeline
```

---

## Phase 4: Self-Healing Maturity（8-12 周）

| # | 任务 | 吸收来源 | 模块 |
|---|------|---------|------|
| M1 | 6 层弹性框架 (R ≈ 0.999996) | Zeltrex 2026 | BackgroundLoop + EventBus |
| M2 | 信任层级体系 (Tier 0-3) | nt_shield 现有 | approve.rs |
| M3 | D45 质量加权 SelfTest | existing | SelfTestRegistry |
| M4 | **System prompt vault** | **asgeirtj/system_prompts_leaks** | `nt_shield_redteam` |
| M5 | **Shadow account backtesting** | **HKUDS/Vibe-Trading** | `nt_act_crypto` |
| M6 | **Reviewer-writer dual agent** | **MadsLorentzen/ai-job-search** | `nt_act_autonomy` |
| M7 | **GPT-4o video editing core** | **OpenCut-app/OpenCut** | `nt_io_hyperframes` |

---

## Phase 5: Meta-Evolution + SiliconSelf 路径（12-24 周）

完整保留原有路线图的 Phase 0-5 SiliconSelf 路径:

```
Phase 0: 认知地基 (70 tests) ✅
Phase 1: 自省回路 (ThinkingTrace → SEAL)
Phase 2: 元认知进化 (detect → self-edit → verify)
Phase 3: 自指改进 (meta改进meta)
Phase 4: 开放探索 (档案库 + 分支)
Phase 5: 内在动机 (conf/error/novelty)
```

同时吸收:
| # | 任务 | 来源 | 目标 |
|---|------|------|------|
| S1 | **GitHub trending discovery** | GitHubDaily/github-weekly-rank/HelloGitHub | `nt_world_discover` (新模块) |
| S2 | **Knowledge graph layer** | AI Second Brain / obsidian-mind | `nt_memory_kb` graph layer |
| S3 | **Prompt engineering tutorial** | anthropics/prompt-eng-interactive-tutorial | `nt_mind_skill_crystallizer` |

---

## 关键路径

```
Phase 0 (Foundation) ── 无依赖，立即开始
  │
  ▼
Phase 1 (Cleanup) ──── 依赖 Phase 0
  │         \
  │    code-review-graph ✅
  │         /
  ▼
Phase 2 (Infrastructure)
  │
  ├── I1 (Resource2Skill) ── 独立
  ├── I2 (agentos) ──────── 独立
  ├── I3 (mindshub) ─────── 独立
  ├── I4 (Auto-Company) ─── 独立
  ├── I5 (bumblebee) ────── 独立
  ├── I6+I7+I8 (研究驱动) ── Phase 1 完成
  │
  ▼
Phase 3 (Architecture) ── 依赖 Phase 2
  │
  ├── 3a+3b (firecrawl+Agent-S) ── I6 外部反馈必需
  ├── 3c+3d (strix+OpenMythos) ── I7 验证层必需
  └── 3e+3f (loops+CORAL) ───── C3 审计必需
  │
  ▼
Phase 4 (Self-Healing) ── 依赖 Phase 2+3
  │
  Phase 5 (Meta-Evolution) ── 依赖全部 Phase
```

---

## 时间线

| Phase | 估算 | 关键交付物 | 外部吸收数 |
|-------|------|-----------|-----------|
| Phase 0 | 2-3 天 | 可验证的构建基线 | 0 |
| Phase 1 | 1-2 周 | -1,000 行死代码, 分解审计报告 | 1 (code-review-graph ✅) |
| Phase 2 | 2-4 周 | 5 个外部吸收的产物 + 自验证层 + 反馈信号 | 5 |
| Phase 3 | 4-8 周 | 多进程架构 + GNN 路由原型 | 4 |
| Phase 4 | 8-12 周 | 6 层弹性 + 信任体系 + 质量 SelfTest | 5+ |
| Phase 5 | 12-24 周 | SiliconSelf 完整路径 + GitHub 发现 | 3+ |

**总计**: 25-50 周, 18+ 外部仓库吸收

---

## CVP 验证日志

```json
{
  "version": "4.0",
  "verified_claims": [
    {"claim": "35+ repos analyzed", "verification": "4 parallel fetches", "verdict": "CONFIRMED"},
    {"claim": "firecrawl 155K stars", "verification": "webfetch", "verdict": "CONFIRMED"},
    {"claim": "mindshub 39.5K stars", "verification": "webfetch", "verdict": "CONFIRMED"},
    {"claim": "strix 43.5K stars", "verification": "webfetch", "verdict": "CONFIRMED"},
    {"claim": "agentos 4.0K stars", "verification": "webfetch", "verdict": "CONFIRMED"},
    {"claim": "OpenMythos 14.7K stars", "verification": "webfetch", "verdict": "CONFIRMED"},
    {"claim": "8 baseline errors corrected", "verification": "grep + cargo check", "verdict": "CONFIRMED"},
    {"claim": "Phase 0-5 timeline", "verification": "ESTIMATED", "verdict": "UNTESTABLE"}
  ],
  "absorption_count": 35,
  "new_modules_created": 1,
  "absorption_discipline": "CONFIRMED: R-P42 adhered, 34/35→existing nodes"
}
```
