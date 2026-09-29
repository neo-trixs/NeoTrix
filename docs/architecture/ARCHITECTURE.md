# NeoTrix 架构技术 Map
#
# 标准化技术架构文档 — 指导后续迭代更新
# 版本: v1.0.0
# 更新: 2026-09-20

## 1. 整体架构拓扑 (C4 Model)

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           NeoTrix System Context (C4 L1)                        │
│                                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐   │
│  │   User       │    │  External    │    │   LLM        │    │   Data       │   │
│  │   Interface  │◄──►│  Services    │◄──►│   Providers  │    │   Sources    │   │
│  └──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│                          NeoTrix Container (C4 L2)                              │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                          Presentation Layer                            │   │
│  │  Tauri Desktop / CLI / API Server / Mobile Bridge                       │   │
│  └───────────────────────────────────┬─────────────────────────────────────┘   │
│                                      │                                         │
│  ┌───────────────────────────────────▼─────────────────────────────────────┐   │
│  │                          Cognition Layer (L6)                           │   │
│  │  Meta-Cognition / Self-Model / Evolution / Governance                   │   │
│  └───────────────────────────────────┬─────────────────────────────────────┘   │
│                                      │                                         │
│  ┌───────────────────────────────────▼─────────────────────────────────────┐   │
│  │                          Emotion Layer (L4-L5)                          │   │
│  │  Emotion Engine / Consciousness Core / Decision Engine                  │   │
│  └───────────────────────────────────┬─────────────────────────────────────┘   │
│                                      │                                         │
│  ┌───────────────────────────────────▼─────────────────────────────────────┐   │
│  │                          Perception Layer (L2)                          │   │
│  │  NT-WORLD / NT-NEXUS / NT-SENSE / Media Engine / Data Sources          │   │
│  └───────────────────────────────────┬─────────────────────────────────────┘   │
│                                      │                                         │
│  ┌───────────────────────────────────▼─────────────────────────────────────┐   │
│  │                          Action Layer (L1)                              │   │
│  │  LLM Providers / IO / Memory / Media / Task Dispatch                   │   │
│  └───────────────────────────────────┬─────────────────────────────────────┘   │
│                                      │                                         │
│  ┌───────────────────────────────────▼─────────────────────────────────────┐   │
│  │                          Substrate Layer (L0)                           │   │
│  │  Core Types / Events / ECS / Telemetry / Error Handling                │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 2. 分层架构设计

### 2.1 Layer 0: Substrate (基础层)
- **职责**: 核心类型、事件系统、ECS、遥测、错误处理
- **模块**: `l0_substrate/`
- **依赖**: 无外部依赖 (纯 Rust)
- **关键类型**:
  - `SharedTypes` - 共享类型定义
  - `EventBus` - 事件总线
  - `EcsWorld` - ECS 世界
  - `Telemetry` - 遥测系统
  - `Error` - 统一错误类型

### 2.2 Layer 1: Action (执行层)
- **职责**: LLM 调用、IO 操作、内存管理、媒体处理、任务分发
- **模块**: `l1_action/`
- **依赖**: L0
- **关键模块**:
  - `nt_io/` - IO 操作 (LLM/Web/Browser)
  - `nt_core_llm/` - LLM 提供者统一接口
  - `nt_memory/` - 内存管理
  - `nt_media/` - 媒体处理
  - `nt_task_decomposition.rs` - 任务分解

### 2.3 Layer 2: Perception (感知层)
- **职责**: 世界感知、数据采集、内容理解
- **模块**: `l2_perception/`
- **依赖**: L1
- **关键模块**:
  - `nt_world/` - 世界模型
    - `source/` - 统一数据源架构
    - `data_source/` - 情报类数据源
    - `osint/` - OSINT 数据源
    - `crawl/` - 爬虫能力
  - `nt_nexus/` - 跨会话记忆
  - `nt_sense/` - 感知处理

### 2.4 Layer 3: Embodiment (具身层)
- **职责**: 安全防护、计算集群、守卫链
- **模块**: `l3_embodiment/`
- **依赖**: L2
- **关键模块**:
  - `nt_shield/` - 安全防护系统
  - `nt_computer/` - 计算集群
  - `nt_computer_fleet.rs` - 计算舰队

### 2.5 Layer 4-5: Emotion & Cognition (情感与认知层)
- **职责**: 情感引擎、意识核心、决策引擎
- **模块**: `l4_emotion/`, `l5_cognition/`
- **依赖**: L3
- **关键模块**:
  - `consciousness_core/` - 意识核心
  - `nt_decision_engine.rs` - 决策引擎
  - `nt_consciousness_tree.rs` - 意识树

### 2.6 Layer 6: Meta (元认知层)
- **职责**: 元认知、自我模型、进化、治理
- **模块**: `l6_meta/`
- **依赖**: L5
- **关键模块**:
  - `nt_governance/` - 治理系统
  - `nt_nexus/` - 跨会话记忆
  - `evolution/` - 进化系统

## X. 五实体投影（FIVE-ENTITY-BLUEPRINT-V3 正典，2026-09-22 接入）

> CrystalState 为单一事实源；五实体是它的五种投影，外围注册表为投影的序列化层。
> 实施编号 E0–E4（旧 D/I/W/P 冻结）；任务清单见 `FIVE-ENTITY-TASK-CHECKLIST.md`（T01–T37）。

| 实体 | 正典类型 | 投影字段 | KB 键 | 状态 |
|---|---|---|---|---|
| Workspace | `l0 nt_core_ws::WorkSpace` | `workspace: WorkspaceProjection` | `workspaces` | ⬜ E1 |
| Agent | `l1 nt_infra_agent_card::AgentCard` | `agents: Vec<AgentProjection>` | `agent_cards` | ⬜ E1 |
| Skill | `skill_loader::ResolvedSkill`＋`skill_evolution::SkillCandidate` | `skills: Vec<SkillProjection>` | `skill_candidates` | ✅ 门禁已落地（P0-2） |
| Task | `l1 nt_act_scheduler::ScheduledTask` | `tasks: Vec<TaskProjection>` | `scheduled_tasks` | ⬜ E1 |
| MCP | `agent::tool::mcp::McpRegistry` | `tools: Vec<ToolProjection>` | `mcp_servers` | ⬜ E1（注册表已融合 cli 版，死链待修） |

路由三层（Skill 触发＞Agent 能力＞CAPABILITY_ROUTES 兜底；召回与裁决分离）；
事件 12 变体见 CoreEvent；tick 同步读镜像（Scheduler 异步墙已立方案）。
E0 顺序：E0.0✅ → E0.2/E0.5（路径改写）→ E0.3/E0.1 → E0.4（最后）。
详见 `FIVE-ENTITY-BLUEPRINT-V3.md`。

## 3. 统一数据源架构 (数据流程节点 Map)

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         UnifiedEngine (统一查询入口)                            │
│                                                                                 │
│  search_media() ──────────┐                                                    │
│  search_intel() ──────────┤                                                    │
│  investigate_osint() ─────┼──► DataSourceRegistry ──► AnyDataSource            │
│  llm_complete() ──────────┤       │                                            │
│  crawl_url() ─────────────┘       │                                            │
│                                   ▼                                            │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                        DataSourceRegistry                               │   │
│  │                                                                         │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │   │
│  │  │ MediaSource │ │ IntelSource │ │ OsintSource │ │ LlmProvider │       │   │
│  │  │ (29 个)     │ │ (3+ 个)     │ │ (20+ 个)    │ │ (30+ 个)    │       │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘       │   │
│  │                                                                         │   │
│  │  ┌─────────────┐ ┌─────────────┐                                       │   │
│  │  │CrawlSource  │ │ LocalSource │                                       │   │
│  │  │ (爬虫能力)  │ │ (本地文件)  │                                       │   │
│  │  └─────────────┘ └─────────────┘                                       │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘

## 4. 数据流程图

### 4.1 媒体搜索流程
```
User Query ──► UnifiedEngine::search_media()
              │
              ▼
              DataSourceRegistry::media_sources(&SourceDomain::Media)
              │
              ├──► NeteaseSource::search()
              ├──► KuwoSource::search()
              ├──► YouTubeSource::search()
              └──► ... (29 个源)
              │
              ▼
              Merge Results ──► MediaSearchResult
```

### 4.2 情报采集流程
```
Query ──► UnifiedEngine::search_intel()
          │
          ▼
          DataSourceRegistry::intel_sources(&SourceDomain::Intel)
          │
          ├──► GdeltBridge::fetch()
          ├──► EdgarBridge::fetch()
          └──► UsgsBridge::fetch()
          │
          ▼
          Merge Results ──► IntelResult
```

### 4.3 OSINT 调查流程
```
Target ──► UnifiedEngine::investigate_osint()
           │
           ▼
           DataSourceRegistry::osint_sources(&SourceDomain::Osint)
           │
           ├──► DnsBridge::investigate()
           ├──► ShodanBridge::investigate()
           └──► CensysBridge::investigate()
           │
           ▼
           Merge Results ──► OsintResult
```

### 4.4 LLM 推理流程
```
Request ──► UnifiedEngine::llm_complete(provider_id)
            │
            ▼
            DataSourceRegistry::find(provider_id)
            │
            ├──► OpenAiProvider::complete()
            ├──► AnthropicProvider::complete()
            ├──► OllamaProvider::complete()
            └──► ... (30+ 个提供者)
            │
            ▼
            LlmResponse
```

## 5. 接口设计规范

### 5.1 DataSource trait (基础 trait)
```rust
pub trait DataSource: Send + Sync + 'static {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn domains(&self) -> Vec<SourceDomain>;
    fn health_check(&self) -> Pin<Box<dyn Future<Output = bool> + Send>>;
    fn requires_key(&self) -> bool;
}
```

### 5.2 MediaSource trait
```rust
pub trait MediaSource: DataSource {
    fn search(&self, query: &str, page: u32) -> Pin<Box<dyn Future<Output = Result<MediaSearchResult, String>> + Send>>;
    fn play_url(&self, item_id: &str, quality: &str) -> Pin<Box<dyn Future<Output = Result<PlaySource, String>> + Send>>;
    fn lyric(&self, item_id: &str) -> Pin<Box<dyn Future<Output = Result<Lyric, String>> + Send>>;
}
```

### 5.3 IntelSource trait
```rust
pub trait IntelSource: DataSource {
    fn fetch(&self, query: &str) -> Pin<Box<dyn Future<Output = Result<IntelResult, String>> + Send>>;
}
```

### 5.4 OsintSource trait
```rust
pub trait OsintSource: DataSource {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>>;
    fn supported_targets(&self) -> Vec<&str>;
}
```

### 5.5 LlmProvider trait
```rust
pub trait LlmProvider: DataSource {
    fn complete(&self, request: &LlmRequest) -> Pin<Box<dyn Future<Output = Result<LlmResponse, String>> + Send>>;
    fn is_free(&self) -> bool;
    fn requires_api_key(&self) -> bool;
}
```

## 6. 模块依赖关系图

```
L6 Meta ────────────────────────────────────────────────────────────────────────►│
  │                                                                             │
  ▼                                                                             │
L5 Cognition ──────────────────────────────────────────────────────────────────►│
  │                                                                             │
  ▼                                                                             │
L4 Emotion ────────────────────────────────────────────────────────────────────►│
  │                                                                             │
  ▼                                                                             │
L3 Embodiment ─────────────────────────────────────────────────────────────────►│
  │                                                                             │
  ▼                                                                             │
L2 Perception ────────────────────────────────────────────────────────────────►│
  │  ├── nt_world/source/ ────────────────────────────────────────────────────►│
  │  │     ├── unified.rs (DataSource/MediaSource/IntelSource traits)          │
  │  │     ├── media_bridge.rs (29 MediaSource adapters)                       │
  │  │     ├── intel_bridge.rs (3+ IntelSource adapters)                       │
  │  │     ├── osint_bridge.rs (20+ OsintSource adapters)                     │
  │  │     ├── llm_bridge.rs (30+ LlmProvider adapters)                       │
  │  │     └── unified_engine.rs (UnifiedEngine entry point)                  │
  │  ├── nt_world/data_source/ ──────────────────────────────────────────────►│
  │  │     ├── nt_world_edgar.rs                                               │
  │  │     ├── nt_world_gdelt.rs                                               │
  │  │     └── ... (12 IntelSource implementations)                           │
  │  └── nt_world/osint/ ────────────────────────────────────────────────────►│
  │        ├── dns.rs, shodan.rs, censys.rs, ...                              │
  │        └── ... (30+ OsintSource implementations)                          │
  │                                                                             │
  ▼                                                                             │
L1 Action ────────────────────────────────────────────────────────────────────►│
  │  ├── nt_io/nt_io_provider/ ──────────────────────────────────────────────►│
  │  │     ├── openai/, anthropic/, ollama/, gemini/, ...                     │
  │  │     └── ... (30+ LlmProvider implementations)                         │
  │  ├── nt_io/nt_io_llm/ ──────────────────────────────────────────────────►│
  │  │     └── UnifiedLlm, LlmRegistry                                       │
  │  └── nt_memory/, nt_media/, nt_task_decomposition.rs                     │
  │                                                                             │
  ▼                                                                             │
L0 Substrate ─────────────────────────────────────────────────────────────────►│
     ├── nt_core_types.rs, nt_core_event.rs, nt_core_ecs.rs, ...              │
     └── ... (30+ core modules)                                               │
```

## 7. 数据流向拓扑

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            Data Flow Topology                                   │
│                                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │   User       │    │  External    │    │   LLM        │    │   Data       │  │
│  │   Query      │───►│  APIs        │───►│   Providers  │───►│   Sources    │  │
│  └──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘  │
│         │                   │                   │                   │           │
│         ▼                   ▼                   ▼                   ▼           │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                     UnifiedEngine (Router)                              │   │
│  │  ┌─────────────────────────────────────────────────────────────────┐   │   │
│  │  │                    DataSourceRegistry                            │   │   │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │   │   │
│  │  │  │ MediaSource │ │ IntelSource │ │ OsintSource │               │   │   │
│  │  │  │  ┌───────┐  │ │  ┌───────┐  │ │  ┌───────┐  │               │   │   │
│  │  │  │  │29 src │  │ │  │3+ src │  │ │  │20+src │  │               │   │   │
│  │  │  │  └───────┘  │ │  └───────┘  │ │  └───────┘  │               │   │   │
│  │  │  └─────────────┘ └─────────────┘ └─────────────┘               │   │   │
│  │  │                                                                 │   │   │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │   │   │
│  │  │  │ LlmProvider │ │CrawlSource  │ │ LocalSource │               │   │   │
│  │  │  │  ┌───────┐  │ │  ┌───────┐  │ │  ┌───────┐  │               │   │   │
│  │  │  │  │30+src │  │ │  │crawl  │  │ │  │local  │  │               │   │   │
│  │  │  │  └───────┘  │ │  └───────┘  │ │  └───────┘  │               │   │   │
│  │  │  └─────────────┘ └─────────────┘ └─────────────┘               │   │   │
│  │  └─────────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                      │                                         │
│                                      ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                         Cache & Storage                                 │   │
│  │  SearchCache / MultiLevelCache / OfflineIndex / SQLite                  │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 8. 迭代更新指南

### 8.1 添加新数据源
1. 确定数据源类型 (Media/Intel/Osint/Llm/Crawl/Local)
2. 在对应 bridge 模块中创建适配器
3. 在 `DataSourceRegistry` 中注册
4. 更新 `UnifiedEngine` 构建方法
5. 运行 `cargo check` 验证

### 8.2 添加新域
1. 在 `SourceDomain` 枚举中添加新变体
2. 更新所有 match 语句
3. 在 `DataSourceRegistry` 中添加查询方法
4. 更新 `UnifiedEngine` 跨域查询逻辑

### 8.3 添加新能力
1. 定义新的 trait (如 `CrawlSource`)
2. 创建桥接层适配现有实现
3. 在 `AnyDataSource` 枚举中添加变体
4. 在 `DataSourceRegistry` 中添加注册和查询方法
5. 更新 `UnifiedEngine` 统一入口

## 9. 测试策略

### 9.1 单元测试
- 每个模块独立测试
- Mock 数据源用于测试
- 异步测试使用 `tokio::test`

### 9.2 集成测试
- 测试完整数据流
- 测试跨模块交互
- 测试错误处理

### 9.3 性能测试
- 测试并发查询性能
- 测试缓存命中率
- 测试源切换延迟

## 10. 部署拓扑

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           Deployment Topology                                   │
│                                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                      │
│  │   Tauri      │    │   CLI        │    │   API        │                      │
│  │   Desktop    │    │   Tool       │    │   Server     │                      │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘                      │
│         │                   │                   │                               │
│         ▼                   ▼                   ▼                               │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                      NeoTrix Core (neotrix-core)                        │   │
│  │  L0 ─► L1 ─► L2 ─► L3 ─► L4-L5 ─► L6                                   │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│         │                   │                   │                               │
│         ▼                   ▼                   ▼                               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                      │
│  │   SQLite     │    │   Vector     │    │   Object     │                      │
│  │   Database   │    │   Store      │    │   Storage    │                      │
│  └──────────────┘    └──────────────┘    └──────────────┘                      │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 11. 版本兼容性

- **Breaking Change**: 修改 trait 签名时增加 major 版本
- **New Feature**: 添加新 trait/模块时增加 minor 版本
- **Bug Fix**: 修复 bug 时增加 patch 版本

## 12. 文档规范

- 每个模块必须有 `//!` 文档注释
- 每个公开 API 必须有 `///` 文档注释
- 架构决策记录 (ADR) 存放在 `docs/adrs/` 目录

---

**注意**: 此架构文档是活文档，随项目迭代持续更新。每次重大架构变更后，必须同步更新此文档。

## 13. 对话面（neobot 即 neotrix 对外对话的一部分，2026-09-26 合体）

- 定位：neobot 不是 side 项目，是 neotrix L3 对话外表（Dialog Facet）。
  同工作区、同版本（workspace 0.21.0）、同库（`crates/neotrix-neobot` 不依赖 core，
  core 单向依赖 neobot lib，桌面壳只认 lib）。
- 单入口：`neotrix dialog <say|agent|models|provider|core|convo|task>`
  与 `neobot` 二进制同律（同 `load_config/open_store` + 同 `pool_models` 聚合），
  `neotrix-core/src/entry/dialog.rs` 薄封装。
- 三端：`neotrix dialog`（引擎 CLI）/ `neobot`（对话管理 CLI）/ NeoBot.app（桌面壳）。
  路由律一致：配对在线先晶体（模型透传池内解析），`env` 本地直连舱除外，离线回落。

## 14. 版本迭代记录（2026-09-26 大清洗）

- 基线：分支 `feat/capability-absorb-20260828` HEAD `8a11227a`；workspace 0.21.0 全员一致
  （当时 `apps/neobot-desktop` NeoBot 0.21.0 与 `src-tauri` NeoTrix 0.22.0 为不同产品，各自版本线）。
- **2026-09-28 更正**：上句括号里的两个桌面端**都不在本仓了** —— `src-tauri` 于
  `5c02e738` 归档、`apps/neobot-desktop` 于 `d5413335` 删除，桌面 App 统一到独立仓
  `~/Downloads/Neo/neobot`。本仓只留 `crates/neotrix-neobot` 作库。故「两个产品各自
  版本线」的说法已不适用于本仓；保留原句是为了让这段基线记录仍是当时的真话。
- 整合：本仓 9 worktree 已全合入 HEAD（diff 0，无需再合；删留待各窗确认）。
- 清洗：`sessions/handoff-global-todo-20260926.md` §8（G-01~G-11＋PARK）为唯一待修清单；
  §39 交接提示词见 `sessions/handoff-S39-20260926.md`。
- 对话面 (§13) + soul online（tools=9，crystal 0.2.0）为本迭代活体证据。

## 15. 统一进化迭代（34 源吸收，2026-09-26）

- 吸收战报＋12 缺口＋P0→P2 路线＋v0.22.0 目标＋归档计划＋任务清单：
  `sessions/handoff-evo-20260926.md`（34/34 成功，零编造）。
- P0 首点名建议：EVO-01 Token 成本门 → EVO-03 DSPy → EVO-02 judge 影子 → EVO-04 高速浏览器环。
- 版本 bump（0.21.0→0.22.0）留待独占窗口（重编风险，见该文件§3）。

## 16. 模块拓扑实测（2026-09-27 审计新增）

> 本节记录**代码实际形态**，与 §1–§2 的设计意图分开。设计图不反映现实时，
> 调试会一直在找不存在的东西 —— 本节就是为了让两者不再混为一谈。
> 可再生实测值与门禁见 `ARCHITECTURE-MAP-ROADMAP-V2.md` §11。

### 16.1 crate 拓扑：哪些是真 crate，哪些是空壳

| crate | 状态 | 说明 |
|---|---|---|
| `neotrix-core` | 主体 | 2,945 .rs 中约 87% 在此 |
| `crates/neotrix-{types,sysctl,game,neobot,audit,abilities,decision-engine}` | 正常 | — |
| `crates/neotrix-{reasoning,consciousness,multi-agent,gateway}` | **曾为空壳** | 抽取重构只搬了 Cargo.toml 与 `pub mod`，代码没搬：54 个 0 字节模块被 L5 当公开 API 再导出。2026-09-27 已摘除空壳门面，真实代码 27 文件 / 12,113 LOC 保留 |
| `crates/nt-lang` | 事实上的孤儿 | 只有 `[[bin]]` 无 `[lib]`，**结构上无法被任何 crate 依赖**；0 依赖者 |
| `crates/nt-core-capability-tree` | **住在 src/ 里的独立 crate** | 4,670 LOC / ~30 处真实调用，根 `Cargo.toml:11` 是 workspace member。路径违反直觉：`cargo` 从不把它当 neotrix-core 的模块编译 |

### 16.2 已拆除 / 曾经从未编译的模块

- **`neotrix-core/src/cli/`（87 文件 / 24,884 LOC）已删除。** 它曾已入库，
  但 `pub mod cli;` **从未出现在任何 commit 的 lib.rs 里** —— Rust 的目录模块
  只解析 `foo.rs` 或 `foo/mod.rs`，故这 87 个文件自入库起从未参与编译。
  替代落点见 `l3_embodiment/{nt_sandbox,nt_sandboxed_shell,nt_shield_enforcer}`、
  `l6_meta/{nt_approval,nt_cost_tracker,nt_laws,nt_permission_profiles,nt_agents_guard}`、
  `l1_action/nt_act/nt_act_cleanup/*`。
- **目录模块旁挂 `lib.rs` 副本**（`l5_cognition/lib.rs`，171 行）是从不被解析的
  陈旧副本 —— 也正因如此，它声明的 `mod nt_core_state;` 一直没暴露为错误。
  已删。这类副本会**掩盖真实的缺失依赖**，比缺文件更危险。

### 16.3 「存在 ≠ 生效」的三种形态（本仓反复出现）

1. **已入库但从未编译** —— `cli/` 树、6 个 md5 相同的重复文件。
2. **已编译但从不执行** —— `nt_act_trade/tests/` 下 3 个文件 (311 个
   `#[test]`) 因父 `mod.rs` 未声明而永不参与编译，`cargo test` 全绿也看不见。
   2026-09-27 已处理: 复活 223 个 (全绿), 删 88 个 (测的是已主动删除的
   `nt_mind::sales_coaching`)。棘轮基线现为 0 条。
   复活它们的过程中查出 5 个真实缺陷, 见 §16.4。
3. **已导出但为空** —— 0 字节模块被 `pub use` 成公开 API。

三者共同的隐蔽性在于：**编译器和测试运行器都报「绿」**。
因此 `scripts/check-truth-surface.sh` 把它们当门禁（退出码）而非文档约定。

### 16.4 复活测试查出的真实缺陷（2026-09-27）

`nt_act_trade/tests/` 的 223 个测试复活后并非「白捡」—— 它们查出了 5 个
此前无人察觉的实现缺陷。共同特征：**指标/统计看起来存在，实际是死代码**。

| # | 位置 | 缺陷 |
|---|---|---|
| 1 | `router.rs` `route()` | 统计记账用 `stats.by_worker.get(..).map(\|c\| c.fetch_add(..))`，而计数器**从未登记** → `by_worker`/`by_rule` 恒为空 map，`stats()` 永远返回空统计。`route()` 里那段是死代码 |
| 2 | `router.rs` `update_load()` | 负增量分支写成 `fetch_max(abs)` 再单独 `fetch_sub(..)`：既非原子（两步之间可被其他线程插入并覆盖），语义也不自洽（为做「饱和减」却先把值抬到 abs） |
| 3 | `orchestrator_v2.rs` 时长统计 | `start.elapsed().as_millis()` 向零截断 → 任何快于 1ms 的任务都记 0，「瞬间完成」与「根本没测量」在指标上不可区分 |
| 4 | `message.rs` `is_expired()` | 用 `num_seconds()` 截断 + `>` 比较 → 消息比 TTL 多活将近一整秒；且 `ttl=0` 的消息刚建好也不算过期 |
| 5 | `orchestrator_v2.rs` `message_rx` | 字段存在但 `message_rx.lock()` 出现 **0 次** —— 编排器只发不收，256 容量的通道发满后 `send()` 会永久挂起（潜伏的背压死锁） |

其中 #1 与 #5 是纯粹的「看起来有、实际不通电」，与 §16.3 的三种形态同源。
**教训：指标类代码最容易被当成「反正只是统计」而跳过验证**——
上表 5 项里有 4 项的表象就是「字段/函数都在，返回值也看着合理」。
