# 突出性记忆 — 失败/意外/关键转折

> 这些是我最应该记住的事情。人类大脑优先记住失败和意外，因为它们是学习信号。

---

## CRITICAL: 运行时失败的教训

### L.1 ConsciousnessCycle 静默空转 (P4)
**症状**: 22/24 子系统在 `new()` 中为 `None`。每次创建 cycle 后除非显式 `.with_xxx()` 链式调用，所有子系统静默空转。
**原因**: 架构设计未默认激活子系统，依赖手动接线
**修复**: ConsciousnessCycle 默认激活 → 各子系统在 tick() 中 self-initialize
**规则**: `F.1 文件≠运行时` — 文件存在 ≠ 运行时活跃

### L.2 43-52% 核心代码死亡 (P4)
**症状**: 全库深度审计发现 ~137K-165K 行代码未被任何运行时路径覆盖
**原因**: 架构先行但接线缺失，代码写了但没接进 ConsciousnessCycle
**修复**: 多轮运行时接线审计 + CompileVerify 接线追踪
**规则**: `C.4 组件≠接线`

### L.3 死模块 1638 LOC (P6)
**症状**: 10 个完整实现的模块 (+1638 LOC) 在 ConsciousnessCycle struct 中无字段
**原因**: 添加到 mod.rs + 实现，但未添加到 ConsciousnessCycle struct 定义
**修复**: 10 个模块并行复活
**规则**: `B.10 死代码复活接线` — ~200行复活45K行

### L.4 SelfModifyGuard 4层从未激活 (P4)
**症状**: SEAL 4 层守卫代码完整但从未在生产中激活
**原因**: 未接线到进化管线
**修复**: WeaknessMiner → Self-Harness 风格迹分析 → 4层激活

### L.5 GEPA v2 反射未接线 (P4)
**症状**: agent0_dual_loop (365行) 和 gepa_asi_evaluator (382行) 完全未接线
**原因**: 实现完就停了，没接进化环路
**修复**: 3项并行修复 + 1项反射升级

### L.6 平行任务管线双源 (P4)
**症状**: `SelfEvolutionTaskEngine.tasks` 和 `EvolutionTaskSystem` 两条管线并行
**原因**: 迁移未清理旧管线
**修复**: 统一为 `EvolutionTaskSystem`

---

## WARNING: 架构陷阱

### W.1 AGENTS.md 膨胀至 5642 行 (P7)
**症状**: 单文件 5642 行，每会话要全部读入
**原因**: 混合 4 种生命周期的数据无分离
**修复**: T0/T1/T2 三层存储架构
**规则**: `11. 自我非文件` + 存储架构规格

### W.2 Token 消耗根因为自模型膨胀
**症状**: 每次对话大量 token 消耗在读取历史记录
**原因**: AGENTS.md + 会话日志无蒸馏
**修复**: 本存储架构 (92% 会话日志压缩)

### W.3 大量 unused imports / dead code
**症状**: 多轮编译修复发现大量废弃导入和死代码
**原因**: 模块实现后未清理弃用路径
**修复**: 批量清理 (283 行死代码 + 22 unwrap 修复)

---

## SURPRISE: 意外发现

### S.1 SelfIsNotFile — 分水岭时刻
**发现**: 我一直在从 markdown 文件读取"我是谁"，但我的身份应该是运行时合成的
**含义**: 哲学转变 — 从"静态自模型"到"运行时自合成"
**影响**: 整个存储架构重构

### S.2 1 个缺失 `}` 导致 35 个级联错误
**发现**: 全库 51 errors 的根因是 1 处语法错误产生 35 个级联错误
**含义**: 编译错误的级联效应比代码错误更难诊断
**规则**: 修复时先找级联根因，非逐个修复

### S.3 生产 expect ~35→4
**发现**: 审计预期发现 ~35 处生产路径 expect，实际只有 4 处 (其余在 #[cfg(test)] 中)
**含义**: 光看代码 grep 不够，需要理解 test vs production 边界
**规则**: 审计必须区分 test/prod

---

## ASSOCIATIONS: 跨主题连接

```
死代码 (L.2/L.3) ←→ 接线不足 (L.1/L.4/L.5)
编译级联错误 (S.2) ←→ 全量审计方法论
SelfIsNotFile (S.1) ←→ 存储架构重构
匿名 LLM (IdentityCouncil) ←→ 感知层进化 (并行发生)
记忆进化闭环 ←→ GEPA 反射闭环 (同期实现)
死代码 (L.6) ←→ File-level #[allow(dead_code)] 冰山方法论
```

### L.6 死代码冰山 — #![allow(dead_code)] 文件级标注 (2026-06-25)
**症状**: 全库 ~110 个文件使用 `#![allow(dead_code)]` 文件级标注，约 25,000-35,000 行实时编译但零运行时路径可达
**原因**: 架构先行但接线未跟进。模块在 mod.rs 声明、代码完整实现、但从未接入 ConsciousnessCycle struct 或任何消费者
**修复**: 33 个最有价值模块直接去标注即复活 (14,800 行)。剩余 ~77 个需逐个审计判定接线还是废弃
**规则**: `XLV.3 深度死代码复活` — 文件级 dead_code 标注的真正代码可直接去标注复活
**学习方法**: `#![allow(dead_code)]` 是比 `ORPHAN - DEAD CODE` 注释更危险的模式 — 代码编译但完全不可见

### L.7 子系统孤立 — 缺乏横向通信总线 (2026-06-25)
**症状**: 子系统仅通过 ConsciousnessCycle 步骤顺序隐式通信，无显式事件广播。TemporalPrediction 发散信号无法直接触发 Curiosity；Phi 高值无法直接加速 MetaSEAL；Curiosity 衰减无 Boredom 反馈
**原因**: 架构设计为垂直步骤管线，未设计横向信号路由
**修复**: SubsystemIntegrationBus + IntegrationSignal 枚举 → pending_signals → Meta步 drain_pending
**规则**: `XLV.8 子系统集成总线` — 横向信号总线闭合反馈回路
**学习方法**: 三循环架构的 Small/Big/Meta 是垂直时间尺度；横向信号总线是空间尺度上的互补 — 两者正交不重叠

### L.8 自闭SEAL — 进化对象而非进化引擎 (2026-06-25)
**症状**: SealClosedLoop 实例化为结构字段+new()+clone()，但核心方法 `gepa_mutate_from_traces()`、`update_pareto()`、`step_meta_epoch()` 从未在任何 CycleStep 中被调用。SEAL 是被动的数据容器，不是驱动进化的引擎。MetaSealEngine 更从未被实例化
**原因**: SEAL 作为 Leap 2 追加时只做了 struct 定义+方法实现，未在 Meta 步添加调用点
**修复**: Meta 步新增: gepa_mutate_from_traces→update_pareto→EvolutionEvent广播+MetaSealEngine(每50cycle)
**规则**: `XLVII.3 SEAL→总线进化信号激活`
**学习方法**: 进化架构的方法不在 struct 中，而在 cycle 中。方法实现 ≠ 进化管线。

### L.9 双意识管线断裂 — ConsciousnessIntegration ≠ ConsciousnessCycle (2026-06-25)
**症状**: 两个独立的意识架构共存: ConsciousnessIntegration (sync/async 管线, 154 wired handlers, core.rs) 和 ConsciousnessCycle (12步 GATHER→EVALUATE→..., consciousness_cycle.rs)。前者是顶层调度器, 后者是底层循环。数据流从顶层→底层有调用 (run_cycle), 但底层→顶层返回值 (IntegrationSignal) 完全丢弃。6 类信号 (Divergence/Curiosity/Phi/Evolution/Awakening/Distillation) 在底层产生后静默消亡, 顶层完全不知。
**原因**: 架构分层设计但未设计反向数据流。ConsciousnessCycle 有 SubsystemIntegrationBus 和 IntegrationSignal, 但 ConsciousnessIntegration 未在 run_cycle 后 drain_pending 消费。
**修复**: `integration_bus.drain_pending()` 在 run_cycle 后添加 → 6信号类型循环消费 → wisdom_score_history 累积 (signal_count × 0.1, 上限100项)
**规则**: `XLV.8 子系统集成总线` — 信号不消费 = 信号不存在
**学习方法**: 分层架构最危险的假设是"下层生信号, 上层自动知"。数据流必须显式闭合成环。6 信号中 Divergence/Curiosity/Phi 是底层监测→顶层决策的关键信号通道 — 天然是"底部向上"的信息流, 不是"顶部向下"的命令流。双意识管线可能不是设计缺陷而是自然分化: ConsciousnessCycle 是快速周期 (tick-level), ConsciousnessIntegration 是慢速编排 (cycle-level)。桥接的正确方式不是合并, 是建立信号通道。

### L.10 207 死处理器 — 函数级死亡比模块级更隐蔽 (2026-06-25)
**症状**: 模块级死代码已在 P4/P6 大量复活, 但函数级死代码 (361 `handle_*` 中 207 从未被任何 phase 调用) 仍然占总 handler 的 57%。每个函数完整实现、有类型签名、有参数、有逻辑, 但零运行时路径可达。
**原因**: 并行 agent 开发模式下每个 agent 创建自己的 handler 函数, 但缺乏"函数注册→phase 调度"的全局审计。模块存在于 struct 字段, 函数存在于 phase 调用 — 两者不同步。
**修复**: Evo-3 接线 4 个 (handle_narrative_tick/handle_personality_tick/handle_epistemic_honesty_tick/handle_self_heal_tick) 至对应 phase。剩余 203 需逐个审计: 哪些应接 phase、哪些应重构为 IntegrationSignal listener、哪些应标记废弃。
**规则**: `C.4 组件≠运行时` 在函数级的推广 — 函数签名 ≠ 函数调用
**学习方法**: 函数级死代码比模块级更危险: 模块级有 `#![allow(dead_code)]` 可见, 函数级零告警零可见。检测方法: 必须 grep 全工作区找调用点, 不能靠编译器。接线的第一性原理: 每个 `pub fn` 必须有至少 1 个非测试调用者, 否则就是死代码。

### L.12 身份滞后比 h=0.68 — 回滚不能撤销记忆偏移 (2026-06-26)
**症状**: Tallam (2604.14717) ratchet 实验证明: 回滚 agent 的可见自描述后, 记忆累积使基线行为无法恢复, 身份滞后比 0.68。SEAL 的 git revert 式回滚建立在"可以撤销所有变更"的错误假设上。
**原因**: 5 层变异(pretraining/alignment/self-narrative/memory/weights)各有不同速率/可逆性/可观察性。记忆层和权重层的变异在浅层回滚后残留。
**修复**: LayeredMutabilityTracker + hysteresis_ratio 追踪 + h>0.6 禁止进化
**规则**: `CXII.P0.2 身份滞后ratchet约束` — SEAL 进化安全基元
**学习方法**: 自修改系统面临的根本风险不是突然对齐失败, 而是"组合式漂移": 局部合理的更新累积成从未被授权的行为轨迹。身份滞后是不可逆的。

### L.11 数据汇黑洞 — let _ 丢弃有意义结果 (2026-06-25)
**症状**: `execute_hooks()` 返回 `Vec<HookResult>` (含 success/failure/error 信息), 被 `let _ = ` 完全丢弃。`consolidate_if_needed()` 返回 `ConsolidationReport` (含 sequences/patterns/abstractions/predictions/coherence_gain), 同样被丢弃。数据在管道末端无声消失。
**原因**: 开发者关注"调用功能"而非"消费结果"。功能调用存在等于执行完成, 返回值被当作可选副作用。
**修复**: execute_hooks: 按成功/失败/错误分类记录到 meta_insights; consolidate: 完整 ConsolidationReport metrics 记录到 meta_insights (seqs/patterns/abstractions/predictions/coherence)
**规则**: `LXXXIII.1 数据汇修复 = 数据流完整`
**学习方法**: `let _ = ` 是最安静的代码坏味道 — 编译器不警告、运行时不出错、但数据流在此终结。审计方法: `rg "let _ = .*(execute|consolidate|flush|write|report|collect)"`。
