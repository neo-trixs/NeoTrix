# NeoTrix

Self-evolving reasoning kernel. Rust workspace，10 个 crate，`neotrix-core` 按
`l0_substrate` → `l6_meta` 七层组织。

## 快速开始

```bash
cargo xl                                       # 日常最轻检查（= cargo test -p neotrix --lib）
cargo check --all-targets -p neotrix          # 快速检查
cargo build -p neotrix                         # 完整构建
```

结构性改动后跑 `cargo clean && cargo build` **两遍**。重型构建前先过内存门：

```bash
sh scripts/ops/nt_mem_gate.sh; echo $?         # 非 0 禁止起构建
```

## 从哪里读起

| 我要 | 读 |
|---|---|
| 动手前的纪律与门 | [`AGENTS.md`](AGENTS.md)（§0 决策树直接给「意图 → 动作 → 判据」） |
| 架构现状 | [`docs/architecture/ARCHITECTURE.md`](docs/architecture/ARCHITECTURE.md) §13 起 |
| 唯一图纸 | [`docs/architecture/NEOTRIX-MASTER-BLUEPRINT.md`](docs/architecture/NEOTRIX-MASTER-BLUEPRINT.md) |
| 唯一排期真源 | [`docs/architecture/FINAL-ROADMAP-2026-09-29.md`](docs/architecture/FINAL-ROADMAP-2026-09-29.md) |
| 开发规则正典 | [`docs/standards/NEOTRIX-STD-1.0.md`](docs/standards/NEOTRIX-STD-1.0.md) · [`RUST-STANDARDS.md`](RUST-STANDARDS.md) |
| 贡献指南 | [`CONTRIBUTING.md`](CONTRIBUTING.md) · 目录规范 [`DOCUMENTATION-MAP.md`](DOCUMENTATION-MAP.md) |

## 硬规则

`#![forbid(unsafe_code)]`；生产代码禁 `unwrap`/`expect`/`panic!`，错误用 `?` 传播；
模块名一律 `nt_` 前缀。细则见 `AGENTS.md`。

## 门

`scripts/` 下 19 个 `check-*.sh` + `scripts/ops/`，pre-commit / pre-push 已接。
查「该跑哪个」：`make find QUERY="死锁"`（19 条意图索引，每条带「何时别用」）。

## 本地模型

权重放 `<repo>/models/`（gitignored，删了只能重下）。启动 llama.cpp **必须**带
`--jinja` `--reasoning off` `--ctx-size <N>`，否则 Qwen3.5 系「装完开不了话」。

---

MIT · v0.23.0 · Rust ≥ 1.81
