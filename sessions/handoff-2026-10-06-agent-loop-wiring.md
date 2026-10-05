# handoff — 2026-10-06 把 AgentLoop 的独有能力接到真实执行环

> 承接自 `handoff-2026-10-05-decor-noise-and-canary-rewiring.md`。**§2 的四条教训
> 是本窗口新添的，§3 是本窗口的成果。**

## 1. 本窗口做了什么

裁决是「接线」而非「删」（用户决定）。实际接了**三项**，另加**一项修 bug**：

| commit | 内容 |
|---|---|
| `4603a95e` | 修 `11280660` 的漏项：删掉悬空的 `mod nt_loop_canary_tests;` |
| `837fefd2` | 把 G27 输出治理器（10 规则）搬进 neobot 并接到真实执行环 |
| `fc731b29` | 接上心智病毒传播防线（arXiv 2608.10218） |
| `907b613e` | 工具输出的凭据检测（补一个真实泄露面） |

**为什么是这三项**：子代理独立复核确认 `AgentLoop` 零生产实例化、
`HiveAgentLoop` 的 `HiveRouter` 在生产侧也零消费者 ⇒ 整条 hive 链也是死的。
于是逐项查 A 独有能力 vs B（`nt_agent.rs`）的缺口，选出**低成本、高价值、
无副作用**的三项。

## 2. 本窗口最该被继承的经验

### 2.1 ⛔ 编译通过 ≠ 接线存在（本轮第 N 次，仍在犯）

`837fefd2` 的接线守门测试我写了**两版**：

① 第一版：`LoopForever` 用 bash `cat creds.env` 触发 ⇒ 测试红，报
   `audit tools: ["bash"]`。
② 定位后发现 **`Bash` 在 `nt_policy.rs:172-173` 是恒定 deny**
   （`default-deny: no explicit allow rule matched`）⇒ 每跳都 `(denied)`
   ⇒ **扫描那段代码根本没被执行到**，而测试失败原因我一开始猜的是
   「bash 被 jail 拦了」。
③ 第三版改用 `ReadFile`（policy 的 allow 分支）+ 专用 `ReadCredsEngine` ⇒ 绿。

⇒ **判据**：接线测试必须证明「被测代码真的执行过」。最快的信号是
   **审计/日志里出现了预期的多条记录，而不只是「调用返回 Ok」**。

### 2.2 ⛔ 扫不通就打印现场，不要猜

`907b613e` 调试时连续猜错三次（cwd 错 / jail 错 / PATH_NEEDLES 错），
最后靠一行 `eprintln!` 打出 `ok=false output="(denied)"` 一次定位。
⇒ **`eprintln!` 调试比推理快一个量级**。事后记得删（本轮删了 2 处）。

### 2.3 ⛔ 测试样本必须取自被测系统的真实判据

`治理违规会把报告标红` 的样本连改两次都不合格：

| 版本 | 样本 | 结果 |
|---|---|---|
| ① | `!violations \|\| !smells` | 零证明力（`smells` 走 `AiSmellDetector`，**绕开** `rule_results`，「治理器恒满分」的变异能漏过） |
| ② | 「综上所述 / 总体来看」各 1 次 | R02 放过（**不在 14 词表里**），实际命中 smells ⇒ 仍漏过 |
| ③ | 同上词 × 3 | 阈值到了但词仍不在表里 ⇒ **仍漏过** |
| ④ | 取自 `HEDGES`（可能/或许/大概）× 3 | 命中 `total >= 3` ⇒ 绿 |

⇒ **写「期望被拒」的样本前，先读判据源码拿到真实词表/阈值**。
⇒ 同理，R07 那条我第一版拿「文件 src/main.rs 共 42 行」当干净文本，
   结果 R07 真去读文件系统 ⇒ 判违规 ⇒ 报错反而暴露了 R07 在正常工作。

### 2.4 ⛔ 死参数是搬移的障碍，先查清再搬

治理器的 `check_fn` / `govern` 签名带 `style: OutputStyleId`，看起来会
把治理器绑死在样式模块上（而样式里 `RundownStyle` 是字面占位符）。
**实测 10 条规则的签名全是 `fn rN_xxx(text: &str, ...)`，无一读它，
`finalize` 也不读** ⇒ 死参数。删掉之后治理器才真正独立于样式模块。

⇒ 搬移被「依赖」卡住时，先查这个依赖是不是**真被读**。

### 2.5 ⛔ 别把窄防护说成宽防护

`nt_prompt_guard` 只防 **agent-to-agent 传播性想法**，**不防 prompt
injection**。真正的注入防护在 `nt_policy.rs` 的 `is_jailbreak_path` /
`looks_like_escape`。代码注释里明确写了「不要把这条当『有了注入防护』讲」。
同理 `nt_secret_scan` **不搬 PII 那 5 条**（误报率高，正常开发输出全是
邮箱/IP）—— **误报率高的检测器会被用户关掉，比没有更坏**。

### 2.6 ⛔ 死代码的「有测试保护」是假象

`nt_io_agent_loop` 曾是「20 条测试全绿 + 零生产实例化」。
`multimodal_transform` 1733 行、40 条测试，但唯一的生产实现是
`PlaceholderAnalyzer`（`format!("[{} #{image_id}: {}]", ...)` 纯字符串拼接）
⇒ 接上等于把图片 alt 文本冒充视觉分析结果。

⇒ **评估能力要看「唯一实现有没有真逻辑」，不看行数与测试数**。

## 3. 接线成果（三项，各带变异验证）

| 能力 | 落点 | 文件 | 变异验证 |
|---|---|---|---|
| G27 输出治理 | `nt_agent.rs` 的「模型给最终答案」唯一收敛点 | `nt_governance.rs`（805 行新文件） | ① 删调用 ⇒ 接线测试红 ② `govern` 恒满分 ⇒ 规则测试红（**收紧断言后才抓到**） |
| 传播防线 | `nt_http_engine::chat_body`（唯一构造 system prompt 处） | `nt_prompt_guard.rs`（3 条单测） | 幂等/空 prompt/原文保留三条 |
| 凭据检测 | `nt_agent.rs` 的 `add_step` 之前 | `nt_secret_scan.rs`（16 条正则 + 4 条单测） | 关掉审计写入 ⇒ 接线测试红 |

三处**纯观测**：不改用户看到的文本、不阻断对话、不进 transcript。
理由写进各自文件头。

## 4. 剩余任务

| # | 任务 | 状态 |
|---|---|---|
| 1 | 5 个 trade 能力调用数 0 → 正 | **阻塞于更根本处**：`TradeCapabilityRegistry` 本身零生产消费，需先决定这些能力该被谁调用 |
| 2 | 金丝雀窗口按 `convo_id` 键化 | 未做。解锁并发测试（详见上一份 handoff §2.3 的三条失败路线） |
| 3 | 能力市场用户可见入口（API/UI） | 未做 |
| 4 | KB namespace/sensitivity → ring 结果层门 | 未做 |
| 5 | UI 门已知失败 + 完整 CI | 未做 |
| 6 | `AgentLoop` / `HiveAgentLoop` 本体处置 | 用户已裁决「接线」，本体保留。但**仍零生产实例化** —— 三项能力搬走后它更空了，见 §5 |

## 5. ⚠️ 遗留判断（建议下窗口处理）

`AgentLoop` 的独有能力被搬走后，它剩下的主要是 `distill_output`（139 行
errors-first 可逆蒸馏）与 `maybe_compact_context`（75 行双相压缩，调 LLM
做摘要）—— 这两项 `nt_agent.rs` **没有等价物**（B 的
`enforce_transcript_budget` 只按条数+字节砍，不调 LLM）。

⇒ **要接就接这两样**，否则 `AgentLoop` 会变成一个「只剩没人用的算法」的空壳，
   而空壳比删掉更容易误导后人（正是本轮治的那个病）。

## 6. 验证口径（干净检出实测）

- `neotrix-neobot`：**552 绿**（基线 531 → 新增 21）
- `neotrix --lib`：13356 绿 / **2 红**
- `check-layer-deps.sh --strict`：`PASS: 0 new violation(s); 13 known/recorded`（rc=0）
- `nt_gate_coverage.py`：PASS，43 道门
- `nt_lock_audit.py`：`neotrix-core/src` 与 `crates/neotrix-neobot/src` 均 **0 处**

### ⚠️ 两个红项均**非本窗口引入**（逐个确证，非推断）

| 红项 | 归属 | 证据 |
|---|---|---|
| `mod_orphan::tests::scan_tree_sorted_by_lines_desc` | 他窗 `64913227` 改 `scan_tree` 时引入 | 在不含本窗口改动的 `a10342a2` 检出上复现 3/3；测试造的临时树 `mod.rs` 为空，期望 2 个孤儿实得 3 |
| `nt_permission_profiles::tests::test_global_state_integration` | 他窗 `64401063` | 该文件不在本窗口 4 次提交的任何 `.rs` 列表内 |

⇒ 两个红项的文件都在 `neotrix-core/`，而本窗口在 core 侧只改了
   `nt_io_agent_loop/mod.rs`（删一行悬空声明）。

## 7. 收工自查

| 项 | 状态 |
|---|---|
| 改动入库 | 全部已提交（`4603a95e` / `837fefd2` / `fc731b29` / `907b613e`） |
| 主树残留 | 我本轮的 7 个文件均 clean |
| worktree | 我开的 `/tmp/nt-v`、`/tmp/nt-head` 均已回收 |
| 他窗 worktree | `merge-b` / `nt-v2` / `nt-v4` / `nt-v7` **未动** |
| 锁审计 | 两 crate 均 0 处，改过 `.rs` 已重跑 |
| 调试残留 | 已删（2 处 `eprintln!`） |

### 未提交改动去哪了

- 主树我本轮的改动：**全部已入库**。
- 他窗 WIP（`.neotrix/capability_*.json`、`nt_astar.rs`、`headless.rs`、
  `llm_judge.rs`、`sandbox.rs`、`nt_approval.rs` 等）：**未碰未提交**，归他窗。