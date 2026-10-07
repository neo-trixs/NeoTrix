# handoff —— 2026-10-07「债务全量刷新 + 同步修复」窗口（接上午那轮）

> 窗口目标：取**全部剩余债务的诚实清单** → 刷新到**实测**状态 → **同步修复**。
> ⛔ **未提交**（用户未要求提交；主树他窗在途，见 §6）。
> 承上轮：`handoff-2026-10-07-trace-and-honest-increment.md`

---

## §1 债务清单在哪（不另立文档）

全量清单在 **`docs/architecture/DEBT-LEDGER-2026-10-07.md`** —— 它自己的头部声明是
「债务**权威单一来源**」。我在它的**末尾追加**了一节
「⭐ 追加：2026-10-07 14:2x」，而**没有**新建第二份清单（新建 = 第二份「关于债务的说法」
= 本仓最主要的缺陷形态）。

⚠️ 我动它之前**连续采样了 3 次 mtime**（13:21:36 稳定 >1h）确认另一窗口没在写。

---

## §2 诚实总账（实测，**不抄文档数字**）

### 门矩阵 14 个 → **12 绿 · 1 红（advisory）· 0 恒红**

| 门 | 实测 | 与 13:21 快照的差异 |
|---|---|---|
| `check-unwrap` | rc=0 · 0 new · **514 known** | 一致 |
| `check-executor-registry` | rc=0 · 5 条对账一致 | 一致 |
| `check-silent-failure` | rc=0 | 一致 |
| `check-doc-drift` | rc=0 · **deadlinks 0** | ⭐ **由红转绿** |
| `check-dead-config-flag` | rc=0 · **新增 0** | ⭐ **由红转绿** |
| `check-orphan-dirs` / `doc-claims` / `claims-numbers` / `agent-config` / `map-check` / `layer-deps` | 全 rc=0 | 一致 |
| `check-test-baseline` | rc=0 · core **13,603** 绿 | 13,599 → 13,603（**非本窗**） |
| `check-naming` | **1612** advisory | 1615 → 1612（本窗新增文件） |

### 死配置（重测）

| 类型 | 字段 | 零读点 | 未接线规格 | **待人工判定** | 新增 |
|---|---:|---:|---:|---:|---:|
| bool | 1603 | 583 | **0** | **131** | **0** |
| numeric | 1129 | 313 | **0** | **51** | **0** |

ⓘ **上节写 184（133+51），本节实测 182（131+51）** —— 差 2，**成因未定位**
（期间 `nt_judge.rs` 被另一窗改过）。⇒ **如实记为不一致**，⛔ 不改文档迎合任一数字。

### 测试

`neotrix` **13,603 绿** · `neotrix-neobot` **581** · `neobot-desktop` **81+3+5=89** ·
`nt_lock_audit` **0**。

---

## §3 同步修掉的 6 项

| # | 债务 | 原来会怎样 | 修法 | 门证 |
|---|---|---|---|---|
| 1 | **`content_omitted` 只写不读**（**我上午自己造的**） | store 段头写明「UI 据此说『内容已略去』」，而轨迹页把字段传到 TS 接口**从不渲染** ⇒ 链路断在最后一格，**与 `AgentRunResult.trace` 同型** | 渲染 + i18n 两语 + 轨迹门加断言（**已变异验证**：删掉渲染即变红） | dead-flag 新增 **1 → 0** |
| 2 | 我新写的吸收记录 **2 条死链** | **外部仓**路径写成本仓相对路径 | 改成「`taste-skill` 仓内 `SKILL.md`」式，去掉目录前缀 | deadlinks 4 → 2 |
| 3 | 另 2 条**既存**死链 | `neotrix-core/.../context_fs.rs`（省略路径）+ `src-tauri/Cargo.toml`（**目录已不存在**） | 补全 / 标注已删 | deadlinks 2 → **0** |
| 4 | `neobot-check-markdown` / `-msg-copy` 红 | 桩里仍是**已删除**的 `neobot_convo_messages` ⇒ **一条消息都不渲染** | 桩改新命令 + `MessagePage` 形状 + `seq` | 两门 **rc=0**（复制按钮文案从 `null→null` 变 `zh「复制代码」→「Copy code」`） |
| 5 | `neobot-ui-smoke` 红 4 项 | `act` 在弹窗打开前执行且**吞异常** ⇒ 注入失败**根本没发生**；`probe()` **先测量后跑 act** ⇒ 看不出 act 有没有生效；之后又用**位置**点击开日志面板，此时已被设置弹窗遮住 | act 先开设置再选 + 不吞异常；测量移到 act **之后**；日志按钮改用**新增**的 `data-testid="nb-logs-open"` + 先 Esc 关弹窗 | **rc=0 PASS**，`set_language` 失败行真的出现在活动面板 |
| 6 | 死开关基线缺第三类 | 「Rust 只写不读、消费者在 Rust 之外」**结构上**不可能有 Rust 读点 | 基线新增 `cross-language-consumer`，写明**界面在哪一行真读了它** + ⛔「不要靠造 Rust 读点消掉它」 | rc=0 |

### ⭐ 第 1 项值得单独说

它**是我自己上一轮的产物**，而且形状与本轮开头修的那个缺陷**完全一样**
（数据一路传到界面、丢在最后一格）。区别只在于：**这次门抓到了**
（`check-dead-config-flag` 报「新增 1 个死开关」），而上次没人抓。
⇒ 这条既是修复，也是对「导出 ≠ 接入」这个反复出现形态的又一个实例。

---

## §4 ⭐ 本轮四条**测量错误**（比修的东西更值钱）

| # | 我犯的错 | 真相 | 纪律 |
|---|---|---|---|
| 1 | `timeout 600` 跑 `check-test-baseline` → 判它 FAIL | 该门**实跑 805s**，**超时被我读成「门失败」** | **超时不等于失败**；判门前先看它要跑多久 |
| 2 | 同上把 `check-doc-drift` 的超时判 FAIL | 单独重跑 **rc=0** | 同上 |
| 3 | `grep -oE '[0-9]+ offender'` 取 naming 数字得「15」 | 真值 **1612**（只吃到后四位） | 从门输出取数 ⇒ **先确认没截断** |
| 4 | 看到 smoke 报「`#root 子元素 = 0 ⛔ 未渲染`」就想改产品 | 那是**故意失败的控制探针**，**不算判据** | 先看它**算不算进判据**再动手 |

⇒ 与 `DEBT-LEDGER` 的「全局 8 条判据」#2/#3/#6 同族，本轮**全部命中**。
⇒ 也解释了为什么我在最终汇总里对每道门都**单独复核**了一遍。

---

## §5 ⚠️ 新发现的基础设施债（建议下一轮优先）

**Chrome 系门绑固定端口**：布局门 9333/9341 · 交互门 9334/9342 · 轨迹门 9352 …

**现象**：连续循环跑时「单跑 PASS、循环里 FAIL」。
**证据**：`nt_check_layout` **连跑 2 次均 PASS** ⇒ 循环里的 FAIL 是**端口/时序串扰**，不是回归。

⇒ ⛔ **任何「批量跑门」的脚本/报告都不可信**（包括我上午那份「19 道门全绿」的清单，
它也是循环跑的 —— 结论对，但**那次绿灯里有串扰成分**，不足为凭）。
⇒ 建议：随机端口，或门间串行锁；并在每道门输出里带上端口，便于事后判串扰。

---

## §6 收工自查

| 项 | 状态 |
|---|---|
| **worktree 去向** | ⛔ **我没开任何 worktree**。门报现存 **5 个**（3 个带未提交改动、**2 个近 3h 有 `.rs` 改动** ⇒ 可能他窗在用）⇒ **一个都没删**（R-DISK-5：删前须 patch 兜底） |
| **未提交改动去哪** | ⛔ 全部留在工作树。**主树共 48 处未提交**（上午 38 + 本轮 10）。⚠️ 其中**混有他窗改动**（`.neotrix/*`、`neotrix-core/*`、`.neotrix/patches/*.patch` 等）⇒ 提交时**必须** `git add <显式路径> && git commit --only <同一批>`，⛔ **禁 `-A`**（AGENTS.md §1：共享 index 下两次实测事故） |
| **我这两轮的完整文件清单** | 见 `git status --porcelain` 中：`apps/neobot-desktop/{STATUS.md,neobot-ui/src/*,src/*.rs,tests/ipc_roundtrip.rs}` · `crates/neotrix-neobot/src/{lib.rs,nt_store/mod.rs}` · `scripts/{dead-flag-baseline.txt,ops/*.mjs}` · `docs/architecture/{DEBT-LEDGER,ABSORPTION-*,ABSORPTION-BATCH-18,ABSORPTION-EXTERNAL}`。未跟踪新建 4 个：`nt_run_trace.rs` · `nt_store_run_trace.rs` · `nt_check_trace.mjs` · `ABSORPTION-2026-10-07-TRACE-AND-HONEST-INCREMENT.md`（提交前需 `git add`） |
| 他窗在途（⛔ 勿动） | `neotrix-core/src/l0_substrate/nt_judge.rs`（13:13）、`TODO.yml`（13:24）、`.neotrix/` 多个。`target` 已 **86,357M** |

---

## §7 下一步建议（按「能否被证伪」排序）

1. ⭐ **给 Chrome 系门做串行/随机端口**（§5）—— 否则一切批量门报告都不可信，
   而这是**元问题**：它决定其它门的结论能不能采信。
2. **死配置 182 按目录分片**（⛔ 禁批量）：先出「读现场清单」再动手；实测**约 1/4 会翻车**。
3. **`nt_dead_flag.py` 段头记下结构性盲区**：只在界面被消费的字段**必然**零读点（§3-6）。
4. `check-naming` **抽 1 层试点**再谈门禁（1612 批量改名 = 不可审的巨量 diff）。
5. `nt_check_bytes` 的 11 处 U+FFFD —— 等写它的窗口收工后再修（我已逐条确认**不在我的文件**）。
6. ⛔ **需人裁决、不该由 agent 单方面做**：D3-Phase-2（`OutputStyleId` 容器取舍）·
   D5（3 能力缺权威 schema）· D6（`LICENSE-EXCEPTIONS` 需签署 · `.project-map` 354M 体积策略）。