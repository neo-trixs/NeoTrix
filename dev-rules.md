# Dev Rules (R-P1 – R-P80) — 惰性加载全量规则

> 本文件由 AGENTS.md 通过 `@dev-rules.md` 声明。**处理编码/审查任务时加载**，其余场景不预加载。加载后内容为强制规则。

## 编码基线 (R-P1 – R-P8)

- **R-P1**: `#![forbid(unsafe_code)]` — zero unsafe in core (lib.rs:18)
- **R-P2**: `#![deny(warnings)]` — temporarily commented out during Cataclysm, re-enable after cleanup
- **R-P6**: Float clamping: `.max(0.0).min(1.0)` not `.clamp()` (works on all Rust editions)
- **R-P7**: `VecDeque::windows()` doesn't exist — collect to `Vec` then slice
- **R-P8**: `make_stage!` macro for SEAL pipeline stage definitions

> R-P3 (`?` 优于 unwrap/expect)、R-P4 (import std→external→crate 排序)、R-P5 (内联测试) 为模型默认行为，删除不产生错误。

## 构建验证 (R-P9, R-P13, R-P17, R-P29, R-P35, R-P51, R-P54)

- **R-P9**: Never trust cached builds — `cargo clean` before definitive error count after structural changes
- **R-P13**: After activating ghost modules, always run clean build (not incremental) to catch cascade errors
- **R-P17**: Use `cargo check` with `--color=never` redirect to file + `grep -c "error"` for reliable error counting — cached incremental builds can show stale errors.
- **R-P29**: 结构变更后（模块添加/删除、路径变更），必须执行与之前跨越不同会话的完整构建才能得到可靠错误计数。增量构建可能缓存过时的诊断结果。
- **R-P35**: 任何结构变更（模块添加/删除、路径变更、mod.rs 编辑）后，先检查 `cargo check --lib -p neotrix` 是否在 <2s 内完成。若是 → 强制 `cargo clean`，重跑获取真实错误计数。
- **R-P51**: 结构变更(文件添加/删除/编辑mod.rs)后, cargo check可能显示0 error但实际有预存错误。必须连续build两次, 或强制cargo clean后build获得真实计数。
- **R-P54**: 任何对生产文件的编辑后, 必须运行`cargo check --lib -p neotrix`验证。如果之前是0 errors但出现新错误, 先确认是否缓存级联(连续build两次)而非假设新引入。

## 工具接地与持久化 (R-P10, R-P15, R-P16, R-P49, R-P52, R-P53)

- **R-P10**: Audit agents hallucinate ~6% of findings — independently verify all specific line allegations
- **R-P15**: Always verify task agent output before proceeding — empty agent results should trigger manual intervention
- **R-P16**: After every edit or batch write, call `verify_persistence()` or re-read the file to confirm the write actually persisted — do not trust tool success messages.
- **R-P49**: 每次文件写入后必须验证: `verify_persistence(file, pattern)` 或 `wc -l` + `grep pattern file`。Python open().write() 需要 f.flush() + os.fsync(f.fileno())。
- **R-P52**: 验证文件内容与路径一致: `verify_content_path_consistency()`检查每个已声明模块的文件内容是否包含匹配的模块声明。
- **R-P53**: `git reset --hard`触发零个git hooks, 也不能被Claude Code PreToolUse拦截。唯一可靠防护: shell函数包装git命令(在~/.zshrc中定义)。

## 架构与接线 (R-P11, R-P12, R-P14, R-P21, R-P22, R-P50)

- **R-P11**: Config struct with `Default` impl is the canonical pattern for hardcoded thresholds — use for all future refactors
- **R-P12**: L1→L8 layer violations are best fixed with `BrainProvider` trait in core/ + Box<dyn BrainProvider> in L1
- **R-P14**: Pipeline disconnect fixes must be verified at compile-time AND runtime — compile check alone is insufficient
- **R-P21**: Pipeline 连接必须双重验证: (1)编译期函数调用链 (2)运行时实际数据写入
- **R-P22**: Bridge 层必须保持与 L3 层的同步：添加字段/变体时必须同时更新 from/to 转换函数
- **R-P50**: 守护脚本必须使用原子操作(mkdir)而非检查-创建(access+mkdir)模式。lockfile需要过期间隔(建议10s)防止僵尸锁。

## 检测系统自审计 (R-P23, R-P30, R-P32, R-P33, R-P36, R-P37, R-P38, R-P39, R-P40)

- **R-P23**: 检测系统必须能检测自身缺陷（Self-Audit of Audit）。任何检测模块只要有硬编码阈值、不匹配模式、或遗漏变体，检测系统本身必须能发现。
- **R-P30**: 检测系统有效性的唯一真正度量是"检测结果是否能影响行为"。SelfTest 仅验证检测函数不崩溃，不验证其输出是否消费。必须在审计中主动追踪: "此检测函数的结果是否被任何生产代码读取？"
- **R-P32 (Dual Observation Independence)**: 监控器必须独立于被监控对象。被监控对象不应能控制自身监控的显示或抑制。检查 `can_emit`/`suppress_monitoring`/`should_report` 类方法。
- **R-P33 (Self-Healing Loop Completeness)**: 检测模块必须追踪其输出是否被任何非测试代码消费。SelfTest 仅验证检测不崩溃，不验证消费。审计必须标记 "自检孤岛"。
- **R-P36 (Behavioral Grounding)**: 检测模块只有在输出能影响行为时才算"接入生产"。日志输出、trace!、或写入未读文件均不算生产接线。唯一有效接线: (1) EventBus 触发行为改变 (2) brain 状态修改 (3) KB 持久化后被其他 handler 消费。
- **R-P37 (Self-Healing Maturity Audit)**: 每个检测模块必须按三级成熟度分类: Observer(Level 1) / Gatekeeper(Level 2) / Healer(Level 3)。审查必须追踪从 Level 1→2→3 的迁移进度。
- **R-P38 (Retry Cap Mandatory)**: 所有 handler loop 必须定义 max_iterations 或 retry_cap。无上限的 `loop {}` 自动标记为 P0 缺陷。事件: Claude Code 的 1279 次连续 compaction failure 案例。
- **R-P39 (SelfTest Ratio Monotonic)**: 跨 session 的 SelfTest 覆盖率必须单调递增。任何 session 导致覆盖率下降必须自动标记回归。当前跟踪基线: 61% (27/44)。
- **R-P40 (Persistent Instance Pattern)**: 模块实例必须存储在 BackgroundLoopHandle 的持久字段中。handler 函数内 `TypeName::new()` 标记为 R-P31 反模式，除非字段不持有状态。

## 审查方法论 (R-P18, R-P19, R-P20, R-P26, R-P27, R-P28, R-P31)

- **R-P18**: Scanner patterns must include `pub(crate) mod` and `mod` in addition to `pub mod` when walking module declarations.
- **R-P19**: 任何架构修复声明必须通过 `cargo check` 验证，不能仅凭文本分析断言修复成功
- **R-P20**: Schema drift 检测必须同时检查: (1)字段存在性 (2)字段类型 (3)序列化/反序列化一致性 (4)DB存储格式
- **R-P26 (SelfTest 三维度验证)**: 审计 `SelfTest` 覆盖率时区分三层次:
  1. *Impl 存在性*: `impl SelfTest for TypeName` 是否在文件中
  2. *注册完整性*: 是否在 `SelfTestRegistry` 中注册（两个位置：run.rs + pipeline.rs）
  3. *生产接线*: 模块的实际检测函数（`evaluate()`/`check()`/`audit()`）是否在非测试代码中被调用
- **R-P27 (可见性链双重验证)**: 审计模块注册时检查两条路径:
  1. `layer_impl/mod.rs`: `pub mod nt_xx;` 存在
  2. `neotrix/mod.rs`: `pub use layer_impl::nt_xx;` 存在（否则模块在 `crate::neotrix::nt_xx::` 下不可见）
- **R-P28 (分类清单比计数更重要)**: 审计发现应输出完整表格而非仅计数。计数可能被缓存构建掩盖。格式: `模块 | 文件 | SelfTest? | 注册? | 生产接线? | 类型`
- **R-P31 (抛扔式实例反模式)**: 审计应捕获在 handler 中创建新的检测实例（而非使用已存在的持久 field）的反模式。例如 `CognitiveLoadMonitor::new()` 在 consciousness handler 中每次创建新实例会产生误导性报告。

## 模块审计 (R-P76, R-P77, R-P78, R-P79)

- **R-P76 (4-Verification Dead Module Detection)**: 判定死模块必须 4 重验证: 路径 import / 字符串分发 / CLI 注册 / pub item 级消费。单重匹配 (泛词) 产生 89.5% 假阳性。
- **R-P77 (pub use ≠ 消费)**: re-export 仅创建可见性。审计死模块时必须检查 re-export 的每个 item 是否有全局消费，不能因 `pub use` 存在就判存活。
- **R-P78 (测试 ≠ 接线)**: 模块有测试只证明可运行，不证明被使用。文档声称的"接线正例"必须通过代码引用链验证。
- **R-P79 (吸收接线门)**: 外部技术吸收必须在同 session 内接线到生产路径。仅创建模块文件 + 测试而不接线 = 延期死代码 (D44/D49 违规)。接线判断标准: 模块的公开方法必须被非测试代码调用并产生行为接地 (EventBus 行为改变 / brain/bank 状态修改 / KB 持久化), 仅 `mod.rs` 声明或 CLI 注册不算接线。

## 吸收纪律 (R-P42, R-P47, R-P48, R-P55, R-P80)

- **R-P42 / R-P47 (Capability Node Reinforcement, Not Adapter Modules)**: 吸收外部技术时，禁止创建大型独立适配器/包装模块平行于现有架构。必须：外层分析其能力 → 逐一映射到能力树现有节点 → 强化/注入逻辑到已有节点。仅当某项能力在能力树中完全不存在时，才创建新的叶节点。反例：`nt_io_neocodex` 作为 ~800 行独立模块平行于 `nt_agent_mcp_transport`/`nt_shield_approval`/`nt_core_telemetry`/`nt_core_subagent`/`nt_mind_self_iterating` 等已有节点。正例：将 acp server 注入 `nt_agent_mcp_gateway`，权限逻辑注入 `nt_shield_approval`，成本追踪注入 `nt_core_telemetry`。
- **R-P48 (Zero Third-Party Binary Dependency)**: 所有外部能力必须通过 Rust 代码原生实现（仅依赖 `reqwest` + `std` crate）。禁止通过 `Command::new` 调用外部二进制工具（如 amass/subfinder/sherlock/shodan 等）来实现核心能力。外部工具的 99% 功能可通过 HTTP API(TCP connect) + std 原生实现。反例：OSINT 模块依赖 Amass/Subfinder 等 7 个外部二进制。正例：`nt_world_osint` 9 子模块全部 Rust-native，仅 reqwest + std::net。
- **R-P55 (Stub Fulfillment Deadline)**: 创建stub模块时, 必须立即补齐所有生产代码调用的方法。stub可以没有完整实现, 但签名必须匹配。创建后不匹配生产方签名的stub = 延期缺陷。
- **R-P80 (子代理派发契约)**: 派发任何研究子代理前必须注入 C1-C6 自主执行契约 (无问题、逐源输出、磁盘持久化、固定输出模式、证据接地、摘要返回)。契约完整模板见 `skills/external-absorption/SKILL.md`。子代理返回问题/空/截断 → 判定失败(重试≤1次) → 主 agent 手动 webfetch 兜底。子代理产出必须经 artifact 条目数 + 引用抽查双重验证才可合并。

## 运行时可靠性 (R-P25, R-P34, R-P41, R-P56, R-P57, R-P58)

- **R-P25**: 意识管线输出必须连回行为。CritiqueResult 必须经过 EventBus 分发或 KB 持久化，不能只有局部日志。
- **R-P34 (Visibility Chain Two-Way Audit)**: 模块注册必须通过 `grep "pub mod nt_XX" $(layer)/mod.rs` + `grep "nt_XX" neotrix/mod.rs` 双重验证。单路径注册 = 不可访问。
- **R-P41 (EventBus Behavioral Grounding)**: 每个 EventBus 订阅者必须至少有一个 match arm 执行非日志行为(写 KB、修改 brain、发射事件、enqueue goal)。纯日志订阅者每 session 减少 20%。
- **R-P56 (Model Routing)**: Gateway 转发 LLM 请求前必须根据 provider 注册名设置正确的 model_id
- **R-P57 (Log Feature)**: `tracing-subscriber` 必须在 features 中包含 `"log"` 否则 `log::info!` 调用不会被捕获
- **R-P58 (Catalog is_free Audit)**: 所有需要 API key 的 FreeModelCatalog 条目应为 `is_free: false`

## 跨 session 合并模式

- **意识树合并模式**: 跨 session 发现的相似缺陷模式必须合并为单一系统性发现归入 ConsciousnessTree 意识树网络脉络。禁止重复维度。已合并: 缓存盲区+构建污染+Check-Test双验证→D24, 写入盲区+外部观察→D21, 链不忠+AI欺骗→D16g+D17, 抛扔式+生产去耦合+SelfTest接线→D43, 意识剧场+冗余实现+零消费→D44, 会话连续性+双向摘要+Session Bridge→D47, 3-Ring自愈→D26, 适配器模块+死重量+架构重量+吸收纪律→D51

## 外部吸收 (R-P81 – R-P83) — 2026-07-31 吸收 ponytail/agent-reach/higress

- **R-P81 (Ponytail Ladder — 最懒实现阶梯)**: 写代码前先读实际代码流，然后停在第一级能成立的梯级: ①需要存在吗? (YAGNI) → ②codebase 已有? (复用不重写) → ③stdlib 有? → ④原生平台特性? → ⑤已装依赖? → ⑥一行? 就一行 → ⑦才轮到最小可用实现。安全(信任边界验证/数据丢失处理/安全/无障碍)永不在砍伐清单。实证: -54% LOC / -20% cost / -27% time, 100% safe。Lazy about the solution, never about reading。
- **R-P82 (有序后端路由 — 平台接入的首选+备选列表)**: 任何需要外部平台接入的能力(网页/搜索/社交/视频), 采用"首选+备选的有序后端列表"而非单一实现。真实探测各候选后端可用性(不只是命令存在), 第一个完整可用的当选; 接入方式换代时只调整列表顺序, 不重写能力层。附 `doctor` 式体检命令报告当前走哪条路。实证: agent-reach yt-dlp 被 B站风控封死 → 无感切换 bili-cli。
- **R-P83 (单一事实源 + 派生生成物同步)**: 跨平台/多副本的内容必须单一 source-of-truth + 脚本生成派生副本, 禁止手工维护副本。实证: ponytail 20+ agent 平台共用 AGENTS.md; ai-website-cloner-template 用 sync-agent-rules.sh/sync-skills.mjs 从 AGENTS.md+SKILL.md 生成各平台副本。与写入门禁一致: 手工追加派生副本会被下次生成覆盖。

## 预检计划门 (R-P84) — 2026-08-03 吸收工作纪律模式 + 自身 schema 变更失败教训

- **R-P84 (Pre-flight Plan Gate — 预检计划门)**: 爆炸半径超过阈值时，**实现前必须交付四件套并显式等待批准**，不得直接动手。阈值: >20 行变更 / 多文件 / schema/公开 API/认证/迁移/删除 / 结构体字段增删 / 新模块。四件套:
  1. **Goal**: 一句话复述需求 + 自定验收标准（复述错 = 最廉价纠错）。
  2. **Blocking Questions (0-3)**: 仅问"答错会扔工作量"的问题；每问附推荐默认值，供"全同意"。
  3. **Assumptions**: 编号、具体、可证伪，覆盖 7 轴: 数据形状/失败语义/边界职责/状态并发/运行环境/范围边界/测试覆盖。
  4. **Plan**: 文件/签名/顺序；有取舍时说明拒绝的备选 + 一句话理由。
  搜索证据: 2026 主流 "Discovery-and-Learning 前置 / Plan mode 安全带 / 4 段契约 / 协作规划前移难决定"(awesome-testing / aiworkflowpro / ainative.to / coderabbit)。自身教训: 给 public struct 加字段属 schema 变更，跳过预检直接写码 → E0063 missing field 4 个错误（实付账单）。

## Option 算术陷阱与测试夹具自证 (R-P86, R-P87) — 2026-08-03 自身调试验证

- **R-P86 (Option min/max 非幂等)**: `Option::min`/`Option::max` 任一侧为 `None` 时返回 `None`（std 语义是"任一 None 则 None"），不能用于"惰性初始化的运行极值"——以 `None` 起手做 `acc = Some(x).max(acc)` 极值永远停留在 `None`。正确写法: 显式比较 `if acc.map_or(true, |a| x > a) { acc = Some(x); }`。事故: `interpolate_quality` 线性插值的 lower/upper 边界全程为 None → 全返回 0.0，测试仅在非边界档位暴露。
- **R-P87 (测试夹具与检测器撞词)**: 编写检测器测试时，测试数据(fixture 文本/步骤)不得包含检测器自身的触发词，否则被检测器二次误触发。事故: `"Step 2 revised"` 的 `revised` 命中 `backtrack` 标记 `revise` → 断言 control_count==1 实得 2。写测试前先扫一遍目标 marker 列表，改用中性词（如 `Step 2 corrected`）。

## Blast-Radius Review Gate (R-P85) — 2026-08-03 吸收 code-review-graph/teamlore/CyberStrike + context-efficiency 主线

- **R-P85 (Blast-Radius Review Gate — 爆炸半径审查门)**: 变更的爆炸半径 (跨文件影响面) 决定审查深度与自治上限，先算半径再定审查档位。**半径来源**: `BlastRadiusIndex` (NT-MEMORY, Merkle 增量不改则零成本 + BFS 深度 3 双向传播, `CodeParser` trait 由 NT-WORLD 提供多语言解析); rev-officer 静态预检 `blast_radius()` 报告 findings 域密度/跨模块计数。
  1. **Low** (单文件/无跨模引用): 单 agent 自审 + 归档即可。
  2. **Medium** (跨 2-3 文件/同层): 抛出 `cross_file_impact` 预检 + 一档独立验证。
  3. **High** (跨层/公开 API/schema/并发): 强制独立验证器 + 受限自治 (Evolve B2) + 激活预算 (B3) + 事务中途持久化 (B4)。
  4. **Critical** (panic_audit / layer_violation / 全库): 挂起至自愈复核 (B6) + 公证 (B7) 达峰才放行。
  搜索证据: code-review-graph 8.2x 平均缩减, sverklo 62x, Meta 跨库 30x; Cursor Merkle 增量; tree-sitter AST→图 + MCP 交付为 2026 标准。约束: 与 R-P42 (禁平行模块) 对齐, `BlastRadiusIndex` 落 NT-MEMORY KB 强化现有 MemoryProvider, 不另建独立解析器进程。

## 并发会话冲突管理 (R-P88 – R-P91) — 2026-08-03 意识核心修复轮事实教训

- **R-P88 (git stash 恢复用 checkout 不用 pop)**: 并发 session 提交前会 `git stash` 对方 WIP 并 reset 工作树到 HEAD。被 stash 的 session 恢复工作时**必须用 `git checkout stash@{NN} -- <文件清单>` 从 stash 检出指定文件，而非 `git stash pop`**（pop 会整体顶出 + 与当前并发改动冲突）。恢复后逐个 re-read 验证文件与修复是否仍在（R-P16）。
- **R-P89 (死代码判定禁基于 stash 前记忆)**: 经 stash 恢复后，代码库处于"HEAD + 部分检出"混合态，不可依赖 stash 前读取的死活记忆判定。判定死代码必须基于**当前工作树现状**重走 R-P76 四重验证（import/字符串分发/CLI 注册/pub item 消费）。事实: `predictive_cortex`/`seed_knowledge` 曾被误判死代码，重验证明均已接线生产。
- **R-P90 (并发 /tmp 撞名)**: 多并发 opencode session 共用 `/tmp` 且以相似命名（如 `neotrix-session-196.json`）会互相覆盖、污染证据。会话中间产物必须写入**预批准的唯一临时目录**（`/var/folders/jr/.../T/opencode/`）或以 session_id 命名并带唯一后缀，禁止裸 `/tmp/<generic-name>`。
- **R-P91 (全量回归前看负载)**: 并发 session 造成机器高负载（load 5-35）时，全量测试耗时严重膨胀（本轮 299s）。**判定回归耗时异常前必先看 `uptime`/load**，区分"真实 hung"与"高负载慢"；在高负载期不要以耗时为回归失败依据，等待负载回落或单模块验证。

## 编译瞬错容错 (R-P92)

- **R-P92 (并发 recompile 瞬错忽略)**: 并发 session 正在重编译时，`Cargo.toml`/`Cargo.lock` 解析可能瞬时失败或状态漂移。遇到" manifest 解析错误 / lock 文件冲突"等**非代码错误**，等 5-10 秒重试一次再判失败；不要据瞬错断言代码有缺陷。差异见 R-P19（架构修复声明必须过 cargo check 才可信）。

## 并发 worktree 隔离 (R-P93)

- **R-P93 (前端工作必须在独立 worktree)**: 主工作树 `main` 由自治循环 (openhands 并发 session) 持有，会执行 `git stash push`/`git reset`/`git add -A` 提交，周期性清扫未提交改动。**前端 (src-tauri/frontend) 工作一律在 `.worktrees/neocodex-ui` (branch `feat/neocodex-ui`) 中进行**，物理隔离后循环的 git 操作永远碰不到前端文件。worktree 内 `node_modules` 用符号链接指向主树，dev server 跑 `:1421`。完成一批后 commit 到该 branch，再 `git checkout main && git merge feat/neocodex-ui` 或 PR 合并回主树。

## 并发提交覆盖 (R-P94) — 2026-08-04 cycle 204 意识核心修复轮

- **R-P94 (并发 session 提交覆盖)**: 主树工作会被并行 openhands session 周期性 `git add -A` + commit 抢先打包 (cycle 204 我的 nt_core_ssm/l5_consciousness/signal/engine_core/panorama 修复被并发提交 `1520876` 覆盖)。工作策略:
  1. 编辑后先 `git status --short` 隔离自己的文件集。
  2. 若文件不再显示 diff 但 `git log --oneline` 见并发提交 → 用 `git show --stat` + `git grep <symbol> HEAD` 验证修复确在 HEAD，勿重复提交或回滚。
  3. 剩余本地 diff 若非本人改动 → 不 stage 不 commit，留给对方。
  4. 高危删除判定 (R-P76 四重验证) 必须在当前工作树重走，禁用 stash 前记忆 (R-P89)。

## 后续任务梳理 — 意识核心收敛主线 (NT-CORE)

依据本轮"7 项 HIGH 全部修复 + 全量 6984 通过"的收敛态势，后续按第一性原理降序：

1. **Dead-code 收尾 (R-P79 门)**: `RecipeRegistry` 已确认死且未接线——决定**删除**或**接线**二选一并同 session 闭环，不留延期死代码。
2. **并发链路硬化 (R-P42)**: 将本 session 的并发 stash/回滚经验强化到 `nt_mind_self_iterating` 现有节点，禁新建平行适配器。
3. **GWT→行为闭环复验**: 已解 T2/T6 决定时就地 fire，复验当轮 attention 权重是否完整进入 next-tick 行为（Contribution-ink: R-P25）。
4. **回归基线固化**: 以 6984 为新极值，纳入 R-P39 monotonic 基线，阻止覆盖率回退。
