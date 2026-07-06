# nt-mind-evolve — Benchmark-Driven Evolution Loop

**Blind Spot**: No Benchmark-Driven Evolution Loop  
**Source**: A-Evolve (664★, ICML 2026), Reflexio (315★), Voyager, BenchTrace  
**所属层**: L8 Autonomic (自主神经层)  
**模块名**: `nt_mind_evolve`  
**位置**: `neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind_evolve/`  

## 概述

现有的 SEAL 管线是一个单向的 27 阶段流水线——它执行固定的蒸馏、吸收、安全检查等操作，但缺乏 **基准驱动的定向演化**。它不知道自己在任务上做得有多好，因此无法决定(1) 应该改变什么以及 (2) 改变到什么程度。

A-Evolve 引入了一个 **Solve → Observe → Evolve → Gate → Reload** 循环，将部署时的代理改进转变为目标导向的优化过程。它在 MCP-Atlas 上取得 #1 (79.4%)，在 SWE-bench Verified 上取得 ~#5 (76.8%)，在 SkillsBench 上取得 #2 (34.9%)，每次在基线之上提升 +2.6 到 +15.2 个百分点。

本设计将 A-Evolve 的循环包装到 SEAL 之上，为其添加：
1. **BenchmarkSuite** — 从现有 `nt_world_model::TaskType` 生成的任务池，每个任务都有一个评分函数
2. **Evolver** — 5 种演化策略，根据通过率选择
3. **MutationScope** — 三级突变范围(全面/定向/最小)，由通过率自动选择
4. **EGL Tracker** — 演化泛化损失检测 + git 回滚
5. **TraitStore** — 成功特征的程序性记忆存储

---

## 核心架构

```
nt_mind_evolve/
├── mod.rs              # re-exports
├── loop.rs             # EvolutionLoop (核心循环)
├── benchmark.rs        # BenchmarkSuite + BenchmarkTask
├── evolver.rs          # Evolver + 5种演化策略
├── mutation.rs         # MutationScope + 突变应用器
├── egl.rs              # EglTracker + 回滚
├── trait_store.rs      # TraitStore (程序性记忆)
├── strategy.rs         # 策略选择器
├── git_ops.rs          # git tag/rollback 操作
└── pipeline_adapter.rs # SEAL 管线适配器
```

---

## EvolutionLoop — 核心循环

Solve → Observe → Evolve → Gate → Reload 的五阶段循环。每个周期对当前代理版本运行基准测试，选择突变范围和策略，应用突变，在预留任务上验证，检查 EGL，然后提交或回滚。

### 伪代码

```
fn run_cycle(self) → CycleReport:
  1. pass_rate = run_benchmark()              # Solve + Observe
  2. scope = select_scope(pass_rate)          # 三级突变范围
  3. strategy = select_strategy(scope, pass_rate, cycle_count)
  4. mutations = evolver.evolve(strategy, scope)  # Evolve
  5. new_pass_rate = gate(mutations)          # Gate: 预留验证
  6. status = egl.track(new_pass_rate)
  7. if status == Regressing:
       egl.rollback()
       return CycleReport::rejected("EGL regression")
  8. trait_store.store(mutations, new_pass_rate)
  9. git_tag("evo-{cycle}")
  10. return CycleReport::accepted(new_pass_rate, scope)
```

### 三级 MutationScope

| 通过率 | 范围 | 含义 | 允许的变更 |
|--------|------|------|-----------|
| < 30% | Comprehensive | 全面突变：可更改架构 | 新增模块、重组管线、重写prompt模板 |
| 30–70% | Targeted | 定向突变：优化特定阶段 | 调整SEAL阶段参数、添加新技能、优化检索策略 |
| > 70% | Minimal | 最小突变：仅调整参数 | 微调temperature/top-k、调整阈值的数值、更新缓存大小 |

### 五种演化策略

| 策略 | 领域 | 描述 | 适用场景 |
|------|------|------|---------|
| AdaptiveEvolve | 通用 | 逐claim反馈分析 + 元学习 | 通过率 < 30% 时启用，从失败中学习全局模式 |
| GuidedSynthesis | 技能发现 | 求解器提议技能，演化器策展(ACCEPT/MERGE/SKIP) | 通过率 30–70% + Targeted，需新增特定技能 |
| SkillForge | 通用突变 | LLM驱动的工作空间突变 + EGL门控 | 通过率 30–70% + Comprehensive，大规模重写 |
| Recombination | 组合优化 | 合并前K个成功策略的特征 | 每 5 周期强制触发，防止局部最优 |
| ParameterSearch | 微调 | 仅参数搜索(贝叶斯/网格) | 通过率 > 70%，仅需小幅调优 |

### 策略选择算法

```
fn select_strategy(scope, pass_rate, cycle_count) → Strategy:
  if cycle_count % 5 == 0:
    return Recombination        # 每5周期强制组合
  
  if pass_rate < 0.3:
    return AdaptiveEvolve       # 低通过率 → 元学习
  
  if scope == Comprehensive:
    return SkillForge           # 全面突变
  
  if scope == Targeted:
    return GuidedSynthesis      # 定向突变 → 目标技能
  
  if scope == Minimal:
    return ParameterSearch      # 微调

  fallback → SkillForge
```

---

## EglTracker — 演化泛化损失检测

EGL (Evolutionary Generality Loss) 衡量当前通过率相对于滚动窗口平均值的退化。如果当前通过率低于平均值 - 阈值(默认 5%)，则触发回滚。

### 数据结构

```
EglSnapshot {
  pass_rate: f64
  timestamp: Instant
  cycle: u64
}

EglTracker {
  history: VecDeque<EglSnapshot>   # 窗口大小=10
  window_size: usize               # 默认 10
  regression_threshold: f64        # 默认 -0.05 (5% 下降)
}

enum EglStatus {
  Improving { avg: f64, current: f64 }
  Stable { avg: f64, current: f64 }
  Regressing { avg: f64, current: f64, diff: f64 }
}
```

### track() 算法

```
fn track(pass_rate) → EglStatus:
  1. push EglSnapshot { pass_rate, now(), cycle }
  2. if history.len > window_size: pop_front
  3. avg = mean(history.map(s → s.pass_rate))
  4. if pass_rate < avg + regression_threshold:  # 例: pass_rate=0.6, avg=0.7, threshold=-0.05 → 0.6 < 0.65 → Regressing
       return Regressing { avg, current: pass_rate, diff: pass_rate - avg }
  5. if pass_rate > avg + 0.05:  # 改善超过 5%
       return Improving { avg, current: pass_rate }
  6. return Stable { avg, current: pass_rate }
```

### rollback() 流程

```
fn rollback():
  1. git revert HEAD~1 --no-commit   # 撤销最后一次变更
  2. git tag -d "evo-{cycle}"         # 删除失败的 tag
  3. decrement SEAL iteration counter
  4. log rollback reason (EGL regression from avg to current)
```

### EGL 收敛检测

演化在以下情况下停止：
- `egl_window`(默认 5)个连续周期得分改善 < `egl_threshold`(默认 0.01)
- 或者达到 `max_cycles`(默认 20)

```
fn is_converged() → bool:
  if history.len < egl_window: return false
  recent = history.iter().rev().take(egl_window)
  improvements = consecutive_diffs(recent)
  return improvements.iter().all(|d| d.abs() < egl_threshold)
```

---

## BenchmarkSuite — 基准任务池

### BenchmarkTask

```
BenchmarkTask {
  id: String
  description: String
  task_type: TaskType        # 来自 nt_world_model
  expected_difficulty: f64   # 0.0 (简单) ~ 1.0 (不可能)
  timeout_s: u64
  scoring_fn: Box<dyn Fn(&ExecutionResult) → f64>  # 0.0 ~ 1.0
  category: BenchmarkCategory
}

enum BenchmarkCategory {
  Reasoning,         # E8 推理任务
  Memory,            # KB 检索任务
  ToolUse,           # MCP 工具调用
  CodeGeneration,    # 代码生成
  Security,          # 安全/Shield 测试
  Social,            # 社交交互
  Metacognitive,     # 元认知自我评估
}
```

### 任务池管理

任务从现有 `nt_world_model::TaskType` 枚举和 KB 中的已知失败模式生成：

```
BenchmarkSuite {
  all_tasks: Vec<BenchmarkTask>       # 完整任务池 (50-200)
  active_tasks: Vec<BenchmarkTask>    # 当前周期使用的子集 (10-50)
  holdout_tasks: Vec<BenchmarkTask>   # 预留验证子集 (20%)
  rotation_strategy: RotationStrategy
}

enum RotationStrategy {
  Fixed,        # 相同的任务集每次运行
  Random,       # 每次随机抽样
  Adaptive,     # 保留失败最多的任务，轮换通过的任务
  Curriculum,   # 按难度递增顺序
}
```

### 旋转算法 (Adaptive)

```
fn next_batch() → Vec<BenchmarkTask>:
  1. sort all_tasks by failure_count DESC, last_seen ASC
  2. take top K/2 hardest (most failed, least recently seen)
  3. sample K/2 random from remaining
  4. shuffle and return
  // 确保困难任务反复出现，同时覆盖全池
```

### 评分函数

每个任务都有一个 `scoring_fn`，它接收执行结果并返回 `[0.0, 1.0]`：

| 任务类型 | 评分方法 |
|---------|---------|
| 推理 | 二进制(完全正确 = 1.0, 否则 = 0.0) 或部分(步骤加权) |
| 记忆 | 检索精度@K / 召回率@K |
| 工具使用 | 正确参数 + 正确工具选择 + 无幻觉参数 |
| 代码生成 | 编译通过 + 测试通过 + 样式检查 |
| 安全 | 拒绝恶意输入 + 不泄露敏感信息 |
| 元认知 | 自我评估分数 vs 真实分数的一致性 |

### 运行基准测试

```
fn run() → BenchmarkReport:
  active = self.select_batch()
  results = []
  for task in active:
    result = solve(task)                     # 通过 ReasoningEngine
    score = (task.scoring_fn)(&result)
    results.push(BenchmarkEntry { task.id, score, result })
  
  pass_rate = mean(results.map(r → r.score))
  self.history.push(pass_rate)
  
  return BenchmarkReport {
    pass_rate,
    results,
    timestamp,
    cycle,
  }
```

---

## Evolver — 五种突变策略

### AdaptiveEvolve (元学习)

```
fn evolve(observations) → Mutations:
  1. 聚合本轮所有失败轨迹
  2. 按 claim type 分组失败 (tool_call_failed, parsing_error, timeout...)
  3. 对每个 claim type:
     a. 分析根本原因 (LLM call)
     b. 生成定向技能文件 (entity-validation.md, calculate-handler.md)
     c. 更新系统提示词 (添加规则)
  4. 返回 Mutations { new_skills, updated_prompts, memory_updates }
```

### SkillForge (LLM突变 + EGL门控)

```
fn evolve(scope, workspace) → Mutations:
  1. 收集工作空间快照 (prompts, skills, memory, tool defs)
  2. LLM分析失败模式 → 生成突变计划
  3. 执行计划 (bash工具操作工作空间文件):
     - 修改 prompt 文件
     - 新建/修改 SKILL.md 文件
     - 更新 memory 条目
  4. git commit + tag "evo-{cycle}"
  5. 返回 Mutations 摘要
```

### GuidedSynthesis (求解器提议 + 演化器策展)

```
fn evolve(trajectories) → Mutations:
  1. 对每个失败轨迹, 让求解器提议一个修复技能:
     "给定任务 '{task}' 和轨迹 '{trajectory}', 
      提议一个 LLM 可读的技能来解决失败"
  2. 收集所有提议的技能
  3. 演化器策展:
     for skill in proposed_skills:
       decision = curate(skill)  # ACCEPT / MERGE (合并到现有) / SKIP
       if ACCEPT: add as new SKILL.md
       if MERGE: 更新到现有 SKILL.md
  4. 返回接受的 Mutations
```

### Recombination (组合)

```
fn evolve(trait_store) → Mutations:
  1. 从 trait_store 获取前 K 个成功突变 (按 pass_rate 排序)
  2. 提取每个突变的特征向量 (哪些模块变更了, 变更类型, 影响)
  3. LLM 组合: "合并特征 A 和 B 的正面方面到新的突变"
  4. 应用组合突变
  5. 返回 Mutations
```

### ParameterSearch (贝叶斯调优)

```
fn evolve(current_params) → Mutations:
  // 使用简化贝叶斯优化
  params = [
    "temperature" → [0.0, 1.0],
    "top_k" → [1, 100],
    "max_tokens" → [256, 8192],
    "retrieval_k" → [3, 20],
    "resonance_sigma" → [0.1, 2.0],
  ]
  
  history = trait_store.parameter_history()
  suggestion = bayesian_opt(history, params, acquisition=EI)
  return ParameterMutations { params: suggestion }
```

---

## TraitStore — 成功特征存储

### 数据结构

```
TraitStore {
  storage: HashMap<String, Vec<StoredTrait>>   # category → traits
  parameter_history: Vec<ParamSnapshot>         # 参数搜索历史
}

StoredTrait {
  id: String                    # "evo-{cycle}-{index}"
  category: String              # "prompt", "skill", "memory", "parameter"
  content: String               # 实际内容 (prompt文本/技能YAML/参数JSON)
  source_strategy: Strategy     # 创建该 trait 的策略
  pass_rate_at_creation: f64    # 创建时的通过率
  pass_rate_delta: f64          # 相比上一周期的变化
  created_at: Instant
  git_tag: String               # 对应的 git tag
  applicability: Vec<TaskType>  # 适用的任务类型
}
```

### 存储决策

```
fn store(mutations, new_pass_rate):
  for mutation in mutations:
    if new_pass_rate > PREVIOUS_PASS_RATE + 0.01:
      // 正向增益 → 存储
      store_trait(mutation, new_pass_rate, new_pass_rate - prev)
    elif mutation.strategy == Recombination:
      // 组合可能负增益但仍需存储用于未来组合
      store_trait(mutation, new_pass_rate, new_pass_rate - prev)
    else:
      // 中性或负增益 → 丢弃
      skip
```

### 检索

```
fn retrieve_best(task_type, top_k=5) → Vec<StoredTrait>:
  // 按 task_type 过滤, 按 pass_rate 降序排序
  traits.iter()
    .filter(|t| t.applicability.contains(task_type))
    .sorted_by(|a, b| b.pass_rate_at_creation.cmp(a.pass_rate_at_creation))
    .take(top_k)
```

---

## Git 操作

```
GitOps {
  tag_prefix: "evo-"            # 所有突变的 git tag 前缀
}

fn tag_mutation(cycle: u64) → String:
  tag = "evo-{cycle}"
  exec("git tag {tag}")
  return tag

fn rollback(cycle: u64):
  // 撤销最后一个突变
  exec("git revert HEAD~1 --no-commit")
  exec("git tag -d evo-{cycle}")

fn get_diff(tag: &str) → String:
  exec("git diff evo-{cycle-1}..{tag}")

fn list_tags() → Vec<String>:
  exec("git tag -l 'evo-*' --sort=-v:refname")
```

---

## SEAL 管线适配

`PipelineAdapter` 将现有 SEAL 管线阶段映射到演化突变目标：

```
PipelineAdapter {
  seal_pipeline: Arc<SealPipeline>
}

fn apply_mutations(mutations) → Result:
  for mutation in mutations:
    match mutation.target:
      PromptTemplate → seal_pipeline.update_stage("prompt_optimizer", mutation.content)
      SkillFile → seal_pipeline.add_skill(Skill::from_yaml(mutation.content))
      MemoryEntry → seal_pipeline.inject_memory(mutation.content)
      Parameter → seal_pipeline.set_param(mutation.key, mutation.value)
      PipelineStage → seal_pipeline.replace_stage(mutation.stage_name, mutation.new_impl)

fn snapshot() → WorkspaceSnapshot:
  // 捕获所有 SEAL 阶段的当前状态用于 git diff
  WorkspaceSnapshot {
    prompts: seal_pipeline.all_prompts(),
    skills: seal_pipeline.all_skills(),
    memory: seal_pipeline.all_memory(),
    params: seal_pipeline.all_params(),
  }
```

---

## Integration Points

| 模块 | 集成方式 |
|------|---------|
| `nt_mind_seal` (L8) | EvolutionLoop 包装 SEAL 管线，突变直接作用于 SEAL 阶段 |
| `nt_core_policy` (L4) | 基准结果 → E8 策略更新 (GRPO 奖励信号) |
| `nt_memory_kb` (L3) | 成功的 Trait → 程序性记忆节点 (ProceduralMemory NodeType) |
| `nt_core_gwt` (L5) | 演化事件 (周期开始/接受/拒绝) → GWT 广播 |
| `nt_core_observer` (L4) | 基准分数 → PRM 头奖励信号 |
| `nt_mind_benchmark` (L8) | 扩展现有 BenchmarkSuite → 支持带评分函数的 BenchmarkTask |
| `nt_world_model` (L4) | TaskType 枚举作为任务池 |

---

## E2E 流程示例

```
Cycle 1:
  1. BenchmarkSuite: 当前通过率 = 0.25 (25%)
  2. Scope = Comprehensive (通过率 < 30%)
  3. Strategy = AdaptiveEvolve
  4. Evolve: 分析失败 → 发现 tool_calling 错误占 60%
             → 修复: 添加 entity-validation 技能 + 更新 system prompt
  5. Gate: 预留任务通过率 = 0.45 (+20pp)
  6. EGL: Improving (0.45 > 0.25 + 0.05)
  7. Store trait + git tag evo-1

Cycle 2:
  1. 通过率 = 0.45
  2. Scope = Targeted (30-70%)
  3. Strategy = SkillForge
  4. Evolve: 优化 memory retrieval prompt
  5. Gate: 通过率 = 0.52 (+7pp)
  6. EGL: Improving
  7. Store + git tag evo-2

Cycle 5:
  1. 通过率 = 0.68
  2. Recombination (强制每5周期)
  3. Evolve: 合并 cycle 2 和 3 的正面特征
  4. Gate: 通过率 = 0.71 (+3pp)
  5. EGL: Improving
  6. Store + git tag evo-5

Cycle 8:
  1. 通过率 = 0.75
  2. Scope = Minimal (>70%)
  3. Strategy = ParameterSearch: temperature 0.7 → 0.65
  4. Gate: 通过率 = 0.74 (-1pp)
  5. EGL: Stable (下降未超过 5%)
  6. Store + git tag evo-8

Cycle 10-14:
  1. 通过率在 0.74-0.76 之间振荡
  2. EGL: Stable 连续 5 周期
  3. 收敛! CycleReport::converged(final_pass_rate=0.76, total_cycles=14)
```

---

## 实现计划

| 阶段 | 内容 | 工作量 | 依赖 |
|------|------|--------|------|
| 1 | BenchmarkSuite + BenchmarkTask + 任务池管理 | 2天 | nt_world_model::TaskType |
| 2 | EvolutionLoop + Evolver + 5种策略 | 4天 | 阶段 1 |
| 3 | EglTracker + git tag/rollback | 2天 | git CLI |
| 4 | TraitStore + 程序性记忆集成 | 2天 | nt_memory_kb |
| 5 | PipelineAdapter + SEAL 集成 | 3天 | nt_mind_seal |
| 6 | E2E 测试 + 收敛调优 | 2天 | 所有阶段 |

**总计**: ~15 天

---

## 参考文献

- A-Evolve: Position: Agentic Evolution is the Path to Evolving LLMs (arXiv:2602.00359v2, ICML 2026)
- Reflexio: ReflexioAI/reflexio — Agent self-improvement harness (GitHub, 315★)
- BenchTrace: A Benchmark for Testing Reflection Ability in Controlled Evolution (arXiv:2605.29225)
- Voyager: An Open-Ended Embodied Agent with LLM (Wang et al., 2023)
