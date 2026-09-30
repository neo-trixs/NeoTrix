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
