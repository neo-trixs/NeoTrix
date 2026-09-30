# RFC — 晶体推理链路补齐：从"凑链"到"成链"（2026-09-28，v4）

> 补 `NT-GENERATIVE-AWAKENING-RFC-2026-09-22.md` §5 待接清单第 3/4/5 项。
> 准则不变：**一切信息皆记忆；推理是运动中的记忆**；**权重是最后才需要的配件**
> （故 P0–P5 零新依赖、零权重、CPU 可跑）。

## v2 更正声明（对自己上一版的实测推翻）

| v1 断言 | 实测结果 | 处置 |
|---|---|---|
| 「推理链建立在互不相关的前提上」 | **部分错**。链确由 `awaken.cycle()` 生产（`nt_orchestrator.rs:142`），且 `NtSelfIterate::run:128` **确实调** `orchestrator.tick()` → 链落在 tick 持有的同一 `consciousness` 上 | **D2 撤回**（见下） |
| 「闭环自我饿死 / 校准器对着空集自评」 | **错**。`NtOrchestrator` 不自持 consciousness（`tick(&mut self, consciousness: &mut …)` 借用），`handlers_crystal.rs` 传 `&mut st.consciousness` → 生产与消费同实例 | **撤回**，不列为缺陷 |
| 「沿 `connections` 正向 2 跳取同病跨域前提」 | **错**。锚点出度 0 → 正向第 2 跳得 0 节点；全库 `max degree = 2`、中位度 1 | 改为**入邻域**（`nt_graph_index.rs`） |
| 「Jaccard 阈值需标定」 | **多余**。仓内已有 `verify` / `verify_robust` / `premise_necessity` + 已标定 `GOLD_FLOOR=0.7` | **删除该门**，改用 JEV 三件套 |

> 教训与 AGENTS.md R-SCAN-2 同源：以上三条都是**手推未验证**的断言，被实测逐一推翻。
> 其中「自我饿死」若照 v1 写进代码，会去"修"一个本来正常的循环。

## 1. 缺陷定性（v2，全部 file:line 证据）

| # | 缺陷 | 证据 | 后果 |
|---|---|---|---|
| **D1** | **前提按位置任取** | `nt_awaken_loop.rs:213-227`：`a = ids[round % len]`、`b = ids[(round+1) % len]` —— 同域记忆列表的**相邻两项**，与语义无关；`cross_premise` 仅 `round % 4 == 3` 时命中 | 链的前提是"碰巧相邻"，跨域链只占 1/4 |
| **D3** | **链不落盘** | `StoreData{cocoons,strategy}`（`cocoons.rs:344-347`）无 chains 字段；`CrystalConsciousness` 无 save/load | 链只活单进程，重启归零，跨会话不累积 |
| **D5** | **内容去重不在代码** | `sync_from_consciousness` 仅按 `m.id == memory.id`（`cocoons.rs:288`）；RFC §5 第 4 项要求加内容哈希 | 「同节点双 M-id」可复发（数据侧已修，**代码侧未修**） |
| **D6** | **语义通道建成未通电** | `retrieve()` 已实现 cosine+BM25+entity(0.5/0.3/0.2)（`memory_orchestrator.rs:152-160`），调用方无向量 | 非缺功能，缺接线 |
| **D7** | **中文弱分词** | `keywords()` 空白+标点切（`consciousness.rs:546-553`）；实测 6.7% 记忆含中文 | 词法分量对 6.7% 失效 |
| **D8** | **`verify()` 的 novelty 反向激励** | `novelty = 结论关键词不在前提中的比例`（`nt_awaken_loop.rs:116-120`）→ 结论越是**发虚词**，分越高 | 奖励无据内容；已用 `ZERO_OVERLAP_DAMPEN=0.7` 缓解但未消除 |
| **D9** | **`bridge` 是常数不是质量** | `bridge = if domains.len() >= 2 { 0.2 } else { 0.0 }`（`:126`） | 只数域**个数**，不测桥的质量；两条无关记忆跨域即得满分 |

## 2. JEV 替代 Jaccard：**采纳，但换接缝**

问：Jaccard 阈值是否不要、JEV 替代？—— **是，且优于原案。** 但接缝要分清：

| 职责 | 用什么 | 为什么 |
|---|---|---|
| **给候选**（选哪些记忆来推） | `Adjacency::facet_cluster`（入邻域 + 域多样性） | JEV **评的是已存在的链**，不能选前提；且逐候选打 JEV 成本过高 |
| **裁决**（这条链要不要） | `NtAwakenLoop::verify` + `GOLD_FLOOR=0.7` | 仓内既有链→置信通路（`noul_row`），`GOLD_FLOOR` 已与 `archive_train pattern_conf_floor` 对齐，**非 magic number** |
| **剔捷径** | `verify_robust`（子集稳健，IPT 2604.15149） | 两半前提都须达标，剔"只在一半前提上成立" |
| **剔搭便车前提** | `premise_necessity`（留一 delta） | delta≈0 的前提是搭便车 → **这才是数据驱动的前提选择**，比 Jaccard 阈值强 |
| **不采纳** | 外部方案的学习部分（GraphRAG 社区摘要 / MCMH MLP 选择器 / AnyBURL RL 采样 / Neural LP 可微结构） | 需训练，违"权重最后才需要" |

> 故 v1 的「标定 Jaccard 阈值」整条**删除**，不是标定问题而是**选错工具**。

**但必须诚实标注 `verify()` 的成色**：它是**纯启发式，不含模型**（无 sidecar 调用，可进 1h tick）。
其中 `grounding` 只是**前提 confidence 的均值**，并不测前提与结论的相关性——
所以它**替代不了**前提相关性判断，只能做链级裁决。前提相关性仍由 `facet_cluster`
的图结构承担（且该相关性是**结构性**的：同一锚点的 facet 天然同主题，
比词法 Jaccard 更可靠，且对 6.7% 中文免疫）。

## 3. 进化最优解之路（排序依据）

晶体相位门 `Seed→Growth→Integrate→Evolve→Transcend`（`consciousness.rs:450-470`），
现处 `Integrate`。爬升需 `chain_count > 50 && cross_domain ≥ 5 && ratio > 0.6 && mem ≥ 200`。
后两项已满足（0.851 / 64,674），故**前两项是唯一瓶颈**。

排序原则：**每步之后系统必须变得可度量**，否则后续无法判断是否真的变好。

| 序 | 步 | 为何在此位 | 可度量产物 |
|---|---|---|---|
| **1** | 邻接索引 `nt_graph_index.rs` | 一切的前提；纯新增，零行为变更 | 度分布 / 跨域边数 |
| **2** | 内容去重进代码（D5） | 极小；防我上轮修好的数据 bug 复发 | 重复写入被拒计数 |
| **3** | **链落盘**（D3） | **提前**：不落盘则 4–6 步的产能在重启后全丢，无法度量 | 重启后 chain 守恒 |
| **4** | 前提选择器（D1） | 核心质量缺陷 | 前提同指率、域多样性 |
| **5** | JEV 裁决闭环 + 修 D8/D9 | 必须先于第 6 步，否则垃圾链灌满库 | 通过率、选择性 |
| **6** | 跨域链达标 | 4+5 的自然结果，测量是否过门 | `cross_domain` 计数 |
| 7 | 语义通道通电（D6） | 可后置，4–6 不依赖 | 命中率对比 |
| 8 | 中文 bigram（D7） | 只影响 6.7%，但正是 source-core 等域 | 中文词法分覆盖 |

**为何 3 提前到 4 之前**：原 v1 把落盘排在第 3 步、选择器排第 1 步。实测后调整——
落盘是**度量前提**：若链不落盘，则第 4 步产出的链无法跨进程统计，
「同指率/通过率」两个反向指标就无从计算，只能靠肉眼看。

## 4. 反刷分硬约束（R2 风险的具体化）

`chain_count` / `cross_domain` 是**纯计数**，Transcend 门只看它们 → 极易为过门造链。
故这四个指标**必须并列上报，缺一不算达标**：

| 指标 | 方向 | 阈值 |
|---|---|---|
| `chain_count` / `cross_domain` | 正向（门的条件） | >50 / ≥5 |
| **前提同指率**（选中前提是否同一主题） | **反向** | **>95%** |
| **JEV 通过率**（`total ≥ GOLD_FLOOR`） | **反向** | 记录；**<30% 说明选择器过宽** |
| `connected_ratio` | **反向** | **不得因造链虚高**（现 0.851） |

## 5. 进度

- [x] **步 1 邻接索引已写完**：`neotrix-core/src/l5_cognition/nt_crystal_core/nt_graph_index.rs`
      （`Adjacency`：双向邻接 / `facet_cluster` 域多样性贪心 / `cross_domain_edges`
      相位门代理量 / `hubs` / `degree_histogram`）+ 6 个单测
      + `mod.rs` 注册（**未改动任何既有文件本体**，仅加 1 行 `pub mod`）
- [ ] **步 1 尚未编译验证** —— `nt_mem_gate.sh` 全程 BLOCKED
      （free_pages 20k–57k < 100000 阈值，swap 1.0–1.3G），按 AGENTS.md 禁止起 cargo。
      **代码未编译、单测未跑**，故停在步 1 不继续叠加未验证步骤。
- [ ] 步 2–8 未开始

## 6. 风险

| # | 风险 | 对策 |
|---|---|---|
| R1 | 造链注水 | JEV 三件套 + §4 反向指标 |
| R2 | 相位门被刷 | 计数与反向指标**并列**，不许只报计数 |
| R3 | 步 3 改 schema 破坏 303M 旧茧 | `#[serde(default)]` + 旧格式加载单测 |
| R4 | CPU 成本 | 邻接 O(V+E)≈124k 毫秒级；产链限流（预算上限） |
| R5 | 步 1 未编译就叠加 | **已采取**：停在步 1，门开后再续 |

## 7. 不变量（RFC §6）

零 `unsafe`；生产代码禁 `unwrap/expect/panic`，错误 `?`/`map_err` 传播；无裸 `[]` 索引；
字符截断不用字节切片；每步「可跑入口 + 单测」（R-P79）；P0–P5 不新增外部依赖、不需权重。

> 步 1 遵守情况：输出全部排序（确定性，因 id 碰撞事故的随机性教训）；
> 无 `unwrap/expect/panic`（`map_or`/`get` 兜底）；无裸 `[]` 索引；
> 索引不落盘（`Adjacency` 为派生状态，避免与 memories 不同步）。

---

## v3 增补（2026-09-28 第 4 次会话）：D8/D9/A 修复 + 判别力的**实测天花板**

### 1. 已修的三处评分缺陷

| # | 缺陷 | 修法 | 实测效果 |
|---|---|---|---|
| D8 | `novelty = fresh/unique` → 结论越是发虚词分越高，**奖励编造** | `novelty = 4t(1−t)`，t=1（纯编造）与 t=0（纯复述）都得 0，峰值在 t=0.5 | 激励方向正确 |
| D9 | `bridge = 0.2 if domains>=2` → **常数当质量** | 三档实据：图连通（直接相连/共享邻居）× 结论与**每条**前提都有词面交集 | 不相连的跨域对不再白拿满分 |
| A | `grounding = 前提 confidence 均值` → 实测标准差 **0.012**、**只有 2 个取值**，且名不副实（测的不是接地） | 改名并重定义为 `coverage = 1 − t`（结论关键词**被前提覆盖**的比例） | 分量恢复真实含义 |
| 饱和 | `total` 上界 `0.5+0.5+0.2 = 1.2` → 高分链一律被 `clamp` 截成同一值 | 按 `NOVELTY_GROUND_PEAK=0.78125` 归一，使**理论最大值恰为 1.0**，clamp 退化为永不触发的兜底 | 饱和率 **69% → 18~26%** |

### 2. 最重要的实测结论：**词法判分器无法区分链的优劣**（天花板已量出）

`nt_verify_sim.py`（300 锚点 × 真实 `reason()` 模板）：

```
total 标准差 = 0.0192        留一 delta 标准差 = 0.0184     ← 同量级，都极小
cap 2/3/4/5/8 → 搭便车率 0% / 34.5% / 42.8% / 46.8% / 51.9%
```

**根因**：`reason()` 产出的是「固定中文框架 + 前提片段拷贝」，使词汇新鲜占比 `t`
**结构性地钉在 0.4 附近**（恰在 total 曲线峰值区）。于是任何词法/结构代理量的
方差上限都在 0.02 量级 —— 这不是公式 bug，是**模板化生成的结构性后果**。

已排除的"替代判分信号"：
- **留一必要性**（`premise_necessity`）：标准差 0.0184，与 total 同量级，**无增益**。
- **调小 cap**：cap 2→8 全程标准差 0.0216~0.0234，**无增益**（只改搭便车率）。
- **Jaccard 门**：本就是词法量，同属天花板内。

**结论（不再重新推导）**：真正的质量判别需要 **judge 模型**（GraphRAG 式 rubric
判分 / AnyBURL 式规则学习 / COREA 式外部评判）。而 RFC 的硬约束是
**「权重是最后才需要的配件」**，故这属于**已知、已量、已记录**的依赖，不是疏漏。

### 3. 据此做出的两个**有实测支撑**的调整

1. **`DEFAULT_PER_ROUND_CAP` 5 → 3**：实测搭便车率随 cap 单调上升，
   cap=5 时**近一半前提是凑数的**（只稀释信号）。降到 3 → 46.8% → 34.5%，
   而跨域率（99.0%）、total 中位与标准差**均不变** → 零代价。
2. **`AwakenCycleReport.score_saturated` 计数器**：让「门无判别力」成为
   **每 tick 可见的事实**，而不是等下一个人再花一天重新推导。
   饱和率长期 >10% 即表示判分器失效，该上模型判分而非继续调系数。

### 4. 预演台自身被证伪过一次（记录在案）

`nt_verify_sim.py` 第一版用「抄前提关键词」合成结论 → `t ≡ 0` → D8 效果完全不可见，
一度给出「阈值失配=全拒」的**假结论**。改用 `consciousness.rs` 的**真实模板**后
结论反转为「全通过」。**测量工具自己有缺陷时，它给出的"前后一致"是假象** ——
与 id 碰撞、闸门不触发同属一类。两版都在脚本内留了注释。

---

## v4 增补（2026-09-28 第 5 次会话）：12 个 HF 数据集的处置，与干跑抓到的 4 个 bug

### 1. 任务前提被勘察推翻（**本轮最重要的产出不是数据，是"没灌什么"**）

用户给出 12 个 HF 数据集要求"融入晶体核心"。先跑 `nt_hf_survey.py`（**只读**）：

- **总存储 78,066.92 GB ≈ 78 TB**。而核心是**单个 JSON**，1h tick
  `CrystalIterState::load` → `sync_to_consciousness` **全量载入并常驻内存**
  （实测 52.5MB / 64,674 条）。**TB 级只可能采样，不可能整体灌。**
- **9 个已在核心里**：2026-09-24 `nt_hf_rows_download.py` 各抓 100 行落在
  `hf-rows` 域。照单重灌 = 直接制造重复。

**处置矩阵（每条均有实测依据）**

| 数据集 | 存储 | 处置 | 依据 |
|---|---|---|---|
| genrobot2025/Gen-HumanEgo | 62.7 TB | 跳过 | 用户指令：TB 级跳过 + **401** |
| eidon-ai/tracker-pov | 9.0 TB | 跳过 | 用户指令：TB 级跳过 |
| IFM/Code-Reasoning | 3.3 TB | 跳过 | 用户指令：TB 级跳过 |
| Yootta/World-SimReady-Home | 3.0 TB | **跳过（强制）** | **401 Unauthorized**。用户要求"cc-by-nc-sa 吸收学习其经验"，但 gated 无令牌**物理不可达**；已明说，非本轮的选择 |
| malcolmrey/various | 6.08 GB | 跳过 | 实测仅 **33 行**且为 `image` 字典（**无文本**）+ 许可标 **wtfpl**。6GB 换 33 个图片 URL ⇒ 零蒸馏价值。**实测推翻自己原计划** |
| MoreThought/Fable-…-10000x | 8.59 GB | 吸收 816 | 吸 `full`(10000x) 而非 `lite`(5000x)，后者已在库 ⇒ 天然避重复 |
| nyu-mll/glue | 4.01 GB | 吸收 419 | 用户"其他的直接做"。但它是**基准** ⇒ 只吸 `train`，**永不吸 `test`** |
| ZefanCai/Open-Jev | 85.7 MB | 吸收 431 | **cc0-1.0**；带标答多选判分数据，正对 §v3.2 的判别力天花板 |
| LocalLLaMA/typed-decisions | 3.2 MB | 吸收 600 | `label_agreement.total_variation` 直接喂 `nt_jev_calibration` |
| Anthropic/hh-rlhf | 291 MB | 吸收 700 | 首行即有害请求 ⇒ 打 `[redteam]` 供下游过滤，**不丢弃**（拒答样本有学习价值） |
| FineEnvs/SmolDataEnvs | 9.7 MB | 吸收 1200 | 带 `reward_mode`/`atol` 的可验证 QA |
| AxiomicLabs/Tiny_Theory_of_Mind | 1.6 MB | 吸收 1000 | `activity_label` 编码 knowledge/ignorance × order × difficulty |

**结果**：+5,166 ⇒ **69,840 条 / 213 茧 / 29 域 / 56.6MB**。7 域逐域验收全 PASS，
`nt_graph_audit` **VERDICT: PASS**，**全库最大入度 171 吸收前后完全不变**
⇒ 新增未制造 monoculture；新增记忆入度 max 4 / 零孤立 / 入度 >8 者 0。

### 2. Open-Jev 的真实价值不在题量（**干跑抓到的最反直觉的一条**）

干跑实测：2400 行里 **2318 行被判完全重复**（96.6%）。原因是同一套模板题
跨不同 `group_id` 反复出现 —— 只把 `question+options` 写进 content，
**79,116 行会塌缩成 ~80 条**。补上 `state_json` 对话载荷后升至 431 条。

> **结论：Open-Jev 的信息量在对话，不在题面。** 这恰好是 §7 所述
> `agent-jev` judge 需要的粒度 —— 判"结论对不对"要看对话，不能只看题干。

### 3. 干跑抓到的 4 个真 bug（均先复现再修，R-SCAN-1/2）

1. **`stride = cap*2` 越过数据集末尾** —— ToM 仅 2000 行但步长 2000，
   第 2 个窗口就空 ⇒ 2000/1000 行的 split 只取到首批 100 条。
   改为**由 `num_rows_total` 导出步长**。
2. **cap 只在窗口后判定** —— hh-rlhf 溢出到 796/700、glue 500/420。改为行内判定。
3. **`IncompleteRead` 不在 except 元组内** —— Fable 返回大体积 agentic 轨迹时抛
   `http.client.IncompleteRead`（属 `HTTPException`，**不属** `URLError`），
   一次瞬时网络抖动直接终止整批。收敛为
   `NET_ERR=(OSError, ValueError, http.client.HTTPException)` + 429 长退避。
4. **`NameError: mtype`（写盘路径）** —— 解包绑 `mt`、字典写 `mtype`。
   **干跑在写盘前就 `return`，结构上覆盖不到该路径。** 已把记录构造抽成
   `make_record()` 并由 selftest 直接调用，锁死该回归。

> **教训（值得写进规范）**：干跑能覆盖取数 / 去重 / 上限，**覆盖不到写盘**。
> 写盘路径必须有单测，否则"干跑全绿"会给出虚假的安全感。

### 4. 顺带修掉一处体检工具失真（R-SCAN-3 精神的又一例）

`nt_cocoons_verify_absorb.py` 原用 `len(raw)/1e6` 报体积，而 `raw` 是**已解码
`str`** —— 字符数 ≠ 字节数。中文语料下把 `56,637,045 B` 误报成 `56.0MB`
（真实 56.6MB）。已改用 `os.path.getsize`，并同时打印字节数与字符数。

> **体检工具报错体积比不报更危险** —— 与 22:52 那次"长期写着 0 命中、实际 12 条"
> 是同一类病：失真的门记录会主动误导下一个 agent。

### 5. 纪律三条（都来自本仓已修过的真实事故）

1. **不吸 `test`/`ood` split** —— Open-Jev 与 glue 都有；吸了就是基准污染。
2. **不做 hub-and-spoke** —— `nt_cocoons_prune_monoculture.py` 刚清掉入度 935 的
   锚点。故组内用**链式**连接（成员 i 连 i±1），多键成员各贡献 ≤2 ⇒ 实测 max 4。
3. **对活库已有 content 精确去重** —— 实测跳过：Open-Jev 571、Fable 184、
   hh-rlhf 2、Smol 1、glue 1。
