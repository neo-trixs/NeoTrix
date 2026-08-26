# App 功能完善路线图 — 吸收相关 AI 桌面应用

> 日期: 2026-08-25 · 状态: Phase 1 已交付, Phase 2 待启动
> 纪律: R-P42 (强化现有节点/复用组件 embedded 模式) · mock-first seam (UI 先行, 后端后接, 组件零改动切换) · R-P79 (每批次接线门) · R-P100 (落地即登记)
> 上游: 会话 sess_20260825_app_phase1 已交付 `/kb` + `/plugins` + Sidebar 导航 (366/366 测试绿)

## 0. 现状盘点 (evidence-first)

| 维度 | 现状 | 证据 |
|------|------|------|
| 后端命令面 | **325+** Tauri 命令 (kb_cmds 用 `#[command]` 宏未计入旧统计) | `src-tauri/src/commands/*.rs` |
| 前端接线覆盖 | ~120 命令被 api/* 引用 (≈37%) | `comm fe_words × be_cmds` |
| 路由 | `/` `/chat` `/globe` `/kb` `/plugins` 共 5 条 | `App.tsx` |
| 组件库 | 33 个组件已存在 (CostDashboard/GitPanel/TerminalPanel/TaskList/ProjectView/CoworkView/ComputerUse/CheckpointTimeline…) | `components/` |
| 设置中心 | SettingsModal 7 section: general/models/appearance/plugins/data/tags/about — **models section 是壳** | `SettingsModal.tsx:28` |
| KB 后端 | kb_cmds.rs 直连 `knowledge.db` (nodes/edges/pack), 前端零消费 | `kb_cmds.rs` |
| memory api | memoryStats/Search/Timeline + API key 存取已有 | `api/memory.ts` |

## 1. 吸收映射表

| # | 功能面 | 吸收源模式 | 目标节点 (R-P42) | 判定 |
|---|--------|-----------|------------------|------|
| A1 | 模型服务商管理页 | LM Studio / Cherry Studio 服务商卡片+连通性测试 | SettingsModal models section 充实 | 强化 |
| A2 | 洞察仪表盘页 | LobeChat analytics + grok-bot Usage & Billing 诚实标注 | CostDashboard 逻辑页面化 + ProviderUsageLedger 视图 | 强化 |
| A3 | KB 文档级管理 | Cherry Studio 库内文档列表+索引状态流 | stores/kb.ts 扩展 docs 域 | 强化 |
| A4 | 技能中心页 | Claude Code skills 面板 | 新路由 /skills + skill_cmds | 新增(具名消费者) |
| A5 | 工作流页 | n8n/Dify 列表+运行态 | 新路由 /workflows + workflow_cmds | 新增(具名消费者) |
| A6 | 记忆管理页 | Chatbox 数据管理 | memory.ts 已有 API → 页面化 | 强化 |
| A7 | 对话搜索/分支 | Chatbook/LobeChat 对话历史搜索 | chatStore 过滤增强 | 强化 |
| A8 | 安全扫描中心 | — (NT-SHIELD 自有) | security_scan_cmds → 页面 | 新增(具名消费者) |

## 2. Phase 2 — 功能面完善 (mock-first)

### B1 洞察页 `/insights` (最高优先: 复用度最大)
- 文件: `routes/Insights.tsx` + `stores/insights.ts`
- 内容: 成本卡 (复用 query 'agent_status' 缓存) + per-provider 用量表 (`global_provider_usage_ledger` 快照命令待暴露 → 见 P3-M2) + insights_daily/weekly 卡片
- seam: `InsightsDataSource` 接口, mock 返回种子序列
- DoD: 页面测试 ≥4; CostDashboard 弹窗保留不删 (入口双轨)

### B2 KB 文档级管理
- 文件: `stores/kb.ts` 扩展 `listDocs/uploadDoc/removeDoc/reindex`; KnowledgeBase.tsx 加库详情视图 (点卡片进抽屉)
- 状态流: ready/indexing/failed 三态徽章 (mock 定时翻转 indexing→ready)
- DoD: 详情抽屉渲染 + 状态流测试 ≥3

### B3 模型服务商管理 (SettingsModal.models 充实)
- 文件: `components/settings/ModelsSection.tsx` (从 SettingsModal 抽出)
- 内容: 服务商卡片列表 (env key 存在性→状态点) + 连通性测试按钮 + 默认模型选择; 数据源 `ProviderDirectoryDataSource` (先读 `discovery.rs` BUILTIN_FREE_PROVIDERS 形状做 mock)
- DoD: 与 provider_cmds 对接面明确列出; 测试 ≥4

### B4 技能中心 `/skills`
- 文件: `routes/Skills.tsx` + `api/skills.ts` (skill_list/get/search 已有后端)
- 内容: 技能卡网格 + 详情侧滑 + 搜索
- DoD: 直接接真后端 (命令已存在, 无需 mock)

### B5 工作流页 `/workflows`
- 文件: `routes/Workflows.tsx` + `api/workflows.ts`
- 内容: 列表+运行态+schedule 标签; 运行按钮触发 run/run_status 轮询
- DoD: 同 B4 直连后端

### B6 记忆管理页 `/memory`
- 文件: `routes/MemoryManager.tsx` — memoryStats/Search/Timeline 三面板
- DoD: 直连后端

## 3. Phase 3 — 后端对接矩阵

| Seam | 切换实现 | 消费命令 (R-P79 消费者具名) |
|------|----------|---------------------------|
| P3-M1 `KbDataSource` → tauri | `api/kb.ts` invoke 封装 | `kb_nodes_list/kb_node_create/...` (以 kb_cmds.rs 实际导出为准, 接线时 grep `#[command]`) |
| P3-M2 ledger 快照命令 | src-tauri 新增只读 command `provider_usage_snapshot()` 读全局账本 | insights 页用量表 |
| P3-M3 `ProviderDirectoryDataSource` | `provider_list/test_connection` | models section |
| P3-M4 insights seam | `insights_daily/weekly/trend/stats` | Insights 页 |
| P3-M5 query 缓存统一 | 全部新页面走 `api/query.ts` TTL 缓存 (对标 CostDashboard 3s agent_status) | 一致性 |
| P3-M6 错误信封 | 所有 invoke 经 `toApiError` 归一 (client.ts 已有) | 一致性 |

顺序原则: 先直连页 (B4-B6 无 seam 成本) → 再 seam 切换页 (B1-B3)。每完成一项跑全量 vitest + typecheck。

## 4. Phase 4 — 打磨 (对接完成后)

- 命令面板接入页面导航 (CommandPalette 增加 "打开知识库/洞察…" 动作)
- 深链: `neotrix://page/kb` (deep_link 插件已有)
- 键盘: `⌘1..⌘5` 页面切换
- 空态引导文案统一走 design-tokens 语义色
- E2E 冒烟: tauri-driver 打包后跑 5 路由可达性

## 5. R-P79 合规矩阵

| 批次 | 消费者 | 状态 |
|------|--------|------|
| Phase 1 (/kb /plugins 导航) | 用户可见路由 × Sidebar 入口 × App.tsx 路由表 | ✅ 已闭环 |
| B1 | Insights 页渲染 ledger/daily | 待启动 |
| B4/B5/B6 | api/skills·workflows·memory 直连既有命令 | 待启动 (无 mock 成本) |
| P3-M2 | 新增只读 command 被 insights 页调用 | Phase 3 首项 |

## 6. 验收门 (每批次)

1. `npm test` 全绿 (基线 366+, 只增不减)
2. `npm run typecheck && npm run build` 绿
3. 新交互均有 aria-label + 测试断言 (对标既有 a11y 纪律)
4. mock-only 页必须在 seam 注释标注对应后端命令名 (防"忘接线"死代码)
5. 落地即 `neotrix-capability bud/strengthen` 登记
