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

## 4. 行为对位：把「复现对方产品」从名字级推到行为级（同日补做）

§0 承诺的目标是「复现他方产品」，而 §2 的矩阵是**名字级**的。当天实测即**自证其局限**：
矩阵把 `get` 报成「仅对方有」，而我方同义方法叫 `cache`（读源码确认 `cache()` 会刷新
时间戳，语义与 `get` 一致）⇒ **名字级矩阵确实会误报**，§1.2 的警告当天就兑现了。
⇒ 补上行为级那一腿。

### 4.1 三件套（单一事实源 = vectors JSON）

| 件 | 路径 | 作用 |
|---|---|---|
| op 脚本 | `.neotrix/parity/response-cache.vectors.json` | 5 个 case / 37 步，**唯一事实源**，两侧都读它 |
| 对方 oracle | `.neotrix/parity/response-cache.vectors.reference.json` | **跑对方实现采集**的逐步观测（`lru@0.18.5`，LICENSE 直读 = MIT） |
| 采集器 | `scripts/ops/nt_parity_ref.py` | 临时 cargo 工程 + `--offline` 跑对方 crate；`--check` = 漂移门 |
| 我方对位 | `neotrix-core/tests/response_cache_parity.rs` | 同一 op 脚本跑 `ResponseCache`，逐步比对 |

**为什么 oracle 不会自证**：reference.json 的每一行都是**对方实现自己吐的**，
不是人手写的期望值。`nt_parity_ref.py --check` 每次都**重跑对方实现**并逐行比对，
说谎即 rc=1。

### 4.2 实跑结论

```
cargo test -p neotrix --test response_cache_parity
  test contains_does_not_disturb_lru_order ... ok
  test response_cache_behaviour_matches_reference_lru ... ok
```

⇒ **37/37 步行为一致**。也就是说 §3 那个 P0 缺口的定性要**改判**：

| 项 | 结论 |
|---|---|
| 语义是否等价 | ✅ **等价**（put/get/len/contains 全子集 37 步零分歧） |
| 复杂度是否等价 | ❌ **不等价**：我方 O(capacity) 扫描 vs 对方 O(1) promote（§3.1） |
| ⇒ 修法优先级 | 属**性能债**而非**正确性债**。P0 降 P1 仍要做（热路径每次写扫全表），但**不是 bug** |

### 4.3 顺带补的一个生产 API（有消费者，非导出）

`ResponseCache::contains(&self, key) -> bool`（`nt_policy.rs`）：非变更式存在性检查，
**故意不刷新时间戳**（否则「问一下在不在」会改变下次淘汰谁）。
消费者 = 对位测试的 `contains` 步骤需要**不扰动顺序**的观测，而对方 `contains` 同样不提升。
配套测试 `contains_does_not_disturb_lru_order` 守着这个语义。

> **该测试第一版是我手推错的**（把 `insert a; insert b` 后的 LRU 写成 `b`）。
> 实跑失败后读实现 + 外部 oracle（case c2 淘汰 `a`）确认**实现是对的、我的期望是错的**，
> 已改期望并把这段写进注释留档。⇒ 又一次「手推 ≠ 实证」。

### 4.4 两条证伪用例（都实测过）

| 证伪 | 做法 | 结果 |
|---|---|---|
| 测试能否发现分歧 | 把 reference 第 13 行 `contains:false` 改成 `true` | ❌ 测试红：`step 13: ours=contains:false reference=contains:true` |
| oracle 能否说谎 | 同上改 + 跑 `--check` | ❌ rc=**1**：`line 13: stored=contains:true live=contains:false` |

### 4.5 本节边界

- 只对位 `put/get/len/contains`；**pinning / hit_count / miss_count 刻意不对位**
  （我方扩展，进则必分歧，那是扩展不是缺陷）。
- 37 步通过**不等于**「行为完全一致」：vectors 没写的语义未被验证。
  （例：`key_for` 的哈希稳定性、`prefetch` 路径都未纳入。）

---

## 5. 共享工作树事故记录（必须留档）

**`519d78b9` 把另一窗口的 `.neotrix/task-index.json` 内容提交进了我的 commit。**
成因链：我在自己工作树加了 3 条索引条目并验证 `nt_find` 能命中；提交前另一窗口
同时改写该文件（加 `nt-callgraph-impact` 等），`git commit --only <path>` 取的是
**工作树状态** ⇒ 我提交了他们的内容，而我的 3 条丢失。
⇒ 与 `sessions/handoff-commit-only-20260929.md` 记录的是**不同**事故：
那次是暂存区核对与提交不原子，这次是**共享单文件本身被并发改写**。
⇒ 教训：`--only` 只隔离「哪些文件」，**不隔离「文件里是什么」**；
共享索引文件（task-index / baseline / layer-map）提交前必须
`stat -f '%Sm'` + 重新 `grep` 自己的条目仍在。

---

## 6. 复跑清单（判据可证伪）

```sh
python3 scripts/ops/nt_decompose.py selftest                 # 6 正例 + 3 证伪
python3 scripts/ops/nt_decompose.py atoms --db .project-map/edges-neotrix-neobot.jsonl \
    --root 'nt_channel_serve::run_once' --depth 3             # 193 原子 / 未截断
python3 scripts/ops/nt_parity_ref.py \
    --vectors .neotrix/parity/response-cache.vectors.json --check   # oracle 未漂移
cargo test -p neotrix --test response_cache_parity                  # 2 绿（含 37 步对位）
```

⇒ 若 `selftest` 转红，或 `atoms` 在**同一输入**上给出**不同原子数**，
说明工具坏了，**不是仓库变了**。
⇒ 若 `--check` rc=1：**先信它**（对方实现变了，或有人手改了 oracle），
再决定是重采还是改 vectors —— 不要直接 `--out` 覆盖掉证据。
