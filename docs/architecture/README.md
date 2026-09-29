# docs/architecture 阅读索引（SIM-27）

> 先读本页，再动手。一页只说三件事：读什么、不读什么、什么已过时。

## 必读（按顺序）

1. `NEOTRIX-MASTER-BLUEPRINT.md` — 唯一图纸入口（D-00~D-15，按图施工）
2. `NEOTRIX-STD-1.0.md` 在 `../standards/` — 唯一规则正典（64→70 条款，MUST/SHOULD）
3. `SIM-PROTOCOL.md` — SIM 登记表（§5）＋最新 SIM（永远读最后两节）
4. `NEOTRIX-IMPLEMENTATION-ROADMAP.md` — WHEN/WHO/DONE（§5 阶段＋§7 节点卡）

## 按需查阅

- `SDB-REGISTRY.md` — SDB 登记（先看版本行诚实口径，再看表）
- `IMPACT-ANALYSIS.md` — 改动影响面（动手前查 blast radius）
- `ABSORPTION-ROUND*.md` — 外部模式出处（论证用，不是指令）
- `SESSION-ABSORPTION-2026-09-21.md` — 12 条血泪教训（新人必读半页）
- `SDB-REGISTRY.md` 的 §Verifier、`REFACTORING-GUIDE.md` — 设计细节库

## 已过时／别读（或只读备案）

- `NEOTRIX-FULL-ARCHITECTURE.md` — 非本会话产物，未纳入索引，内容可能过期
- `ARCHITECTURE.md` — L4-L5 合并写法，早于 L0-L6；围栏疑似失衡（SIM-22 finding，待主人重写）
- `TODO.md` — 2026-09-20 基线已 STALE（横幅在案），P0 开工先重跑 check
- ~~`docs/dev-rules.md`、根 `dev-rules.md`~~ — ✅ 均已于 2026-09-29 删除；legacy 内容在 `../standards/archive/`，规则以正典为准

## 铁律

图上没有的不做；没有验证命令的任务不开工；一次只填一张图的一格。
