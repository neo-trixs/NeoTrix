# NeoTrix 全特性吸收地图 · 进化任务清单（最终合并版）

> **四轮 45 仓**（09-27 的 109+385 · 09-28 的 8 源 · 09-29 的 30 源 + 15 源）的
> **特性级**总清单 + 按最优解重排的**执行路线**。
>
> **分工**：`DECISION-D3-EVOLUTION-EVAL.md`（D-3 结论）·
> `DECISIONS-REQUIRED-2026-09-29.md`（9 项待裁决）·
> `FINAL-ROADMAP-2026-09-29.md`（四轮路线）· 本文 = **合并真源**。
>
> **核心结论**：「自进化机制不是护城河，**能证明自进化是否有效**才是」——
> 已从 11 仓扩到 **45 仓**验证，例外只有 4 个。

---

## 0. 本轮自我完善已完成（✅ 4 项）

| ID | 做了什么 | 关键发现 | 验证 |
|---|---|---|---|
| **A9** ⭐ | **补上自进化闭环的断口** | `seal_loop.rs:632` **硬传 `None, None`**，而 `EvalHarnessApi` 的 impl **早已存在**（`nt_harness.rs:316`）⇒ **实现了但永远收不到调用**。效果上 = 候选变更**从不接受回归检验**就持久化 | 隔离树 0 错误 + 12,187 测试全绿 |
| **A3** | **空账本** —— 已实证**不是缺陷** | 我上一轮标它 P0 是**误判**。实测隔离树 **12,187 passed / 0 failed**，且脚本 `:95-102` 全绿时 `exit 0` 压根不看账本 ⇒ **空账本是事实正确** | 非空门证明：注入 `assert_eq!(1+1,3)` 后 `--strict` **exit 1** 且指名 |
| **D-3** | **评测三件套融合** | 二选一是**伪问题** —— 实为三层。`eval_engine` 的 `llm_judge.rs:86` 是 `let normalized = 1.0;`（**恒满分、从不调 LLM**）⇒ ⛔ 不接 | `nt_evolution_eval` 754 行 + 19 测试全绿 |
| **门修** | `check-truth-surface.sh:169` 假阳性 | `#[path]` 正则要求与 `mod` **同行** ⇒ 我的两行写法让真实文件被判 UNREACHABLE。**这是同类缺陷第 3 个实例** | 修后注入真孤儿仍报 ⇒ 没把门改瞎 |

---

## 1. 特性总表（按来源分档）

### 1.1 ✅ 已落地（35 项）

| 类 | 数量 | 代表 |
|---|---|---|
| 门与验证 | 15 | `check-ci-refs.sh`（新）· `check-truth-surface` 扩域（**212 条首次可见**）· `capability-truth` CI 阻塞态 · `Epistemic` 接线 |
| 记忆与知识 | 8 | `temporal_facts` 真双时间（**91 测试**）· D2 冲突消解 · `write_memory_entry()` 5 消费者 · RRF 融合 |
| 进化验证（新） | 4 | `nt_evolution_eval`：臂中立 / 噪声地板 / veto / 账本 · `eval_harness` 注入点 |
| 其他 | 8 | skill 路径解析 · 产物写时序（先审计后执行）· 三态 StopReason 等 |

### 1.2 ⬜ 待做（按批次重排，见 §2）

### 1.3 ⛔ 已证伪 / 不吸收（46 项）

| 类别 | 数量 | 代表 |
|---|---|---|
| 提交前提被证伪 | 8 | `NVlabs/kda` 非 Kimi Delta Attention · `supermemory` 引擎闭源 · `aether-search` RRF 是 22 行逐字移植 · `Aegis` 非安全防护 · `typesafe` 非类型安全 · `BongoCat` 不注入 · `excel-codex-bridge` 无公式/单元格 · `Infographic` 无约束求解器 |
| 落点已消失 | 4 | `UnifiedApi`（`src-tauri` 已归档，全仓 0 命中）· `ring_defense`（不在 baseline） |
| 前提已满足 | 3 | `CapabilityRegistry` 4→1 ✅ · `maturity_audit` 已接 CI ✅ · `temporal_facts` 已是双时间 ✅ |
| 零吸收 | 1 | `hanshuang-codex`（MIT 覆盖不了 `crack-keygen`/`edr-bypass-re`） |
| 许可红线 | 12 | AGPL（Varen/BongoCat）· LGPL（PI-Desktop）· 无 LICENSE（jev-dsh-decision/dshfind/Hands-On-AI）· 闭源（supermemory/cue.im/weco/waldo/ramp）· 非 OSI（PolyForm/ELv2） |
| 溯源待澄清 | 2 | `qybaihe/mu`（MIT 声明 Mario Zechner vs 提交作者 qybaihe）· `Aegis`（双署名零提及） |

---

## 2. 最优解执行路线（D-3 裁决后重排）

```
┌─ 批次 A ✅ 完成（零 .rs，~2 天）────────────────────────────────┐
│  A1 CI 幻影门 ✅   A2 未编译代码可见 ✅   A3 空账本 ✅（非缺陷）   │
│  A4 nt_manifest 数字追溯 ✅（已 PASS）                            │
│  A5 skill policy 接线 ✅      A6 改判 handoff-evo ✅            │
│  A7 臂中立契约 ✅（nt_evolution_eval）                            │
│  A8 限制四段式 ✅    A9 自进化闭环断口 ✅  ← 本轮               │
└─────────────────────────────────────────────────────────────────┘
                            │  依赖 A（门已可信）
┌─ 批次 B：测量面（唯一能让 NeoTrix 领先而非跟随）────────────────┐
│  B1 门可满足性元门        ⬜  ← 最高优先，它是元门               │
│  B2 门记录带 env 指纹      ⬜  ← nt_evolution_eval::EnvFingerprint│
│  B3 证伪门（预注册/四事实） ⬜  ← nt_evolution_eval::Preregistration│
│  B4 maturity 降级落盘      ⬜  ⭐ 陷阱：overrides 与 registry md5 │
│                            │     相同 ⇒ merge 路径未测过       │
└─────────────────────────────────────────────────────────────────┘
                            │  ✅ 2026-09-29 已补注入点
┌─ 批次 B′：自进化闭环（新，D-3 产物）──────────────────────────┐
│  B5 注入真实 harness       ⬜  eval_harness 字段已加，需接线     │
│  B6 噪声地板的数据源        ⬜  需同臂重复运行 ≥2 次             │
│  B7 CaseSpec 的来源         ⬜  转写自 nt_verify_oracle          │
│  B8 接 mu 证明有效的 2 个决策点 ⬜ 技能隐藏/蜂群过滤（非 admission）│
└─────────────────────────────────────────────────────────────────┘
┌─ 批次 C：策略面（⚠️ 另一窗口正在写 neobot）───────────────────┐
│  C1 三态工具策略  C2 StopReason 正交  C3 策略地板               │
│  C4 DNS qtype    C5 MCP capability  C6 两级熔断+半开探针租约     │
│  C7 审批=DB行+原子事务+纪元栅栏                                 │
└─────────────────────────────────────────────────────────────────┘
┌─ 批次 D：记忆与检索 ──────────────────────────────────────────┐
│  D1 权威头  D2 留存标签  D3 delta-ops  D4 move退役  D5 记忆层无LLM│
│  D6 两表checkpoint  D7 nodes表双时间 ⛔  D8 多因子打分  D9 缓存键含码│
└─────────────────────────────────────────────────────────────────┘
┌─ 批次 E：桌面回路（D-5 落点待定）────────────────────────────┐
│  E1 元素寻址  E2 capture_id  E3 8态终局  E4 计划失效门          │
│  E5 turn generation 栅栏  E6 可逆性感知置信度门控 ⭐             │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. 下一批的具体任务（可直接开工）

### ⬜ B1 · 门可满足性元门（最高优先）

| | |
|---|---|
| **问题** | `i-have-adhd` 的发布门**永远无法通过**（"no blocking findings" 是绝对规则，相邻规则却是比较规则），**是跑出来才发现的**。本仓 14 个 `check-*.sh` 中**无一记录了非空门证明**（实测 `grep -l "非空门\|non-empty\|注入" scripts/check-*.sh` = 0 命中） |
| **动作** | 新建 `scripts/check-gate-satisfiable.sh`：对每道 `--strict` 门，**注入已知违规确认它红**（非空门）**且**确认存在已知合规输入确认它绿**（可满足性） |
| **验收** | 每道门两条证明；**任何一门缺证明即红** |
| **已有基础** | 本轮已给 `check-ci-refs` / `check-truth-surface` / `check-test-baseline` 三道门做了非空门证明 |

### ⬜ B2 · 门记录带 env 指纹

- `nt_evolution_eval::EnvFingerprint` **已实现**（含排序去重 ⇒ 同一环境必同一指纹）
- 需接：`nt_manifest.py` 的门记录 / 每次 CI 跑批
- 验收：改 `Cargo.lock` 后 `stale` 必须 exit 1

### ⬜ B5–B7 · 自进化闭环接线（D-3 之后）

| ID | 任务 | 状态 |
|---|---|---|
| B5 | `SelfIteratingBrain.eval_harness` 字段 | ✅ **本轮已加**（默认 `None` = 向后兼容） |
| B5b | 从 `entry/brain.rs:91 build_brain()` 注入真实 `EvalHarness` | ⬜ 需决策：启动时构造 L6 harness 会让 L5 依赖 L6 具体类型，**违反跨层隔离** ⇒ 可能需要一个 factory trait |
| B6 | 噪声地板数据源 | ⬜ 需同臂重复运行 ≥2 次。**单次运行判决必然被 `insufficient_evidence` 否决** —— 这不是 bug |
| B7 | `CaseSpec` 来源 | ⬜ 转写自 `nt_verify_oracle::verify_deterministic`（同为纯函数、无 provider） |

---

## 4. 需你决策的（9 项，见 `DECISIONS-REQUIRED-2026-09-29.md`）

| # | 事项 | 状态 |
|---|---|---|
| D-1 | 265 条未编译 `.rs`（68,603 行） | 待定：先删 3 条 archive → 抽样 20 条 |
| D-2 | `hybrid_retrieval` 接线 | ⭐ **可接**（实测 0 编译错误），但会引入第 6 份 `rrf_fuse` |
| D-4 | `docs/` 恢复 vitepress | 建议不恢复 |
| D-5 | 桌面回路落点 | 建议落 neobot（阻塞 E1–E6 全部） |
| D-6 | `mu` 署名链 | 法务 |
| D-7 | `Aegis` 22 skill | ⛔ 只取方法论（与 superpowers 同名冲突） |
| D-8 | C1/C2 时机 | 等他窗 |
| ~~D-3~~ | eval_engine | ✅ **已裁决** |
| ~~D-9~~ | Provenance 默认 | ✅ **不动**（作者已辩护） |

---

## 5. 三条纪律（用代价换来的）

1. **门要问可满足性**（B1）。本轮 `check-ci-refs` 第一版是坏的
   （`[ -n "$val" ] && return 0` 把「非空」当「合法」，**抓到注入的幻影反而放过**）
   —— 是**非空门证明**抓出来的。**没有这步，我会交付一个永远绿的假门**。
2. **「数字异常」不等于「缺陷」**。本轮我把「test 账本 0 字节」标成 P0，
   实测发现**那是事实正确**（12,187 全绿）。同样 0 字节，
   `truth-surface-baseline.txt` 有 265 条（真存量）、
   `test-failures-baseline.txt` 有 0 条（**真已清零**）⇒ **含义完全相反**。
   **⇒ 必须问「这个数字本该是什么」。**（同源于「先证伪提交的前提」——
   我对自己的结论也该这么做。）
3. **「实现了」≠「被调用了」**（A9）。`EvalHarnessApi` 的 impl 一直存在，
   但调用方硬传 `None`，**全仓无注入点** ⇒ 效果与没有闸门完全相同。
   **⇒ 接线审计要找「谁在调用」，不是「谁被实现了」。**
