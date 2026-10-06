# handoff — 2026-10-06 磁盘清理 + 交接文档梳理 + P0 数据丢失修复

> 分支 `feat/capability-absorb-20260828` · 提交 `83bc568b` `0adfa4ce` · 窗口：cleanup-and-defect-triage

## 0. 三件事，按重要性

| | 事项 | 结果 |
|---|---|---|
| 1 | **P0：一次写命令销毁能力注册表** | 已修 + 加门 |
| 2 | 70 份 handoff 建索引 + 缺陷单一入口 | 已入库 |
| 3 | 磁盘清理 | 回收 **300.3 GiB** |

## 1. P0：写命令把注册表从 318 节点覆盖成 41 节点（退出码 0）

### 实测证据

拿已提交的 `.neotrix/capability_registry.json` 喂真二进制，跑 `get`（**只读**子命令）：

| | 节点 | 边 |
|---|---|---|
| 输入 | 318 | 43 |
| 输出 | **41** | **0** |

3/3 复现，**退出码 0**。丢失的 277 个含全部 `exp::*` 与 `core::*`。

### 三处缺陷叠加

1. `CapabilityNode::kind` 是**唯一没有 `#[serde(default)]`** 的语义字段，
   而已提交文件里 **318 个节点全部没有这个键** ⇒ `from_str::<RegistryExport>`
   **整体失败**，报 `missing field 'kind'`。
2. `load_registry` 用 `Err(_)` 吞掉错误，把**任何**解析失败都当成「老 schema」
   ⇒ `migrate_legacy` 迁移出 **0 节点**。
3. `save_registry` 在 `run()` 末尾**无条件**执行 ⇒ 把 41 个 roadmap 节点写回。

### 修法（`83bc568b`）

- `kind` 加 `#[serde(default)]` + `Default for CapabilityKind = Skill`。
  取 `Skill` 而非 `Gap` 的依据：**实测**该文件里 `consciousness::gap::*`
  一个都没有（缺口由 `ConsciousnessRuntime` 运行期注册并显式带 `Gap`）
  ⇒ 默认值不会让缺口混入市场。
  ⚠️ 已在 `Default` 的 doc 里写明：若将来把缺口持久化进此文件，
  必须先给它们写显式 `kind: "gap"`，否则默认值会把「我不会」伪装成「我会」。
- 迁移判别改为**结构**（顶层有 `domains` 且无 `nodes`），并抽成
  `looks_like_legacy_schema()`；新 schema 的损坏文件**响亮报错并拒绝继续**。

### 附带修：快照跨进程不确定

`metadata: HashMap` + 直接 `to_string_pretty(&结构体)` ⇒ 照抄 HashMap 迭代序，
`RandomState` 每进程重播种 ⇒ **同一输入连跑 5 次产出 5 个不同 md5**（实测）。
这就是工作树里两个 `.neotrix/*.json` 出现 706 增/706 删**假 diff** 的成因
（已核实：语义差异 **0 条**、318 节点逐字段一致）。

⇒ `canonical_json()` 改为**经 `serde_json::Value` 中转**。

⚠️ **机制与最初设想不同**（已在代码 doc 写明）：本 crate 是
`serde_json = "1.0"`，**没有** `preserve_order` ⇒ `Value::Object` 是
`BTreeMap` ⇒ 中转那一步本身就排序。我最初额外写的递归 `sort_json_keys`
经实测是**死代码**（掏空后输出依然有序）⇒ **已删**，不留无法证伪的死代码。

## 2. 新门：`scripts/ops/nt-registry-determinism.sh`（已接 CI + task-index 78 条）

两条判据：① 5 次**独立进程**输出逐字节一致；② 节点/边数与基准相同（318/43）。

**为什么必须跨进程**：同一进程内 HashMap 迭代序稳定 ⇒ 单进程测试**抓不到**。
我为此写过 3 条单测，把 `sort_json_keys` 掏成空操作后**全部照样绿**
（实测 8/8 绿）⇒ 零区分力。

门自身踩了两个坑（已修，**双向验证过**）：

- `$first_md5）` —— **全角括号紧跟变量**，bash 把全角字节并入变量名
  ⇒ `unbound variable`（AGENTS.md 记的「全角标点吃字节」）。
- 二进制陈旧时拿旧结果误判 ⇒ 需 `NT_REGISTRY_REBUILD=1`。
  ⛔ `cargo clean` 后二进制消失，脚本会**自动重建** —— 已实测该自愈路径 PASS。

### 变异验证（判别标准：拆修复必须精确报红）

| 变异 | 结果 |
|---|---|
| 撤掉 `kind` 的 `serde(default)` | ✅ 2 条红 |
| `looks_like_legacy_schema` 恒真 | ✅ 红 |
| `save_registry` 退回直接序列化 | ⛔ 单测抓不到（跨进程缺陷的固有限制）⇒ 交给门 |
| `sort_json_keys` 掏空 | ⛔ 抓不到，且**证实它是死代码** ⇒ 已删 |

门双向验证：有 bug 时 `FAIL: 第 2 次输出与第 1 次字节不同`（RC=1）；
修好后 `PASS … 844f88012a…`（RC=0）。

## 3. 交接文档梳理（`0adfa4ce`）

- `sessions/README.md` —— 70 份 handoff 按 8 主题归类，**逐份入表零遗漏**
  （脚本对账：入表 70 / 实际 70）。此前**无任何索引**，只能靠文件名猜。
- `sessions/OPEN-DEFECTS.md` —— 遗留缺陷**单一入口**，12 条按 P0/P1/P2 分级，
  逐条标注 `已核实` / `未核实` / `已修`。已核实项含：
  金丝雀窗口确为进程全局、`maybe_compact_context` 仅测试调用、
  core 旧 distill 死循环的精确条件、core 2 处 U+FFFD、
  `scan_tree_sorted_by_lines_desc` 非本轮引入。
- `.gitignore` 补两条 `!sessions/README.md` / `!sessions/OPEN-DEFECTS.md`
  —— 不用 `git add -f` 硬塞（「是否入库」不该取决于个人操作而非策略）。

## 4. 磁盘清理

| 项目 | 处置 | 理由 |
|---|---|---|
| `target/` | **已删**，回收 **300.3 GiB**（314,663 文件） | 用户授权；纯编译产物；删前已确认无 cargo 在跑 |
| 14 个 `dist/` | 已删（~7M） | 用户授权；全部 gitignored，git 无影响 |
| `node_modules` ×3（797M） | **保留** | 用户选择；删了要联网 `pnpm install` |
| `.project-map/`（354M） | **保留** | gitignored 且可再生，但不在授权范围 |
| `models/`（4.2G） | **⛔ 未动** | AGENTS.md 明令：gitignored、git 保护不到、删了只能重下 |
| `.neotrix/patches/` + `worktree-salvage/` | **⛔ 未动** | 他窗 patch 兜底（R-DISK-5） |
| `.git/lost-found/`（309 个 dangling） | **未动** | 是取证证据，不是垃圾 |
| `models/training/repo_cards.jsonl.bak-*` | **未动** | 在受保护区内，不擅自处置 |

可用空间：**48 GiB → 280 GiB**。

## 5. 收工自查

- [x] 我的改动全部入库（`82824585` `83bc568b` `0adfa4ce`，另 `e27c0e03` 为上一窗口）
- [x] 未动他窗未提交改动：`neotrix-core/src/l3_embodiment/*`、`l6_meta/nt_approval.rs`、
      `.neotrix/capability_*.json` 仍为脏（**不是我的**，我这两个 `.neotrix` 文件
      从头到尾未被我的写盘路径触碰 —— 门脚本只写 `mktemp` 临时目录，已审计确认）
- [x] 自己开的 worktree 已收掉（本轮用 `mktemp -d`，退出时 `trap` 清理）
- [x] 改了 `.rs` ⇒ 锁审计已重跑（capability-tree + core 均 **0** 处）
- [x] 新门脚本**先审写操作再跑**（R-SCAN-4）：唯一写入目标是 `mktemp` 临时目录

## 6. 下一步（按优先级）

1. **执行通路**：`capability_invoke` 仍不执行能力本体 ⇒ 裁决 A/B/C（见
   `handoff-2026-10-06-capability-invoke.md` §3）。⛔ 别让 neobot 直接
   `use neotrix_core`（循环依赖）。
2. **计数语义**：`dispatch_by_capability` 只解析就计数 ⇒ 污染 `never_invoked`
   （见 OPEN-DEFECTS P1-4）。
3. **U+FFFD**：core 2 处（OPEN-DEFECTS P2-8），建议加门防复发。
4. 金丝雀按 `convo_id` 键化（P1-5）；`maybe_compact_context` 接预算（P1-6）。
5. `.project-map/`（354M）是否清理 —— 需你确认，不在本轮授权内。