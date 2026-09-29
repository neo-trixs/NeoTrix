# D4 语义好奇心 — 设计（2026-09-24，只设计不动码）

> 上游：`docs/plans/2026-09-24-human-inspired-refinement.md` §D4（M4，权重 0.15）。
> 现状只有"新颖"（覆盖稀疏度），没有"有用"；升级即补上有用性判断。

## 1. 现状定点（均已读）

| 件 | 位置 | 机制 |
|---|------|------|
| 驱动 | `nt_mind/consciousness/curiosity_drive.rs:55` `ingest_gap_reports` | avg_sparsity 三档定 level（0.2/0.4/0.7）；intensity=(gap+sparsity)/2；search terms 是模板串（`"Time research frontier"`，质量低） |
| 消费 | `handlers_core.rs:333` `handle_curiosity` | top-2 queries → Wikipedia evolve + DialogueAbsorbBridge（吸收闭环已有，缺的是"问什么值得问"） |
| ICM 候选 | `nt_crystal_core/nt_predict_loop.rs`（`surprise()`/`observe()`/`domain_uncertainty()`，FEP 预测误差→爬取优先级） | ✅ 现成 substrate，直接复用当预测误差项 |
| 裁判候选 | sidecar engine = encoder 式打分（`encode_paths`，255 candidates，无 generate 路） | ⚠️ Qwen3 权重在盘（`models/qwen3-06b`），但 sidecar 无生成端点；VLM 本地零资源 |

## 2. 三级火箭（按成本排序）

- **L1 规则裁判（先行，零模型零内存）**：`interestingness = sparsity × P(absorb|domain)`，
  其中 P(absorb|domain) 取自该域历史探索的 absorbed/score_delta 均值（`[bg-research]` 日志
  + KB 可算）。"有用"第一次有数据定义，不玄学。
- **L2 ICM 误差（复用）**：`NtPredictLoop.surprise()` 接入 curiosity intensity，
  `domain_uncertainty` 高的域加权。纯 Rust 内调用，零新增依赖。
- **L3 LLM 裁判（排期最后）**：sidecar 加 `/api/judge` 生成端点（engine 需补 generate 路，
  另起则 ~1.2GB 与现 sidecar 双占）。SENSEI 式 VLM 语义打分是目标形态，
  但等 L1/L2 跑出数、内存窗开了再做。

合成公式：`score = 0.5·sparsity + 0.3·pred_error + 0.2·judge`（L1 先行时 judge=历史吸收率代理；
权重跑出数后回校准）。judge < 阈值 → 只记录不触发爬取（新颖且有用才动）。

## 3. NT-PLAY 游乐场接线

top signals 翻成自对弈开局（`SelfPlayLoop` episodes_per_iteration=8 轻量）+ novelty bonus
进 reward → 探索→对弈→轨迹→D3 蒸馏，全链闭环（D4 的产出是 D3 的 Teacher-C 原料）。

## 4. 验证

有用探索占比 = absorbed>0 的探索 / 总探索，周环比涨（与 D2 周报同口径，可并表）。
模板 search terms 质量问题顺带修：terms 改由 gap 维度 + 域历史高吸收词拼出。

## 5. 工时序

L1（1h，纯 Rust）→ L2 接入（30min）→ 跑两周数 → L3 立项。
