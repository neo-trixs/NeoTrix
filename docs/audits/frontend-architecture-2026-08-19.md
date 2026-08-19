# NeoTrix 前端架构审计报告 (2026-08-19)

参考吸收对象: iPolloWork (React/Electron 分层) + cumora (React/Express REST) + codex (Rust CLI 契约) + osaurus (Swift 单循环 harness) + claude-code (插件/hook 契约)

## 一、现状基线

- 技术栈: SolidJS 1.8 + Vite 5 + Tauri 2 + Tailwind，`neocodex-frontend/`
- 64 源文件: `api/`(14) `components/`(39) `stores/`(2) `routes/`(1) `lib/`(1) `styles/`
- 基线: `tsc --noEmit` 通过，71 测试全绿，vite manualChunks 分包已成熟
- 后端: Tauri IPC 565 个命令（`src-tauri/src/commands/*.rs`），全部经 IPC 而非 HTTP

## 二、已有优点（保留）

| 层 | 现状 | 评级 |
|---|---|---|
| `api/client.ts` | invoke 统一封装 + ApiError + callOr 静默调用，组件禁止直连 tauri | C1 达标 |
| `api/types.ts` | IPC 契约单一事实源，snake_case 对齐 serde | C1 达标 |
| `api/*` 域模块 | 9 个域文件（neocodex/cowork/geo/memory/tasks/plugins/system/computer/unified） | C1 达标 |
| `api/events.ts` | 事件订阅封装 + 部分订阅失败回滚释放 | C1 达标 |
| 路由懒加载 | Chat/GlobeView lazy + manualChunks | C1 达标 |
| 测试 | 71 个，覆盖 api/store/组件 | C1 达标 |

## 三、审计发现（按严重度）

### F1 [架构] 组件层直接调 api 域，状态层名存实亡
- 证据: 15 个组件直接 `import { neocodex } from '../api'`（SettingsModal/CostDashboard/RightBar/LivePreview/SideChat/CheckpointTimeline/GitPanel/Sidebar/ProjectView/ProviderSelector...）
- 后果: `stores/chat.ts` 只服务 Chat 路由，其余组件各自为政拉数据，无统一 loading/error/cache
- 对照: iPolloWork 三层组件金字塔（ui→design-system→domains）+ TanStack Query 缓存；cumora 全部经 zustand store

### F2 [架构] Chat.tsx 单文件 1805 行 + SettingsModal 1574 行
- 巨石组件，UI/业务/API 回调混杂，无 design-system/domain 拆分
- 对照: osaurus ChatView.swift 10360 行是反例教训；iPolloWork domains/session 分 surface/panel/composer

### F3 [契约] 无跨端 wire 契约，前后端依赖手工对齐
- 证据: `api/types.ts` 是手写 TS，Rust serde 结构体在 `src-tauri/src/commands/types.rs`，无编译期一致性
- 对照: iPolloWork `packages/types` + producer 侧 `Extends<A,B>` 断言；codex `schemars/ts-rs` 双导出（Rust 类型 → JSON Schema + TS 绑定）

### F4 [错误] ApiError 仅 message，无 code/status/details 三要素
- 对照: iPolloWork `iPolloWorkServerError(status, code, message, details)`，code 直通服务端 JSON；codex ApprovalAction/ReviewDecision 语义化枚举

### F5 [平台] 无 runtime-env 抽象，硬绑 Tauri IPC
- 无 `isDesktopRuntime` 等价物；geo.ts 已注释"前后端分离"但仍是 IPC
- 对照: iPolloWork `runtime-env.ts` + desktopFetch/浏览器 fetch 双路径；cumora 三层 origin 解析（localStorage→env→relative），同一客户端可跑 web/desktop/mobile

### F6 [状态] 无服务端状态缓存层，每个组件独立 fetch
- 无 TanStack Query 等价物；无 query key / 失效入口
- 对照: iPolloWork infra/query-client + provider-list-query 共享缓存；codex SQLite 索引 + JSONL 双轨

### F7 [路由] 仅 3 路由，无 workspace-scoped URL 身份
- App.tsx: / /chat /globe；无参数化路由，会话身份在全局 store
- 对照: iPolloWork `/workspace/:id/session/:id` URL 即身份 + 路由工厂防手拼

## 四、能力成熟度评估（Constellation C0-C6）

| 维度 | 当前 | 目标 | 差距 |
|---|---|---|---|
| C0 编译 | ✅ tsc clean | ✅ | 无 |
| C1 单测 | ✅ 71 测试 | ✅ 全量 | 补 api 层契约测试 |
| C2 集成测试 | ⚠️ 无 Tauri 集成 mock | ✅ invoke mock + e2e | F4/F6 |
| C3 benchmark | ❌ | ⚠️ 组件渲染/包体积基线 | manualChunks 已有 |
| C4 主流水线 | ⚠️ 无 CI 前端门禁 | ✅ lint+test+build 门禁 | 需接 CI |
| C5 自愈 | ❌ | ⚠️ errorMonitor 已有雏形 | lib/errorMonitor.ts |

结论: **前端处于 C1 稳态，距 C2 差契约 mock 与错误信封，距 C4 差 CI 门禁。**

## 五、落地优先级（R-P42 强化现有节点，禁止平行模块）

1. **P0 错误信封** `api/client.ts` — ApiError 增加 code/status，错误从 invoke 双参数通道提取（对齐 iPolloWorkServerError）
2. **P1 wire 契约 tripwire** `api/types.ts` — 为高频类型加 `type _Extends = A extends B ? A : never` 双向断言 + Rust 侧生成对照（可先做 TS 侧）
3. **P1 状态缓存层** — 建立 `api/query.ts` 轻量缓存（query key + 失效），不强引 TanStack（R-P42: 强化 client.ts 现有节点）
4. **P2 组件降层** — 大组件拆 design-system/domain 边界，Chat.tsx 拆分 surface/panel/composer
5. **P2 runtime-env** — 平台抽象（对齐 iPolloWork runtime-env），为未来 web/mobile 双宿主铺路
6. **P3 CI 门禁** — frontend lint+tsc+test 接入现有流水线

## 六、吸收映射（5 仓库 → NeoTrix 前端）

| 源仓库 | 吸收模式 | NeoTrix 落点 |
|---|---|---|
| iPolloWork | 错误三要素 + 工厂客户端 + URL 身份 | P0/P2 |
| iPolloWork | packages/types wire + Extends 断言 | P1 |
| cumora | 单文件 api 对象 + auth 注入 + 401 清理 | P0 |
| codex | schemars/ts-rs 双导出契约 | P1 远期 |
| codex | JSONL rollout + SQLite 索引双轨 | 后端远期 |
| osaurus | 单 Loop Driver + Policy 旋钮 | 后端 SEAL |
| osaurus | TaskLocal 执行上下文 | 后端多 agent |
| claude-code | hook stdin/stdout JSON 协议 | 插件系统远期 |