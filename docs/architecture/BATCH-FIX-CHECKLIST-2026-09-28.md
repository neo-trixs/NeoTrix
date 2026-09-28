# 批量修复任务清单 — 2026-09-28

> 配套 `EVOLUTION-ROADMAP-CODE-NODES-2026-09-28.md`（节点论证）+ `DIR-REMEDY-2026-09-28.md`（目录解法）。
> **本文是可直接执行的任务清单。执行前必读 §0 三道闸。**

---

## 0. ⛔ 本轮**未执行**任何代码修改，及其原因

按 AGENTS.md「下刀前查并发」执行，2026-09-28 **14:46** 实测到**三个硬阻断**：

| # | 阻断 | 实测证据 |
|---|---|---|
| 1 | **他窗正在写** | `crates/neotrix-neobot/src/nt_channel_dispatch.rs` mtime **14:45:28**（当时 14:46:38）· `nt_channel_serve.rs` 14:40:07 · `neotrix/nt_crystal_core/cocoons.rs` 14:38:26 |
| 2 | **内存闸关闭** | `sh scripts/ops/nt_mem_gate.sh` → **exit 2**；free_pages 在 60 秒内 **45,503 → 8,250**。AGENTS.md：*"非 0 禁止起构建"* |
| 3 | **961 个文件未提交** | `M` 565 · `??` 211 · `D` 185。且目标文件 `nt_core_capability_types.rs` 本身是 `M`（他窗 WIP） |

**⇒ 批量重构 = 写入他窗在制品 + 无法验证（不能编译）= 2026-09-22 三次覆盖事故的复现路径。**
本文因此交付**清单 + 已证伪项修正**，代码修改留待树清空后按 §2 顺序执行。

---

## 1. 🔴 本轮**证伪**的旧路线图条目（**先读这条，否则会按错的计划删活代码**）

### 1.1 ⛔ `nt_core_gate/nt_tool_registry.rs` **不是 stub —— 删它会打断 shield enforcer**

2026-09-27 路线图 0.2 把它标为「45 行 stub，建议整文件删除」。
**实测证伪**：

| 证据 | 内容 |
|---|---|
| **活消费者** | `neotrix-core/src/l3_embodiment/nt_shield_enforcer.rs:388-390` `fn write_action_registry() -> &'static ...nt_core_gate::ToolRegistry`（`LazyLock` 静态） |
| **它是什么** | `ToolSpec` 驱动的**可逆性事实源**：`.register(ToolSpec::reversible("write_file","undo_file"))` / `.irreversible("git_force_push")` / … |
| **模块树已声明** | `l5_cognition/nt_core_gate/mod.rs:28` `pub mod nt_tool_registry;` + `:35` `pub use nt_tool_registry::*;` |
| **它自己的注释** | *"从 `nt_core_gate/mod.rs` 纯搬移, 行为零变更"* + *"同一注册表同时发 tool spec 与 gate config, **两者不能分歧**"* |

> **`reversibility`（可逆/不可逆）是与 `nt_act/tool_registry.rs` 的 `ToolStats`（运行期计数）不同的领域概念。**
> 两者**不是重复，是正交**。DIR-AUDIT「同名不同型」第 4 次实例。
> ⇒ **动作：撤销删除计划。** 改记为「同名不同轴，保留并加注释」。

### 1.2 ⛔ `agentic_browse::ToolRegistry` 也是**正交**，不是冗余

`l2_perception/nt_world/crawl/agentic_browse.rs:34` —— `Vec<_ToolAction>`，crawl 域局部，
自包含（`:65` `:302` `:318` `:331` `:410` 全部在同文件）。**不动。**

### 1.3 ⛔ `nt_file_ability/capability.rs:185` 的 `CapabilityRegistry` 确实零消费者

`l5_cognition/nt_core/capability/registry.rs:454` 的注释自述：
*"全仓 `capability::CapabilityRegistry` 精确搜索除本文件外无命中"* —— 独立复核一致。
⇒ **这一份才是真冗余，可删。**

### 1.4 ⛔ `l5_cognition/nt_core/capability/registry.rs:462` 也是零外部消费者

`Vec<Capability>` + `tag_index`，DIR-AUDIT 记为「仅自测」。
⇒ 可删，但**须先确认 `mod.rs` 无 `pub use` 泄漏**（见 B-3 步骤）。

---

## 2. 任务清单（按可执行顺序；**每项都有验收与回滚**）

> 标记：🟢 零风险 · 🔵 中风险（需编译验证）· 🔴 高风险（结构性）

### A 组 · 门与元数据（🟢 不碰任何 `.rs`，**当前树状态下即可做**）

| ID | 任务 | 落点 | 验收 | 状态 |
|---|---|---|---|---|
| **A-1** | 层归属显式化：新建 `.neotrix/layer-map.json` | 新文件 | `python3 -c "import json;print(len(json.load(open('.neotrix/layer-map.json'))))"` == 4 | ✅ **4 trees + 3 unresolved，已验** |
| **A-2** | 新建 `scripts/check-naming.sh`（报告式，先不阻塞） | 新文件 | 输出 ≈ **1,644**（当前实测） | ✅ **实测 1,644；strict exit=1** |
| **A-3** | 新建 `docs/architecture/OWNERSHIP.md` 唯一裁决表（含本文件 §1 的 4 条证伪） | 新文件 | `grep -c '^##' OWNERSHIP.md` >= 7 | ✅ **8 节** |
| **A-4** | `check-layer-deps.sh` 扩到读 `layer-map.json` | 改脚本 | `--strict` 仍 **PASS 0 new** | ✅ **84→94 sites，+10 首次入基线，strict exit=0** |
| **A-5** | 刷新 `AGENTS.md` 门记录（本轮已做，14:45 两条 0 命中） | 已完成 | — | ✅ |

**A-1 内容（已验证的事实，直接可用）**：
```jsonc
{
  "neotrix/nt_crystal_core": { "layer": "l5_cognition", "role": "primary",
    "note": "52 files/21134 lines. L1 有 6 个活消费者: nt_dialogue_tui, nt_tui_app,
             nt_stdin_human, nt_crystal_llm_bridge, nt_free_pool,
             nt_core_task_dispatcher/nt_dispatcher_core. 禁止当死代码删." },
  "neotrix/nt_jev": { "layer": "l5_cognition", "role": "primary",
    "note": "决策三原语; 被 nt_crystal_core 5 处消费" },
  "neotrix/nt_file_ability": { "layer": "l1_action", "role": "primary" },
  "neotrix/ffi": { "layer": "l0_substrate", "role": "primary" }
}
```

### B 组 · 冗余清理（🔵 **需要编译验证** ⇒ 等 A 组 + 树清空后）

| ID | 任务 | 精确步骤 | 验收 | 风险 |
|---|---|---|---|---|
| **B-1** | 删 `neotrix/nt_file_ability/capability.rs:185` 的 `CapabilityRegistry` | ① grep 确认零消费者（已做）② 删结构体+impl ③ 查 `nt_file_ability/mod.rs` 的 `pub use` | `grep -rn "pub struct CapabilityRegistry" neotrix-core/src crates src-tauri/src \| wc -l` **4 → 3** | 🟢 |
| **B-2** | 删 `l5_cognition/nt_core/capability/registry.rs:462` 的 `CapabilityRegistry` | 同上，先查 `l5_cognition/nt_core/capability/mod.rs` | 同上 **3 → 2** | 🔵 |
| **B-3** | `neotrix-types` 包内 `SkillRegistry` 2→1（`core/skill.rs:54` / `core/skills/mod.rs:25`） | 保留 1，改另一处 `pub use` | `SkillRegistry` **4 → 3** | 🟢 |
| **B-4** | **在 `nt_core_gate/nt_tool_registry.rs` 加防误删注释**（§1.1 证伪） | 文件头加「live consumer: nt_shield_enforcer.rs:388」 | 注释在位 | 🟢 |
| **B-5** | 加 CI 断言：三类注册表各 == 1 的**目标值**（先 advisory） | `.github/workflows/ci.yml` 加 job | 打印当前数，不阻塞 | 🟢 |

> **B 组不做完不要接路线图 4.x**：能力计数不稳，自进化加新能力时会再加第 5 份。

### C 组 · 扁平缺陷（🔴 结构性，**必须编译验证**）

| ID | 任务 | 前置 | 说明 |
|---|---|---|---|
| **C-1** | `UnifiedApiImpl` 脱 stub | 树清空 + mem gate OPEN | `src-tauri/src/stub.rs:275` 持状态；`:289` 字面量→委派 `nt_crystal_core`；`main.rs:52` `:387` `:407` |
| **C-2** | 20 处 `Orchestrator*` 收敛 | B 组完成 | `neotrix-core/src/pipeline/`（7 文件/1,130，D/E/B/A/R/X 阶段命名）作真典 |
| **C-3** | 三棵记忆树裁决 + 合并 | P2 裁决表先出 | `l4_emotion/nt_memory`(236/82,071) 存储真典 · `l5_cognition/nt_mind`(422/132,990) 经验树真典 · `l6_meta/memory`(7/2,048) 待裁 |
| **C-4** | `neotrix::neotrix::` 双命名消除（20 处） | A-1 完成后**降级为可选** | 改 `mod.rs` 声明即可吸收，调用方零改动 |

### D 组 · 跨域错位（🔴）

| ID | 任务 | 落点 | 说明 |
|---|---|---|---|
| **D-1** | `neotrix-sysctl`（唯一 `unsafe` FFI）从 L5 剥离 | 5 个 L5 包 `Cargo.toml` | 只让 `l0_substrate` 依赖它，上层走 trait。**本项目 `#![forbid(unsafe_code)]` 与之直接冲突** |
| **D-2** | `nt-lang` 孤儿（5 文件/273 行，只有 `[[bin]]`） | `crates/nt-lang/` | 删或补 `[lib]` + 消费者 |
| **D-3** | `ARCHITECTURE.md:97` 纠正 | 文档 | 声称 `nt_computer/` 是「计算集群」；**实测是 fs/process trait**，`screenshot()` 默认 `None`，neobot 侧 `NoopBackend` 唯一后端 |
| **D-4** | 🔴 **【本轮新发现】`neotrix/ffi` 声明 L0 却引用 L5** | `neotrix/ffi/consciousness_tree.rs:290`（`IITPhiCalculator`）`:320-321`（`metacalib`）· `ffi/seal_pipeline.rs:131-132`（`TrainingCycleConfig` / `KBContext`） | 纳入分层门后**首次可见**。uniffi 绑定层依赖认知层 ⇒ 要么它其实属 L5，要么这层该下沉。**无任何既有门能发现** |

### E 组 · 能力补齐（🔵，依赖 A–D）

| ID | 任务 | 落点 | 来源 |
|---|---|---|---|
| **E-1** | 证伪门（0.2） | `crates/neotrix-audit/` + CI | harness-engineering `evals/README.md` + backpass |
| **E-2** | 记忆权威头 + 五个留存标签 | `experience_tree/mod.rs:29` | loopx + nanobot |
| **E-3** | delta-ops 取代整体重写 | `experience_tree/mod.rs:241` `:355` | Hindsight `reflect/delta_ops.py` |
| **E-4** | 成本归因插点 | `anthropic/anthropic.rs:93`（P0-4 断点处） | cost-xray |
| **E-5** | GUI 执行回路 5 项 | `l3_embodiment/nt_computer.rs` | agent-desktop + cua-driver + OpenAdapt |

---

## 3. 执行前三道闸（**每组之间都要重跑**）

```bash
# 闸 1：无并发写入（AGENTS.md 硬规则）
find neotrix-core/src crates src-tauri/src -name '*.rs' -mmin -5 | head   # 必须为空
git status --porcelain | wc -l                                           # 应 ≈ 0
# 记忆：三窗口覆盖事故（2026-09-22）就是这么发生的

# 闸 2：内存（必须 OPEN）
sh scripts/ops/nt_mem_gate.sh; echo "exit=$?"                           # 必须 0

# 闸 3：干净检出可构建（脏树不是合法 oracle）
bash scripts/check-fresh-build.sh --full                                  # 必须 PASS
```

**⇒ 三个都不是 0/PASS，就不要开始。** 当前的真实状态：闸 1 **红**（他窗在写）、闸 2 **红**（exit 2）。

---

## 4. 回滚规程（每组必做）

```bash
# 起点：先建专用分支，**不要在 M 状态的树上直接改**
git switch -c refactor/batch-20260928

# 回滚单组
git diff --stat                        # 先看改了什么
git checkout -- <具体文件>              # 只回滚自己的文件，**不用 git reset --hard**
                                          # 那会连他窗的 961 个改动一起丢
```

> **⛔ 绝不用 `git reset --hard` / `git checkout .`** — 当前树含他窗 961 个未提交改动，
> 一条 `reset --hard` 就是 2026-09-22 的第三次覆盖事故。

---

## 5. 「导出 ≠ 调用」—— 裁决表填写规约

本轮第 4 次撞上这个坑（`CapabilityRegistry` ×4 / `SearchResult` ×9 / 三个决策引擎 / **JEV 反向**）。

| 判据 | 用什么 | 不用什么 |
|---|---|---|
| 活/死 | **构造点** `Type::new(` / `Type {` 的**调用点** | ❌ `pub mod` 声明 |
| | `mod.rs` 的 `pub use` **不算**证据 | ❌ 导出计数 |
| | 跨 crate 的实际 `use` 路径 | ❌ 名字相同 |
| 字段集不同 | = **正交，不是重复** | ❌ 同名就合并 |

**已按此规约复核的结论**（写进 `OWNERSHIP.md`）：

| 目标 | 结论 | 构造点证据 |
|---|---|---|
| `nt_core_gate::ToolRegistry` | **活，正交，保留** | `nt_shield_enforcer.rs:390` `.register(ToolSpec::reversible(...))` |
| `agentic_browse::ToolRegistry` | **活，正交，保留** | 同文件 `:65` `:302` `:318` `:331` `:410` |
| `nt_file_ability::CapabilityRegistry` | **死，可删** | 精确搜索 0 命中 |
| `nt_core::capability::CapabilityRegistry` | **死（仅自测），可删** | 同上 |
| `nt_act::ToolRegistry` | **真典**（运行期 `ToolStats`） | 770 行 dispatch + stats |
| `nt_jev` + `nt_crystal_core` | **活，L1 6 消费者** | `nt_dialogue_tui.rs` 等 6 处 |
| 3 个决策引擎 | **死**（构造点全在 `#[test]` 内） | — |

---

## 6. 一页纸：现在能做什么

**当前树状态下**（有他窗在写 + 内存闸红）：

- ✅ **能**：写 A-1/A-2/A-3（全新文件，零冲突）、修文档、刷新门记录
- ❌ **不能**：改任何 `.rs`、跑任何 cargo

**树清空后**（闸全绿）：按 **B → C → D → E** 顺序，每组结束跑 §3 三闸 + `cargo check -p neotrix`。

**最该先做的三件**（若只做三件）：
1. **B-1 + B-2**（删 2 份死 `CapabilityRegistry`，4→2）—— 零风险，计数立刻好看
2. **B-4**（给 `nt_core_gate` 加防误删注释）—— **阻止下一次按错误计划删活代码**
3. **A-1**（`layer-map.json`）—— 让 43,834 行的第二棵树第一次**对架构门可见**


---

## 7. 执行记录（2026-09-28 15:0x）

### ✅ A 组全部完成（**未动任何 `.rs`**，故无需编译即可验证）

| 门 | 结果 |
|---|---|
| `check-layer-deps.sh --strict` | **exit 0**，94 known（原 84，+10 来自第二棵树） |
| `check-naming.sh` | **1,644**（advisory），`--strict` exit 1 |
| `check-doc-drift.sh` | exit 0，127（他窗 WIP 增量，非本会话） |
| `nt_lock_audit.py`（两处） | **0 / 0**，exit 0 |
| `check-truth-surface.sh --strict`（本地） | exit 1 / 16 UNCOMMITTED_DEP —— **干净检出 oracle 实测 exit 0**（`git worktree add --detach`），**非本会话引入** |

### A-4 的意外收获：第二棵树首次「被看见」

`neotrix/` 纳入分层门后立即暴露 **10 处此前不可见的跨层引用**：

| 树（声明层） | 违规 | 位置 |
|---|---|---|
| `neotrix/nt_file_ability`（L1） | → L2 / L5 / L6 | `tests.rs`（测试引 `l2_perception`+`l5_cognition`）· `capability.rs:277+` · `format_route.rs` · `selftest.rs` · `excel/selftest_excel.rs` |
| `neotrix/ffi`（L0） | → L5 / L6 | `consciousness_tree.rs:290+` · `seal_pipeline.rs:131+` |
| **`neotrix/nt_crystal_core`（L5）** | **0** | ✅ **干净** —— 声明 L5 与依赖事实一致 |

> **`ffi` 声明 L0 却引用 L5 认知（`IITPhiCalculator` / `TrainingCycleConfig`）—— 这是真正的跨域错位，
> 之前没有任何门能发现。** 建议 C 组加一条：`ffi` 的层归属或代码位置需要裁决。

### ⏸️ B 组为何仍未执行

B-1/B-2 是**零风险删除**（已复核两处 `CapabilityRegistry` 精确搜索 0 消费者），
但**「删除 `.rs` + 无法编译验证」在本会话不可接受**：

- 内存闸 **exit 2 BLOCKED**（free_pages 45,503 → 8,250）
- 他窗 961 个未提交改动 + 实时写入（`nt_channel_dispatch.rs` 14:45:28）

⇒ **不写不可验证的代码。** B 组留待树清空后按 §2 顺序执行。
