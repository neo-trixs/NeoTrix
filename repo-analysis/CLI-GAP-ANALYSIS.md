# NeoTrix CLI — 与主流 Code CLI 差距分析

> 对标对象: Claude Code / OpenCode / Aider / Cursor CLI / Codex CLI
> 分析目的: 明确 NeoTrix CLI 在真实代码 agent 场景的缺失项和差异化优势

---

## 一、差距全景矩阵

| 能力维度 | NeoTrix | Claude Code | OpenCode | Aider | Cursor CLI | Codex CLI |
|----------|---------|-------------|----------|-------|------------|-----------|
| **文件编辑** | ❌ | ✅ 字符串替换 | ✅ 字符串替换 | ✅ SEARCH/REPLACE | ✅ 代理编辑 | ✅ 代理编辑 |
| **多文件协调** | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Git 集成** | ❌ | ✅ /diff/rewind/worktree | ✅ pr/github | ✅ 自动提交/undo | ✅ worktree | ✅ /review/fork |
| **交互式 REPL** | ✅ 半成品 | ✅ 成熟 TUI | ✅ 成熟 TUI | ✅ REPL | ✅ 成熟 TUI | ✅ 成熟 TUI |
| **Session 持久化** | ❌ | ✅ | ✅ | ✅ /resume | ✅ ls/resume | ✅ resume/fork |
| **`--json` 输出** | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ |
| **Shell 补全** | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ |
| **项目配置** | ❌ | CLAUDE.md | AGENTS.md | .aider.conf.yml | .cursor/rules/ | AGENTS.md |
| **模型切换** | ❌ | ✅ --model | ✅ -m | ✅ --model/--weak-model | ✅ --model | ✅ --model |
| **MCP CLI** | ❌ 仅库内 | ✅ mcp | ✅ mcp | ❌ | ✅ mcp | ✅ mcp |
| **Sandbox** | ❌ | ❌ | ❌ | ❌ | ✅ sandbox | ✅ sandbox |
| **Init 流程** | ❌ | ✅ --init | ✅ /init | ❌ | ✅ login | ✅ login |
| **Permission 模型** | ❌ | ✅ 4 种模式 | ✅ --dangerously | ❌ | ✅ --yolo | ✅ --full-auto |
| **流式输出** | ❌ 模拟 | ✅ | ✅ | ✅ | ✅ | ✅ |
| **子代理并行** | ✅ (库内) | ❌ | ✅ | ✅ architect+editor | ❌ | ✅ Code Cloud |
| **OpenAI 兼容** | ✅ (服务端) | ❌ | ❌ | ❌ | ❌ | ❌ |
| **自进化推理** | ✅ SEAL/E8 | ❌ | ❌ | ❌ | ❌ | ❌ |

**核心结论**: NeoTrix 在 21 个维度中, 14 个缺失或半成品, 7 个领先。缺失的 14 个全是**代码 agent 基本功**, 领先的 7 个是**推理引擎差异化**。

---

## 二、按严重程度分组

### P0 — 阻塞级 (不可用)
> 没有这些, NeoTrix 就是一个 "能推理但不能干活" 的 CLI

| # | 差距 | 现状 | 主流做法 | 影响 |
|---|------|------|---------|------|
| 1 | **文件读写编辑** | 无任何文件编辑能力 | SEARCH/REPLACE 块 / 字符串替换 / git apply | 不能改代码, 不能作为 code agent 使用 |
| 2 | **多文件协调** | 无 | 单回复多编辑块 / 子代理并行 | 不能改超过 1 个文件, 任何实用场景都不可用 |
| 3 | **Git 集成** | 无 | 自动 commit + undo + diff preview | 无版本追踪, 改错无法回滚 |
| 4 | **权限/确认流** | 无 | approve-on-write / sandbox gate | 无安全护栏, 不能在生产项目使用 |

### P1 — 严重 (可用但痛苦)
> 有基本编辑能力后, 这些决定是否留下来

| # | 差距 | 现状 | 主流做法 | 影响 |
|---|------|------|---------|------|
| 5 | **--json / --output-format** | 无 | `{text|json|stream-json}` | 不能 pipe 给其他工具,不能 CI 集成 |
| 6 | **Session 持久化** | TUI session 内存态,退出消失 | `~/.claude/sessions/` 存磁盘 | 不能 resume, 不能 fork |
| 7 | **项目级别配置** | 无 | CLAUDE.md / AGENTS.md / .cursor/rules | 不能告诉 agent 项目规范 |
| 8 | **Init 流程** | 无 | `claude --init` / `codex login` | 用户不知道如何开始 |
| 9 | **--help / 子命令补全** | 手动 arg 解析, 无 clap 导出 | clap derive + `completion` subcommand | -h 不全,zsh 不补全 |

### P2 — 体验 (锦上添花)
> 核心能力就绪后, 这些决定用户黏性

| # | 差距 | 现状 | 主流做法 | 影响 |
|---|------|------|---------|------|
| 10 | **进度/流式渲染** | 50ms 轮询 + 模拟分块 | 真实 SSE 逐 token | 大任务看不到进度 |
| 11 | **Vim 编辑模式** | 仅 backspace, 无编辑 | codex vim mode / aider nano | 输入效率低 |
| 12 | `/undo` `/diff` 等命令 | 无 | 每回合查看/回滚 | 不能精细化控制 |
| 13 | **Shell 补全脚本** | 无 | `opencode completion zsh` | 减少认知负担 |
| 14 | **MCP CLI 子命令** | 库内 MCP 但 CLI 不暴露 | `claude mcp add` / `opencode mcp list` | 不能管理工具 |

---

## 三、差异化优势（NeoTrix 有、竞品没有）

> 这些是构建 CLI 时应保留并强化的 USP

| 能力 | 实现位置 | 描述 |
|------|---------|------|
| **SEAL 自进化** | `self_iterating/` — 2556 行, 55 tests | 每次推理后自动调整能力向量, 积累策略原则 |
| **E8 推理状态机** | `core/e8/` — 2030 行 | 64 态六十四卦推理, 非线性共振转移 |
| **HyperCube VSA** | `core/hypercube/` — 402 行 | MAP 超向量, 4096 维 bundle/bind/permute |
| **Awakening 自测量** | `awakening/mod.rs` — 259 行 | Φ, FCS, USK 觉醒度量, 因果预测 |
| **GWT 注意力路由** | `core/consciousness/` — 1141 行 | 专家模块通过 salience 竞争广播 |
| **OpenAI 兼容 API** | `server/openai_compat.rs` — 新 | `/v1/chat/completions` 端点到 SEAL |
| **基准测试套件** | `clawbench_bench.rs` + `reasoning_bench.rs` | 量化的推理质量评估 |

---

## 四、混合定位策略

```
NeoTrix 不应尝试复制 Claude Code / Aider
而应定位为: 推理引擎提供商 (OpenAI 兼容) + 轻量 CLI 外壳

               ┌─────────────────────────────┐
               │    neotrix CLI (轻客户端)     │
               │  ┌───────────────────────┐   │
               │  │ 编辑功能 (SEARCH/     │   │
               │  │ REPLACE + git apply)  │   │
               │  └───────────────────────┘   │
               │            ↕                  │
               │  ┌───────────────────────┐   │
               │  │ 推理引擎 (E8+SEAL+    │   │
               │  │ Awakening) (USP)      │   │
               │  └───────────────────────┘   │
               │            ↕                  │
               │  ┌───────────────────────┐   │
               │  │ OpenAI 兼容 API 服务   │   │
               │  │ ← OpenCode/Aider 调   │   │
               │  └───────────────────────┘   │
               └─────────────────────────────┘
```

### 四象限分类

|  | 传统 Code Agent (Claude Code/Aider) | 推理增强引擎 (NeoTrix) |
|---|---|---|
| **编辑能力** | ✅ SEARCH/REPLACE, git 深度集成 | ❌ 缺失, 需构建 |
| **推理深度** | 单模型 prompt → 编辑 | ✅ E8 64 态转移 + SEAL 自进化 |
| **可衡量提升** | 无 | ✅ ClawBench + Awakening Φ 度量 |
| **生态接入** | 独立 CLI | ✅ OpenAI 兼容 → OpenCode/Aider 作为前端 |

### 构建优先级

```
Phase 1 — 编辑能力 (P0, 2-3 天)
  ├── file_read:  读取文件到 context
  ├── file_edit:  SEARCH/REPLACE 块 (复用 Aider 匹配算法)
  ├── file_write: 新文件写入
  ├── git diff:   编辑后显示变化
  └── git commit: 自动 commit

Phase 2 — CLI 重构 (P1, 1 天)
  ├── clap 重构:  子命令 + --help + shell completion
  ├── --json:     benchmark/server 命令支持 JSON 输出
  ├── --model:    模型选择
  └── session persist: TUI session 落盘

Phase 3 — 安全护栏 (P0, 1 天)
  ├── permission model: read-only / acceptEdits / fullAuto
  ├── confirmation on write
  └── sandbox mode (文件系统隔离)

Phase 4 — 体验优化 (P2, 2 天)
  ├── 真实 SSE 流式输出
  ├── /diff / /undo 命令
  ├── MCP CLI 子命令
  └── init 流程
```

---

## 五、文件编辑方案选型

| 方案 | 复杂度 | 容错性 | 参考 | 推荐 |
|------|--------|--------|------|------|
| **SEARCH/REPLACE 块** | 中 | 高 (模糊匹配) | Aider | ✅ **首选** |
| 精确字符串替换 | 低 | 低 | Claude Code | 备选 |
| 完整文件写入 | 低 | 低 | OpenCode | 只用于新文件 |
| LLM 生成统一 diff | 高 | 中 | — | 不推荐 (易错) |

Aider 的 SEARCH/REPLACE 匹配策略已经过生产验证: 精确匹配 → 空格灵活 → 省略号 (`...`) → 模糊匹配, 失败后通知 LLM 修正。

---

## 六、关键决策

1. **不重建 TUI 框架** — 现有 ratatui 5 面板已经够用, 优先补 Session 持久化 + 流式渲染
2. **不写 Permission 系统** — 直接用文件系统 ACL (只读模式 open file, 写模式 by default 需要确认)
3. **借用 Aider 的 SEARCH/REPLACE** — 复用 `aider` 的匹配逻辑 (AGPL? 需检查许可证), 或自己实现模糊匹配
4. **OpenAI 兼容 API 优先于 CLI 编辑** — 服务端模式让 OpenCode/Aider 直接调用, CLI 编辑稍后构建
5. **Benchmark 作为 CLI 招牌** — `neotrix bench all` 输出量化证据, 这是竞品都没有的能力
