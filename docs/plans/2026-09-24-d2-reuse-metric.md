# D2 结晶复用率 — 设计（2026-09-24，只设计 + 基线实测）

> 上游：`docs/plans/2026-09-24-human-inspired-refinement.md` §D2（M2，权重 0.2）。
> 复用率 = 意识体的"智商表"：从未知基线做到可度量，周环比涨。

## 1. 现状定点（均已读/实测）

| 层 | 位置 | 状态 |
|---|------|------|
| 结晶器 | `nt_mind/nt_mind/auto_crystallizer.rs`（`total_crystallized`/`unverified` 计数器，反幻觉门+幻觉桶） | ✅ 有；计数器内存态 |
| 复用字段 | `l6_meta/nt_core_self/skill_crystal.rs:38,44` `use_count`/`last_used`，`CrystalRegistry`（上限 50，自动剪枝） | ⚠️ 字段有，但 +1 只发生在**重提炼去重**（`:142-153`），不是真实调用复用；registry 无持久化（KB 无 crystal 表） |
| 持久 helper | KB `causal_rules` 170 条 / `procedural_memory` 26 条 / 磁盘 skills 33+25 | ✅ 有；无调用日志 |
| 经验池 | KB `experience` 表 6694 行 + kv `experience` 10409 键；`skills_index` 0、`insights` 0、`evolution_records` 0 | 池深，结晶出口窄 |

## 2. 基线快照（2026-09-24 11:25，只读实测）

```text
causal_rules=170  procedural_memory=26  experience=6694(+kv 10409)
disk_skills=33+25  skills_index=0  insights=0  evolution_records=0
reuse_rate=未知（无 invocation 日志，无法计算——这就是 D2 第一刀）
```

## 3. 度量定义

```text
helper(t)      = 当周新增结晶单元（causal_rule + procedural + crystal + skill doc）
reused(h, W)   = h 在创后 W=7 天内被调用 ≥1 次（调用 = 路由命中/加载/引用）
reuse_rate(W)  = |{h: reused}| / |{h: created}|          （主指标，周 cohort）
reuse_depth    = Σuse_count / |helpers|                  （辅指标，调用深度）
```

目标：首月把 reuse_rate 从"未知"做到可度量，之后周环比 +5pp（refinement 原话：周环比涨）。

## 4. 插桩（三处最小改动，实现等内存窗口）

1. **持久化**：KB 新增 `helper_use_log(helper_id, ts, context)` + registry 落盘
   （`CrystalRegistry` → kv `crystal_registry`，复用 route_learner 跨会话存活模式）。
2. **调用打点**：`match_trigger`（`skill_registry.rs:33`）、`find_best_agent_for`
   （`nt_infra_agent_card.rs:372`）、skill load 路径各加一行 `record_use(id)`。
   注意与 D1 同点位——两针同打一次，避免重复改。
3. **周报**：`nt_reporter.sh` 加一行 `report-reuse`（读 KB 聚合，周环比），
   输出 `sessions/report-reuse.md`。

## 5. 验证

- `cargo xl` + 新单测（打点单调性：调 3 次 use_count=3；落盘 roundtrip）。
- 首周基线落盘即 D2 交付（未知→已知），之后每周同一口径复算。

## 6. 工时序

插桩（1h）→ 跑一周自然流量 → 首份 reuse 周报（即基线）→ 按周跟踪。
