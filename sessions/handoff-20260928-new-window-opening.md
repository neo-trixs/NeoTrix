你现在接手 NeoTrix 仓库（`/Users/neo/Downloads/neotrix`）的**统一修复**。之前有 8 个窗口并行工作过，每个都留了交接。你是唯一被指派继续的窗口。

## 第一件事：读文档，不要先动代码

按这个顺序读（**8 份交接不要通读，按下表取用**）：

| 顺序 | 文档 | 作用 |
|---|---|---|
| 1 | `sessions/handoff-consolidate-all-windows-20260928.md` | 另一窗口的汇总总览，含**三条硬约束**（内存闸/禁并发构建/禁 `git add -A`） |
| 2 | `sessions/handoff-20260928-consolidated.md` | 我这份，含 **§2 勘误表（7 条已被证伪的台账前提，动手前必读）** + T1~T10 执行顺序 + **§4 九个坑** |
| 3 | `docs/architecture/LESSONS-20260928-fresh-checkout.md` | 方法论教训 10 条，元教训在最后 |
| 4 | 其余 6 份 `sessions/handoff-*.md` | 其他领域的债（neobot / crystal / cocoons / hf-batch / medical / test-debt），**按需查阅**，不必通读 |

两份汇总**互补不冲突**：他那份覆盖 neobot/crystal/吸收批次，我这份覆盖构建/门禁/测试债。他已把转述其他窗口的内容标注为「未逐条复核，接手前请自己复核」。

## 三条硬约束（违反即阻塞）

1. **内存闸 BLOCKED 时禁止 cargo**：`sh scripts/ops/nt_mem_gate.sh; echo $?` 必须为 0 才起构建。注意用 `rc=$?` 捕获，别用 `if [ $? -eq 0 ]`（那读到的是 `echo` 的退出码）
2. **禁多窗口并发** `--all-targets` / `--test`，`CARGO_BUILD_JOBS=1`
3. **禁 `git add -A` / `git reset --hard`**，主树有约 960 个他人未提交改动

## 最容易让你浪费几小时的一件事

**主工作树因含他人未提交修复而「看起来正常」。它不是可信的地面真相。**

- 本轮有 57 条测试失败，其中 **37 条在工作树里早已修好、只是从未入库**
- `check-truth-surface --strict` 在一个跑不了 cargo 的仓库上报 **0 offender**
- 我曾三次差点因此做错决策：把 CI 说成"绿的"、把 `supersedes` 说成"死字段"、把 4.1 的前提当成成立

所以：**任何「X 是好的/坏的」断言，只能在干净检出上验证**。

```bash
W=$(mktemp -d)/wt; git worktree add --detach "$W" HEAD && cd "$W"
# …在这里跑真实命令，最后 git worktree remove --force "$W"
```

## 台账里已被证伪的前提（照原文做会出错）

| 项 | 台账原文 | 实测 |
|---|---|---|
| 4.1 | 给决策引擎加 JEV 四件套 | **前提证伪** —— 三个同名引擎全无生产消费者，已标 ⛔ |
| 2.1 | supersession 要新建 | 记忆库层**早已实现**（91 测试全绿），缺的只有 experience_tree 自身 |
| 2.2 | 真双时间从零做 | `temporal_facts` **已经是真双时间**，缺的只有 `nodes` 表 |
| 5.2 | `McpRegistry` 在 `#[cfg(test)]` | **错** —— 在生产面且已接线；真缺口是客户端不发 capability 协商字段 |
| 4.4 | `nt_shield_audit` 覆盖率账本 | 该模块**同样未接线**（只被一个事件名字符串引用） |
| 路线图 §1.4 | 改 `paged_kv`/`kb_kv`/`vector_index` 做两表 | 三处**全是内存结构、无一张表**，方向指错 |

## 当前真实状态（已实测，别重新怀疑）

```
cargo check --lib -p neotrix     exit=0
cargo check -p neotrix (bins)    exit=0
cargo test  --lib                 12143 passed / 3 failed
check-fresh-build.sh --full       PASS
check-layer-deps.sh --strict     PASS (0 new / 102 known)   ← 勘误见下，原写 94
check-truth-surface.sh --strict  PASS
nt_lock_audit.py                  0 处
```

**3 条剩余失败** = `nodes` 表无法存双时间（2 条，阶段级工程，见交接 T6）+ Noise 握手协议（1 条，⛔ **需产品决定**）

**分层违规 102 条** = 92（l\*_ 层）+ 10（`neotrix/` 第二棵树）。棘轮已就位，`--update-baseline` **只应向下** —— 调大基线等于把债藏起来。

> ### ⚠️ 上面「94」是脏树测量，照抄会打断 CI（`bdf1e9f1` 已修，15:2x 实测）
>
> 那 8 条 `l*_` 之所以从 92 掉出基线，只因**主工作树有 393 个未提交的 `.rs` 修复** ——
> 新 clone 里它们是真实违规。干净检出三态：
>
> | 状态 | baseline | violations | RC | 第二棵树覆盖 |
> |---|---|---|---|---|
> | `[A]` HEAD 原样（= 新 clone） | 92 | 92 | 0 | **0 个文件** |
> | `[B]` 照抄 94 | 94 | 102 | **1** | 有 |
> | `[C]` 干净检出重算（**已入库**） | **102** | 102 | 0 | 有 |
>
> `[A]` 是最危险的：门报 `PASS: 0 new`，却**完全没看** `neotrix/` 第二棵树的
> 128 文件 / 42,070 行 —— 因为 `.neotrix/layer-map.json` 与 `check-naming.sh`
> 当时只在主树未入库。
>
> **102 不是「调大基线藏债」**：92 是**残缺门**下的债，102 是**完整门**下的债。
> **元教训**：这是「脏树不是合法测量台」的**第三次**复发 ⇒ 写进门记录/正典文档的
> 数字必须记「测量台 = 干净检出 @ commit」。同一批修正：`naming` 基线
> 1,644（脏树）→ **1,646**；`layer-map.json` 首版 8 个数字 7 个是脏树值
> （最大 `nt_crystal_core` 21,134 → **19,371**，−8%）。

## 从哪开始

交接文档 §3 给了 T1~T10 的依赖与阻塞关系。建议起步：

- **T2 DNS 白名单**（1.1）—— 定位已完成可直接执行，零编译风险起步
- **T4 MCP capability 协商**（5.2）—— 真缺口已确认，有生产消费者
- **T5 experience_tree supersession**（2.1）—— 照 `temporal_facts` 已验证形态做，文件 clean 无归属冲突
- **T6 nodes 表双时间**（风险最高）—— 建议照 T5 的「每版本独立 id」形态做，**可完全避开复合主键与外键问题**

**T7（Noise 握手）先别动**，需人拍板：对齐 spec 还是明确降级。

## 工作方式要求

1. **动手前先验证前提**（本轮 4 次"前提证伪"省下大量返工）
2. **不要用过滤式 grep 下结论** —— 我用 `grep -v "supersedes: None"` 得出"死字段"的错误结论，恰好滤掉了所有真实写入/读方
3. **判提交成功看 `git show --stat`**，不要信提交前的 `git diff --cached`（pre-commit hook 会改 index）
4. **不要碰其他窗口的 960 个在制品**。若必须提交跨文件变更，用「worktree 内提交 + `git update-ref <ref> <new> <old>` CAS 推进分支 + `git reset`(mixed) 同步索引」；若需部分暂存某个文件，用内容标记分离 hunk 后 `git apply --cached`（我全程用这个方法，他窗对 `AGENTS.md` 的 7 行改动完好未被我吞）
5. **mtime 晚于当前小时 = 他方在写**，换文件或先通报
6. Python 改文件时：**切片赋值必须传 list**（传 str 会按字符摊平）；含反引号的 heredoc 必须用 `<<'PY'`

## 过程中请更新记录

`TODO.md` 是唯一任务台账。做完一项就更新状态 —— 但注意本轮教训：**同一件事不要重复开多节**，我曾把测试失败记了 3 节且互相矛盾（含一节写着"未解决"），已收敛为唯一权威结论。

新发现的问题记到 `docs/architecture/DIR-AUDIT-2026-09-27.md` 或对应正典文档，别只留在对话里。
