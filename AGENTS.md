# NeoTrix — Agent Guide (slim)

> 全量 codebase 索引已归档：`docs/architecture/CODEBASE-WIKI-2026-09-21.md`
> （2426 `.rs` / 798707 行快照，含文件树 / Domain 表 / Key Types / CLI 表）。
> 查文件位置用 `glob`/`grep` 现查，不要把全量表塞回本文件。本文件保持 < 100 行。

## Build & Test

```bash
cargo xl                              # 最轻检查（= check -p neotrix --lib），日常只用这个
cargo check --all-targets -p neotrix    # 快速检查
cargo test -p neotrix --lib             # 单元测试
cargo build -p neotrix                  # 完整构建
```

结构性改动后：`cargo clean && cargo build` 跑两遍，以拿到真实错误数。

## 并行公约（2026-09-21 事故复盘）

- 同一工作区只留 1 个 watcher，多任务用单窗口 Task 子代理；真并行走 `.worktrees/` 隔离。
- 禁多窗口同时跑 `--all-targets` / `--test` 全量构建（16G 机必爆 swap）。
- 关窗口前写 `sessions/handoff-<窗口>.md`（模板见 `sessions/HANDOFF-TEMPLATE.md`），收齐 + stash 兜底后再关。
- 写文件前重读（R-P16），禁整文件覆写他人内容；`stash pop / checkout -- <path>` 前先喊一声（2026-09-22 三次覆盖事故）。

## 模块前缀规范

- 所有模块名用 `nt_` 前缀（如 `nt_core_cache`、`nt_mind`、`nt_shield`）。
- 分层：`l0_substrate` / `l1_action` / `l2_perception` / `l3_embodiment` / `l4_emotion` / `l5_cognition` / `l6_meta`，详见 `docs/architecture/ARCHITECTURE.md`。
- 编码标准见 `RUST-STANDARDS.md`（生产代码禁 `unwrap`/`expect`/`panic!`，错误用 `?` 传播）。

## 硬规则

- `#![forbid(unsafe_code)]` —— 永不加 `unsafe`。
- 编辑后必须重读文件验证落盘（R-P16）。
- 外部技术必须同会话接到生产可用（R-P79）。

## 微操作公约（Agentation 思想吸收，skill: nt-locate）

- 改码前先定点：`python3 scripts/ops/nt_locate.py --component X --source-file Y`（选择器→文件:行），读上下文再下刀，无定点不改。
- 闭环：点选/标注 → 定点 → 最小改动 → 单测验证。

## 正典索引

- 架构：`docs/architecture/ARCHITECTURE.md`、`FUSION-ARCHITECTURE.md`
- 文档规范：`DOCUMENTATION-MAP.md`
- 路线图：`ARCHITECTURE-MAP-ROADMAP-V2.md`
- 待办：`TODO.md`（顶部为人工摘要区）；事故与分诊：`sessions/handoff-disease-list-20260927.md`

## 三道闸（2026-09-27 事故后置入，违反即阻塞）

- 重型 cargo 前：`sh scripts/ops/nt_mem_gate.sh; echo $?` — 非 0 禁止起构建
- 死锁静态扫描：`python3 scripts/ops/nt_lock_audit.py neotrix-core/src`（当前 0 命中）
- sidecar 按需：`sh scripts/ops/nt_sidecar.sh {start|stop|status}` — 用完即停
- 硬规则细则见 `RUST-STANDARDS.md` §17（锁/构建/卡死判别/Git/修 bug 判据/字节安全）
