# 吸收批次：18 个外部仓（2026-10-07）

> **入参**：18 条裸 URL，分 3 批粘贴（10 + 1 + 7）。无其他说明文字。
> **依据**：`NEOTRIX-STD-1.0.md` **NTS-B10**（B10.1–B10.4）· 操作面 `skills/external-absorption/SKILL.md` §URL-only 熔炼模式。
> **⛔ 本批未取任何逐字代码。** 取设计形状，取信号字段，取机制判据。
> **原始四字段矩阵**（子代理产出，主 agent 已按 R-P16 复核）落 `notes/absorption-2026-10-07-mem-{A,B,C}.md`。
> ⚠️ `notes/` **被 gitignore**（`.gitignore:20`）⇒ 那三份是**过程件**，本篇才是**入库件**。

---

## 0. 信号初筛 —— ⭐ 本批最有价值的一张表

【实测】`raw.githubusercontent.com` 逐仓 × {main,master} × {LICENSE,LICENSE.md,LICENSE.txt,COPYING} 取首行：

| 许可档 | 仓 | 数 |
|---|---|---|
| ✅ **Apache-2.0** | `mem0ai/mem0` `getzep/graphiti` `topoteretes/cognee` `NevaMind-AI/memU` `CaviraOSS/OpenMemory` `letta-ai/letta` `letta-ai/letta-code` `olow304/memvid` `ix-infrastructure/Ix` `elstongun/leviathan` `tester-army/e2e` `TauricResearch/TradingAgents` | **12** |
| ✅ **MIT** | `vectorize-io/hindsight` `alchaincyf/huashu-art-motion` | **2** |
| ⛔ **AGPL-3.0**（代码层否决） | `volcengine/OpenViking` `CarterPerez-dev/Cybersecurity-Projects` | **2** |
| ⛔ **非商用** | `NanmiCoder/MediaCrawler`（`NON-COMMERCIAL LEARNING LICENSE 1.1`） | **1** |
| ⛔ **无 LICENSE** | `vectorize-io/agent-memory-benchmark`（4 分支 × 4 文件名全 404） | **1** |

⇒ **14 宽松 / 4 受限或无**。

⭐⭐ **本批暴露两件与旧台账相反的事**：

1. **`OpenViking` 是 AGPL-3.0。** 而 `neotrix-core/.../context_fs.rs` 的文件头**自述
   「基于 OpenViking 模式」。⇒ **一个 AGPL 源的名字，已经写进了我方生产代码的注释。**
   这是记录一致性问题（`scripts/check-license.sh` 管的就是这个），
   **不是**「我方抄了它」—— 抄的是模式（虚拟 FS），模式不受版权保护。
   ⇒ **该做的不是删代码，是让记录说清「模式来源 + 未取代码」。**
2. **GitHub API 匿名限流 403** ⇒ 许可核实**不能只走 API**。
   【实测】`raw.githubusercontent.com` 全程可用 ⇒ **许可判定的主路径必须是 raw，不是 API。**

---

## 1. ⭐ 复核推翻了一条子代理判定（R-SCAN-1：先读现场）

子代理 B 报：`MediaCrawler` 的「断点续爬」映射到 `nt_crawl_sources.rs:556-567`，
判 **new**，理由「只有局部 `cursor` 变量，无持久化 checkpoint」。

**主 agent 读现场后推翻**：

```rust
// nt_crawl_sources.rs:553-583  run_hf_queue_batch
let mut cursor = 0usize;                                    // :556
loop {
    if ok + fail >= max_items || cursor >= max_items * 4 { break; }   // :558
    let item = store::claim_hf_pending_url(conn)              // :561 ← 从 DB 领取 pending
    cursor += 1;                                             // :567
    ... store::mark_crawl_complete(conn, &item.id, ...)      // :571/:578 ← 逐条落完成位
}
```

⇒ `cursor` 是**单次调用的尝试预算**（`max_items * 4` = 防毒丸条目死循环），
**不是 resume 指针**；而**持久化 resume 早已存在** —— 就是队列表的 `pending/completed` 位
（`claim_hf_pending_url` 领 + `mark_crawl_complete` 落）。
⇒ **正确判定是「强化」，且这段代码本就正确。**

⭐ 这是本批最该记的一条：**按判定表去改，会把 bug 修进正确代码。**
与 `ABSORPTION-PRECONDITION-GATE` §二 同型 —— **用一个不完整的检索直接跳到「所以该做 X」。**

---

## 2. 熔炼五段

```
0 信号初筛 → 1 零克隆取源 → 2 熔炼成束 → 3 化为已有 → 4 落账
```

| 段 | 本批实测 |
|---|---|
| **0 信号初筛** | ✅ 本篇 §0 即成品。⭐ **主路径 = raw，非 API**（API 403） |
| **1 零克隆取源** | ✅ raw README，18/18 可达，**0 BLOCKED**，无一次 `git clone` |
| **2 熔炼成束** | ✅ 复用 `l2_perception/nt_core_code_search.rs` 的 `context_bundle()`；依赖清单维仍缺（见 §4） |
| **3 化为已有** | ✅ 全部落既有模块，**零新建平行模块**（NTS-B10 合规） |
| **4 落账** | ✅ 本篇入库 |

---

## 3. 判定总账

| 批 | 强化 | 新增 | 降级/ledger intake | 说明 |
|---|---|---|---|---|
| A（6 记忆系统） | 34 | 5 | — | mem0 / graphiti / cognee / hindsight / memU+LongMemory / OpenMemory |
| B（6 agent-memory 运行时） | 18 | 9 | 4 | letta ×2 / OpenViking / AMB / memvid / MediaCrawler |
| C（6 异构） | 20 | **0** | 4 | Ix / leviathan / CyberSec / e2e / TradingAgents / art-motion |
| **合计** | **72** | **14** | **8** | |

⭐⭐ **72 : 14 —— 四分之三的外部「机制」我方早已有。**
NeoTrix 的记忆层（`consolidation/ distillation/ entity_linking/ hybrid_retrieval/
add_only_writes/ decay_forgetting/ evidence_ledger/ coverage_ledger/`）比这批**多数**源**更深**。

⇒ **本批的真实产出不是 14 个新能力，而是：**
1. §0 的**许可分档表**（含两条与旧台账相反的发现）；
2. §1 的**一条被推翻的判定**（防一次错误修复）；
3. **一批 intake**（记而不做）。

---

## 4. 14 条「新增」的接线裁决（Cycle 1201 三选一）

⭐ **本 session 一条都没接线。** 理由不是「不方便」：

| # | 机制 | 源 | 映射落点 | 裁决 |
|---|---|---|---|---|
| 1 | LoCoMo/LongMemEval 准确率评测台 | mem0 | `benches/memory_bench.rs`（现仅 Criterion 吞吐） | 📋 intake |
| 2 | COGX 厂商中立交换格式 | cognee | `nt_resource_ingester.rs` | 📋 intake |
| 3 | mental model / knowledge page | hindsight | `distillation/persist.rs` | 📋 intake |
| 4 | disposition 倾向（怀疑/字面/共情） | hindsight | `selective_memory.rs` | 📋 intake |
| 5 | 原生文字（非拉丁）实体 | hindsight | `entity_linking/extractor.rs` | 📋 intake |
| 6 | L0/L1/L2 预读摘要门 | OpenViking ⛔AGPL | `context_fs.rs` | 📋 intake（仅设计） |
| 7 | 目录作用域语义检索（`find` vs `search`） | OpenViking ⛔AGPL | `fusion_engine.rs` | 📋 intake（仅设计） |
| 8 | 检索/生成**分阶段计时** + must/must-not 信念集 | AMB ⛔无LICENSE | `tiered_memory/traits.rs` | 📋 intake（仅度量定义） |
| 9 | 嵌入模型**持久绑定** + `ModelMismatch` fail-fast | memvid | `vector_index.rs` | 📋 intake |
| 10 | 编解码器自动升级 | memvid | `tiered_pipeline/mod.rs` | 📋 intake |
| 11 | 断点续爬持久 checkpoint | MediaCrawler ⛔非商用 | `nt_crawl_sources.rs` | ❌ **撤回 —— 见 §1，本就已有** |

（余 3 条见 `notes/…-B.md`；同裁决。）

**为什么全列 📋 而不是 ✅ 接线** —— 三条实体依据：

1. **R-P79 的判据是「有活调用点」，不是「有文件」。** #1/#9/#10 若只加字段/加枚举
   而无读取方，就是本仓正在治理的**零读点开关**病
   （见 commit `9d7bdfa0`「功能早就在跑，开关却零读点」、`CLAIMED-BUT-NOT-ENFORCED-2026-10-05.md`）。
2. **#6/#7 动的是 `context_fs.rs` 与 `fusion_engine.rs`**，属结构性改动
   ⇒ 按硬规则须 `cargo clean && cargo build` **跑两遍**取真实错误数，
   本会话不具备该预算。
3. **8 条 intake 与 14 条新增全部记入演化账本**（NTS-B10.2）⇒ **不丢弃**。

⭐ **诚实结论**：本批 94 条机制里，**我方真正欠的只有 3–4 个**，且都不是「照抄能补」的 ——
要么需要评测台（工程量），要么需要依赖面核实（见 `ABSORPTION-GITHUB-SKILL-FORGE` §4），
要么需要预算。

---

## 5. 规则改动（本会话产出）

| 文件 | 改什么 |
|---|---|
| `docs/standards/NEOTRIX-STD-1.0.md` | **NTS-B10** 增 B10.1 URL-only / B10.2 熔炼化为已有 / B10.3 前置门降级为分流 / B10.4 记录真伪唯一硬停 |
| `skills/external-absorption/SKILL.md` | 增「默认路径：URL-only 熔炼模式」+ 裸 URL /「熔炼」触发词 |
| `docs/architecture/ABSORPTION-PRECONDITION-GATE-2026-10-03.md` | 增「已被 NTS-B10.3 改判」指针（原文保留） |
| `docs/architecture/ABSORPTION-GITHUB-SKILL-FORGE-2026-10-07.md` | forge 本体吸收记录（5 条全强化） |
| **本篇** | 18 仓批次入库 |

---

## 6. 方法论沉淀（三条）

1. ⭐⭐ **许可核实的主路径是 `raw.githubusercontent.com`，不是 API。**
   API 匿名 403 时若就此下「许可未知」，会把 **4/18** 的源误归为「无许可 = 不可用」；
   而 raw 全程可用。⇒ 顺序：**raw → API 交叉 → 才判未知**。
2. ⭐⭐ **`context_fs.rs` 文件头自述「基于 OpenViking 模式」，而 OpenViking 是 AGPL-3.0。**
   ⇒ 「模式来源」与「代码来源」**必须分开记**：模式不受版权保护，但记录若含糊，
   下一个读者会以为我方 vendor 了 AGPL。**门开着，但门框上要写清谁从哪儿来**（NTS-B10.4）。
3. ⭐ **72:14 的比例，本身就是吸收成熟度的度量。**
   若某批「新增」占比畸高，先怀疑**检索不全**，再怀疑**外部真有新东西** ——
   本批就有 1 条 14 分之 1 的「新增」被读现场推翻（§1）。
