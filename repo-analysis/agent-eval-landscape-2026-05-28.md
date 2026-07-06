# Agent Evaluation Landscape — 2026-05-28

> 最近 7 天 agent evaluate 项目/论文扫描归档。
> 来源: GitHub Trending + arXiv + 多源交叉检索。

---

## 📄 关键论文

### 通用评估框架

| 论文 | 时间 | 核心贡献 |
|------|------|----------|
| **Agentick** (arXiv:2605.06869) | ~05-22 | 统一 RL/LLM/VLM 序列决策基准。37 任务 ×4 难度 ×5 观测模态。GPT-5 mini 领先(0.309 ONS)，PPO 占优规划。推理 harness 放大 3-10× |
| **AlphaEval** (arXiv:2604.12162) | 生产基准 | 7 公司 94 生产任务评估 agent 产品。Claude Code+Opus 4.6 最高 64.41/100。**scaffold 影响 ≥ model** (同一 Opus 4.6: 64.41 vs 53.45) |
| **AgentAtlas** (arXiv:2605.20530) | ~05-27 | 六态控制决策/9 类轨迹失败分类。移除 taxonomy label → 所有模型挤在 0.54-0.62 |
| **From Model Scaling to System Scaling** (arXiv:2605.26112) | ~05-27 | Harness 缩放 > 模型缩放。pass^k 比 pass@1 更反映真实可靠性 |
| **TRACE** (arXiv:2602.21230) | WWW'26 | Deep Research Agent 轨迹级评估, 650 任务含陷阱/路径 |
| **Claw-Eval** (arXiv:2604.06132) | 300 人工任务 | 轨迹不透明漏 44% 安全违规。Pass^3 稳定性评估 |

### 专项评估

| 论文 | 核心贡献 |
|------|----------|
| **Agent-ValueBench** (arXiv:2605.10365) | 28 价值系统, 4335 价值冲突任务。影响: LawKeeper 可借鉴 |
| **AgentProp-Bench** (arXiv:2604.16706) | 2000+ 任务, substring judge κ=0.049(随机水平), 3-LLM ensemble κ=0.432 |
| **General AgentBench** (arXiv:2602.18998) | CMU+Meta: test-time scaling 的 context ceiling + verification gap |
| **Survey on Evaluation of LLM-based Agents** (arXiv:2503.16416) | 首个 Agent 评估综述, 5 维度全景分析 |

---

## 🏗 关键项目

| 项目 | ⭐ | 更新 | 独特价值 |
|------|-----|------|---------|
| **openclaw/clawbench** | 96 | 05-17 | 动力系统诊断(陷阱/极限环/扩散), C(q) 约束指数, 信噪比加权 |
| **Exgentic/exgentic** | 60 | 05-18 | 通用评估框架, 统一接入 tau2/AppWorld/SWE-bench/BFCL 等 7 基准 |
| **microsoft/STATE-Bench** | 25 | 05-22 | 450 企业多轮任务, pass^5 + UX Score |
| **evolvent-ai/Terrarium** | 43 | 05-26 | living environment 多轮数据引擎, Python 任务编程 |
| **evolvent-ai/ClawMark** | 104 | 05-13 | 多日工作基准(100 任务×13 专业), 零 LLM-as-judge |
| **vercel-labs/agent-eval** | 167 | 05-06 | A/B 测试 agent, 控制变量实验 |
| **claw-eval/claw-eval** | 524 | 05-05 | 300 人工验证任务, Completion/Safety/Robustness 三维 |
| **GAIR-NLP/AgencyBench** | 85 | ACL'26 | 6 能力×32 场景×138 任务, 平均 1M tokens |
| **GAIR-NLP/AlphaEval** | new | 生产环境 | 94 生产任务, 14 agent 配置评测 |
| **hkust-nlp/AgentVista** | 46 | 02月 | 209 超难视觉任务, 最好模型仅 27.3% |
| **IsThatYou/auto-bench-audit** | 2 | **05-26** | 自动审计 agent 基准本身(任务歧义/环境冲突/评估 bug) |

---

## 🔍 五大关键趋势

### 1. pass^k 取代 pass@1
τ-bench leaderboard: Opus 4.5 pass@1 赢(0.70), Qwen3.5 pass^4 赢(0.56)。单次跑分误导部署决策。

### 2. Scaffold > Model
AlphaEval 实证: 同一 Opus 4.6 经不同 scaffold 差 11pp (Claude Code 64.41 vs Codex 53.45)。GPT-5.2 经 Claude Code 仅 54.91。

### 3. 基准自身可信度危机
- AgentProp-Bench: substring judge κ=0.049 (随机水平), 3-LLM ensemble 才 κ=0.432
- auto-bench-audit: 自动发现任务歧义/环境冲突
- Claw-Eval: 轨迹不透明漏 44% 安全违规

### 4. 学术 vs 生产鸿沟
最好的 AlphaEval 生产评测仅 64/100, 而学术基准 80%+。生产要求 implicit constraints, multi-modal, long-horizon, domain expertise。

### 5. 轨迹级评估
AgentAtlas 的 6 态控制决策 + 9 类轨迹失败。ClawBench 的动力系统分类(陷阱/极限环/扩散)。TRACE 的轨迹效用+效率+鲁棒性。

---

## 💡 对 NeoTrix 的迁移路径

| 启发 | 优先级 | 对接模块 | 实现思路 |
|------|--------|---------|----------|
| pass^k 可靠性指标 | P1 | BenchmarkSuite | pass^k 替代单次 pass@1, 3 次独立运行 |
| 轨迹级评估 | P1 | pipeline.rs | 在 SEAL pipeline 中注入 AgentAtlas 六态分类 |
| 价值观对齐 | P2 | LawKeeper | 借鉴 Agent-ValueBench 测试框架 |
| 基准审计 | P2 | auto-bench-audit 思路 | 定时自检 BenchmarkSuite 的区分度 |
| 生产级评估 | P2 | AgentTeam | 借鉴 AlphaEval 的 scaffold-aware 评估 |
| 多日工作流 | P2 | background_loop | 借鉴 ClawMark 的 timeline-driven 多阶段任务 |
| living environment | P3 | Terrarium | 可变异环境的连续测试 |
| 自动 eval 监控 | P0 | EvalMonitor+background_loop | 定期扫描新基准, 评估对 NeoTrix 的关联度 |

---

## 📊 当前 NeoTrix 评估覆盖缺口

| 维度 | 当前状态 | 外部标准 | 缺口 |
|------|---------|---------|------|
| pass^k 可靠性 | 单次通过率 | τ-bench, ClawEval 的 Pass^3 | 需 3 次独立运行 |
| 轨迹诊断 | 无 | AgentAtlas 六态/ClawBench 动力系统 | 需注入 pipeline |
| 多模态 | 无 | AgentVista 209 视觉任务 | 低优先级 |
| 生产级 | BenchmarkSuite | AlphaEval 94 任务 | 需生产环境任务 |
| 安全/鲁棒性 | LawKeeper | Claw-Eval 三维评分 | 可补充 robustness 维度 |
| 价值观 | LawKeeper | Agent-ValueBench 4335 任务 | 可补充 align 测试 |
| 基准自身可信度 | 无 | auto-bench-audit | 需定期自检 |
