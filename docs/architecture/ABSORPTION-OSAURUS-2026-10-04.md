# 吸收 osaurus（+ Atlas 同批）—— NeoBot 架构裁定

> 日期：2026-10-04 · 方法：**先实测再下结论**（本轮 3 次否证了自己的假设，见 §5）
> 对标源：`osaurus-ai/osaurus`（Swift/macOS harness，MIT）、`pacifio/atlas`（Tauri agent desktop）

## 1. 本轮裁定的两个真缺陷（⭐⭐ 均可复现）

### 1.1 ⭐⭐⭐ `ring_*` 四棵子树 **2,152 行从未参与编译**

| 目录 | 文件 | 行 |
|---|---:|---:|
| `defense/ring_core` | 4 | 642 |
| `defense/ring_inner` | 5 | 610 |
| `defense/ring_outer` | 5 | 560 |
| `defense/ring_boundary` | 4 | 340 |
| **合计** | **18** | **2,152** |

**证据（可复现）**
```sh
# defense/mod.rs 只声明了 5 个 mod，四个 ring_* 目录一个都没声明
grep -n 'pub mod' neotrix-core/src/l3_embodiment/nt_shield/defense/mod.rs
# → unified_defense / refusal_tamper / guardrail_traversal
#   / reasoning_protection / anti_distillation

# 测试收集数 = 0 ⇒ 从未编译
cargo test -p neotrix --lib ring_inner   # test result: ok. 0 passed
# 对照：真实可达的同类门有 4 条
cargo test -p neotrix --lib nt_retrieval_gate   # test result: ok. 4 passed
```

⭐⭐⭐ **为何严重**（不只是「死代码」）
1. ⭐⭐ **它从来没被类型检查过** ⇒ ⭐⭐ **腐化完全不可见**：字段名/签名/依赖漂移
   都不会有任何信号。
2. ⭐⭐ 读代码的人会以为它**是活的** ⇒ ⭐⭐ 决策基于假地面真相。
3. ⭐⭐ 它的 4 条单测**随模块一起不被编译** ⇒ ⭐⭐ 「有测试」也一并是假的。

⭐⭐⭐ **这正是 AGENTS.md §6.2 警告的「导出 ≠ 调用」的**更坏形态：
⭐⭐⭐ 这里连「导出」都没有 —— **`mod.rs` 根本没声明**，⭐⭐ 所以编译器
⭐⭐ 连「这段代码能不能编译」都不再关心。

**⛔ 本轮不删**：⭐⭐ 删除 2,152 行属**不可逆裁决**，且需要先回答
「这四层防御是**已完成待接线**还是**已被 L4 方案取代**」——
⭐⭐ `nt_retrieval_gate.rs`（L4，4 测全绿）已经承担了检索门职责，
⭐⭐⭐ 看起来 L3 `ring_*` 是**同一需求的更早版本**。
⇒ ⭐⭐ **待人裁决**：归档 / 接线 / 删除，⭐⭐ 不由 agent 单方面删。

### 1.2 ⭐⭐ `DispatchMode::Parallel` **名字撒谎**（已修，`267fe544`）

⛔ 改前：变体叫 `Parallel`，注释写「并行独立处理 (同步场景等价 Emit)」——
⭐⭐ **前半句承诺并发，后半句自认等价 Emit（= 串行同步）**；实现是
⭐⭐ `for h in &self.handlers { h(event, &|| {}) }` ⇒ ⭐⭐ **既不并行，又阻塞 producer**。

⭐⭐ 对标 Atlas `atlas-bus` 的核心契约：
> lagging subscribers **drop rather than block the producer**

✅ 已改为 `Independent`，语义 = ⭐⭐ **panic 隔离 + 留痕**（`handler_panic_count()`），
⛔ **刻意不做真并发**（L0 是无运行时依赖的同步基元，每事件 spawn 会变成线程工厂）。

⭐⭐ 附带抓出**我自己实现里的真 bug**：第一版把计数器做成 `static AtomicUsize`
⇒ ⭐⭐ cargo test 并行跑时互相污染（**1 个 panic 测出 2**）⇒ 改为**实例字段**。

## 2. 从 osaurus 吸收、⭐⭐ 本轮**未**实施的设计（记录判据）

⭐⭐ osaurus 最有价值的一条是**隐私过滤器**的三个机制，⭐⭐ 与我方
`nt_net` / 记忆拒密钥行 / `check-net-guard.sh` 直接相关：

| 机制 | osaurus 原文要点 | 我方现状（实测） | 裁定 |
|---|---|---|---|
| ⭐⭐ **对出网字节断言** | “Verify **wire-level** redaction in the Insights panel — it captures the **exact bytes the cloud saw**” | ⭐⭐ 仅有 `check-net-guard.sh`，⭐⭐ 断言 **URL**，⛔ **不捕获实际出网字节** | ⭐⭐⭐ **高价值缺口**：门测的是**意图**，⛔ 不是**后果** |
| ⭐⭐ **fail-closed on send** | “**Fail-closed**: if the post-scrub scan finds anything that leaked, **the send is blocked**” | ⭐⭐ 检索门 L4 版**默认恒 admit**（4 测明写「闸崩溃时必须照旧检索」）⇒ ⭐⭐ **刻意 fail-open** | ⭐⭐ **不算缺陷**：⭐⭐ 检索是**读**，⛔ 不是发送；⭐⭐ osaurus 的 fail-closed 针对**发送** |
| ⭐⭐ **逐条裁决 ⛔ 非整批丢弃** | 每条命中进 review sheet，批准后换 `[PERSON_1]` 占位符 | ⭐⭐ L3 `retrieval_gate`（**不可达**，见 1.1）的 `allowed` 是 ⭐⭐ **all-or-nothing** | ⭐⭐ 随 1.1 一并裁决 |

⭐⭐⭐ **一句话可迁移**：⭐⭐ **门要断言「后果」而不是「意图」**。
⭐⭐ 我方大量门断言的是「代码里有这个模式」，⭐⭐ 而 osaurus 断言的是
⭐⭐「云端**实际收到**的字节里有没有它」。⭐⭐ 这是**门设计的分水岭**。

## 3. 从 Atlas 吸收、⭐⭐ 本轮已落地/已核验

| Atlas 机制 | 出处 | 我方裁定 |
|---|---|---|
| ⭐⭐ event bus：**lagging subscriber drop，绝不阻塞 producer** | `crates/atlas-bus` | ✅ 已落地（`Independent` + panic 隔离，`267fe544`） |
| ⭐⭐ 单所有者 `SessionActor`：每 session 一个 tokio task | `crates/atlas-agents/src/actor.rs` | ⭐⭐ **待评估**：⭐⭐ 我方 UI 侧已有 `busyByConvo`，⭐⭐ ⭐ **Rust 侧是否仍有共享状态竞争 ⛔ 未测** |
| ⭐⭐ 一条通道 + `kind` 字段，⛔ 不是每种消息一个通道 | Atlas `atlas:*` | ⭐⭐ 待对照 `neobot-desktop` 事件面 |
| ⭐⭐ `acpSessionId` 是单一真源，「重建/改写它就是 bug」 | Atlas | ⭐⭐ **高度相关**：⭐⭐ 我方 `convo_id` 贯穿 store/分页/前端，⭐⭐ 应立同款契约 |
| ⭐⭐ `spawn_blocking`：阻塞调用会冻结整个 UI | Atlas | ✅ **已核验：0 违规**。⭐⭐ `open_store()` 本身是**同步 fn**，⭐⭐ 每次调用短时 ⇒ ⭐⭐ 不违反该红线（⭐⭐ 我原以为有 25 处违规，⭐⭐ 实测被否证） |

## 4. ⭐⭐ 三条可执行的后续（按性价比）

1. ⭐⭐⭐ **立「出网字节」门**：⭐⭐ 捕获真实请求体，⭐⭐ 对**后果**断言，
   ⭐⭐ 抄 osaurus 的 Insights 思路。⭐⭐ 这是 §2 里唯一的**高价值缺口**。
2. ⭐⭐ **裁决 `ring_*` 2,152 行**（归档 / 接线 / 删除）—— ⭐⭐ **需人裁决**，见 1.1。
3. ⭐⭐ **`convo_id` 单一真源契约**（抄 Atlas 的 `acpSessionId`）：⭐⭐
   立门禁止重建/改写该 id。

## 5. ⭐⭐⭐ 本轮的 3 次否证（⭐⭐ 比结论更值钱）

⭐⭐ **纪律**：AGENTS.md §5「扫描器告警 ≠ 缺陷」、§R-SCAN-1b「grep 命中不构成证据」。

| # | 我的假设 | 实测结果 | 处置 |
|---|---|---|---|
| 1 | `DispatchMode::Parallel` 有生产调用方 ⇒ 改名有风险 | ⭐⭐ `rg` ⇒ **零调用方** | ✅ 零风险，改 |
| 2 | 25 个 Tauri command 违反 Atlas `spawn_blocking` 红线 | ⭐⭐ 逐函数体扫描 ⇒ **0 违规**（`open_store()` 是同步短调用） | ⛔ **撤销「有 25 处违规」的判断** |
| 3 | L3 `retrieval_gate` 的 `secret/keys` 会绕过置信门 | ⭐⭐ **不成立**：用 `contains` 子串匹配，⭐⭐ 被挡住 | ⛔ 撤销「严重绕过」判断 |
| 4 | ⭐⭐ 记忆里的 `nt_retrieval_gate.rs` 在 `crates/neotrix-neobot` | ⭐⭐ **不存在该路径**；⭐⭐ 真身在 `L4/.../nt_memory_kb/` | ⭐⭐ 一度读错文件，⭐⭐ 据此加的测试**已撤销**（不留在错误前提上） |

⭐⭐⭐ **元教训**：⭐⭐ 第 4 条最险 —— ⭐⭐ 若没在提交前 `git checkout` 撤销，
⭐⭐ 就会**把一个针对不存在文件的测试留在仓里**，⭐⭐ 而它「看起来像」有效防护。

## 6. 相关文件

- `neotrix-core/src/l3_embodiment/nt_shield/defense/mod.rs` —— ⭐⭐ **四个 ring_* 未在此声明**（1.1 根因）
- `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_core/ring_inner/ring_outer/ring_boundary/` —— ⭐⭐ 2,152 行不可达
- `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/nt_retrieval_gate.rs` —— ⭐⭐ **真正在用的**检索门（4 测全绿）
- `neotrix-core/src/l0_substrate/nt_core_dispatch.rs` —— ⭐⭐ `Independent` + panic 隔离（本轮已修）
- `scripts/check-net-guard.sh` —— ⭐⭐ 现有网络门；⭐⭐ 断言 URL，⛔ 非出网字节