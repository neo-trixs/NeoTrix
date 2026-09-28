# Handoff — 剩余任务汇总（供单窗口统一修复，2026-09-28）

> **给下一个窗口的一句话**：本轮已把「干净检出跑不了任何 cargo」修到全绿，
> 但**任务台账的多个前提是错的**。动手前先读 §2 的勘误表 —— 否则会去"修"已经
> 正确工作的机制，或重造已经存在的东西。

## 1. 当前可交付状态（干净检出实测）

```
cargo check --lib -p neotrix     exit=0
cargo check -p neotrix (bins)    exit=0
cargo test  --lib --no-run       exit=0
check-fresh-build.sh --full       PASS
check-layer-deps.sh --strict     PASS (0 new / 94 known)
check-truth-surface.sh --strict  PASS
nt_lock_audit.py                  0 处
cargo test --lib                 12143 passed / 3 failed
```

**本轮新增三道门（勿删）**，全部带「非空门」回归证明：

| 门 | 作用 | 证明 |
|---|---|---|
| `scripts/check-fresh-build.sh` | 脏树不是合法 oracle → 自建 `git worktree add --detach HEAD` 再跑 | 修复前的 `5fcb291b` 上 tier1 得 exit=101 |
| `scripts/check-layer-deps.sh --strict` | 拦**新增**分层违规，既有 92 处记账 | 注入 L5→L6 后 exit=1 并精确报出 |
| `scripts/check-test-baseline.sh` | 57 条失败变可数的债 | 账本态 PASS；**暂建议模式不拦**（见 T1） |

## 2. ⚠️ 勘误表 —— 台账里已被证伪的前提（**动手前必读**）

| 项 | 台账原文 | 实测结论 |
|---|---|---|
| **4.1** | 给决策引擎加 JEV 四件套 | **前提证伪**：三个「决策引擎」全无生产消费者（gateway 再导出在无人启用的 optional feature 后；`new()` 仅在 `#[test]` 内；`WeightedScorer`/`DecisionRecommender` 外部引用 0）。已标 ⛔ |
| **2.1** | supersession 形态要新建 | **记忆库层早已实现**：`nt_memory_curation.rs:182/238` D2 冲突消解 + `tests.rs:564`；`nt_memory_historian/nt_temporal_facts.rs` 是整套版本链，**91 测试全绿**。真正缺的只有 **experience_tree 自身** |
| **2.2** | 真双时间要从零做 | `temporal_facts:41` **已经是真双时间**（`valid_from`/`valid_until` + `created_at`），且用「每版本独立 id」绕开复合主键。真正缺的只有 **`nt_core_kb_primitives` 的 `nodes` 表** |
| **5.2** | `McpRegistry` 在 `#[cfg(test)]` 里 | **错**：`agent.rs:496/505` 的 `#[cfg(test)]` 只挂单个测试辅助项，registry 在 `pub mod tool{`(420) = 生产面，**且已接线**（`entry/interactive.rs:94/108/111`）。真缺口是客户端不发 `protocolVersion`/`clientCapabilities` |
| **4.4** | `nt_shield_audit` 覆盖率账本 | `nt_shield_audit` **同样未接线**（只被一个事件名字符串 `nt_core_event.rs:179` 引用） |
| **路线图 §1.4** | 改 `paged_kv`/`kb_kv`/`vector_index` 做两表 | 三处**全是内存结构、无一张表**，「两表 `live_facts`/`retired_facts`」无处落地。方向指错 |
| **能力成熟度** | 「24 项降标，2 项升 C2」 | 成立；但其中 `OcrEngine` 的 ID 曾被我判为无效，**实为有效**（`nt_file_ability.rs:70` → `visual/mod.rs:6` 链式 re-export） |

## 3. 剩余任务（按建议执行顺序，标注依赖与阻塞）

### 第一优先：无前置、可独立完成

**T1 · 测试门转 `--strict`**（5 分钟 + 观察）
账本已无 flaky，技术上可拦。**但先在 CI 连跑数轮**确认那 3 条不再进出
（历史上 flaky 按运行变动），再把 `ci.yml` 的
`bash scripts/check-test-baseline.sh` 改为 `--strict`。

**T2 · 1.1 DNS 白名单**（定位已完成，可直接执行）
`egress_types.rs:14-21` 的 `SandboxEgressRule` **只有 `host`/`port`/`allow` 三字段，
全文件零 DNS 概念**。要做：① DNS qtype 白名单（当前只管 A/AAAA）② qname 长度上限
③ 把 `dns_allow` 从「学习提示」改成**真过滤器**。
⚠️ **不要拦 OSINT 的 `dns.rs:74 query_doh`** —— 那是合法侦察。

**T3 · 1.2 attempt/outcome 解耦**
现状：outcome 会衰减 attempt 的严重性。要做：两者成为**独立字段**，
任何情况下 outcome 不得衰减 attempt 严重性。
落点：`nt_core_telemetry.rs`（事件 schema）· `nt_safety_monitor.rs` · `nt_core_guardian/health.rs`

**T4 · 5.2 MCP capability 协商**（真缺口已确认，有生产消费者）
客户端不发 `protocolVersion`/`clientCapabilities`（`agent.rs` + `mcp_server.rs` 命中 0）。
协议层 775 行已稳定且要求每请求必带。生产入口：`entry/interactive.rs` + `headless.rs`。

**T5 · 2.1 experience_tree supersession**（勘误后收窄）
照 `temporal_facts` **已验证**的形态：每版本独立 id + `supersedes`/`superseded_by` 指针，
**绕开主键重写**（这正是「今天可迁」的含义）。当前 `ExperienceEntry` 无任何
`lifecycle_state`/`supersedes`；路线图 §1.1 要的 9 字段里 tree 只满足 `source`。
`experience_tree/mod.rs` 当前 clean，无归属冲突。

### 第二优先：有前置或需产品判断

**T6 · A1 `nodes` 表双时间**（依赖 T5 的形态决策；**风险最高**）
`nt_core_kb_primitives.rs:188` 的 `id TEXT PRIMARY KEY` 无时间维 ⇒
**双时间根本无法存历史**（插第二版撞 UNIQUE）。而 `nodes_as_of()`/`node_history()`
本就按 `transaction_time` 过滤排序、期望同 id 多行 —— **schema 与查询函数矛盾**。
我试改复合主键，**又暴露两处**，已回退（不做半迁移）：
- `edges.source_id/target_id REFERENCES nodes(id) ON DELETE CASCADE`（:248-249）
  是**真外键**，复合主键下失效
- `nodes_as_of()` 文档写「latest version visible at that time」**实现却返回全部版本**
- **既有 DB 需真实数据迁移**（`CREATE TABLE IF NOT EXISTS` 不改已存在的表）
⇒ 建议照 T5 的「每版本独立 id」形态做，**可完全避开复合主键与外键问题**。

**T7 · A2 Noise 握手**（⛔ **需产品决策，别擅自改**）
`noise_handshake.rs:190-194`：responder 执行 `_create_message2` 时要算
`es = DH(自己 ephemeral, 对端 static)`，但按 Noise **IK** 模式 responder
**根本拿不到**对端 static（它正是被加密送达的）⇒ 恒 `InvalidState`。
且该文件 :71-80 自陈协议名与 spec `Noise_IKpsk2_25519_ChaChaPoly_SHA256`(39 字节) 不一致。
**该模块无生产调用方**（`DIR-AUDIT §六`）。要先决定：**对齐 spec，还是明确降级并改名/改注释**。

**T8 · 1.3 / 1.4**（安全，需写入规范/冻结数据）
- 1.3 严重性校准对 + 职责分离 → 写入 `RUST-STANDARDS.md §17.5` 或 audit 规则
- 1.4 非不可宽化策略地板 → `crates/neotrix-neobot/src/nt_policy.rs:75` `evaluate_policy`
  现为 deny 全集（`human-control:77`/`computer-allow:84`/`computer-host:90`/
  `workspace-jail:98,106`/`unknown-tool:119`/`default-deny:121`/`evaluate_extra_deny:137`），
  要把「允许绕过批准的能力集合」变成**测试钉死的冻结数据**

### 第三优先：逐处棘轮 / 阶段工程

**T9 · A3 分层违规（现 94（84 条在 l*_ 层 + 10 条来自此前逃过检查的 `neotrix/` 第二棵树；另一窗口 2026-09-28 纳入该树，**属覆盖面扩大而非新增债**）** —— 棘轮已就位（`--update-baseline` 只应向下）。
建议**按文件聚类**而非按类别硬啃：
1. 先挑单文件只违反 1 条规则的叶子违规
2. 门已排除 `!*facade*` / `!*l1_facade*` / `!traits.rs` —— 这三类是**官方认可的跨层通道**，
   很多违规可能只需把引用改走 facade
3. `nt_io_web/api.rs` 违规密度高（7+）但**正被并发编辑**，需先确认归属
⛔ **不要**为数字好看而整体解禁或调大基线。

**T10 · 3.1 / 3.2 / 4.3 / 4.4 / 5.1 / 5.3 / 5.4** —— 见 `TODO.md` 对应行，均未动。

## 4. 本轮踩过的坑（**每个都会让下一个窗口浪费几小时**）

1. **干净检出是唯一可信的地面真相。** 主工作树因含他人未提交修复而「看起来正常」。
   本轮 57 条测试失败里 **37 条在工作树里早已修好、只是从未入库**。
2. **「工作树能编译」≠「可独立提交」。** 拷入 `entry/mod.rs` 会拉进 `dialog.rs`，
   后者需要 HEAD 没有的 `neotrix-neobot` 依赖 ⇒ 错误数从 2 **反弹到 58**。
   迭代时要**连 `Cargo.toml` 一起纳入**。
3. **过滤式 grep 会系统性剪裁你看到的世界。** 我用 `grep -v "supersedes: None"`
   + 只看前 12 条，得出「supersedes 是死字段」的**错误结论** ——
   恰好滤掉了所有真实写入方与读方。差点去"修"一个已正确工作的机制。
4. **半迁移比不迁移更糟。** T6 改到一半暴露外键与查询的更深问题 ⇒ 果断回退并记录。
5. **默认 `return` 会吞掉状态。** `git rev-parse --short HEAD` 捕获失败时仍打印上次值，
   我据此误以为提交成功。**判成功要看 `git show --stat`（post-hook 真相）**。
6. **pre-commit hook 会改 index。** `git diff --cached`（提交前快照）与
   `git show --stat`（事后）可能不一致 —— 以前者判断是否吞了他窗文件会误报。
7. **unquoted heredoc 会执行反引号。** `<<PY`（未加引号）里的 `` `//` `` 被当命令替换。
   含反引号/特殊字符的 Python 块一律用 `<<'PY'`。
8. **Python 切片赋值传 str 会按字符摊平。** `lines[i:i] = "字符串"` ⇒ 每个字符成一行。
   我用它把 `TODO.md` 写成 1706 行。**切片赋值必须传 list**；且改前
   `git checkout -- <file>` 可救 —— 但那会丢他窗改动，先确认归属。
9. **我的验收循环自己骗过我两次**：`for f in "x.sh --strict"` 把 flag 算进文件名 ⇒
   bash exit=127 被我读成「门失败」。

## 5. 并发纪律（本轮全程遵守）

- 工作树 ~960 个脏文件、17 个他窗 worktree，**全程未碰**，本方临时 worktree 残留 0
- 我用「worktree 内提交 + `git update-ref <ref> <new> <old>` CAS 推进分支 +
  `git reset`(mixed) 同步索引」，**全程不触碰他窗在制品**
- 取证期间他窗正在编辑 `nt_crystal_core/{cocoons,nt_premise_selector,nt_awaken_loop}.rs`，
  已**排除**在我的提交外（`cocoons.rs` 的 M 版自身带 E0597 半重构错误）
- **mtime 晚于当前小时 = 他方在写，换文件或先通报**（`AGENTS.md` 微操作公约）

## 6. 已知未完的小项

- **全仓 6+ 处测试 `set_var("HOME")` 改进程全局环境变量**，与读 HOME 的测试竞态。
  已让受害测试自足（`skill_loader`/`checkpoint`），**根因未除**。
  正解是给所有 HOME 改写点加共享 `Mutex`（先例 `agent.rs:506` `TEST_MCP_SERIAL`）；
  当时因 `cipher.rs` 属他窗在制品未做。
- `apps/neobot-desktop/frontend/` 23 个 + `gen/` 4 个仍未入库（归属裁决，
  见 `handoff-disease-list-20260927.md:103`）。`cargo check` 不需要它们，`tauri build` 需要。
- 剩余 3 条测试失败 = T6 的 2 条 + T7 的 1 条。
