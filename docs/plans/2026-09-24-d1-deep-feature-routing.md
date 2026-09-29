# D1 深特征路由 — 设计（2026-09-24，只设计不动码）

> 上游：`docs/plans/2026-09-24-human-inspired-refinement.md` §D1（M1+M5，权重 0.3）。
> 本文件只设计不动码；实现等 ftv4 跑完内存窗口。

## 1. 现状定点（字面路由 3 处，均已读）

| # | 位置 | 字面机制 | 病症 |
|---|------|----------|------|
| L3-1 | `neotrix-core/src/skill_registry.rs:33` `match_trigger` | 整句 query 作单个 trigger，`st.contains(t)` 子串命中，取**首个** | 长句恒 miss、短词误 hit、无排序 |
| L3-2 | `neotrix-core/src/skill_loader.rs:423` `compute_score` | name==100 / name含50 / desc含20 / tag×10 / trigger×15 | 全字面分，同义零分 |
| L2 | `neotrix-core/src/l1_action/nt_infra_agent_card.rs:353` `match_capabilities` | capability name **精确相等**交集计数；调用方喂的是全文 query（`skill_registry.rs:126`） | 全文 vs 短名精确比对 → 恒 0 → 直落 DirectLlm |

## 2. 深特征表（MECE，纯规则零模型）

```text
意图动词（8）：search{找/查/搜/search} compare{比/对比/compare} synthesize{写/总结/生成/write}
  debug{修/报错/fix} execute{跑/执行/run} navigate{打开/前往/open} extract{提取/解析/parse}
  measure{测/评估/eval}
约束（6）：time_limit{限时/快} cost_budget{便宜/免费} lang{中文/英文} modality{图/音频/代码}
  offline{离线/本地} approval{审批/确认}
资源/域（7）：repo{仓库/代码} paper{论文} web{网页/搜索} kb{知识库} colab{colab}
  browser{浏览器} shell{命令}
```

抽取 = 查表命中（中英同义词）→ `DeepFeatures{verbs[], constraints[], domains[]}`；
无命中 → 空特征，路由回退现行字面逻辑（fail-open，不降级）。

## 3. 接入设计

- 新模块 `neotrix-core/src/nt_route_features.rs`（`nt_` 前缀 ✓）：纯函数
  `extract_deep_features(query) -> DeepFeatures` + `score_by_features(skill, feats) -> f64`，
  自带单测（同义 query 对照组）。
- 接入点（1 处，最小改动）：`route_entity_aware`（`skill_registry.rs:103`）Layer-3 之前加预遍——
  特征非空则用特征匹配排序取首（替代`match_trigger`整句contains），
  特征空则走原逻辑；Layer-2 的 `wanted` 由“全文单串”换成“tags + 特征域词”。
- `compute_score` 加特征加权项（verbs 命中 +30，domain 命中 +20），权重与现行字面分同量级，
  避免一刀切。

## 4. 验证（D5 同题命中率涨）

- 回归：D5 C01–C05 全过（行号漂移允许，语义不变）。
- 对照组 10 组同义 query（例：“修 flaky 单测” vs “fix flaky test” 应同路由）：
  字面路由命中率（基线，预期 ~3/10）vs 深特征路由（目标 ≥7/10）。
- 验收：`cargo xl` + `cargo test -j1 -p neotrix --lib -- nt_route_features` 全绿。

## 附：同义对照组 10 组（2026-09-24 编写，期望同路由）

| # | A（中文） | B（英文/换述） | 期望特征 |
|---|-----------|---------------|----------|
| 1 | 修 flaky 单测 | fix flaky test | debug+repo |
| 2 | 找 chromiumoxide 的 user_data_dir 用法 | how to set user data dir in chromiumoxide | search+repo |
| 3 | 对比 LoRA 与全量微调显存 | compare LoRA vs full finetune memory | compare+paper |
| 4 | 跑 D5 验收集 | run gaia mini eval set | execute+eval |
| 5 | 总结 ftv4 训练曲线 | summarize ftv4 training loss curve | synthesize+logs |
| 6 | 打开 colab 探针报告 | open the colab probe log | navigate+file |
| 7 | 评估 sidecar 延迟 | measure sidecar latency | measure+sidecar |
| 8 | 提取 smelt 的 800 条新卡主题 | extract topics of the 800 new smelt cards | extract+data |
| 9 | 写 D1 深特征路由设计 | write the D1 deep-feature routing design | synthesize+docs |
| 10 | 查训练 42994 步之后有没有掉线 | check whether training dropped after step 42994 | search+logs |

## 5. 工时序

实现（~1h，需内存窗口）→ 对照组评测（20min）→ 合入（邻窗协调，避免 §4 式整段搬家冲突）。
