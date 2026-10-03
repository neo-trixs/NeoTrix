# 外部吸收：waku-agent + herdr-gpui（2026-10-03）

> ⛔ **取证边界（先声明）**
> - **`waku-agent`**：`git clone --depth 1` **成功**，一手源码。**MIT**。
>   ⭐ **不是 Rust 项目** —— 318 个 `.py`、1 个 `.ts`。
> - **`herdr-gpui`**：`git clone --depth 1` **成功**，一手源码。**Apache-2.0**。
>   409 个 `.rs`，4 个 crate。
> - ⛔ 本文只写**实测读到的**内容；未读的文件一律不写。

---

## 1. waku-agent —— ⭐⭐⭐ **「失败方向由失败模式决定」**（本文最可迁移的一条）

### 1.1 事实（F1，`waku/memory/slot_gate.py:1-19`，95 行）

waku 的记忆有两道闸（⭐ **两闸的判据完全不同，不是同一件事**）：

| 闸 | 判据 | 环境变量 |
|---|---|---|
| `select` | 检索回来的事实里，**哪些进 prompt** | `WAKU_SLOT_GATE_KEEP` |
| `keep` | consolidation 提议的事实里，**哪些落库** | `WAKU_KEEP_GATE_MIN` |

⭐⭐ 两条 docstring 的关键设计（逐字要点）：

1. **两道闸都 fail-open，且写明理由**
   > 「Off, or any failure — the network, a timeout, an answer without a score —
   > and every fact goes through, exactly as before:
   > ⭐ **a slow or broken judge must never cost a memory**.」

2. ⭐⭐ **判据是反事实 necessity，不是相似度**
   > 「how much **does leaving this one out change the answer**?」

3. ⭐⭐ **阈值自带测量出处**
   > `WAKU_SLOT_GATE_KEEP` (0.5, **measured 12 of 12** on the lab's cases in this shape)

4. ⭐ **rubric 是有序的「后果」等级，不是「相关度」等级**：
   > `Not at all. Irrelevant here.` → `A lot. The answer would be vaguer…`
   > → `Completely. **Without this the request cannot be answered.**`

### 1.2 ⭐⭐⭐ 于是得到本文最可迁移的一条

**fail-open 还是 fail-closed，取决于「失败时损失什么」：**

| 面 | 方向 | 理由 |
|---|---|---|
| ⭐ waku 记忆准入 | **fail-open** | 失败= **丢记忆**（不可恢复）> 留一条无用记忆（可回收） |
| ⭐ NeoTrix SSRF 守卫 | **fail-closed** | 失败= **发出内网请求**（不可撤销）> 拒掉一次请求（可重试） |

⇒ ⭐⭐ **两边都对。**「永远 fail-closed」同样是**未经论证的教条**。
⇒ ⭐ 对 NeoTrix 的直接要求：**任何新增的守卫都必须写明它的失败方向与理由**，
否则读者无法判断这是设计还是疏忽。

### 1.3 ⭐⭐ 第二个可迁移点：**阈值必须自带测量出处**

waku 把「0.5，且在 lab 的这类样本上 12/12 命中」写进常量定义旁的注释。
⭐ **对照本仓的教训**：`LESSONS-20260929-checked-is-not-verified.md` 记着
mu 的 admission/chunk 准入**成本最高（54% token）、收益为零**（2412 块 drop 0 个）。
⇒ ⭐ **差别就在这里**：waku 在模块 docstring 里写
**「Measured in the lab: about 260 ms and $0.02 per 1000 decisions」**（`jev.py:11-12`），
把**单位决策成本**与代码放在一起 ⇒ **下一个人的第一眼就能判断值不值。**
⇒ ⭐ 对 NeoTrix 的要求：**任何带成本的准入/准入式机制，docstring 必须写单位成本。**

### 1.4 `retrieval_gate.py` —— ⭐⭐ 「HERO MOMENT #1」是**要不要检索**

其 docstring 给出的论证值得抄：
> 「Default-on retrieval is (a) slow — an extra search before every reply — and
> (b) worse: **irrelevant memories bias the answer ("over-interpretation")**」

⇒ 判据：「does **THIS** message need the user's memory?」，并**顺带产出检索 query**。
⭐⭐ ⭐ **这是 NeoTrix 的一个真实空白**：本仓 `hybrid_retrieval` 是**默认检索**
（`neotrix-core/src/l4_emotion/nt_memory/hybrid_retrieval/mod.rs`），
⭐ **但没有任何地方问过「这条消息需要记忆吗」**。
⇒ 这是一条**独立的**、与能力树平行的候选改进（检索准入闸）。

### 1.5 ⭐ `jev.py` 的两个工程约束（可直接抄）

- ⭐ 「the key comes from the **environment only**, so nothing here reads a file」
  ⇒ ⭐ **密钥只从环境变量取，代码里不读文件**（比「读配置文件」更小攻击面）。
- ⭐ 「any failure **raises** instead of **exiting**, so the caller can **fail open**」
  ⇒ ⭐ **库不 `sys.exit`**，把失败判定权交还调用方（与 §1.2 的 fail-open 配套）。

---

## 2. herdr-gpui —— ⭐⭐ 协议 crate 的**依赖纯度**与 **surface 协议分层**

### 2.1 事实（F2，`crates/herdr-protocol/`，1473 行）

4 个 crate：`herdr-client` / `herdr-gpui` / `herdr-protocol` / `test-support`。

⭐⭐ **`herdr-protocol` 的依赖只有 5 个**：
`base64` / `bincode` / `serde` / `serde_json` / `thiserror`
⇒ ⭐ **零 UI 依赖、零框架依赖。**

模块划分：
| 模块 | 行数 | 职责 |
|---|---|---|
| `wire.rs` | 697 | ⭐ 线协议（帧/握手/版本） |
| `surface_scroll.rs` | 178 | ⭐ 滚动 |
| `surface_delta.rs` | — | ⭐ 增量表面更新 |
| `surface_reuse.rs` | — | ⭐ 表面复用 |
| `codec.rs` / `frame.rs` / `endpoint.rs` / `error.rs` | — | 编解码 / 帧 / 端点抽象 / 错误 |

### 2.2 ⭐⭐⭐ 与 NeoTrix 的直接对撞：**surface 协议 vs neobot 的虚拟化列表**

⭐⭐ **这不是巧合，是同题**：
- **herdr**：把「表面」做成**协议层**概念（`surface_delta` / `surface_reuse` / `surface_scroll`），
  逻辑在**协议 crate** 里，GUI 只渲染 ⇒ ⭐ **可脱离 GUI 测试**。
- **NeoTrix/neobot**：已有 `nt_ui_realtime_surface.rs`（`Surface` trait + 五方法契约 + 帧循环驱动），
  neobot-ui 侧有自己的**虚拟化会话/消息列表**。

⇒ ⭐⭐ **可迁移的价值**：把「增量 + 复用 + 滚动」三件事**收敛到一个协议层**，
而不是**每个 UI 各自实现**。这与本轮在 neobot 上的发现**同型**：
`nt_crystal_dispatch`（354 行）曾零消费者 ⇒ ⭐ **能力建好但没人用，是本仓反复出现的形态。**

### 2.3 ⭐ neobot 的 IPC 分层可直接照抄

⭐⭐ 本轮 neobot 的 IPC 修复（`7405d8dd`/`9e42e89b`）暴露过一个问题：
**契约扫描曾只扫 `main.rs`**（`nt_api_contract.py`），后改为 `main.rs + lib.rs`。
⇒ ⭐ herdr 的做法是**更彻底的一层**：⭐ **协议本身独立成 crate，依赖只有 5 个**
⇒ 契约变更会在**编译期**暴露，而不是靠扫描脚本追。

---

## 3. ⭐ 与本轮已有发现的合流（本日全景）

| # | 发现 | 出处 |
|---|---|---|
| 1 | ⭐ **接线分三级**（声明 / 分发 / **使用**），L1+L2 绿 ≠ 能力被使用 | `9e5b47e9` |
| 2 | ⭐ **建好但零消费者**是规模化形态（209 个扩展 API 中 50 个零调用） | `138c43e4` |
| 3 | ⭐ **失败方向由失败模式决定**（waku 记忆 fail-open ↔ NeoTrix SSRF fail-closed） | 本文 §1.2 |
| 4 | ⭐ **阈值/成本必须自带测量出处**（waku 把 260ms/$0.02 写进 docstring） | 本文 §1.3 |
| 5 | ⭐ **能力树启动时是空的**（5 个节点生产者零调用）→ `7187e0b9` 已接线 | `7187e0b9` |
| 6 | ⭐ **协议独立成纯依赖 crate**，使契约变更在编译期暴露 | 本文 §2.3 |

⇒ ⭐⭐ **一条主线贯穿全部**：
**「有机制」与「机制被真实使用」之间的距离，比机制本身的复杂度更重要。**
今日六项发现里有一半是关于**距离**的。

## 4. 建议动作（**均未实施**，按价值排序）

1. ⭐⭐⭐ **检索准入闸**（§1.4）—— 本仓的**真实空白**：默认检索 + 从不问「需要吗」。
   ⭐ 与能力树**平行**，互不依赖 ⇒ 可独立做。
2. ⭐⭐ **给新增守卫写明失败方向与理由**（§1.2）⇒ 改一处文档规范，收益覆盖全仓。
3. ⭐⭐ **协议 crate 化**（§2.3）—— 把 neobot 的 IPC 契约抽成独立 crate（依赖仅 serde/thiserror）
   ⇒ 契约变更编译期可见，⛔ 不再靠 `nt_api_contract.py` 追。
4. ⭐ **成本注释规范**（§1.3）⇒ 带成本的机制必须在 docstring 写单位成本
   （⭐ 直接来自 mu 的 54% token / 零收益教训）。