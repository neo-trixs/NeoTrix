# Bend 项目吸收分析

> 吸收自: https://github.com/bend-lang/bend (22K★)
> 日期: 2026-09-20
> 状态: 待评审

---

## 1. 项目概览

Bend 是一个面向后 AGI 时代的编程语言，核心定位：

| 维度 | Bend | NeoTrix 启发 |
|------|------|-------------|
| **速度** | C 级单核，CUDA 级多核 | Agent 执行引擎可借鉴 |
| **验证** | LAWS.bend + 数学证明 | NT-LANG 可扩展为 Law-based 验证 |
| **并行** | 自动分治，无需线程/锁 | 多 Agent 协调可借鉴 |
| **语法** | Python 风格 + 依赖类型 | nt-lang 语法设计参考 |

---

## 2. 核心概念吸收

### 2.1 LAWS.bend → NT-LAWS.nt (最关键启发)

**Bend 的做法**：
```bend
# LAWS.bend - 声明不可违反的规则
law you_cant_win:
  for moves: List<Move>
  board = replay(start(), moves)
  is_won(board) == False{}

# PROOF.bend - 提供数学证明
def Laws.you_cant_win(moves):
  # ... AI 必须写出证明
```

**对 NeoTrix 的启发**：

NeoTrix 的 `AGENTS.md` 规则目前是**文本约束**，AI 可以"忽略"或"误读"。Bend 的 LAWS 机制将规则提升为**类型系统强制**——编译器保证规则不被违反。

**提议：NT-LAWS.nt**
```nt
# NT-LAWS.nt - Agent 行为不变量

law memory_never_leaks:
  for action: AgentAction
  for data: SensitiveData
  {ActionResult.leaks(action, data) == False{} : Bool}

law cost_stays_below:
  for session: Session
  {Session.total_cost(session) < 1.00 : Bool}  # $1 上限

law handoff_includes_context:
  for h: Handoff
  {Handoff.has_context(h) == True{} : Bool}
```

**实现路径**：
1. nt-lang parser 增加 `law` / `proof` 语法
2. 编译到 Rust 时生成 `const_assert!` 或 trait bound
3. Agent 执行前验证当前操作不违反任何 law

### 2.2 自动并行 → Agent 并行调度

**Bend 的做法**：
```bend
def pow2(+d: Nat) -> U32:
  match d:
    case 0n: 1
    case 1n+p:
      a b = pow2(p) pow2(p)  # 自动并行
      (a + b : U32)
```

**对 NeoTrix 的启发**：

当前 `neotrix-multi-agent/src/parallel.rs` 需要显式管理线程。Bend 的模型启示：

```nt
# 声明式并行，而非命令式
def coordinate_agents(tasks: List<Task>) -> List<Result>:
  match tasks:
    case Nil: Nil
    case Cons(t, ts):
      results = coordinate_agents(ts)  # 递归自动并行
      r = execute(t)
      Cons(r, results)
```

**实现路径**：
1. 在 `nt_core::task_decomposer` 中增加自动并行标记
2. 分治模式自动映射到 `rayon::par_iter` 或 Actor 并发
3. 保持 `#![forbid(unsafe_code)]`，不引入原始线程操作

### 2.3 依赖类型 → Agent 能力类型系统

**Bend 的做法**：
```bend
# 类型依赖于值
def head(+xs: Array<N, T>) -> T:
  match xs:
    case (Cons(x, _)): x
    # 编译器知道数组非空，无需运行时检查
```

**对 NeoTrix 的启发**：

Agent 能力可以用依赖类型精确表达：

```nt
# 类型级别表达能力约束
type Agent<caps: CapabilitySet> = {
  id: UUID,
  capabilities: caps
}

# 只有具备特定能力的 Agent 才能执行任务
def dispatch<N: Nat, caps: CapabilitySet>(
  agent: Agent<caps>,
  task: Task<caps>
) -> Result:
  # 类型系统保证 agent 有能力执行 task
  execute(agent, task)
```

**实现路径**：
1. Rust 层面用 trait + phantom type 表达能力约束
2. nt-lang 编译时生成正确的 trait bound
3. 运行时零开销，编译时捕获能力不匹配

### 2.4 快速证明检查 → 实时 Agent 验证

**Bend 的做法**：
- 证明检查 < 1 秒（其他系统需要分钟）
- AI 每次修改代码后立即验证

**对 NeoTrix 的启发**：

Agent 每次行动后可以**自验证**：
1. 执行前：检查是否违反任何 NT-LAWS
2. 执行中：类型系统保证能力匹配
3. 执行后：快速验证结果不变量

---

## 3. 架构映射

```
Bend 概念              →  NeoTrix 对应
─────────────────────────────────────────
LAWS.bend              →  NT-LAWS.nt (待实现)
PROOF.bend             →  nt_proof 模块 (待实现)
并行运行时              →  neotrix-multi-agent/parallel.rs
依赖类型系统            →  neotrix-types + trait bounds
快速证明检查            →  nt_governance::policy_engine
语法 (Python+DT)       →  nt-lang parser
编译目标 (C/CUDA/Metal) →  codegen/rust.rs (可扩展)
```

---

## 4. 待实现特性优先级

| 优先级 | 特性 | 工作量 | 价值 |
|--------|------|--------|------|
| **P0** | NT-LAWS.nt 语法 + parser | 2-3 天 | 极高 - 规则强制执行 |
| **P1** | Agent 并行声明式标记 | 1-2 天 | 高 - 简化多 Agent 协调 |
| **P2** | 依赖类型 trait bound 生成 | 3-5 天 | 中 - 编译时能力检查 |
| **P3** | 实时证明检查集成 | 5-7 天 | 中 - Agent 自验证 |

---

## 5. 与现有规则的对齐

Bend 吸收与现有规则的关联：

- **R-P111 架构治理**: NT-LAWS 应纳入治理框架
- **R-P161 对抗输入管线**: LAWS 可作为管线的类型级验证
- **R-P186 Agent 交接**: Handoff 的 context 包含可证明的不变量
- **R-P191 内联测试**: LAWS 是测试的超集——证明 > 测试

---

## 6. 风险与限制

1. **Rust 类型系统限制**: Rust 不原生支持依赖类型，需要 phantom type + trait 模拟
2. **学习曲线**: 依赖类型对 Agent 开发者有门槛
3. **编译时间**: 类型级计算可能增加编译时间（需监控）
4. **Bend 自身限制**: Bend 无 type classes、无宏，设计选择需取舍

---

## 7. 下一步行动

1. [ ] 在 nt-lang 中实现 `law` / `proof` 语法原型
2. [ ] 选择 1-2 个关键不变量用 NT-LAWS 表达
3. [ ] 测试 Agent 是否能生成满足 LAWS 的代码
4. [ ] 评估并行声明式标记对 multi-agent 的影响
