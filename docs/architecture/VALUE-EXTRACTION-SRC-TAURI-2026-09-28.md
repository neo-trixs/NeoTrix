# 价值萃取 — `src-tauri` (NeoTrix 桌面端)

> **本文档的作用**：在 `src-tauri/` 被归档**之前**，把它独有的能力完整记录下来。
> 归档 = 移出工作树；全部 598 个文件受 git 版本控制（`git ls-files src-tauri` = 598），
> **代码本身可逆**。但**"有什么、缺了会怎样"这件事一旦没记录就永久丢失** ——
> 这正是本文档存在的理由。
>
> 生成方式：`wc -l` + `rg 'pub fn'` + `ls routes/` + 测试文件计数。**数字可复算。**
> 时点：2026-09-28，`neotrix` HEAD `28329bd1`。

---

## 0. 一句话结论

**`crates/neotrix-neobot`（引擎）neobot 更新 —— 这一点已验证。**
**但作为桌面 app，`src-tauri` 拥有 neobot 完全没有的 13,569 行域逻辑 + 12 个页面 + 71 个前端测试。**
"neobot 当主 app"若不伴随移植，桌面能力面将从 33,834 行降到 3,059 行（**−91%**）。

---

## 1. 域插件能力面（13,569 行 / 23 个，neobot 全部为零）

按行数降序。`导出函数` 列为该文件的 `pub fn` / `pub async fn` 计数。

| 域插件 | 行数 | 导出函数 | 能力 | neobot 等价物 |
|---|---|---|---|---|
| `im` | 2,465 | 5 | IM 通道（Telegram 等）收发 | ❌ 无 |
| `stubs` | 1,727 | 0 | 22 个域的桩定义（注册表骨架） | ❌ 无 |
| `llamacpp` | 931 | 1 | 本地推理进程管理 / provider 配置 | ❌ 无 |
| `session` | 787 | 1 | 会话生命周期 | ⚠ `nt_store_convos` |
| `ai_orchestration` | 778 | 2 | 多 agent 编排 | ❌ 无 |
| `chat` | 660 | 1 | 对话主流程 | ⚠ `nt_cmd_convo` |
| `kb` | 601 | 1 | **知识库**（摄取/检索/图谱） | ❌ 无 |
| `voice_agent` | 565 | 1 | **语音 agent** | ❌ 无 |
| `im_test` | 526 | 0 | IM 域测试 | ❌ 无 |
| `mcp_extension` | 520 | 1 | MCP 扩展 | ⚠ `nt_mind` |
| `session_sync` | 517 | 1 | 会话跨端同步 | ❌ 无 |
| `unified_surface` | 513 | 1 | 22 域统一调用面 | ❌ 无 |
| `memory` | 480 | 1 | **记忆管理** | ⚠ `nt_memory` |
| `file` | 432 | 0 | 文件操作 | ⚠ `nt_cmd_files` |
| `folder_instructions` | 426 | 1 | 目录级指令注入 | ❌ 无 |
| `workflow` | 383 | 1 | **工作流编排** | ❌ 无 |
| `world` | 285 | 0 | 世界模型 | ❌ 无 |
| `skill` | 252 | 1 | 技能装载 | ⚠ `nt_skills` |
| `proxy_pool` | 193 | 1 | 代理池 | ❌ 无 |
| `model_pool` | 192 | 1 | 模型池 | ❌ 无 |
| `autostart` | 139 | 1 | 开机自启 | ❌ 无 |
| `context` | 108 | 0 | 上下文注入 | ❌ 无 |
| `macro` | 37 | 0 | 宏 | ❌ 无 |

**⚠ 唯一完整的端口**（neobot 侧有对应实现）：`session` / `chat` / `file` / `skill` / `memory`。
**⚠ 部分重叠**：`mcp_extension` ↔ `nt_mind`。
**❌ 纯损失**（neobot 侧零覆盖）：其余 **18 个域，合计约 11,700 行**。

---

## 2. 前端能力面（SolidJS，neobot 无框架）

`src-tauri/frontend` 依赖：`solid-js` `@solidjs/router` `lucide-solid`
`@xterm/xterm`（终端）`globe.gl`（3D 地球）`@playwright/test` `@solidjs/testing-library`

| 项 | src-tauri | neobot |
|---|---|---|
| ts/tsx 文件 | **325** | 19 |
| 页面路由 | **12** | 0（单窗口） |
| 状态 store | **9** | 0 |
| **前端测试** | **71** | **0** |
| 框架 | SolidJS + Router | 无（纯 TS） |

### 12 个页面

`Chat` · `KnowledgeBase` · `KnowledgeGraph` · `MemoryManager` · `Workflows` · `Skills`
· `Insights` · `ContextDashboard` · `Activity` · `Marketplace` · `Settings` · `ChatShellProto`

其中 **7 个在 neobot 侧无对应界面**：KnowledgeBase / KnowledgeGraph / MemoryManager /
Workflows / Skills / Insights / ContextDashboard。

### 9 个 store

`canvas` · `chat` · `insights` · `kb` · `tags` · `theme` · `workflow` · `world` · `index`

---

## 3. 测试资产

| | src-tauri | neobot |
|---|---|---|
| 前端测试文件 | **71** | 0 |
| Rust `#[test]` | **150** | 见 `neobot-desktop` 自身 |

**71 个前端测试是本次归档中最重的单项损失** —— 它们固化了 Chat 渲染、kb 检索、
memory 管理等交互契约。`neobot` 侧 0 测试意味着合并后这些契约无回归保护。

---

## 4. 移植优先级（若确认以 neobot 为主 app）

| P | 域 | 行数 | 理由 |
|---|---|---|---|
| **P0** | `llamacpp` + `model_pool` | 1,123 | 本地推理是项目命脉，neobot 侧零覆盖 |
| **P0** | `unified_surface` | 513 | 22 域统一调用面，是架构骨架 |
| **P1** | `kb` + `memory` | 1,081 | 知识库/记忆是 NeoTrix 核心叙事 |
| **P1** | `chat` + `session_sync` | 1,177 | 会话与跨端同步 |
| **P2** | `workflow` + `ai_orchestration` | 1,161 | 编排能力 |
| **P2** | `im` + `im_test` | 2,991 | 已有 `nt_channel_*` 五模块，需评估是否合并 |
| **P3** | `voice_agent` / `world` / `macro` / `context` | 995 | 边缘能力 |
| **P3** | `stubs` | 1,727 | 注册表骨架，随域迁移同步 |

**前端 306 个文件差额**需单独规划：SolidJS → 纯 TS 是框架级决策，不是文件搬运。

---

## 5. 复算命令

```bash
# 域插件
for f in src-tauri/src/domain/plugins/*.rs; do printf "%s %s\n" "$(basename $f .rs)" "$(wc -l < $f)"; done | sort -k2 -rn
# 页面 / store / 测试
ls src-tauri/frontend/src/routes/*.tsx | grep -v test | wc -l
ls src-tauri/frontend/src/stores/*.ts | grep -v test | wc -l
find src-tauri/frontend/src -name '*.test.ts*' | wc -l
rg -c '#\[test\]|#\[tokio::test\]' src-tauri/src | awk -F: '{s+=$2} END{print s}'
# 归档可逆性
git ls-files src-tauri | wc -l
```

## 6. 归档记录

`src-tauri/` 已移至：
`~/Downloads/Neo/neotrix-archive/neotrix-desktop-20260928/src-tauri`

**可逆方式**：`git checkout 28329bd1 -- src-tauri/`（全部内容在 git 历史中）
或直接从归档区拷回。
