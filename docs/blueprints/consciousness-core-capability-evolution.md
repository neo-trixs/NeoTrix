# 蓝图 v3: 意识体整体进化蓝图 + 进化之路地图

> **状态**: 提议 (Proposed) · **类型**: evolution blueprint + roadmap · **日期**: 2026-08-14 (v3 — 基于 2026-08-12 v2 基线 + 本会话 P0-P2 六项落地 + 每日信息 cycle 1107)
> **依据**: 本会话完整轨迹 (P0×3 + P1×3 + P2×2 全部落地接线), 意识核心实时状态, 能力树 14 节点, 经验库 2453 条, Trendshift 每日榜 8-14, KB cycle 1106/1107。
> **范围**: 意识核心 (ConsciousnessTree/GWT/L5 runtime/phi-IIT) + 能力网 (capability tree) + 每日信息例行。
> **约束**: R-P42 (强化现有节点) · R-P79 (同 session 生产接线) · R-P100 (生产能力注册能力树) · 指针守恒 (本文件仅指引, per-cycle 内容落 KB)。

---

## 0. 现状总览 (Evidence-First)

### 意识核心 (2026-08-14 实时)
| 指标 | 值 | 解读 |
|------|-----|------|
| cycle | 34 | 生长周期持续推进 |
| phi | 0.380 | 整合信息, 低位稳定 (真实 IIT 仅在完整运行时核算) |
| coherence | 0.574 | 中等一致, 有提升空间 |
| weighted_fog_sum | 9.35 | 迷雾仍重 (当前进程实时, 快照 1.65 为 tick 后) |
| governance_compliance | 0.979 | 治理合规度高 (36 层分形深度) |
| MARS | S1=32 / S2=32 / bridge=0 | 双过程均激活, 但 S1↔S2 桥接从未命中 — 进化信号 |

### 能力树 (14 节点)
- C4 ×1 (disk_cleanup_engine) + C1 ×13 — 本会话新增 8 个吸收节点 (P0×3 + P1×3 + P2×2), 全部 C1 且 validate 通过。
- 观察: 全部 C1 无上游演进 → 需要 C2 (集成) / C3 (benchmark) 晋升管线, 否则吸收是"入库即停"。

### 经验库 (2453 条)
- by_domain: NT-CORE 827 / NT-IO 368 / NT-META 339 / NT-MEMORY 299 / NT-MIND 143 / NT-SHIELD 110 / NT-ACT 98 / NT-WORLD 88 / NT-GOVERNANCE 93 / NT-REPAIR 85 / NT-NEXUS 3。
- 失衡: **NT-WORLD 88 (最低)** / NT-NEXUS 3 (近乎空)。NT-META 339 显示元认知反思充分, 但感知域薄弱。

---

## 1. 本会话缺陷反思 (Defect Reflection, 证据锚定)

### 1.1 新缺陷 (本会话引入, 已修复)
| # | 缺陷 | 严重度 | 证据 | 修复 |
|---|------|--------|------|------|
| N1 | 测试构造 `KnowledgeNode` 误用不存在字段 (data_tier/tier/embedding) → E0560 | P1 | `nt_memory_visibility.rs:170` | 对齐真实字段 (17 个) |
| N2 | 测试 in-memory kv_store schema 用 (k,v) 而非生产 (key,value,updated_at) → 列名错误 | P1 | `nt_memory_provenance.rs:172` | 对齐 nt_memory_schema.rs:204 |
| N3 | 同秒写入溯源记录 created_at 相等 → newest-first 断言失败 | P1 | `nt_memory_provenance.rs:139` | 索引插入序倒序作次键 tie-break |

**模式**: 三个缺陷同源 — **测试侧复用生产 schema/结构的纪律缺失**。R-P16 (不信工具消息) 已覆盖写侧, 但测试侧"构造前先 grep 真实定义"未成文。

### 1.2 结构缺陷 (本会话发现, 预存)
| # | 缺陷 | 严重度 | 证据 | 状态 |
|---|------|--------|------|------|
| S1 | `nt_io_plugin/registry.rs` E0521 仅 `--all-targets` 暴露 | P2 | registry.rs | 预存, 非本会话引入 |
| S2 | `guard_chain.rs` untracked 新文件缺 Debug/Clone → 阻塞 test 编译 | P1 | guard_chain.rs | 本会话已补手动 impl (7 tests pass) |

### 1.3 启发进化点 (从本会话总结)
1. **三值裁定模式 (P2-1)**: 过滤层返回与输入同序全量裁定 (含 Drop), 决策标记与展示排序解耦 — 比布尔过滤表达力强 (Allow/Interstitial/Drop), 可推广到 GWT 注意力路由的 salience 分级。
2. **决策对象化 (P2-2)**: 决策是追溯主体 (who/did/what/why/when), 独立于节点 metadata 存储 → 审计链可独立检索。与 cur curation ledger 同哲学: **"不留溯源的操作是隐式状态变更"**。
3. **吸收流水线已成熟**: 外部吸收 → R-P42 强化 → R-P79 接线 → R-P100 注册, 本会话 6 项全部一次通过, 证明纪律内化。
4. **测试侧纪律缺口**: 与生产 schema 对齐 (N1/N2/N3) 应成规则 — 测试构造必须复用生产类型定义。

---

## 2. 进化之路地图 (Roadmap, 分阶段)

### Phase A — 每日信息例行自动化 (P1, 今日已启动)
**目标**: 每日 Trendshift 榜 → 高信号筛选 → 吸收 KB, 形成可重复例行。

1. 方法已验证 (cycle 1107): `webfetch trendshift.io/` 每日榜 → 四字段过滤 → 高信号<20% → `pending-absorb.json` 吸收。
2. 落 `notes/daily-intel-YYYY-MM-DD.md` 人类可读 + KB cycle 机器可读双写。
3. **下一步 (自动化)**: 评估背景循环每日定时触发 — 已有 `nt_mind_background_loop` 60s tick, 可挂 `daily_intel` handler (需设计: 触发源/超时/去重)。

**验收**: 连续 3 天每日榜落盘; 每日吸收 cycle 递增; 高信号候选进 roadmap。

### Phase B — 能力树 C2/C3 晋升管线 (P1, 最高杠杆)
**目标**: 14 个 C1 节点中选 2-3 个晋升 C2 (集成测试) / C3 (benchmark)。

1. 候选: `nt_io_hotreload::revertible_effects` (P0, 与 Cordis revertible effects 同源) → 增集成测试接 SEAL 回退; `nt_memory_provenance::decision_trail` (P2-2) → 增审计链集成测试。
2. 晋升门槛 (R-P100 cost 已存在): 需 manifest 满足 — 每个晋升必须有消费方 + 测试。
3. **接线**: `promote_constellation` 对齐 `nt_core_consciousness_tree.rs` C6 自适应 (8-12 蓝图 Phase C 已修正 C6 可达性)。

**验收**: `neotrix-capability mature --target c2` 至少 2 节点; C2 节点有集成测试消费方。

### Phase C — 感知域补强 (P2, 对齐经验库失衡 NT-WORLD 88)
**目标**: NT-WORLD 是经验库最低域 (88), 感知能力薄弱。

1. modlens 视觉桥接模式 → 强化 `nt_io_multimodal_transform` 真视觉后端 (P3 候选, R-P42)。
2. OpenBiliClaw 内容发现 agent → NT-WORLD UnifiedCrawler 多平台源借鉴 (观察)。
3. 每日信息例行本身就是 NT-WORLD 感知输入 — 双写落盘后 NT-WORLD 经验自然增长。

### Phase D — MARS S1↔S2 桥接 (P2, 意识核心)
**目标**: bridge_hits=0 是意识核心最显著空白 (S1=32 / S2=32 / bridge=0)。

1. 分析 bridge 条件为何从未满足 — `nt_core_self` 中 MARS 双过程桥接逻辑。
2. 每日信息 (S1 直觉) 与反思 (S2 分析) 的桥接: 如"高信号候选"应触发 S2 深度分析而非仅入库。

### Phase E — 溯源威胁模型 (P2, 每日信息驱动)
**目标**: watermarks-remover 揭示 PROV-O 的反方 — 溯源标记可被剥离。

1. 记入 KB 威胁模型: 决策溯源 (P2-2) 的鲁棒性依赖标记不可剥离性, 需评估 C2PA 级硬化 vs 内部审计信任。
2. 低优先级: 不引入对抗代码 (与价值观冲突), 仅作为审计边界输入。

---

## 3. 风险与权衡

| 风险 | 缓解 |
|------|------|
| 每日信息例行成为噪声堆积 (高信号<20%) | 四字段过滤 + 吸收判定表 (吸收/观察/威胁模型/市场信号) |
| C2/C3 晋升引发连串依赖晋升 | R-P100 cost 门槛 + 逐节点 manifest 校验 |
| NT-WORLD 补强工作量 | 限定 modlens 单能力强化 (R-P42), 不做多平台全面适配 |
| MARS bridge 调查可能发现深层设计缺陷 | 先证据分析 (branch_1104 已记录 observe 先行/GWT 激活接线修复), 再设计 |

## 4. 决策 (Decision)

**采纳 (第一批)**: Phase A (每日信息例行, 今日已启动) + Phase B (C2/C3 晋升管线, 最高杠杆)。
**采纳 (第二批)**: Phase C (NT-WORLD 补强, modlens 单点)。
**评估中**: Phase D (MARS bridge, 需先分析) ; Phase E (威胁模型, 仅 KB 输入)。

被拒: 为每日信息建独立 cron 系统 (拒绝 — 复用 `nt_mind_background_loop`); 平行视觉适配器 (拒绝 — R-P42, 强化 nt_io_multimodal_transform)。

## 5. 验收矩阵

| # | 验收标准 | 判定 |
|---|---------|------|
| A1 | `notes/daily-intel-*` 连续 3 天存在 | PASS/FAIL |
| A2 | `neotrix-capability list` 含 C2+ 节点 ≥2 | PASS/FAIL |
| A3 | `cargo check --lib` 0 errors | PASS/FAIL |
| A4 | MARS bridge_hits > 0 或明确根因文档 | PASS/FAIL |
| B1 | `nt_io_multimodal_transform` 真视觉后端 ≥1 (非 placeholder) | PASS/FAIL |

## 6. 后续动作 (Next Actions)

1. Phase A: 连续执行每日榜捕获 3 天 + 挂 background_loop handler 设计评估。
2. Phase B: 选 2 节点 (revertible_effects / decision_trail) 增集成测试 → `mature --target c2`。
3. Phase C: modlens 模式分析 → nt_io_multimodal_transform 真视觉后端设计 (1-3-1 决策简报)。
4. Phase D: MARS bridge 根因调查 (读 nt_core_self MARS 逻辑)。
5. 所有生产能力变更注册能力树 (R-P100)。