# handoff — 2026-10-06 工具输出 errors-first 蒸馏（AgentLoop 最后一项能力）

承接 `handoff-2026-10-06-agent-loop-wiring.md`。**§2 是本窗口教训，§5 是遗留。**

## 1. 本窗口做了什么

| commit | 内容 |
|---|---|
| `8196dd10` | `truncate_output` 从砍尾留头改为 errors-first 可逆蒸馏 |

用户裁决「接线」后的最后一项：`AgentLoop` 侧独有的 `distill_output`（吸收 repowise
「压缩再给模型读」）与 `maybe_compact_context`。本窗口接了前者。

**`maybe_compact_context` 未接**，理由见 §5。

## 2. 本窗口最该被继承的经验

### 2.1 ⭐⭐⭐ **收窄必须发生在最靠里的那一层**

本窗口最贵的一条。

我先把 `distill_output` 接在**外层**（写进 transcript 前），写完测试是红的。
查下去才发现：`truncate_output` 早已在**内层**砍过一刀 ⇒ **外层拿到的是残缺
文本**，错误行在内层就已经没了 ⇒ **外层再怎么蒸馏也救不回来**。

⇒ 蒸馏已**下沉到 `truncate_output`**（工具输出的统一收窄点，5 个 executor 共用）。
   外层接线随后被变异验证为**冗余**并移除（撤掉外层 ⇒ 测试仍绿）。

⇒ **判据**：更精细的算法被更粗糙的前置截断架空，就是**永远走不到的死代码**。
   接「更好的算法」之前先问：**同一份数据上，前面还有没有别的收窄？**

### 2.2 ⛔ 搬移会带进真实缺陷，且可能直接挂死进程

core 侧 `nt_loop_step.rs:156-161` 的兜底裁剪有死循环：

```
while 超预算 && clipped.len() > 8 {
    keep = char_indices().nth(len * 0.7);   // keep ≈ 0.7 * len
    truncated.push_str("\n…[truncated]…");  // +11 字符
}
```

`len` 落在 9..~15 时 `cut = floor(0.7*len)`，但 push 固定 11 字符 ⇒
**长度可能不降反升** ⇒ 永不退出。

**实测**：这条代码让测试进程**挂死**（我不得不 kill）。core 侧那段逐字相同
⇒ **同一缺陷仍在 core 侧**，属他窗文件，未擅改。

⇒ 修法不能是「把 0.7 改成 0.5」—— 任何固定比例都可能不降。改为
  **每轮强制缩短 + 预算上界判断**，并加 `SHORT_FALLBACK`（预算小到连省略
  标记都装不下时用单字符 `…`，否则长标记本身 ≈7 token 就已超预算，违反
  「任何输入都不超预算」这条硬不变量）。

### 2.3 ⛔ 测试判据必须区分「被处理」与「超预算」两件事

接线守门测试改了**三次**才拿到判别力：

① 查 `steps` 表 ⇒ **错的面**。蒸馏结果只进 transcript，而 transcript 是
   `run_loop` 的局部变量（**不落库**）⇒ 改用引擎收到的 history 快照。
② 夹具 200 行 ≈ 8206 字节 ≈ **2051 token**，落在 3000 预算内 ⇒
   `distill_output` **原样透传**（这是正确行为！）⇒ 测试红 ⇒
   **差点把正确实现当缺陷去「修」**。
③ 夹具加到 600 行 + 加前置断言「Tool 行确实被收缩」。

⇒ 「期望被处理」的断言必须先确认输入**真的越过阈值**，否则测的是「没触发」。

### 2.4 ⛔ heredoc 里写 `\n` 会破坏 Rust 字符串字面量

本轮一次 Python heredoc 把 `\n` 写成了真实换行，把 `format!` 里的字符串
拆成多行语法错误。同一轮里还漏写了 `PYEOF` 结束符，让 shell 把后半段命令
当成 Python 源码。

⇒ 改 Rust 文件时优先用 `edit` 工具；必须用脚本时，检查 diff 里的字符串
   字面量有没有被换行劈开。

### 2.5 ⛔ `cargo fmt -p <crate>` 会格式化**整个 crate**，不只你改的文件

我为格式化自己的新文件跑了 `cargo fmt -p neotrix-neobot` ⇒ **59 个文件被
重排**（他窗的 WIP 全部被 fmt 污染）。

幸而 `git commit --only` 保护了提交内容（只有 3 个文件入库），事后逐文件
diff 确认那 56 个是**纯 fmt**（数组元素重排、无逻辑变更）才 `git checkout`
还原。

⇒ **在共享树上不要跑 `cargo fmt -p <crate>`**。要格式化单个文件用
   `rustfmt <file>`，或直接写对格式。

## 3. 修掉的缺陷（不是「更精细」，是「假信息」）

`truncate_output` 原先砍尾留头，两个**具体**损失：

① **exit code 与尾部摘要被丢掉** —— 而那是命令成败的最终结论；
② 8 KiB 之后的 `error:` / `FAIL:` 行一起被砍 ⇒ 模型看到一段正常的前缀，
   **得到「命令成功」的错觉**。

第 ② 条是最坏的一类错误：不是信息缺失，是**假信息**。而 neobot 的产出直接
面向用户（IM 通道）与下游模型。

## 4. 验证口径（干净检出 `/tmp/nt-w`，HEAD 含他窗 `96056d65`）

- `neotrix-neobot`：**558 绿**（上一窗口 552 → 新增 5 条 distill 单测 + 1 条接线守门）
- `neotrix --lib`：13358 绿 / **1 红**
- `check-layer-deps.sh --strict`：`PASS: 0 new violation(s); 13 known/recorded`
- `nt_lock_audit.py crates/neotrix-neobot/src`：**0 处**

### 已验证的两个变异

| 变异 | 结果 |
|---|---|
| `truncate_output` 改回砍尾留头 | 接线测试**红**（报 exit code 丢失） |
| 撤掉外层蒸馏接线 | 测试**仍绿** ⇒ 证明内层已足够（外层冗余，已移除） |

### ⚠️ 唯一红项**非本窗口引入**

`l0_substrate::nt_core_platform::mod_orphan::tests::scan_tree_sorted_by_lines_desc`
—— 他窗 `64913227` 改 `scan_tree` 时引入（测试造的临时树 `mod.rs` 为空，
期望 2 个孤儿实得 3）。已在不含本窗口改动的检出上复现 3/3。
上一窗口的另一个红项（`nt_permission_profiles`）本轮已不复现。

## 5. 剩余任务

| # | 任务 | 状态 |
|---|---|---|
| 1 | **core 侧 `distill_output` 死循环未修** | ⚠️ 该文件属他窗。**这段代码仍在 core 里**，若有人接线 `AgentLoop` 会挂死进程 |
| 2 | `maybe_compact_context` 未接 | 需新增一次 LLM 调用改变成本模型；`nt_agent` 侧有 `nt_cost.rs` 账本但该调用未纳入预算。需先决定是否值得 |
| 3 | 5 个 trade 能力调用数 0 → 正 | **仍阻塞于更根本处**：`TradeCapabilityRegistry` 本身零生产消费 |
| 4 | 金丝雀窗口按 `convo_id` 键化 | 未做（详见 `handoff-2026-10-05-…canary-rewiring.md` §2.3 的三条失败路线） |
| 5 | 能力市场用户可见入口（API/UI） | 未做 |
| 6 | KB namespace/sensitivity → ring 结果层门 | 未做 |
| 7 | UI 门已知失败 + 完整 CI | 未做 |

### 关于 #1 的处置建议

那段死循环现在**只存在于 core 的 `AgentLoop` 里，而 `AgentLoop` 零生产
实例化** ⇒ 当前不产生运行时危害。但：
- 它是一个**已确证的挂死缺陷**，任何人接线 `AgentLoop` 都会踩到；
- 它与 neobot 侧已修的实现**逐字不同**，两处会漂移。

⇒ 建议：由 core 的属主按 neobot 侧的修法同步一次；或直接把 core 侧的
   `distill_output` 标记为「已废弃，见 `neotrix-neobot/src/nt_output_distill.rs`」。

## 6. 收工自查

| 项 | 状态 |
|---|---|
| 改动入库 | `8196dd10`（3 个文件） |
| 主树残留 | `crates/neotrix-neobot/` 完全 clean |
| fmt 污染 | 59 个文件被 `cargo fmt -p` 波及 ⇒ 逐文件确认为纯 fmt 后已还原 56 个（我的 3 个已入库） |
| worktree | `/tmp/nt-w` 已回收 |
| 他窗 WIP | 未碰未提交（`.neotrix/capability_*.json`、`scripts/ops/nt_security_wiring.py`、`.neotrix/patches/*.patch` 等） |
| 锁审计 | 0 处 |
| 调试残留 | 已清（2 处 `eprintln!` + 2 个临时 probe 模块） |

### 未提交改动去哪了

- 主树我本轮的改动：**全部已入库**。
- 他窗 WIP：**未碰未提交**，归他窗所有。