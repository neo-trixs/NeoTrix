# handoff-layout-gates-20260929 — 目录治理 + 门修复 + CJK 分词

> 会话：2026-09-29 约 15:50–20:30
> 状态：**全部产出已本地提交，未推送**（远端仍在 `ff672104` / 2026-09-17）

---

## 1. 一句话

从「梳理 9 目录 + 13 文档」开始，最终交付 **3 件事**：把目录/文档规范从散文变成门、
清掉两批已失效的残留、修了权威分词函数对中文的失明。过程中我自己出了两次事故，
都写进了 AGENTS.md 的规则。

---

## 2. 我做的（10 笔，按主题）

### 2.1 CI 门（此前从未真正工作过）

| 提交 | 内容 |
|---|---|
| `09b1c2de` | **gitleaks 是三重恒绿幻影**——路径错（`--config .gitleaks.toml` 不存在，真身 `config/`）+ `\|\| true` 吞退出码 + 判据 `grep '"Severity"'` 恒不匹配（gitleaks JSON 报告**没有** Severity 字段）。三层嵌套，任何一层单独修都不够。判据改为**基线棘轮**，双向注入验证 |
| `0a674087` | gitleaks **版本对齐**（基线按 8.30.1 生成，CI 却装 8.18.2；8.18.2→8.30.1 有实质规则变更）。加版本闸 + `check-gitleaks-version.sh` 自门，接在**扫描之前** |

### 2.2 目录/文档治理

| 提交 | 内容 |
|---|---|
| `45b522bf` | 根目录 **13 → 10 文档**（删 3 份：2 份零引用废止件 + 1 份 ARCHIVED 桩；删 `config/` 4 份孤儿含一个与根目录**同名但内容不同**的 `deny.toml`） |
| `40b9f022` | `skills/` **21 → 16 目录**（删 5 个已吸收/无用；其中 `mcp-gateway` 是**幻觉文档**——其 `## Built-in Tools` 列的 API 在全仓 `.rs` 零命中）。修 2 处恒失效的自检协议 |
| `7fef5315` | **14 条既存死链清零**。查出 **2 处不实记述**（TODO 声称的 `nt_ipc_keys.py`/`nt_smoke.sh` 从未入库，那组「实测 97/97」是推演）。一处**永久丢失**（`handoff-neobot-absorption-20260928.md` 三处恢复途径逐一核实全落空） |
| `ce794ff7` | **立 `check-layout.sh`**——`DOCUMENTATION-MAP.md` 的 4 条「禁止」此前全是散文，实测根目录放文件三道门全过。⇒ **一次清扫是临时的，门才是长期的** |
| `1654f994` | `apps/`（43）+ `src-tauri/`（15）残留移出到 `Neo/neotrix-archive/desktop-residual-20260929/`，`mv` 非 `rm`，**58/58 SHA-256 逐字节校验一致** |
| `77eefa9a` | 登记 2 条实测待办（含 8 个 `is_cjk` 副本的环形依赖取证） |

### 2.3 门自身修复

| 提交 | 内容 |
|---|---|
| `a053076d` | **删除声明门在 `--only` 提交下永远读不到本次消息**（git 对 pre-commit 传 0 个参数 → 恒走 `COMMIT_EDITMSG` 回退 → 而 `--only` 不写该文件）。**两条纪律互锁死**：越需要声明的场景越声明不了 |
| `f78b56a0` | `keywords()` CJK 失明（见 §4） |

### 2.4 文档

`33c88056` 补 `dev-rules.md` 删除等价性的**实证**（11+60 绿，含对照实验：桩存在与桩删除两态结果完全相同）。

---

## 3. 我自己出的两次事故（已写进 AGENTS.md）

### 3.1 ⛔ 删掉 115.2 GiB（AGENTS.md R-SCAN-4）

新写的 `check-disk.sh` 在「建议」文案里写了**反引号包裹的示例命令**：

```bash
echo "  建议：优先 `cargo clean -p <crate>` 局部回收"
```

干跑时 bash **真的执行了** `cargo clean` ⇒ 删掉 115.2 GiB，当时有 4 个他窗 cargo 在构建。

**损失**：`target/` 是 gitignored 纯生成物 ⇒ **源码与 git 状态零损失**（他窗的
`audio_decode.rs`/`thumbnail.rs`/`Cargo.lock` 改动逐项验证完好），代价只是重跑编译。

**教训**（R-SCAN-4）：① 反引号在 bash 里是命令替换不是排版，**`bash -n` 抓不到**；
② 门脚本的干跑本身可能有副作用；③ R-SCAN-2「喂真实输入跑」的前提是先读码确认无副作用。

### 3.2 ⛔ `git stash pop` 失败

为验证「既存测试失败与我的改动无关」用了 stash 做对照实验，pop 时撞上他窗并发改动
（`async_tool_executor.rs`）⇒ 失败。**核查为零损失**：`stash pop` 检测到冲突即中止，
未覆盖任何文件，他窗 6 个脏文件 + 217 个未跟踪文件全完好。

**教训**：共享工作树里**不该用 stash 做对照实验**。

---

## 4. CJK 分词修复（`f78b56a0`）

`CrystalConsciousness::keywords` 按空白/标点切分 ⇒ 中文整句只成 1 个 token：

```
keywords("我的支付一直失败收不到验证码") -> ["我的支付一直失败收不到验证码"]
keywords("支付网关")                     -> ["支付网关"]   # 永不相交
```

**关键发现**：同目录 `nt_shared_mind.rs:22` 与 `nt_crystal_task_fusion.rs:57`
**早就实现了 CJK bigram**，只有 `consciousness.rs:561` 漏了。而本函数按其自身注释
是**权威分词**（外部评测口径刻意与它绑定）⇒ 按既有实现对齐，不造新东西。

**测试**：该文件此前没有任何测试模块。新增 6 个，TDD 红→绿。红阶段时另外 4 个
（英文口径/停用词/退化输入/src tag）**当场就绿** ⇒ 证明只需改 CJK 切分。

⚠️ **一个既存失败不是我的**：`test_decide_calibrated_identity_matches_plain` 浮点
1-ULP 断言（`…17926` vs `…1792`），已用 stash 回退复测，回退后同样失败。未修。

---

## 5. 门的现状（22 道）

新立 3 道，全经**注入测试**（不是手推）：

| 门 | 拦什么 | 验证 |
|---|---|---|
| `check-layout.sh` | 根目录白名单 / `neotrix-core/docs/` 禁无日期前缀 / **与 DOCUMENTATION-MAP 交叉校验**（规范腐化） | 3 类注入全抓到，`--strict` exit 1；恢复后 0 |
| `check-disk.sh` | `target/` 膨胀 | advisory 恒 exit 0（R-DISK-1：有 cargo 在跑时由人决定何时清） |
| `check-gitleaks-version.sh` | 基线/门版本错配 | 注入 2 类全抓到 |

**门自门**：未登记 0 / 恒红 0 / 探针失败 0。

### 门自身踩的 4 个 bug（都修了，值得记）

1. 白名单尾斜杠不匹配 ⇒ 25 项误报
2. `find` 路径拼重复
3. **`$VAR` 紧跟全角标点**，bash-3.2 参数展开吃掉字节 ⇒ `unbound variable`（**本会话踩 3 次**）
4. ⛔ **`if ! cmd | tail -8; then` 是死代码**——管道退出码取自 `tail`（恒 0），
   分支永不执行，**门形同虚设而我以为拦住了**。修法：先落 mktemp 再判退出码。
   ⚠️ 与「⛔ pre-commit 门防不了」同源：**门自己的失败路径也要测**

---

## 6. 未完成（下一窗接手）

| # | 事项 | 阻塞 |
|---|---|---|
| 1 | **`is_cjk` 8 副本统一**（口径 4 种、无一为 `pub`、两个「单一事实源」互指且 `context_budget` 根本不存在） | 需编译验证。详案见 `TODO.md` 待办 11 |
| 2 | **IPC 键名校验器重做**（原 `nt_ipc_keys.py` 从未入库，缺口仍敞开） | 详案见 `TODO.md` 待办 12 |
| 3 | **2,496 行冒烟测试可能已丢失**（`nt_smoke_*.rs` 被 `d5413335` 删除，而 `crates/neotrix-neobot/` 无 `tests/`） | 复活方法：`git show 620e9712 -- apps/neobot-desktop/tests/` |
| 4 | 词干还原（`{"invoice"} & {"invoices"} == ∅`） | 需外部依赖或自研规则，另一个决策 |
| 5 | `profile.dev` 要不要关 `incremental`（曾占 64G） | 纯取舍：省磁盘 vs 每次全量重编 |
| 6 | `neotrix-core/docs/` 3 个既存违规（`{FUSION,ANALYSIS,CRYSTAL}_ARCHITECTURE.md`） | 已在账本，处置需决策 |

---

## 7. 收工自查（模板 §8）

**worktree 去向**：本会话**全程用主树，未开 worktree** ⇒ 无需 `prune`。
`git worktree list` 显示 `/private/tmp/nt-v9`（detached `a005db44`）是**他窗的**，
带未提交改动**不在任何提交里** ⇒ ⛔ **勿删**（`worktree-gate` 亦如此判定）。

**我的未提交改动**：**零**。`git status` 里剩 5 个文件全是他窗的
（`Cargo.lock` / `security.rs` / `audio_decode.rs` / `thumbnail.rs` / `results.tsv`）。
⚠️ 暂存区里也有他的 2 个文件——`--only` 每次都把它们挡住了，**别裸跑 `git commit`**。

**推送状态**：**未推送**（按用户指令）。远端 `ff672104` / 2026-09-17，
本地领先 **519** 个提交。⚠️ 这是**唯一副本**——`.git` 不会自动同步到别处。

**归档区**：`/Users/neo/Downloads/Neo/neotrix-archive/desktop-residual-20260929/`
（59 文件，含 README 记录取证结论与恢复命令）。其中 15 个 `.ts/.tsx` 创建于
今日 19:22、**git 从无** ⇒ 可能是某扇窗的未提交工作，若那扇窗还在用请 `mv` 回去。
