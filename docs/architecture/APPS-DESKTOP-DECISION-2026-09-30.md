# 决策：`apps/neobot-desktop` 回流本仓（2026-09-30）

> **状态**：已裁决（用户 C 选项）。本文件是**反转 `d5413335`** 的唯一理由出处。
> 任何与 `d5413335`、`CONTRIBUTING.md:27`、`ARCHITECTURE-MAP-ROADMAP-V2.md`
> 旧注释相冲突的读法，以本文件为准。

## 反转的是什么

| 时点 | 决策 | 落地 |
|---|---|---|
| 2026-09-27 前后 | 桌面 App **移出**本仓 | `d5413335` 删除 `apps/neobot-desktop`（97 个注册 IPC） |
| 2026-09-29 | 移出**执行**并归档 | `mv` 到 `Neo/neobot-archive/desktop-residual-20260929/`（58/58 SHA-256 一致）；`CONTRIBUTING.md` 改写为「本仓只保留 `crates/neotrix-neobot` 作库」 |
| **2026-09-30** | ⛔ **推翻，恢复 `apps/`** | 本决策；`check-layout.sh` 的 `ALLOW_DIRS` 加回 `apps` |

## 为什么反转

**取证事实**（2026-09-30 08:31 实测）：

- `apps/` 内容从 2 个文件增长到 **63 个**，含
  `tauri.conf.json` · `frontend/src/{ui,host,plugin}/` · 全套
  `icons/`（含 `ios/` `android/` 多尺寸）
- ⇒ **不是临时产物，是实质推进中的桌面端工程**

用户裁决：**C —— 确认 `apps/neobot-desktop` 留在本仓**。

## 落地改动（本决策同步的四处）

| 文件 | 改动 |
|---|---|
| `scripts/check-layout.sh` | `ALLOW_DIRS` 加 `apps\|源码\|桌面 App（tauri.conf.json + frontend/ + icons/）` |
| `scripts/check-layout.sh` | `RESIDUAL_KNOWN` 上方注释记录本次反转，并指回本文件 |
| `CONTRIBUTING.md` | 「移除了本仓的 `apps/neobot-desktop`」需改为「已回流」 |
| 本文件 | 决策唯一理由出处 |

## ⛔ 未决 / 需后续确认

1. **`apps/` 全部 63 个文件仍未入库**（tracked=0）⇒
   干净检出会缺整个桌面 App。`check-untracked-assets.sh` 覆盖 `docs/ crates/`
   `scripts/ skills/ sessions/ evals/ config/` 与 `neotrix-core/*`，
   **不含 `apps/`** ⇒ 该门目前不会报它。
   ⇒ **待办**：给该门加 `apps` 覆盖，或给 `apps/` 定单独的入库策略
   （1.3M，含大量多尺寸 PNG 图标，需评估是否该进 git）。
2. **`src-tauri/` 仍在归档区**，未回流 ⇒ `RESIDUAL_KNOWN` 对其保持空。
   若它也回流，需同样处理。
3. **workspace 归属未定**：`apps/neobot-desktop` 是否为
   根 `Cargo.toml` 的 workspace member？`TODO.md:1073` 记着
   `apps/neobot-desktop/Cargo.toml` 曾是 member ⇒ 需复核当前状态。
4. `CONTRIBUTING.md:34,77` 与 `TODO.md` 多处引用 `apps/neobot-desktop`
   为「已移出/历史路径」⇒ 本次只改了最直接的一处，其余按需再改，
   **未做批量替换**（避免把历史记述当现状改掉）。

## 纪律

反转型决策（推翻既有裁决）**必须**同时满足三条，否则下个 agent 会读到矛盾事实：

1. 写 ADR（本文件），记清「反的是什么、为什么反」
2. 在**门**里留反向注释（`check-layout.sh` 的 `RESIDUAL_KNOWN` 上方）
3. 改**文档里会被当现状读的句子**（`CONTRIBUTING.md`）

⇒ 三处互相指向，才能让 `git log`、门输出、文档三者一致。

## 追加（2026-10-01）：`apps/` 的**棘轮范围**与「零存量豁免」的不对称

### 事实（读源码核实，非印象）
| 门 | 扫描根 | 覆盖 `apps/`？ |
|---|---|---|
| `scripts/check-unwrap.sh` | `os.walk(".")`，`SKIP` 只含 target/.git/models/node_modules/.worktrees | ✅ **是**（全仓门） |
| `scripts/check-silent-failure.sh` | `ROOTS = ["neotrix-core/src"] + crates/*/src` | ❌ 否 |
| `scripts/check-truth-surface.sh` | `SCAN_ROOTS = "neotrix-core/src crates"` | ❌ 否 |

⇒ **不是「两个门不一致」，是两档策略**：core+crates 是「已治理区」，
`check-unwrap.sh` 是**唯一的全仓门**。

### ⛔ 订正一处「零存量豁免」的误解
`unwrap-baseline.txt` 里 `apps/` **0 行** —— ⛔ **不是**「`apps/` 被排除在范围外」，
而是**时间差**：`apps/` 于 2026-09-30 进仓并成为 workspace member
（`Cargo.toml:13`），而基线最后一次**定点维护**在 2026-10-01 08:45，
两次改动都没顺带扫 `apps/` ⇒ **是遗漏，不是策略**。

⇒ **裁决：`apps/` 自进仓起即在全仓棘轮内，且不享有存量豁免。**
其余代码有 **710 条**祖父债，`apps/` 从 **0** 开始。
⭐ 收窄范围是**反方向**：它已是 tracked + 参与编译，收窄会降低覆盖，
且与本文件「2026-09-30 恢复 `apps/`」的裁决直接冲突。

### ⛔ 订正 ⛔未决 #3
原文写 workspace 归属「未定」—— ⛔ **已被 `Cargo.toml:13` 的
`"apps/neobot-desktop"` 推翻**（它是 workspace member、参与编译）。此项不再是未决。

## ⚠️ 附带发现：`check-unwrap` 的 `path:line` 键会**静默换身份**（比范围问题更重要）

门以 `"%s:%d" % (path, line)` 为键，**不校验该行的 token 身份**。
⇒ 一次行号位移可同时产出 **1 处假 NEW + 1 处假 STALE**，而报告里看不出它们同源。

**已实证的一例**（`nt_core_code_search.rs`）：
- 基线原 `:495` / `:499`；`55dd1989`（2026-10-01）在**其上方 147 行**处 1 行换 5 行（+4）
  ⇒ 两处 expect 真实位置变成 499 / 503
- ⭐ 而 `499` **恰好被另一条 expect 填上** ⇒ 字符串键相等 ⇒ 门认为「499 依旧合规」
  ⇒ **静默把身份换到了另一个违规上**，同时对 503 报假 NEW。

⇒ **推论：「0 new violation」不能证明代码没退化。**
任何在违规之上的插入都会伪造或吞掉信号 —— 这是 `path:line` 键的固有代价。
⏳ 待办：是否给基线行加 token 指纹（如 `path:line:token`）以堵住身份调包，
尚未评估迁移成本。
