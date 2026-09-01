# 能力网统一架构骨架 (Capability Network Architecture Skeleton)

> 本文档定义能力网的**统一架构模式**和**进化路径**。每个能力类别必须遵循同一套骨架设计。

---

## 设计原则

1. **统一模式**: 每个能力类别 = `Trait` + `Registry` + `Router` + `Bridge`
2. **零重复**: 共享类型放 `types.rs`，不跨模块复制
3. **进化可追踪**: C0→C6 constellation 级别，每个模块有明确的进化路径
4. **能力可组合**: L5 编排器通过 Trait 接口调用 L1 能力，不依赖具体实现
5. **能力可发现**: Registry 自动注册，Router 按需路由

---

## 一、统一架构模式 (Every Category Must Follow)

```
┌─────────────────────────────────────────────────────────┐
│                    能力类别 (Capability Category)          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐            │
│  │  Trait   │──▶│ Registry │──▶│  Router  │            │
│  │ (契约)   │   │ (发现)   │   │ (路由)   │            │
│  └──────────┘   └──────────┘   └──────────┘            │
│       │                             │                   │
│       ▼                             ▼                   │
│  ┌──────────┐                ┌──────────┐               │
│  │ Provider │                │  Bridge  │               │
│  │ (实现)   │                │ (L1↔L5) │               │
│  └──────────┘                └──────────┘               │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 必须包含的文件结构

```
nt_{category}/
├── mod.rs              # 模块入口 + re-exports
├── types.rs            # 共享类型 (不可放实现)
├── trait_def.rs        # 核心 Trait 定义
├── registry.rs         # 实现注册 + 发现
├── router.rs           # 智能路由 (选择最佳 Provider)
├── bridge.rs           # L1↔L5 桥接
└── providers/          # 具体实现 (可插拔)
    ├── mod.rs
    └── {provider}.rs
```

---

## 二、核心 Trait 契约

### L1ActionModule — 每个能力的基座 Trait

```rust
/// L1 能力模块基座 — 所有能力类别必须实现
pub trait L1Capability: Send + Sync {
    /// 能力唯一标识 (如 "messaging.whatsapp", "media.linkedin")
    fn capability_id(&self) -> &str;

    /// 能力类别
    fn category(&self) -> CapabilityCategory;

    /// 当前成熟度 (C0-C6)
    fn constellation(&self) -> ConstellationLevel;

    /// 健康检查
    fn health_check(&self) -> CapabilityHealth;

    /// 能力描述
    fn description(&self) -> &str;

    /// 依赖的其他能力
    fn dependencies(&self) -> Vec<&str> { Vec::new() }
}

/// 能力健康状态
pub struct CapabilityHealth {
    pub healthy: bool,
    pub latency_ms: Option<f64>,
    pub error_rate: f64,
    pub last_check: u64,
    pub message: Option<String>,
}
```

### 按类别的专用 Trait

```rust
// ── 通信能力 ──
pub trait MessagingProvider: L1Capability {
    fn send(&self, msg: &Message) -> Result<String, MessagingError>;
    fn receive(&self, since: Option<u64>) -> Result<Vec<Message>, MessagingError>;
    fn get_status(&self, id: &str) -> Result<MessageStatus, MessagingError>;
}

// ── 内容能力 ──
pub trait ContentProvider: L1Capability {
    fn publish(&self, post: &Post) -> Result<String, ContentError>;
    fn get_engagement(&self, post_id: &str) -> Result<EngagementMetrics, ContentError>;
    fn best_posting_times(&self) -> Vec<(u32, u32)>;
}

// ── 数据能力 ──
pub trait DataStore: L1Capability {
    fn store(&self, key: &str, value: &[u8]) -> Result<(), DataError>;
    fn load(&self, key: &str) -> Result<Option<Vec<u8>>, DataError>;
    fn query(&self, query: &str) -> Result<Vec<QueryResult>, DataError>;
}

// ── 搜索能力 ──
pub trait SearchEngine: L1Capability {
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError>;
    fn index(&self, doc: &Document) -> Result<(), SearchError>;
}

// ── 安全能力 ──
pub trait SecurityGuard: L1Capability {
    fn check(&self, action: &Action) -> SecurityVerdict;
    fn audit(&self, log: &AuditEntry) -> Result<(), SecurityError>;
}
```

---

## 三、能力分类 (8 Categories)

```
能力网 (L1 Capability Network)
│
├── CAT-1: 通信 (Communication)
│   ├── Messaging (WhatsApp/Email/SMS/Telegram/WeChat/Slack)
│   ├── Notification (桌面/移动推送)
│   └── Mention (@提及解析)
│
├── CAT-2: 内容 (Content)
│   ├── SocialMedia (LinkedIn/Instagram/Facebook/Twitter/TikTok)
│   ├── SEO (搜索引擎优化)
│   └── Media (图片/视频/文档生成)
│
├── CAT-3: 数据 (Data)
│   ├── KB (知识库 — SQLite + BM25 + Graph + Embedding)
│   ├── Historian (证据/时间线/假设网络)
│   ├── Spatial (地理空间)
│   └── Lead (询盘/CRM)
│
├── CAT-4: 搜索 (Search)
│   ├── Semantic (语义搜索)
│   ├── Graph (图搜索)
│   └── Web (网页搜索/爬取)
│
├── CAT-5: 执行 (Execution)
│   ├── Tool (Bash/Read/Write/Edit/Glob/Grep)
│   ├── Code (代码生成/重构)
│   ├── Sandbox (沙箱执行)
│   └── Voice (语音输入)
│
├── CAT-6: 安全 (Security)
│   ├── Shield (隐身网络/代理/防火墙)
│   ├── Sandbox (执行隔离)
│   └── Audit (审计/权限)
│
├── CAT-7: 协调 (Coordination)
│   ├── Orchestrator (DAG任务编排)
│   ├── Agent (多智能体协作)
│   ├── MCP (工具注册/发现/治理)
│   └── Plugin (插件生命周期)
│
└── CAT-8: 认知 (Cognition)
    ├── LLM (模型路由/池/故障转移)
    ├── Reasoning (推理引擎)
    ├── Memory (工作记忆/双脑)
    └── Self (自我进化/元认知)
```

---

## 四、进化路径 (Constellation C0-C6)

| 级别 | 名称 | 要求 | 典型产出 |
|------|------|------|----------|
| **C0** | 编译通过 | `cargo check` 通过，有 mod.rs + types.rs | 骨架文件 |
| **C1** | 单元测试 | `#[cfg(test)] mod tests` 通过 | 核心逻辑验证 |
| **C2** | 集成测试 | 与其他能力协作测试通过 | 端到端验证 |
| **C3** | 基准测试 | 有 benchmark，性能指标明确 | 性能基线 |
| **C4** | 主流管道 | 注册到 CapabilityRegistry，可被路由 | 生产可用 |
| **C5** | 自愈 | 有 SelfTest impl，自动检测退化 | 自动恢复 |
| **C6** | 自进化 | 有进化日志，可自主优化参数 | 持续改进 |

### 每个类别的进化路线图

```
CAT-1 通信
  C0 nt_io_messaging骨架 → C1 模板引擎测试 → C2 WhatsApp/Email联调
  → C3 吞吐基准 → C4 注册到能力网 → C5 消息投递自检 → C6 模板自动优化

CAT-2 内容
  C0 nt_act_media骨架 → C1 内容生成测试 → C2 多平台发布联调
  → C3 排期效率基准 → C4 注册到能力网 → C5 内容质量自检 → C6 策略自进化

CAT-3 数据
  C0 nt_memory_kb骨架 → C1 CRUD测试 → C2 搜索+图联调
  → C3 查询性能基准 → C4 注册到能力网 → C5 数据一致性自检 → C6 索引自优化

CAT-4 搜索
  C0 搜索骨架 → C1 语义搜索测试 → C2 混合搜索联调
  → C3 召回率基准 → C4 注册到能力网 → C5 搜索质量自检 → C6 索引自进化

CAT-5 执行
  C0 工具骨架 → C1 工具执行测试 → C2 沙箱联调
  → C3 执行速度基准 → C4 注册到能力网 → C5 执行安全自检 → C6 工具自组合

CAT-6 安全
  C0 安全骨架 → C1 规则引擎测试 → C2 防火墙联调
  → C3 检测速度基准 → C4 注册到能力网 → C5 威胁自检 → C6 规则自进化

CAT-7 协调
  C0 编排骨架 → C1 DAG执行测试 → C2 多Agent联调
  → C3 调度延迟基准 → C4 注册到能力网 → C5 死锁自检 → C6 策略自进化

CAT-8 认知
  C0 LLM骨架 → C1 路由测试 → C2 故障转移联调
  → C3 延迟基准 → C4 注册到能力网 → C5 路由质量自检 → C6 路由自进化
```

---

## 五、Registry + Router 模式

```rust
/// 能力注册中心 — 每个类别的 Provider 注册到这里
pub struct CapabilityRegistry<C: L1Capability> {
    providers: IndexMap<String, Box<C>>,
    health_cache: HashMap<String, CapabilityHealth>,
}

impl<C: L1Capability> CapabilityRegistry<C> {
    /// 注册 Provider
    pub fn register(&mut self, provider: Box<C>) { ... }

    /// 按 ID 获取
    pub fn get(&self, id: &str) -> Option<&C> { ... }

    /// 健康检查
    pub fn health_check_all(&self) -> HashMap<String, CapabilityHealth> { ... }

    /// 获取最佳 Provider (按健康度+延迟排序)
    pub fn optimal(&self) -> Option<&C> { ... }
}

/// 能力路由器 — 按上下文选择最佳 Provider
pub struct CapabilityRouter<C: L1Capability> {
    registry: CapabilityRegistry<C>,
    routing_rules: Vec<RoutingRule<C>>,
}

impl<C: L1Capability> CapabilityRouter<C> {
    /// 路由到最佳 Provider
    pub fn route(&self, context: &RouteContext) -> Result<&C, RoutingError> { ... }

    /// 带 fallback 的路由
    pub fn route_with_fallback(&self, ctx: &RouteContext) -> Result<&C, RoutingError> { ... }
}
```

---

## 六、Bridge 模式 (L1↔L5)

```rust
/// L5 编排器通过此桥接调用 L1 能力
/// 不直接依赖具体 Provider，只依赖 Trait
pub struct CapabilityBridge {
    messaging: Box<dyn MessagingProvider>,
    content: Box<dyn ContentProvider>,
    data: Box<dyn DataStore>,
    search: Box<dyn SearchEngine>,
}

impl CapabilityBridge {
    /// 从 Registry 构建 Bridge
    pub fn from_registries(
        msg_reg: &CapabilityRegistry<dyn MessagingProvider>,
        content_reg: &CapabilityRegistry<dyn ContentProvider>,
        data_reg: &CapabilityRegistry<dyn DataStore>,
        search_reg: &CapabilityRegistry<dyn SearchEngine>,
    ) -> Self { ... }

    /// 获取消息能力
    pub fn messaging(&self) -> &dyn MessagingProvider { self.messaging.as_ref() }

    /// 获取内容能力
    pub fn content(&self) -> &dyn ContentProvider { self.content.as_ref() }
}
```

---

## 七、重构优先级

| 优先级 | 类别 | 当前状态 | 重构动作 |
|--------|------|----------|----------|
| **P0** | types.rs | 各模块自定义类型 | 统一到 `l1_action/types.rs` |
| **P0** | traits.rs | 4个死 trait | 重写为 `L1Capability` + 8个类别 trait |
| **P1** | CAT-3 数据 | 77文件 mega-module | 拆分为 KB/Historian/Spatial/Lead 4个子类别 |
| **P1** | CAT-8 认知 | 27文件 provider | 统一 `LlmProvider` trait → `L1Capability` |
| **P2** | CAT-1 通信 | 新建 | 按统一模式完善 |
| **P2** | CAT-2 内容 | stub→完整 | 按统一模式完善 |
| **P3** | CAT-5 执行 | 已有 | 统一 trait |
| **P3** | CAT-6 安全 | 已有 | 统一 trait |
| **P4** | CAT-7 协调 | 已有 | 统一 trait |

---

## 八、文件模板 (New Category)

创建新能力类别时，复制此模板：

```rust
// nt_{category}/mod.rs
pub mod types;
pub mod trait_def;
pub mod registry;
pub mod router;
pub mod bridge;
pub mod providers;

// Re-exports
pub use types::*;
pub use trait_def::*;
pub use registry::CapabilityRegistry;
pub use router::CapabilityRouter;
pub use bridge::CapabilityBridge;

#[cfg(test)]
mod tests {
    // 每个类别必须有:
    // C0: cargo check 通过
    // C1: 至少3个单元测试
}
```

```rust
// nt_{category}/types.rs
// 只放类型定义，不放实现
use serde::{Deserialize, Serialize};

pub struct {Category}Config { ... }
pub enum {Category}Error { ... }
pub struct {Category}Result { ... }
```

```rust
// nt_{category}/trait_def.rs
use super::types::*;

pub trait {Category}Provider: Send + Sync + 'static {
    fn provider_id(&self) -> &str;
    fn is_available(&self) -> bool;
    fn health_check(&self) -> CapabilityHealth;
    // 类别专用方法...
}
```
