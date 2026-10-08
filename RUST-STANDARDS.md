# NeoTrix Rust 编码标准

> 基于 alibaba/open-code-review 规则 + Rust 最佳实践 + Clippy pedantic

## 1. 错误处理（Error Handling）

### 规则
- **禁止** `unwrap()` / `expect()` / `panic!()` / `todo!()` / `unimplemented!()` 在生产代码中
- **禁止** `let _ =` 丢弃 Result 错误（必须 logging 或 propagation）
- **禁止** `Result<T, String>` 作为公开 API 返回类型
- **必须** 使用 `?` 操作符传播错误
- **必须** 错误包含上下文信息（what failed + why）

### 模式
```rust
// ❌ 禁止
let value = config.get("key").unwrap();
let _ = some_operation();
fn do_thing() -> Result<(), String> { ... }

// ✅ 正确
let value = config.get("key")
    .ok_or_else(|| DomainError::missing("key"))?;
if let Err(e) = some_operation() {
    tracing::warn!("operation failed: {e}");
}
fn do_thing() -> IpcResponse<()> { ... }
```

### Tauri 命令层 — 两种模式

**模式 A：IpcResponse<T>（NeoTrix 默认）**
```rust
#[tauri::command]
pub async fn my_command() -> IpcResponse<MyData> {
    match do_work().await {
        Ok(data) => ipc::ok(data),
        Err(e) => ipc::err("MY_ERROR", &e.to_string()),
    }
}
```

**模式 B：thiserror + serde tagged enum（Tauri 官方推荐）**
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

// 前端收到 { kind: "Io", message: "..." } 的 TypeScript discriminated union
#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum AppErrorKind<'a> {
    Io(&'a str),
    NotFound(&'a str),
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let kind = match self {
            Self::Io(msg) => AppErrorKind::Io(msg),
            Self::NotFound(msg) => AppErrorKind::NotFound(msg),
        };
        kind.serialize(serializer)
    }
}

#[tauri::command]
pub async fn my_command() -> Result<MyData, AppError> {
    Ok(do_work()?)
}
```

### Domain 层
- 使用 `DomainError` 作为内部错误类型
- 实现 `std::error::Error` trait

### Library vs Application Errors
- **Library errors**：用 `thiserror` 定义具体枚举（`#[derive(thiserror::Error)]`）
- **Application errors**：用 `anyhow` 快速传播（`anyhow::Result` + `.context("msg")`）
- **Tauri 命令边界**：将库错误转为 `IpcResponse<T>` 或 `Result<T, AppError>`

## 2. 所有权和借用（Ownership & Borrowing）

### 规则
- **禁止** 不必要的 `clone()` — 优先使用借用、迭代器、Cow
- **禁止** 在 async 函数中持有 `std::sync::Mutex` guard 跨 await
- **禁止** 使用 `Rc<RefCell<T>>` — 优先使用 `Arc<RwLock<T>>`
- **必须** 尽早释放锁（最小化 lock scope）

### 模式
```rust
// ❌ 禁止
let data = mutex.lock().unwrap().clone();
let guard = self.lock.lock().unwrap();
some_async_op().await;  // 跨 await 持有锁

// ✅ 正确
let data = { let g = self.lock.lock().unwrap(); g.clone() };
let result = { let g = self.lock.lock().unwrap(); g.compute() };
drop(guard);  // 显式释放
some_async_op().await;
```

## 3. 序列化（Serialization）

### 规则
- **必须** 所有 `#[derive(Serialize, Deserialize)]` 枚举添加 `#[serde(rename_all = "snake_case")]`
- **必须** 字段名使用 `#[serde(rename)]` 如果 JSON key 与 Rust 字段名不同
- **禁止** 依赖默认 PascalCase 序列化（除非有特殊需求）

### 模式
```rust
// ❌ 禁止
#[derive(Serialize, Deserialize)]
enum Status { Active, Inactive }  // 序列化为 "Active"

// ✅ 正确
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status { Active, Inactive }  // 序列化为 "active"
```

## 4. 并发和异步（Concurrency & Async）

### 规则
- **禁止** `std::sync::Mutex` 在 async 上下文中（使用 `tokio::sync::Mutex`）
- **禁止** 在 async 函数中调用阻塞 I/O（使用 `tokio::fs` 或 `spawn_blocking`）
- **禁止** `block_on()` 在 tokio 运行时内（会死锁）
- **必须** 使用 `Arc` 包装跨任务共享的状态
- **必须** 验证 `Send + Sync` 约束

### 模式
```rust
// ❌ 禁止
async fn process() {
    let file = std::fs::read("data.txt");  // 阻塞！
    tokio::runtime::Handle::current().block_on(other_future());  // 死锁！
}

// ✅ 正确
async fn process() {
    let file = tokio::fs::read("data.txt").await?;
    other_future().await;
}
```

## 5. 类型设计（Type Design）

### 规则
- **必须** 使用枚举而非布尔值/字符串表示状态
- **必须** 使用 Newtype 模式包装原始类型（如 `UserId(String)`）
- **必须** 实现 `Display` / `Debug` 用于所有公开类型
- **禁止** 在公开 API 中暴露具体容器类型（使用 `&[T]`、`impl Iterator`）

### 模式
```rust
// ❌ 禁止
fn process(enabled: bool, status: &str) { ... }

// ✅ 正确
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessMode { Enabled, Disabled }
fn process(mode: ProcessMode, status: &Status) { ... }
```

## 6. 性能（Performance）

### 规则
- **禁止** 在热路径中不必要分配（`format!`、`collect()`、`to_string()`）
- **必须** 已知大小时预分配集合（`Vec::with_capacity`、`HashMap::with_capacity`）
- **禁止** O(n²) 嵌套循环（使用 `HashMap`/`HashSet` 或排序）
- **必须** 使用 `iter()` 而非 `clone()` + `iter()`

### 模式
```rust
// ❌ 禁止
let mut items = Vec::new();
for i in 0..1000 { items.push(i); }
let names: Vec<String> = users.iter().map(|u| u.name.clone()).collect();

// ✅ 正确
let mut items = Vec::with_capacity(1000);
for i in 0..1000 { items.push(i); }
let names: Vec<&str> = users.iter().map(|u| u.name.as_str()).collect();
```

## 7. 安全（Security）

### 规则
- **禁止** 使用 `format!()` 构建 SQL 查询（使用参数化查询）
- **禁止** 日志中输出密钥、令牌、密码、PII
- **禁止** `static mut`（使用 `OnceLock` / `AtomicPtr`）
- **必须** 验证外部输入（路径、URL、序列化数据）
- **必须** 使用审查过的加密库（`sha2`、`ring`、`rustls`）

### 模式
```rust
// ❌ 禁止
conn.execute(&format!("SELECT * FROM users WHERE id = '{}'", id));
tracing::info!("API key: {}", api_key);

// ✅ 正确
conn.query_row("SELECT * FROM users WHERE id = ?1", [id], |r| ...);
tracing::info!("API key loaded, len={}", api_key.len());
```

## 8. 测试（Testing）

### 规则
- **必须** 公开 API 有单元测试
- **必须** 错误路径有测试（不只是 happy path）
- **必须** 边界条件有测试
- **禁止** 测试中使用 `unwrap()`（使用 `assert!` + 友好消息）

## 9. 文档（Documentation）

### 规则
- **必须** 所有 `pub` 函数/类型有 `///` 文档注释
- **必须** 文档注释使用完整句子
- **必须** 代码示例使用 `/// # Examples` 格式

## 10. 模块结构（Module Structure）

### 规则
- **必须** 每个模块有清晰的单一职责
- **必须** 类型定义在使用它的模块中（或共享类型模块）
- **禁止** 循环依赖（模块 A 依赖 B 同时 B 依赖 A）
- **必须** 内部实现使用 `pub(crate)` 而非 `pub`

## 11. 架构规则（Architecture Rules）

### 依赖方向
```
commands → domain → util
   ↓
  main.rs (注册 + 启动)
```
- **禁止** domain → commands（层级违规，破坏可测试性）
- **禁止** service → commands（同上）
- **必须** 共享逻辑提取到 util 层

### 错误传播链
```
DomainError → AppError → IpcError → IpcResponse<T> → Frontend
```
- 每个领域错误必须实现 `From<XxxError> for AppError`
- Tauri 命令边界统一转为 `IpcResponse<T>`
- **禁止** 命令层返回 `Result<T, String>`

### 类型单一事实源
- 相似类型必须在首次发现时统一
- **禁止** 两个模块定义结构相同但名称不同的类型
- 共享类型放在 domain 层或独立的 types 模块

### 状态生命周期
- 创建 Tauri managed state 时必须同时：1) 注册到 invoke_handler 2) 创建对应命令
- **禁止** 创建未使用的 state（死代码）
- 定期审计 managed states 的实际使用情况

### API 面管理
- 内部实现模块：`pub(crate) mod`
- 仅跨 crate 暴露的类型：`pub mod`
- **禁止** 所有模块都用 `pub mod`

## 12. 开发方法论（Development Methodology）

### 正确的开发顺序
1. **定义规则** → 编码标准 + 架构约束
2. **配置门禁** → clippy/fmt/pre-commit hooks
3. **统一架构** → 错误类型、模块层级、状态管理
4. **写业务代码** → 在已建立的框架内开发

### 审计方法论
1. **自动化优先** → clippy/fmt/cargo check 先行
2. **结构性审查** → 依赖图、层级违规、死代码
3. **人工审查** → 仅处理自动化发现不了的问题

### 代码质量防线
- 编译时：clippy pedantic + strict lints
- 运行时：tracing 日志 + 错误传播链
- 架构时：层级规则 + 类型单一事实源

## 13. 外部最佳实践（External Best Practices）

> 吸收自 Tauri 官方文档、Microsoft RustTraining、tauri2-template、生产级 Rust 项目

### thiserror + anyhow 组合
- Library errors：用 `thiserror` 定义具体枚举（`#[derive(thiserror::Error)]`）
- Application errors：用 `anyhow` 传播（`anyhow::Result` + `.context("msg")`）
- Tauri 命令边界：将库错误转为 `IpcResponse<T>` 或 `Result<T, AppError>`

### 模块布局：分层结构
```
src/
├── lib.rs          # Crate root - re-exports 公共 API
├── config.rs
├── domain/         # 业务逻辑层
├── infrastructure/ # 基础设施层
└── util.rs         # 内部工具 (pub(crate))
```

### API 设计：人体工程学参数
- `impl Into<String>` 降低调用方负担
- `impl AsRef<Path>` 降低借用负担
- `Cow<'_, str>` 避免不必要的分配

### 类型安全：Parse, don't validate
- 用 `TryFrom` 解析即验证，类型保证合法性
- 编译器阻止非法值传入

### 可见性控制：最严格优先
- `pub(crate)` 优于 `pub`
- `pub(super)` 用于模块内共享
- 无修饰符 = 私有实现

## 14. 跨域审计发现（Cross-Domain Audit Findings）

> 2026-08-28 审计结论，持续更新

### 事件命名
- **规则**：所有事件名必须 snake_case
- **禁止**：kebab-case (`neotrix-check-updates`)、colon (`neotrix:new-session`)、URI (`neotrix://file-drop`)
- **例外**：Tauri 内置插件事件保留原格式 (`neotrix_update_*`)

### 错误代码体系
- **现状**：3套并行系统 — `ErrorCode`枚举(System1) + `DomainError`裸字符串(System2) + `ipc::err()`硬编码(System3)
- **问题**：120个唯一代码，12个语义冲突（`NOT_FOUND`在System2有27处，System3有5个变体）
- **规则**：新错误代码必须添加到 `commands/error.rs` 的 `ErrorCode` 枚举
- **禁止**：在 `commands/*.rs` 中使用裸字符串作为错误代码
- **规则**：`From<DomainError> for AppError` 必须映射到正确变体，不能全部落入 `AppError::Other`

### 状态包装
- **规范模式**：`State<Arc<RwLock<T>>>` — 适用于需要可变状态的管理器
- **可接受模式**：`State<Arc<T>>` — 适用于内部自管理并发的类型（如 PtyManager）
- **可接受模式**：`State<T>` — 适用于不可变配置或简单管理器

### 路径管理
- **规则**：所有 `~/.neotrix` 路径必须通过 `config::AppConfig::base_dir()` 获取
- **规则**：所有项目级 `.neotrix` 路径必须通过 `config::AppConfig::project_dir()` 获取
- **禁止**：在业务代码中直接硬编码 `.neotrix` 路径字符串

## 15. 开发能力进化记录（Development Capability Evolution）

> 吸收自外部最佳实践 + 内部审计经验

### 防Bug编译时检查（Clippy Restriction Lints）
18个restriction lint覆盖6类bug：
- **Panic预防**: `panic`, `todo`, `unimplemented`, `unreachable`, `unwrap_used`, `expect_used`
- **索引安全**: `string_slice`, `indexing_slicing`, `get_unwrap`, `unwrap_in_result`
- **Async死锁**: `await_holding_lock`, `await_holding_refcell_ref`, `large_futures`
- **错误吞没**: `let_underscore_must_use`, `unused_result_ok`, `map_err_ignore`
- **Unsafe卫生**: `undocumented_unsafe_blocks`, `allow_attributes`, `allow_attributes_without_reason`

### 错误上下文链（Error Context Chains）
- 在每个 `?` 运算符处添加 `.context("doing X")`
- 生产环境错误链是最有价值的调试工具
- 规则：禁止裸 `?`，必须有上下文

### Tauri 2 安全模式
- **Capability最小化**: 只授予前端实际使用的权限
- **后端插件 ≠ 前端能力**: Rust后端直接使用插件API时，不需要前端JS能力
- **定期审计**: 每季度检查capability与实际使用的匹配度

### 架构公理（Architecture Axioms）
1. **单层信封**: API响应禁止嵌套 `{ok, data: {ok, data}}`
2. **事件命名统一**: 全部snake_case，禁止kebab/colon/URI格式
3. **路径集中管理**: 所有路径通过config模块获取，禁止硬编码
4. **命令瘦包装**: Tauri命令只做参数转换+调用domain服务
5. **错误代码集中**: 所有错误代码在error_codes.rs定义，禁止裸字符串
6. **DomainError智能路由**: From impl按code映射到正确AppError变体

### 跨域审计方法论
1. **三路并行审计**: 结构/API/事件同时审计
2. **先审计后修复**: 不审计就修是盲目行动
3. **记录经验**: 每次修复后记录pattern到经验库

## 16. CLI 命令架构禁令（CLI Architecture Ban）

> 2026-09-21 确立并落地：Agent 管理迁移到自动编排，`neotrix-core/src/cli/` 已完全删除

### 设计原则
1. **用户对话层零 CLI**：用户只说意图，系统自动路由到合适的 agent
2. **自动编排**：系统自动 spawn/kill/manage agent，零手动管理
3. **观测走 API，入口走意图**：状态观测用 `AutoOrchestrator::status()/instances()`；
   headless REPL 的自由文本经 `route_headless()` 自动分发，slash 仅保留观测/调试/应急

### 禁止的模式
```rust
// ❌ 禁止：任何用户手动管理 agent 的命令（已随 cli/ 删除）
/agent spawn <name> <mode>        // 系统应自动 spawn
/agent list                       // 系统应自动管理
/agent talk <id> <message>        // 系统应自动路由
/agent background <name> <mode>   // 系统应自动管理
/board create|move|assign         // 看板管理命令（已删除；状态观测走 API）
```

### 允许的模式
```rust
// ✅ 允许：API 观测 + headless 观测类 slash（显式调试/应急入口）
// API: AutoOrchestrator::status() / instances() / recycle()
// headless: /status /stats /think /evo /mem /cortex /mcp list|status|search
//           /goal status|history /proxy status /workflow list /exit
// 自由文本: route_headless() 高置信自动路由（research→/recall，absorption→/absorb，memory→/mem）
```

### 架构要求
1. **AutoOrchestrator**（`l6_meta::nt_auto_orchestrator`）：意图统一入口，`handle_intent/complete_task/recycle/status/instances/route_headless`
2. **AgentLifecycleManager**：生命周期，自动 spawn/recycle，成本预算熔断
3. **IntentClassifier**：关键词意图分类（15 任务类型），`route_headless` 阈值 0.8
4. **基础设施归位**：审批/成本/权限/规则→`l6_meta`；沙箱/安全执行→`l3_embodiment`；连接/路由→`l1_action`；JSONL→`l0_substrate`

### 迁移指南（已执行完毕，2026-09-21）
1. **移除**：`commands/`（52 文件）、`nt_subagent/`、`tui/`、整个 `cli/` 目录、`pub mod cli`
2. **迁移**：10 模块按上表归位，9 处外部引用改新路径（`seal_loop`/`factory`/`consciousness_core`/`main` 等）
3. **收敛**：headless 自由文本经 `route_headless` 自动分发；`handle_intent` 返回副本状态修正为 `Running`

### 验证清单
- [x] AutoOrchestrator 实现完成（含单测 13 项）
- [x] AgentLifecycleManager 实现完成
- [x] IntentClassifier 实现完成
- [x] `cli/` 完全删除，`cargo check --all-targets` 零 error 零 warning
- [x] 开发规则更新（本节）
- [ ] 文档更新

---

## 17. 卡死/内存事故沉淀的硬规则（2026-09-27，8 起生产事故换来的）

> 来源：一次会话内定位并根除 8 条卡死/内存爆炸根因（`chunk_planner` 无限循环、
> `DeferredLoader` 自死锁、KB 搜索二次加锁、RISE 守卫活到函数尾、架构守卫二次方空转、
> 3 处"测试/审计路径里再起 cargo"、sidecar 3G 常驻）。每条都有 file:line 实证。
> 完整分诊表见 `sessions/handoff-disease-list-20260927.md`。

### 17.1 锁与循环（最高频事故源）

- **R-LOCK-1 非重入 Mutex 不得同块二次获取**：`let g = x.lock()` 之后若在同一
  `{}` 作用域内再 `x.lock()`，同线程永久阻塞（`std::sync::Mutex` 不可重入）。
  典型形态是"为图省事在已持锁的函数里调另一个也加锁的方法"。
  守门：`python3 scripts/ops/nt_lock_audit.py neotrix-core/src`（块身份栈扫描，
  `--selftest` 自带正/负样本）。**2026-09-27 22:52 实测 3 条，非"0 命中"**；
  旧记录已证伪订正。**告警数会随代码变动漂移，跑一次记一次，别引用旧值。**
- **R-LOCK-2 锁守卫不得活到函数尾**：只在某个分支里 `drop(g)` 等于没 drop。
  读计数用独立作用域 `{}`，自增另开一次 `lock()`。
- **R-LOCK-4 早退路径必须显式释放守卫再委托**（2026-09-27 新增，1 真死锁）：
  `let conn = self.conn.lock()?;` 的守卫活到函数尾，若某条 `return
  self.other_method()` 分支上的 `other_method` 内部也 `self.conn.lock()`，
  就是**永久自死锁**。`kb_search.rs:545-549` `pq_search` 即此形态
  （`semantic_search` 首行就 `self.conn.lock()`），修法是 `return` 前 `drop(conn)`。
  判别：持锁函数里**任何**调用了同锁方法的分支，都要确认守卫已释放。
- **R-LOCK-5 `*self.x.lock() = v;` 是赋值型临时锁，`;` 处即释放**，
  不构成"持有守卫"。`audit_indirect` 尚未识别此形态（见 R-SCAN-1），
  读代码时别把它当持锁，也别因为工具报了就去"修"它。
- **R-LOOP-1 循环步长必须单调递增**：任何 `start = end - overlap` 形式，
  当 `end` 触顶后会**倒退或原地踏步** → 无限循环 + 每轮 clone → 内存爆炸。
  必须同时满足 `overlap < stride` 且"赋值后严格大于原值"（双重保险）。
- **R-LOCK-3 RwLock 的 layer/mask 不可混用**：`layers_compatible` 这类
  "a_layer & b_mask" 写反会让跨层判定全部漏判。

### 17.2 构建纪律（16G 机器，OOM 连坐）

- **R-BUILD-1 同一工作区只允许一个 cargo**（含子代理）。串行 `-j1` + 后台轮询。
  两个 `rustc` 峰值可达 4G+。
- **R-BUILD-2 内存门必须看退出码，不能只看输出行**：
  `sh scripts/ops/nt_mem_gate.sh; echo $?` → 非 0 即禁止起重型构建。
  （本规则因"只看输出行"被违反过一次，180MB 空闲时起了 rustc。）
- **R-BUILD-3 sidecar 按需**：`sh scripts/ops/nt_sidecar.sh {start|stop|status}`。
  权重常驻 3G，是 OOM 的第一嫌疑人；用完即 `stop`。
- **R-BUILD-4 测试/审计路径禁止再起 cargo**：内层 `cargo` 抢外层构建锁
  → 100% 死锁（`read_output→poll`），锁空闲时还会拉起整个编译器。
  需要真实构建的检测件：生产真跑，测试构建跳过（`cfg!(test)` 或
  `NT_SKIP_CARGO_CHECK=1`），或抽出可注入执行器（`BuildCheckOutcome` 先例）。
- **R-BUILD-5 禁 `cargo clean`**；`target/debug/incremental` 可删（纯缓存），
  `target/debug/deps` 保留（活指纹，删了全量重编数小时）。
- **R-BUILD-6 内存闸是全工作区共享的，别人的 cargo 会把你的闸拉黑**
  （2026-09-27 新增）：`nt_mem_gate.sh` 报 BLOCKED 时，先 `ps -Ao pid,rss,etime,command
  | grep -E 'rustc|cargo'` 确认**是不是自己**起的。若是他窗的 `cargo check --tests`
  （两个 rustc 实测 3.6G+4.2G），**不要 kill、也不要去 join** —— 先通报、等它
  跑完释放内存。`--tests` 是 AGENTS.md 明文禁止的重型档位。
  (4) **取闸口退出码要用变量，别用 `$?`**：`sh gate.sh; echo $?` 之后紧跟
  `if [ $? -eq 0 ]` 判断的是 **`echo` 的退出码（恒 0）**，不是闸口的。正确写法：
  `sh gate.sh; rc=$?; if [ $rc -eq 0 ]; then ...`。本轮即因此在闸口 BLOCKED 时
  误起了一次构建（幸好是 12 秒的叶子 bin，结果正确，但**规则被违反了**）。

- **R-BUILD-7 门必须验证"真的在跑"，不只是"脚本存在"**
  （2026-09-27 新增）：`.github/workflows/ci.yml` 长期是非法 YAML（某 `- name:` 值未加引号
  且含冒号），**主 workflow 从未被解析** ⇒ 其中的 `check-truth-surface.sh --strict` 门
  从未执行。而文档里「基线 0 条 ✅」被当作门有效的证据。**基线再准，门没跑等于没有门。**
  新增/改动任何门之后必须：(1) `python3 -c "import yaml;yaml.safe_load(open('<workflow>'))"`
  确认 workflow 可解析；(2) 确认 job 名出现在 `yaml.safe_load(...)['jobs']` 里；
  (3) 至少一次在 CI 上看到它跑（绿或红都算）。

### 17.3 卡死 vs 空转的判别（省下大量盲猜）

- `ps -o %cpu,rss,time` → CPU≈0 = **阻塞**；CPU 打满 = **空转/死循环**。
- `sample <pid> 1 -file /tmp/s.txt` 最快定性：
  - 栈底 `__psynch_mutexwait` / `mutex.rs:lock` → 锁问题，看 R-LOCK-1/2
  - 栈底 `read_output` → `poll` → 在等子进程（多半是 R-BUILD-4）
  - 某函数采样占比 300-800/N 帧 → 二次方/无界循环，看 R-LOOP-1
- 定位后**先 kill 测试进程**再改代码，否则两个重活叠加必 OOM。

### 17.4 Git 纪律（多人/多窗口共享工作区）

- **R-GIT-1 提交前必须 `git diff --cached --name-only`**：暂存区是共享的，
  别人的暂存会被你的 `git commit` 一并带走（本规则被违反过一次，卷进 9 个他人删除）。
- **R-GIT-2 一律 `git commit -- <paths>` 做 pathspec 限定提交**，不裸 `git commit`。
- **R-GIT-3 改文件前先 `git status --porcelain <file>`**：他人 `M` 的文件不要重写；
  若是纯 rustfmt 差异可叠加语义修改，但**必须在提交信息里写明含他人变更**。
- **R-GIT-4 禁止 `--no-verify`**：门禁拒绝时先查是不是别人的树坏了
  （本轮被拒 3 次全是他窗并发重构），用 `cargo check --lib`（排除 test cfg）
  验证自己的生产改动，等对方提交后再跑测试放行。
- **R-GIT-5 `git status` 只告诉你"脏没脏"，不告诉你"是否正在被写"**
  （2026-09-27 新增）：落盘前补一条 `stat -f "%Sm" <file>`，mtime 距今几秒内
  = 他人正在写。本轮 `nt_lock_audit.py` mtime 比检查早 12 秒、告警数在同一会话里
  从 12 变 3；`kb_search.rs` 我改完后另一窗口的 `assert s.count(o)==1`
  恰好失败，才没被重复打补丁 —— **反过来证明 `assert` 做幂等守卫是有效手法**，
  无人值守的批量改写脚本应当一律先 `assert` 再写。

### 17.5 修 bug 的判据（先分类再动手）

分诊四类，**每条失败先归类再改**：
- **P 生产 bug** → 修生产，测试自然绿
- **S 契约漂移** → 生产是有意改的，改**夹具**，并写明"旧预期为何过期"
- **U 未接线 stub** → 实现，或显式 `#[ignore]`；**禁止改松断言凑绿**
- **E 环境依赖** → 改确定性断言（相对时间、`:memory:`、命令断言）或 `#[ignore]`

- **R-FIX-1 修生产必然撞旧断言时，先判断谁对**。本轮 3 例是**测试钉死了错值**
  （键名 `"Tdd"` vs `name()` 的 `"TDD"`、分支数硬编码 11、`950_000` 恰撞 0.95 阈值）。
- **R-FIX-2 翻 `Ord`/派生顺序前必须审调用点**：本仓 `Severity` 判别序
  "越严重越小"看着是反的，但 `osint/sweep.rs:225` 显式依赖它做 `min_severity`
  过滤 —— 盲翻会**静默反转**过滤器。承重的"反直觉约定"要顺着它，不是纠正它。
- **R-FIX-3 返回值语义要看别的测试钉的是什么**。本轮 `ItemStack::add` 返回
  "剩余量"（被 `test_item_stack_add_capped` 钉住），而调用方 `add_item` 把它当
  "已加入量"从 remaining 里减 → 合并成功时继续开新栈（5+3 = 11）。
  这类 bug 的症状是"数值多了一点"，只有顺着契约链读才能发现。
- **R-FIX-4 修复合查询/解析器后，原先"靠 bug 才通过"的断言会现形**：
  本轮 `parse_range_query` 用 8080 断言 `< 1000`（恒假），此前因 `&&` 被空格
  截断成只剩左项才碰巧为真。**这属于修复暴露出的正确结果，不是回归。**

### 17.6 字节/字符安全（禁 panic 铁律的常见破口）

- **R-STR-1 禁止按字节切字符串**：`&text[..n]` 对中文会切在多字节字符中间
  → 直接 panic。必须按 `char_indices()` 边界回退（本轮 `truncate_preserving` 即此）。
- **R-STR-2 token/长度估算要分口径**：汉字 3 字节，纯 `len()/4` 会把中文高估 3 倍；
  同一口径只能有一处实现（`LlmNarrator::estimate_tokens` 委托共享实现即为此）。
- **R-STR-3 时间戳别用 `Instant::now()` 自减**：`duration_since(Instant::now())`
  恒为 ~0；`Instant::now().elapsed()` 同样恒为 ~0。要么存进程内单调基准
  （`OnceLock<Instant>` + `elapsed().as_millis()`），要么用 `SystemTime`。
  本轮 4 处，其中熔断器因此**永久锁死 Open**、审批 id 恒为 `conf_0` 撞号。

### 17.7 第三轮追加（代理并行 + 门禁阻塞 + 自检纪律暴露的新规则）

- **R-COMPILE-1 `deny(warnings)` 下类型延后推断会变硬错**：
  `let mut n = 0;` 后接 `n.saturating_add(1)`，若首次使用点在几行之后，rustc 报
  **E0689 ambiguous numeric type**。修法是在**声明处**标注（`let mut n: u32 = 0;`），
  不是在使用点 `as` 转换 —— 后者会把错误顺着字段类型扩散到别处（本轮
  `speedup_factor` 改成 f64 后又撞上 `total_target_verifications: u32`）。
- **R-COMPILE-2 改动公共度量结构体前先查 `Default`**：`PerformanceMetrics` 没有
  `Default`，补 derive 是最小改动；用 `or_insert_with(Default::default)` 前必须确认。
  注意同名不同型陷阱：`CapabilityMeta.metrics` 是 `CapabilityMetrics`，
  而报表的 `metrics` 是 `PerformanceMetrics`，两者不可互填。
- **R-TEST-1 修编译错误时不要改测试**：子代理改生产代码后编译失败，正确做法是回退
  自己的类型改动去迁就原有字段类型，而不是让测试适配。
- **R-BUG-1 "从未被调用过的模块"里可能有构造即 panic 的死代码**：
  `noise_handshake` 的 `hash[..27].copy_from_slice(<25 字节>)` 让
  `_initiator`/`_responder` **第一行就炸**，全模块从未成功构造过 —— 而它的测试
  "失败"看起来只是断言不符。**症状会伪装成下游问题**，要顺着栈顶往上读。
  修法：`hash[..name.len()].copy_from_slice(name)`，让长度自适应，杜绝再次漂移。
- **R-BUG-2 "写进去了却读不出来"是一类独立病**：`record_latency` 用
  `if let Some(metrics) = map.get_mut(..)` 静默丢弃未注册能力的数据，
  而 `get_status` 只遍历 registry → 数据写了读不到。两端都要修
  （写入端 `or_insert_with`，读取端为"有指标未注册"合成报告）。
- **R-GIT-6 门禁被内存阻塞时，修改会长期停留在"已改未验"**（2026-09-29 改号：
  本条此前误用 `R-GIT-5`，而 §17.4 已有一条同号规则（`git status` 不告诉你
  是否正在被写，被 6 处引用）—— **同号两义会让引用指向错的那条**，故升号）：
  此时必须在 handoff 里写清 **①未提交文件清单 ②验证命令 ③pathspec 提交清单**，
  否则下一对话无法安全接手，且共享暂存区可能被他人 `git add -A` 卷走。
- **R-VERIFY-1 授权不转移记账义务**：用户或上游说「不用编译/不用验证」时，**可以**
  照做（R-GIT-6 的三清单照旧适用），但**必须**在提交信息与 handoff 里写明 ①未验证的
  范围 ②接手命令 ③回退方式
  （`git revert <sha>`）。授权改变的是「做不做」，不是「记不记」。2026-09-29 实测：
  只照做不记账 ⇒ 下一个接手者把未验证提交当成已验证代码，而 `neotrix-core` 依赖
  `neotrix-neobot`，一处编译错就是主二进制 + 12170 单测全红。
- **R-VERIFY-2 事实变，文档必须跟着变**：任何让既有文档变假的操作（提交/回滚/删除/
  改名/状态翻转）**当场** grep 那句话。**R-P16 管「写没落盘」，本条管「还成不成立」**
  —— 落盘但已失效的句子比没写更危险，因为它让人以为状态已知。2026-09-29 实测：
  handoff 写「刻意未提交」，提交完即成假话。
- **R-VERIFY-3 机械化判据采纳前必须自测**：任何 `grep`/`jq`/脚本判据，先拿**一个已知
  合规的样本**跑一遍、确认它返回预期值；返回 0 时**先怀疑判据，再怀疑对象**。
  **固定词表判据会假阴性，而假阴性比没判据更坏**（让人以为「没记录」，实际记了）。
  2026-09-29 实测：`grep -c "未编译"` 对写「未经编译验证」的合规提交返回 0。
  判据要查**语义要点**（有没有提、是不是否定），不查**固定措辞**。
- **R-VERIFY-4 「字段/签名存在」不是能力**：判「X 有 Y 能力」必须跑真输入看输出，或 grep
  该路径的**占位特征**（`todo!()` / `unimplemented!()` / `启发式` / 占位返空 vec）。
  这是 R-SCAN-1 的自指版（那条管扫描器，这条管**自己的签名**）。2026-09-29 第二次同型
  翻车：`OcrResult` 有 `bounding_boxes` 字段，但 `PaddleOcrEngine::run_inference`
  返回空 vec（第一次是 `nt_jev` 导出 ≠ 调用）。
- **R-VERIFY-5 「测试全绿」≠「测试有效」**：有效测试的唯一证据是**能杀死变异**，
  且必须选**强**变异。2026-09-29 实测：清零一位可能恒等（`(0x00<<4)|0x54 == 0x54`
  ⇒ 对所有 `<0xXY>` 形式无效果）、返回 `None` 可能让调用方走 fallback 仍通过、
  **返回错常量**才是强变异（实测杀 7 个测试）。「**变异未落地**」与「测试无效」是两回事，
  必须区分（`nt_mutation_check.py` 用 exit 2 / 1 区分）。变异前 `assert count==1`、
  写后校验落地 —— 本轮我因缺这一步，把「变异没生效」误判成「测试无判别力」。
- **R-VERIFY-6 证据只能否证，不能自动推出「环境有问题」**：
  解码/编译失败时**先质疑自己的假设**，再考虑外归。2026-09-29 实测翻车：
  `ed25519-dalek 1.0.1` 报 `no SigningKey in the root`，我判「本地 registry 缓存损坏」，
  删目录 + `cargo fetch` 重解包 —— **结果仍缺 `signing.rs`**，判断被自己的实验证伪。
  查 docs.rs 才知：`SigningKey`/`VerifyingKey` 是 **2.0** 的名字，1.0.1 叫
  `SecretKey`/`PublicKey` ⇒ **是我们代码用 2.0 API 却声明 1.0 依赖**。
  我当时说「不该改代码去适配坏依赖」——方向完全反了。
  ⇒ 外归（环境坏/依赖崩/他窗的锅）必须**有独立证据**（如重下后仍复现），
  否则只是把「我不知道原因」包装成一个说得通的解释。
- **R-VERIFY-7 注释里的路径/符号/TODO 必须复核**：
  2026-09-29 实测 `self_model` feature 自落地起就编不过（E0599 ×2），
  而注释写着「真实实现在 `l0_substrate::nt_core_cross_layer::SelfModel`，TODO(T6)」——
  **该文件里根本没有 `SelfModel`**，注释写于功能落地前且从未复核。
  真实实现（facade 别名 `ValueSelfModel`，完整关键词投影打分，被 17 处引用）早就写好，
  只是调用方**用错了 facade 别名**（同名 ≠ 同一符号，三个 `SelfModel` 三个别名）。
  ⇒ 注释写「TODO」时必须确认它**至今仍未实现**；未复核的 TODO 会把「已完成的代码」
  误判成「未完成的功能」，进而做出错误的产品决策（本轮我确实误判过一次）。
- **R-ENV-1 三个并发 agent 窗口 ≈ 4.4G**，加上供 App 的 9B `llama-server` 670M，
  16G 机器上 `cargo` 会被挤到 OOM。**开重型构建前必须查其它窗口是否在跑**
  （`pgrep -c rustc` + 看 `ps -o %cpu,time` 判断是否真在干活，而非空挂）。
- **R-DISK-1 回收磁盘先分「工作区 / 生成物」两半，只删生成物**：
  `du -sh <wt>/target` vs `du -sh --exclude=target <wt>` 一比即知。多 worktree
  场景下 `.worktrees/*/target` 常占仓库体积的 **90%+**（本轮实测 17G 中 16.6G
  是 target）。`target` 已 gitignore ⇒ 是纯生成物，删零风险；**但 worktree 本体
  可能带未提交工作（本轮 3 个巨型 worktree 分别有 12/6/8 处脏文件）⇒ 绝不可
  `rm -rf` worktree 目录**。判据：`git check-ignore -q target` 确认生成物身份。
- **R-DISK-2 删 worktree 必须过两道闸**：`git worktree remove` 前要求
  ①`git status --porcelain` **为空**（有脏文件 = 未保存工作，`kb-flock-fix-2`
  曾有 758 处）；②`git branch -a --contains <HEAD>` **非空**（HEAD 已合入
  分支，删了不丢提交）。两闸全过才删，之后 `git worktree prune`。
  **"HEAD 含于某分支"不等于"可删"** —— 未提交改动不在任何提交里。
- **R-DISK-3 删 `target/` 后必须复验脏文件计数未变**：`git status --porcelain
  | wc -l` 清理前后逐一比对（本轮 758/20/16/9/4/4/2/2/1 全部一致才算过）。
  这是 R-P16 在文件系统操作上的等价物 —— 体积数字好看不等于没删错东西。
- **R-DISK-4 有 cargo 在跑时不碰主 `target/`**（含他窗构建）：
  `ps aux | grep -c '[c]argo'` 非 0 即让位。主 target 由 pre-commit build gate
  频繁重建，清了立刻又要 6.9G。
- **R-DISK-5 要删「带未提交改动的 worktree」时，先 patch 兜底再删**：
  850 处未提交改动（2026-09-28 实测 15 个 worktree，其中单个 758 处）**不在任何
  提交里**，`git branch -a --contains` 判不出来 ⇒ 删目录即永久丢失。
  兜底三步：①`git -C <wt> diff > <NN>-<name>.patch` ②
  `git -C <wt> status --porcelain | grep '^??' > <NN>-<name>.untracked.txt`
  （patch **不含**未跟踪内容，必须单独留清单）③
  **`git -C <wt> apply --check --reverse <patch>` 逐个校验可回放**，
  全过才执行 `git worktree remove --force`。删完写 README 说明回放方式。
- **R-DISK-6 `grep -q '^pattern'` 匹配不到已有配置 ≠ 配置缺失**：
  追加前先 `grep -n 'pattern'` 看全貌 —— 本轮误判 `.worktrees/` 未配置，
  实际它在 `.gitignore:70` 中部（我用了行首锚 `^` 才漏判），差点重复追加。
  追加前先 `grep -c` 计数，命中则跳过。
- **R-DISK-7 反面教材：判据自相矛盾时不得下结论**（2026-09-28 事故）。
  我移除 `ratchet` 前打印「无进程占用」但紧接着又打印出「脏文件: 4 处 /
  最后修改: 21:22」——**两个判据互相否定，我却仍按"空壳可删"执行了
  `remove --force`**，事后靠 R-DISK-5 的 patch 兜底才完整恢复。
  三个具体错法：
  ① **`lsof +D <大目录>` 常超时无输出 ≠ 无进程占用**（本轮该目录近 3k 文件）；
     判进程占用应看 `ps aux | grep <worktree名>` 或 `lsof | grep <path>`。
  ② **mtime 判据不得写死时间窗**（我写 `newermt '2026-09-28 19:00'`，
     而文件 21:22 才改过）；应比对该文件**自身生命周期内的最新 mtime**，
     或直接 `find -newermt '-1 hour'`。
  ③ **同一命令块内出现互相矛盾的判据时必须停下**，不能挑一个信的继续。
  通则：删除动作前，**所有**判据必须同向为"可删"，任一判据指向"有工作"即放弃。
  这与 AGENTS.md「mtime 近期变动 = 他方在写，换文件或先通报」是同一条纪律，
  此前只用于主工作树，本轮证明对 worktree 同样成立。
- **R-DISK-8 追加代码到脚本/hook 末尾前，先确认前面没有提前 `exit`**：
  本轮把 worktree 收工门加到 `.githooks/pre-push` 末尾，**它 3 天内从未执行过** ——
  该文件第 24 行原有 `exit 0`（Egress gate 之后），hook 到那里就终止了。
  而我当时**实测过**该段逻辑（mock 测 0/3/4 三档全部正确）却没发现它不可达
  ⇒ **测逻辑 ≠ 测可达性**。
  验证手法：改完 hook/script 后跑
  `bash -c 'set -euo pipefail; <前置门通过路径>; echo REACHED_TAIL'`
  或直接 `bash -x hook 2>&1 | grep REACHED`，确认新代码真的被执行。
  通用版：**任何"在文件末尾追加"的结构性改动，都要复验前序控制流**。
  这与 R-P16（重读验证落盘）互补：R-P16 验"写进去了"，本条验"跑得到"。
- **R-SCAN-4 示请示并得到决定后，动手前仍须复核一次现场**（2026-09-28 立）。
  本轮 pre-commit 门红在他窗未提交文件，我取证确认归属后向用户请示，
  用户选定「顺手删掉那个 unused import」；但动手前复核那一行时发现
  **内容已变、mtime 是 42 秒前** —— 那个窗口还活着且已自己修好。
  若照已做出的决策下手，就往一个正在被编辑的文件里写入。
  **请示与执行之间存在时间窗，那个窗口里世界可能已变。**
  与 AGENTS.md「mtime 近期变动 = 他方在写」互补：本条管的是
  「决定已做出、但尚未执行」这个阶段。

### 17.8 存在性断言的查证纪律（2026-09-29 立，两次踩中同一条）

**规则 R-EXIST-1（硬）**：断言某资产「不存在 / 丢失 / 从未入库」之前，
必须**同时**满足两个判据，缺一不可：

| 判据 | 命令 | 只查它会漏掉什么 |
|---|---|---|
| ① git 历史 | `git log --all --oneline -- <path>` | 磁盘上存在但未入库的**全部**资产 |
| ② 文件系统 | `find . -name '<glob>'` | 已入库但**已删除**的资产 |

⛔ **只查 git ⇒ 断言必错**。「git 里没有」≠「不存在」。

**我连踩两次的实录（同一会话内）**：

- **第一次（09-29 早）**：判定 `DESIGN-CHANNEL-DISPATCH.md`
  「从未入库、永久丢失，需重写设计」，并把恢复点写进 `TODO.md`。
  **真相**：文件一直在磁盘上、**1445 行、§11/§13 完好**。
  我只跑了 `git log --all`（零命中）就下结论，**没跑 `find`**。
  它是 `TODO.md:303` 那条 P0（IM `/stop` 兑现）的**设计依据** ——
  按我的结论，本可以开工的活被判成「设计缺失，无法开工」。
- **第二次（09-29 晚）**：全仓测绘发现 **78 份文档 + 82 份非文档资产**
  （含 `neotrix-core/tests/` **26 个集成测试**）躺在磁盘上但从未入库。
  干净检出上它们**全部消失** ⇒ 门跑的是「测试从未存在」的空集。
  **其中 `architecture_constraints.rs`（分层约束测试）形同虚设。**

**根因与 R-P41（门记录声称已做而实现从未入库）同源**：
都是把「git 视角的事实」误当成「世界的事实」。R-P41 管的是
「声称已入库其实没有」，本条管的是「声称没有其实有」——**同一个洞的两面**。

**规则 R-EXIST-2（硬）**：新写的门脚本在**干跑之前**必须先审有无写操作。
判别标准：除 `git status` / `git ls-files` / `find` / `grep` / `du` / `df` / `ps`
等只读命令外，**不得有任何写操作**。这是 R-SCAN-4 的前置要求 ——
本轮 `check-disk.sh` 因在「建议」文案里写反引号示例命令，
被 bash 当命令替换**真的执行了 `cargo clean`，删掉 115.2 GiB**。

**规则 R-EXIST-3**：共享工作树里发现「未入库资产」时，**先定性再入库**。
不得把编译产物（`*.rlib`）、运行时 DB（`*.db`）、`.bak-*` 备份、
`.disabled` 死测试、含密钥的配置（`config/*.toml`）一并 `git add`。
入库前逐条 `git check-ignore -v` + 内容抽检，**含密钥的绝不 add**。

---

### 17.9 显示/解析配对纪律（2026-10-08 立，一个 bug 换来的）

**规则 R-DISP-1（硬）**：渲染层若给某个**标识符**加了装饰
（方括号 / 引号 / `#` 前缀 / `>` 引用号 / 颜色转义 / 项目符号），
⛔ 解析层**必须容忍**该装饰 —— 因为「用户照抄屏幕」是最自然的行为，
而照抄失败**没有任何提示**，只会表现为「我明明按它说的做了」。

⛔ **报错拒绝不是兼容性，是把 bug 转移给用户**：加装饰的是显示层，
真实标识符永不含装饰 ⇒ 把屏幕上那个**合法字符串**判成非法输入，
是在要求用户去猜一个他无从得知的内部约定。

**2026-10-08 实测事故（`ntcode --line` 永远收敛不了）**：

| 输入 | 结果 |
|---|---|
| `ok [review-fc47da46]`（**照抄屏幕**） | `[round2] 内需1项已上窗` ⇒ `终态：Stalled`、exit **2** |
| `[review-fc47da46]: 收到` | 同上，同样卡死 |
| `ok review-fc47da46`（裸 id） | `内需清空，收敛` ⇒ `终态：Converged`、exit **0** |

根因是两侧不对齐：`nt_crystal_dialogue.rs` 的 `format!("  [{}] …", d.id)`
**加**方括号，而 `nt_stdin_human.rs` 的四条 arm（`ok ` / `no ` / `<id>!` / `<id>:`）
**原样存**不剥 ⇒ 与真实裸 id 永不相等 ⇒ 批准静默失效 ⇒ 同一道复核门原样重上。

**修法**：解析层剥掉**成对**外层装饰（`normalize_demand_id`），
裸 id 走原路径 ⇒ 两种输入都成立。⛔ 不做「不成对就报错」——
宁可原样保留也不要猜用户意图（有测试钉住这条）。

**规则 R-DISP-2（指针守恒的配对版）**：这类知识**必须同时落在「因」与「治」两侧** ——
只写在修复处是**半更新**：下一个人改**显示侧**时看不到任何提示，
就会把同一个 bug 以另一种样子请回来。
⇒ 显示侧那行必须有反向交叉引用（本轮已补：
`nt_crystal_dialogue.rs` 的 `[{}]` 旁注明「解析层会剥」+ 新增装饰时必须同步告知解析层）。

**规则 R-DISP-3**：断言 / 门 / 门记录若声称「某开关已生效」，
判据必须是**唯一真源**（真源文件 / 二进制本体），
⛔ 不是「配置文件里那个值」也不是「设置界面上的开关」。
本轮同族事故：某 CLI 的 `adsEnabled:false` 被**构建期常量短路**成死开关
（字节偏移实证：`LR()` 的对象字面量里 `FREEBUFF_MODE:"true"` 是硬编码字面量，
而邻居全是 `process.env.X`）⇒ 回执文案「Ads disabled.」证明的是**写进去了**，不是**生效了**。

完整取证与 9 条同族教训见
`docs/architecture/LESSONS-2026-10-08-cli-plugin-and-shared-index.md` L1 / L7。
