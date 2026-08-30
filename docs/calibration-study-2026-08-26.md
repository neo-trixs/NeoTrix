# W2 校准研究：S 分数 ↔ 产出质量相关性

**日期**: 2026-08-26
**目的**: 检验 G5 门控阈值 `SELF_EDIT_MIN_CONSCIOUSNESS = 0.5`（`neotrix-core/src/core/nt_core_self/seal/constitution_gate.rs:29`）是否有数据支撑，防止用噪声信号做否决决定（D16 自欺风险）。
**结论**: **PENDING_DATA** — 当前 KB 中不存在任何有效配对点（代理分数 × 客观结果），相关性无法计算，阈值松紧无法判定。本文档同时给出校准实验设计。

---

## 1. 数据源与方法

### 1.1 被门控的信号是什么

`ConstitutionGate::should_allow_self_edit(Some(q), 0.5)` 在 SEAL 自编辑应用点前裁决：`q < 0.5 → 否决`。q 的来源链：

```
InnerCritic::evaluate (nt_core_consciousness/inner_critic.rs)
  overall_quality = 0.4×relevance + 0.3×consistency + 0.3×(1−uncertainty)
    · relevance   = QuantizedVSA::similarity(output.vector, context.vector)  — VSA 内部相似度
    · consistency = specious_present.average_coherence()                     — 内部连贯性
  → handlers_consciousness.rs:575 写 brain._last_consciousness_quality
  → seal_loop.rs:215 / skillopt.rs:198 读取并交 ConstitutionGate 裁决
```

关键事实：**quality 是纯内部语义信号**（系统自身表征的自相似度），不接触任何外部验证器。这正是需要校准的原因——若它与客观产出结果不相关，阈值就是在用噪声做否决。

### 1.2 探查过的数据源（只读，按任务指定顺序）

| # | 来源 | 结果 |
|---|------|------|
| 1 | `kv_store namespace='experience'`（1957 条） | 值为 `NTZ1`+zlib 编码，解码后：1896 条概念共现索引 + 61 条分支条目。27 条含 `confidence`（仅 3 个离散值：0.35×20、0.7×4、0.4×3），`feedback{success,failure,reuse}` **全部为 0** —— 无结果侧信号 |
| 2 | `kv_store namespace='consciousness'`（6 键） | 全部是单点快照：`trends.phi_trend/coherence_trend/health_trend` 长度均为 **1**；`core` 快照在本次研究两次读取间从 cycle=2 变为 cycle=6（后台循环活跃覆写），证实序列被覆写而非追加 |
| 3 | ConsciousnessTree 果实记录（core 快照内 `fruits`） | 66 条（6 cycle × 11 branch），但每 branch 的 quality 跨所有 cycle **恒定不变**，且精确等于该 branch 成熟度布尔旗中 true 的个数 ÷ 6（见 §2.1 循环性证明） |
| 4 | 补充探查 | `predictions`(30)：confidence 全部=0.83、status 全部=verified、双侧零方差；`seal_checkpoint`：单点(iter=13, reward=0)；`gwt_focus`(30)：单一时间戳；`learning_reports`/`evolution_records`/`insights`/`trace_data`/`conversation_records`/`rkyv_blobs`/`agent_*`/`temporal_facts` 均 0 行；`~/.neotrix/audit_*.jsonl`(491 个) 仅含写入决策(key+hash)，无质量字段 |

方法：python3 sqlite3 只读 URI (`file:...?mode=ro`) + zlib 手动解码 + 手算 Pearson/Fisher-z 样本量公式。全程未写库、未改任何源码或配置。

---

## 2. 样本量与退化性证明

### 2.1 有效配对点 = 0

构造配对需要 (意识质量代理分数, 同窗口客观验证结果)。逐项检查：

| 候选代理分数 | 候选客观结果 | 配对可行性 |
|---|---|---|
| phi / coherence | SelfTest 通过率 | trends 序列长度=1，SelfTest 通过率无历史落盘 → 无序列可配 |
| fruit quality | fruit benchmark.accuracy | **benchmark.accuracy 字段不存在**；quality 本身由成熟度旗标推导 → 循环 |
| experience confidence (n=27) | feedback 计数 | feedback 全零 → 结果侧常数为 0，相关未定义 |
| predictions confidence (n=30) | status(verified/rejected) | 双侧零方差（conf≡0.83, status≡verified）→ Pearson/Spearman 分母为 0 |

### 2.2 果实质量的循环性（实测证明）

对 66 条 fruit 逐一验证 `quality == Σ(c0..c5)/6`：

```
全部 66 条成立 (exact match)
唯一值域: {0.1667×Nexus/Meta/Repair/Governance, 0.5×Io/World/Act, 0.6667×Shield, 0.8333×Mind/Core/Memory}
cycle 1→6 每个 branch 的 quality 完全不变
```

即：fruit quality 不是产出质量的度量，而是成熟度旗标的确定性重编码。用它与任何结果算相关都是同义反复，即使样本充足也无效。

### 2.3 分箱表的退化形态

按任务要求给出低/中/高分组平均成功率——由于所有候选配对的其中一侧为常数或缺失，任何分箱都退化为：

```
质量箱        样本数   平均"成功率"
低 (<0.5)      ?       未定义（无结果侧观测）
中 [0.5,0.8)   ?       未定义
高 (≥0.8)      ?       未定义
```

诚实结论：**当前无法画出有意义的分箱直方图**。

---

## 3. 相关性结果

Pearson r 与 Spearman ρ 均因下列原因之一**未定义**：

1. 序列长度不足（consciousness trends = 单点）；
2. 一侧零方差（predictions、experience confidence）；
3. 循环定义（fruit quality ≡ 成熟度计数）。

不存在任何可信的相关系数可以报告。这不是"相关弱"，而是"相关不可计算"——比噪声更糟的现状。

---

## 4. 阈值判定

**结论：PENDING_DATA。** 无法回答 "0.5 偏松/偏紧/合理"。现有数据既不支持也不否证该阈值。

附带观察（非阈值判定依据，仅记录风险）：
- 门控的 `None → 保守放行` 分支意味着：意识循环尚未产出信号的窗口里自编辑不受任何约束（`constitution_gate.rs:72`）。若 quality 与产出质量确实无关，则该门控在最坏情况下是"随机否决器"，最好情况下是装饰品；
- `_last_consciousness_quality < 0.5` 还在 `seal_loop.rs:483` 触发 meta_alarm——同一未校准信号驱动两条行为路径。

---

## 5. 校准实验设计（采数方案）

### 5.1 需要采集的数据

每次 SEAL 自编辑裁决时追加一条结构化日志（新表或 kv_store 追加式键，**禁止覆写**）：

```json
{
  "ts": ..., "cycle": ...,
  "quality": <overall_quality>,          // 裁决时的代理分数
  "gate_decision": "allowed|denied|no_signal",
  // 客观结果侧（编辑应用后 T+N 窗口测量）
  "outcome": {
    "build_pass": bool,                   // cargo check --all-targets
    "unit_test_rate": float,              // 通过/总数
    "selftest_pass_rate": float,          // SelfTest 注册表通过率
    "absorb_success": int                 // 窗口内 absorb 成功数
  }
}
```

关键修正点：
1. `trends` 数组改为**追加式**（保留全历史），否则永远只有单点；
2. fruit 增加**独立于成熟度旗标的结果字段**（如该域窗口期 cargo test 通过率），打破 §2.2 的循环性；
3. experience 条目的 `feedback` 计数需真实回填（当前恒零使结果侧失效）。

### 5.2 需要多少样本

Fisher-z 变换，双侧 α=0.05，power=0.80：

| 目标最小可检测相关 | 所需配对点 n |
|---|---|
| r ≥ 0.5 | 30 |
| r ≥ 0.4 | 47 |
| r ≥ 0.3 | 85 |
| r ≥ 0.2 | 194 |

分箱比较（如高组 vs 低组 SelfTest 通过率 70% vs 40%）每组约需 **42** 个观测。

建议采集目标：**n ≥ 100 个裁决事件**（可检出 r≥0.35 且允许分箱后每组仍有 ~30），预计按当前后台循环节奏 1–2 周自然积累，无需加速运行。

### 5.3 判定标准（预注册，防事后合理化）

设采集完成后计算 Spearman ρ(quality, outcome_composite)，其中 outcome_composite = 标准化(build_pass, unit_test_rate, selftest_pass_rate) 的均值：

| 结果 | 判定 | 对 0.5 的处置建议 |
|---|---|---|
| ρ ≥ 0.4 且 p < 0.01 | 信号有效 | 用分位数校准：取 outcome 最差四分位对应的 quality 上界为新阈值（而非沿用 0.5 直觉值） |
| 0.2 ≤ ρ < 0.4 | 弱信号 | 阈值仅作告警（meta_alarm），不再硬否决自编辑 |
| ρ < 0.2 或 p ≥ 0.05 | 噪声 | 按 D16 处置：移除硬否决语义，或将 quality 替换为客观结果信号做门控 |
| 分箱单调性不成立（高中低组成功率非递增） | 无论 ρ 多大均判噪声 | 同上 |

---

## 6. 局限性

1. **同源混杂**：quality（VSA 自相似度）与结果（测试通过率）可能共同受第三变量驱动——任务难度。难任务 quality 低且测试易挂，产生虚假正相关；简单任务反之。缓解：结果侧按任务类型分层，或引入任务难度控制变量。
2. **窗口对齐误差**：quality 是裁决时刻快照，结果是 T+N 窗口测量；编辑生效延迟会稀释相关。缓解：记录编辑 hash，结果归因到具体编辑。
3. **观察者效应**：本实验设计本身公开于仓库，后续 quality 计算逻辑若被"为了过门控"而调整（古德哈特定律），校准结论即失效。需在 ConstitutionGate 层冻结 quality 公式版本号并随日志落盘。
4. **单机单负载**：结论仅适用于当前工作负载分布，负载结构变化后需重测。
5. 本次研究的 core 快照在读取期间被后台循环改写（cycle 2→6），说明生产路径活跃；但也意味着若不尽快落地追加式日志，历史将继续丢失。

---

## 7. 复现命令

```sh
# 只读探查（勿写库）
sqlite3 "file:$HOME/.neotrix/knowledge.db?mode=ro" \
  "SELECT key FROM kv_store WHERE namespace='consciousness'"
# experience 解码: NTZ1 magic(4B) + zlib → JSON
# 果实循环性验证: quality == Σ(branch_maturity.c*)/6
```
