# ABSORPTION-BATCH3-MECHANISMS — Vibe-Trading 三机制落地 + 扫描器收口（2026-10-05）

> 承接 `ABSORPTION-BATCH2-VIBETRADING-2026-10-05.md`。本轮把 Vibe-Trading 提炼的
> 三项机制真正接进生产，并**用我们自己的 `nt_determinism` 原语**承载它们
> （不另造哈希器 —— 重造即重新引入位段别名缺陷类）。

---

## 1. 三路并行落地（文件集严格互斥，cargo 全部由我串行持锁跑）

| 路 | 产物 | 行数 | 验证 |
|---|---|---|---|
| **P8** | `neotrix-core/src/l0_substrate/nt_core_artifact_verdict.rs`（新）+ `l0_substrate/mod.rs` 单行注册 | 1561 | ✅ **25 passed** |
| **P9** | `neotrix-core/src/l5_cognition/nt_core_gate/nt_provenance.rs`（新）+ `nt_core_gate/mod.rs` 单行注册 | 2724 | ✅ **44 passed** |
| **P7** | `crates/neotrix-neobot/src/nt_agent.rs`（可逆性 shadow 记录） | +559 | ✅ **33 passed / 1 ignored** |

### 1.1 P8 产物级裁决层（`nt_core_artifact_verdict`）

**核心决定：纯度是结构性的，不是纪律性的。**
`evaluate(&EvalCase, Option<&ArtifactBundle>)` —— 无 `&dyn Executor`、无 `&mut Write`、无 `Clock`，
全是自有数据 ⇒ **你无法把 agent 运行偷渡进来，编译期就拒绝**。

- **四值裁决** `Pass`/`Fail`/`NotEvaluable`/`InvalidArtifact`，退出码 1/2/0 映射。
  ⭐ **不用 `Result`**：`Err` 只有一个变体 ⇒ 「评不了」与「评了但失败」塌缩，
  未测试的运行会**伪装成干净**。
- ⭐ `VerdictRecord` **私有字段 + 4 个构造器** ⇒ `Fail` **不可能**在不给 evidence 的情况下被构造出来。
  这是 API 形状保证，不是调用方自觉。
- ⭐ `EvidenceRef` 拆成 `artifact` + `locator` **两个字段** ⇒ 位置**形状**（`:12` vs `#/status`）可机检。
- **`Issue` 只有 `Block` 与 `NotEvaluable`，无 `Pass`**（P9 同理）⇒ **通过 = 没有行**，
  于是「检查没跑」永远不可能长得像「检查跑了且通过」。
- **检查是声明行**（`ArtifactCheck` trait + 调用方组装的 registry）⇒ 加检查不动求值控制流。
  P8 自述其设计依据是 Vibe-Trading 另一子系统的原话：
  *"every incident landed as another inline rule… Adding a check is adding a row plus a fixture pair."*
- ⭐ **不变量检查**：工具调用↔结果双向 join（孤儿调用 / 孤儿结果各判 `FAIL`）。
  P8 途中**自己抓到一个真 bug**：初版把 join key 放进单一桶，于是**每一次健康配对都被误判**
  `INVALID_ARTIFACT`。它的评语值得抄进纪律：
  > *「一个恒失败的假阳性比没有不变量更糟 —— 它训练人们关掉检查。」*

### 1.2 P9 溯源门（`nt_provenance`）

**降级发布状态机**（先修正、后切除 —— 与 Vibe-Trading 同序）：

```
解析 → 无 Block ⇒ Released{degraded:false, 字节完全相同}
  有 Block 且预算>0 ⇒ NeedsRevision{issues, remaining}      ← 由调用方消费
  有 Block 且预算耗尽:
     无法定位被标记的 span ⇒ Refused{ISSUE_NOT_CUTTABLE}
     切掉的正好是全部 claim ⇒ Refused{NO_PRICE_OBSERVED}
     切除 → **重新解析** → 用**同一个门**复验
        仍 Block ⇒ Refused{REMAINDER_STILL_FAILS}
        否则   ⇒ Released{degraded:true, cut_text}
```

- ⭐ **`ReleaseVerdict` 的 `Released` 变体携带 `&ReleasedText`** ⇒ 「不可能只拿到裁决
  而拿不到文本」是**变体里的借用**，不是约定。
- ⭐ **`degraded()` 由 `claims_cut > 0` 派生** ⇒ 该标志**不可能与事实不符**。
- ⭐ **切后必须重解析**：span 是**位置**，切一次全体漂移。
- `Refusal` **完全没有取文本的方法** ⇒ fail-closed 在类型上就是终态。
- `Span` **可被构造成垃圾**（start>end、越界、非字符边界），`slice()` 返回 `Option`
  ⇒ 只有 span **能说谎**时 `IssueNotCuttable` 才可达。

### 1.3 P7 可逆性 shadow（`nt_agent`）

**决定：只测量，不改行为。** 目的是让「这个类别到底有没有居民」从**可争论**变成**可测量**。

- 漏斗**真实位置是 `nt_agent.rs:795`**，不是审计说的 600 ⇒ **且根本没有超时**：
  `execute_tool` 是裸 await，真实超时在各 executor 内部。
- 分类覆盖 neobot 自己的全部 16 个 `ToolName`：`ReadOnly`(8) / `Reversible`(4) / `Irreversible`(4)，
  `Unknown(_)` → **Irreversible**（安全默认）；`match` **无 `_ =>` 分支**
  ⇒ 新增变体是**编译错误**；测试另钉 `ALL_TOOLS.len() == 16`。
- 记录进**既有 per-tool `steps` 行**（执行后写），**刻意不写 audit 行** ——
  audit 行是执行前写的（守「先写审计再执行」律），回填实测时延等于**倒着写账本**。
  被拒/干跑记 `elapsed_ms=none` 而非 `0`，免得「从未执行」污染时延分布。
- ⭐ 复用 `nt_determinism::Digest` 出 `digest=<u64>`，零新依赖零新哈希。

---

## 2. ⭐ P7/P8/P9 推翻了我 brief 里的 5 处前提（延续上轮结论）

| 我的错误前提 | 实测更正 |
|---|---|
| 工具漏斗在 `nt_agent.rs:600` 附近 | **是 795**，而且**根本没有超时**可分流 |
| `ToolReversibility` 在 `neotrix-core` | 在 **`neotrix-gateway/src/gate.rs:111`**；且它**零生产居民** —— 全仓只有它自己测试模块构造过 2 个值（`gate.rs:1283/1289`），`Compensable` **连构造器都没有** ⇒ 上轮我说它「有 10 个已登记工具的真实居民」**不成立**，在此更正 |
| `Reversible` 类可映射到 `WouldWait` | `nt_changes.rs:69-81` 的 before-image 是**条件性**的（`:91-98`）且**无 `revert` 函数** ⇒ **`Reversible` 在决策上塌进 `Irreversible`**（安全侧）。**策略轴目前是二元的** |
| `neotrix-neobot` 可直接 import 可逆性类型 | 结构性禁止（`neotrix-core` 依赖 neobot，反向即环）⇒ 只能从 neobot 自己的 `ToolName` 分类 |
| 我要求 P8 加「工具调用↔结果」不变量检查 | **该不变量在 `nt_agent.rs:576` 被写成规则却从无代码校验**，且**当前 schema 下不可校验**：`steps` 表 `INSERT INTO steps(task_id,n,tool,ok,output)`（`nt_store_routines.rs:402`）**无 `tool_call_id` 列**，两侧 join **用不同键** ⇒ 今日诚实的裁决只能是 `NOT_EVALUABLE`。**补一列即可，不是重写** |

⚠️ 另一项 P8 发现：`emit_tool_call`/`emit_tool_result`（`nt_core_jsonl.rs:149,161`）
**零调用者** ⇒ 基于 `--json` 的任何 join 不变量今日 `NOT_EVALUABLE`。

---

## 3. ⛔ 本轮遇到的**外部阻断**，及归因方法（比结果本身更值钱）

**三次编译失败，全部不在我的文件里**，但其中一次**确实是我的**：

| # | 症状 | 归属判据 |
|---|---|---|
| 1 | `lifetime may not live long enough`（`artifact_verdict.rs:963`） | ✅ **我的** —— 漏了 `&'c self`。已修（补 `&'c`），随后 25/25 绿 |
| 2 | `CheckStatus::Pass` / `.detail` 不存在（`nt_shield_audit`） | ⛔ 他窗：变体真值是 `Passed`（`mod.rs:94`），`cloudflare_patterns.rs` 当时在改 |
| 3 | `duplicate serde attribute` + `CapabilityKind` 冲突 impl ×5 | ⛔ 他窗：`node.rs` 未提交 **+61 行**，`git show HEAD` 证实 **`CapabilityKind` 在 HEAD 根本不存在** ⇒ 他窗新加的类型与 `neotrix-core/src/l5_cognition/nt_core/capability/registry.rs:282` 的**同名类型**撞了 |

**归因方法（可复用）**：
1. 报错文件**是否在我的清单内** → 不在即外部。
2. `git status --porcelain <file>` → 空则该错误存在于 HEAD，与我无关。
3. `git show HEAD:<file>` → 若目标符号**在 HEAD 不存在**，则是并发新增。
4. **同一批命令里后续目标是否成功** → 我的 B8 里 `artifact_verdict` 25/25 绿而
   `provenance`/`nt_agent` 失败 ⇒ 同一依赖链上**只有他窗那两个 crate**坏。

⛔ **未按 AGENTS.md §4.2 用干净检出测量**：脏树下计数不可信（`14→15 sites / 13 baseline`）。
但**违规归属**不依赖计数 —— 我的两个新文件 `grep` 均**不在**违规清单内，
两条 NEW（`nt_model_cli.rs`、`nt_capability_bridge.rs`）`git status` 皆空 ⇒ HEAD 既有。

---

## 4. 门状态（带核实时间戳，R-SCAN-3）

| 门 | 值 | 时间 |
|---|---|---|
| `nt_lock_audit.py neotrix-core/src` | **可疑 0 处** | 2026-10-05 14:28 |
| `nt_lock_audit.py crates/neotrix-neobot/src` | **可疑 0 处** | 同上 |
| `check-layer-deps.sh --strict` | rc=1，**15 sites / 13 baseline → 2 new** | 同上 |
| ⛔ 2 条 NEW 归属 | `nt_model_cli.rs`、`nt_capability_bridge.rs`，两者 `git status` **皆空** ⇒ HEAD 既有，**非本会话** | 同上 |
| ✅ 我的新文件 | `artifact_verdict` / `provenance` **均不在**违规清单 ⇒ **L0→neobot 新边未触发分层门**（P8 的担心未成立，但这是**新增依赖边**，棘轮会记） | 同上 |

---

## 5. ⭐ 本轮最诚实的一条：P9 自述「我为了满足测试而造了一条规则」

P9 在报告里主动指认（原文要点）：

- ⭐ **`witness=` 检查是「承重的虚构」** —— `witness=<claim id>` **是我发明的协议**，
  Vibe-Trading 无此字段。它存在只因需求 6 要求「两条不产生级联的检查」
  且需求 4 要求 `REMAINDER_STILL_FAILS` **可达** ⇒
  **「我建了一条规则来让 fail-closed 分支可达 —— 这就是为满足测试而造机械的定义。」**
  它建议真实运行时**删掉它**，让该分支只经真实重解析产物可达。
- ⭐ **`NoPriceObserved` 是错映射** —— Vibe-Trading 指的是「行情没返回价格」，
  它映射成「我们切掉了全部 claim」⇒ **不同事实，同名** ⇒ 调用方会读错结论。
- ⚠️ **需求 6 与需求 7 自相矛盾**（6 说缺 formula 算违规，7 说载荷缺失算 `not_evaluable`），
  它**单方面**用三态表化解，并如实记录了自己的第一版把「空 formula」当畸形注解
  ⇒ **违规分支不可达**、是测试抓出来的。
- ⚠️ **切除留下 `bench `/`view ` 孤儿** —— 字节级切除可审计但产出用户读起来像坏散文。
  自评「这是文件里最弱的输出质量决策」。

⇒ **我保留这些缺陷未改**，因为它们是**设计判断**而非编译错误，
且 P9 已给出诚实记录。改它们需要产品决策（`witness` 是否为真协议、
切除粒度是否应为行级），**不该由我在共享树上单方面决定**。

---

## 6. 下一步（按杠杆）

1. ⛔ **补 `steps.tool_call_id` 一列**（`nt_store_routines.rs:402`）——
   这是「工具调用↔结果」不变量从 `NOT_EVALUABLE` 变可判的**唯一前置**。
   一列，不是重写。
2. **删掉 `witness=` 检查**（P9 自述为虚构）并**改掉 `NoPriceObserved` 命名** ——
   两个都是诚实性问题，零风险，先做。
3. **切除粒度改行级 + 折叠空白**（P9 自评最弱项）。
4. **P7 的 shadow 数据读出来后**，才决定要不要把 `Reversible` 从 `Irreversible` 里分出来
   —— ⚠️ 先查 `nt_changes` 的 before-image 是否**真能 undo**（P7 已指出无 `revert` 函数），
   否则分开就是**放宽安全侧**，不能做。
5. ⛔ **补跑全量 `cargo test -p neotrix --lib`（13,259 个）** —— 仍未做，不得声称。

---

*本轮全部改动未提交。`node.rs` / `cloudflare_patterns.rs` 等为他窗并发工作，
⛔ 提交时必须 `git commit --only` 排除。*
