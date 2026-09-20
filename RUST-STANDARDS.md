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
