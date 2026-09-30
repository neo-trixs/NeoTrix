# DECOMPOSE-PARITY-2026-09-30 — 原子级拆解 + 对方产品对位（G1/G4 下游 · G3 对位）

> 触发：`CAPABILITY-GAP-2026-09-30` §1.2 实测「**吸收与复现能力：只有文档，没有工具**」
> 且「**没有能力矩阵 / 差距表**（可比对的 spec）」⇒ 「原子级拆解技术逻辑」与
> 「复现他方产品」两项目标都无法工具化。
> 本文件记录：工具落地 + **第一次真实跨仓对位** + 它抓到的第一个真实架构缺口。
> 纪律：每条判断标注【实测】并给可复跑命令；不可判的写「不可判」，不猜。

---

## 0. 一句话结论

**「我方有他方的能力吗」从此是一条命令。** 首次实跑即抓到一个**功能等价但算法劣势**
的架构缺口：我们的 LRU 缓存淘汰是**每次插入 O(capacity) 全扫描**，对方产品是 O(1)。

---

## 1. 落地物

| 项 | 值 |
|---|---|
| 工具 | `scripts/ops/nt_decompose.py`（468 行，纯标准库，无网络，无第三方代码执行） |
| 子命令 | `atoms` / `parity` / `selftest` |
| 自证 | `python3 scripts/ops/nt_decompose.py selftest` → **6 正例 + 3 证伪用例全绿** |
| 注册 | `.neotrix/task-index.json` → `id: nt-decompose-atoms`、`nt-decompose-parity` |
| 边表输入 | `.project-map/edges-*.jsonl`（THIR 委托编译器，见 `nt_calledges.py`） |

### 1.1 原子单元的判据（可证伪，且全部来自边表/路径，不猜语义）

| oracle | 判据（可测量） | 含义 |
|---|---|---|
| `K1-atomic-leaf` | callee 属 `core/std/alloc/proc_macro/test` **且**边表内出度 0 | 原子单元，无我方逻辑 |
| `K2-persistent` | callee 命中落盘词表（write/save/persist/store/append/commit/insert/put_/sync_all…） | 需持久化裁决 |
| `K3-symbol-bound` | 其余：THIR 已解析的 `FnDef(DefId)` 边 | 符号绑定已证，语义不可从边表判定 |
| `K4-external-decidable` | span 落在 `tests/`/`benches/`/`scripts/`/`.github/` | **已有外部裁决点** |
| `unjudgeable` | 缺 span 等证据不足 | 显式留空，不猜 |

### 1.2 三条硬纪律（写进 docstring，且有证伪用例守着）

1. **宁缺勿错**：`--root` 子串命中多个节点 ⇒ **rc=2 并列出候选**，绝不挑一个
   （证伪用例 2：歧义 root 必须报错）。
2. **对位诚实**：`parity` 是**名字级**对位（叶子名归一化后比字符串），
   矩阵头**显式写明**「同名不保证语义等价，异名不代表能力缺失」。
3. **不可达不得混入**：证伪用例 1 断言不在可达子图里的节点**不得**出现在原子集。
4. **矩阵自带判据行**：摘要 `[parity] shared=… only_mine=… only_theirs=…`
   **写进产物文件**（不只 stdout）⇒ 重定向/干跑后仍可机器判。

### 1.3 与既有工具的边界（不重复造轮子的诚实交代）

| 工具 | 干什么 | 与本工具的重叠 |
|---|---|---|
| `nt_callgraph.py --impact/--deps/--unreachable` | 单库可达性**查询**（人读文本） | 与 `atoms` 的遍历**重叠** |
| `nt_decompose.py atoms` | 入口 → **原子记录**（含 file:line + oracle），机器可读 | 本工具独有：原子记录 + oracle 分类 |
| `nt_decompose.py parity` | **两个不同代码库**的原子集对位 | `nt_callgraph` 做不到（它单库） |

⇒ 重叠部分保留理由：`nt_callgraph` 是人读查询，`atoms` 是机器可读契约记录，
且 `atoms` 的输出能被 `parity` 消费。若日后要合并，应先把 `nt_callgraph` 改成
返回结构体（当前是 argv 驱动 + 文本打印），**不在本轮做**（结构改动另跑双 clean）。

---

## 2. 第一次真实跨仓对位【实测】

### 2.1 为什么这个对位不是模拟数据

对方原子集用**同一工具**在**外部第三方 crate** 上生成：

```sh
# 外部源取自本机 cargo registry（已 vendor 的依赖，无需联网、无需执行其代码）
cp -R ~/.cargo/registry/src/*/lru-0.18.5 /tmp/nt-parity/lru-src   # license = MIT（LICENSE 直读）
cd /tmp/nt-parity/lru-src
python3 scripts/ops/nt_calledges.py --crate . --out /tmp/nt-parity/lru-edges.jsonl
#   ⇒ [calledges] 356/356 edges (100%)
python3 scripts/ops/nt_decompose.py atoms --db /tmp/nt-parity/lru-edges.jsonl \
    --root 'lru[479a]::{impl#11}::put' --depth 3 --out /tmp/nt-parity/lru-put.json
```

我方原子集取自**生产代码**（同一个 `nt_calledges` 抽取器）：

```sh
python3 scripts/ops/nt_decompose.py atoms --db .project-map/edges-neotrix-core.jsonl \
    --root 'resilience::nt_policy::{impl#0}::insert' --depth 3 \
    --out .project-map/atoms/ours-response-cache-insert.json
python3 scripts/ops/nt_decompose.py parity \
    --mine .project-map/atoms/ours-response-cache-insert.json \
    --theirs /tmp/nt-parity/lru-put.json \
    --out .project-map/atoms/PARITY-lru-vs-response-cache.md
```

### 2.2 对位结果

`[parity] shared=7 only_mine=6 only_theirs=13`
（我方 13 原子 / 对方 24 原子，均未截断）

| 层分布 | 我方 | 对方 |
|---|---:|---:|
| `l1_action`（我方 L1 网关层） | 13 | 0 |
| `lru`（对方本体） | 0 | 7 |
| `hashbrown` | 0 | 3 |
| `core` / `alloc` | 0 | 12 / 2 |

**共有 7**：`filter` `get_mut` `insert` `len` `map` `new` `remove`
**仅我方 6**：`contains` `finish` `hash` `hash_key` `iter` `min_by_key`
**仅对方 13**：`attach` `detach` `swap` `as_ptr` `replace_or_create_node`
`capturing_put` `into_raw` `assume_init` `cap` `unwrap` `get` `replace`

---

## 3. 它抓到的第一个真实缺口【实测，已读源码逐行确认】

R-SCAN-1b 纪律：**grep/矩阵只给候选行，结论必须读那一行本身。** 已读
`neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/resilience/nt_policy.rs:94-111`。

### 3.1 P0：LRU 淘汰是 O(capacity) 全扫描，对方是 O(1)

我方 `ResponseCache::insert`（满容量时）：

```rust
let lru_key = self.entries.iter()
    .filter(|(k, _)| !self.pinned.contains(k))
    .min_by_key(|(_, (_, t))| *t)     // ← 全表扫描
    .map(|(k, _)| *k);
if let Some(k) = lru_key { self.entries.remove(&k); }
```

原子集里 `min_by_key` + `iter` + `filter` + `remove` **同时出现**，正是这次扫描。
对方 `lru::LruCache::put` 的原子集里出现的是 `attach`/`detach`/`swap`/
`replace_or_create_node` —— 链表式 O(1) 提升（promote）。

⇒ **功能等价，复杂度劣势**：我们的实现在缓存打满后**每次写都扫全表**。
`ResponseCache` 是网关 `resilience` 层的响应缓存，容量上限为 `capacity` 字段
（见同文件 `new`）。⇒ 这是热路径上的持续成本。

**两个候选修法（都需要独立一轮 + 测试，本轮不做）**：
- (a) 引第三方 `lru`（MIT，registry 已有）——最小改动，但引入依赖 + 它的 slab/unsafe；
- (b) 自建 intrusive 链表 ——零依赖，但要手写 `unsafe` ⇒ **本仓 `#![forbid(unsafe_code)]` 直接排除**；
  ⇒ **在本仓约束下 (a) 是唯一可行解**，这本身就是一条架构结论。

**附带发现（P1，doc-claim 债）**：同文件 `pin`/`unpin`/`pinned_count` 三个
doc comment 都写着 `/// Note: Real implementation needed — …`，但函数体是**完整实现**。
⇒ 这是**模板残留的假声明**，`check-doc-claims.sh` 只查「zero consumers」类断言，
**抓不到这一类** ⇒ 已记入 `TODO.md`，作为 doc-claim 门的下一个类目候选。

---

## 4. 本轮没做的（边界声明）

- **没做语义等价判定**：矩阵是名字级。语义等价需给每个原子配外部 oracle
  （K4），本次只有 0 个 K4 命中 ⇒ **当前矩阵只能回答「同名项是否都在」**。
- **没给 parity 建门**：名字级对位会误报（异名 ≠ 缺能力），按 G7 裁决，
  工具输出**不做硬门**，只作报告。
- **没重跑全量测试**：本轮**零 `.rs` 改动**（只加脚本/文档/task-index），
  按 R-SCAN-3 不需要重跑门链；但 `check-feature-gates` 未触发（无 feature 改动）。
- **没删 `/tmp/nt-parity`**：外部源副本在仓外、MIT、非入库资产；
  按 R-DISK-1「只删生成物」原则保留可复跑证据，会话收尾时清理。

---

## 5. 复跑清单（判据可证伪）

```sh
python3 scripts/ops/nt_decompose.py selftest                 # 6 正例 + 3 证伪
python3 scripts/ops/nt_decompose.py atoms --db .project-map/edges-neotrix-neobot.jsonl \
    --root 'nt_channel_serve::run_once' --depth 3             # 193 原子 / 未截断
```

⇒ 若 `selftest` 转红，或 `atoms` 在**同一输入**上给出**不同原子数**，
说明工具坏了，**不是仓库变了**。
