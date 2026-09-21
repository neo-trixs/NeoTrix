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

> 2026-09-21 确立：Agent 管理迁移到自动编排，禁止 CLI 命令架构设计

### 设计原则
1. **用户对话层零 CLI**：用户只说意图，系统自动路由到合适的 agent
2. **自动编排**：系统自动 spawn/kill/manage agent，零手动管理
3. **CLI 仅用于观测/调试**：status/logs/budget/kill 用于开发调试和应急干预

### 禁止的模式
```rust
// ❌ 禁止：用户手动管理 agent
/agent spawn <name> <mode>        // 系统应自动 spawn
/agent list                       // 系统应自动管理
/agent talk <id> <message>        // 系统应自动路由
/agent background <name> <mode>   // 系统应自动管理
```

### 允许的模式
```rust
// ✅ 允许：观测/调试/应急
/agent status                     // 查看编排器状态
/agent instances                  // 列出 agent 实例
/agent budget                     // 查看成本消耗
/agent kill <id>                  // 应急终止 agent
/mcp list                         // 查看 MCP 工具
/mcp search <query>               // 搜索 MCP 工具
```

### 架构要求
1. **AutoOrchestrator**：自动编排器，处理用户意图，自动选择/创建/管理 agent
2. **AgentLifecycleManager**：生命周期管理器，自动 spawn/kill/recycle agent
3. **IntentClassifier**：意图分类器，自动识别任务类型
4. **CLI 命令**：仅用于观测/调试/应急，不用于管理

### 迁移指南
1. **移除**：spawn/list/talk/background/tasks 等管理命令
2. **降级**：status/instances/budget/kill 保留为调试命令
3. **新增**：AutoOrchestrator + AgentLifecycleManager + IntentClassifier
4. **文档**：更新所有文档，说明自动编排架构

### 验证清单
- [ ] AutoOrchestrator 实现完成
- [ ] AgentLifecycleManager 实现完成
- [ ] IntentClassifier 实现完成
- [ ] CLI 命令降级为观测/调试
- [ ] 开发规则更新
- [ ] 文档更新
