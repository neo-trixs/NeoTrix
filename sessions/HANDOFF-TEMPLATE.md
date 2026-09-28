# Handoff 交接模板（复制改名用）

> 用法：复制本文件为 `sessions/handoff-s000.md` / `handoff-s001.md` / `handoff-s002.md`，
> 在旧窗口里让 agent 按下面结构填写，写完必须重读一遍确认落盘。
> **2026-09-27 变更**：`.gitignore` 已从「整目录忽略 `sessions/`」改为「忽略 `sessions/*` + 白名单
> `handoff-*.md` 与本模板」。**交接文档现在入库** —— 因为 `AGENTS.md` 与 `RUST-STANDARDS.md §17`
> 都引用它们作为证据来源，模板不入库则交接规范不可执行。
> 代价：`sessions/` 下 71 份 `exp-session`/`report-*` 草稿仍被忽略，不入库（这是想要的）。

## 1. 会话标识

- 窗口：s000（tty 名，如 s000/s001/s002）
- 分支：`feat/capability-absorb-20260828`（写实际分支）
- 交接时间：2026-09-21

## 2. 目标（一句话）

> 这个窗口原本要干什么？

## 3. 已完成

- [ ] 已做的事1（附文件路径 + 行号，如 `neotrix-core/src/agent.rs:120`）
- [ ] 已做的事2

## 4. 正在改的文件（关键！逐个列）

| 文件完整路径 | 改到什么程度 | 是否可独立提交 |
|---|---|---|
| 例：`neotrix-core/src/agent.rs` | 重构了 X 函数，未编译验证 | 否，需配合 Y |

## 5. 下一步（按优先级排序）

1. 下一步1（具体到文件+动作）
2. 下一步2

## 6. 阻塞点

- 阻塞1：缺什么信息 / 等谁的改动 / 哪个测试过不去
- 无阻塞写"无"

## 7. 给接手会话的话

- 恢复命令：先读本文件，再跑 `git status --short` / `git diff --stat` 核对
- 禁止事项：不要跑 `cargo check --all-targets`，只用 `cargo check -p neotrix --lib`
- 风险提示：与其他窗口改了同一文件？（写清楚文件名）

## 8. 收工自查（2026-09-28 起**必填**，空着视为交接未完成）

收工是**自己的义务**，不是接手者的义务。以下三项逐条回答，不得留空：

### 8.1 worktree 去向

- 跑 `sh scripts/ops/nt_worktree_gate.sh check`，把输出粘在这里：

```
<粘贴 check 输出>
```

- 本会话**新建**的 worktree，逐个说明去向（`prune --force` 收掉 / 明确移交他人 /
  仍活跃需后续）：

| worktree | 用途 | 去向 |
|---|---|---|
| 例：`.worktrees/xxx` | 隔离某改动 | 已 `prune --force` 移除 / 移交窗口B / 仍活跃 |

- **禁止手删 worktree 目录**（`rm -rf .worktrees/*`）—— 2026-09-28 实测有 850 处
  未提交改动不在任何提交里，手删即永久丢失。必须走 `prune`（内建双闸 + patch 兜底）。

### 8.2 未提交改动的去向

列出本会话结束时的所有未提交改动，逐个说明**最终去了哪**：

| 文件 | 改动内容 | 去向（勾一个） |
|---|---|---|
| 例：`neotrix-core/src/foo.rs` | 修了 X | ☐ `git add` 已提交（`<hash>`） ☐ patch 兜底（路径） ☐ 明确弃用（理由） |

> 「留给下一个 agent」**不算合法去向**。弃用也要写明理由，让接手者能判断。

### 8.3 门状态

- `sh scripts/ops/nt_worktree_gate.sh check` 的 exit code：`<0/3/4>`
- 提交前是否跑过 `cargo xl` / `cargo check -p neotrix --lib`：☐ 是 ☐ 否
- 若有门红（如分层门 `N new`），写清是「他窗 WIP」还是「本会话引入」：
