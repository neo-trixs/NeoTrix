# handoff — 吸收 Anthropic《AI-native SDLC playbook》(2026-10-06)

## 1. 会话标识

- 日期：2026-10-06
- 任务：读 `https://claude.com/blog/the-ai-native-sdlc-playbook`，与本仓实测对账后**吸收完善能力**
- 窗口：与另两个窗口**并行**（见 §4 风险）
- 产物：5 个新文件 + 6 个修改（清单见 §8.2）

## 2. 一句话结果

**接了 3 处真缺口（G1 工件链 / G2 配置无门 / G3 两套严重度并存），
拒绝照抄 6 项（G7 其中最险的是 `bands.yaml` —— 本仓早已有 `AutonomyLevel` 且已接线）。**
新增门 `check-agent-config.sh` 已同会话进 CI + 登记 + 入索引 + 探针证非空（R-P79）。

## 3. 已完成（可复核）

| 产物 | 补的缺口 | 证据 |
|---|---|---|
| `docs/plans/_TEMPLATE.md` | G1 无每变更工件 | 9 节工件模板，`status:` 推进 |
| `REVIEW.md` | G3 两套严重度并存 | 对齐 `Severity::numeric()`，9 级正典 |
| `scripts/check-agent-config.sh` | G2 配置无回归门 + G4 过程指标空 | 6 项判据 C1–C6 |
| `scripts/probes/check-agent-config.sh` | 非空性 | 2 处注入，各自指名注入对象 |
| `scripts/artifact-chain-baseline.txt` | 棘轮 | 66 件存量，不追溯 |
| `docs/architecture/ABSORPTION-AI-NATIVE-SDLC-2026-10-06.md` | 出处 | 含「不照抄的 5 项」+ 一条既有红门 |

**接线（R-P79 四处全接，缺一即等于没做）**：
`ci.yml` 有跑点 · `gate-registry.tsv:20` 已登记（injectable）· `task-index.json` 第 79 条可检索 · `AGENTS.md` §6 外部吸收行已挂 · `docs/architecture/README.md` 已入阅读索引。

**本仓已更强因而未照抄**：`gate-registry.tsv` 门非空/可满足双判据 · 9 份探针 ·
`nt_gate_coverage.py` 接线元门 · `AutonomyLevel` + `required_autonomy()` 13 类映射（**已生产接线**）。

## 4. 与其他窗口改同一文件？（§4 必答）

**未改任何 `.rs`。** 但工作区里有 **17 个不是我的文件**，其中 2 个是别窗在途源码：
`neotrix-core/src/l6_meta/nt_approval.rs`、`crates/neotrix-types/src/core/nt_core_approval.rs`、
另加 `Cargo.toml` / `.cargo/config.toml` / `.neotrix/capability_registry.json` / 7 个 patch。

⚠️ **`git add` 时 index.lock 被占**：实测另有**两个窗口**在跑 `git commit --only`
（PID 30299 提交 `nt_approval.rs`，PID 34920 提交 `input_validator.rs`，都带 `sleep` 重试循环）。
**我未删任何 lock、未抢 index**，改用有界轮询（`for i in seq 1 12; ... sleep 15`）等待后 `git add` 成功。

## 5. 遗留判断（需要人来定，我不该替他定）

1. ⛔ **`check-ci-refs.sh --strict` 在 main 上本来就是红的**（clean-HEAD 实测）：
   `ci.yml:50` 用 `actions/setup-python@v5`，但 13 条溯源清单里**没有**它。
   **我没有修，也不该顺手修** —— 补它需要该 action 的真实 `sha256`，那要联网取。
   **编一个哈希进溯源清单比红门坏得多**（把「无记录」换成「假记录」）。
   ⇒ 留给有网的人按 `nt_shield` 溯源规程补。
2. `check-skill-gate.sh` strict 恒红（存量 59 条缺 frontmatter）。本门**故意不复用它**，
   只守引用完整性。存量清理由别的议题处理。
3. `REVIEW.md §6` 三个指标（首次审查耗时 / 发现被门重复报出的比例 / 逃逸缺陷）**明确标注未接线**。
   要接线需要 `nt_sdlc_metrics.py`，本轮没做 —— 造一个没人跑的仪表不如留一个诚实的缺口。

## 6. 验证数字（clean-HEAD 工作树实测，非推理）

测量台：`git worktree add --detach HEAD`（AGENTS.md §4.2 要求，脏树值不可信），跑完已 `git worktree remove`。

| | clean HEAD | 本会话后 |
|---|---|---|
| `--strict` 门总数 | 18 | 19（+1 = 我的） |
| 已登记 | 17 | 18（+1） |
| **恒红门** | **5** | **4** |
| 探针失败 | 1（`check-doc-claims` rc=2） | 1（同一处） |

**⇒ 我引入 0 个新红；恒红数从 5 降到 4**（`check-license-js` 转绿是他窗 pnpm-lock 改动，非我）。

我改动涉及的 11 道门全绿：`check-agent-config` / 其探针 / `check-layout` /
`check-untracked-assets` / `check-truth-surface` / `check-orphan-dirs` / `nt_gate_coverage` /
`nt_find --audit` / `nt_manifest audit` / `check-doc-claims` / `nt_scan_surface`。

**既有红（非我引入，已归因）**：`check-ci-refs`（setup-python 溯源缺口）、
`check-doc-drift`（NEW offender `nt_core_event_bus.rs`，别窗/既有）。

## 7. 两条本轮踩到并修掉的自身 bug（留作教训）

1. **`$VAR（` 吃字节**：全文角括号后 bash 把 `$REVIEW（` 当**一个**变量名 →
   `REVIEW�: unbound variable`。门与探针各中一次。修法 `${REVIEW}`。
   ⇒ 这正是本仓 `scripts/probes/_lib.sh:80-83` 已记录的坑，**我照样踩了** ⇒
   写含中文标点的 bash，`rg -n '\$[A-Za-z_][A-Za-z0-9_]*[^A-Za-z0-9_ "$/{.=)\]}!\[\-]'` 应成为收尾自检。
2. **`rg -oh` 打印帮助**：ripgrep 的 `-h` 是 `--help` 不是 `--no-filename` ⇒
   一次统计动作返回了 124 行 help。⇒ 统计类命令改用 python，**别信 rg 的 `-h`**。

## 8. 收工自查（必填）

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` 输出：

```
[worktree-gate] worktree=2 个 | 合计 147M | target 占 0M
[worktree-gate] 带未提交改动: 1 个 | 近3h有改动: 0 个
[worktree-gate] ⛔ 1 个 worktree 的未提交改动**不在任何提交里**：
[worktree-gate]      ⛔ /Users/neo/Downloads/neotrix/.worktrees/merge-b
[worktree-gate]    删它们必须先 patch 兜底（R-DISK-5）：sh scripts/ops/nt_worktree_gate.sh prune
```

本会话新建的 worktree：

| worktree | 用途 | 去向 |
|---|---|---|
| `$TMPDIR/opencode/ntx-baseline` | clean-HEAD 基线测量（AGENTS.md §4.2 要求） | 跑完确认 dirty=0，已 `git worktree remove` 移除 |

**`.worktrees/merge-b` 不是本会话建的**（本会话一个 `.worktrees/` 下的 worktree 都没建），
故未动它 —— ⛔ 禁手删，且它带未提交改动。

### 8.2 未提交改动的去向

**本会话 13 个文件：7 个新文件已 `git add` 入暂存区；6 个既有文件为工作区修改（未暂存）。**
未提交 —— 提交前请先 `git diff --cached --name-only` 核对暂存区是否被别窗污染。

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `REVIEW.md` | 新增 · 审查策略正典 | ☐ `git add` 已暂存 |
| `docs/plans/_TEMPLATE.md` | 新增 · 工件模板 | ☐ `git add` 已暂存 |
| `docs/architecture/ABSORPTION-AI-NATIVE-SDLC-2026-10-06.md` | 新增 · 吸收出处 | ☐ `git add` 已暂存 |
| `scripts/check-agent-config.sh` | 新增 · 配置回归门 | ☐ `git add` 已暂存 |
| `scripts/probes/check-agent-config.sh` | 新增 · 非空门探针 | ☐ `git add` 已暂存 |
| `scripts/artifact-chain-baseline.txt` | 新增 · 棘轮基线（66 件） | ☐ `git add` 已暂存 |
| `scripts/gate-registry.tsv` | +1 行登记我的门 | 未暂存（仅工作区改） |
| `scripts/check-layout.sh` | ALLOW_FILES +`REVIEW.md` | 未暂存 |
| `.github/workflows/ci.yml` | +16 行跑点 | 未暂存 |
| `.neotrix/task-index.json` | +1 条索引 | 未暂存 |
| `.opencode/agent/review.md` | 对齐 REVIEW.md，删自造词表 | 未暂存 |
| `AGENTS.md` | 75→79 条索引（陈旧数）+ §6 挂出处 | 未暂存 |
| `docs/architecture/README.md` | 按需查阅 +1 条 | 未暂存 |

⛔ **提交必须用 `git commit --only <上表 13 个文件>`** —— 暂存区里可能已有别窗内容
（`nt_gate_coverage.py` 已实测共享 index 会连带提交他窗删除）。**提交前先 `git diff --cached --name-only` 核对。**

### 8.3 门状态

- `nt_worktree_gate.sh check` 的 exit code：`3`（1 个带脏改动的 worktree，**非本会话所建**）
- `cargo xl` / `cargo check`：☐ **否** —— 本会话**未改任何 `.rs`**，零编译影响。
  （新增的 6 个文件全是 `.md` / `.sh` / `.txt` / `.json`，不参与任何 crate。）
- 门红归因：**全部为他窗 WIP 或既有债，本会话引入 0 个**（归因表见 §6）。
- ⛔ 未跑 `--all-targets` 全量构建：`nt_mem_gate.sh` rc=0 但当时有另两窗在跑 git commit；
  且无 `.rs` 改动，全量构建无验证价值。
---

## 9. 补记（同会话后续，2026-10-06 晚）—— §8.2 已作废，两件事都落了

§8.2 写的「**未提交**」现已不成立。实际落账：

| 提交 | 内容 | 文件数 |
|---|---|---|
| `a131d9bd` | 变更工件链 + `REVIEW.md` + `check-agent-config.sh` + 探针 + 棘轮基线 + 索引/接线 + 本交接件 | 14 |
| `149af337` | `actions/setup-python@v5` 溯源补记（**修掉 main 上既有红门**）+ `nt_sdlc_metrics.py` 落地 + `REVIEW.md §6` 换成实测数 | 5 |

**两个提交都完整在 HEAD 历史里**（`git merge-base --is-ancestor` 已核实）。
中间夹了别的窗口的 `2cc4badf` / `1f37626e` —— 那是并发窗口各自的提交，
`1f37626e` 只含它自己的 3 个 LESSONS/handoff 文档，**没有捎带我的文件**。

### 9.1 上一版 §5「遗留判断 1」已解决

原文说 `check-ci-refs.sh` 红是「需联网、留给有网的人」。后来**发现有网**，
且该溯源清单 `_comment` 自带离线可复现流程 ⇒ 按流程**实测**补记
（ls-remote 取 commit + codeload tarball 取 sha256，**两次独立下载 sha256 一致**，
`file` 确认是真 gzip tarball 而非错误页）。

⇒ `check-ci-refs.sh --strict` **rc=0**；`provenance_check.sh` **14/14 PASS**；
**恒红门 4 → 3**（clean-HEAD 基线是 5）。

⚠️ 遗留：`@v5` 是**可移动 tag**，记录只对那一刻成立。真要不可变须改
`uses: @<commit SHA>` —— 独立决策，未做。

### 9.2 原 §5「遗留判断 3」已部分解决

`REVIEW.md §6` 拆成两节：§6.1 是**真能测**的 3 项（工件覆盖率 0.9% /
工件→diff 漂移 NO_BASELINE / 返工代理 1.5%），§6.2 是**测不出**的 3 项
（如实标 `UNMEASURED` + 写明为何测不出）。**没有造没人跑的空仪表。**

⚠️ `--self-test` 当场抓到我自己的两个真 bug（详见提交说明）：
① coverage 窗口建在「只含 code commit」的列表上 ⇒ 工件单独成 commit 时永远进不了窗口；
② `git log -n N` 配 pathspec 与不配 pathspec **是两个不同的窗口**。
**两个都是跑出来的** —— 没有 `--self-test`，指标就会永远输出 0，而 0 与
「写坏了」不可区分。

### 9.3 仍未修（需要 cargo 或产品码改动，非本会话范围）

`check-doc-drift`（NEW offender `nt_core_event_bus.rs`）· `check-unwrap` ·
`check-silent-failure`（后两者存量债，`gate-registry.tsv` 已记）·
`check-doc-claims` 探针 rc=2（门有效性未获证，既有）。

### 9.4 并发实况

本会话全程与**两个窗口**并行。它们在跑 `git commit --only` 提交
`nt_approval.rs` / `input_validator.rs`。本会话**未删任何 index.lock**，
一律用有界轮询等待（`for i in seq; git add … || sleep`）。

---

## 10. 第二轮收尾（2026-10-06 晚，`cdaf48a8`）—— 修的是**门自己**，不是代码债

第一轮遗留的三项，本轮全部处理。**三处都是门有 bug**，逐条先读现场再动手。

| 项 | 真相 | 处置 |
|---|---|---|
| `check-doc-claims` 探针失败 | ⛔ **门是好的，探针陈旧**：`noise_ik` 已接进生产（TODO.md:642 自己记着），注入的「零消费者」断言变成真的 | 探针改**运行时自选符号**（硬编码符号必然腐化） |
| `check-doc-drift` 3 个 deadlink | 1 真（`nt_io_eli5.rs` 已由 `d065591b` 删除但文档没说）+ 2 假（探针注入文件按设计只在探针运行期存在） | TODO.md 补 3 处订正 + 门加**双条件**窄判据 |
| `check-orphan-dirs` 未登记 | 可注入但没人登记 | 补探针 + 登记 |

### 10.1 三条自曝（都是「跑出来的，不是看出来的」）

1. ⛔ **我先把 `check-doc-drift` 改坏了**：给关键词表加 `删除`（原只有 `已删`）后门变绿，
   但我**不信绿**、注入真死链复测 ⇒ **没抓到**。根因：TODO.md 212KB 里 `删除` 满地都是，
   ±3 行窗口几乎必然命中 ⇒ 真死链全被跳过。已撤回。
2. **探针的字节安全判据用错了**：旧版 `git diff --quiet` 把**我自己的合法未提交改动**
   误判成「探针没还原」（TODO.md 那 3 处订正）⇒ 探针假失败。改为与**探针开始时校验和**比对。
   ⛔ 这正是本仓反复吃的错：**分不清「我的改动」与「残留」**。
3. **`$VAR（` 全角括号坑，本轮又踩 3 次**（门 1 + 探针 2）。已写收尾扫描脚本自查。

### 10.2 ⚠️ 一条元教训：探针绿 ≠ 门没被改坏

`check-doc-drift` 的探针只覆盖该门的**「无 module doc」判据**，**不覆盖 deadlink 分支**
⇒ 我把 deadlink 分支改坏时，**探针照样绿**。若我当时只看探针，就会把一个坏门提交。

⇒ 推论：**门的探针只覆盖它自己声明的那条判据**。改门必须另做针对性复测，
不能拿「探针绿」当「门可信」。本轮我是靠注入真死链才发现的。

### 10.3 我的 doc 被别人的 commit 捎带了（如实说明）

`nt_core_event_bus.rs` 的 module doc 落在 `854d4131 style: 清掉装饰性 ⭐`（别的窗口），
**不是**我的 commit。内容正确、`check-doc-drift` 绿，只是归属不同。
根因：我把它留在工作区没及时 `--only` 提交，共享 index 把工作区改动卷进了别人的 commit。
⇒ 与 2026-09-29 那次同型：**及时 `--only` 提交是共享树下唯一护栏。**

### 10.4 元门最终读数

| | clean HEAD | 现在 |
|---|---|---|
| `--strict` 门总数 | 18 | 19 |
| 已登记 | 17 | **19**（未登记 0） |
| **恒红门** | **5** | **2** |
| 探针执行失败 | 1 | **0** |

### 10.5 仍未修（非我引入，逐条查证过）

`check-unwrap` 24 NEW / `check-silent-failure` 5 NEW —— **全在他窗在途文件**
（`nt_governance.rs` / `coverage_ledger.rs` / `resonance.rs` / `nt_pet.rs` /
`agent_orchestrator.rs` / `traits.rs`）。⛔ 不动手：改别人正在写的文件
正是 AGENTS.md §3 碰撞事故的定义。

⚠️ 其中 `nt_core_event_bus.rs` 那条：基线记 112 行、HEAD 实际 154 行 ⇒
**在我动手前就已 stale 20 行**（他窗提交所致），我加的 22 行 doc 只是让它更远。
**不是我的 doc 制造的违规。**

### 10.6 收工义务（本轮）

- `nt_worktree_gate.sh check`：本轮**未新建任何 worktree**（第一轮那个基线测量用的
  已 `git worktree remove`）。
- 本轮改动 5 个文件全部 `--only` 提交（`cdaf48a8`），暂存区已清空。
