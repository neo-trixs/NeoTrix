# NeoTrix 前端重新审计报告 (2026-08-19) — FPAM 吸收后复审

基于 5 仓库吸收经验（iPolloWork/cumora/claude-code/codex/osaurus）对 P0-P3 改动后的 neocodex-frontend 复审。
方法：FPAM（Invariant Freeze + ADI 三遍 + 覆盖率矩阵 + 严重度分级）。基线 109 测试全绿 + tsc clean + 零依赖环。

## 一、Invariant Freeze（本次审查不变量）

| 不变量 | 状态 |
|---|---|
| 组件禁止直连 `@tauri-apps/*`（经 api/ 层） | ✅ TrafficLights 已收敛，但 SettingsModal 违约 |
| 组件禁止直连 invoke | ✅ 全部经 api/ |
| API 错误统一经 ApiError 信封（errText/toApiError） | ⚠️ 信封已建，消费者未全接线 |
| localStorage 经 lib/env.ts 收敛层 | ⚠️ tags.ts 已收敛，SettingsModal 直连 |
| 测试 mock 一律经 vi.mock('@tauri-apps/api/core') | ✅ |

## 二、覆盖率矩阵

| 维度 | 对照源 | 结果 | 状态 |
|---|---|---|---|
| 错误信封三要素 | iPolloWork | code 已建，8+ 组件仍 `String(e)` | 📢 Warning |
| wire 契约守卫 | iPolloWork/codex | types.test.ts 已建 | ✅ |
| 共享查询缓存 | iPolloWork | query.ts 已接线 2 消费方 | ✅ |
| 轮询去重+visibility | osaurus | CoworkView/ScheduledTasks 自带 | ✅ |
| 单流式事实源 | osaurus | 仅 Chat.tsx 一处 | ✅ |
| 零依赖环 | iPolloWork | madge 验证通过 | ✅ |
| 域模块无死代码 | D44 | 9 域全部有消费方 | ✅ |
| 路由身份(URL-scoped) | iPolloWork | 3 静态路由，无参数化 | ℹ️ 设计取舍 |
| runtime-env 收敛 | cumora | env.ts 建 + 2 消费方 | ⚠️ 半完成 |
| CI 门禁 | — | coverage 脚本修复 + typecheck | ✅ |

## 三、发现清单（按严重度）

### 📢 Warning-1: 架构约定执行不彻底（SettingsModal 双重违约）
- `src/components/SettingsModal.tsx:2-3` 直连 `@tauri-apps/plugin-dialog` + `@tauri-apps/plugin-fs`
- `:201,:224` 直连 localStorage，绕过 env.ts
- 违反 audit 报告 F1 与本次建立的收敛层约定
- 建议: 下沉 `api/fs.ts`（saveFileDialog + writeTextFile 封装）+ localStorage 改 `storageGet/Set`

### 📢 Warning-2: 错误信封半落地（errText 未被消费者采用）
- `String(e)` 错误转写遍布 SettingsModal(8处)/ComputerUse(4)/ProjectView/GitPanel/Chat
- 刚建的 `errText()`（ApiError 感知）零生产消费
- 建议: 组件统一改 `setError(errText(e))`，让 code 语义可用（未来差异化提示）

### 📢 Warning-3: SettingsModal 巨石 1574 行
- 多 section 单文件（general/api-key/mcp/memory/plugins/updates 6 区）
- 对照 iPolloWork domains 拆分思路：按 section 拆子组件，props 传 store
- 这是继 Chat.tsx 后第二大拆分点

### ✅ Info-1: 轮询循环已单 Driver（已修复）
- 抽 `lib/usePolling.ts` hook：统一 enabled 守卫 / in-flight 去重 / visibility 守卫 / cleanup / immediate 首刷
- 4 处接线完毕：Chat(15s,immediate)/CostDashboard(5s,enabled)/CoworkView(10s)/ScheduledTasks(10s,enabled)
- 生产代码 setInterval 清零（仅 hook 内一份）；6 测试

### ℹ️ Info-2: 路由无 URL 身份
- `/ /chat /globe` 静态路由，会话身份在全局 store
- iPolloWork 的 `/workspace/:id/session/:id` 模式在单会话桌面 app 价值有限
- 若未来支持多窗口/深链（codex resume 模式）再引入

## 四、修复优先级（R-P42 强化现有节点）

1. ✅ **W-2 错误接线**（已完成）：12 组件 `String(e)` → `errText(e)`，生产路径 String(e) 清零
2. ✅ **W-1 API 下沉**（已完成）：建 `api/fs.ts`（saveFileDialog/openFileDialog/writeTextFileAt），SettingsModal + PluginMarketplace 移除 tauri 插件直连，localStorage 改 storageGet/Set
3. ✅ **W-3 SettingsModal 拆分**（已完成）：抽 8 文件——GeneralSection/AppearanceSection/DataSection/TagsSection/AboutSection/McpSection/TagRow/settingsIcons；update 状态机下沉 AboutSection 自管理；破坏性确认统一父组件回调。**1574 → 591 行**（-983）；清理死信号 themePref
4. ✅ **I-1 usePolling hook**（已完成）：`lib/usePolling.ts` 统一 4 处轮询，生产 setInterval 清零，6 测试

## 五、结论

C1→C2 的架构基础已夯实（信封/契约/缓存/收敛层/CI 门禁）。
本轮重审证明：**约定建立了，但消费端接线不彻底**（W-1/W-2 皆是"建而不用"）。
按 R-P79 纪律已同 session 接线闭环：**错误信封 + fs 收敛 + env 收敛全部接入生产路径**。
基线 124 测试全绿 + tsc clean + 零依赖环 + 组件零 tauri 直连（唯一例外 TrafficLights 窗口 API）。