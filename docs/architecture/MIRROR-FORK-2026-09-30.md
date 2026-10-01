# MIRROR-FORK-2026-09-30 — `neotrix-types` 是 `neotrix-core` 的**冻结旧分叉**

> 触发：`fn_drift` 报出 22 个**跨层**同名副本，`nt_dup_dead` 又判定「重复但不可删」。
> 两个结论互相矛盾 ⇒ 说明「按函数名去重」这个视角本身不够，需要**模块级 + 跨 crate** 视角。
> 顺藤摸下去发现的不是零散重复，是一条**结构性分叉**。
> 纪律：每条数字都是实测，命令可复跑；**本文件不执行删除**，理由见 §5。

---

## 1. 结论（一句话）

**`crates/neotrix-types` 里有一份 `neotrix-core` 的部分镜像，冻结在 2026-07-06，
而真身在 2026-09-17 之后。两份都活着、都编译、都过测试 —— 于是「冗余」从未被看见。**

---

## 2. 铁证：同一函数的两个调用者来自两棵平行子树

`hexagram_hadamard` 的 4 个调用者（编译器解析的边，非 grep）：

```
neotrix[7f32]       l5_cognition::nt_core_walsh::new  → core 的 nt_core_e8
neotrix_types[dbe0] core::nt_core_walsh::new          → types 的 nt_core_e8
```

⇒ `nt_core_walsh` **在两个 crate 里各有一份**，各自依赖自己那棵里的 `hexagram_hadamard`。
这不是「同名函数」，是**两个真身并存**。

---

## 3. 规模【实测】

`scripts/ops/nt_mirror_scan.py`（按**模块名**跨 crate 配对 + 函数体归一化比对）：

| 指标 | 值 |
|---|---|
| 同名模块出现在 ≥2 个 crate 的对数 | **72** |
| 其中函数体重叠 ≥50% 的（疑似真分叉） | **19** |
| 涉及 `neotrix-types` 的镜像对 | **18** |
| types 侧（**全部 added=2026-07-06**）合计 | **185.8 KB** |
| 对侧（较新真身，2026-09-17…09-25）合计 | 251.8 KB |

重叠度最高的几对（逐字相同函数 / 同名函数）：

| 模块 | 重叠 | types 侧 | 真身侧 |
|---|---:|---|---|
| `self_referential` | **100%** (15/15) | 8.8 KB `2026-07-06` | `l6_meta` 9.0 KB `2026-09-17` |
| `vectors_group_a` | **100%** (1/1) | 11.1 KB `2026-07-06` | `l2_perception` 10.0 KB `2026-09-17` |
| `thinking_trace` | 96% (23/24) | 7.5 KB `2026-07-06` | `l6_meta` 7.6 KB `2026-09-17` |
| `weakness` | 96% (22/23) | 14.8 KB `2026-07-06` | `l6_meta` 16.1 KB `2026-09-17` |
| `system_identity` | 94% (15/16) | 6.9 KB `2026-07-06` | `l6_meta` 7.0 KB `2026-09-17` |
| `nt_core_walsh` | 92% (24/26) | 14.1 KB `2026-07-06` | `l5_cognition` 15.0 KB `2026-09-17` |

> 「100% 重叠」不是巧合：`vectors_group_a` 整个文件就是**一个函数**、逐字相同。

---

## 4. 可删性：证据齐了，但**结论是现在别删**

### 4.1 支持「可删」的证据（两条独立路径，都实测）

| 判据 | 结果 |
|---|---|
| 外部是否按**模块路径**引用 types 侧 | `neotrix_types::…$module` **0 命中**（18 个全查） |
| 外部是否按**类型名**引用（路径无关，`AGENTS.md` 警告过的坑） | `neotrix_types::<TypeName>` **0 命中**（55 个公开类型全查） |

⚠️ 第一版统计曾报「外部命中 81/37/44」——**那是假阳性**：匹配到的是
`neotrix-core` 自己的同名模块与同名类型（分叉的另一个面），不是 types 侧。
**裸名统计更不可用**：1,318 次命中里绝大多数是 core 侧同名类型 + `.worktrees/` 副本。
⇒ 这正是 AGENTS.md 那条「删 `pub use` 前用能匹配花括号的形式查消费方」的现实版本。

### 4.2 但删除会**级联**，且方向是架构决策

18 个冻结模块**全部**在 `neotrix-types` 内部有依赖者（实测清单）：

| 模块 | 内部依赖者 | 依赖者举例 |
|---|---:|---|
| `nt_core_hex` | 5 | `core/mod.rs`、`nt_core_gwt/{resonance,workspace}.rs` |
| `nt_core_graph` / `nt_core_walsh` | 4 / 3 | `core/mod.rs`、`nt_core_bank/bank/bank_impl/*` |
| `archive` / `offload` / `thinking_trace` / `weakness` | 3 | `nt_core_self/mod.rs`、`nt_core_meta/*`、`nt_core_bank/*` |
| `scanner` / `reasoning_strategy` / `vectors_group_a` | 2 | `nt_core_knowledge/{types,sources}.rs`、`skill_crystal.rs` |
| 其余 8 个 | 1 | 各自一个 `mod.rs` |

⇒ 动一个就要改 `core/mod.rs` 等 6 个 `mod.rs`，**约 25 个文件 / 186 KB**。

### 4.3 ⛔ 本轮**不执行**删除，三条理由

1. **依赖方向决定「谁能用谁」**：`neotrix-core` 依赖 `neotrix-types`（`Cargo.toml:97`），
   所以 **types 无法依赖 core**。于是「让 types 删掉、core 保留」对 `nt_core_walsh`
   这类**types 内部消费者**不成立 —— 它需要 `hexagram_hadamard`，而那份在 core 里。
   ⇒ 真正的收敛方向是「**types 只保留自己内部需要的那几个，其余由 core re-export**」，
   这是一次**有产品含义的重构**，不是清理。
2. **真身方向是 owner 决策**：也可能反着来 —— 让 core 改从 types 取（core 依赖 types，合法），
   从而 types 成为唯一真身。两条路都成立，**判据是意图不是文本相似度**。
3. **共享工作树 + 结构改动纪律**：`AGENTS.md` 要求结构性改动
   `cargo clean && cargo build` **跑两遍**；当前有其他窗口在并行构建
   （`apps/neobot-desktop`、`crates/neotrix-neobot` 均有未提交 WIP），
   此刻起 186 KB 级联删除 + 双 clean 极可能与他窗互撞且无法干净验证。

⇒ **本轮交付的是「决策所需的全部证据 + 可执行计划」，不是删除动作。**
这与上轮 `nt_dup_dead` 的「0 条可删」不矛盾：那轮证据只够判「不可据零入边删」；
这轮证据够判「这 185.8 KB 是冻结旧分叉」，但**仍不够替 owner 决定收敛方向**。

---

## 5. 可执行计划（下一轮/owner 裁决后照做）

**前置**：裁决 A 或 B
- **A（推荐）** core 是唯一真身 ⇒ types 内部只保留确有内部消费者的少数模块，
  其余删除；core 侧不动。收益：types 回到「低层原语」定位。
- **B** types 是唯一真身 ⇒ core 改 `use neotrix_types::…`，删除 core 侧同名模块。
  收益：单点真身且低层化。风险：触及 L2/L5/L6 的 40+ 文件，层依赖门需复核。

**执行清单（两条路共用）**
1. `python3 scripts/ops/nt_mirror_scan.py --min-overlap 0.5` 冻结清单（工具已在库）
2. 按 §4.2 表格逐模块列出内部依赖者，逐个确认是「仅 mod.rs 声明」还是「真实使用」
3. 先删**依赖者只有 mod.rs 声明**的 8 个（context_window / intrinsic_motivation /
   metacognition_loop / metacognitive_evaluator / self_referential / silicon_self /
   system_identity / vsa_holon，合计约 68 KB）
4. `cargo clean && cargo build` **跑两遍**；再 `cargo test -p neotrix --lib`
5. `bash scripts/check-layer-deps.sh --strict`（现有 19 条已知债不得增加）
6. `python3 scripts/ops/nt_mirror_scan.py --min-overlap 0.5` 复跑，镜像对数应下降

**每步的判据**：`cargo build` 两遍都 0 error 且测试数不低于删除前 −（删除文件的测试数）。

---

## 6. 复跑清单

```assert
file:scripts/ops/nt_mirror_scan.py                      # 跨 crate 模块级镜像审计
cmd:python3 scripts/ops/nt_mirror_scan.py selftest      # 自证 4 例（含 2 例证伪）
cmd:python3 scripts/ops/nt_mirror_scan.py --min-overlap 0.5   # 清单可复跑（数字见 §3）
```

⚠️ 本文件**不含**删除断言 —— 因为删除尚未发生。
若将来执行 §5，必须把「镜像对数下降」写成 `cmd:` 断言，让台账自己盯住进度。