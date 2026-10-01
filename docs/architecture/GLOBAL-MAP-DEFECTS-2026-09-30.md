# 全域 map · 每链路缺陷 · 进化路线核心建议（2026-09-30）

> 交付对应七问：外部资料 → 底层模型逆向推理 → 审计等基础能力缺失点 →
> 全域 map 每链路缺陷 → 进化路线核心建议 → 原子级拆解 → 复现对方产品。
> 每条判断标注【实测】/【外部】/【推理】。**未实证的不写成结论**（今天已因此更正过两次）。
> 配套：`CAPABILITY-GAP-2026-09-30.md`（缺口清单）· `CALLGRAPH-FEASIBILITY-2026-09-30.md`（调用图实测）

---

## 第一部分 · 外部资料（检索所得，按「与本仓相关性」排序）

### 1.1 复现他方产品：RepoZero 是最直接对应的

`arXiv 2605.07122`《RepoZero: Evaluating Code Repository Generation》

- **核心设计动作：把「从零生成」重述为「仓库复现」** ——
  只给 **API 规格** + 少量示例用例，agent 必须重实现整个仓库，
  使其**可观测行为与原实现一致**
- 由此获得**严格黑盒验证**：原仓库充当**确定性 oracle**，逐字符串比对输出
- **三个必须记住的数字**：
  1. **~40% 可执行的代码仍然通不过输出等价** ⇒
     **「能跑通」远不等于「行为正确」**，只按执行成功打分的基准**严重高估**正确性
  2. 最强 agent 通过率仅 **30%–55%**
  3. 失败模式排序：**非可执行代码 > 漏掉白盒用例 > 黑盒输出不匹配**；运行时错误反而罕见
- 反泄漏设计：禁外部依赖、禁跨语言代理（如 Py2JS agent 不得 shell out 到 Python）
  ⇒ **强制真实现，禁止 API 桥接**

### 1.2 原子级拆解：RePro 的「fingerprint」就是机制

`arXiv 2508.16671`《RePro: Reflective Paper-to-Code Reproduction Enabled by Fine-Grained Verification》

- 把论文/规格拆成 **fingerprint = 一组可验证的二值判据（binary criteria）**，
  三条性质：**comprehensive（完备）/ accurate（准确）/ atomic（原子）**
- 抽取管线：抽 guide → 落到具体句子 → 标准化 → 过滤
- 使用：verifier 逐条判 **pass/fail + 差异说明** → planner 汇总成修订计划 →
  editor 按计划做**定点最小修改**
- **迭代 4 次最优**（第 5 次反而下降 —— 收益递减/过拟合）
- 失败分析：**57.9% 的正确修订属于「数学写对」，42.1% 属于「核心算法逻辑」**

### 1.3 规格为什么会丢：ReproAgent 的诊断

`arXiv 2608.24291`《ReproAgent: Contract-Guided Paper-to-Code Reproduction》（EMNLP 2026 Findings）

- **核心诊断：规格是分裂的**
  1. **显式内容**（算法、指标、制品）**在长 agent 轨迹中逐渐丢失**
  2. **隐式细节**（框架默认值、来自相关工作的惯例）**根本不在原文里**
- 解法：**持久实现契约（persistent implementation contract）+ 双通道**
  - `implementation-requirement` 通道：原文片段 → **代码义务（obligations）**
  - `reference-evidence` 通道：从**相关仓库**检索内容与结构证据
  - 契约绑定到 **work package**，再投影为**文件级契约**，跨生成与修复两阶段消费

### 1.4 黑盒 agent 产品的复现：AgentXRay

`arXiv 2602.05353`《Agentic Workflow Reconstruction》

- 任务定义：**只用输入-输出访问**，合成一个**可编辑的白盒替身工作流**去逼近黑盒系统
  （实验目标含 ChatDev / MetaGPT / TeachMaster / ChatGPT / Gemini）
- 方法：**在「统一原语空间」上做 MCTS + Red-Black 剪枝**
- 判据用 **SFE（静态功能等价）** 作可扩展代理（avg 0.426，优于 AFlow 0.339、
  Opus+ReAct 0.299）
- **红黑剪枝是关键**：剪枝让搜索深度从 L=2 走到 L=6，token 省 8–22%

### 1.5 辅助证据

- **AutoReproduce**（ACL 2026）：**paper lineage** 从被引文献里挖隐式知识；
  ⚠️ 明确警告 **LLM judge 会高估一致性** ——「文本描述的泛化性奖励了宽泛的功能相似，
  而非精确复现」
- **HIRAS**（ACL 2026 Findings）：消融显示**更细粒度的职责分解优于单体设计**；
  层次监督单独贡献约 **+10%**
- **代码图侧**（上轮已检索）：`codegraph`（委托类型检查器）· `arXiv 2606.22417`
  （受控消融，跨 ≥3 文件收益最大）· `grafel`（39 语言/跨仓/消息总线）· `ArchAgent`（架构恢复）

---

## 第二部分 · 从底层技术模型逆向推理【推理】

把「原子级拆解技术逻辑」与「复现对方产品」两个目标做**逆向分解**，
直到落到共同的最小基元：

| 目标 | 分解到底 |
|---|---|
| 原子级拆解技术逻辑 | 拆成**可独立判 pass/fail 的二值判据**（＝RePro fingerprint） |
| 复现对方产品 | 拆成**可判等价的行为规格**（＝RepoZero API spec + oracle） |

⇒ **两者共用同一个基元：一个「契约」。**

再往下推，契约要成立**必须同时满足四条**，缺一条就会退化成别的东西：

| # | 性质 | 缺了会怎样 |
|---|---|---|
| **K1 原子** | 每条可独立判定，不依赖其他条 | 变成"整体感觉对不对" |
| **K2 持久** | 跨长轨迹不丢失 | 复现到一半忘了要求（ReproAgent 实测的失败模式） |
| **K3 绑定** | 绑定到**具体符号/文件**，不是目录或字符串 | 无法定位失败，无法定点修 |
| **K4 外部可判** | oracle 在被测系统**之外**且确定性 | 退化成自证（自己声明自己对） |

**K4 是最关键也最常被忽略的一条。** RepoZero 的全部价值来自「原仓库当 oracle」；
AutoReproduce 警告的「LLM judge 高估」正是 K4 缺失的病症。

### 2.1 用四条性质反向审计 NeoTrix【实测】

| 性质 | NeoTrix 现状 | 证据 |
|---|---|---|
| **K1 原子** | ⛔ 判据存在但**不原子**：`check-layout` 判**文件树**、`check-layer-deps` 判**字符串**、`unwrap-baseline` 判**列表成员**。没有一条判「这个 unwrap 是否合理」「这个回滚是否真的发生」 | 门脚本清单 |
| **K2 持久** | ⛔ 判据**随发现漂移**：层债 baseline 8 → 19（我今天改的）；unwrap 712；静默失败 47 → 38。没有一份契约被固定下来 | `scripts/*-baseline.txt` |
| **K3 绑定** | ⛔ **只绑文件路径，不绑符号**。今天删 10 个 re-export 时门抓不到，正是因为 `layer-deps-baseline.txt` 记的是**文件**不是 `symbol@line` | `scripts/layer-deps-baseline.txt` 格式实测 |
| **K4 外部可判** | ⛔ **全部内部自证**：11 个门都是「自己的代码对自己的静态断言」。唯一接近的是 12,209 测试，但它是**同一份代码自测** | 门清单 + 测试 |

⇒ **四条性质，NeoTrix 一条都不满足。这就是「基础能力缺失点」的根。**

对照 RepoZero 的实测：**即使 40% 可执行代码仍语义错误**——
说明「跑通」远不够，**必须有外部 oracle 的等价性判据**。本仓今天恰在反面印证：
`--all-targets` 0 error + 12,209 测试全绿，而 `ios-bridge` / `onnx` 两个 feature 是断的。

---

## 第三部分 · 全域 map 的每链路缺陷

> ⚠️ **本节已于同日第二次修订**（原版按「无调用图 / 无存活率 / 无等价判据」写，
> 而这三条在同一天被工具关闭了）。这就是本节新增断言块的原因：
> **地图自己会腐烂，且腐烂时比没有更危险** ⇒ 每条「现状」都带**可执行谓词**，
> 跑 `python3 scripts/ops/nt_map_reconcile.py` 就知道它还成不成立。
> 本节断言块当前 **11/11 成立**（同日实测）。

把全域 map 看成一条链，**每一环都标注缺陷与其根因性质**：

```
源码 ─①→ 符号索引 ─②→ 结构拓扑 ─③→ 静态门 ─④→ 动态测试 ─⑤→ 外部吸收 ─⑥→ 复现对标
      ✅ 有边        ⚠️ 只有结构     ✅ 门自证  ✅ 有外部  ✅ 有存活率  ✅ 行为级判据
      670,093 边      无意图          14 个      oracle     48 条索引   lru/strsim
```

| 链 | 现状【实测·带可执行断言】 | 缺的是 | 状态 |
|---|---|---|---|
| ① 源码→符号 | `nt_mapgen` 82,835 符号 + `nt_calledges` **670,093 边**（THIR 委托编译器，非正则） | 无 | ✅ **G1/G2 已关** |
| ② 符号→结构 | `nt_topology` + `CODE-TOPOLOGY.md`；**只有结构（树/层/密度），无意图** | K1 意图层 | ⛔ **未做**（唯一仍开着的结构缺口） |
| ③ 结构→静态门 | 14 个门；静默失败 **32 契约全带 criterion+oracle**；feature 门控 6 个 | K4 外部性 | 🟡 门自证已破，但「门本身可满足性」只有 1 门 |
| ④ 静态→动态 | **12,213** 测试 + 行为对位 **54 步外部 oracle**（lru 37 + strsim 17） | K4 | 🟡 **K4 已有真实外部源**（不再自证） |
| ⑤ 动态→吸收 | 38+ 源；`nt_absorption_live` + 48 条任务索引 + 许可台账 | K2/K3 | ✅ **G5/G6 已关** |
| ⑥ 吸收→复现 | `nt_decompose parity`（名字级）+ `nt_parity_ref`（**行为级**） | K4 语义级 | ✅ **已关**（见 3.2 边界） |

```assert
file:scripts/ops/nt_calledges.py            # ① 调用边抽取器存在（THIR 委托编译器）
file:scripts/ops/nt_callgraph.py           # ①④ 可达性/影响面查询（G4，已入库 7940f13c）
file:scripts/ops/nt_absorption_live.py     # ⑤ 吸收存活率（G5）
file:scripts/ops/nt_decompose.py           # ⑥ 原子拆解 + 名字级对位
file:scripts/ops/nt_parity_ref.py          # ⑥ 行为级对位采集器
file:scripts/check-license.sh              # ⑥ 许可门（G6）
file:scripts/ops/nt_map_reconcile.py       # 本节这张表的自检器
test:levenshtein_counts_chars_not_bytes@neotrix-core/src/l4_emotion/nt_memory/entity_linking/linker.rs  # 字节距离回归测试仍在（被删则文档失效）
test:levenshtein_matches_reference@neotrix-core/tests/nt_capability_parity.rs  # ④ 行为对位判据仍在
cmd:python3 scripts/ops/nt_decompose.py selftest   # ⑥ 拆解器自证仍绿（6 正例 + 3 证伪）
```

**这张断言块当场抓到的三件事**（都是真发现，不是演练）：

| 抓到 | 真相 | 处置 |
|---|---|---|
| `file:nt_callgraph.py` → **exists but not in git index** | 当时 G4 的可达性工具是**另一窗口的在途未提交工作** | 当时从断言里删掉；**数分钟后该窗口提交了（`7940f13c`）⇒ 已加回断言**（见下） |
| `file:nt_map_reconcile.py` 未入库 | 提交前必然如此 | 提交后复跑转 HOLDS（实测） |
| `nlit:…@本文件` → **literal present** | ⛔ **我写的断言是自指的**：为了解释改动，文档必须引用旧措辞，于是 `nlit` 永远失败 | 删掉该断言。**教训：谓词会因解释它自己而失效** |
| `cmd:nt_map_reconcile.py` → **超时 120s** | ⛔ **自检断言调用了正在自检的工具 ⇒ 无限递归** | 删掉该断言。**教训：自检不能把自己写进自己的判据**（与上一条同源：凡「为解释而写」的东西都会污染判据） |

### 3.4 边界：谁的能力算进这张地图

**判据是「已入库」，不是「谁写的」。** `file:` 谓词含「被 git 跟踪」判定，
天然把在途未提交的工作排除在外 —— 地图不能把别人的在途成果记成既成事实，
那正是本文件第 3.3 节第 4 条自己刚犯的错。

⏱ **这条边界当场自证了一次**：写 3.4 时 `nt_callgraph.py`（G4 可达性/影响面）
尚未入库，我据此把它排除；**几分钟后另一窗口提交了 `7940f13c`**
（`ede6ea7f` 提交了 G6 的 `check-license.sh`）⇒ 断言已加回，两者现均 HOLDS。

⇒ 教训：**「未入库」是时点状态，不是结论。** 散文里写「某某还没入库」这类
时点判断时，应当**只写进断言、不写进散文** —— 断言会在入库那一刻自动转 HOLDS，
而散文不会自己更新（这正是本工具存在的理由）。

### 3.1b 副本漂移审计（fn-drift）—— 新增能力，且已抓到 3 个真缺陷

**审计缺口**：本仓有 `nt_dup_types`（重复**类型**），但**没有「重复函数实现」审计**。
而本会话的两个真 bug 都是**副本漂移**（同名/同用途实现语义不一致）。

工具 `scripts/ops/nt_fn_drift.py`：候选来自**编译器解析过的调用边**（不是正则扫源码），
逐对提取函数体并归一化（去注释与字符串内容）后判定：

| 判定 | 数量【实测】 | 含义 |
|---|---:|---|
| DIFFERENT | 160 name / 815 pairs | 同名不同语义 ⇒ **需人工裁决**（多数是「同名不同域」的合法重复，如 `osint::investigate` ×15） |
| IDENTICAL | 29 name | 归一化后逐字相同 ⇒ 纯重复，可合并 |
| UNRESOLVED | 24 name | 抽不出定义 ⇒ **不猜** |
| **命中缺陷形状** | **7 name** | 自动标注本会话已实证过的两类形状，优先分诊 |

两个形状（都是本会话**踩过两次**的复发型缺陷）：
- `ERR-DIVERGENCE` 一侧 panic、另一侧宽容 ⇒ 同一操作两种失败语义
- `UNIT-DIVERGENCE` 一侧 `.len()`（字节）、另一侧 `.chars().count()` ⇒ 单位混用

**首轮实跑即抓到 3 个真缺陷**（全部逐处读源码确认，非凭工具输出）：

| 缺陷 | 位置 | 后果 | 状态 |
|---|---|---|---|
| `now_ts` 错误处理漂移 | 全仓 **13 份副本**：11 份宽容、**2 份 panic**（`reference_view.rs:97`、`harness/refinement.rs:133`）| 容器/虚机时钟早于 1970 时，一个只读时间戳 helper 能打崩进程 | ✅ 已统一为宽容，基线 716→714 |
| `truncate` 按**字节**切 | `nt_io/nt_io_hive_agent_loop.rs:259` | `&s[..max]` 在非字符边界 **panic**。调用点 `truncate(&response, 200)` 是**模型输出**，200 不是 3 的倍数 ⇒ **中文回复约 2/3 概率 panic** | ✅ 已改边界安全 + 回归测试 |
| `truncate` 字节切 + **下溢** | `nt_file_ability/table_presenter.rs:152` | `&s[..max_len - 3]`：① 非边界 panic；② `max_len < 3` 时下溢 | ✅ 同上 |

**第二轮（`--units` 子命令）把 7 个形状命中逐族量化**，并已修其中影响最重的一处：

| 族 | 副本数 | 字节版 / 字符版 | 后果【实测】 | 状态 |
|---|---:|---|---|---|
| `estimate_tokens` | 7 | 3 / 3 | `seal_core/model_router` 用 `(bytes × 0.3)`，喂给 `classify_tier` 的 `tokens>800/1500` 门槛（T4=最贵档）⇒ **对中文低估约 10%**（0.9 vs 1.0 token/字），该升档时没升 | ✅ 已改为复用 CJK 感知单一事实源 |
| `truncate` | 8 | 2 / 4 + 2 混合 | 见 §3.1b 首轮 | ✅ 已修 2 处 panic |
| `truncate_chars` | 4 | 1 / 3 | 名字叫 chars 但按字节判长度（快路径无害，主体用 `char_indices`）| ⛔ 未动（**不判为 bug**：其 `s.len() <= max` 只是保守快路径，主体边界安全） |
| `tokenize` | 6 | — | 分词规则各异（`is_ascii_alphanumeric` vs 含 CJK 切分）⇒ **多数是有意差异** | ⛔ 未动（未取证） |

⚠️ **我自己也算错过一次并已更正**：最初注释写「中文会比英文早 2/3 篇幅升到 T4」。
实测算的是 `(bytes×0.3)` 中 bytes=chars×3 **与 0.3 抵消** ⇒ 旧口径**偏低**，
后果是**该升没升**。方向与我最初写的相反，已按实测改。

⚠️ **工具的已知局限（写在这里以防后人误用）**：只能发现**同名**副本。
本会话那个字节/字符 `levenshtein` bug 是 `levenshtein` vs `levenshtein_distance`
**不同名** ⇒ 本工具抓不到，只能靠行为对位（`nt_parity_ref`）。
⇒ **两套工具互补，不互相替代**：同名查 fn-drift，跨名查行为对位。

```assert
file:scripts/ops/nt_fn_drift.py             # 副本漂移审计器
cmd:python3 scripts/ops/nt_fn_drift.py --units estimate_tokens   # 单位一致性分诊可复跑
test:estimate_tokens_is_cjk_aware@neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/model_router.rs  # 字节估 token 回归测试仍在
test:test_resolve_rpc_url_uses_default@neotrix-core/src/l1_action/nt_act/nt_act_crypto/evm.rs  # env 竞态测试仍在
cmd:python3 scripts/ops/nt_fn_drift.py selftest   # 自证 7 例（含 3 例证伪：字符串/注释花括号、lifetime、声明-only）
test:truncate_never_panics_on_cjk@neotrix-core/src/l1_action/nt_io/nt_io_hive_agent_loop.rs  # 字节切 panic 回归测试仍在
test:truncate_is_boundary_safe_and_no_underflow@neotrix-core/src/l1_action/nt_file_ability/table_presenter.rs  # 下溢回归测试仍在
nlit:.expect("system time after UNIX epoch")@neotrix-core/src/l1_action/nt_act/reference_view.rs  # now_ts 已不再 panic
```

### 3.2 「复现对方产品」现在能做到什么、做不到什么【实测边界】

| 能力形态 | 能否对位 | 实例 |
|---|---|---|
| **算法型**（有公认参考实现） | ✅ 能 | LRU 缓存语义（`lru`）· 编辑距离（`strsim`） |
| **策略型**（数值是产品决策） | ⛔ **不能，也不该** | `search_scorer::fuzzy_match` 返回 0.85/0.75 是**人为定的权重**；`writing_style::fuzzy_similarity` 是词重叠启发式。⇒ registry 里**没有**对应 crate 可当 oracle |

⇒ **这是本轮最重要的边界发现**：不是所有能力都「可复现」。
把策略型实现硬凑一个参考实现，只会产生**自证的假判据**。
正确做法是把它当**规格问题**（写清意图 + 独立 oracle），不是当对位问题。
⚠️ 我上一轮把这两个函数记为「未验证」，那仍属**未取证**，不得读作「有 bug」——
与本仓 2026-09-30 那次「把 23 处模板残留当成真缺陷」是同一类风险。

### 3.3 一个横切所有链路的缺陷

**本仓所有判据都是「同一份代码对自己说话」。**
11 个门 + 12,213 测试 = 一个**自洽但可能整体错**的系统。
今天的三次实证：

1. `--all-targets` 0 error + 全部门 rc=0，而 `ios-bridge` **断**（我今天引入并修复）
2. 同上，而 `onnx` 的 4 个 error **已存在多时**
3. 12,209 全绿，而 `tier_archival.prune` 的测试**只覆盖了错误不可能发生的那条分支**
4. （本轮新增）`nt_map_reconcile.py` v1 用行内正则匹配谓词 ⇒ 全仓 **18 条假阳性**
   （把 `package.json` 的 `test:headed` 脚本名当成断言）⇒ **新写的工具自己过了头**

⇒ **K4 缺失是横切性缺陷，不是某一环的问题。** 本轮已把 ④ 从「自证」推进到
「**有真实外部 oracle**」（54 步），但 ③ 的门**仍然自证**。

---

## 第四部分 · 核心建议（按杠杆排序）

### 建议 A（最高）：建「行为契约层」，把 map 从 4 层补到 5 层

> ✅ **已执行并已兑现（2026-09-30 同日）**。落地形态：
> `nt_decompose.py`（原子拆解 + 名字级对位）+ `nt_parity_ref.py`（**跑对方实现**采 oracle）
> + `nt_capability_parity.rs`（我方侧注册表）。注册能力：`lru_core`、`levenshtein`。
>
> **它抓到的第一个真 bug**：中文实体链接按 **UTF-8 字节**算编辑距离
> （`linker::levenshtein` + 消费者 `max_len` + `semantic_entropy::char_similarity` 三处
> 单位混用）⇒ ASCII 全对、**CJK 全错**，阈值 0.6 一类的配置对中英混排**系统性漏合并**。
> 详见 `DECOMPOSE-PARITY-2026-09-30.md` §7。
>
> **但建议 A 的原始形态只覆盖了 ③⑤⑥ 的「契约」侧，K4（外部 oracle）它没解决** ——
> 真正兑现 K4 的是 建议 D（行为级对位）。两条建议合起来才是本节主张的 5 层。

```
现在：源码 → 符号 → 结构 → 门 → 测试
补一环：源码 → 符号 → 结构 → 【契约】 → 验证
                              ↑
                    (target, criterion, oracle, location, status)
```

一条契约就是上面四条性质的载体。**判据直接抄 RePro + RepoZero，不发明。**

**为什么这是最高杠杆**：它是 ③⑤⑥ 三条链的共同底座；
没有它，「原子拆解」和「复现对标」都只能停在文档层。

**第一刀（可立即做，不需要新工具）**：给现有的 38 条静默失败基线 + 19 条层债
**升级为契约**——每条补 `criterion`（为什么它必须是可观测的）与
`oracle`（用哪条测试或哪次实测证明它仍成立）。
这一步把 `K3` 从「文件」升到「符号 + 判据」，且**不依赖调用图**。

### 建议 B：外部 oracle 优先做一件小事 —— doc 承诺对账

今天已抓到 1 例（`nt_core_reasoning.rs` doc 承诺 vs 实现已删）。
把它扩成门：**每个模块的 doc 承诺 vs 生产可达性**。
这是**唯一低成本、天然外部可判**的 oracle 来源（doc 是给人看的，
代码不是给人承诺服务的 —— 二者不一致即为缺陷）。

对应 RepoZero 的精神：oracle 不能是被测系统自己。

### 建议 C：吸收链补两个工具（依赖建议 A 的 K2/K3）

- `nt_decompose`：把外部条目切成**原子契约**（RePro 管线：抽 guide → 标准化 → 过滤），
  落 YAML，**不是 markdown**
- `nt_absorption_live`：对每条契约查「今天是否仍可达」
  —— **G5 的直接解法**；判据：跑一次应当复现出 212 死文件量级，
  若报「全部存活」则判据无效而非仓库健康
- `check-license.sh`：吸收外部代码时过许可（外部横评把许可列为选型一等公民，
  `GitNexus` 是 PolyForm Noncommercial **禁商用**）

### 建议 D：复现对标的可计算形态

按 RepoZero 的重述方式，**「复现对方产品」= 只持有对方的行为规格，
要求我方行为等价**。落到本仓：

1. 对方 = 确定性 oracle（API 规格 + 可复现输入 → 期望输出）
2. 我方 = 同一输入下的实际输出
3. 判据 = **输出等价**，不是「功能都写了」（AutoReproduce 警告：
   LLM judge 会因文本泛化而高估宽泛相似）

⚠️ RepoZero 的实测难度：**~40% 可执行代码仍不等价，最强 agent 30–55% 通过率。**
⇒ **不要把「复现」当成写代码任务，它是等价性验证任务。**

### 建议 E：⛔ 明确不做（避免重复我今天的错误）

- ⛔ 不给死代码建门（工具自述不可靠）
- ⛔ 不用正则猜 call 边（违反「委托」与「宁缺勿错」）
- ⛔ 不用「文档写了」当能力已具备（乐观措辞比没写更有害，今天更正过两次）
- ⛔ 不同轮并行做 A 与 C（C 依赖 A 的 K2/K3）

---

## 第五部分 · 能力边界的诚实说明

- **「原子级拆解**任何**技术逻辑」**不是代码产物，是**契约集**。
  能工具化的部分：判据的**抽取、绑定、验证**；不能工具化的部分：
  「这两段逻辑算不算同一个原子单元」—— 这是语义判断。
  外部也无定论：RepoZero 明确把「如何度量架构可读性与结构完整性」列为**开放问题**。
- **「复现**任意**对方产品」**在 RepoZero 实测里最强只有 30–55%**。
  这不是工程投入不足，是**任务本身的可复现性有上限**。
  把它当「照着做一遍」会系统性高估自己。
- 我能交付的是**机械底座**（契约层 + 判据 + oracle 接线）；
  不能交付的是**语义判断**。把这两者混为一谈，就是我今天犯过三次的错。

---

## 第六部分 · 一句话路线

```
已关：构建维度盲区（feature 门控）· 语义维度盲区（静默失败 4+6）· 检索层盲区（rg -E）
已关：结构维度（调用边）—— `scripts/ops/nt_calledges.py` 落地，全 workspace 670,093 条边，
  THIR 委托（`ty: FnDef(DefId)` 直接读编译器解析），精度经 fn 指针/dyn/闭包 toy（3/4）与
  25 条随机边核对验证；`.project-map/edges-all.jsonl`（gitignored，低频再生）
下一刀：契约层剩余 35 条静默失败逐条裁决（K3，不依赖调用图）
之后：吸收工具化 + 存活率（关 K2/K4）；跨仓查询合并（边表已有跨仓目标）
```

未推送：按指令仅本地提交。

---

## 附录 · 建议 B 的**已证否**结果（2026-09-30 追加）

建议 B 是「doc 承诺对账门」。**实测结论：这一类不适合做成通用门。已停手。**

### 真实缺陷确实存在（不是假的）

`crates/neotrix-reasoning/src/reasoning_core.rs:3`：

```
//! 提供 KB/经验 → Kernel context 自动注入的 ContextBuilder。
```

`ContextBuilder` 曾在 `0e36ebe6` 定义为 `struct`，现文件 **0 命中** ——
**能力被删、doc 未跟**。这正是「文档承诺 vs 生产可达性」的活实例。

### 但两种判据都不可用（实测，非推测）

**判据一：doc 提到的标识符在代码里找不到。**

| 版本 | 候选数 | 问题 |
|---|---:|---|
| 搜索体含 doc 注释 | **0** | ⛔ **自满足**：doc 里提到 ⇒ 在源码里找得到 |
| 搜索体去注释（正确版） | **2,423** 个不同标识符 | top 全是英文单词：`Whether`(138) `Get`(35) `Convenience`(33) `Currently`(24) `Tracks`(24) `Wraps`(20) `Falls`(17) |

⇒ 中文注释里首字母大写的英文词被当成类型名。**门开出来就是 2,423 个误报**
—— 比本会话刚修好的「空跑绿灯」更糟（噪声红的门同样会被忽略）。

**判据二：用 git 历史当 oracle（外部于工作树，本可满足 K4）。**

- 概念验证通过：`ContextBuilder` 确有「曾定义 → 现消失 → doc 仍提」的历史
- 但实现未证且**成本无界**：需对每个候选跨全历史查询
- 本次实测的 oracle 命令本身在当前 git 版本下不产出（`git grep -o` + tree-ish），
  我的 pathspec 也写错了目录名 ⇒ **该路线本轮未证**

### 结论与替代

⛔ **不建这个门。** 与我本会话否掉「死代码门」的理由**同构**：
工具/判据自身不可靠时，建门只是把不可靠放大成 CI 红灯或噪声绿。

✅ **替代机制：审计时人工发现，而非模式匹配。**
`ContextBuilder` 是子代理**读代码**时发现的，不是 grep 出来的 ——
这本身就是更可靠的机制。

📌 **若将来仍要做**，唯一有希望的方向是窄化到
「**同一文件**的 doc 与**同一文件**的类型定义不一致」，
而不是跨 crate 全集匹配（后者必然被英文词淹没）。
