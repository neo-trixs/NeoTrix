# Handoff — 2026-09-27 目录架构统一轮

## 1. 会话标识

- 窗口：本窗口（opencode 主窗口）
- 分支：`feat/capability-absorb-20260828`
- 交接时间：2026-09-27 23:5x
- 验证基线：`cargo check -p neotrix --lib` **exit=0 / 1m07s**（本轮唯一编译验证）

## 2. 目标（一句话）

> 解决"目录架构与主代码进化之路不契合"，并把三份审计合并成**唯一**任务清单 + 交接文档。

## 3. 已完成（12 项，全部已验证落盘）

### 3.1 修真 bug（唯一经 cargo 验证的代码改动）

- [x] **`:memory:` 泄露** —— `KnowledgeBase::open` 把 SQLite 哨兵 `":memory:"` 当**文件路径**传给
      `Connection::open`，在磁盘真建库；随后 `db_path.with_extension("lock")` 的 flock 侧车以
      `create(true)` 再造 `:memory:.lock`。两者都落在进程 CWD（实测即仓库根，0 字节文件生成于 22:22）。
      全仓 **9 个调用点**（测试代码）用 `Some(PathBuf::from(":memory:"))` 表达"内存库"。
      **修法在被调用方拦截一处，全 9 点生效**：`l4_emotion/nt_memory/nt_memory_kb/kb_core.rs:101-111`
      关键发现：同族 `TemporalFactLedger::open`（`nt_memory_historian/nt_temporal_facts.rs:63-68`）
      **早就有正确处理**，即这个 bug 类已知并修过一处、唯独漏了 `KnowledgeBase::open`。
      按同一模式补齐，未另发明写法。

### 3.2 删除（死代码，git history 可追回）

- [x] `neotrix-core/src/protocol/` —— 312 行 NIP-01 事件总线。**从未被编译**（不在 `lib.rs`）→ 那 7 个
      测试**从未真正跑过**；Nostr/NIP-01/Buzz 在 914k 行代码库零足迹。
- [x] `neotrix-core/src/adapter/` —— 7 行陈旧桩，自述"已合并进 `nt_io_provider`"；后者已完全消失，坐实合并残留。
- [x] `games/` —— 只剩 `.DS_Store`。

### 3.3 .gitignore（3 处）

- [x] 清 3 条指向已删 `games/neotrix-guixu/` 的陈旧规则（`:105` `:106` `:198`）
- [x] `sessions/` → `sessions/*` + 白名单 `!sessions/handoff-*.md` + `!sessions/HANDOFF-TEMPLATE.md`。
      **原为整目录忽略，11 个文件全靠 `git add -f` 硬塞** —— 规范证据是否入库取决于个人操作而非策略。
- [x] `HANDOFF*.md` → `/HANDOFF*.md`。**无前导斜杠会匹配任意层级，且位于文件末尾（git 最后匹配者胜），
      把上面的白名单覆盖掉** —— 这是模板无法入库的直接原因。

### 3.4 文档与索引

- [x] `sessions/HANDOFF-TEMPLATE.md` **入库**（`AGENTS.md` 引用它做交接模板）
- [x] `AGENTS.md` 正典索引 → 指向 `docs/architecture/NEOTRIX-MASTER-BLUEPRINT.md`
- [x] `DOCUMENTATION-MAP.md` 同上 + 标注 `FUSION-ARCHITECTURE.md` 已废止
- [x] 锁审计门记录刷新：`22:52 实测 3 条` → `23:4x 复测 0 条`

### 3.5 规则与经验沉淀

- [x] `AGENTS.md` 新增 **R-SCAN-1/2/3**（扫描器告警≠缺陷 / 手推≠实证 / 门记录必带时间戳）
      + 「下刀前查 mtime」并发检测
- [x] `RUST-STANDARDS.md §17` 新增 **R-LOCK-4**（早退路径必须显式释放守卫再委托）·
      **R-LOCK-5**（`*self.x.lock() = v;` 是赋值型临时锁）· **R-BUILD-6**（内存闸全工作区共享，别人的
      cargo 会把你的闸拉黑）· **R-GIT-5**（`git status` 不告诉你"是否正在被写"）
- [x] `docs/architecture/LESSONS-2026-09-27-scanner-trust.md`（123 行推理链留档）
- [x] 本文件 `sessions/HANDOFF-TEMPLATE.md` 的 `.gitignore` 说明已同步

### 3.6 任务清单统一

- [x] `TODO.md` **522 → 248 行重建**：
      - **保留** 1-93 行已验证的人工摘要区（逐字节 `head -93` 保全，非转录）
      - **修** 损坏时间戳 `2026-13-02T01:58:00`（13 月非法）→ `2026-09-27（人工重建）`
      - **删除** 12 个 2026-09-20 自动生成过时组：`task-1..6` `ag-1..8` `bend-1..4` `ds-1..5`
        `fc-1..7` `cf-1..5`。依据：① `TODO.yml` 自述"4 个 error 阻塞"而实测 exit=0；
        ② `ag-1` Pre-Execution Firewall **0 文件**、`ag-4` `nt-policies.yaml` **0 文件**；
        ③ `ag-6` 人工审批已被 neobot `nt_policy.rs:75` fail-closed 网关取代（含 `human has control`）；
        ④ `ag-5` hash chain 已在 `nt_memory/coverage_ledger.rs` 存在。
      - **追加** 统一进化清单：6 阶段 / 24 项，全部带 `file:line`（见 `TODO.md` 末节）
- [x] `TODO.yml` **不动** —— 被两个 pre-commit 钩子消费（`git_hook.rs:25-49`、
      `safety_tools/git-hook.sh:9`），删不得；且 `git_hook.rs:58-61` **提交时自动 `neotrix todo sync`**，
      会自愈。已确认新 `TODO.md` 的 `[ ]` 计数为 0 → 钩子首检为 no-op，不会误拦截提交。

## 4. 正在改的文件（逐个列）

| 文件 | 改到什么程度 | 可独立提交 |
|---|---|---|
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/kb_core.rs` | `:101-111` 加 `:memory:` 哨兵拦截 | ✅ **已 cargo 验证 exit=0** |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/kb_search.rs` | `:552` 加 `drop(conn)` 修真自死锁（与另一 agent 的 `:334`/`:363` 同族） | ⚠️ 由他窗 `cargo check --tests` 覆盖验证 |
| `.gitignore` | 3 处（§3.3） | ✅ |
| `AGENTS.md` | 正典索引 + R-SCAN-1/2/3 + mtime 检测 + 门记录刷新（77 行，< 100 ✅） | ✅ |
| `RUST-STANDARDS.md` | §17 新增 R-LOCK-4/5、R-BUILD-6、R-GIT-5 | ✅ |
| `DOCUMENTATION-MAP.md` | 正典索引 | ✅ |
| `TODO.md` | 522 → 248 行重建 | ✅ |
| `sessions/HANDOFF-TEMPLATE.md` | `.gitignore` 说明同步 | ✅ |
| `docs/architecture/DIR-AUDIT-2026-09-27.md` | 新增 204 行 | ✅ |
| `docs/architecture/EVOLUTION-ROADMAP-CODE-NODES-2026-09-27.md` | 新增 298 行 | ✅ |
| `docs/architecture/ABSORPTION-EXTERNAL-2026-09-27.md` | 新增 427 行 | ✅ |
| `docs/architecture/LESSONS-2026-09-27-scanner-trust.md` | 新增 123 行 | ✅ |
| 暂存区 | 仅 `sessions/HANDOFF-TEMPLATE.md`（我显式 add，非他人） | — |

**未提交**（按惯例不主动 commit；用户未要求）。工作树另有约 950 个**他人**脏路径，见 `TODO.md` 头部
「第三轮 30 项未提交修改」段及其 pathspec 清单。

## 5. 下一步（按优先级）

1. **0.1 脱 stub** —— 最高杠杆。`src-tauri/src/stub.rs:275`（零状态单元结构体）· `:289`（返回字面量）·
   `main.rs:52`（注册源）。**注意 `stub.rs` 至今 clean 未改，而 `main.rs` 他人已 M** —— 按 R-GIT-3 处理。
   隐藏工作量：`UnifiedResponse.metadata`（`stub.rs:292-303`）**已预留**
   `consciousness_state{phi,coherence,gwt_resonance}` + `confidence`，与 L5 意识核类型级吻合。
2. **0.5 `maturity_audit()` 接 CI** —— 投入产出比最高。机制**已完整实现且带自愈**
   （`nt_core_capability_tree/src/registry.rs:484-485` 自动下调声称等级），缺的只是没人调它。
3. **0.4 删 `neotrix-types` 包内 `SkillRegistry` 自重复** —— 零风险第一刀
   （`core/skill.rs:54` 与 `core/skills/mod.rs:25` 同包内重复）。
4. **0.2 删 `ToolRegistry` 45 行 stub** —— `l5_cognition/nt_core_gate/nt_tool_registry.rs:11`。
5. **1.1 DNS qtype 白名单** —— `egress_types.rs:14-21`（`SandboxEgressRule` 只有 3 字段，零 DNS 概念）。
   **全行业空白**：7 个沙箱仓无一做 qtype 过滤。
6. **P0-1 `nt_core_capability_tree` 移出源码树** —— 等 `cli.rs` 他人改完再动。

## 6. 阻塞点

- **`nt_core_capability_tree/src/cli.rs` 他人正在改**（`load_registry` 错误处理重构）→ P0-1 移位暂缓。
- **内存闸周期性 BLOCKED** —— 3 个 opencode 窗口 + 浏览器 + Clash + 4.2GB 压缩器常驻，
  `Pages free` 常低于 100000 阈值。**用 `ps -Ao pid,rss,etime,command | grep -E 'rustc|cargo'`
  先确认是不是自己起的**（R-BUILD-6）。他人构建**不要 kill、不要 join**，通报并等。
- **禁 `cargo check --all-targets` / `--test`**（AGENTS.md 明文）—— 会同时起 2 个 rustc（实测 3.6+4.2GB）。

## 7. 给接手会话的话

- **恢复命令**：先读本文件 → `sh scripts/ops/nt_mem_gate.sh; echo $?`（必须 0）→
  `git status --porcelain <目标文件>` 查脏 + `stat -f "%Sm" <file>` 查是否他方在写（R-GIT-5）→
  再动手
- **禁止事项**：不跑 `cargo check --all-targets` / `--test`；不 `git add -A`；不 `--no-verify`；
  不动他人在写的文件；不沿用旧门记录数值
- **风险提示**：
  - `src-tauri/src/main.rs`、`nt_core_capability_tree/src/registry.rs`、`Cargo.toml`
    （他人加了 `serde_yaml = "0.9.34"`）、`nt_core_capability_tree/src/cli.rs` **均已 M**
  - `kb_search.rs` 有另一 agent 的同族修复（`:334` `:363`），我的在 `:552`，
    他的 `assert s.count(o)==1` 幂等守卫恰好阻止了重复打补丁
  - `nt_lock_audit.py` 曾被实时编辑（告警数 12→3→0）。**现 0 条，但见 R-SCAN-3：跑一次记一次，别引用旧值**

## 8. 本轮的三次自我纠正（留给下一棒，别重犯）

| # | 我犯的错 | 根因 | 教训 |
|---|---|---|---|
| 1 | 把 12 条死锁告警当真的，准备"修"其中 2 条**正确代码** | 没读现场就信静态扫描器 | R-SCAN-1：扫描器告警是**线索**不是结论；「无定点不改」对它同样成立 |
| 2 | 手推 `audit_indirect` 得出"不会误报"，而实测会 | 用推理代替实验 | R-SCAN-2：**必须把真实代码形态喂进去跑**。我推的形态恰好不是出 bug 的那个形态 |
| 3 | 审计里 3 条建议全错（`nt-lang` 该删、`protocol/` 是空壳、路线图该归档） | ①用了 `ls *.rs`/行数/引用数等**代理指标**下结论<br>②**没先查 `TODO.md` 已有的 B 类清单** —— `nt-lang` 与 `nt_core_capability_tree` 早已被列为待裁决 | 评估工作量先读代码再报数；提"新发现"前先搜既有记录 |

> 这三条与 3.1 的 `:memory:` bug 属同一类：**用指标代替内容**。
> `TODO.md:44-50` 早就写着那两个待裁决项，我却在 DIR-AUDIT 里当新发现重报了一遍。
