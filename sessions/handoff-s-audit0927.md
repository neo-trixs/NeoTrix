# Handoff — 结构性审计线（cycle `audit0927` / `audit0927b`）

> 按 `sessions/HANDOFF-TEMPLATE.md` 结构填写。**本线只做只读审计 + 定点修复**，
> 与 `handoff-disease-list-20260927.md`（卡死/内存专项线）并行，两侧清单会分叉，
> **不要互相覆盖**，只标注各自改了什么（见该文 §10）。

## 1. 会话标识

- 窗口：s-audit0927（结构性审计线）
- 分支：`feat/capability-absorb-20260828`
- 交接时间：2026-09-27 23:0x
- 最后提交：`4462fb2b`（本线共 14 个提交，`677489fa` … `4462fb2b`）

## 2. 目标（一句话）

审计「项目文件目录 + map」，找出**代码存在但工具链看不见**的结构性债务并除根，
把「文档约定」变成「退出码门禁」。

## 3. 已完成

- [x] **修 HEAD 无法独立编译** —— 全新 clone 必然构建失败。补 29 文件/7,339 LOC
      （已入库代码声明却未入库的 mod 目标，取传递闭包）+ 2 处缺失 `mod` 声明。
      验证：broken mod refs 9 → 0。
- [x] **删 24,884 LOC「已入库但从未编译」** —— `neotrix-core/src/cli/` 87 文件；
      `pub mod cli;` 从未出现在任何 commit 的 lib.rs 里。另删陈旧副本
      `l5_cognition/lib.rs`（目录模块不解析 lib.rs，它声明的 `mod nt_core_state;`
      反而掩盖了真实缺失依赖）。
- [x] **摘掉 4 个抽取 crate 的空壳门面** —— 54 个 0 字节模块 + L5 的 46 条空再导出。
      真实代码 27 文件/12,113 LOC 与 22 个有效再导出全部保留。
- [x] **真值面门禁** `scripts/check-truth-surface.sh`（EMPTY / UNDECLARED / TRACKED /
      UNCOMMITTED_DEP 四类），棘轮**列表**基线，接线 Makefile + `ci.yml --strict`。
      基线 60 → **0**。三向验证：对旧提交检出 8 个 / 现 HEAD 0 / 活体注入触发。
- [x] **复活 223 个从不编译的测试**（`test_extractors` 107 + `test_orchestration` 116，
      全绿），删 88 个（测已被 `3bba2507` 删除的 `nt_mind::sales_coaching`）。
- [x] **修 8 个真实实现缺陷** —— 详见路线图 §16.4 与 `ARCHITECTURE.md` §16.4。
      多数表象是「函数都在、返回值也看着合理」。
- [x] **6 个永久挂起测试清零** —— 全量套件从「跑不完」变成 **101s 跑完**。
      通过 11,299 → 12,042；失败 124 → 50。
- [x] **修两条静默失效的护栏** —— `.githooks/{post-checkout,pre-merge-commit}` 是悬空
      符号链接（深度错 + 目标已在 `d3bfb953` 被删），致 `reset --hard` 护栏一直没生效；
      已取回并修正，实测 981 个未提交改动时拦截成功。
- [x] **补锁审计的跨函数扫描** —— 原 `nt_lock_audit.py` 只扫单函数体内二次 `lock()`
      （词法），漏掉「A 持锁 → 调 `self.B()` → B 再锁」的**调用图**形态，
      而本轮就在 `runtime_monitor.rs` / `auto_inspector` 又找出两处同族死锁。
- [x] **Map 刷新** —— 路线图 §11 事实对账层（每行附再生命令）+ 头部状态戳；
      `ARCHITECTURE.md` §16 模块拓扑实测 + §16.4 缺陷清单。
- [x] **经验入 KB** —— cycle `audit0927`（16 条）+ `audit0927b`（4 条），
      10 条关键词路由，`route-verify` 0 ghost。

## 4. 正在改的文件（关键！逐个列）

**本线无未完成的半成品。** 全部 14 个提交已落盘且验证通过。

| 文件完整路径 | 状态 |
|---|---|
| `scripts/check-truth-surface.sh` + `truth-surface-baseline.txt` | 新增，已接 CI |
| `scripts/ops/nt_lock_audit.py` | 已扩展跨函数扫描，selftest 2 正 2 负 |
| `ARCHITECTURE-MAP-ROADMAP-V2.md` / `docs/architecture/ARCHITECTURE.md` / `TODO.md` | 已刷新 |
| `sessions/handoff-disease-list-20260927.md` §10 | 已追加交叉核对（该文件属并行线，用 `git add -f`） |

## 5. 下一步（按优先级排序）

任务清单已写入 `TODO.md` 顶部「🔴 P0·本线遗留」5 项。要点：

1. **50 个失败测试** —— 独立成轮，别与结构清理混做。分布见 TODO。
2. **`--test-threads=4` SIGSEGV** —— 崩在 `l6_meta::healing::predictive_maintenance::trend::tests`
   之后；同模块单/双线程不复现（311 passed）。**CI 暂用 `--test-threads=2`**。
3. **三处同名类型双定义** —— `ExtractConfig` / `EmailConfig` / `PlatformRegistry`。
4. **`crates/nt-lang`** —— 无 `[lib]` 故结构上无法被依赖，却占 member 槽。
5. **`nt_core_capability_tree` 归属** —— 住在 `neotrix-core/src/neotrix/` 下的独立 crate。

## 6. 阻塞点

- **无硬阻塞。** 但注意两条外部约束：
  - `nt_mem_gate.sh` 在低内存时 BLOCKED 重型 cargo（并行线实测过 free 207MB → exit=2）。
    重型构建前先 `sh scripts/ops/nt_mem_gate.sh; echo $?`。
  - **`--all-targets` 仍红**：2 个 example（`nt_whisper_spike.rs` 缺 `required-features`
    导致 `ort` 未解析；`v2_quick_start.rs:78` `McpToolDef` 缺 4 字段）。**本线未修**，
    留给接手会话（pre-commit 只跑 `cargo check --tests`，不覆盖 examples）。

## 7. 给接手会话的话

- **恢复命令**：先读本文件 + `TODO.md` 顶部，再
  `git log --oneline -15` / `git status --short` 核对。
- **安全网**：`refs/audit/snapshot-20260927`（已 pin，GC 不可回收）+ 15 MB untracked
  备份 tar 于 `/var/folders/.../T/opencode/nt-audit-backup-20260927/`。
  另：`git reset -q`（清 index 不动工作树）。
- **禁止事项**：
  - 不要跑 `cargo check --all-targets`（长期红，见 §6），用 `cargo check -p neotrix --lib`。
  - 不要 `git add -A` —— 树上有 ~950 个他窗在途文件，会被卷进你的提交。
    一律 `git commit -- <paths>` 做 pathspec 限定。
  - 不要 `git clean -fd` / `git reset --hard`（护栏会拦，但别试）。
- **风险提示**：
  - **多窗并发**：本线期间连续 4 次撞到他窗在途编辑导致的编译失败
    （`fofa.rs` 多余 `}`、`linker.rs` `pub` 位置错、`nt_core_llm/mod.rs` 标注缺失、
    `ui.rs` 类型不匹配）。**处置原则：不改，等稳定**（比对 mtime + 轮询 md5）。
    必要时最小修并**在报告里明示可 revert**。
  - `kb_search.rs` 的死锁修复（`drop(conn)`）是**另一窗在 22:51 独立提交的**，
    不在本线 14 个提交内 —— 别误认领。
  - `sessions/` 在 `.gitignore`，但 `handoff-disease-list-20260927.md` 是**已跟踪**文件，
    改动需 `git add -f`。
- **别重犯的判断错误**：本线两次把「无法编译/不可修」上升为「设计冲突」而实际是
  「import 指错模块」。**先量再判**；遇同名符号先确认是否旧名/新名关系。
