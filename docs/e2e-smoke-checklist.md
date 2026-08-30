# App E2E 冒烟清单 — tauri-driver 骨架 (Phase 4 收尾)

> 状态: 清单就绪, 待打包环境启用 · 覆盖 vitest 无法验证的集成面
> 前置: `npm run tauri build` 产物 + WebDriver 已启动 (`tauri-driver` + WebKitWebDriver)

## 1. 路由可达性 (7 页)

| # | 动作 | 断言 |
|---|------|------|
| R1 | 启动 app | `/chat` 渲染, Sidebar 可见 |
| R2 | 点击侧栏「知识库」 | URL `/kb`, 标题「知识库」 |
| R3 | 点击「插件」 | `/plugins` 市场列表挂载 |
| R4 | 点击「洞察」 | `/insights` 成本卡渲染, progressbar 存在 |
| R5 | 点击「技能」 | `/skills` 技能卡 ≥0 且域分组头存在 |
| R6 | 点击「记忆」 | `/memory` 统计四格渲染 |
| R7 | 点击「流程」 | `/workflows` 列表或空态 |

## 2. 快捷键 (补 vitest 2 个 it.skip)

| # | 动作 | 断言 |
|---|------|------|
| K1 | 真实键盘 ⌘2 | 跳转 `/kb` (**对应 PageShortcuts.test skip#1**) |
| K2 | ⌘7 | `/workflows` (**skip#2**) |
| K3 | 输入框聚焦时 ⌘3 | 不跳转 |

## 3. Deep Link

| # | 动作 | 断言 |
|---|------|------|
| D1 | shell 执行 `open "neotrix://page/kb"` (macOS) | app 激活且路由至 `/kb` |
| D2 | 冷启动 `open neotrix://page/insights` | 直达 `/insights` |

## 4. 后端命令回路 (真 DB)

| # | 动作 | 断言 |
|---|------|------|
| C1 | /kb 「入库」填 title+text 提交 | 文档列表出现该行, 状态「已索引」; `sqlite3 ~/.neotrix/knowledge.db "SELECT count(*) FROM nodes WHERE metadata LIKE '%kb_doc%'"` 增长 |
| C2 | 删除该文档 | 行消失, DB 计数回落 |
| C3 | 设置→模型→测试连通 (配 base_url 的 provider) | 出现 ● 延迟 或 ○ 不通 (无网络时) |
| C4 | 触发一次对话后开 /insights | 用量账本出现真实 provider 行 |

## 5. 边界

- B1 断网状态跑 R2-R7 (provider 相关面板不白屏)
- B2 debug 构建验证 updater 未注册 (NEOTRIX_UPDATER 门, 打包边界纪律)

## 实现说明

- 骨架采用「清单驱动」而非 tauri-driver 脚本: 首轮人工/半自动执行并记录,
  第二轮再固化为 `tauri-driver` + WebDriver 脚本 (元素选择器依赖打包产物 DOM)。
- K1/K2 即 `src/components/PageShortcuts.test.tsx` 中两个 `it.skip` 的正式补验位;
  通过后回填移除 skip。
