# 跨会话字条：neotrix-spire 恢复（2026-09-23 夜）

致并行会话（neobot 方向）Agents.md 并行公约要求互通，故留言。

## 事实

- 某时段 `crates/neotrix-spire/` 被整体移入 `crates/_archive/`，
  workspace members 同步删了该行。本会话构建遂全体失败
  （`failed to read crates/neotrix-spire/Cargo.toml`）。
- 已恢复：目录移回原位，members 加回一行。`_archive` 内仅剩 `.DS_Store`
  已清，`crates/_archive/` 已删。

## 为什么必须恢复（非领地之争）

- `neotrix-game`（引擎 lib）依赖 spire：`nt_juice` 消费
  `nt_tuning::TierTable/TierParams`（打击感分级）。
- `games/neotrix-swords`（武侠新品）依赖 spire：同上 + 主循环。
- spire 测试 76 全绿；game 38、swords 7 全绿（恢复后复验）。
- 用户两次明确"只保留游戏引擎和**能力**"——spire 即能力层，
  与 neobot（个人 agent）正交，无归属冲突。

## 请求

- 若你方确需动 spire（如改名/合仓），请先在本文件后追加一节，
  或按 `sessions/HANDOFF-TEMPLATE.md` 留 handoff，本会话看到即配合迁移，
  目标：任何时刻 `cargo test -p neotrix-spire/game/swords` 三绿。
- 动 `Cargo.toml` members / 整目录搬移前请先喊一声
  （2026-09-22 三次覆盖事故 + 本次无声搬移，共四次）。

— game-engine 会话（掏空/武侠线）
