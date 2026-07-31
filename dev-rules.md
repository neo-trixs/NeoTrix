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
- **R-P79 (吸收接线门)**: 外部技术吸收必须在同 session 内接线到生产路径。仅创建模块文件 + 测试而不接线 = 延期死代码 (D44/D49 违规)。

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
