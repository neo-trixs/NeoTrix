# MIRROR-BANK-2026-09-30 — `nt_core_bank` 不是镜像冗余：**两个各自演化的记忆库**

> 承接 `MIRROR-FORK-2026-09-30` 的剩余清单。本轮对最大的那片（`nt_core_bank`）
> 做完取证后**否决了删除**，并给出架构设计方案。
> 纪律：每个判断都有可复跑命令；**否决也留档**（否则下一轮会有人再删一次）。

---

## 0. 一句话结论

**两侧各 156 KB、体量精确对称，但内容已经各自演化。**
强制合并会**丢失 51 个函数 + 39 个 pub 类型** ⇒ 这不是冗余清理的对象。
真正的问题不是「有两份」，而是**两套记忆系统没有分工，且没人知道该用哪个**。

---

## 1. 体量对称（实测）

| 侧 | 路径 | 体积 | 文件 | 函数名 | pub 名 |
|---|---|---:|---:|---:|---:|
| types（07-06 冻结） | `crates/neotrix-types/src/core/nt_core_bank` | **156 KB** | 18 | 166 | 108 |
| core（09-17 起演化） | `neotrix-core/src/l1_action/nt_core_bank` | **156 KB** | — | 189 | 90 |

体量对称到让人以为是镜像 —— **但函数级比对否掉了这个假设**。

## 1.5 ⚠️ 修正：上一轮报的「26 个分歧」**虚高**，真实是 **12 个**

本轮做了类型感知 + 格式化感知的重新判定（工具 `scripts/ops/nt_diverge.py`），
**推翻了我上一轮自己写进文档的数字**：

| 口径 | 同名对 | 逐字相同 | 「分歧」 |
|---|---:|---:|---:|
| 上一轮（按**叶子名**比 + 未去尾逗号） | 99 | 73 | ~~26~~ |
| **本轮（按 owner 配对 + 去尾逗号）** | **77** | **41** | **12** |

两处系统性假阳性：
1. **尾逗号**：`None=>return v()};` vs `None=>return v(),};` —— 已修
   `nt_fn_drift.normalize_code`（v3）。
2. **同名不同类型**：`is_empty` 低层属 `impl CompressionReport`、高层属
   `impl RagEngine` —— **两个不同类型的同名方法**，按叶子名比会误算成分歧。
   本轮实测 COLLISION **24** 对，全部属此类。
3. 顺带修了一个更深的既有缺陷：`nt_diverge` 初版用 `nt_fn_drift.extract_fn`
   取函数体，而后者返回**同名函数的第一个**（与 owner 无关）⇒ 曾把
   `Alpha::pick`（两侧都是 `1`）判成 DIVERGENT。已改为**按 owner 块内切取**。

### 12 个真分歧的正确定性（逐条读源码）

| 类别 | 分歧点 | 判定 |
|---|---|---|
| **纯格式化** | `l1_prompt` `l2_prompt` `save_scene` | 去空白后完全相同 ⇒ 非语义差异 |
| **仅路径不同** | `enable_hypergraph` | `crate::core::…` vs `crate::l1_action::…`，同构 |
| **结构分叉** | `touch` `empty` `new`×2 | 高层 `ReasoningMemory` **没有** `last_used_at` 字段；`Bm25Index` 没有 `idf` ⇒ 漏写的那行**不是 bug**，是结构已分叉 |
| **数据差异** | `initialize_with_everos_knowledge` | 低层播种 **5** 条、高层 **4** 条，**且两侧测试分别断言 5 与 4** ⇒ 被各自测试固化的产品决策 |
| 其它 | `search` `build` | 随 `Bm25Index` 的 `idf` 字段差异连带 |

⇒ **结论收紧**：这不是「26 处语义债」，而是 **12 处分歧，其中 0 个是 bug**。
其中真正需要人判断的只有 `initialize_with_everos_knowledge`（该播种几条）。

---

## 2. 内容已各自演化（决定性取证）

| 判据 | 实测值 | 含义 |
|---|---:|---|
| 同名函数 | 99 | 一半以上同名 |
| 其中**逐字相同** | **73** | 这部分是真冗余 |
| 同名但**实现不同** | **12**（上一轮误报 26，见 §1.5） | ⚠️ 其中 0 个是 bug，4 类成因已逐条定性 |
| types 独有（core 全仓无同名） | **51** | ⚠️ core 缺这些能力 |
| — 其中在 core **其它模块**找得到的 | 16 | 只是搬了位置（如 `apply_decay`、`cosine_similarity`） |
| — core **确实没有**的 | **35** | 真实能力缺口 |
| types 独有 pub 类型 | **39** | 删了就丢公开 API |

### 三个「同名不同语义」实例（同 `ReasoningHexagram` 那一类）

`nt_core_hex` 的 `ReasoningHexagram` 两侧都有 `pub struct ReasoningHexagram(pub u8)`，
但 `new()` 一边 `assert!(bits < 64)` **panic**、一边 `Self(bits & 0x3F)` **静默掩码**；
`axis()` 一边直移位、一边 `i & 7` 环绕。
⇒ **同名类型、不同语义**，且都已确认当前无越界（core 侧所有构造都在 `0..64`）
⇒ 这是**语义错位但当前无缺陷**，已记录不修（改动会波及 40+ 文件且无实测收益）。

## 3. 为什么这些「零调用函数」不能删

rg 统计 types 全仓零调用函数 91 个（生产 89 + 测试 2）。逐个查可见性：

| 类别 | 数量 | 处置 |
|---|---:|---|
| `pub`（库 API 面） | **25** | ⛔ **不删**：`neotrix-types` 是 pub 库，`pub fn` 是给下游 crate 的 API。仓内零调用 ≠ 无人使用 |
| 私有 | 64 | 理论上可删，但分散在 8 个文件、且多在 `#[cfg(test)]` 邻近，收益远小于误删风险 |

⇒ 「仓内零调用」在本仓**不是**死代码判据。这与 `nt_dup_dead` 的结论一致：
本仓边表有已知假阴性（`textual-prod` 桶），静态判据不足以支撑删除。

---

## 4. 架构设计方案（真正的修复）

### 4.1 问题定义

不是「两份代码」，而是：**两个记忆后端并存，无分工、无标记、无指引**。
新代码（和 agent）面对 `nt_core_bank` 该用哪个，只能靠猜。

### 4.1b ✅ 已执行一处**单点真身收敛**（本轮，唯一动了生产代码的一次）

`nt_diverge.py` 在 41 个「同 owner + 逐字相同」的函数里，找出 **owner 为 `(free)`
（自由函数，不依赖结构体）的 2 个**：

| 函数 | 原状 | 处置 |
|---|---|---|
| `tokenize` | 两侧各一份**逐字相同**的 `pub` 实现 | 实现留在 `neotrix-types`，高层改 `pub use` |
| `rrf_fuse` | 同上 | 同上 |

**为什么这两个能收敛而其余 39 个不能**：自由函数不带 `self`、不依赖结构体字段，
可以直接跨 crate 复用。其余 39 个是 `impl ReasoningBank` 等**结构体方法**，
而两侧 `ReasoningBank` 已**结构分叉**（高层没有 `last_used_at`、`Bm25Index` 没有 `idf`）
⇒ 收敛它们必须先统一结构体，那是更大的重构。

**方向合法性**：`core` 依赖 `types`（`Cargo.toml:97`）⇒「高层引用低层」唯一合法。
**为什么必要**：两份逐字相同的实现**只会各自漂移**——本会话已实测同类漂移
（`now_ts` 13 份副本里 2 份 panic、`truncate` 8 份里 2 份字节切 panic）。
**测试保留**：高层 `mod tests` 里针对这两个函数的测试**没有删** ——
它验证的是**行为**，改实现不应让测试消失。

**验证**：`cargo check --workspace --all-targets` 0 error 且**我改的两个文件 0 警告** ·
`neotrix --lib` **12,217 绿** · 确认高层文件里 `fn tokenize`/`fn rrf_fuse` 定义已消失。

### 4.2 方案：**分层 + 显式归属，不删代码**

依赖方向已定：`neotrix-core` **依赖** `neotrix-types`（`Cargo.toml:97`），
所以 `types` 是低层库、`core` 是上层实现。二者不是镜像，是**基类与实现**。

| 层 | 归属 | 内容 |
|---|---|---|
| **契约层**（types） | 只保留**跨后端共用**的契约类型与纯函数 | 73 个逐字相同的函数 + 42 个同名 pub 类型 |
| **实现层**（core） | 各自演化的实现 | 26 个同名不同实现 + 51 个 core 独有 |

**具体三步**（每步独立可验证、独立可回滚）：

1. **标注**：在 `types/nt_core_bank/mod.rs` 顶部写明「本模块是低层契约库；
   上层实现在 `neotrix-core/l1_action/nt_core_bank`；新增能力请加到上层」。
   成本：1 行注释。收益：消除「该用哪个」的歧义。
2. **冻结**：把 `types` 侧该簇加入 `nt_mirror_scan` 的观察名单，
   在 CI 里**只报告不失败**（G7：相似度不足以当门），让漂移可见而非隐形。
3. **收敛**（**需 owner 决策，不在本轮**）：把两侧**逐字相同的 73 个函数**
   提取到 `types` 的公共位置，core 改为 `use` 它。
   ⛔ 前置：26 个同名不同实现的函数必须先逐个定性（是「core 更对」还是「types 更对」）。
   在这之前不动 —— 这 12 个才是真正的架构债（其中 0 个是 bug，见 §1.5）。

### 4.3 为什么第 3 步不做

26 个同名不同实现的函数里，每一个的「谁对」都是**领域判断**：
例如 `apply_freshness_ranking` 只在 types 有、`cosine_similarity` 两边参数不同。
在没有行为对位（`nt_parity_ref`）覆盖这些函数之前，合并等于**赌**。
⇒ 记录为 TODO P1，并给出可执行入口（见 §5）。

## 5. 下一步的可执行入口

```assert
file:scripts/ops/nt_mirror_scan.py               # 镜像/分叉观察（只报告）
cmd:python3 scripts/ops/nt_mirror_scan.py selftest
cmd:python3 scripts/ops/nt_mirror_scan.py --min-overlap 0.5   # 清单可复跑（数字见 MIRROR-FORK §3）
```

**要推进第 3 步，需要的行为对位**（本仓已有工具链）：
给 `nt_core_bank` 的公共函数配一份 vectors，让 `nt_parity_ref` 采 core 侧行为作为
oracle，再逐个判定 26 个分歧点谁对。**先有判据，再动代码。**

⚠️ 本文件**不含**删除断言 —— 因为本轮**否决了删除**。
若将来执行 §4.2 第 3 步，须把「26 个分歧点已定性」写成可核对清单，
而不是直接合并。
---

## 5. 单点真身收敛：判据、已做范围、以及**为什么不能全做**（2026-09-30 补充）

### 5.1 筛选判据（三道，缺一不可）

对「`neotrix-types` ⇄ `neotrix-core` 各层」的 **183 个「同 owner + 归一化后逐字相同」**
的函数（其中 owner 为自由函数者 35 个，剔除 8 个 `test_*` 后 27 个）：

| 道 | 判据 | 实测结果 |
|---|---|---|
| ① 类型感知确认 | `nt_diverge.py`：owner 相同 + 归一化后全等 | 排除「同名不同 owner」的假重复 |
| ② **返回类型可跨 crate** | 返回值不能是 crate 本地类型 | **这是决定性一道** |
| ③ 常量/外部依赖一致 | 函数体引用的常量必须逐个比对相等 | E8：16 常量三方全一致 |

### 5.2 ②为什么是决定性的（本轮最重要的架构结论）

`Hexagram`、`FermionState` 在两个 crate 里是**各自独立定义的同名类型**：

```rust
// crates/neotrix-types/src/core/nt_core_e8.rs:139
pub struct Hexagram { pub bits: u8 }
// neotrix-core/src/l2_perception/nt_core_e8/nt_e8_hexagram.rs:14
pub struct Hexagram { pub bits: u8 }   // ← 同名，但是**不同**类型
```

收敛 `-> Vec<Hexagram>` 的函数，会把 core 侧返回类型**换成 types 的类型**。
那**不再是函数收敛，而是类型统一**，爆炸半径 = core 内 **57 个**引用 `Hexagram` 的文件。

⛔ 因此**「逐字相同的自由函数」不等于「可收敛」**。
必须先问：返回值是不是跨 crate 的同一类型？

### 5.3 本轮实际收敛（12 个函数 + 4 个常量）

| 文件 | 数量 | 返回类型 | 处置 |
|---|---|---|---|
| `nt_e8_roots.rs` | 4 | `Vec<Vec<i8>>`/`(usize,usize)`/`bool` | ✅ 收敛（`62cb175f`） |
| `nt_e8_homology.rs` | 8 | `bool`/`Vec<(&str,bool)>` | ✅ 收敛（`62cb175f`） |
| `nt_world/nt_world_e8.rs` | 4 常量 | `usize` | ✅ 收敛（`4326bda2`） |
| `nt_e8_hexagram.rs` | 3 | `Vec<Hexagram>` | ⛔ 类型统一是另一决策 |
| `nt_e8_fermion.rs` | 2 | `Vec<FermionState>` | ⛔ 同上 |

**净效果**：`nt_e8_roots.rs` 与 `nt_e8_homology.rs` 消除 **124 行**重复实现
（`62cb175f`：+37/−124），core 侧不再有第二份真身。

### 5.4 顺带查出的跨域错位与死常量

* **跨域错位**：`nt_world` 域重复定义 `nt_core_e8` 域的 E₈ 数学常量（同名 4、值 4/4 相同）。
  E₈ 常量真源唯一在 `nt_core_e8`，`nt_world` 是消费方却影子式复写。
* **死常量**：上述重复的 4 个里有 3 个（`E8_DIM` `DAYAN_NUMBER` `OBSERVABLE_DOF`）
  在 `nt_world_e8.rs` 内**只出现定义处、零使用** —— 不只是重复，还是没人用的重复。
* **第三份副本**：E₈ 常量另有 `nt_world/nt_world_e8.rs` 一份 ⇒ 实为
  types / `nt_e8_constants.rs` / `nt_world_e8.rs` **三方**，本轮做了三方逐值比对。

### 5.5 两条可复用的操作纪律（本轮我自己踩出来的）

1. **按字符串切片改代码会留残渣**：首版脚本切函数后残留半截 `────` 分隔线，
   编译报 `unknown start of token: ─`。**方框制表符出现在代码位置**是很好定位的信号。
2. **计数式判断必须读那一行本身（R-SCAN-1b）**：删导入时 `TOTAL_LINES` /
   `LO_SHU_CONSTANT` 显示「仍被引用」，实为**我自己写的说明注释里提到了它们**。
   同理，我写文档时漏了 `src/` 被 `check-doc-drift` 抓出 ⇒
   **文档里的路径是会被门校验的，不是纯排版**。

---

## 6. 收敛战役：已做范围、剩余阻塞、以及两条新工具（2026-09-30 汇总）

### 6.1 最终成绩

| 对象 | 收敛前 | 收敛后 |
|---|---|---|
| 跨 crate 逐字相同的**自由函数** | 27 个候选 | **6 个已收敛**，7 个确认阻塞 |
| `RRF_K`（同值常量） | 5 份定义 | **1 份**（唯一真身在 types） |
| `RESONANCE_THRESHOLD` | 4 份定义 | **1 份** |
| `nt_world_e8.rs` 里的 E8 数学常量 | 4 份影子复写 | **0**（改为引用） |
| net 消除重复实现 | — | **约 160 行** |

已收敛清单：`tokenize` `rrf_fuse`（bank + l4 bm25 两处）、
`hadamard_matrix` `hexagram_hadamard` `e8_root_norm_counts`
`verify_hadamard_orthogonality` + 8 个 `verify_*` 身份校验、
`verify_total_fermions` `su3_generators`。

### 6.2 剩余 7 个为何全部阻塞（这是**类型统一**问题，不是冗余问题）

`all_reasoning_states` `all_sm_fermions` `king_wen_sequence`
`shao_yong_sequence` `evolve_strategy_entry` `create_backend`
`capability_vector_group_a`

全部因为返回值/参数是 **crate 本地类型**：`ReasoningHexagram` `FermionState`
`Hexagram` `Box<dyn VsaBackend>` `CapabilityVector`。收敛它们 = 把两 crate 的
同名类型**统一**，爆炸半径：core 内 **57 个**文件引用 `Hexagram`。

⛔ 这是**架构决策**，不是重构收尾。要做需先回答：E8/GWT 的领域类型真源放
`neotrix-types`（契约层）还是各层自持（实现层）？当前仓库的分层意图
（types = 跨后端契约）指向前者，但需要一次显式决策 + 分阶段迁移，
不能夹带在清理提交里。

### 6.3 新增两条工具（都有自证，且都被真实代码验证过）

| 工具 | 覆盖维度 | 自证 |
|---|---|---|
| `scripts/ops/nt_const_dup.py` | **常量**：同名同类型 → `IDENTICAL`/`DIVERGENT` | 9 例（含 5 证伪） |
| `scripts/ops/nt_diverge.py`（已修） | 函数：owner 感知的 `IDENTICAL`/`DIVERGENT`/`COLLISION` | 4 例（含 3 证伪） |

`nt_const_dup` 存在的理由：常量级重复**函数级工具看不见**，而本轮最好的两个
发现（`nt_world_e8` 的 E8 常量、`RESONANCE_THRESHOLD`）**都是常量级**的。

### 6.4 修 `nt_diverge` 的 owner 误判（本轮最有价值的一条工具修复）

**症状**：`entry_count` 被报成「逐字相同的自由函数」，但两侧明明都是 `&self`
方法（`KnowledgeProvider::entry_count` vs `KnowledgeStorage::entry_count`）。

**根因**：`owner_before` 靠花括号配平判断「最近的 `impl` 是否已闭合」。若某个
`impl` 在**本 fn 之前**已闭合，它返回 `(free)` ⇒ 两个**不同结构体**的同名方法
被当成「同一函数的两个副本」。

**危害**：把**不可收敛的方法**混进收敛候选清单 —— 清单看起来「还有 27 个可做」，
实际只有 10 个，其中仅 4 个可动。**工具的误判会直接放大决策错误。**

**修法**：地面真相是**签名里有没有 `self`**（有 self 绝不可能是自由函数）。
宿主类型不可确定时标 `(method:unknown)` 且**不参与配对**，避免二次错配。
已补**回归自证**（真实踩中形态），并同时验证「真自由函数仍须正常识别」，
防止判据做过火。

**修复效果**：候选 27 → 17 对（剔 `test_*` 后 10 个真候选）。

### 6.5 本轮我自己的三次误判（全部由工具/编译器抓出，已留档）

1. **断言 `MODULE_COUNT=15` 是 bug** → 读代码发现数组实为 15 个（我手数漏了
   `CADGeneration`）⇒ 两侧各自自洽，真缺陷只是**注释与代码矛盾**。
2. **把 `RESONANCE_THRESHOLD` 误归为「world 自有、不重复」** → 实际与 gwt 侧
   同值同概念，下一笔已补收敛。
3. **以为该常量定义在 E8 块内** → 实际在文件 64 行 ⇒ E0255 重复定义。

⇒ 三次的共同教训：**命中项必须读那一行本身**，且「归入大概率不重复」与
「判定为缺陷」一样需要证据（R-SCAN-1b）。

### 6.6 自查清理：`pub` 死副本编译器不会报

`bank/iteration.rs` 的 `pub const RRF_K` 在我上一轮收敛 `rrf_fuse` 后变成
**零使用的重复定义**，但因为是 `pub`，**编译器不报 unused** ⇒ 静默留存。

三条删除依据：① 真身已统一；② 零使用；③ 该模块在 `mod.rs` 里是
`mod iteration;`（**私有**）且从未转出该常量 ⇒ 从来不是公开 API。

⚠️ 一般教训：**收敛一个函数后要回头看它留下的常量/辅助函数** ——
`pub` 修饰会让编译器失去检查能力，这类残留只能靠人工/工具发现。
`nt_const_dup.py` 能发现「同名多份定义」，但**发现不了「单份却已无人使用」**
⇒ 后者仍需 `dead_code` 类工具或人工核对。

---

## 7. ⛔ P1 跨域错位（未修，需架构决策）：L0 里藏着 L5 的 IIT Phi 陈旧分叉

### 7.1 事实（全部实测，非推断）

| 事实 | 证据 |
|---|---|
| l5 有**活**的 IIT Phi | `l5_cognition/nt_core/nt_iit_phi.rs`（481 行），**6+ 个消费者**（l5 `evolution_loop` `evolution_daemon`、l6 `healing`×3） |
| l0/l2 有**陈旧副本** | `l0_substrate/nt_core_consciousness_types.rs`：同名 `PhiReport` `IITPhiCalculator` + 3 个 `PHI_*` 常量。⛔ **本节初稿只查到 2 份，是不完整的** —— 后续用 §7.6 的新门查出实为**三份**，`l2_perception/nt_world/nt_world_model_v2.rs` 还有第 3 份（4 方法，缺 `compute_phi` `record` `resonance_matrix` `subsystem_analysis`）|
| 两份**已经漂移** | l5 impl 块 **5,920 字符 / 8 方法**；l0 impl 块 **3,634 字符 / 6 方法**。l0 **缺** `compute_from_state` `subsystem_analysis`，块小 **38%** |
| 副本的**唯一**非测试消费者 | `l0_substrate/ffi/consciousness_tree.rs:290`，而 `ffi` 整体在 `#[cfg(feature = "ios-bridge")]` 之下 ⇒ **默认构建下这份副本无人使用** |
| 副本为何被「保活」 | `nt_core_consciousness_types.rs` 内一条测试 `iit_phi_calculator_new` 只断言构造函数字段 ⇒ **测试给了它一条假命** |
| 为何要复制 | l0 全仓 **0 处**引用 l5/l6 ⇒ 层规则禁止 l0→l5。**这份副本是绕过层规则的手工变通** |

### 7.2 严重性：不是「重复」，是「同一算法有两个会分叉的实现」

`compute_phi` 存在于两份中。若两份的数值逻辑有任何差异，则**同一个 IIT Φ
指标在 FFI 路径与正常路径下会给出不同的值**，且没有任何机制会报错。
而现状已经证明它们**确实在分叉**（方法集合不同、块大小差 38%）。

### 7.3 为什么本轮**不**直接修（三个理由，都不是「懒得修」）

1. **直接删会破坏 `ios-bridge` feature**：`check-feature-gates.sh` 覆盖该feature，
   删掉副本会让 `--features ios-bridge` 编译失败 ⇒ 不是「删了没人发现」的死代码。
2. **正解是架构迁移而非清理**：把 IIT Phi 移到契约层 `neotrix-types`
   （与 §7.1 中 `CALIB_*` 同一判据：真身必须落在**两个约束都能满足**的位置）。
   这要动 481 行 + l5/l6 的 6 个消费者 + FFI，**是迁移不是收敛**。
3. **工作树共享**：`l6_meta/healing/` 等消费者是活跃区域，跨窗改动风险高。

⇒ 正确处置是**把它记成显式架构债**（本节），而不是夹带进清理提交。

### 7.4 给后续决策者的具体建议（按风险从低到高）

1. 先补一条**门**：把「同一算法名在 L0 与 L5/L6 两侧各有一份 `impl`」
   加进 `nt_mirror_scan` 的检查项（当前它只比同名**类型**，不比同名 `impl` 方法集）。
2. 若要保留 FFI 路径：把 `IITPhiCalculator` 迁到 `neotrix-types`，
   让 `ffi/consciousness_tree.rs` 与 l5 消费者**共用同一实现**，
   然后删掉 l0 副本及其那条保活测试。
3. 若判定 FFI 不需要 Φ：直接从 `ffi/consciousness_tree.rs:290` 摘掉该调用，
   删 l0 副本 + 保活测试（改动最小，但需确认 FFI 的 ABI 契约是否依赖 Φ 输出）。

⚠️ 三条路都要求先确认 `ios-bridge` 的对外 ABI 契约 —— 这属于产品/接口决策，
不是重构能单方面决定的。

### 7.5 附带产出：新工具 `nt_pub_dead.py`

本节正是它扫出来的。补上另两个工具的**已知盲区**：

* `nt_const_dup` / `nt_diverge` 只能发现「**同名多份定义**」，
  发现不了「只有一份、但没人用」。
* 实测代价：2026-09-30 收敛 `rrf_fuse` 后，core 侧留下一份
  `pub const RRF_K`，变成零使用的重复定义 —— 但 `pub` 项**编译器不报 unused**，
  于是静默留存。
* `nt_pub_dead.py` 找的正是这一类（16,779 条 `pub` 定义 → 1,858 条零引用候选）。

⛔ 但**零引用 ≠ 死代码**（6 类正当豁免见其 `--help`），且实测命中率约 11%
⇒ 它是**人工分诊的输入**，不是自动判决。工具首版就把 `impl Trait for X`
的方法全报成候选（命中率因此虚高），已加作用域识别修正 —— 记录这件事是因为
**「工具命中率过高」本身就是工具不可用的信号**，应当立即修而不是调阈值。

### 7.6 建议第 1 条已落地：`nt_mirror_scan` 新增「跨层同名 impl 方法集」检查

新增 `report_dup_impls()`，扫「跨层同名 `struct`/`impl` 且方法集不一致」。
**原工具看不见 §7 这类缺陷**：它按**模块名**匹配，而该分叉藏在两个
不同模块名的文件里（`nt_core_consciousness_types.rs` vs `nt_iit_phi.rs`）。

**判据经四次修正才可用，每一版都有实测数字**（这也是本节最值得留档的部分）：

| 版本 | 判据 | 命中 | 问题 |
|---|---|---|---|
| v1 | 方法集不相等即报 | **135** | `Actor`/`Channel` 等 J=0 的**同名巧合**全报 |
| v2 | 0 < Jaccard < 1 | **94** | J=0.08 实测只共享 1 个方法（`new` 撞名） |
| v3 | 交集 ≥3 且 J ≥ 0.5 | 13 | ⛔ **漏掉已知真例** `IITPhiCalculator` |
| **v4（现用）** | 以方法最多者为基准 + **缺 ≥2 个方法** + **pairwise J ≥ 0.4** | **16** | 可操作 |

⛔ **v3 漏报的原因值得单独记**：`IITPhiCalculator` 实为**三份**副本，
用「所有站点的共同交集」时，交集被离群副本拉低到 3 个（J=0.38）⇒
**度量本身有缺陷**，不是阈值没调好。
⇒ 改成**与最大站点比**（`max` 而非 `all-intersection`），离群副本不再拉低指标。

⚠️ 这条与 §7.5 的教训同源：**判据错时不要靠调阈值掩盖，要改度量**。
四次修正中 v1→v2→v4 都在加**语义条件**（是否同源、是否缺方法），
而不是单纯挪数字；v3→v4 则是**换了度量方式**。

新增自证（3b）把这四个坑全部固化：判据必须判陈旧分叉为分叉、
且必须**不**把同名巧合判成分叉。

---

## 8. 跨层陈旧分叉 16 组：优先级分诊（2026-09-30）

`nt_mirror_scan --dup-impl` 输出的全部 16 组。**本节只做分诊，不动代码** ——
每一组的「真身该放哪」都是架构决策，且多数涉及类型统一（同 §6.2 的阻塞）。

### 8.1 按「共享面 Jaccard」分层（实测值，非估计）

**A 类 — J ≥ 0.71，同源性极强，最可能是真冻结分叉（4 组）**

| 类型 | 基准（方法数） | 陈旧侧 | J | 缺的方法 |
|---|---|---|---|---|
| `IITPhiCalculator` | l5 `nt_iit_phi.rs` (8) | l0 `nt_core_consciousness_types.rs` (6) | 0.75 | `compute_from_state` `subsystem_analysis` |
| | | l2 `nt_world_model_v2.rs` (4) | 0.50 | `compute_phi` `record` `resonance_matrix` `subsystem_analysis` |
| ~~`WalshMemoryIndex`~~ | ~~l0 `nt_core_cross_layer.rs` (12)~~ | ~~types (9) / l5 (9)~~ | ~~0.75~~ | ⛔ **本行是错的，已剔除**（见 §8.4） |
| `QuantumSuperposition` | l5 `nt_core_quantum_fusion.rs` (14) | l0 `nt_core_consciousness_types.rs` (10) | 0.71 | `fuse_evidence` `fuse_with` `decoherence_factor` `collapse_evidence` |
| `BoundingBox` | l2 `nt_world/explore/types.rs` (8) | l1 `nt_memory_spatial/types.rs` (6) | 0.75 | `_width_deg` `_height_deg` |

⇒ **A 类也不等于「基准就是真身」**，仍须逐个核实谁是活的。
（`WalshMemoryIndex` 曾被列在这里，**后经核实剔除** —— 见 §8.4。）

**B 类 — J 0.50~0.71，中等共享，可能是真分叉也可能只是相关实现（5 组）**
`SelfTestRegistry`(0.44) `KnowledgeHyperCube`(0.42) `Vec2`(0.42)
`OneObserver`(0.40) `MicCapture`(0.40)

**C 类 — J 0.50，缺 2 个方法，多半是同名不同域（7 组）**
`AuditTrail` `AgentTrajectory` `CrtTimeScale` `FullReasoningState`
`GateDecision` `HyperCoord` `SpecialistModule`

### 8.2 分诊方法论（本节最可复用的部分）

1. **方法集差异只是「候选」，不是「缺陷」**。`Vec2` 同时出现在 ECS 数学库与
   游戏渲染组件里 —— 那是两个合理同名类型。**J=0.40 已是判据下界**，
   再低就是巧合。
2. **「方法最多的那份」不等于真身**。`WalshMemoryIndex` 的实测消费者
   与方法数量**相反** ⇒ 下一轮判断必须以**消费者计数**为准，方法数只用来
   选比较基准。
3. **每个候选都要落到两个计数上**（本轮只对 `IITPhiCalculator` 与
   `WalshMemoryIndex` 做了，其余 14 组**未核实**）：
   - 缺失方法**有没有人调**（`wh_dot`/`wh_inverse`/`wh_transform` 实测各 3 个文件在用
     ⇒ 说明活的是**有这些方法的那份**，即 l0）
   - 每个副本**各有多少消费者**
   两个计数任一为 0 ⇒ 才可能是死副本；两个都 >0 ⇒ 是活的双实现，**必须迁移**。
4. ⛔ **不要用本节的表直接删代码**。§6.2 与 §7 已两次证明：
   「看着是重复」删掉会**破坏 feature 门控**或**删掉一个能力**。

### 8.3 建议的下一步顺序（按「收益/风险」排序）

1. `WalshMemoryIndex` —— 与本会话已做的 `hexagram_hadamard` 同主题，
   且缺的那 3 个方法**实测有 3 个文件在用** ⇒ 收敛收益明确。
2. `QuantumSuperposition` —— J=0.71，l0 是纯接收侧（低层持有高层算法的副本）。
3. `IITPhiCalculator` —— 见 §7，**需先决定 `ios-bridge` 的 ABI 契约**，成本最高。
4. B/C 两类先不动：共享面小，很可能是同名不同域，改动收益低、误伤风险高。

### 8.4 ⛔ 纠正：§8 里 `WalshMemoryIndex` 一行是**错的**（已剔除）

§8 初稿把它列为 A 类「types/l5 各缺 3 个方法」。**核实后不成立**：

* `wh_dot` / `wh_inverse` / `wh_transform` 在 types 与 l5 **都存在**，
  只是**私有** `fn`；我的工具 `impl_methods` 首版**只收 `pub fn`**
  ⇒ 把「**可见性不同**」误报成「**缺方法**」。
* 实际方法数：types **13**（9 pub + 4 私有）、l5 **13**（同）、
  l0 **12**（全 pub）。types 与 l5 **完全一致** ⇒ 不构成分叉。
* 已修工具：新增 `all_visibility` 参数与 `impl_visibility()`，
  「缺方法」与「可见性不同」**分开报告**；修正后该组判据不成立，已从名单消失。

**但核实过程产生了真结论**：types 与 l5 的 `WalshMemoryIndex` 是
**功能等价的两份实现**（11/13 方法逐字相同，2 处差异是 types 侧多一个
`.take(WH_DIM)` —— 而它在当前代码里是**空操作**，因为
`x`/`y` 都由 `vec![0.0; WH_DIM]` 构造）。
⇒ 已把 l5 的 416 行重复实现收敛为 `pub use` 引用 types 真身（提交 `25e02c99`）。

### 8.5 两条新的方法论教训（本轮付出代价换来的）

1. **「有差异」≠「有缺陷」**。我一度想把那个 `.take(WH_DIM)` 报成
   「l5 缺边界保护 ⇒ 潜在 panic」，核实后发现是**空操作** ⇒ **不报**。
   差异存在只说明实现不同，不说明一方有害。
2. **⛔ 消费者计数必须含「重导出链」**。我用路径限定 grep 得出
   「l5 那份 0 消费者」，实际它有 **2 个**消费者
   （`nt_action_facade` / `nt_feel_facade` 都 `pub use` 了它）——
   属正当豁免⑥。⇒ 只搜 `模块::符号` 这种直接形式会**系统性漏掉**
   经 facade 的间接引用，**据此判死代码会误杀**。

⚠️ 附带一条工程纪律：这次收敛里我犯的错误（跨 crate 私有成员不可见导致
**12 个测试编译失败**）**只有 `cargo test` 能抓到**，`cargo check` 抓不到
—— 测试代码只在 test 目标里编译。⇒ 涉及改实现或改可见性时，
`cargo check` 通过**不能**作为完成依据。

---

## 9. ⛔ 新发现的盲区：242 个文件**从未被编译**（含 994 个从不运行的测试）

### 9.1 为什么这重要：它们的「绿灯」是假的
`nt_dup_dead` / `nt_pub_dead` 都基于源码**文本**分析「有什么」，
而这类文件的问题是「**是否被编译**」—— 文本再丰富也不回答这个问题。
⇒ 本轮新增 `nt_mirror_scan --orphan`：从 crate 根（`lib.rs`/`main.rs`）
沿 **mod 声明链**走一遍，走不到的文件即候选。

**规模（实测）**：磁盘 2,762 个 `.rs`，可达 2,550，**孤儿 212 个**，
其中含 **994 个 `#[test]`** —— 这些测试**从未运行过**。

⚠️ 它们在 CI/本地 `cargo test` 输出里**根本不出现**，
所以「12,217 全绿」并不代表它们被验证过。

### 9.2 成因分类（按目录聚合，均为实测）

| 目录 | 文件数 | 说明 |
|---|---|---|
| `l5_cognition/nt_core/multi_agent/`（含 coordinator/ graph_orch/） | 15 | **被独立 crate 取代**：`l5_cognition/mod.rs` 用 `pub use neotrix_multi_agent::multi_agent;`，全仓无 `mod multi_agent` 声明 ⇒ **15 文件 / 4,655 行 / 177 测试**全部从未编译 |
| `l1_action/nt_act/agent_loop/` | 9 | 待核实 |
| `l2_perception/nt_world/temporal_kg/` | 8 | 待核实 |
| `l5_cognition/nt_core/nt_consciousness_core/` | 7 | 待核实 |
| `l4_emotion/nt_memory/{hybrid_retrieval,tiered_memory,decay_forgetting}` | 17 | 待核实 |
| 其他零散 | 156 | 待核实 |

### 9.3 ⛔ 本轮**不删**任何孤儿（本节只取证）

理由：
1. **规模** —— 212 文件 + 994 个测试远超一次清理的合理范围；
2. **可能有正当引用** —— `include!` 宏、feature 门控下的 `mod`、
   外部路径依赖，静态不可判定；
3. **`multi_agent` 那 15 文件**虽然成因已查实（被独立 crate 取代），
   但仍需确认独立 crate **功能完全覆盖**旧目录（4,655 行不是小数目）。

⇒ 处置方式：清单已由工具固化（`nt_mirror_scan --orphan --limit 400`），
可按目录分批逐个核实后处置。

### 9.4 工具实现中踩的四个坑（记录下来，因为都很反直觉）

| 坑 | 现象 | 修法 |
|---|---|---|
| 1 | 候选路径都不存在时**没有 break**，沿用上一轮路径继续拼 | 两个候选都落空即 `continue` |
| 2 | 漏了 Rust 2018 **「同名 .rs 与同名目录并存」**规则（`nt_memory_search.rs` + `nt_memory_search/`） | 非 `mod.rs` 文件内的 `mod`，**优先找同名子目录** |
| 3 | 修坑 2 时把 `lib.rs` 也按「去 `.rs` 得 x_stem」处理 ⇒ 算出 `src/li/`（不存在），可达数 **2187 → 10** | `lib.rs`/`main.rs` 列为**特例** |
| 4 | 漏了 **cargo 自动发现 `src/bin/*.rs`** ⇒ 20 个 bin 全成假孤儿 | 把 bin 目录纳入入口 |

⇒ 这四次都是「**误报量异常大**」暴露的：343 → 242 → 212。
**误报率高就立即查判据，不要靠调阈值掩盖**（§7.5 同源纪律）。
