# 交接 —— 游戏源码吸收 · 第二批（修「尺子」+ 补安全洞）

> 2026-10-06 · 分支 `feat/capability-absorb-20260828`
> 承接 [`handoff-2026-10-05-game-source-absorption.md`](handoff-2026-10-05-game-source-absorption.md)
> 上一批遗留 5 项，本批**全部收口**。

---

## 1. 交付（5 个提交）

| hash | 内容 |
|---|---|
| `5c81f3ac` | **门**：「造了资产但没人调用」棘轮门 —— 首次抓出 **15,786 行从未编译的代码** |
| `f883230d` | `agent_guardrails`（1,492 行）**首次编译** —— 全仓唯一的破坏性命令/凭据泄露检测 |
| `a10342a2` | 管道进解释器检测 + 凭据模式补齐 —— **四轮对抗复验**后才敢提交 |
| `4dc52fea` | 游戏源码**实现级**深读记录（补上一批的文档欠账） |
| `9c6eae6c` | LLM 驱动的 `exec_shell` 接入护栏 —— 堵一条无任何检查的 `sh -c` 通道 |

---

## 2. ⭐ 本批最根本的一件事：**先修尺子**

`5c81f3ac` 新增 `scripts/ops/nt_security_wiring.py`，因为查清了「造了资产但没人调用」
**为什么能反复发生** —— 现有两个门有**结构性盲区**：

- `check-truth-surface.sh` **没有「编译了但零消费者」这一类**；
- `check-ext-wiring.py:95` 用 `text.split("#[cfg(test)]")[0]` 切测试块
  ⇒ 文件把测试 mod 放**前面**时其后全部生产代码隐形。实测 **2,365 个 `.rs` 里 198 个
  踩坑**，最极端的 `stagnation.rs` 是 `cfg(test)` 在第 32 行 / 共 820 行
  ⇒ **后面 788 行隐形、283 项资产被误判成死的**；
- 且它只认 `register_*`/`on_*` 注册式命名 ⇒ `DnsEgressPolicy` 这类非注册式策略资产
  一个都看不见（按 `secur|policy|gate` 关键词筛会**漏掉** `demote_mislabeled`、
  `SkillAudience::admits`、`ActionFacade`）。

新门：**全量枚举 pub 项**（零关键词预筛）+ **花括号配对**切测试块 +
**词法掩码**剥注释与字符串（实证 `code_review.rs:500` 的
`r#"Command::new("sh")…"#` 是字符串不是代码）。四类发现，其中
`ORphan_module` 是新类别 —— **首次即抓出 14 个从未编译的目录 / 15,786 行**。

基线 2,150 条已落库，`--strict` 绿，CI job 从「只报不拦」切成**真拦**。

---

## 3. ⭐ 三次「差点改错方向」，全部靠测量拦下

| # | 我的判断 | 实测结果 |
|---|---|---|
| 1 | 「把 `ShieldEnforcer::check_all` 接进 `exec_shell` 就是补上安全洞」 | 默认 `Suggest` 模式下它对**每一条**命令（含 `echo hello`）返回 `RequireApproval`，而该 TUI 路径**无审批 UI** ⇒ **接入即把 shell 功能 100% 关掉** |
| 2 | 「不可采纳启发会让 A\* 返回次优路径」 | 第一版测试**是绿的**；靠「同边代价 Dijkstra 当 oracle + 60 布局对账」才拿下 **28/60（47%）次优** |
| 3 | 「管道检测能拦住 `curl … \| sh`」 | 第一版只拦**远程取数**形态（`4/24` 误报，接线即破坏功能）；且四轮复验连续找出 `sudo`/`env`/`timeout` 包装器绕过、`python3.11` 版本后缀、`s""h` 相邻引用拼接等 |

⇒ **第 3 条的教训最值钱**：我先写了「剥掉包装器」的版本，**独立 crate 实测发现
它是半覆盖** —— `curl … | sudo -u root sh` 仍漏。子代理建议的正解
（**每包装器一张参数元数表**）是对的，但我最终选择**结构性反转**：
不再试图完美剥包装器，改为**扫描 sink 片段内所有 token**。
四轮打地鼠证明「枚举包装器」是无底洞（修好 `nice -n` 就冒出 `nice --adjustment`、
修好 `sudo -u` 就冒出 `su`/`doas`/`exec`/`taskset`）。

---

## 4. ⛔ 三次事故，都必须记录

1. **未提交代码被整体覆盖**：`piped_into_interpreter` 初版 + 凭据模式 + 探针，
   在我等待编译阻塞期间被另一窗 `checkout`/`stash` 覆盖（4 个文件全回 HEAD，零 diff）。
   ⇒ 已重放，并把「改完立即提交」当作硬纪律。
2. **`git commit --amend` 撞上他窗刚提交的 HEAD**：`a10342a2` 之后我试图 amend
   补正提交信息，结果 `--amend` 命中了**另一窗 6 秒前刚建的 `6440106`**。
   经 `git rev-parse` + 树哈希比对确认是**空操作**（树哈希完全相同、无内容丢失），
   并已用 `git update-ref` 恢复其原始 hash。
   ⛔ **结论：共享树里永远不要 `--amend`。**
3. **`check-commit-deletions.sh` 在 `--only` 下看不见提交信息**
   （`TODO.md:2660-2682` 记录的死锁，绕过办法是先写进
   `$(git rev-parse --git-path COMMIT_EDITMSG)`）。

---

## 5. ⭐ 我**否决**的删除建议（另一窗的取证把它全部推翻）

我曾建议删 3 个孤儿目录（1,507 行）。独立取证**逐条推翻**：

| 目录 | 我的理由 | 取证结论 |
|---|---|---|
| `nt_consciousness_core/archive` | 依赖已消失的 `unified_learning` | 依赖确实断了，但 **`da6cf04a` 已明确复核并保留**（\"有意的归档区，名字即语义\"）⇒ 我在重复一个已被否决的提议 |
| `l6_meta/coordination/nt_meta_cleanup` | 与已编译实现「重复」 | ⛔ **字节级零重复**（9 文件对全仓 2,860 个 `.rs` 做 md5，各仅 1 份）—— 只是**重名**不是重复。且**另一窗 10 分钟前正在接线它**（未提交 `pub mod nt_meta_cleanup;`）⇒ 禁删 |
| `l5_cognition/nt_core/io_skills` | 85 行纯 stub，孪生体已删 | ⛔ **正因为孪生体已被 `d065591b` 删除，它现在是唯一副本** ⇒ 我的论证方向反了 |

⇒ **一个都没删。** 本仓既有标准（`d065591b`）：**只有内容等价证明才构成删除理由**，
时间戳、文件名、符号名集合都不可靠。这三个都不满足。

---

## 6. 遗留（**都需先裁决**，本批未动）

| 项 | 阻塞点 |
|---|---|
| 13 个孤儿目录的处置 | 3 个建议删但已被否决（§5）；4 个接线后仍是死的（**接线等于关闭监控**）；1 条依赖链须先裁决情绪枚举正源 |
| 1,966 条 `ZERO_CONSUMER` | **693 条判据只是「目录名含治理词」= 弱信号噪音**；真正可行动的是 name/method 判据那 ~693 条 |
| 「下载后执行」规则 | `sh < payload` / `bash <(curl x)` / `curl … && sh` / `;` / 换行 —— 现有规则只按 `\|` 切分 |
| `agent_guardrails` **输出侧** | 幻觉/不安全代码/数据外泄检测仍零生产调用方（输入侧已接） |
| 1,466 行陈旧 patch | `git apply --reverse --check` 已失败 ⇒ 零恢复价值，建议删（未擅自删 untracked） |

---

## 7. 收工自查

### 7.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` → **exit 4**，摘要：

```
[worktree-gate] worktree=4 个 | 合计 13177M | target 占 12866M
[worktree-gate] 带未提交改动: 2 个 | 近3h有改动: 1 个
[worktree-gate] ⛔ 2 个 worktree 的未提交改动不在任何提交里：
[worktree-gate]      ⛔ .worktrees/merge-b
[worktree-gate]      ⛔ .worktrees/nt-v4
```

- 本会话**新建 worktree 数量：0**（4 个全非本窗口；worktree 数已由 6 降至 4，
  是其他窗口收掉的）。
- 临时验证 crate（非 git worktree，不在门视野内）：`astar-probe`、
  `guard-probe2/3/4/5`、`final-check` —— **全部已 `rm -rf`**。

### 7.2 未提交改动的去向

本会话结束时**我自己的代码改动全部已入库**（5 个提交，逐个 `git commit --only`）。
主树剩余未提交项全部是他窗在途 WIP，本会话**未触碰、不声明去向**。

| 文件 | 去向 |
|---|---|
| `nt_action_facade.rs` · `nt_agent_exec.rs` | ☑ 已提交 `9c6eae6c` |
| `agent_guardrails/{mod,input_validator,policy_engine}.rs` | ☑ 已提交 `a10342a2` + `9c6eae6c` |
| `guard/mod.rs` | ☑ 已提交 `f883230d` |
| `scripts/ops/nt_security_wiring.py` · `scripts/security-wiring-baseline.txt` | ☑ 已提交 `5c81f3ac` |
| `Makefile` · `.github/workflows/ci.yml` | ☑ 已提交 `5c81f3ac` |
| `docs/architecture/ABSORPTION-RUST-GAME-SOURCES-2026-10-06.md` | ☑ 已提交 `4dc52fea` |
| `.neotrix/patches/2026-10-05-*.patch`（1,466 行） | ⚠️ **明确弃用** —— `git apply --reverse --check` 已失败 ⇒ 零恢复价值，属「陈旧记录比没有更危险」。未擅自删 untracked 文件 |

### 7.3 门状态

- `nt_worktree_gate.sh check` exit **4**（带未提交改动的 worktree 存在 —— 全为他窗）
- 提交前是否跑过 `cargo test -p neotrix --lib`：
  - `5c81f3ac`：✅ `--self-test` 28 项全绿 + `--strict` 绿 + `--audit` 统计
  - `f883230d`：✅ 全量 **13319 passed / 1 failed**（那 1 条他窗在途）
  - `a10342a2`：⚠️ 提交时**未能**跑（他窗 staged 删除 + `#[cfg(test)] mod` 声明残留阻断，
    持续 40+ 分钟）。**提交后阻塞解除即补跑：`agent_guardrails` 45 passed / 0 failed** ✅
  - `4dc52fea`：N/A（纯文档）
  - `9c6eae6c`：✅ `agent_guardrails` **46 passed / 0 failed**；
    全量 **13358 passed / 1 failed / 38 ignored**
    （那 1 条 `mod_orphan::scan_tree_sorted_by_lines_desc` 是他窗已提交的
    `db24c44a`/`64913227` 引入新分类却没同步测试夹具期望；本提交在
    `l0_substrate/` 下 diff **为空**）
- `nt_lock_audit.py`：每个 `.rs` 提交前均重跑 —— `neotrix-core/src` **0 处**（RC=0）、
  `crates/neotrix-neobot/src` **0 处**（RC=0）
- `cargo clippy -p neotrix --lib`：本批改动文件**零 lint**
  （整体红在 `crates/neotrix-types/` 的 `allow_attributes_without_reason`，已入库代码撞新 lint）
- 门红归属：全部为他窗 WIP，非本会话引入