# Handoff — 晶体核心基础健康修复（2026-09-28 第 3 次会话）

> 任务：过滤重复 + 自主发现问题即时修正 + 保持链路健康高效。
> 承接 `handoff-medical-absorb-20260928.md`、`handoff-mimo-rlenv-absorb-20260928.md`。
> **已完成**。前两份 handoff 的「遗留 1/2」在本轮结清。

---

## 0. 一句话

前两轮遗留的「1,645 个重号 M-id」**不是数据冗余，是 id 碰撞在吃掉不同记忆**。
本轮查清根因、修掉数据、堵住复发路径，并把另外 3 个同源缺陷一并修掉。

## 1. 事故定性：0 组内容相同

逐组比对 1,645 个重号组：**内容完全相同的组 = 0**。每组里都是**不同的记忆**
共用一个 id（如 `M-000003` 同时是 arxiv 论文、itch.io 素材、x.com 推文、
github 仓库、kaggle 数据集五完全不同条目）。

杀伤路径：`sync_to_consciousness`（`cocoons.rs:297-310`）按
`memories.insert(id, mem)` 逐条插入，后到覆盖先到；而 `self.cocoons` 是
**HashMap，迭代序不确定** → 每组里哪条存活是**随机的**。

> 结果：64,720 条落盘，只有 62,040 条真正对运行时可见 —— 2,680 条记忆在
> 随机幸存者的阴影下永久不可达。

## 2. 两处根因

1. **id 分配无互斥**：多个吸收脚本各自独立 `fast_max_mid(COCOONS)` 算 start id，
   互不加锁。涉事 8 个茧的时间戳集中在 `1790303525..1790303837`（**5 分钟内**）。
2. **id 空间臆造**：`nt_hf_smelt_chains.py` 的 `read_base()` 在
   `archive.mids.json` 缺失时**静默回退硬编码 477231 / 479262**。那个高位空间
   在活库里从不存在（活库用 M-000001.. 连续低空间）→ **767 条 connections
   永久悬空**。且 `.mids.json` 现已不在，回退成了唯一路径 = 每次跑都必坏。

## 3. 修了什么（数据 + 复发路径 + 顺带发现）

| # | 问题 | 规模 | 处置 |
|---|---|---|---|
| 1 | M-id 碰撞遮蔽不同记忆 | 1,645 id / 2,680 条 | 重编号为 M-062972..M-065651（首次出现者保持原 id，对外引用语义不变） |
| 2 | 悬空连边 | 3,201 边 / 771 目标 | 删除（目标本身无内容，删不丢信息） |
| 3 | **内容级完全重复** | 42 组 / 46 份 | 折叠保留首现（7 字段全同 + 入边 0，逐份核验） |
| 4 | **茧超 retention 上限** | 2 个（1645 / 1006 > 1000） | 切分为 `{cid}-p1` |
| 5 | `splice()` 产出非规范逗号 | 127 处 `}\n  ,\n` | 修 `splice()` rstrip + 规范化存量 |
| 6 | `read_base()` 臆造 id 空间 | 767 条悬空的源头 | 改为缺失即 `SystemExit`，**不再回退硬编码** |
| 7 | 写入无碰撞闸 | — | 新增 `guard_id_collision()`，接入 base 供所有写者继承 |
| 8 | `_mid_state.json` 陈旧 | max_mid 62971 | 刷新为真实值（否则回滚会命中缓存 → 撞我新编号） |

### 第 4 项是功能缺陷，不是美观问题

`store_memory`（`cocoons.rs:132-141`）满容量直接
`return Err("Cocoon memory capacity reached")`。故 `jev-builds`（1645）与
`github-smelt`（1006）这两个域**永久写入拒收** —— 新记忆加不进去。
切分后最大茧 = 1000，与全库分片口径一致。

### 第 3 项逐份核验

46 份全同副本**逐份比对 7 个字段**（content/memory_type/domain/connections/
confidence/strength/importance）全同，且入边 0。其中 2 份属医患集，是**源数据
重复**（同一疾病的 `related_drugs` 把同一药列了两次，如
`[drug] Tetanus: METRONIDAZOLE ...`），折叠无损。计数对账：
56,297 蒸馏 − 2 折叠 = **56,295** ✓（无回滚/重跑残留）。

## 4. 复发路径已堵

`guard_id_collision()` 加在 `nt_hf_digest_to_cocoons.py`（共享写入原语），
两个蒸馏器接入，提交前比对盘上已有 id，撞号即 `SystemExit`。

> **闸门自身先测过**：初版 `MID_RE` 只捕获数字（`scan_max_mid` 要
> `int(group(1))`），导致 `want={"M-000003"}` 与 `have={"000003"}` 永不相交
> —— **闸门永不触发**。假安全感比没闸更坏（与 AGENTS.md R-SCAN-3 同理）。
> 已修并双向实测（撞号必拦 / 全新号放行 / 空输入不误报），且钉进 base selftest。

## 5. 顺带修的解析器缺陷（读输出读出来的）

**验收器自己有 bug**：`mems` 用 `setdefault`（首现）而 `--before` 侧用 dict
推导（末现），把既存重号 id 判成「存量被改写 1645」。两边统一 first-wins 后
才得真实结论（数据本来没问题，是尺子坏了 —— 尺子坏了比数据坏更隐蔽）。

## 6. 当前总账（全部不变量为 0）

```
memories=64,674   unique=64,674   shadowed=0
内容重复组=0   悬空连边=0   非法 memory_type=0   非 M-id 连边=0
超限茧=0   茧键≠体内 id=0   空内容=0   splice 瑕疵=0
connected=55,051 (85.1%)    51.8MB / 205 茧
canonical roundtrip 字节一致 = True
```

`connected_ratio` 由 87.5% 降到 85.1% 是**变好**：删掉的是「连了个不存在的东西」
的假信号，原数值被 3,201 条死边虚高。

**规范往返字节一致 = True** 是本轮最重要的结构性副产品：修复前该往返有 127 处
差异，导致任何结构化改写都无法证明无损；现在 parse→dump 可字节还原，
后续任何健康修复都能被严格验证。

## 7. 工具

| 脚本 | 作用 |
|---|---|
| `nt_cocoons_dedupe_ids.py` | 四步健康修复（重编号 / 折叠内容重复 / 删悬空 / 拆超限），各自可关 |
| `nt_cocoons_verify_absorb.py` | 通用验收器（按 domain 前缀） |
| `nt_cocoons_repair_memory_type.py` | 毒记忆修复（上一轮） |
| `nt_hf_digest_to_cocoons.py` | 共享写入原语 —— 本轮修了 `splice()` 并加碰撞闸 |

全链 10 个脚本 selftest 绿。

## 8. 遗留（**唯一**一项）

**Rust 侧加载实证仍缺**。`nt_mem_gate.sh` 本轮全程 BLOCKED
（free_pages 20248~76802 < 100000，swap 1.0~1.3G），按 AGENTS.md 未起任何 cargo。

现在这条验证比之前更值钱：全库 0 悬空 / 0 重号 / 0 非法类型，**数据侧已无可指摘**，
剩下的就是确认 `CocoonStore::load()` 能真吃下 51.8MB。门开后跑任一走
`CocoonStore::load()` 的 bin，确认 `stats().total_memories == 64674`
（若为 0 即复现 CrossDomain 静默归零）。

```bash
sh scripts/ops/nt_mem_gate.sh && \
cargo run -p neotrix --bin nt-train-export -- --help
```

## 9. 门记录（本轮实测，非沿用旧值）

- `nt_lock_audit.py neotrix-core/src` → **0 处可疑**（2026-09-28）
- `nt_mem_gate.sh` → **BLOCKED**（2026-09-28 多次实测，free_pages 20248~76802）
- `nt_sidecar.sh status` → DOWN
- 未触碰他窗改动

## 10. 复现

```bash
python3 scripts/ops/nt_cocoons_dedupe_ids.py --selftest
python3 scripts/ops/nt_hf_digest_to_cocoons.py --selftest   # 含闸门双向实测
python3 scripts/ops/nt_cocoons_dedupe_ids.py --dry-run      # 已修复后应全为 0
python3 scripts/ops/nt_cocoons_verify_absorb.py --prefix rl-
python3 scripts/ops/nt_cocoons_verify_absorb.py --prefix medical- --conf-cap 0.75
```


---

## 11. 追加（2026-09-28 第 4 次会话）：Rust 侧加载实证**已结清**

前几轮唯一遗留项（`CocoonStore::load()` 能否真吃下活库）在本轮**由二进制实证**：

```
[live] cocoons=205 memories=64674 unique=64674 total_recalled=64674
[live] edges=63302 isolated=7416 hubs(in>=8)=2212 cross_domain_edges=61098 domains=22
[live] connected_ratio=0.885
[live] anchor=M-005702 in_deg=39 cluster=39 domains={medical-clinical, medical-consult,
       medical-disease, medical-literature, medical-pharma} cross=true
test result: ok. 1 passed
```

来源：`neotrix-core/src/neotrix/nt_crystal_core/nt_graph_index.rs::live::live_graph_index`
（`#[ignore]`，显式 `--ignored --nocapture` 跑）。

**两条被此前静态分析断言、现由二进制证实/证伪的结论**：

1. **CrossDomain 静默归零地确已解除**：`memories == unique == 64674`
   = Rust `HashMap` 看到的条数与盘上条数**完全一致**。修复前是
   64,720 落盘 / 62,040 入 HashMap（2,680 条被遮蔽）。去重在运行时层得到确认。
2. **「max degree = 2」是错的**（我当时只量了**出度**）。**总度**实测：
   有一个 **935** 的巨型枢纽，2,212 个节点入度 ≥8，connected_ratio **0.885**。
   图不是「均匀稀疏」，而是**枢纽-辐条**结构。这反而对前提选择**有利**
   （枢纽是丰富的候选源）。此前「结构上近乎无用」的判断一并撤回。

## 12. 追加：发现一处**我自己造的** monoculture 枢纽

总度 935 的头号枢纽 `M-062005` = `[verifier] 判分器工程` 锚点 ——
**925 份 rl-rubric 全部指向它**。这是我上一轮 RL 蒸馏时连线口径造成的：
每份 rubric 都连了「族锚点 + 判分锚点」，于是判分锚点吞掉整个 rl 域。

后果：若前提选择从该枢纽走入邻域，会拿到 925 份**彼此毫无语义关联**的
环境 rubric（925 个不同疾病族）→ 产出「前提同指率」极低的垃圾链。

**已加的对冲**（`nt_graph_index.rs`）：
- `specificity(target) = 1/ln(1+入度)` —— IDF 式特异度，泛化枢纽排后
- `mega_hubs(k)` —— 诊断用，直接暴露 monoculture
- `facet_cluster` 域内按「谁指向我」的入邻域升序（专属者优先），确定性保持

**未做（需你定夺）**：是否从**源头**改 RL 蒸馏器的连线口径（削掉
「每份 rubric → 判分锚点」这条边，只留族锚点）。数据侧改动 = 一次重灌
（`.bak.rlenv` 备份在）；代码侧改动 = 改 `nt_rlenv_distill_to_cocoons.py` 的 conns。

---

## 13. 第 4 次会话续：monoculture 根治 + 图级体检工具（Rust 编译因他窗阻塞）

### 13.1 monoculture 已从**源头**根治

| 位置 | 动作 |
|---|---|
| **源头** `nt_rlenv_distill_to_cocoons.py` | rubric 连线 `[族锚点, 判分锚点]` → **只连族锚点**。干跑实测最大入度 **935 → 157** |
| **存量** 新脚本 `nt_cocoons_prune_monoculture.py` | 定点重定向 925 条边（**不回滚** —— `.bak.rlenv` 是 RL 吸收前快照，回滚会连带撤销三轮健康修复） |
| **回归闸** `nt_graph_index.rs` 活库测试 | `mega_hubs(400)` 断言，monoculture 复活即红。阈值 400 远高于任何语义正确枢纽（实测 171）、远低于曾经的 935 |

**二进制确认**：`widest_hub=M-018218 in_degree=171`、`mega_hubs(>=400)=[]`、
`M-062005`（判分锚点）入度 935 → **10**（正好是那 10 条 `[verifier-pattern]`）、
925 份 rubric 族不匹配 = 0、跨域边 61,098 → 60,173（跨域性未损）。

### 13.2 新增图级体检（零构建，与 Rust 互为交叉验证）

`scripts/ops/nt_graph_audit.py` —— 记忆级验收**发现不了**图级坏掉：
monoculture 下每条记忆单独看都合法，但链会静默变垃圾。

Python 与 Rust 两套独立实现口径一致，数字**逐项吻合**：
`64674 / edges 62377 / cross_domain 60173 / isolated 7406 / ratio 0.885 / max入度 171`。
两边对不上即有一边算错。当前 **VERDICT: PASS**（内容重复 0 / 非法类型 0 / 悬空 0 /
非 M-id 0 / 超限茧 0 / 键不符 0 / 规范往返字节一致）。

### 13.3 源码层静态闸（防同类复发）

`--scan-scripts` 扫 ops 源码里**超过活库 max** 的硬编码 M-id
（思路同 `nt_lock_audit.py`，零构建）。这类字面量是 3,201 条悬挂边的源头：
一跑就重新注入，而记忆级验收看不见。

实测 **11 处**（已滤掉注释/文档字符串/自测假值，5 个假阳性）：

```
M-477231  nt_hf_smelt_chains.py:5      M-477231  nt_ascend_engine.py:94
M-479262  nt_hf_smelt_chains.py:6      M-479512  nt_foundation_laws.py:6
M-479531/479532  nt_foundation_laws.py:11
M-479535  nt_foundation_laws.py:1487   M-479628  nt_web_mine.py:55
M-479942  nt_foundation_laws.py:978
```

**未擅自改这 6 个脚本**（越界 + 可能与他窗冲突），列为待办：
它们的 id 全部指向活库**不存在**的 477k–479k 外来空间（活库 max = M-065651）。
另：`nt_corpus_mine.py:32` 的 `BASE_MID = 479512` 是**裸数字**无 `M-` 前缀，
当前扫描器抓不到，属已知盲区。

### 13.4 步 2（内容去重进代码）代码已写，**编译待他窗**

`sync_from_consciousness_counted()` + `SyncReport`（去重闸可观测，不静默）已落
`cocoons.rs`，5 个单测就位。**但 `-p neotrix` 依赖 `neotrix-neobot`，
他窗正在改 `nt_types.rs`（新增 `ToolName::ReadImage` 未补 match），编译不过。**
已用 `rustfmt --check` 独立确认我的两个文件格式/语法干净（零依赖检查）。
门开且他窗收尾后跑：`cargo test -p neotrix --lib -- sync_dedup_tests`。
