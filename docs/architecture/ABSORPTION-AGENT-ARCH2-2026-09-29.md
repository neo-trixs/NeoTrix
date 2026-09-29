# 外部项目架构吸收（第二轮）— 2026-09-29

> 承接 `ABSORPTION-AGENT-ARCH-2026-09-28.md`（第一轮，8 源）。
> **本文件只记结论与裁决，不记调研过程。** 每条都标状态，且「已落地」必须有验证命令。
> 口径：`核心代码` = 已接线且在生产生效；`门` = 已接线但 advisory；`未落地` = 已取证待做。

## 0. 本轮的输入与去重

用户给出 46 个 URL。**第一轮已吸收 8 个**（logo-design-skill / awesome-autoresearch /
Understand-Anything / deepseek-harness / CLI-Anything / aliyun-handbook / claude plugin 规范 /
Horizon / ralph-playbook）。**本轮真新增 30 个**，已逐一 `webfetch` 取证。

**先证伪的四个前提**（不先证伪就会照着错的前提干活）：

| 提交的前提 | 实测结论 |
|---|---|
| `NVlabs/kda` 是 Kimi Delta Attention | **不是**。是 *Kernel Design Agents*（CUDA kernel 的文档/prompt 工作流）。**仓内无 `.rs`/`.cu`，无数学，无 recurrence，无 benchmark** |
| `supermemory` 的记忆引擎可吸收 | **引擎闭源**。MIT 仓只有 SDK/UI/框架 wrapper。事实抽取、时序矛盾消解、自动遗忘**不在仓内** ⇒ 只可吸收外围接线 |
| `antvis/Infographic` 有布局约束求解器 | **没有**。约束是「模板名 ↔ 数据形状」的**命名约定**，无校验、无报错 |
| `qc-skills` 是质量控制门 | **仓内没有 QC 门**。5 个 prompt-only 图像 skill。**最有价值的东西在 `skills/archive/qc-skill-tester/` —— 已归档因而对所有消费者不可见** |
| `dsh-market` 有自测量 | 有大量边缘状态机、**零自测量**（60+ 条 README 规则各自成状态机，无不变式页） |

**3 个仓无 LICENSE**（`jev-dsh-decision` / `dshfind` / `Hands-On-AI-Engineering`）⇒ 只取设计，
不取代码。`agent-scripts` 明确说 `INVOCATION_ID` 被所有子进程继承，所以"隐藏重启按钮"的
判定**必须同时**要求 systemd 标记 **且** 是 unit 主 PID —— 这条负向知识可直接抄。

## 一、本轮的核心发现（比任何单个机制都重要）

### 1.1 🔴 「结构严格」与「结果可测」是**两条独立轴**，而全行业几乎没人占第二条

我把 30 个新源逐一检查「有没有在测自己主张的东西」：

| 源 | 主张 | 仓内有测量吗 | 证据 |
|---|---|---|---|
| **Soup** | 4GB 卡跑 8B 微调 | **✅ 全场最佳** | 17 份 gate 记录，**连失败的 gate 都当一等行发布**；13 次逐字节相同配置跑出 2.43× 吞吐差（CV 35.3%）并分解为**可加**噪声；预提交决策规则 `4908d52b`；README 自承头条数字**已过期未重测** |
| **kev** | 小模型做结构化判断 | **✅ 参考实现** | `scripts/verify_claims.py` + `docs/claims.json`：**每个印在文档上的数字都能 CI 追到原始结果文件**，且按印刷精度比对。数据集降级记录（"不能当门用"）。诚实标注 development/test |
| **i-have-adhd** | 一套响应风格 skill | **✅** | 14 case × 3 trial = 84 行盲评 A/B + 加权 4.045→4.473；**并发布了一次失败的发布门**（"The gate fails on one rule"） |
| **rrsi** | 递归自我改进 | **✅ 有 OOD 协议** | 每个数字都对照**同一时间窗内未进化的 H₀**（控住 benchmark 漂移）；4 个域 9 行表；**诚实报出 in-dist +6.0 vs OOD +1.8 的 3 倍差距** |
| **hindsight** | 记忆系统 | **✅ 三层** | 外部 benchmark + 黑盒系统 eval（**且明确标注"不在 PR 上跑"**）+ 成本 eval。judge 与被测模型**必须分开配**，否则测的是自我认同 |
| **BugTraceAI** | 漏洞扫描 | **◐** | 32 个编号漏洞的目标仓 + 79.2% + 缺口表 + **数字上的过期横幅** |
| **cline** | 编码 agent | **⚠️ 框架完整但没在跑** | `evals/ARCHITECTURE.md` 自陈：smoke test **temporarily disabled**、CI workflow **removed**、`benchmarks/tool-precision/DEPRECATED.md`。**69.5k★ 仓库的核心测量被一次重构变成孤儿** |
| K-Dense scientific-agent-skills | 165 skill | **❌ 对有效性** | 结构/安全**极强**（`tests/_contract` AST 契约）。但 `AGENTS.md` 17.9KB 里 measure/evaluate/benchmark **零出现**。**只测「良构与安全」，完全不测「有用」** |
| browser-harness | "每个任务都变强" | **❌ 零** | 18.2k★。自工作区累积 helper 的核心卖点**从未测量**，连"累积了 N 个"都没有 |
| strands harness-sdk | "benchmarked defaults" | **❌ 零** | 8.5k★。无任何仓内 harness 测量 harness 的默认是否优于基线 |
| gryph / dsh-market / avibe / self_driving_data_analyst | 各主张 | **❌ 零** | 均为常规正确性测试 |
| anydoc / InsForge / OpenShell / PanelUI / awesome-design-md / Infographic | 各主张 | **部分/无** | anydoc 有 482 判定的盲评对位基准（**但语料不可分发 ⇒ 不可独立复现**） |
| weco / cue.im | 各主张 | **◐ / ❌** | weco「测量就是产品」（指标由你提供）；cue.im 无 |

> **⇒ 对 NeoTrix 的直接含义（这是本轮最贵的一条）**：
> `EVOLUTION-ROADMAP-2026-09-28 §0.1` 说「全行业没人证明过自进化有效」，本轮把它从
> 11 个仓扩到 **30 个仓**，并且找到了**反例的解法**：`kev/scripts/verify_claims.py` 与
> `Soup/benchmarks/gate-*.md`。
> **差异不是纪律，是「产出证据的机制」** —— 两者都是**把证据做成可执行物**（CI 脚本 /
> 门记录 + harness JSON 记 `git_sha`），而不是事后整理的报告。
> **NeoTrix 若把 4.x 阶段的自进化接上，必须同时接上这一层**，否则就是第 12 个
> cline 式孤儿。

### 1.2 ⛔ 反面教材 1：**重构会孤儿化测量基础设施，而没有任何东西会告诉你**

`cline` 的 eval 框架是本轮见过最完整的（3 层、`pass@k` **和** `pass^k`、`FLAKY` 作为
一等裁决态、flakiness 单独成指标、失败分类器带 issue 链接）。然后：

> `evals/ARCHITECTURE.md`: *"Smoke test CI is **temporarily disabled**.
> `.github/workflows/cline-evals-regression.yml` was **removed** until the build step
> is repointed at the new SDK CLI."*

⇒ **NeoTrix 的直接推论**：本仓 `scripts/check-test-baseline.sh` 的 baseline 文件
**实测是 0 字节**（`wc -l` = 0）。一份空 baseline 意味着 `--strict` 下**任何一条测试失败
都红**，而 CI `:65` 调的就是它。**这与 cline 的失效形态同源：门还在，内容没了。**

### 1.3 ⛔ 反面教材 2：**安全控制的枚举与词汇会各走各的**

`gryph` 同一份策略有**三套表示**：Go 的 `Decision` 枚举 **3 值**（Allow/Block/Guidance）、
`policy.schema.json` 的 `action` **6 值**（allow/warn/guidance/block/escalate/defer）、
文档 **6 值**。映射不明显。`allow`↔`Allow`、`warn`↔? 没有覆盖性断言。

⇒ **NeoTrix 直接推论**：`nt_policy.rs` 的 `evaluate_extra_deny` 目前只有 4 种
`deny tool:|cmd:|path:|actor:`。若要加条件（见 3.4 的 gryph CEL 洞见），**必须先有一个
「规则动作集合 == 代码枚举集合」的编译期或 CI 期断言**，否则就是第四套表示。

### 1.4 ⛔ 反面教材 3：**默认放行（fail-open）的安全层**

`gryph` 的 `Config.FailOpen` **默认为 true**：检查器出错 ⇒ 记一条 warning 然后放行。
对照 `strands` 的 `HumanInTheLoop`：分类器格式错 / 非布尔 / 抛异常 ⇒ **一律「需要批准」**。
`security-harness` 的 cache 同理：*"a stale entry in a security tool does not make the tool
slow, it makes it **wrong**"* ⇒ 失效键缺失时返回 `"no-skills-dir"` 让**所有查找 miss**，
"which is the right failure: **slow, not wrong**"。

⇒ **NeoTrix 直接推论**：`nt_policy.rs:75` `evaluate_policy` 现在是 **deny 全集 + default-deny**
（正确）。但 **`check-truth-surface.sh` 自己的 advisory 模式是 fail-open**
（"advisory, always exit 0"）—— 一个**门**的失效方向必须按它保护的东西来定：
防代码腐化的门可以 advisory，**防交付性失败的门不行**。

### 1.5 🟢 可直接抄的负向知识（这批仓的真正价值在这里）

| 源 | 机制 | 为什么可抄 |
|---|---|---|
| `kev` `scripts/verify_claims.py` | 印在文档上的数字 → 追到原始结果文件，按**印刷精度**比对。派生量：`macro_mean`（跨路径均值）、`over_requested`（把「已评测题」的准确率折算回「请求的全部题」，**被拒的算错**） | 把 `LESSONS-20260928-verification-must-be-executable` 从散文变成可执行物 |
| `K-Dense` `tests/_contract/structure.py` | `CHECKS: dict[str, Callable]` + `_meta` 驱动 `subTest(rule=…, skill=…)` ⇒ **加一条规则自动全仓生效**，无需逐 skill 改 | 「接线纪律」本身可审计化 —— 对应 NeoTrix 的「导出 ≠ 调用」 |
| `K-Dense` `structure.py` | **用 `ast` 解析，从不执行** ⇒ 165 个 skill 的契约能在一个解释器里安全跑完 | 门要能覆盖全仓，前提是不能跑被检代码 |
| `K-Dense` `_COMMAND_PATH` | 三条正则分三种失败形态，第三条匹配**可执行位置**（`python scripts/x.py` / `./scripts/run.sh`）⇒ 专治"跑命令得到 No such file" | fence 块内无反引号，前两条正则会漏 |
| `security-harness` `kb_version()` | 对**知识库本身**做 SHA-256 ⇒ KB 一变，全部缓存失效，**没有版本号要人记得递增** | 比"记得 bump 版本号"强一个量级 |
| `security-harness` `pr_impact` | `introduced`/`aggravated`/`pre_existing`，**后两者拿不准时必须选 `pre_existing`** —— *"Blocking a merge over code the author never wrote is how a required check gets deleted."* | 决定「该门拦谁」 |
| `security-harness` `warn` → 报 `success` | *"a warning that blocks a merge is a failure with extra steps"* | 门被删的机制性原因，比技术缺陷更致命 |
| `hindsight` `delta_ops.py` | 块按 **id 寻址、不按 index**（*"an index must be counted by the model, an off-by-one is still in range and silently overwrites"*）；未触及的段**物理复制**、永不重新生成 | 生成式改写必然漂移 |
| `hindsight` 策略作用域 | 首个匹配的策略**整包接管**，后来的**不填空** —— 因为 per-setting 混合导致「没人说得清这个 scope 在用哪条策略」 | 同 NeoTrix `.neotrix/layer-map.json` 显式真源的路子 |
| `hindsight` `reranking.py` | 乘法 boost `combined = CE × recency × temporal × proof`，**信号缺失时该因子恒为 1.0**；并显式识别「reranker 是恒等透传」并从 RRF rank 重新播种 | 缺失信号必须塌成中性，不能塌成 0 |
| `avibe` 三态工具策略 | `allow`(静默) / `allow+advice` / `deny`，中间态给"回合内合法但跨回合有损"的工具；每个 deny **指名可复制的替代命令** | 见 3.1 |
| `avibe` | `process_identity_recycled` —— **PID 会复用，持久化的 PID 不是安全句柄** | 见 3.5 |
| `rrsi` 退火 | 候选的编辑预算 `b_t = ceil(b_min + (b_max−b_min)·½(1+cos(πt/T)))` 随轮次退火到 1 ⇒ 每条历史记录最终都是**单个组件**的证据 | 没有退火 ⇒ 一个 5 编辑候选给一个 ΔS，**归因不可能** |
| `rrsi` `selection.py` | `S*` 是**单调 running max**且下限是对它测的 ⇒ 候选可以在低于当前最优时被接受；四种独立的拒绝理由各自产出人可读串（串里嵌数字） | 「别退化进噪声」与「追最大值」解耦 |
| `rrsi` `critic.py` | 6 类拒收，其中 **DEGENERATE 包含「无操作却声称有机制」与「删掉/禁用既有安全机制而无替代」**；**解析失败 3 次后 fail-closed** | 见 3.2 |
| `rrsi` `History.render` | 未被测量的门失败**只保留最近 4 条** —— *"a wall of aborts is a feedback loop, not evidence."* | 上下文工程的硬规则 |
| `rrsi` 域 guard | **否决式（veto），非加权式** —— 候选不能用别处的分数买下安全维度的回归 | 加权求和会让安全回归可购买 |
| `cline` 失败分类器 | 区分 `provider_bug` / `transient` / `harness` / `environment` / `policy` / `auth`，每条带 `issue:` 链接 ⇒ 频繁触发的模式拿到跟踪 issue 而不是进"预期失败"堆 | 把"Cline 编码差"与"docker 抖了"分开，否则分数不可解释 |
| `cline` 裁决三态 | `PASS` / **`FLAKY`** / `FAIL` —— flaky 是**独立信号**，不是要被平均掉的噪声 | |
| `hindsight` judge 设计 | judge 与被测**必须分开配模型**（同模型 = 测自我认同，fixtures 会告警）；temp=0 的"未达标"**在更高温度重问并以多数一致才成立** | |
| `hindsight` schema | `when/where/who/why` 若是必填字符串，模型在严格 schema 下**没有合法方式说"未提供"** ⇒ 只能编最近的合理值。改 nullable | **必填的结构字段会制造幻觉** |
| `hindsight` 语料不变式 | 每个 subject 内部一致（曾有一版同一 release 部署在三个不同日期）；waves 不拆 subject（一个 release 的所有事实同行，否则后一批读起来像"更正"） | *"learned by getting them wrong"* |
| `bugtrace` L0–L6 阶梯 | 便宜的证据先上，**停在第一个确认点**；反射但自证不了的打 `NEEDS_CDP_VALIDATION` 移交 | |
| `anydoc` 错误分类 | `Unsupported/NeedsOcr/Malformed` = "记下这个文件、取下一个"；其余 = 中止。`ResourceLimit`（解压/嵌套/节点数上限）= **一等错误，不是 panic** | 资源炸弹要有自己的错误分支 |
| `anydoc` IR | **IR 是「解析后」的** —— 样式级联、编号、交叉引用在构造 IR 之前解完 ⇒ IR 无歧义。且 `Math(String)` 存**不带定界符的 LaTeX**，目标格式不烙进 IR | |
| `cocoindex` 组件路径 | 增量身份是**声明的字符串**，不是内容哈希（改代码会破内容哈希，跨进程会破对象身份）；删除 = 路径缺失 | 约 50 行可移植到任何增量索引 |
| `cocoindex` 备忘键 | `hash(inputs) + hash(code)` —— 只键入输入的缓存**改 transform 后产出陈旧仍存** | **这是正确性 bug，不是优化** |
| `cocoindex` LMDB 三条 | ① 绕过 `run_txn` 丢 fsync 合并、并发提交**退化 10–100×**；② LMDB **无 savepoint**，"abort" 必须在函数体层处理；③ 写事务**必须同 OS 线程**开始并结束（写锁线程所属，释放失败**被静默忽略** ⇒ 后续所有写者永久死锁），而 `heed` 把 `RwTxn` 标 `Send`，**编译器抓不到** | 第四类死锁：`nt_lock_audit.py` 抓不到 |
| `cocoindex` API 面纪律 | *"Do not pre-expose tunables 'in case someone needs them'"* | 直接对冲「8 类重复类型 / knob 蔓延」 |
| `OpenShell` 验证器 | **5 态**（`within_boundary`=0 / `exceeds_boundary`=1 / `error`=2 / `unsupported`=3 / `inconclusive`=3），*"**Only `within_boundary` means that the check passed**"*；**`coverage.domains` 是一等输出行** | 二态 pass/fail 的部分覆盖验证器 = 安全 bug |
| `OpenShell` 差分提案风险 | 不做策略文本 diff，做**可达能力集 diff**：`l7_bypass_credentialed`（把 OpenShell 无法检视的流量发到有凭据的主机） | 看着更窄的规则可能更宽 |
| `OpenShell` 不对称能力 | agent **只能加**网络规则，不能删、不能改 filesystem/process；无法检视的协议 **API 根本没有字段，收到也忽略** | |
| `i-have-adhd` marker 对账 | **只有最新 marker 生效且 marker 带类型**；`session_compact` 后规则集**必须重新注入** —— *"**Reconciliation, not injection.**"* | 唯一一条"压缩后存活"的现成机制 |
| `i-have-adhd` 门可满足性 | 它的发布门**永远无法通过**（"no blocking findings"是绝对规则，而相邻规则是比较规则），**是跑出来才发现的，不是想出来的** | **⇒ NeoTrix 的每道门都该被问：是否存在任何能通过的实现？** |
| `steipete/agent-scripts` | 主张是**一致性**不是能力 ⇒ 不做 benchmark，改用**确定性 pre-commit 门**（`validate-skills` exit 1）。*"when the claim is an invariant, enforce the invariant instead of measuring a proxy."* | |
| `dsh-market` 重启按钮 | 当 DSH 进程**就是 systemd 的主进程时按钮自动隐藏**；判定要**同时**有 systemd 标记**且**是 unit 主 PID | `INVOCATION_ID` 被所有后代继承 |
| `awesome-dsh-plugin` 门死 | 门解析失败时**显式发一个红的 `checks` 结论「The gate could not run」** —— *"A gate that dies before posting is indistinguishable from one that never needed to run"* | 记录了 2026-08-18 的真实事故 |
| `K-Dense` 拒绝策略 | `AGENTS.md` 的 "Out of scope, and routinely declined"：不做通用 SWE skill、不做宽编排器、不给已能直连的服务加第二 provider | *"A skill library's biggest failure mode is selection competition, and it is addressed by **refusal policy**, not by quality."* |
| `PanelUI` `lifecycle-matrices.json` | 每个组件对**同样 9 个生命周期场景**作答，每条附 `evidence[{file, tests[]}]` 指针；允许显式 **N/A + 理由** | 让「声称 ≠ 证实」和「导出 ≠ 调用」一样可查 |
| `PanelUI` 主题 | `family`（品牌，配 light+dark）× `mode`（light/dark/system）**两轴正交**，`setFamily` 不改 mode、`toggleMode` 不改品牌 | 对应 `des` 的 Primitive→Semantic 分层 |

## 二、明确不吸收（会冲突 / 重复 / 前提已被证伪）

| 模式 | 来源 | 不吸收的理由 |
|---|---|---|
| `supermemory` 的记忆引擎机制 | supermemory | **闭源**，不在仓内。仓内只有 SDK/UI/wrapper |
| `NVlabs/kda` 的注意力机制 | NVlabs/kda | **前提证伪** —— 是 Kernel Design Agents，仓内无数学 |
| Infographic 的容错解析（`id@{...}` 静默忽略 / 重名 last-wins / 操作符过度归一） | antvis/Infographic | 对**流式 LLM 输出**极好，对**策略 DSL / 配置**是灾难。同一解析器，相反的要求 |
| 「用 `differential reach` 做策略变更风险」 | OpenShell | 需要形式化可达性分析（他们用 SMT）。NeoTrix 现无此基础 ⇒ **只取「5 态 + coverage 行」这条可移植的** |
| 训练专用 RL/TITO 配方 | fireworks `fw-ai/cookbook` | 需要真实 rollout 基础设施。**只取那条可判断的纪律**：先用你手上的信号选方法，别默认 SFT |
| 自建小模型（kev 4 个权重） | kev | 与 `LOCAL-LLAMA-2026-09-28` 路线重叠，**不重复投资**；只取 `verify_claims.py` |
| `PanelUI` 的 React Native 组件 | PanelUI | 与 Tauri 桌面端不是同一技术栈。**只取 `lifecycle-matrices.json` 那个「每行为一指针」的方法** |
| `awesome-design-md` 的**token 数值** | VoltAgent | LICENSE 明说是提取的公开 CSS 值，但含专有字体名与品牌识别 ⇒ **读结构，不 vendor 数值** |
| 「按生态年龄插值权重」的排名算法 | dshfind `scoring.mjs` | 精巧但**历史分不可重放**（未归档输入快照），且 AI 评的 25% 在同版本号下会静默换判 |
| `bugtrace` Swarm Graph 可视化 | BugTraceAI | 自陈是 *"a representation, not an event-by-event source of truth"*，且 ladder 的 "confirmed at" 是 **indicative rather than exact**。高成本、低真值 |
| 自建 prompt 目录式 skill 框架 | qc-skills | NeoTrix 已有 `skill_loader` + `index.json` + 门。**只取它归档的 `qc-skill-tester` 五阶段协议**（Stage 0 provider readiness 分离是本轮最好的评测纪律之一） |

## 三、待落地（按价值排序，均已对 NeoTrix 现状取证）

落点见 `EVOLUTION-ROADMAP-CODE-NODES-2026-09-29.md` §节点表。此处只给「为什么现在做」。

| 优先 | 模式 | 来源 | 对 NeoTrix 的实测缺口 |
|---|---|---|---|
| **P0** | **门的存在性检查**（门解析失败必须显式红） | awesome-dsh-plugin | `.github/workflows/ci.yml` **4 个 job 指向 gitignored 目录**（`neocodex-frontend/` 在 `.gitignore:136`，`e2e/` 0 跟踪文件）。任何检出上这 4 个 job 必挂 |
| **P0** | **未被编译代码的检测**（class 2 扩到全部 module 目录） | K-Dense `CHECKS` 契约 | `check-truth-surface.sh:69` 把 UNDECLARED 限定在**字面叫 `tests` 的目录** ⇒ 实测 **212 个 .rs 从不被任何 target 编译**，含 `nt_act/tool_registry.rs`(770行)、`l6_meta/nt_meta/eval_engine/`(4文件，唯一的 dataset/experiment/llm-judge 抽象)、`nt_crystal_core/observability.rs` |
| **P0** | **claims → 证据的可执行追溯** | kev `verify_claims.py` | 有 `nt_manifest.py`（查 `file:line` 有效性）但**不查数字**。且本轮发现 3 处文档数字已错（frontmatter 债 32/59 非 41/58；SKILL.md 68 个非 67；`registry.rs:466` 非 `:459`） |
| **P1** | **三态工具策略**（allow / allow+advice / deny） | avibe `agent_tool_policy.rs` | `nt_policy.rs:75` 只有 Allow/Deny 二态；`looks_like_escape` 是 16 词 needle 表 —— **命中即 deny，不给出路** |
| **P1** | **StopReason 与 TurnStatus 正交** | strands `event_loop.py` | `TurnStatus`（Done/Continue/NeedsClarification/Blocked/Waiting）**混了「模型选择停止」与「预算停止」与「策略停止」**。`nt_agent.rs:698` 跑满 → `Waiting`：**预算耗尽与「人可接手」是同一态** |
| **P1** | **门可满足性检查** | i-have-adhd | `check-test-baseline.sh` 的 baseline **0 字节** ⇒ `--strict` 下任何失败都红，CI `:65` 正在调它。**没有一道门被问过"是否存在能通过的实现"** |
| **P1** | **门记录的 `git_sha`/env 指纹** | Soup `harness/*.py` | `nt_manifest.py:58` **已有** `env_fingerprint()`（head+dirty+cargo_lock 的 sha256 前 16）⇒ **地基已在，只需扩到门记录** |
| **P2** | **delta-op 式记忆更新**（块按 id 寻址，未触及段物理复制） | hindsight `delta_ops.py` | `nt_temporal_facts.rs` 是**整节点版本链**（每版本独立 id）⇒ 形态已对，但**没有「未触及内容不重新生成」这条保证** |
| **P2** | **多因子检索打分：信号缺失塌成 1.0 而非 0** | hindsight `reranking.py` | `bm25.rs` 只有 `rrf_fuse`；`kb_search.rs:23` 的 LRU key 是 `format!("search:{}:{}", query, limit)`、失效是**全量 `clear()`**，**无内容哈希** |
| **P2** | **内容寻址的缓存键（含路径，不只含字节）** | K-Dense `content_hash()` | 同上。重命名/删除必须改哈希，否则陈旧条目活下来 |
| **P2** | **不可编译代码当正确性 bug（`hash(inputs)+hash(code)`）** | cocoindex `@coco.fn` | `SemanticCache`（`nt_core_cache.rs`）是 exact/semantic/response 三级，**无代码哈希** ⇒ 改 prompt 不失效 |
| **P3** | **K-Dense 拒绝策略**（Out of scope 清单） | K-Dense `AGENTS.md` | `skills/index.json` 60 条无「我们不做什么」一节 |
| **P3** | **每行为一指针的生命周期契约** | PanelUI | 无 |

## 四、本轮被证伪的 NeoTrix 既有结论（留档防重犯）

| 文档原文 | 实测 |
|---|---|
| `ABSORPTION-AGENT-ARCH-2026-09-28.md:13` 「frontmatter 欠账 41/58」 | **32/59**（`check-skill-gate.sh` 实跑：`32/59 SKILL.md without frontmatter`）。59 = `categories.*.skills` 之和 = loader 可见的那部分 |
| `check-skill-gate.sh:15-19` 头注释「看不见 67 个真实 SKILL.md」「只有 24/67」 | 磁盘 **68** 个 / **34** 个有 frontmatter。另：`skill_index` 60 条、`categories.*.skills` 59 条，差 1 是顶层 `skills/SKILL.md` |
| `check-skill-gate.sh:11` 「58 条存量」 | index 合计 **59** |
| `TODO.md:526` 「`maturity_audit()` 接 CI 门 ⬜」，引 `registry.rs:459` | **已接且阻塞态**（`ci.yml` `capability-truth`），真实行号 **`registry.rs:466`** |
| 「`SkillInvocationPolicy` 是核心代码 4 测试」 | 字段在，但 `visible_to_model()/visible_to_user()/is_trusted_only()` **零消费者** ⇒ 是**门不是核心代码** |
| 「`nt_crystal_serve` 用真典 `SkillRegistry`」 | 唯一调用方 `nt_crystal_serve.rs:1303`；其 `with_defaults()` 指向 `$CWD/.neotrix/skills` + `~/.neotrix/skills`，**两处都不含 `index.json`** ⇒ 「真典」实际扫不到东西 |
| 「`.neotrix/context-manifest.json` 是上下文清单」 | **它不是** —— 它是 5 条 claims/evidence 记录。仓内**无任何上下文装配清单**（`.neotrix` 下同名不同物） |
| 「4 份 `CapabilityRegistry`」 | 仍 4（未变） |
| 「有 `evals/` 目录」 | **全仓无 `evals/` 目录** |
| 「`scripts/check-test-baseline.sh` 账本在跑」 | baseline 文件 **0 字节**；CI `:65` 调它 ⇒ 空账本 = 「任何失败即红」 |

## 五、许可证台账（决定能抄什么）

| 允许抄代码 | 仅可抄设计 |
|---|---|
| MIT：`hindsight` `browser-harness` `anydoc` `PanelUI` `Infographic` `dsh-market` `qc-skills` `K-Dense` `i-have-adhd` `agent-scripts` `avibe`<br>Apache-2.0：`harness-sdk` `InsForge` `cocoindex` `OpenShell` `rrsi` `cline` `kev` `Soup` `BugTraceAI` `fw-ai/cookbook`<br>CC0：`awesome-dsh-plugin`<br>CC-BY-4.0（文档/prompt 段）+ Apache-2.0（源码段）：`NVlabs/kda` | **无 LICENSE**：`jev-dsh-decision` `dshfind` `Hands-On-AI-Engineering`<br>闭源不可核：`supermemory`（引擎）`cue.im` `weco` `glean/waldo` `primeintellect/ramp` |
