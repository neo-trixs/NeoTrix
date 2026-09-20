# NT-LANG 演进路线图

> 基于 Bend 吸收，规划 NeoTrix 自有语言的渐进式替换路径
> 日期: 2026-09-20

---

## 核心原则

**渐进替换，而非重写** — 每一步都必须可回退，可验证。

---

## Phase 0: 基础层 — 在 Rust 之上加 DSL (当前 → 1 个月)

### 目标
让 nt-lang 从"测试生成器"变成"声明式 DSL"，编译到 Rust 代码。

### 做法
```
.nt 文件 → nt-lang parser → .rs 文件 → rustc → 二进制
```

### 语法示例
```nt
# NT-LAWS.nt — 声明不变量（编译时检查）
law memory_never_leaks:
  for action: AgentAction
  ensures: !leaks_memory(action)

law cost_stays_below:
  for session: Session  
  ensures: total_cost(session) < 1.0

# NT-AGENT.nt — 声明 Agent
agent ResearchAgent:
  capabilities: [web_search, paper_read]
  max_cost_per_hour: 0.50
  
  def search(query: String) -> List<Paper>:
    # 业务逻辑...
```

### 编译目标
```rust
// 自动生成的 Rust 代码
pub struct ResearchAgent {
    capabilities: Vec<Capability>,
    max_cost_per_hour: f64,
}

impl ResearchAgent {
    pub fn search(&self, query: &str) -> Vec<Paper> {
        // 业务逻辑...
    }
}

// 编译时不变量检查
const _: () = assert!(
    !leaks_memory(action),
    "Law memory_never_leaks violated"
);
```

### 关键约束
1. **不引入新运行时** — 编译到 Rust，复用 tokio/rayon
2. **不破坏现有代码** — .nt 文件是可选的，Rust 代码照常工作
3. **每个 law 必须有对应 Rust 测试** — 证明机制是增强，不是替代

---

## Phase 1: 类型层 — 依赖类型模拟 (1-3 个月)

### 目标
用 Rust 的 trait + phantom type 模拟依赖类型，实现编译时能力检查。

### 做法
```nt
# 声明能力类型
capability WebSearch:
  max_results: Nat
  rate_limit: Float  # requests/second

capability PaperRead:
  formats: List<String>

# Agent 带能力参数
agent ResearchAgent<caps: [WebSearch, PaperRead]>:
  def search(query: String) -> List<Paper>:
    # 类型系统保证 this 有 WebSearch 能力
    require WebSearch in caps
    # ...
```

### 编译到 Rust
```rust
// Phantom type 表达能力
pub struct ResearchAgent<WebSearch, PaperRead> {
    _phantom: PhantomData<(WebSearch, PaperRead)>,
}

// Trait bound 保证能力
impl<WS: WebSearchCap, PR: PaperReadCap> ResearchAgent<WS, PR> {
    pub fn search(&self, query: &str) -> Vec<Paper>
    where
        WS: HasWebSearch,
        PR: HasPaperRead,
    {
        // ...
    }
}
```

### 关键约束
1. **Rust 编译器做检查** — 不自己实现类型检查器
2. **零运行时开销** — phantom type 在编译后消失
3. **错误信息友好** — 生成清晰的编译错误，不是类型推导垃圾

---

## Phase 2: 并行层 — 声明式并行 (3-6 个月)

### 目标
让 Agent 并行变成"声明"而非"编码"。

### 做法
```nt
# 声明哪些可以并行
parallel def batch_search(queries: List<String>) -> List<Paper>:
  match queries:
    case Nil: Nil
    case Cons(q, qs):
      # 这两个调用自动并行
      papers = batch_search(qs)
      result = search_one(q)
      Cons(result, papers)
```

### 编译到 Rust
```rust
pub fn batch_search(queries: Vec<String>) -> Vec<Paper> {
    match queries.as_slice() {
        [] => vec![],
        [q, qs @ ..] => {
            // rayon 自动并行
            let (papers, result) = rayon::join(
                || batch_search(qs.to_vec()),
                || search_one(q),
            );
            let mut results = vec![result];
            results.extend(papers);
            results
        }
    }
}
```

### 关键约束
1. **只用 rayon/ tokio** — 不引入新并行原语
2. **平衡分治** — 自动检测递归深度，避免线程爆炸
3. **保持 forbid(unsafe)** — 并行抽象必须是安全的

---

## Phase 3: 证明层 — 真正的证明检查 (6-12 个月)

### 目标
引入轻量级证明检查，让 Agent 的不变量有数学保证。

### 做法
```nt
# 证明文件
proof memory_never_leaks:
  for action in all_agent_actions:
    # 归纳法
    match action:
      case ReadMemory(_):
        # 读不泄漏
        trivial
      case WriteMemory(_, data):
        # 写需要权限检查
        require has_permission(action.agent, data.classification)
      case SendNetwork(_, payload):
        # 网络发送需要扫描
        require !contains_sensitive(payload, data.classification)
```

### 实现策略
1. **不从零造证明系统** — 集成 SMT solver (z3) 或轻量级 prover
2. **只检查关键路径** — 成本不变量、安全不变量、交接不变量
3. **证明失败 = 编译失败** — 不是运行时错误

### 关键约束
1. **证明必须快** — < 1 秒，否则失去实时性
2. **证明可以缓存** — 代码不变，证明不变
3. **可以 `@unsafe` 跳过** — 紧急情况允许绕过（但记录审计日志）

---

## Phase 4: 自举 — NT-LANG 编译 NT-LANG (12+ 个月)

### 目标
nt-lang 能编译自身，成为真正的语言。

### 做法
```
nt-lang.nt → nt-lang(旧) → nt-lang(新) → ...
```

### 前置条件
1. Phase 0-3 稳定运行 3 个月
2. 至少 10 个 .nt 文件在生产使用
3. 证明检查器通过形式化验证 (bend.lean 模式)

---

## 风险控制

| 风险 | 缓解措施 |
|------|----------|
| 语言设计错误 | Phase 0 只加语法，不改语义；可随时回退到纯 Rust |
| 编译时间爆炸 | 只对 .nt 文件做高级检查；Rust 文件走原生编译 |
| 类型系统复杂 | 只用 Rust 已有的 trait/phantom；不发明新类型理论 |
| 团队学习成本 | Phase 0 语法极简；文档 + 示例先行 |

---

## 成功指标

| 阶段 | 指标 | 目标 |
|------|------|------|
| Phase 0 | .nt 文件数 | ≥ 5 个核心模块用 .nt |
| Phase 0 | 编译通过率 | 100% |
| Phase 1 | 能力检查覆盖 | Agent 注册 100% 用 .nt 声明 |
| Phase 2 | 并行声明覆盖 | 多 Agent 协调 100% 用 parallel 关键字 |
| Phase 3 | 证明覆盖 | 关键不变量 100% 有 proof |
| Phase 4 | 自举 | nt-lang 能编译自身 |

---

## 第一步行动

立即可以开始的：
1. 在 nt-lang parser 中添加 `law` 关键字解析
2. 选择 `memory_never_leaks` 作为第一个不变量
3. 编译到 Rust 的 `const_assert!` 宏

这不需要任何新依赖，纯语法糖，今天就能做。
