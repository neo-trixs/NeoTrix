# NeoTrix Desktop V2 — 前端交互功能完善 & 后端 API 统一对接设计

## 1. 当前状态总览

### 1.1 后端 Domain Plugin 架构

已注册 12 个域插件（`main.rs:62-74`）：

| 域 | 实现状态 | 数据库 | 关键 actions |
|---|---|---|---|
| **session** | ✅ 真实实现 | `desktop.db` (SQLite) | create/delete/switch/rename/tag/archive/restore/search/list/clear/export |
| **chat** | ✅ 真实实现 | `desktop.db` | send_stream/stop/history/compact/regenerate/side_chat |
| **kb** | ✅ 真实实现 | `knowledge.db` | kv_set/kv_get/kv_list/node CRUD/fts_search |
| **memory** | ✅ 真实实现 | `memory.db` | list/search/clear/stats/timeline |
| **file** | ✅ 真实实现 | - | read/write/tree/diff/search |
| **agent** | ⚠️ 部分实现 | config.toml | provider_config/provider_status/pool_sufficiency/discover_models |
| **plugin** | 🔴 stub | - | list/install/uninstall/enable/disable |
| **workflow** | 🔴 stub | - | list/create/delete/run/status |
| **tool** | 🔴 stub | - | mcp_list/mcp_register/harness_execute |
| **system** | 🔴 stub | - | window/pty/update/config |
| **security** | 🔴 stub | - | scan/permission/stealth |
| **ext** | 🔴 stub | - | remote/channel/cowork |

### 1.2 前端 API 双轨问题

```
路径 A:  component → neocodex.ts → adapter.ts → domain_call (fallback to invoke)
路径 B:  component → domain.ts → domain_call 直接
路径 C:  skills.ts/memory.ts → client.ts → invoke 直接 (不经 adapter)
```

**问题**：
- `adapter.ts` 的 DOMAIN_MAP 映射了 70+ 命令到 domain，但 action 名称可能与后端不匹配
- fallback 机制隐藏了真实失败
- 三套路径并行，维护成本高

### 1.3 前端功能完成度

| 路由 | UI | API 接线 | Mock 残留 |
|---|---|---|---|
| Chat | ✅ 完整 (1167+ 行) | ✅ neocodex_* 全接 | 无 |
| Insights | ✅ 完整 | ⚠️ agent_status 接线 | `insights.ts` 纯 mock |
| KnowledgeBase | ✅ 完整 | ⚠️ kb_doc_* 接线 | `kb.ts` 库级 mock |
| Skills | ✅ 完整 | ✅ skill_* 接线 | 无 |
| MemoryManager | ✅ 完整 | ✅ memory_* 接线 | 无 |
| Workflows | ✅ 完整 | ⚠️ workflow_* 走 stub | 后端 stub |
| Marketplace | ✅ 完整 | ✅ market_* 直接 invoke | 无 |

## 2. 统一 API 对接设计

### 2.1 架构决策

**方案**：废弃 `adapter.ts`，所有 API 统一经 `domain.ts` 的 typed helpers。

```
component → api/{domain}.ts → domain.ts::call() → invoke('domain_call', { domain, action, args })
```

### 2.2 Action 名称对齐表

前端 `adapter.ts` 的 action 映射 → 后端 domain plugin 实际 action：

| 前端命令 | adapter 映射 | 后端 domain | 后端 action | 对齐 |
|---|---|---|---|---|
| `neocodex_list_sessions` | session.list_sessions | session | `list` | ⚠️ 需对齐 |
| `neocodex_create_session` | session.create_session | session | `create` | ⚠️ 需对齐 |
| `neocodex_delete_session` | session.delete_session | session | `delete` | ⚠️ 需对齐 |
| `neocodex_switch_session` | session.switch_session | session | `switch` | ⚠️ 需对齐 |
| `neocodex_rename_session` | session.rename_session | session | `rename` | ⚠️ 需对齐 |
| `neocodex_tag_session` | session.tag_session | session | `tag` | ⚠️ 需对齐 |
| `neocodex_send_message_stream` | chat.send_message_stream | chat | `send_stream` | ⚠️ 需对齐 |
| `neocodex_stop_stream` | chat.stop_stream | chat | `stop` | ⚠️ 需对齐 |
| `neocodex_provider_config` | agent.provider_config | agent | `provider_config` | ✅ |
| `neocodex_agent_status` | agent.status | agent | `status` | ✅ |
| `kb_doc_ingest` | kb.doc_ingest | kb | `doc_ingest` | ⚠️ 需确认 |
| `memory_stats` | memory.stats | memory | `stats` | ✅ |
| `memory_search` | memory.search | memory | `search` | ✅ |
| `skill_list` | - (直接 invoke) | - | - | 需走 domain |

### 2.3 统一策略

**Phase 1: 对齐 action 名称**
- 后端 session plugin 的 action 名称改为 `list_sessions`/`create_session`/`delete_session` 等（与前端 adapter 映射一致）
- 或前端改为使用 domain.ts 的 `session.list()` 等（与后端一致）

**选择**：前端改用 `domain.ts` 的 typed helpers（因为 domain.ts 已经定义了正确的 action 名称）

**Phase 2: 迁移 api/*.ts 模块**
- `api/neocodex.ts` → 拆分为 `api/session.ts` + `api/chat.ts` + `api/agent.ts`
- `api/skills.ts` → 改用 `domain.ts` 的 typed helpers 或新增 `skill` domain
- `api/memory.ts` → 改用 `domain.ts` 的 `memory` typed helpers
- `api/kb.ts` → 改用 `domain.ts` 的 `kb` typed helpers
- `api/workflows.ts` → 改用 `domain.ts` 的 `workflow` typed helpers
- `api/market.ts` → 改用 `domain.ts` 的 `plugin` typed helpers 或新增 `market` domain

**Phase 3: 删除 adapter.ts**
- 移除旧命令映射
- 保留 `client.ts` 仅作为 `invoke` 的类型安全封装

## 3. 实现计划

### Phase 1: 后端 action 名称对齐（本次）

需要修改的后端文件：

1. **`domain/plugins/session.rs`** — 对齐 action 名称
   - `list` → `list_sessions`（或前端改用 `list`）
   - `create` → `create_session`
   - `delete` → `delete_session`
   - `switch` → `switch_session`
   - `rename` → `rename_session`
   - `tag` → `tag_session`
   - `untag` → `untag_session`
   - `archive` → `archive_session`
   - `restore` → `restore_session`
   - `search` → `search_sessions`
   - `clear` → `clear_session`
   - `export` → `export_session`

2. **`domain/plugins/chat.rs`** — 对齐 action 名称
   - `send_stream` → `send_message_stream`
   - `stop` → `stop_stream`
   - `history` → `get_session_messages`
   - `compact` → `compact_session`
   - `regenerate` → `regenerate`
   - `side_chat_get` → `get_side_chat`
   - `side_chat_send` → `send_side_chat`

3. **`domain/plugins/kb.rs`** — 补齐缺失 actions
   - `doc_ingest` → 确认是否存在
   - `doc_list` → 确认是否存在
   - `doc_delete` → 确认是否存在
   - `doc_reindex` → 确认是否存在

4. **`domain/plugins/memory.rs`** — 补齐缺失 actions
   - `export` → 确认是否存在
   - `import` → 确认是否存在

5. **新增 skill domain** — 或在 agent domain 中添加 skill actions
   - `skill_list` → 扫描 skills 目录
   - `skill_get` → 读取单个 skill
   - `skill_search` → 搜索 skills

6. **新增 market domain** — 或在 plugin domain 中添加 market actions
   - `market_status` → 市场状态
   - `market_search` → 搜索插件
   - `market_install` → 安装插件
   - `market_uninstall` → 卸载插件
   - `market_list_installed` → 已安装列表

### Phase 2: 前端 API 迁移（本次）

1. **创建 `api/session.ts`** — 会话域 typed helpers
2. **创建 `api/chat.ts`** — 对话域 typed helpers
3. **创建 `api/agent.ts`** — Agent 域 typed helpers
4. **修改 `api/skills.ts`** — 改用 domain 调用
5. **修改 `api/memory.ts`** — 改用 domain 调用
6. **修改 `api/kb.ts`** — 改用 domain 调用
7. **修改 `api/workflows.ts`** — 改用 domain 调用
8. **修改 `api/market.ts`** — 改用 domain 调用
9. **删除 `api/adapter.ts`** — 废弃旧路由
10. **修改 `api/neocodex.ts`** — 保留为兼容层，内部调用新模块

### Phase 3: 功能接线（本次）

1. **Insights 页** — `stores/insights.ts` 接线后端数据源
   - 后端需实现：`insights_daily`、`insights_weekly`、`provider_usage_snapshot`
   - 或在 agent domain 中添加这些 actions

2. **KnowledgeBase 页** — `stores/kb.ts` 库管理接线
   - 后端需实现：`kb_library_list`、`kb_library_create`、`kb_library_rename`、`kb_library_delete`

3. **Workflows 页** — 后端 workflow plugin 从 stub 变为真实实现

### Phase 4: 验证（本次）

1. `npx vite build` — 前端构建通过
2. 所有路由页面 API 调用测试
3. 确保无 fallback 隐藏失败

## 4. 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| 后端 action 名称改动导致旧命令失败 | 高 | 保留旧命令作为别名，或一次性迁移 |
| 部分 domain plugin 未实现 | 中 | stub 返回合理默认值，UI 显示占位 |
| 前端 API 迁移引入 bug | 中 | 逐模块迁移，每步验证构建 |
| Insights/KB mock 切换后数据为空 | 低 | 保留 mock 作为 fallback |

## 5. 文件变更清单

### 后端 (src-tauri/src/)
- `domain/plugins/session.rs` — action 名称对齐
- `domain/plugins/chat.rs` — action 名称对齐
- `domain/plugins/kb.rs` — 补齐 doc_* actions
- `domain/plugins/memory.rs` — 补齐 export/import actions
- `domain/plugins/stubs.rs` — agent domain 补齐 provider 相关 actions
- `domain/plugins/mod.rs` — 新增 skill/market domain（或在现有 domain 中添加）

### 前端 (neocodex-frontend/src/)
- 新增 `api/session.ts` — 会话域 typed helpers
- 新增 `api/chat.ts` — 对话域 typed helpers
- 新增 `api/agent.ts` — Agent 域 typed helpers
- 修改 `api/skills.ts` — 改用 domain 调用
- 修改 `api/memory.ts` — 改用 domain 调用
- 修改 `api/kb.ts` — 改用 domain 调用
- 修改 `api/workflows.ts` — 改用 domain 调用
- 修改 `api/market.ts` — 改用 domain 调用
- 删除 `api/adapter.ts` — 废弃旧路由
- 修改 `api/neocodex.ts` — 保留为兼容层
- 修改 `api/index.ts` — 更新导出
- 修改 `stores/insights.ts` — 接线后端数据源
- 修改 `stores/kb.ts` — 库管理接线
