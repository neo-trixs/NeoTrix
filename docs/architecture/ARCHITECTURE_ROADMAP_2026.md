# 自主 Agent 架构路线图 2026

> 整合: MAGE · CLAUDE.md 12规则 · 10层Claude环境 · Shadow Mode · Loop Engineering · Blockchain AI · 2026前沿
> 目标: 从可靠人机协作 → 闭环自优化 → 完全自主RSI

---

## 五层架构栈

```
Layer 5: RSI 闭环层        自改进引擎 · DGM 进化搜索 · SEAL
Layer 4: 基础设施层         MCP · Tools · 去中心化GPU · Blockchain
Layer 3: 验证与安全层       Shadow Mode · Loop Engineering · 测试
Layer 2: Agent 操作系统层   CLAUDE.md · 规则引擎 · 子Agent · Skills
Layer 1: 语言/表示层        MAGE · IR · 可验证二进制
```

---

## Layer 1: 语言/表示层

### MAGE 核心定位
MAGE 是整个栈的基础语言，专为 Agent 设计，支持四种可互换形式：
- **人类可读源码** — 开发者编写/审查
- **Agent token-efficient 源码** — LLM 友好（GPT-4o 示例仅 41 tokens）
- **声明式神经网络定义** — 原生 AI 语义
- **二进制 IR** — 可验证、可部署至区块链

### 与传统语言的对比

| 维度 | Python/Rust | MAGE |
|------|-------------|------|
| Token 效率 | 高 (千级) | 极低 (十级) |
| 可验证性 | 无原生 | 内置 (二进制 IR) |
| 自修改安全 | 危险 | 编译器保障 |
| 区块链部署 | 复杂 | 原生 |
| 学习曲线 | 熟知 | 新语言 |

### 实施建议
- **新项目**：核心 Agent 组件优先使用 MAGE
- **存量系统**：外围集成保留 Python/Rust，通过 FFI/IR 桥接
- **验证管道**：MAGE → 编译 → 二进制 IR → 形式化验证 → 部署

---

## Layer 2: Agent 操作系统层

### CLAUDE.md 规则系统

12条核心规则（保持 <200 行，理想 60 行）：

```
┌─────────────────────────────────────────────┐
│  规则 1: Think Before Coding               │
│  规则 2: Simplicity First                  │
│  规则 3: Surgical Changes                  │
│  规则 4: Fail Loudly                        │
│  规则 5: Hard Token Budgets                │
│  规则 6: One Agent One Directory           │
│  规则 7: Verification Loop                 │
│  规则 8: Plan → Execute → /test            │
│  规则 9: Keep PROGRESS.md                  │
│  规则 10: Parallel Dispatch                │
│  规则 11: Dependency-Aware Batching        │
│  规则 12: Compile-First Verification       │
└─────────────────────────────────────────────┘
```

拆分策略：`.claude/rules/*.md` 带 `paths` glob，避免上下文污染。

### 10层环境快速搭建

| # | 层 | 优先级 | 描述 |
|---|-----|--------|------|
| 1 | **Projects + Structured Briefs** | P0 | XML标签: task/context/success_criteria/constraints |
| 2 | **MCP 服务器** | P0 | GitHub、Context7、Playwright、Sentry、Filesystem |
| 3 | **子Agent + Skills** | P0 | Researcher/Writer/Reviewer/Security 角色分离 |
| 4 | **Commands** | P0 | `/plan` `/compact` `/test` `/goal` |
| 5 | **Operating Manual** | P0 | CLAUDE.md + scoped hooks |
| 6 | **Artifacts** | P1 | 结构化输出/缓存 |
| 7 | **Session Management** | P1 | 上下文窗口优化 |
| 8 | **Observability** | P1 | 错误追踪、Token 使用监控 |
| 9 | **Caching Layer** | P2 | 检索增强生成缓存 |
| 10 | **A/B Testing** | P2 | 规则/提示词版本对比 |

### 子Agent 架构模式

```
User Request
    │
    ▼
Orchestrator Agent
    │
    ├──→ Researcher Agent  (搜索/分析/事实核查)
    ├──→ Writer Agent      (代码/文档生成)
    ├──→ Reviewer Agent    (代码审查/安全检查)
    └──→ Security Agent    (权限/边界/危险模式检测)
```

每个子Agent 有独立 CLAUDE.md（精简版）和 scoped hooks。

---

## Layer 3: 验证与安全层

### Shadow Mode（渐进自主）

```
Level 1: Observe + Propose     ──  分类/起草/自信度评分
Level 2: Draft + Human Review  ──  生成草案，人类确认
Level 3: Limited Action        ──  窄域自动执行（测试通过后解锁）
Level 4: Full Autonomy         ──  完全自主（基准持续达标后）
```

**指标驱动**：
- 准确率 > 90% → 升级权限
- 风险自动标记 + 人工回退 < 5% → 升级
- Shadow Mode 记录：`shadow_log_{session}.json`

### Loop Engineering

```
for each task:
    Plan → Execute → Run Tests
    if tests fail:
        Analyze failure → Fix → Run Tests
        if iterations > MAX:
            Escalate to human
    Commit changes → Update PROGRESS.md
```

**关键参数**：
- `MAX_ITERATIONS = 5`（默认）
- `FAIL_LOUDLY = true`（失败必须报告，不静默忽略）
- `COMPILE_FIRST = true`（编译通过后才运行测试）

### 验证管线（6层，参考 P156 StackedValidation）

| 层 | 检查项 | 短路 |
|----|--------|------|
| 1 Syntax | 结构完整性 | ✅ |
| 2 Type/Safety | 类型 + 危险模式 | ✅ |
| 3 Self-Consistency | 不自相矛盾 | ✅ |
| 4 Regression | 后向兼容 | ✅ |
| 5 Benchmark | 性能阈值 | ✅ |
| 6 Meta | 元精度校准 | ✅ |

---

## Layer 4: 基础设施层

### MCP 工具集成

| 工具 | 用途 | 优先级 |
|------|------|--------|
| GitHub MCP | 代码管理/PR/Issue | P0 |
| Playwright MCP | 浏览器自动化 | P0 |
| Filesystem MCP | 文件操作 | P0 |
| Sentry MCP | 错误追踪 | P1 |
| Context7 MCP | 上下文检索 | P1 |
| Database MCP | 持久化 | P1 |

### 去中心化基础设施（Blockchain AI）

从 Awesome Blockchain AI 仓库提炼的关键方向：

**身份与协调层**
- Agent 注册表（链上 Agent 身份）
- Proof-of-Personhood（人类 vs Agent 区分）
- 自主交易（Fetch.ai、Bittensor、HOL 模式）

**计算与数据层**
- 去中心化 GPU（Akash、Render Network）
- 数据市场（Ocean Protocol）
- 模型权重验证（zk-SNARKs 推理证明）

**经济层**
- Agent 微支付（DeFi rails）
- 激励对齐（Token 经济学）
- 结果验证（智能合约托管）

**验证层**
- MAGE IR → 区块链部署
- 智能合约确保 artifact 不可篡改
- 执行可审计、可追溯

### 2026 趋势
> AI Agent 将成为区块链的主要用户群体。区块链提供 Agent 经济所需的信任、支付、身份三大基础设施。

---

## Layer 5: RSI 闭环层

### 从辅助到自主的四阶段

```
Phase 1: 辅助          ████████░░  当前
Phase 2: 闭环自优化   ██████████  6-18个月
Phase 3: 进化搜索     ██████████  12-24个月
Phase 4: 完全自主     ██████████  18个月+
```

### Phase 1: 辅助（当前）
- Claude 贡献 80%+ 代码（Anthropic 2026 报告）
- 工程师产出提升 8x
- 人类全程审核 + 决策

### Phase 2: 闭环自优化（6-18 个月）
- Loop Engineering：Plan → Execute → Test → Learn
- SelfModifyGuard + Gödel 一致性检查
- 指标驱动的 Shadow Mode 升级
- **关键技术栈**：NeoTrix SEAL（Self-Evolution Architecture Loop）

### Phase 3: 进化搜索（12-24 个月）
参考：Sakana Darwin Gödel Machine (DGM) 2026
- 生成变体 → Benchmark 验证 → 保留精英
- 进化因果追踪（E1，已实现）
- 叠加验证管线（E3，已实现）
- 扩展到架构/训练配方/内核优化

```
DGM 进化循环：
for generation in 0..MAX_GEN:
    variants = mutate(elite_population)
    for variant in variants:
        score = benchmark(variant)
        if score > threshold:
            add_to_population(variant)
    elite_population = select_top_k(population)
```

### Phase 4: 完全自主（18 个月+）
- 多 Agent 协作网络
- Blockchain 经济激励
- 人类角色：Coder → Director/Reviewer → Strategy Setter
- 人类始终保留最终否决权

---

## 实施路线图（实用时间线）

### Week 1：启动
- [x] 创建 CLAUDE.md（12 规则模板）
- [x] 配置 Shadow Mode hooks（Level 1 起步）
- [x] 搭建 /plan → /test 基础指令

### Week 2-4：10层环境
- [x] Projects + Structured Briefs
- [x] MCP 服务器安装（GitHub、Playwright、Filesystem）
- [x] 子Agent 模板（Researcher/Writer/Reviewer）
- [x] Loop Engineering 强制测试循环

### Month 1-3：核心构建
- [ ] 用 MAGE 重写核心 Agent 组件
- [ ] 验证管线集成（6层 StackedValidation）
- [ ] 自动化基准收集（性能基线）
- [ ] Shadow Mode Level 2 解锁

### Month 3-6：自主提升
- [ ] 子Agent + MCP 生态完整集成
- [ ] Shadow Mode Level 3 部分解锁
- [ ] Loop Engineering 迭代次数优化
- [ ] PROGRESS.md 自动维护

### Month 6-12：进化引入
- [ ] DGM 变体生成器（MutationOp 扩展）
- [ ] 进化因果追踪（已实现: EvolutionCausalGraph）
- [ ] 叠加验证管线全自动（已实现: StackedValidationPipeline）
- [ ] Benchmark 驱动选择

### Month 12-18：去中心化
- [ ] Agent 区块链身份注册
- [ ] MAGE IR → 链上部署
- [ ] 多 Agent 经济激励试点
- [ ] Shadow Mode Level 4 评估

### Month 18+：完全自主
- [ ] 持续自进化循环
- [ ] 人类仅战略级监督
- [ ] 去中心化 Agent 经济运行

---

## 风险控制矩阵

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| Agent 产生危险代码 | 中 | 高 | Shadow Mode + 6层验证 + 人类最终否决权 |
| 进化失控 | 低 | 极高 | SelfModifyGuard + Gödel 检查 + 沙箱隔离 |
| 区块链安全漏洞 | 中 | 高 | MAGE 可验证二进制 + 形式化验证 |
| Token 成本失控 | 高 | 中 | Hard Token Budgets + 规则 5 |
| 上下文污染 | 高 | 中 | .claude/rules 拆分 + glob 隔离 |

---

## 与 NeoTrix 现有系统的映射

| 架构层 | NeoTrix 对应模块 | 状态 |
|--------|-----------------|------|
| L5 RSI 闭环 | SEAL (Self-Evolution Loop, SelfModifyGuard) | ✅ |
| L5 进化搜索 | EvolutionCausalGraph, EvolutionPredictor | ✅ P150 |
| L3 验证 | GodelConsistencyChecker, StackedValidationPipeline | ✅ P156 |
| L3 Shadow Mode | SarDiagnostic, SafetyGate | ✅ |
| L3 Loop Engineering | SelfEvolutionLoop, MetaImprovementLoop | ✅ |
| L2 规则系统 | AGENTS.md (12 条第一原理) | ✅ |
| L2 子Agent | TriAgentPipeline, SubAgent system | ✅ |
| L2 记忆 | DecentMem, MemoryOps, TiMem | ✅ |
| L4 MCP | ToolOrchestrator, ToolSafety | ✅ |
| L4 Blockchain | — | ❌ 待实现 |

**当前进度**: 五层架构中 L2-L5 的核心组件已存在。L1 (MAGE) 和 L4 区块链组件待实施。

---

## 关键决策原则

1. **MAGE First** — 新 Agent 组件优先用 MAGE，利用其 token 效率和可验证性
2. **Shadow Mode 渐进** — 从 Observe 起步，指标驱动升级，永不跳过 Level
3. **验证不妥协** — 6层管线始终运行，fail-fast 短路
4. **人类最终否决权** — 任何自主级别，人类可一键回退
5. **经验蒸馏** — 每次会话结束蒸馏新经验到经验树（CLIX 模式）
6. **编译清零** — 代码变更必须编译通过再提交，不拖延错误

---

## 参考资源

- **MAGE**: mage-lang (Anthropic-inspired Agent-native language)
- **CLAUDE.md 12 Rules**: claude-howto 技能栈
- **Shadow Mode**: Anthropic 安全部署框架
- **Loop Engineering**: Verifiable Agent 循环模式
- **Darwin Gödel Machine**: Sakana AI, ICLR 2026
- **Awesome Blockchain AI**: github.com/steven2358/awesome-blockchain-ai
- **NeoTrix Codebase**: EVOLUTION_ROADMAP_v6.md, TODO-ROADMAP.md
