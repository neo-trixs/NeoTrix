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
